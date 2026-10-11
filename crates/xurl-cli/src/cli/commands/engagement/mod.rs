//! The commands that act on a post from the caller's account, and the lists
//! of what the caller engaged with: `like`, `unlike`, `repost`, `unrepost`,
//! `bookmark`, `unbookmark`, `bookmarks`, `bookmark-folders`, and `likes`.

use super::{Run, act_from_me_on_post, list_for_user};
use crate::cli::Commands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;

mod bookmarks;

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
            all,
            max_pages,
            folder,
            expand,
            until_known,
            archive,
            common,
        } => {
            bookmarks::run_bookmarks(
                run,
                max_results,
                all,
                max_pages,
                folder.as_deref(),
                &expand,
                until_known.as_deref(),
                archive.as_deref(),
                &common,
            )
            .await?;
        }
        Commands::BookmarkFolders {
            max_results,
            all,
            common,
        } => {
            bookmarks::run_bookmark_folders(run, max_results, all, &common).await?;
        }
        Commands::Likes {
            max_results,
            common,
        } => list_for_user(run, max_results, None, &common, Client::get_liked_posts).await?,
        _ => unreachable!("run_subcommand routes only the engagement commands here"),
    }
    Ok(())
}
