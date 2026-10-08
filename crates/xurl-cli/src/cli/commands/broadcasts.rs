//! The `broadcasts` family: the caller's broadcast chat moderators.

use serde_json::json;

use super::{DryRun, Run, act_on_user, make_client, send_or_report, with_flags};
use crate::cli::failure::CommandResult;
use crate::cli::{BroadcastsCommands, ModeratorsCommands};
use xdk::api::Client;

pub(super) async fn run(target: BroadcastsCommands, run: Run<'_>) -> CommandResult<()> {
    match target {
        BroadcastsCommands::Moderators { action } => match action {
            ModeratorsCommands::List { common } => {
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
                    .then(|| DryRun::new(json!({"command": "broadcasts-moderators-list"})));
                let client = make_client(cfg, auth)?;
                let call = with_flags(client.get_chat_moderators(), &common, None);
                send_or_report(out, stdout, dry_run, call).await?;
            }
            ModeratorsCommands::Add {
                target_username,
                common,
            } => {
                act_on_user(
                    run,
                    "broadcasts-moderators-add",
                    &target_username,
                    &common,
                    Client::add_chat_moderator,
                )
                .await?;
            }
            ModeratorsCommands::Remove {
                target_username,
                common,
            } => {
                act_on_user(
                    run,
                    "broadcasts-moderators-remove",
                    &target_username,
                    &common,
                    Client::remove_chat_moderator,
                )
                .await?;
            }
        },
    }
    Ok(())
}
