//! App registry commands under `auth apps`: add, update, remove, list, and the
//! per-app redirect-URI pair.

use std::io::Write;

use serde_json::json;

use super::secret::{self, SecretArg, SecretError};
use super::{
    AuthCtx, Gate, RedirectUriGetResponse, RedirectUriSetResponse, build_app_status_entries,
    destructive_dry_run_context, env_bearer_app, gate_destructive, print_no_apps_registered,
};
use crate::cli::failure::{CommandResult, Failure};
use crate::cli::hints::NextStep;
use crate::cli::output::OutputConfig;
use crate::cli::{AppCommands, RedirectUriCommands};
use xdk::auth::Auth;
use xdk::config;
use xdk::error::{Error, Result};

pub(super) fn run_app_command(cmd: AppCommands, ctx: AuthCtx<'_>) -> CommandResult<()> {
    match cmd {
        AppCommands::Add {
            name,
            client_id,
            client_secret,
            client_secret_file,
            redirect_uri,
        } => {
            let secret = resolve_client_secret(client_secret, client_secret_file)
                .map_err(|e| e.report("xr auth apps add", ctx.out, ctx.stderr))?
                .unwrap_or_default();
            add(ctx, &name, &client_id, &secret, redirect_uri.as_deref())
        }
        AppCommands::Update {
            name,
            client_id,
            client_secret,
            client_secret_file,
            redirect_uri,
        } => {
            let secret = resolve_client_secret(client_secret, client_secret_file)
                .map_err(|e| e.report("xr auth apps update", ctx.out, ctx.stderr))?;
            update(ctx, &name, client_id, secret, redirect_uri.as_deref())
        }
        AppCommands::Remove { name, force } => remove(ctx, &name, force),
        AppCommands::RedirectUri { command } => {
            run_redirect_uri_command(command, ctx.auth, ctx.flags.dry_run, ctx.out, ctx.stdout)
                .map_err(Failure::from)
        }
        AppCommands::List => list(ctx),
    }
}

fn add(
    ctx: AuthCtx<'_>,
    name: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: Option<&str>,
) -> CommandResult<()> {
    let AuthCtx {
        auth,
        flags,
        out,
        stdout,
        ..
    } = ctx;
    if flags.dry_run {
        let ctx = json!({
            "command": "app-add",
            "name": name,
            "has_redirect_uri": redirect_uri.is_some(),
        });
        out.print_dry_run(stdout, true, 0, &ctx);
        return Ok(());
    }
    auth.token_store.add_app(name, client_id, client_secret)?;
    if let Some(uri) = redirect_uri {
        auth.token_store.set_app_redirect_uri(name, uri)?;
    }
    let is_default = auth.token_store.get_default_app() == name;
    let next = NextStep::sign_in((!is_default).then_some(name), out.format.is_structured());
    if out.format.is_structured() {
        let payload = json!({
            "message": format!("App {name:?} registered."),
            "default": is_default,
            "next_step": next,
        });
        out.print_success(stdout, &payload);
    } else {
        let suffix = if is_default { " (default)" } else { "" };
        let next_cmd = next.display_invocation().unwrap_or("xr auth oauth2");
        out.print_message(
            stdout,
            &format!("\x1b[32mApp {name:?} registered{suffix}.\x1b[0m Next: {next_cmd}"),
        );
    }
    Ok(())
}

fn update(
    ctx: AuthCtx<'_>,
    name: &str,
    client_id: Option<String>,
    client_secret: Option<String>,
    redirect_uri: Option<&str>,
) -> CommandResult<()> {
    let AuthCtx {
        auth,
        flags,
        out,
        stdout,
        ..
    } = ctx;
    if client_id.is_none() && client_secret.is_none() && redirect_uri.is_none() {
        return Err(Error::validation(
            "Nothing to update. Provide --client-id, --client-secret, and/or --redirect-uri.",
        )
        .into());
    }
    if flags.dry_run {
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
            name,
            &client_id.unwrap_or_default(),
            &client_secret.unwrap_or_default(),
        )?;
    }
    if let Some(uri) = redirect_uri {
        auth.token_store.set_app_redirect_uri(name, uri)?;
    }
    out.print_ok_message(stdout, &format!("\x1b[32mApp {name:?} updated.\x1b[0m"));
    Ok(())
}

fn remove(ctx: AuthCtx<'_>, name: &str, force: bool) -> CommandResult<()> {
    let AuthCtx {
        auth,
        flags,
        out,
        stdout,
        ..
    } = ctx;
    let ctx = json!({"command": "app-remove", "name": name});
    if flags.dry_run {
        let ctx = destructive_dry_run_context(ctx, force);
        out.print_dry_run(stdout, true, 0, &ctx);
        return Ok(());
    }
    match gate_destructive(
        force,
        flags.no_interactive,
        flags.quiet,
        &format!("Remove app {name:?}?"),
    )? {
        Gate::Proceed => {}
        Gate::Declined => return Ok(()),
        Gate::ConfirmationRequired => return Err(Failure::Unconfirmed(ctx)),
    }
    auth.token_store.remove_app(name)?;
    out.print_ok_message(stdout, &format!("\x1b[32mApp {name:?} removed.\x1b[0m"));
    Ok(())
}

fn list(ctx: AuthCtx<'_>) -> CommandResult<()> {
    let AuthCtx {
        auth, out, stdout, ..
    } = ctx;
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
        return Ok(());
    }
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
        // Inline the effective redirect URI and its source.
        out.print_message(
            stdout,
            &format!(
                "{marker}{name}{client_hint} [redirect_uri: {} ({})]",
                entry.redirect_uri,
                entry.redirect_uri_source.as_text_label()
            ),
        );
    }
    Ok(())
}

/// The client secret from its plain flag or its file twin.
fn resolve_client_secret(
    value: Option<String>,
    file: Option<String>,
) -> std::result::Result<Option<String>, SecretError> {
    let [client_secret] = secret::resolve_from_process([SecretArg {
        file_flag: "--client-secret-file",
        value,
        file,
    }])?;
    Ok(client_secret)
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
                        return Err(Error::validation("no default app set; specify NAME"));
                    }
                    default.to_string()
                }
            };

            let env = auth.redirect_uri_override().map(str::to_string);
            let stored = auth
                .token_store
                .get_app_redirect_uri(&target)
                .map(str::to_string);
            let resolved = config::resolve_redirect_uri_from(env.clone(), stored.as_deref());

            if out.format.is_structured() {
                let response = RedirectUriGetResponse {
                    app: target.clone(),
                    effective_redirect_uri: resolved.uri.clone(),
                    effective_source: resolved.source,
                    stored_redirect_uri: stored.clone(),
                };
                let value = serde_json::to_value(&response)?;
                out.print_success(stdout, &value);
            } else {
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
            if out.format.is_structured() {
                let response = RedirectUriSetResponse {
                    status: "ok",
                    app: name.clone(),
                    redirect_uri: uri.clone(),
                };
                let value = serde_json::to_value(&response)?;
                out.print_response(stdout, &value);
            } else {
                out.print_ok_message(stdout, &format!("Set redirect URI for {name:?}"));
            }
        }
    }
    Ok(())
}
