//! The commands that write or remove a post: `post`, `reply`, `quote`, and
//! `delete`.

use serde_json::json;

use super::{
    Gate, Run, destructive_dry_run_context, dry_run_or_validate, gate_destructive, make_client,
    send_or_report, with_flags,
};
use crate::cli::failure::{CommandResult, Failure};
use crate::cli::{Commands, CommonFlags};
use xdk::api::shortcuts;
use xdk::error::EXIT_GENERAL_ERROR;

pub(super) async fn run(cmd: Commands, run: Run<'_>) -> CommandResult<()> {
    match cmd {
        Commands::Post {
            text,
            media_ids,
            common,
        } => post(run, &text, &media_ids, &common).await,
        Commands::Reply {
            post_id,
            text,
            media_ids,
            common,
        } => reply(run, &post_id, &text, &media_ids, &common).await,
        Commands::Quote {
            post_id,
            text,
            common,
        } => quote(run, &post_id, &text, &common).await,
        Commands::Delete {
            post_id,
            force,
            common,
        } => delete(run, &post_id, force, &common).await,
        _ => unreachable!("run_subcommand routes only the post-writing commands here"),
    }
}

async fn post(
    run: Run<'_>,
    text: &str,
    media_ids: &[String],
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
        "command": "post",
        "body": text,
        "media_ids": media_ids,
    });
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
        shortcuts::validate_post_body(text)?;
        shortcuts::validate_media_attachments(media_ids)
    })?;
    let client = make_client(cfg, auth)?;
    let call = with_flags(client.create_post(text, media_ids), common, None);
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}

async fn reply(
    run: Run<'_>,
    post_id: &str,
    text: &str,
    media_ids: &[String],
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
        "command": "reply",
        "post_id": post_id,
        "body": text,
        "media_ids": media_ids,
    });
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
        shortcuts::validate_post_id(post_id)?;
        shortcuts::validate_post_body(text)?;
        shortcuts::validate_media_attachments(media_ids)
    })?;
    let client = make_client(cfg, auth)?;
    let call = with_flags(client.reply_to_post(post_id, text, media_ids), common, None);
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}

async fn quote(run: Run<'_>, post_id: &str, text: &str, common: &CommonFlags) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({
        "command": "quote",
        "post_id": post_id,
        "body": text,
    });
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
        shortcuts::validate_post_id(post_id)?;
        shortcuts::validate_post_body(text)
    })?;
    let client = make_client(cfg, auth)?;
    let call = with_flags(client.quote_post(post_id, text), common, None);
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}

async fn delete(
    run: Run<'_>,
    post_id: &str,
    force: bool,
    common: &CommonFlags,
) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        stderr,
    } = run;
    let ctx = json!({"command": "delete", "post_id": post_id});
    let client = make_client(cfg, auth)?;
    let call = with_flags(client.delete_post(post_id), common, None);
    if flags.dry_run {
        let ctx = destructive_dry_run_context(ctx, force);
        let dry_run = dry_run_or_validate(true, ctx, || shortcuts::validate_post_id(post_id))?;
        send_or_report(out, stdout, dry_run, call).await?;
        return Ok(());
    }
    match gate_destructive(
        force,
        flags.no_interactive,
        flags.quiet,
        &format!("Delete post {post_id}?"),
    )? {
        Gate::Proceed => {}
        Gate::Declined => return Ok(()),
        Gate::ConfirmationRequired => {
            out.print_confirmation_required(stderr, &ctx, EXIT_GENERAL_ERROR);
            return Err(Failure::Emitted {
                exit_code: EXIT_GENERAL_ERROR,
            });
        }
    }
    let dry_run = dry_run_or_validate(false, ctx, || shortcuts::validate_post_id(post_id))?;
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}
