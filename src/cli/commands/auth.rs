/// Auth subcommand handlers — OAuth2, OAuth1, Bearer, app management.
use std::io::Write;

use serde::Serialize;
use serde_json::json;

use super::{Gate, gate_destructive};
use crate::auth::Auth;
use crate::cli::{AppCommands, AuthCommands, RedirectUriCommands};
use crate::config::{self, ResolveSource};
use crate::error::{EXIT_GENERAL_ERROR, Result, XurlError};
use crate::output::{OutputConfig, OutputFormat};
use crate::store::TokenStore;

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
#[derive(Debug, Clone, Serialize)]
struct AppStatusEntry {
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
    /// Whether the app has a bearer token present (presence only).
    bearer: bool,
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

#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub(super) fn run_auth_command(
    cmd: AuthCommands,
    mut auth: Auth,
    flags: AuthGlobalFlags,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<()> {
    let AuthGlobalFlags {
        no_interactive,
        verbose: _,
        dry_run,
        quiet,
        app_explicit,
    } = flags;
    match cmd {
        AuthCommands::Oauth2 {
            no_browser,
            step,
            auth_url,
            username,
        } => {
            if dry_run {
                let ctx = json!({
                    "command": "auth-oauth2",
                    "no_browser": no_browser,
                    "step": step,
                    "username": username,
                });
                out.print_dry_run(stdout, true, 0, &ctx);
                return Ok(());
            }
            let username_arg = username.as_deref().unwrap_or("");
            // R13/KTD4: credential-less-default warning. Fires when the user
            // did not pass `--app`, the default app has no `client_id`, and at
            // least one other registered app does. Routed via
            // `OutputConfig::info` so `--quiet` and `--output json` suppress it.
            if !app_explicit && let Some(msg) = credential_less_default_warning(&auth.token_store) {
                out.info(stderr, &msg);
            }
            if !no_browser {
                // Standard interactive flow
                auth.oauth2_flow(username_arg, out, stdout)?;
                out.print_message(stdout, "\x1b[32mOAuth2 authentication successful!\x1b[0m");
            } else {
                let pending_path = crate::auth::pending::default_pending_path()?;
                match step {
                    Some(1) => {
                        if auth_url.is_some() {
                            return Err(crate::error::XurlError::auth(
                                "--auth-url is only used with --step 2, not --step 1",
                            ));
                        }
                        let url = auth.remote_oauth2_step1(&pending_path)?;
                        match out.format {
                            crate::output::OutputFormat::Json
                            | crate::output::OutputFormat::Jsonl => {
                                let envelope = serde_json::json!({
                                    "auth_url": url,
                                    "instructions": "Open the URL in a browser, authorize, then copy the redirect URL and run step 2"
                                });
                                out.print_response(stdout, &envelope);
                            }
                            crate::output::OutputFormat::Text => {
                                out.print_message(
                                    stdout,
                                    "Open this URL in a browser on a machine with a display:",
                                );
                                out.print_message(stdout, "");
                                out.print_message(stdout, &format!("  {url}"));
                                out.print_message(stdout, "");
                                out.print_message(
                                    stdout,
                                    "After authorizing, copy the redirect URL from your browser's address bar",
                                );
                                out.print_message(
                                    stdout,
                                    "(it will show an error page — that's expected).",
                                );
                                out.print_message(stdout, "");
                                out.print_message(stdout, "Then run:");
                                out.print_message(
                                    stdout,
                                    "  echo '<redirect-url>' | xr auth oauth2 --no-browser --step 2 --auth-url -",
                                );
                            }
                        }
                    }
                    Some(2) => {
                        let url_value = auth_url.ok_or_else(|| {
                            crate::error::XurlError::auth(
                                "--auth-url is required for step 2. Pass the redirect URL from your browser, \
                                 or use --auth-url - to read from stdin",
                            )
                        })?;

                        let redirect_url = if url_value == "-" {
                            let mut line = String::new();
                            std::io::stdin().read_line(&mut line).map_err(|e| {
                                crate::error::XurlError::auth_with_cause(
                                    "Failed to read redirect URL from stdin",
                                    &e,
                                )
                            })?;
                            let trimmed = line.trim().to_string();
                            if trimmed.is_empty() {
                                return Err(crate::error::XurlError::auth(
                                    "No redirect URL provided on stdin. \
                                     Pipe the URL or paste it and press Enter",
                                ));
                            }
                            trimmed
                        } else {
                            url_value
                        };

                        auth.remote_oauth2_step2(&redirect_url, username_arg, &pending_path)?;
                        out.print_message(
                            stdout,
                            "\x1b[32mOAuth2 authentication successful!\x1b[0m",
                        );
                    }
                    None => {
                        return Err(crate::error::XurlError::auth(
                            "--no-browser requires --step 1 or --step 2",
                        ));
                    }
                    _ => unreachable!("clap value_parser restricts to 1..=2"),
                }
            }
        }
        AuthCommands::Oauth1 {
            consumer_key,
            consumer_secret,
            access_token,
            token_secret,
        } => {
            if dry_run {
                let ctx = json!({"command": "auth-oauth1"});
                out.print_dry_run(stdout, true, 0, &ctx);
                return Ok(());
            }
            auth.token_store.save_oauth1_tokens(
                &access_token,
                &token_secret,
                &consumer_key,
                &consumer_secret,
            )?;
            out.print_message(
                stdout,
                "\x1b[32mOAuth1 credentials saved successfully!\x1b[0m",
            );
        }
        AuthCommands::App { bearer_token } => {
            if dry_run {
                let ctx = json!({"command": "auth-app"});
                out.print_dry_run(stdout, true, 0, &ctx);
                return Ok(());
            }
            auth.token_store.save_bearer_token(&bearer_token)?;
            out.print_message(stdout, "\x1b[32mApp authentication successful!\x1b[0m");
        }
        AuthCommands::Status => {
            // Read through the runner-constructed store so tempdir-based
            // CLI tests observe the same `~/.xurl` the runner saw (KTD7).
            let ts = &auth.token_store;
            let apps = ts.list_apps();
            let default_app = ts.get_default_app();

            if apps.is_empty() {
                out.print_message(
                    stdout,
                    "No apps registered. Use 'xr auth apps add' to register one.",
                );
                return Ok(());
            }

            let entries = build_app_status_entries(ts, &apps, default_app);

            match out.format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let value = serde_json::to_value(&entries)?;
                    out.print_response(stdout, &value);
                }
                OutputFormat::Text => {
                    for (i, (name, entry)) in apps.iter().zip(entries.iter()).enumerate() {
                        let Some(app) = ts.get_app(name) else {
                            continue;
                        };
                        let marker = if name == default_app { "\u{25b8}" } else { " " };
                        let client_hint = if app.client_id.is_empty() {
                            "(no credentials)".to_string()
                        } else {
                            format!("client_id: {}...", entry.client_id_hint)
                        };
                        out.print_message(stdout, &format!("{marker} {name}  [{client_hint}]"));

                        // R19 + R24: surface the effective redirect URI + source.
                        out.print_message(
                            stdout,
                            &format!(
                                "      redirect_uri: {} [{}]",
                                entry.redirect_uri,
                                entry.redirect_uri_source.as_text_label()
                            ),
                        );
                        if let Some(stored) = entry.redirect_uri_stored.as_deref() {
                            out.print_message(
                                stdout,
                                &format!("      stored_redirect_uri: {stored}"),
                            );
                        }

                        if entry.oauth2_users.is_empty() && !entry.oauth2_unnamed {
                            out.print_message(stdout, "      oauth2: (none)");
                        } else {
                            for u in &entry.oauth2_users {
                                if *u == app.default_user {
                                    out.print_message(stdout, &format!("    \u{25b8} oauth2: {u}"));
                                } else {
                                    out.print_message(stdout, &format!("      oauth2: {u}"));
                                }
                            }
                            // KTD8: render the unnamed (`/me`-failed salvage)
                            // slot after named users, labelled `(unknown user)`.
                            if entry.oauth2_unnamed {
                                out.print_message(stdout, "      oauth2: (unknown user)");
                            }
                        }

                        if entry.oauth1 {
                            out.print_message(stdout, "      oauth1: \u{2713}");
                        } else {
                            out.print_message(stdout, "      oauth1: \u{2013}");
                        }

                        if entry.bearer {
                            out.print_message(stdout, "      bearer: \u{2713}");
                        } else {
                            out.print_message(stdout, "      bearer: \u{2013}");
                        }

                        if i < apps.len() - 1 {
                            out.print_message(stdout, "");
                        }
                    }
                }
            }
        }
        AuthCommands::Clear {
            all,
            oauth1,
            oauth2_username,
            bearer,
            force,
        } => {
            let ctx = json!({
                "command": "auth-clear",
                "all": all,
                "oauth1": oauth1,
                "oauth2_username": oauth2_username,
                "bearer": bearer,
            });

            // Force/confirmation gate runs BEFORE dry-run so an unconfirmed
            // destructive op in interactive mode does not leak a dry-run
            // envelope. Dry-run still composes with --force.
            let target = if all {
                "all credentials"
            } else if oauth1 {
                "OAuth1 tokens"
            } else if oauth2_username.is_some() {
                "OAuth2 token"
            } else if bearer {
                "bearer token"
            } else {
                ""
            };
            if !target.is_empty() {
                match gate_destructive(force, no_interactive, quiet, &format!("Clear {target}?"))? {
                    Gate::Proceed => {}
                    Gate::Declined => return Ok(()),
                    Gate::ConfirmationRequired => {
                        out.print_confirmation_required(stderr, &ctx, EXIT_GENERAL_ERROR);
                        return Err(XurlError::EnvelopeAlreadyEmitted {
                            exit_code: EXIT_GENERAL_ERROR,
                        });
                    }
                }
            }

            if dry_run {
                out.print_dry_run(stdout, true, 0, &ctx);
                return Ok(());
            }

            if all {
                auth.token_store.clear_all()?;
                out.print_message(stdout, "All authentication cleared!");
            } else if oauth1 {
                auth.token_store.clear_oauth1_tokens()?;
                out.print_message(stdout, "OAuth1 tokens cleared!");
            } else if let Some(username) = oauth2_username {
                auth.token_store.clear_oauth2_token(&username)?;
                out.print_message(stdout, &format!("OAuth2 token cleared for {username}!"));
            } else if bearer {
                auth.token_store.clear_bearer_token()?;
                out.print_message(stdout, "Bearer token cleared!");
            } else {
                return Err(XurlError::validation(
                    "No authentication cleared! Use --all to clear all authentication.",
                ));
            }
        }
        AuthCommands::Apps { command } => {
            return run_app_command(
                command,
                &mut auth,
                AppGlobalFlags {
                    no_interactive,
                    dry_run,
                    quiet,
                },
                out,
                stdout,
                stderr,
            );
        }
        AuthCommands::Default { app_name, username } => {
            if dry_run {
                let ctx = json!({
                    "command": "auth-default",
                    "app_name": app_name,
                    "username": username,
                });
                out.print_dry_run(stdout, true, 0, &ctx);
                return Ok(());
            }
            if let Some(app_name) = app_name {
                auth.token_store.set_default_app(&app_name)?;
                out.print_message(
                    stdout,
                    &format!("\x1b[32mDefault app set to {app_name:?}\x1b[0m"),
                );
                if let Some(user) = username {
                    auth.token_store.set_default_user(&app_name, &user)?;
                    out.print_message(
                        stdout,
                        &format!("\x1b[32mDefault user set to {user:?}\x1b[0m"),
                    );
                }
            } else {
                // Interactive picker
                if no_interactive {
                    return Err(XurlError::auth(
                        "Interactive prompt required. Pass app name as argument: xr auth default <app-name>",
                    ));
                }

                let apps = auth.token_store.list_apps();
                if apps.is_empty() {
                    out.print_message(
                        stdout,
                        "No apps registered. Use 'xr auth apps add' to register one.",
                    );
                    return Ok(());
                }

                let app_choice = match dialoguer::Select::new()
                    .with_prompt("Select default app")
                    .items(&apps)
                    .interact_opt()
                {
                    Ok(Some(idx)) => apps[idx].clone(),
                    Ok(None) => return Ok(()),
                    Err(e) => {
                        return Err(XurlError::validation(format!("Selection error: {e}")));
                    }
                };

                auth.token_store.set_default_app(&app_choice)?;
                out.print_message(
                    stdout,
                    &format!("\x1b[32mDefault app set to {app_choice:?}\x1b[0m"),
                );

                let users = auth.token_store.get_oauth2_usernames_for_app(&app_choice);
                if !users.is_empty()
                    && let Ok(Some(idx)) = dialoguer::Select::new()
                        .with_prompt("Select default OAuth2 user")
                        .items(&users)
                        .interact_opt()
                {
                    let user = &users[idx];
                    auth.token_store.set_default_user(&app_choice, user)?;
                    out.print_message(
                        stdout,
                        &format!("\x1b[32mDefault user set to {user:?}\x1b[0m"),
                    );
                }
            }
        }
    }
    Ok(())
}

/// Subset of `AuthGlobalFlags` needed by app management.
#[derive(Debug, Clone, Copy)]
struct AppGlobalFlags {
    no_interactive: bool,
    dry_run: bool,
    quiet: bool,
}

fn run_app_command(
    cmd: AppCommands,
    auth: &mut Auth,
    flags: AppGlobalFlags,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<()> {
    let AppGlobalFlags {
        no_interactive,
        dry_run,
        quiet,
    } = flags;
    match cmd {
        AppCommands::Add {
            name,
            client_id,
            client_secret,
            redirect_uri,
        } => {
            if dry_run {
                let ctx = json!({
                    "command": "app-add",
                    "name": name,
                    "has_redirect_uri": redirect_uri.is_some(),
                });
                out.print_dry_run(stdout, true, 0, &ctx);
                return Ok(());
            }
            auth.token_store
                .add_app(&name, &client_id, &client_secret)?;
            if let Some(ref uri) = redirect_uri {
                auth.token_store.set_app_redirect_uri(&name, uri)?;
            }
            out.print_message(stdout, &format!("\x1b[32mApp {name:?} registered!\x1b[0m"));
            if auth.token_store.list_apps().len() == 1 {
                out.print_message(stdout, "  (set as default app)");
            }
        }
        AppCommands::Update {
            name,
            client_id,
            client_secret,
            redirect_uri,
        } => {
            if client_id.is_none() && client_secret.is_none() && redirect_uri.is_none() {
                return Err(XurlError::validation(
                    "Nothing to update. Provide --client-id, --client-secret, and/or --redirect-uri.",
                ));
            }
            if dry_run {
                let ctx = json!({
                    "command": "app-update",
                    "name": name,
                    "has_client_id": client_id.is_some(),
                    "has_client_secret": client_secret.is_some(),
                    "has_redirect_uri": redirect_uri.is_some(),
                });
                out.print_dry_run(stdout, true, 0, &ctx);
                return Ok(());
            }
            if client_id.is_some() || client_secret.is_some() {
                auth.token_store.update_app(
                    &name,
                    &client_id.unwrap_or_default(),
                    &client_secret.unwrap_or_default(),
                )?;
            }
            if let Some(ref uri) = redirect_uri {
                auth.token_store.set_app_redirect_uri(&name, uri)?;
            }
            out.print_message(stdout, &format!("\x1b[32mApp {name:?} updated.\x1b[0m"));
        }
        AppCommands::Remove { name, force } => {
            let ctx = json!({"command": "app-remove", "name": name});
            // Force/confirmation gate runs BEFORE dry-run so an unconfirmed
            // destructive op in interactive mode does not leak a dry-run
            // envelope.
            match gate_destructive(
                force,
                no_interactive,
                quiet,
                &format!("Remove app {name:?}?"),
            )? {
                Gate::Proceed => {}
                Gate::Declined => return Ok(()),
                Gate::ConfirmationRequired => {
                    out.print_confirmation_required(stderr, &ctx, EXIT_GENERAL_ERROR);
                    return Err(XurlError::EnvelopeAlreadyEmitted {
                        exit_code: EXIT_GENERAL_ERROR,
                    });
                }
            }
            if dry_run {
                out.print_dry_run(stdout, true, 0, &ctx);
                return Ok(());
            }
            auth.token_store.remove_app(&name)?;
            out.print_message(stdout, &format!("\x1b[32mApp {name:?} removed.\x1b[0m"));
        }
        AppCommands::RedirectUri { command } => {
            return run_redirect_uri_command(command, auth, dry_run, out, stdout);
        }
        AppCommands::List => {
            // Read through the runner-constructed store so tempdir-based
            // CLI tests observe the same `~/.xurl` the runner saw (KTD7).
            let ts = &auth.token_store;
            let apps = ts.list_apps();
            let default_app = ts.get_default_app();

            if apps.is_empty() {
                out.print_message(
                    stdout,
                    "No apps registered. Use 'xr auth apps add' to register one.",
                );
                return Ok(());
            }

            let entries = build_app_status_entries(ts, &apps, default_app);

            match out.format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let value = serde_json::to_value(&entries)?;
                    out.print_response(stdout, &value);
                }
                OutputFormat::Text => {
                    for (name, entry) in apps.iter().zip(entries.iter()) {
                        let Some(app) = ts.get_app(name) else {
                            continue;
                        };
                        let marker = if name == default_app {
                            "\u{25b8} "
                        } else {
                            "  "
                        };
                        let client_hint = if app.client_id.is_empty() {
                            String::new()
                        } else {
                            format!(" (client_id: {}...)", entry.client_id_hint)
                        };
                        // R20: inline the effective redirect URI + source hint.
                        out.print_message(
                            stdout,
                            &format!(
                                "{marker}{name}{client_hint} [redirect_uri: {} ({})]",
                                entry.redirect_uri,
                                entry.redirect_uri_source.as_text_label()
                            ),
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

/// Builds the credential-less-default-app warning when applicable.
///
/// Returns `Some(message)` when the default app exists with an empty
/// `client_id` AND at least one other registered app has a non-empty
/// `client_id`. Returns `None` otherwise.
///
/// Caller decides whether to emit (callers gate this on the user not having
/// passed `--app` per R13). The message uses plain ASCII (no ANSI escape
/// codes) per KTD4 and is routed through `OutputConfig::info` so `--quiet`
/// and `--output json` suppress it.
fn credential_less_default_warning(ts: &TokenStore) -> Option<String> {
    let default_name = ts.get_default_app();
    let default_app = ts.get_app(default_name)?;
    if !default_app.client_id.is_empty() {
        return None;
    }

    let credentialed: Vec<(String, String)> = ts
        .list_apps()
        .into_iter()
        .filter(|name| name != default_name)
        .filter_map(|name| {
            let app = ts.get_app(&name)?;
            if app.client_id.is_empty() {
                None
            } else {
                Some((name, truncate(&app.client_id, 8).to_string()))
            }
        })
        .collect();

    if credentialed.is_empty() {
        return None;
    }

    let mut msg = String::new();
    msg.push_str(&format!(
        "warning: --app not specified. The OAuth2 token will be saved to the \"{default_name}\" app,\n"
    ));
    msg.push_str("which has no client credentials stored. API calls will fail with 401 errors.\n");
    msg.push('\n');
    msg.push_str("App(s) with credentials available:\n");
    for (name, hint) in &credentialed {
        msg.push_str(&format!("  --app {name}  [client_id: {hint}...]\n"));
    }
    msg.push('\n');
    let first = &credentialed[0].0;
    msg.push_str(&format!("Run instead:  xr auth oauth2 --app {first}"));
    Some(msg)
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

/// Builds the typed JSON intermediate for `auth status` and `auth apps list`.
///
/// Constructs each `AppStatusEntry` field-by-field from named accessors per
/// R23 + KTD11; no `From<&App>` and no `Serialize`-on-`App`. The
/// `REDIRECT_URI` env var is read once per entry to drive the resolver.
fn build_app_status_entries(
    ts: &TokenStore,
    apps: &[String],
    default_app: &str,
) -> Vec<AppStatusEntry> {
    let env = std::env::var("REDIRECT_URI").ok();
    apps.iter()
        .filter_map(|name| {
            let app = ts.get_app(name)?;
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
                bearer: app.bearer_token.is_some(),
                default: name == default_app,
                oauth2_unnamed: app.unnamed_oauth2_token.is_some(),
            })
        })
        .collect()
}

fn run_redirect_uri_command(
    cmd: RedirectUriCommands,
    auth: &mut Auth,
    dry_run: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
) -> Result<()> {
    match cmd {
        RedirectUriCommands::Get { name } => {
            let target = match name.as_deref() {
                Some(n) => n.to_string(),
                None => {
                    let default = auth.token_store.get_default_app();
                    if default.is_empty() {
                        return Err(XurlError::validation("no default app set; specify NAME"));
                    }
                    default.to_string()
                }
            };

            let env = std::env::var("REDIRECT_URI").ok();
            let stored = auth
                .token_store
                .get_app_redirect_uri(&target)
                .map(str::to_string);
            let resolved = config::resolve_redirect_uri_from(env.clone(), stored.as_deref());

            match out.format {
                crate::output::OutputFormat::Json | crate::output::OutputFormat::Jsonl => {
                    let value = serde_json::json!({
                        "app": target,
                        "effective_redirect_uri": resolved.uri,
                        "effective_source": resolved.source,
                        "stored_redirect_uri": stored,
                    });
                    out.print_response(stdout, &value);
                }
                crate::output::OutputFormat::Text => {
                    out.print_message(stdout, &format!("app: {target}"));
                    out.print_message(stdout, &format!("effective_redirect_uri: {}", resolved.uri));
                    out.print_message(
                        stdout,
                        &format!("effective_source: {}", resolved.source.as_text_label()),
                    );
                    let stored_display = stored.as_deref().unwrap_or("(none)");
                    out.print_message(stdout, &format!("stored_redirect_uri: {stored_display}"));
                }
            }
        }
        RedirectUriCommands::Set { name, uri } => {
            if dry_run {
                let ctx = json!({
                    "command": "redirect-uri-set",
                    "app": name,
                    "redirect_uri": uri,
                });
                out.print_dry_run(stdout, true, 0, &ctx);
                return Ok(());
            }
            auth.token_store.set_app_redirect_uri(&name, &uri)?;
            match out.format {
                crate::output::OutputFormat::Json | crate::output::OutputFormat::Jsonl => {
                    let value = serde_json::json!({
                        "status": "ok",
                        "app": name,
                        "redirect_uri": uri,
                    });
                    out.print_response(stdout, &value);
                }
                crate::output::OutputFormat::Text => {
                    out.print_message(stdout, &format!("Set redirect URI for {name:?}"));
                }
            }
        }
    }
    Ok(())
}
