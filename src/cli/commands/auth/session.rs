//! Session state commands: `auth status`, `auth clear`, and the default-app
//! and default-user selection, including its interactive picker.

use std::io::Write;

use serde_json::json;

use super::{
    AuthCtx, AuthGlobalFlags, Gate, build_app_status_entries, env_bearer_app, gate_destructive,
    print_no_apps_registered,
};
use crate::auth::Auth;
use crate::cli::output::OutputConfig;
use crate::error::{EXIT_GENERAL_ERROR, Error, Result};

pub(super) fn status(auth: &Auth, out: &OutputConfig, stdout: &mut dyn Write) -> Result<()> {
    // Read through the runner-constructed store so tempdir-based
    // CLI tests observe the same `~/.xurl` the runner saw (KTD7).
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
        let value = serde_json::json!({ "apps": serde_json::to_value(&entries)? });
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
                out.print_message(stdout, &format!("      stored_redirect_uri: {stored}"));
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

pub(super) fn clear(args: ClearArgs, ctx: AuthCtx<'_>) -> Result<()> {
    let ClearArgs {
        all,
        oauth1,
        oauth2_username,
        bearer,
        force,
    } = args;
    let AuthCtx {
        auth,
        flags,
        out,
        stdout,
        stderr,
    } = ctx;
    let AuthGlobalFlags {
        no_interactive,
        dry_run,
        quiet,
        ..
    } = flags;
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
                return Err(Error::EnvelopeAlreadyEmitted {
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
        out.print_ok_message(stdout, "All authentication cleared!");
    } else if oauth1 {
        auth.token_store.clear_oauth1_tokens()?;
        out.print_ok_message(stdout, "OAuth1 tokens cleared!");
    } else if let Some(username) = oauth2_username {
        auth.token_store.clear_oauth2_token(&username)?;
        out.print_ok_message(stdout, &format!("OAuth2 token cleared for {username}!"));
    } else if bearer {
        auth.token_store.clear_bearer_token()?;
        out.print_ok_message(stdout, "Bearer token cleared!");
    } else {
        return Err(Error::validation(
            "No authentication cleared! Use --all to clear all authentication.",
        ));
    }
    Ok(())
}

/// Arguments of `xr auth default`: the app to make default and the `OAuth2`
/// user to make default within it. Either may be absent, which sends the
/// command to its interactive picker.
pub(super) struct SetDefaultArgs {
    pub(super) app_name: Option<String>,
    pub(super) username: Option<String>,
}

pub(super) fn set_default(args: SetDefaultArgs, ctx: AuthCtx<'_>) -> Result<()> {
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
        out.print_dry_run(stdout, true, 0, &ctx);
        return Ok(());
    }
    if let Some(app_name) = app_name {
        auth.token_store.set_default_app(&app_name)?;
        out.print_ok_message(
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
        // Interactive picker — gate on `--no-interactive` AND on
        // TTY-ness of stdin/stderr (the dialoguer transports). Without
        // a real terminal the dialoguer state machine would block on
        // `/dev/null` or panic; emit the canonical `no-tty` envelope
        // and return `EnvelopeAlreadyEmitted` so the runner skips its
        // generic error path.
        if !out.is_interactive_terminal() {
            out.print_error_envelope(
                stderr,
                "no-tty",
                EXIT_GENERAL_ERROR,
                "no default app set; pass --app or run 'xr auth default <name>' interactively",
            );
            return Err(Error::EnvelopeAlreadyEmitted {
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

        auth.token_store.set_default_app(&app_choice)?;
        out.print_ok_message(
            stdout,
            &format!("\x1b[32mDefault app set to {app_choice:?}\x1b[0m"),
        );

        // Defensive TTY re-check before the second dialoguer prompt
        // (the user picker). The outer gate already guarantees this in
        // the current flow; the explicit check keeps the invariant
        // local to the call site so future refactors can't drop it.
        let users = auth.token_store.get_oauth2_usernames_for_app(&app_choice);
        if !users.is_empty()
            && out.is_interactive_terminal()
            && let Ok(Some(user)) = prompt_select("Select default OAuth2 user", &users)
        {
            auth.token_store.set_default_user(&app_choice, &user)?;
            out.print_message(
                stdout,
                &format!("\x1b[32mDefault user set to {user:?}\x1b[0m"),
            );
        }
    }
    Ok(())
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
    writeln!(handle, "{label}:").map_err(|e| Error::validation(format!("prompt failed: {e}")))?;
    for (idx, item) in items.iter().enumerate() {
        writeln!(handle, "  {}) {item}", idx + 1)
            .map_err(|e| Error::validation(format!("prompt failed: {e}")))?;
    }
    write!(handle, "Choice [1-{}]: ", items.len())
        .map_err(|e| Error::validation(format!("prompt failed: {e}")))?;
    handle
        .flush()
        .map_err(|e| Error::validation(format!("prompt failed: {e}")))?;
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
