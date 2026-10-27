//! The `bookmark-folders` family: the folders the caller sorts bookmarks
//! into, and the posts in each.

use serde_json::json;

use super::{Run, act_from_me, list_for_user, list_owned_by};
use crate::cli::BookmarkFoldersCommands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;
use xdk::api::shortcuts;

pub(super) async fn run(action: BookmarkFoldersCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        BookmarkFoldersCommands::List {
            max_results,
            common,
        } => {
            let shortcut = Client::get_bookmark_folders;
            list_for_user(
                run,
                "bookmark-folders-list",
                max_results,
                None,
                &common,
                shortcut,
            )
            .await
        }
        BookmarkFoldersCommands::Posts {
            folder_id,
            max_results,
            common,
        } => {
            let ctx = json!({"command": "bookmark-folders-posts", "folder_id": folder_id});
            let validate = || shortcuts::validate_bookmark_folder_id(&folder_id);
            list_owned_by(
                run,
                ctx,
                validate,
                max_results,
                None,
                &common,
                |client, my_id, n| client.get_bookmark_folder_posts(my_id, &folder_id, n),
            )
            .await
        }
        BookmarkFoldersCommands::Add { name, common } => {
            let ctx = json!({"command": "bookmark-folders-add", "name": name});
            let validate = || shortcuts::validate_bookmark_folder_name(&name);
            act_from_me(run, ctx, validate, &common, |client, my_id| {
                client.create_bookmark_folder(my_id, &name)
            })
            .await
        }
    }
}
