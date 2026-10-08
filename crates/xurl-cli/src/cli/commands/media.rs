/// Media subcommand handlers — upload, status, alt text, and subtitles.
use serde_json::json;

use super::{DryRun, Run, dry_run_or_validate, make_client, send_or_report, with_flags};
use crate::cli::failure::CommandResult;
use crate::cli::{CommonFlags, MediaCommands, ProcessingWait, SubtitlesCommands};
use xdk::api::{self, shortcuts};
use xdk::error::Result;

pub(super) async fn run(cmd: MediaCommands, run: Run<'_>) -> CommandResult<()> {
    match cmd {
        MediaCommands::Upload {
            file,
            media_type,
            category,
            wait,
            auth_type,
            username,
            trace,
            headers,
        } => {
            let args = UploadArgs {
                file,
                media_type,
                category,
                wait,
                auth_type,
                username,
                trace,
                headers,
            };
            upload(args, run).await?;
        }
        MediaCommands::Status {
            media_id,
            auth_type,
            username,
            wait,
            trace,
            headers,
        } => status(run, &media_id, auth_type, username, wait, trace, &headers).await?,
        MediaCommands::AltText {
            media_id,
            text,
            common,
        } => alt_text(run, &media_id, &text, &common).await?,
        MediaCommands::Subtitles { action } => subtitles(action, run).await?,
    }
    Ok(())
}

/// The arguments of `media upload`.
struct UploadArgs {
    file: String,
    media_type: String,
    category: String,
    wait: ProcessingWait,
    auth_type: Option<String>,
    username: Option<String>,
    trace: bool,
    headers: Vec<String>,
}

async fn upload(args: UploadArgs, run: Run<'_>) -> Result<()> {
    let UploadArgs {
        file,
        media_type,
        category,
        wait,
        auth_type,
        username,
        trace,
        headers,
    } = args;
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        stderr,
    } = run;
    let client = make_client(cfg, auth)?;
    let auth_type = auth_type.unwrap_or_default();
    let username = username.unwrap_or_default();
    if flags.dry_run {
        let ctx = json!({
            "command": "media-upload",
            "file": file,
            "media_type": media_type,
            "category": category,
        });
        let waits = wait.0.is_some();
        let credential =
            api::media_upload_preflight(&file, &auth_type, &username, &headers, waits, &client)
                .await;
        DryRun::new(ctx).answer(out, stdout, [credential]);
        return Ok(());
    }
    let outcome = api::execute_media_upload(
        &file,
        &media_type,
        &category,
        &auth_type,
        &username,
        trace,
        wait.0,
        &headers,
        &client,
    )
    .await?;
    // A structured answer is one document, so INIT's goes only to a human
    // who asked for verbose text.
    if flags.verbose && !out.format.is_structured() {
        out.print_response(stdout, &serde_json::to_value(&outcome.init)?);
    }
    // The upload completed at FINALIZE; a processing failure after it
    // still leaves the media id on stdout for the caller to act on.
    out.print_response(stdout, &serde_json::to_value(outcome.response())?);
    if let Some(Err(err)) = outcome.processing {
        return Err(err);
    }
    out.status(
        stderr,
        &format!(
            "Media uploaded successfully! Media ID: {}",
            outcome.init.data.id
        ),
    );
    Ok(())
}

async fn status(
    run: Run<'_>,
    media_id: &str,
    auth_type: Option<String>,
    username: Option<String>,
    wait: ProcessingWait,
    trace: bool,
    headers: &[String],
) -> Result<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let client = make_client(cfg, auth)?;
    let auth_type = auth_type.unwrap_or_default();
    let username = username.unwrap_or_default();
    if flags.dry_run {
        let ctx = json!({"command": "media-status", "media_id": media_id});
        let credential =
            api::media_status_auth_preflight(&auth_type, &username, headers, &client).await;
        DryRun::new(ctx).answer(out, stdout, [credential]);
        return Ok(());
    }
    let response = api::execute_media_status(
        media_id, &auth_type, &username, wait.0, trace, headers, &client,
    )
    .await?;
    out.print_response(stdout, &serde_json::to_value(&response)?);
    Ok(())
}

async fn alt_text(
    run: Run<'_>,
    media_id: &str,
    text: &str,
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
        "command": "media-alt-text",
        "media_id": media_id,
        "text": text,
    });
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
        shortcuts::validate_media_id(media_id)?;
        shortcuts::validate_alt_text(text)
    })?;
    let client = make_client(cfg, auth)?;
    let call = with_flags(client.set_media_alt_text(media_id, text), common, None);
    send_or_report(out, stdout, dry_run, call).await?;
    Ok(())
}

async fn subtitles(action: SubtitlesCommands, run: Run<'_>) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    match action {
        SubtitlesCommands::Add {
            video_id,
            subtitles_id,
            language,
            display_name,
            category,
            common,
        } => {
            let ctx = json!({
                "command": "media-subtitles-add",
                "video_id": video_id,
                "subtitles_id": subtitles_id,
                "language": language,
                "name": display_name,
                "category": category.upload_name(),
            });
            let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
                shortcuts::validate_media_id(&video_id)?;
                shortcuts::validate_media_id(&subtitles_id)?;
                shortcuts::validate_language_code(&language)
            })?;
            let client = make_client(cfg, auth)?;
            let call = client.add_media_subtitles(
                &video_id,
                category,
                &subtitles_id,
                &language,
                display_name.as_deref(),
            );
            send_or_report(out, stdout, dry_run, with_flags(call, &common, None)).await?;
            Ok(())
        }
        SubtitlesCommands::Remove {
            video_id,
            language,
            category,
            common,
        } => {
            let ctx = json!({
                "command": "media-subtitles-remove",
                "video_id": video_id,
                "language": language,
                "category": category.upload_name(),
            });
            let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
                shortcuts::validate_media_id(&video_id)?;
                shortcuts::validate_language_code(&language)
            })?;
            let client = make_client(cfg, auth)?;
            let call = client.remove_media_subtitles(&video_id, category, &language);
            send_or_report(out, stdout, dry_run, with_flags(call, &common, None)).await?;
            Ok(())
        }
    }
}
