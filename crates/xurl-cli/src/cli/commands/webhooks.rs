//! The `webhooks` family: the app's registered webhooks.

use serde_json::json;

use super::{
    DryRun, Gate, Run, destructive_dry_run_context, dry_run_or_validate, gate_destructive,
    make_client, send_or_report, with_flags,
};
use crate::cli::failure::{CommandResult, Failure};
use crate::cli::{CommonFlags, WebhooksCommands};
use xdk::api::shortcuts;

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
        WebhooksCommands::Replay {
            webhook_id,
            from,
            to,
            common,
        } => replay(run, &webhook_id, &from, &to, &common).await,
    }
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
