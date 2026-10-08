//! The social-graph commands: `follow`, `unfollow`, `following`,
//! `followers`, `mute`, `unmute`, `muted`, `block`, `unblock`, and `blocked`.

use super::{Run, act_from_me_on_user, list_for_user};
use crate::cli::Commands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;

pub(super) async fn run(cmd: Commands, run: Run<'_>) -> CommandResult<()> {
    match cmd {
        Commands::Follow {
            target_username,
            common,
        } => {
            let shortcut = Client::follow_user;
            act_from_me_on_user(run, "follow", &target_username, &common, shortcut).await?;
        }
        Commands::Unfollow {
            target_username,
            common,
        } => {
            let shortcut = Client::unfollow_user;
            act_from_me_on_user(run, "unfollow", &target_username, &common, shortcut).await?;
        }
        Commands::Following {
            max_results,
            of,
            common,
        } => {
            let shortcut = Client::get_following;
            list_for_user(
                run,
                "following",
                max_results,
                of.as_deref(),
                &common,
                shortcut,
            )
            .await?;
        }
        Commands::Followers {
            max_results,
            of,
            common,
        } => {
            let shortcut = Client::get_followers;
            list_for_user(
                run,
                "followers",
                max_results,
                of.as_deref(),
                &common,
                shortcut,
            )
            .await?;
        }
        Commands::Mute {
            target_username,
            common,
        } => {
            let shortcut = Client::mute_user;
            act_from_me_on_user(run, "mute", &target_username, &common, shortcut).await?;
        }
        Commands::Unmute {
            target_username,
            common,
        } => {
            let shortcut = Client::unmute_user;
            act_from_me_on_user(run, "unmute", &target_username, &common, shortcut).await?;
        }
        Commands::Muted {
            max_results,
            common,
        } => list_for_user(run, "muted", max_results, None, &common, Client::get_muted).await?,
        Commands::Block {
            target_username,
            common,
        } => {
            let shortcut = Client::block_user;
            act_from_me_on_user(run, "block", &target_username, &common, shortcut).await?;
        }
        Commands::Unblock {
            target_username,
            common,
        } => {
            let shortcut = Client::unblock_user;
            act_from_me_on_user(run, "unblock", &target_username, &common, shortcut).await?;
        }
        Commands::Blocked {
            max_results,
            common,
        } => {
            list_for_user(
                run,
                "blocked",
                max_results,
                None,
                &common,
                Client::get_blocked,
            )
            .await?
        }
        _ => unreachable!("run_subcommand routes only the social-graph commands here"),
    }
    Ok(())
}
