/// Media subcommand handlers — upload and status.
use crate::api::{self, ApiClient};
use crate::auth::Auth;
use crate::cli::MediaCommands;
use crate::config::Config;
use crate::error::Result;
use crate::output::OutputConfig;

pub(super) fn run_media_command(
    cmd: MediaCommands,
    cfg: &Config,
    auth: Auth,
    out: &OutputConfig,
) -> Result<()> {
    match cmd {
        MediaCommands::Upload {
            file,
            media_type,
            category,
            wait,
            auth_type,
            username,
            verbose,
            trace,
            headers,
        } => {
            let mut client = ApiClient::new(cfg, auth);
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
            )
        }
        MediaCommands::Status {
            media_id,
            auth_type,
            username,
            verbose,
            wait,
            trace,
            headers,
        } => {
            let mut client = ApiClient::new(cfg, auth);
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
            )
        }
    }
}
