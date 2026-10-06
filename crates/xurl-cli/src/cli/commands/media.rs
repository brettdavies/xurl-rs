/// Media subcommand handlers — upload, status, alt text, and subtitles.
use std::io::Write;

use serde_json::json;

use super::{dry_run_or_validate, make_client, print_typed, with_flags};
use crate::cli::output::OutputConfig;
use crate::cli::{MediaCommands, SubtitlesCommands};
use xdk::api::{self, shortcuts};
use xdk::auth::Auth;
use xdk::config::Config;
use xdk::error::Result;

#[allow(clippy::too_many_arguments)]
pub(super) async fn run_media_command(
    cmd: MediaCommands,
    cfg: &Config,
    auth: Auth,
    verbose: bool,
    dry_run: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<()> {
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
            if dry_run {
                let ctx = json!({
                    "command": "media-upload",
                    "file": file,
                    "media_type": media_type,
                    "category": category,
                });
                out.print_dry_run(stdout, true, 0, &ctx);
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let outcome = api::execute_media_upload(
                &file,
                &media_type,
                &category,
                &auth_type.unwrap_or_default(),
                &username.unwrap_or_default(),
                trace,
                wait.0,
                &headers,
                &client,
            )
            .await?;
            if verbose {
                out.print_response(stdout, &serde_json::to_value(&outcome.init)?);
            }
            out.print_response(stdout, &serde_json::to_value(&outcome.finalize)?);
            // The upload completed at FINALIZE; a processing failure after it
            // still leaves the media id on stdout for the caller to act on.
            match outcome.processing {
                Some(Ok(processing)) => {
                    out.print_response(stdout, &serde_json::to_value(&processing)?);
                }
                Some(Err(err)) => return Err(err),
                None => {}
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
        MediaCommands::Status {
            media_id,
            auth_type,
            username,
            wait,
            trace,
            headers,
        } => {
            let client = make_client(cfg, auth)?;
            let response = api::execute_media_status(
                &media_id,
                &auth_type.unwrap_or_default(),
                &username.unwrap_or_default(),
                wait.0,
                trace,
                &headers,
                &client,
            )
            .await?;
            out.print_response(stdout, &serde_json::to_value(&response)?);
            Ok(())
        }
        MediaCommands::AltText {
            media_id,
            text,
            common,
        } => {
            let ctx = json!({
                "command": "media-alt-text",
                "media_id": media_id,
                "text": text,
            });
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_media_id(&media_id)?;
                shortcuts::validate_alt_text(&text)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.set_media_alt_text(&media_id, &text), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)
        }
        MediaCommands::Subtitles { action } => match action {
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
                let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                    shortcuts::validate_media_id(&video_id)?;
                    shortcuts::validate_media_id(&subtitles_id)?;
                    shortcuts::validate_language_code(&language)
                })?;
                if !proceed {
                    return Ok(());
                }
                let client = make_client(cfg, auth)?;
                let call = client.add_media_subtitles(
                    &video_id,
                    category,
                    &subtitles_id,
                    &language,
                    display_name.as_deref(),
                );
                let response = with_flags(call, &common, None).send().await?;
                print_typed(out, stdout, &response)
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
                let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                    shortcuts::validate_media_id(&video_id)?;
                    shortcuts::validate_language_code(&language)
                })?;
                if !proceed {
                    return Ok(());
                }
                let client = make_client(cfg, auth)?;
                let call = client.remove_media_subtitles(&video_id, category, &language);
                let response = with_flags(call, &common, None).send().await?;
                print_typed(out, stdout, &response)
            }
        },
    }
}
