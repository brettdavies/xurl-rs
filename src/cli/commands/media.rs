/// Media subcommand handlers — upload and status.
use std::io::Write;

use serde_json::json;

use crate::api::{self, Client};
use crate::auth::Auth;
use crate::cli::MediaCommands;
use crate::cli::output::OutputConfig;
use crate::config::Config;
use crate::error::Result;

#[allow(clippy::too_many_arguments)]
pub(super) async fn run_media_command(
    cmd: MediaCommands,
    cfg: &Config,
    auth: Auth,
    verbose: bool,
    dry_run: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
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
            let client = Client::new(cfg, auth)?;
            let outcome = api::execute_media_upload(
                &file,
                &media_type,
                &category,
                &auth_type.unwrap_or_default(),
                &username.unwrap_or_default(),
                trace,
                wait,
                &headers,
                &client,
            )
            .await?;
            if verbose {
                out.print_response(stdout, &serde_json::to_value(&outcome.init)?);
            }
            out.print_response(stdout, &serde_json::to_value(&outcome.finalize)?);
            if let Some(processing) = &outcome.processing {
                out.print_response(stdout, &serde_json::to_value(processing)?);
            }
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
            let client = Client::new(cfg, auth)?;
            let response = api::execute_media_status(
                &media_id,
                &auth_type.unwrap_or_default(),
                &username.unwrap_or_default(),
                wait,
                trace,
                &headers,
                &client,
            )
            .await?;
            out.print_response(stdout, &serde_json::to_value(&response)?);
            Ok(())
        }
    }
}
