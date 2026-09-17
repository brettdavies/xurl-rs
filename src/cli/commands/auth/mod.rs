/// Auth subcommand handlers — OAuth2, OAuth1, Bearer, app management.
use std::io::Write;

use serde::Serialize;

use super::{Gate, gate_destructive};
use crate::auth::Auth;
use crate::cli::AuthCommands;
use crate::cli::failure::CommandResult;
use crate::cli::output::OutputConfig;
use crate::config::{self, ResolveSource};
use crate::error::Error;
use crate::store::TokenStore;

mod apps;
mod session;
mod signin;
mod types;

use types::BearerSource;
pub(crate) use types::{RedirectUriGetResponse, RedirectUriSetResponse};

/// Bundle of global flags relevant to auth subcommands.
///
/// Mirrors the parent module's `GlobalFlags`, scoped to the subset auth needs.
/// `verbose` is reserved for future auth handlers that surface verbose request
/// tracing (e.g. the manual OAuth2 step exchange).
#[derive(Debug, Clone, Copy)]
pub(super) struct AuthGlobalFlags {
    pub(super) no_interactive: bool,
    #[allow(dead_code)]
    pub(super) verbose: bool,
    pub(super) dry_run: bool,
    pub(super) quiet: bool,
    pub(super) app_explicit: bool,
}

/// Per-app status/list entry rendered under `--output json`.
///
/// Built field-by-field from named accessors on `App` and `TokenStore`.
/// `From<&App>` and `Serialize`-on-`App` are forbidden: `App` holds
/// `client_secret`, OAuth2/OAuth1 tokens, and the bearer string, so
/// wholesale-forwarding would leak credentials.
///
/// A secret-exclusion test in `tests/cli_tests.rs` asserts no credential
/// field name or value appears in the rendered JSON.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub(crate) struct AppStatusEntry {
    /// App name as stored in `~/.xurl`.
    name: String,
    /// First 8 characters of `client_id` (never the full value, never `client_secret`).
    client_id_hint: String,
    /// Effective `OAuth2` redirect URI from the resolver.
    redirect_uri: String,
    /// Precedence layer that produced [`Self::redirect_uri`].
    redirect_uri_source: ResolveSource,
    /// Stored per-app redirect URI; only present when the env var overrides it.
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_uri_stored: Option<String>,
    /// `OAuth2` usernames present in the app (no tokens, just names).
    oauth2_users: Vec<String>,
    /// Whether the app has `OAuth1` credentials present (presence only).
    oauth1: bool,
    /// Whether a bearer credential is available for the app: stored on it,
    /// or supplied by `XURL_BEARER_TOKEN` for the active app (presence only).
    bearer: bool,
    /// Where the bearer comes from; omitted when `bearer` is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    bearer_source: Option<BearerSource>,
    /// Whether this app is the default.
    default: bool,
    /// Whether the app has an unnamed (`/me`-failed salvage) `OAuth2` token.
    ///
    /// Omitted from JSON output when `false` per KTD9 — a `true` value signals
    /// that `App.unnamed_oauth2_token.is_some()`.
    #[serde(skip_serializing_if = "is_false")]
    oauth2_unnamed: bool,
}

/// Helper for `#[serde(skip_serializing_if)]` on `bool` fields that default false.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(b: &bool) -> bool {
    !*b
}

/// State every auth verb handler shares: the credential store behind [`Auth`],
/// the global flags, the output configuration, and the two output streams.
struct AuthCtx<'a> {
    auth: &'a mut Auth,
    flags: AuthGlobalFlags,
    out: &'a OutputConfig,
    stdout: &'a mut dyn Write,
    stderr: &'a mut dyn Write,
}

pub(super) fn run_auth_command(
    cmd: AuthCommands,
    mut auth: Auth,
    flags: AuthGlobalFlags,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> CommandResult<()> {
    let ctx = AuthCtx {
        auth: &mut auth,
        flags,
        out,
        stdout,
        stderr,
    };
    match cmd {
        AuthCommands::Oauth2 {
            no_browser,
            step,
            auth_url,
            username,
        } => signin::oauth2(
            signin::Oauth2Args {
                no_browser,
                step,
                auth_url,
                username,
            },
            ctx,
        ),
        AuthCommands::Oauth1 {
            consumer_key,
            consumer_secret,
            access_token,
            token_secret,
        } => signin::oauth1(
            signin::Oauth1Args {
                consumer_key,
                consumer_secret,
                access_token,
                token_secret,
            },
            ctx,
        ),
        AuthCommands::App { bearer_token } => signin::bearer(bearer_token, ctx),
        AuthCommands::Status => session::status(&auth, out, stdout),
        AuthCommands::Clear {
            all,
            oauth1,
            oauth2_username,
            bearer,
            force,
        } => session::clear(
            session::ClearArgs {
                all,
                oauth1,
                oauth2_username,
                bearer,
                force,
            },
            ctx,
        ),
        AuthCommands::Apps { command } => apps::run_app_command(command, ctx),
        AuthCommands::Default { app_name, username } => {
            session::set_default(session::SetDefaultArgs { app_name, username }, ctx)
        }
    }
}

/// Truncates a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        match s.char_indices().nth(max_len) {
            Some((byte_idx, _)) => &s[..byte_idx],
            None => s,
        }
    }
}

/// Renders the zero-app state for `auth status` and `auth apps list`.
///
/// Text names the command that fixes it, and reports the environment bearer
/// when one is set, since that alone can already drive app-only calls.
/// Structured output carries an empty `apps` array inside the same success
/// envelope, so a caller iterates it without a zero-app special case.
fn print_no_apps_registered(
    auth: &Auth,
    out: &OutputConfig,
    stdout: &mut dyn Write,
) -> CommandResult<()> {
    // An empty `apps` map means two different things. Saying "nothing is
    // registered" about a file the loader could not read would send the
    // reader to `apps add`, which then refuses.
    if auth.token_store.load_failed() {
        return Err(Error::token_store(format!(
            "cannot read the token store at {}: it exists but could not be loaded; inspect or move it",
            auth.token_store.file_path.display()
        ))
        .into());
    }
    if out.format.is_structured() {
        out.print_success(stdout, &serde_json::json!({ "apps": [] }));
        return Ok(());
    }
    out.print_message(
        stdout,
        "No apps registered. Run: xr auth apps add NAME --client-id ID --client-secret SECRET",
    );
    if auth.env_bearer_token_present() {
        out.print_message(
            stdout,
            &format!("bearer: \u{2713}{}", BearerSource::Env.as_text_label()),
        );
    }
    Ok(())
}

/// Name of the app `XURL_BEARER_TOKEN` applies to, when it is set.
///
/// The env bearer wins the bearer precedence for whichever app is active:
/// the `--app` selection when given, otherwise the store's default.
fn env_bearer_app(auth: &Auth) -> Option<String> {
    if !auth.env_bearer_token_present() {
        return None;
    }
    Some(
        auth.token_store
            .get_active_app_name(auth.app_name())
            .to_string(),
    )
}

/// Builds the typed JSON intermediate for `auth status` and `auth apps list`.
///
/// Constructs each `AppStatusEntry` field-by-field from named accessors per
/// R23 + KTD11; no `From<&App>` and no `Serialize`-on-`App`. The caller
/// supplies the `REDIRECT_URI` value that drives the resolver and, when
/// `XURL_BEARER_TOKEN` is set, the name of the app the env bearer applies to.
fn build_app_status_entries(
    ts: &TokenStore,
    apps: &[String],
    default_app: &str,
    redirect_uri_override: Option<&str>,
    env_bearer_app: Option<&str>,
) -> Vec<AppStatusEntry> {
    let env = redirect_uri_override.map(str::to_string);
    apps.iter()
        .filter_map(|name| {
            let app = ts.get_app(name)?;
            let bearer_source = if env_bearer_app == Some(name.as_str()) {
                Some(BearerSource::Env)
            } else if app.bearer_token.is_some() {
                Some(BearerSource::Store)
            } else {
                None
            };
            let stored = ts.get_app_redirect_uri(name).map(str::to_string);
            let resolved = config::resolve_redirect_uri_from(env.clone(), stored.as_deref());
            let stored_field = if resolved.source.is_env_var() && stored.is_some() {
                stored
            } else {
                None
            };
            Some(AppStatusEntry {
                name: name.clone(),
                client_id_hint: truncate(&app.client_id, 8).to_string(),
                redirect_uri: resolved.uri,
                redirect_uri_source: resolved.source,
                redirect_uri_stored: stored_field,
                oauth2_users: ts.get_oauth2_usernames_for_app(name),
                oauth1: app.oauth1_token.is_some(),
                bearer: bearer_source.is_some(),
                bearer_source,
                default: name == default_app,
                oauth2_unnamed: app.unnamed_oauth2_token.is_some(),
            })
        })
        .collect()
}
