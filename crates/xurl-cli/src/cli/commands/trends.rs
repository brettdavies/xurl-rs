//! The `trends` family: the trends for a place, and the caller's
//! personalized trends.

use serde_json::json;

use super::{Run, send_validated};
use crate::cli::TrendsCommands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;

pub(super) async fn run(action: TrendsCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        TrendsCommands::Place { woeid, max, common } => {
            let ctx = json!({"command": "trends-place", "woeid": woeid, "max": max});
            send_validated(
                run,
                ctx,
                || Ok(()),
                &common,
                |client: &Client| client.get_trends(woeid, max),
            )
            .await
        }
        TrendsCommands::Personal { common } => {
            let ctx = json!({"command": "trends-personal"});
            send_validated(
                run,
                ctx,
                || Ok(()),
                &common,
                Client::get_personalized_trends,
            )
            .await
        }
    }
}
