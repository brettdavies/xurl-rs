//! The commands that act on a post from the caller's account, and the lists
//! of what the caller engaged with: `like`, `unlike`, `repost`, `unrepost`,
//! `bookmark`, `unbookmark`, `bookmarks`, and `likes`.

use super::{Run, act_from_me_on_post, list_for_user};
use crate::cli::Commands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;

pub(super) async fn run(cmd: Commands, run: Run<'_>) -> CommandResult<()> {
    match cmd {
        Commands::Like { post_id, common } => {
            act_from_me_on_post(run, "like", &post_id, &common, Client::like_post).await?;
        }
        Commands::Unlike { post_id, common } => {
            act_from_me_on_post(run, "unlike", &post_id, &common, Client::unlike_post).await?;
        }
        Commands::Repost { post_id, common } => {
            act_from_me_on_post(run, "repost", &post_id, &common, Client::repost).await?;
        }
        Commands::Unrepost { post_id, common } => {
            act_from_me_on_post(run, "unrepost", &post_id, &common, Client::unrepost).await?;
        }
        Commands::Bookmark { post_id, common } => {
            act_from_me_on_post(run, "bookmark", &post_id, &common, Client::bookmark).await?;
        }
        Commands::Unbookmark { post_id, common } => {
            act_from_me_on_post(run, "unbookmark", &post_id, &common, Client::unbookmark).await?;
        }
        Commands::Bookmarks {
            max_results,
            common,
        } => {
            let shortcut = Client::get_bookmarks;
            list_for_user(run, "bookmarks", max_results, None, &common, shortcut).await?;
        }
        Commands::Likes {
            max_results,
            common,
        } => {
            let shortcut = Client::get_liked_posts;
            list_for_user(run, "likes", max_results, None, &common, shortcut).await?;
        }
        _ => unreachable!("run_subcommand routes only the engagement commands here"),
    }
    Ok(())
}
