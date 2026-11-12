//! Session state commands: `auth status`, `auth clear`, and the default-app
//! and default-user selection, including its interactive picker.

use std::io::Write;

use serde_json::json;

use super::{
    AuthCtx, AuthGlobalFlags, StoredAccountEntry, build_app_status_entries, env_bearer_app,
    gate_destructive, print_no_apps_registered, rehearsed, rehearsed_asking,
};
use crate::cli::envelope::{ErrorBody, Reason};
use crate::cli::failure::{CommandResult, Failure};
use crate::cli::hints::NextStep;
use crate::cli::output::OutputConfig;
use xdk::auth::Auth;
use xdk::error::{EXIT_GENERAL_ERROR, Error, Result};
use xdk::store::TokenStore;

pub(super) fn status(auth: &Auth, out: &OutputConfig, stdout: &mut dyn Write) -> CommandResult<()> {
    // Read through the runner-constructed store so tempdir-based
    // CLI tests observe the same `~/.xurl` the runner saw.
    let ts = &auth.token_store;
    let apps = ts.list_apps();
    let default_app = ts.get_default_app();

    if apps.is_empty() {
        return print_no_apps_registered(auth, out, stdout);
    }

    let entries = build_app_status_entries(
        ts,
        &apps,
        default_app,
        auth.redirect_uri_override(),
        env_bearer_app(auth).as_deref(),
    );

    if out.format.is_structured() {
        let value =
            serde_json::json!({ "apps": serde_json::to_value(&entries).map_err(Error::from)? });
        out.print_success(stdout, &value);
    } else {
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

            // Surface the effective redirect URI and its source.
            out.print_message(
                stdout,
                &format!(
                    "      redirect_uri: {} [{}]",
                    entry.redirect_uri,
                    entry.redirect_uri_source.as_text_label()
                ),
            );
            if let Some(stored) = entry.redirect_uri_stored.as_deref() {
                out.print_message(stdout, &format!("      stored_redirect_uri: {stored}"));
            }

            if entry.oauth2_users.is_empty() && !entry.oauth2_unnamed {
                out.print_message(stdout, "      oauth2: (none)");
            } else {
                for (u, account) in entry.oauth2_users.iter().zip(&entry.oauth2_accounts) {
                    let suffix = account.text_suffix();
                    if *u == app.default_user {
                        out.print_message(stdout, &format!("    \u{25b8} oauth2: {u}{suffix}"));
                    } else {
                        out.print_message(stdout, &format!("      oauth2: {u}{suffix}"));
                    }
                }
                // Render the unnamed (`/me`-failed salvage)
                // slot after named users, labelled `(unknown user)`.
                if entry.oauth2_unnamed {
                    out.print_message(stdout, "      oauth2: (unknown user)");
                }
            }

            if entry.oauth1 {
                let suffix = entry
                    .oauth1_account
                    .as_ref()
                    .map(StoredAccountEntry::text_suffix)
                    .unwrap_or_default();
                out.print_message(stdout, &format!("      oauth1: \u{2713}{suffix}"));
            } else {
                out.print_message(stdout, "      oauth1: \u{2013}");
            }

            match entry.bearer_source {
                Some(source) => out.print_message(
                    stdout,
                    &format!("      bearer: \u{2713}{}", source.as_text_label()),
                ),
                None => out.print_message(stdout, "      bearer: \u{2013}"),
            }

            if i < apps.len() - 1 {
                out.print_message(stdout, "");
            }
        }
    }
    Ok(())
}

/// Arguments of `xr auth clear`: one selector per credential kind the command
/// can remove, plus the confirmation bypass.
pub(super) struct ClearArgs {
    pub(super) all: bool,
    pub(super) oauth1: bool,
    pub(super) oauth2_username: Option<String>,
    pub(super) bearer: bool,
    pub(super) force: bool,
}

/// What `xr auth clear` removes, picked once from the four selectors. When
/// several are given the first of this order wins: all, `OAuth1`, an `OAuth2`
/// login, bearer.
enum ClearTarget<'a> {
    All,
    OAuth1,
    OAuth2(&'a str),
    Bearer,
}

impl<'a> ClearTarget<'a> {
    fn from_args(args: &'a ClearArgs) -> Option<Self> {
        if args.all {
            Some(Self::All)
        } else if args.oauth1 {
            Some(Self::OAuth1)
        } else if let Some(username) = &args.oauth2_username {
            Some(Self::OAuth2(username))
        } else if args.bearer {
            Some(Self::Bearer)
        } else {
            None
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::All => "all credentials",
            Self::OAuth1 => "OAuth1 tokens",
            Self::OAuth2(_) => "OAuth2 token",
            Self::Bearer => "bearer token",
        }
    }

    fn prompt(&self, app: &str) -> String {
        format!("Clear {} in app {app:?}?", self.label())
    }

    fn cleared_message(&self) -> String {
        match self {
            Self::All => "All authentication cleared!".to_string(),
            Self::OAuth1 => "OAuth1 tokens cleared!".to_string(),
            Self::OAuth2(username) => format!("OAuth2 token cleared for {username}!"),
            Self::Bearer => "Bearer token cleared!".to_string(),
        }
    }

    /// Clears the target from the app `app` names, or the default app when it
    /// is empty. The checks come first, so a rehearsal of this is refused as
    /// the real run is.
    fn clear(&self, store: &mut TokenStore, app: &str) -> Result<()> {
        let login = match self {
            Self::OAuth2(username) => Some(*username),
            Self::All | Self::OAuth1 | Self::Bearer => None,
        };
        store.refuse_if_clear_target_missing(app, login)?;
        match self {
            Self::All => store.clear_all_for_app(app),
            Self::OAuth1 => store.clear_oauth1_tokens_for_app(app),
            Self::OAuth2(username) => store.clear_oauth2_token_for_app(app, username),
            Self::Bearer => store.clear_bearer_token_for_app(app),
        }
    }
}

fn nothing_to_clear() -> Error {
    Error::validation("nothing-to-clear")
}

pub(super) fn clear(args: ClearArgs, ctx: AuthCtx<'_>) -> CommandResult<()> {
    let AuthCtx {
        auth,
        flags,
        out,
        stdout,
        ..
    } = ctx;
    let AuthGlobalFlags {
        no_interactive,
        dry_run,
        quiet,
        ..
    } = flags;
    let app = auth.app_name().to_string();
    let ctx = json!({
        "command": "auth-clear",
        "app": auth.token_store.get_active_app_name(&app),
        "all": args.all,
        "oauth1": args.oauth1,
        "oauth2_username": args.oauth2_username,
        "bearer": args.bearer,
    });
    let force = args.force;
    let target = ClearTarget::from_args(&args);

    let clear_named = |store: &mut TokenStore| match &target {
        Some(target) => target.clear(store, &app),
        None => Err(nothing_to_clear()),
    };
    if dry_run {
        // Only a named target stops the real run to ask.
        let ask = target.as_ref().map(|_| force);
        return rehearsed_asking(&auth.token_store, clear_named, ctx, ask, out, stdout);
    }
    let Some(target) = &target else {
        return Err(nothing_to_clear().into());
    };
    auth.token_store.rehearse(clear_named)?;
    let prompt = target.prompt(auth.token_store.get_active_app_name(&app));
    gate_destructive(force, no_interactive, quiet, &prompt, &ctx)?;

    target.clear(&mut auth.token_store, &app)?;
    out.print_ok_message(stdout, &target.cleared_message());
    Ok(())
}

/// Arguments of `xr auth default`: the app to make default and the `OAuth2`
/// user to make default within it. Either may be absent, which sends the
/// command to its interactive picker.
pub(super) struct SetDefaultArgs {
    pub(super) app_name: Option<String>,
    pub(super) username: Option<String>,
}

pub(super) fn set_default(args: SetDefaultArgs, ctx: AuthCtx<'_>) -> CommandResult<()> {
    let SetDefaultArgs { app_name, username } = args;
    let AuthCtx {
        auth,
        flags,
        out,
        stdout,
        stderr,
    } = ctx;
    let AuthGlobalFlags { dry_run, .. } = flags;
    if dry_run {
        let ctx = json!({
            "command": "auth-default",
            "app_name": app_name,
            "username": username,
        });
        let Some(app) = app_name.as_deref() else {
            out.print_dry_run(stdout, true, 0, &ctx);
            return Ok(());
        };
        let save = |store: &mut TokenStore| save_default(store, app, username.as_deref());
        return rehearsed(&auth.token_store, save, ctx, out, stdout);
    }
    if let Some(app_name) = app_name {
        save_default(&mut auth.token_store, &app_name, username.as_deref())?;
        out.print_ok_message(stdout, &default_set_message(&app_name, username.as_deref()));
    } else {
        // Interactive picker — gate on `--no-interactive` AND on
        // TTY-ness of stdin/stderr (the dialoguer transports). Without
        // a real terminal the dialoguer state machine would block on
        // `/dev/null` or panic; emit the canonical `no-tty` envelope
        // and hand back only the exit code so the runner skips its
        // generic error path.
        if !out.is_interactive_terminal() {
            out.emit_error_envelope(
                stderr,
                ErrorBody {
                    reason: Reason::NoTty,
                    exit_code: EXIT_GENERAL_ERROR,
                    message: Some(
                        "no default app set; pass --app or run 'xr auth default <name>' interactively"
                            .to_string(),
                    ),
                    next_step: Some(NextStep::name_default_app()),
                    ..ErrorBody::default()
                },
            );
            return Err(Failure::Emitted {
                exit_code: EXIT_GENERAL_ERROR,
            });
        }

        let apps = auth.token_store.list_apps();
        if apps.is_empty() {
            return print_no_apps_registered(auth, out, stdout);
        }

        let app_choice = match prompt_select("Select default app", &apps)? {
            Some(name) => name,
            None => return Ok(()),
        };

        // Defensive TTY re-check before the second dialoguer prompt
        // (the user picker). The outer gate already guarantees this in
        // the current flow; the explicit check keeps the invariant
        // local to the call site so future refactors can't drop it.
        let users = auth.token_store.get_oauth2_usernames_for_app(&app_choice);
        let user_choice = if !users.is_empty() && out.is_interactive_terminal() {
            prompt_select("Select default OAuth2 user", &users)
                .ok()
                .flatten()
        } else {
            None
        };
        save_default(&mut auth.token_store, &app_choice, user_choice.as_deref())?;
        out.print_ok_message(
            stdout,
            &default_set_message(&app_choice, user_choice.as_deref()),
        );
    }
    Ok(())
}

/// Saves the default app, with its default user when one was named, as one
/// change to the store.
fn save_default(store: &mut TokenStore, app: &str, user: Option<&str>) -> Result<()> {
    match user {
        Some(user) => store.set_default_app_and_user(app, user),
        None => store.set_default_app(app),
    }
}

/// The success message for `auth default`: one for the whole change, so a
/// structured caller reads one document whether or not a user was set.
fn default_set_message(app: &str, user: Option<&str>) -> String {
    match user {
        Some(user) => {
            format!("\x1b[32mDefault app set to {app:?} and default user to {user:?}\x1b[0m")
        }
        None => format!("\x1b[32mDefault app set to {app:?}\x1b[0m"),
    }
}

/// Lists `items` on stderr with numeric indices and prompts the user to pick
/// one via stdin. Returns the chosen item on success or `Ok(None)` on EOF /
/// blank input / out-of-range / non-numeric input.
///
/// Stays dialoguer-free so the binary doesn't carry an interactive prompt
/// library dependency. Callers must verify TTY readiness independently.
fn prompt_select(label: &str, items: &[String]) -> Result<Option<String>> {
    use std::io::BufRead;
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    writeln!(handle, "{label}:").map_err(|e| Error::io(format!("prompt failed: {e}")))?;
    for (idx, item) in items.iter().enumerate() {
        writeln!(handle, "  {}) {item}", idx + 1)
            .map_err(|e| Error::io(format!("prompt failed: {e}")))?;
    }
    write!(handle, "Choice [1-{}]: ", items.len())
        .map_err(|e| Error::io(format!("prompt failed: {e}")))?;
    handle
        .flush()
        .map_err(|e| Error::io(format!("prompt failed: {e}")))?;
    drop(handle);

    let stdin = std::io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_err() {
        return Ok(None);
    }
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let Ok(idx) = trimmed.parse::<usize>() else {
        return Ok(None);
    };
    if idx == 0 || idx > items.len() {
        return Ok(None);
    }
    Ok(Some(items[idx - 1].clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_confirmation_names_what_goes_and_the_app_it_goes_from() {
        assert_eq!(
            ClearTarget::OAuth2("alice").prompt("work"),
            r#"Clear OAuth2 token in app "work"?"#
        );
        assert_eq!(
            ClearTarget::All.prompt("default"),
            r#"Clear all credentials in app "default"?"#
        );
    }
}
