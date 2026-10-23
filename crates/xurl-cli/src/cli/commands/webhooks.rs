//! The `webhooks` family: the app's registered webhooks, and the Account
//! Activity subscriptions and filtered-stream links that deliver to them.
//! `webhooks_listen.rs` holds the local receiver.

use serde_json::json;

use super::webhooks_listen::{Listener, listen};
use super::{
    DryRun, Gate, Run, destructive_dry_run_context, dry_run_or_validate, gate_destructive,
    make_client, send_or_report, with_flags,
};
use crate::cli::failure::{CommandResult, Failure};
use crate::cli::{
    CommonFlags, WebhookStreamLinksCommands, WebhookSubscriptionsCommands, WebhooksCommands,
};
use std::net::SocketAddr;
use std::path::Path;
use std::time::SystemTime;

use xdk::api::shortcuts;
use xdk::error::Error;
use xdk::webhooks::{read_checkpoint, replay_window};

pub(super) async fn run(action: WebhooksCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        WebhooksCommands::List { common } => list(run, &common).await,
        WebhooksCommands::Add { url, common } => add(run, &url, &common).await,
        WebhooksCommands::Validate { webhook_id, common } => {
            validate(run, &webhook_id, &common).await
        }
        WebhooksCommands::Remove {
            webhook_id,
            force,
            common,
        } => remove(run, &webhook_id, force, &common).await,
        WebhooksCommands::Subscriptions { action } => subscriptions(action, run).await,
        WebhooksCommands::StreamLinks { action } => stream_links(action, run).await,
        WebhooksCommands::Listen {
            port,
            bind,
            path,
            secret,
            allow_unsigned,
            max_events,
            forward,
            header,
            forward_timeout,
            program,
            checkpoint,
        } => {
            let listener = Listener {
                bind: SocketAddr::new(bind, port),
                path,
                secret,
                allow_unsigned,
                max_events,
                forward,
                headers: header,
                forward_timeout,
                program,
                checkpoint,
            };
            listen(run, listener).await
        }
        WebhooksCommands::Replay {
            webhook_id,
            from,
            to,
            from_checkpoint,
            common,
        } => match from_checkpoint {
            Some(checkpoint) => {
                replay_from_checkpoint(run, &webhook_id, &checkpoint, &common).await
            }
            // clap requires both bounds when no checkpoint is given.
            None => {
                let (from, to) = (from.unwrap_or_default(), to.unwrap_or_default());
                replay(run, &webhook_id, &from, &to, &common).await
            }
        },
    }
}

async fn subscriptions(action: WebhookSubscriptionsCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        WebhookSubscriptionsCommands::Count { common } => {
            let Run {
                cfg,
                auth,
                flags,
                out,
                stdout,
                ..
            } = run;
            let dry_run = flags
                .dry_run
                .then(|| DryRun::new(json!({"command": "webhooks-subscriptions-count"})));
            let client = make_client(cfg, auth)?;
            let call = with_flags(
                client.get_account_activity_subscription_count(),
                &common,
                None,
            );
            send_or_report(out, stdout, dry_run, call).await?;
            Ok(())
        }
        WebhookSubscriptionsCommands::List { webhook_id, common } => {
            let Run {
                cfg,
                auth,
                flags,
                out,
                stdout,
                ..
            } = run;
            let ctx = json!({"command": "webhooks-subscriptions-list", "webhook_id": webhook_id});
            let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
                shortcuts::validate_webhook_id(&webhook_id)
            })?;
            let client = make_client(cfg, auth)?;
            let call = with_flags(
                client.get_account_activity_subscriptions(&webhook_id),
                &common,
                None,
            );
            send_or_report(out, stdout, dry_run, call).await?;
            Ok(())
        }
        WebhookSubscriptionsCommands::Add { webhook_id, common } => {
            let Run {
                cfg,
                auth,
                flags,
                out,
                stdout,
                ..
            } = run;
            let ctx = json!({"command": "webhooks-subscriptions-add", "webhook_id": webhook_id});
            let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
                shortcuts::validate_webhook_id(&webhook_id)
            })?;
            let client = make_client(cfg, auth)?;
            let call = with_flags(
                client.create_account_activity_subscription(&webhook_id),
                &common,
                None,
            );
            send_or_report(out, stdout, dry_run, call).await?;
            Ok(())
        }
        WebhookSubscriptionsCommands::Check { webhook_id, common } => {
            let Run {
                cfg,
                auth,
                flags,
                out,
                stdout,
                ..
            } = run;
            let ctx = json!({"command": "webhooks-subscriptions-check", "webhook_id": webhook_id});
            let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
                shortcuts::validate_webhook_id(&webhook_id)
            })?;
            let client = make_client(cfg, auth)?;
            let call = with_flags(
                client.check_account_activity_subscription(&webhook_id),
                &common,
                None,
            );
            send_or_report(out, stdout, dry_run, call).await?;
            Ok(())
        }
        WebhookSubscriptionsCommands::Remove {
            webhook_id,
            user_id,
            force,
            common,
        } => unsubscribe(run, &webhook_id, &user_id, force, &common).await,
    }
}

async fn unsubscribe(
    run: Run<'_>,
    webhook_id: &str,
    user_id: &str,
    force: bool,
    common: &CommonFlags,
) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({
        "command": "webhooks-subscriptions-remove",
        "webhook_id": webhook_id,
        "user_id": user_id,
    });
    let valid = || {
        shortcuts::validate_webhook_id(webhook_id)?;
        shortcuts::validate_user_id(user_id)
    };
    let client = make_client(cfg, auth)?;
    let call = with_flags(
        client.delete_account_activity_subscription(webhook_id, user_id),
        common,
        None,
    );
    if flags.dry_run {
        let ctx = destructive_dry_run_context(ctx, force);
        let dry_run = dry_run_or_validate(true, ctx, valid)?;
        send_or_report(out, stdout, dry_run, call).await?;
        return Ok(());
    }
    match gate_destructive(
        force,
        flags.no_interactive,
        flags.quiet,
        &format!("End user {user_id}'s subscription to webhook {webhook_id}?"),
    )? {
        Gate::Proceed => {}
        Gate::Declined => return Ok(()),
        Gate::ConfirmationRequired => return Err(Failure::Unconfirmed(ctx)),
    }
    let dry_run = dry_run_or_validate(false, ctx, valid)?;
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}

async fn list(run: Run<'_>, common: &CommonFlags) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let dry_run = flags
        .dry_run
        .then(|| DryRun::new(json!({"command": "webhooks-list"})));
    let client = make_client(cfg, auth)?;
    let call = with_flags(client.get_webhooks(), common, None);
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}

async fn add(run: Run<'_>, url: &str, common: &CommonFlags) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({"command": "webhooks-add", "url": url});
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || shortcuts::validate_webhook_url(url))?;
    let client = make_client(cfg, auth)?;
    let call = with_flags(client.create_webhook(url), common, None);
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}

async fn validate(run: Run<'_>, webhook_id: &str, common: &CommonFlags) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({"command": "webhooks-validate", "webhook_id": webhook_id});
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
        shortcuts::validate_webhook_id(webhook_id)
    })?;
    let client = make_client(cfg, auth)?;
    let call = with_flags(client.validate_webhook(webhook_id), common, None);
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}

async fn remove(
    run: Run<'_>,
    webhook_id: &str,
    force: bool,
    common: &CommonFlags,
) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({"command": "webhooks-remove", "webhook_id": webhook_id});
    let client = make_client(cfg, auth)?;
    let call = with_flags(client.delete_webhook(webhook_id), common, None);
    if flags.dry_run {
        let ctx = destructive_dry_run_context(ctx, force);
        let dry_run =
            dry_run_or_validate(true, ctx, || shortcuts::validate_webhook_id(webhook_id))?;
        send_or_report(out, stdout, dry_run, call).await?;
        return Ok(());
    }
    match gate_destructive(
        force,
        flags.no_interactive,
        flags.quiet,
        &format!("Delete webhook {webhook_id}?"),
    )? {
        Gate::Proceed => {}
        Gate::Declined => return Ok(()),
        Gate::ConfirmationRequired => return Err(Failure::Unconfirmed(ctx)),
    }
    let dry_run = dry_run_or_validate(false, ctx, || shortcuts::validate_webhook_id(webhook_id))?;
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}

async fn replay(
    run: Run<'_>,
    webhook_id: &str,
    from: &str,
    to: &str,
    common: &CommonFlags,
) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({
        "command": "webhooks-replay",
        "webhook_id": webhook_id,
        "from": from,
        "to": to,
    });
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
        shortcuts::validate_webhook_id(webhook_id)?;
        shortcuts::validate_replay_time(from)?;
        shortcuts::validate_replay_time(to)
    })?;
    let client = make_client(cfg, auth)?;
    let call = with_flags(
        client.create_webhook_replay(webhook_id, from, to),
        common,
        None,
    );
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}

/// `replay --from-checkpoint`: the window from the minute `listen
/// --checkpoint` recorded to the current one, sent as `--from` and `--to`
/// are. The window rides beside X's answer, so a caller sees what a clamp
/// left out.
async fn replay_from_checkpoint(
    run: Run<'_>,
    webhook_id: &str,
    checkpoint: &Path,
    common: &CommonFlags,
) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let mut ctx = json!({
        "command": "webhooks-replay",
        "webhook_id": webhook_id,
        "checkpoint": checkpoint.display().to_string(),
    });
    let now = SystemTime::now();
    let window = match read_checkpoint(checkpoint, now).and_then(|at| replay_window(at, now)) {
        Ok(window) => window,
        Err(error) if flags.dry_run => {
            out.print_dry_run_refusal(stdout, &ctx, &with_both_fixes(error));
            return Ok(());
        }
        Err(error) => return Err(with_both_fixes(error).into()),
    };
    ctx["from"] = json!(window.from);
    ctx["to"] = json!(window.to);
    ctx["clamped"] = json!(window.clamped);
    if let Some(lost_from) = &window.lost_from {
        ctx["lost_from"] = json!(lost_from);
    }
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
        shortcuts::validate_webhook_id(webhook_id)
    })?;
    let client = make_client(cfg, auth)?;
    let call = with_flags(
        client.create_webhook_replay(webhook_id, &window.from, &window.to),
        common,
        None,
    );
    if dry_run.is_some() {
        send_or_report(out, stdout, dry_run, call).await?;
        return Ok(());
    }
    let mut answer = serde_json::to_value(call.send().await?).map_err(Error::from)?;
    answer["clamped"] = json!(window.clamped);
    if let Some(lost_from) = window.lost_from {
        answer["lost_from"] = json!(lost_from);
    }
    out.print_response(stdout, &answer);
    Ok(())
}

/// A checkpoint that could not be read, with both ways forward named.
fn with_both_fixes(error: Error) -> Error {
    match error {
        Error::Io { message, source } => Error::Io {
            message: format!(
                "{message}. Give --from and --to instead, or start `xr webhooks listen \
                 --checkpoint <FILE>` so the next restart has one"
            ),
            source,
        },
        other => other,
    }
}

async fn stream_links(action: WebhookStreamLinksCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        WebhookStreamLinksCommands::List { common } => {
            let Run {
                cfg,
                auth,
                flags,
                out,
                stdout,
                ..
            } = run;
            let dry_run = flags
                .dry_run
                .then(|| DryRun::new(json!({"command": "webhooks-stream-links-list"})));
            let client = make_client(cfg, auth)?;
            let call = with_flags(client.get_webhook_stream_links(), &common, None);
            send_or_report(out, stdout, dry_run, call).await?;
            Ok(())
        }
        WebhookStreamLinksCommands::Add { webhook_id, common } => {
            let Run {
                cfg,
                auth,
                flags,
                out,
                stdout,
                ..
            } = run;
            let ctx = json!({"command": "webhooks-stream-links-add", "webhook_id": webhook_id});
            let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
                shortcuts::validate_webhook_id(&webhook_id)
            })?;
            let client = make_client(cfg, auth)?;
            let call = with_flags(
                client.create_webhook_stream_link(&webhook_id),
                &common,
                None,
            );
            send_or_report(out, stdout, dry_run, call).await?;
            Ok(())
        }
        WebhookStreamLinksCommands::Remove {
            webhook_id,
            force,
            common,
        } => unlink_stream(run, &webhook_id, force, &common).await,
    }
}

async fn unlink_stream(
    run: Run<'_>,
    webhook_id: &str,
    force: bool,
    common: &CommonFlags,
) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({"command": "webhooks-stream-links-remove", "webhook_id": webhook_id});
    let client = make_client(cfg, auth)?;
    let call = with_flags(client.delete_webhook_stream_link(webhook_id), common, None);
    if flags.dry_run {
        let ctx = destructive_dry_run_context(ctx, force);
        let dry_run =
            dry_run_or_validate(true, ctx, || shortcuts::validate_webhook_id(webhook_id))?;
        send_or_report(out, stdout, dry_run, call).await?;
        return Ok(());
    }
    match gate_destructive(
        force,
        flags.no_interactive,
        flags.quiet,
        &format!("Stop delivering the filtered stream to webhook {webhook_id}?"),
    )? {
        Gate::Proceed => {}
        Gate::Declined => return Ok(()),
        Gate::ConfirmationRequired => return Err(Failure::Unconfirmed(ctx)),
    }
    let dry_run = dry_run_or_validate(false, ctx, || shortcuts::validate_webhook_id(webhook_id))?;
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}
