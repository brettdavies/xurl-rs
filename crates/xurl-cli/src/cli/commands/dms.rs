//! The direct-message commands: `dm`, `dms`, `dm-block`, and `dm-unblock`.

use serde_json::json;

use super::{Run, act_on_user, list_mine, send_to_account, validate_account};
use crate::cli::Commands;
use crate::cli::failure::CommandResult;
use xdk::api::{Client, shortcuts};

pub(super) async fn run(cmd: Commands, run: Run<'_>) -> CommandResult<()> {
    match cmd {
        Commands::Dm {
            target_username,
            text,
            common,
        } => {
            let ctx = json!({
                "command": "dm",
                "target_username": target_username,
                "body": text,
            });
            let validate = || {
                validate_account(&target_username)?;
                shortcuts::validate_dm_body(&text)
            };
            send_to_account(
                run,
                ctx,
                validate,
                &target_username,
                &common,
                |client, id| client.send_dm(id, &text),
            )
            .await?;
        }
        Commands::Dms {
            max_results,
            common,
        } => list_mine(run, "dms", max_results, &common, Client::get_dm_events).await?,
        Commands::DmBlock {
            target_username,
            common,
        } => {
            act_on_user(
                run,
                "dm-block",
                &target_username,
                &common,
                Client::block_dms,
            )
            .await?
        }
        Commands::DmUnblock {
            target_username,
            common,
        } => {
            let shortcut = Client::unblock_dms;
            act_on_user(run, "dm-unblock", &target_username, &common, shortcut).await?;
        }
        _ => unreachable!("run_subcommand routes only the direct-message commands here"),
    }
    Ok(())
}
