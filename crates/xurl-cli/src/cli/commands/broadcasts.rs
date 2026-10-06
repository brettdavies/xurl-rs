//! The `broadcasts` family: the caller's broadcast chat moderators.

use super::{Run, act_on_user, make_client, print_typed, with_flags};
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
                    out,
                    stdout,
                    ..
                } = run;
                let client = make_client(cfg, auth)?;
                let response = with_flags(client.get_chat_moderators(), &common, None)
                    .send()
                    .await?;
                print_typed(out, stdout, &response)?;
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
