/// Media subcommand handlers — upload and status.
use std::io::Write;

use serde_json::json;

use crate::api::{self, ApiClient};
use crate::auth::Auth;
use crate::cli::MediaCommands;
use crate::config::Config;
use crate::error::Result;
use crate::output::OutputConfig;

fn make_client(cfg: &Config, auth: Auth, out: &OutputConfig) -> ApiClient {
    let mut client = ApiClient::new(cfg, auth);
    client.set_output(out.clone());
    client
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run_media_command(
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
            let mut client = make_client(cfg, auth, out);
            api::execute_media_upload(
                &file,
                &media_type,
                &category,
                &auth_type.unwrap_or_default(),
                &username.unwrap_or_default(),
                verbose,
                trace,
                wait,
                &headers,
                &mut client,
                out,
                stdout,
                stderr,
            )
        }
        MediaCommands::Status {
            media_id,
            auth_type,
            username,
            wait,
            trace,
            headers,
        } => {
            let mut client = make_client(cfg, auth, out);
            api::execute_media_status(
                &media_id,
                &auth_type.unwrap_or_default(),
                &username.unwrap_or_default(),
                verbose,
                wait,
                trace,
                &headers,
                &mut client,
                out,
                stdout,
                stderr,
            )
        }
    }
}
