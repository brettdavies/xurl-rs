//! The `lists` family: the caller's lists, one list's posts, members, and
//! followers, creating, changing, and deleting a list, and following and
//! pinning one.

use serde_json::json;

use super::{
    Run, act_from_me, act_on_user_with, list_for_user, list_paged, send_confirmed, send_validated,
};
use crate::cli::failure::CommandResult;
use crate::cli::{CommonFlags, ListMembersCommands, ListsCommands};
use serde::Serialize;
use serde::de::DeserializeOwned;
use xdk::api::shortcuts::{self, ListDetails};
use xdk::api::{ApiResponse, Call, Client};

pub(super) async fn run(action: ListsCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        ListsCommands::List {
            max_results,
            of,
            common,
        } => {
            let shortcut = Client::get_owned_lists;
            list_for_user(
                run,
                "lists-list",
                max_results,
                of.as_deref(),
                &common,
                shortcut,
            )
            .await
        }
        ListsCommands::Followed {
            max_results,
            of,
            common,
        } => {
            let shortcut = Client::get_followed_lists;
            list_for_user(
                run,
                "lists-followed",
                max_results,
                of.as_deref(),
                &common,
                shortcut,
            )
            .await
        }
        ListsCommands::Memberships {
            max_results,
            of,
            common,
        } => {
            let shortcut = Client::get_list_memberships;
            list_for_user(
                run,
                "lists-memberships",
                max_results,
                of.as_deref(),
                &common,
                shortcut,
            )
            .await
        }
        ListsCommands::Pinned { common } => {
            list_for_user(run, "lists-pinned", None, None, &common, |client, me, _| {
                client.get_pinned_lists(me)
            })
            .await
        }
        ListsCommands::Get { list_id, common } => {
            let ctx = json!({"command": "lists-get", "list_id": list_id});
            let validate = || shortcuts::validate_list_id(&list_id);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_list(&list_id)
            })
            .await
        }
        ListsCommands::Posts {
            list_id,
            max_results,
            common,
        } => {
            let ctx = json!({"command": "lists-posts", "list_id": list_id});
            let validate = || shortcuts::validate_list_id(&list_id);
            list_paged(run, ctx, validate, max_results, &common, |client, n| {
                client.get_list_posts(&list_id, n)
            })
            .await
        }
        ListsCommands::Followers {
            list_id,
            max_results,
            common,
        } => {
            let ctx = json!({"command": "lists-followers", "list_id": list_id});
            let validate = || shortcuts::validate_list_id(&list_id);
            list_paged(run, ctx, validate, max_results, &common, |client, n| {
                client.get_list_followers(&list_id, n)
            })
            .await
        }
        ListsCommands::Members { action } => members(action, run).await,
        ListsCommands::Add {
            name,
            description,
            private,
            common,
        } => {
            let details = ListDetails {
                name: Some(name),
                description,
                private: private.then_some(true),
            };
            let ctx = json!({"command": "lists-add", "list": details});
            let validate = || {
                shortcuts::validate_new_list(
                    details.name.as_deref().unwrap_or_default(),
                    details.description.as_deref(),
                )
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.create_list(&details)
            })
            .await
        }
        ListsCommands::Update {
            list_id,
            name,
            description,
            private,
            public,
            common,
        } => {
            let changes = ListDetails {
                name,
                description,
                private: (private || public).then_some(private),
            };
            let ctx = json!({"command": "lists-update", "list_id": list_id, "changes": changes});
            let validate = || {
                shortcuts::validate_list_id(&list_id)?;
                shortcuts::validate_list_changes(&changes)
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.update_list(&list_id, &changes)
            })
            .await
        }
        ListsCommands::Remove {
            list_id,
            force,
            common,
        } => {
            let ctx = json!({"command": "lists-remove", "list_id": list_id});
            let prompt = format!("Delete list {list_id}?");
            let validate = || shortcuts::validate_list_id(&list_id);
            send_confirmed(run, ctx, &prompt, force, validate, &common, |client| {
                client.delete_list(&list_id)
            })
            .await
        }
        ListsCommands::Follow { list_id, common } => {
            from_me(run, "lists-follow", &list_id, &common, Client::follow_list).await
        }
        ListsCommands::Unfollow { list_id, common } => {
            from_me(
                run,
                "lists-unfollow",
                &list_id,
                &common,
                Client::unfollow_list,
            )
            .await
        }
        ListsCommands::Pin { list_id, common } => {
            from_me(run, "lists-pin", &list_id, &common, Client::pin_list).await
        }
        ListsCommands::Unpin { list_id, common } => {
            from_me(run, "lists-unpin", &list_id, &common, Client::unpin_list).await
        }
    }
}

async fn members(action: ListMembersCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        ListMembersCommands::List {
            list_id,
            max_results,
            common,
        } => {
            let ctx = json!({"command": "lists-members-list", "list_id": list_id});
            let validate = || shortcuts::validate_list_id(&list_id);
            list_paged(run, ctx, validate, max_results, &common, |client, n| {
                client.get_list_members(&list_id, n)
            })
            .await
        }
        ListMembersCommands::Add {
            list_id,
            target_username,
            common,
        } => {
            let ctx = json!({"command": "lists-members-add", "list_id": list_id});
            let validate = || shortcuts::validate_list_id(&list_id);
            act_on_user_with(
                run,
                ctx,
                validate,
                &target_username,
                &common,
                |client, id| client.add_list_member(&list_id, id),
            )
            .await
        }
        ListMembersCommands::Remove {
            list_id,
            target_username,
            common,
        } => {
            let ctx = json!({"command": "lists-members-remove", "list_id": list_id});
            let validate = || shortcuts::validate_list_id(&list_id);
            act_on_user_with(
                run,
                ctx,
                validate,
                &target_username,
                &common,
                |client, id| client.remove_list_member(&list_id, id),
            )
            .await
        }
    }
}

/// Follows, unfollows, pins, or unpins a list from the caller's account.
async fn from_me<T>(
    run: Run<'_>,
    command: &str,
    list_id: &str,
    common: &CommonFlags,
    shortcut: impl FnOnce(&Client, &str, &str) -> Call<ApiResponse<T>>,
) -> CommandResult<()>
where
    T: Serialize + DeserializeOwned + Default,
{
    let ctx = json!({"command": command, "list_id": list_id});
    let validate = || shortcuts::validate_list_id(list_id);
    act_from_me(run, ctx, validate, common, |client, me| {
        shortcut(client, me, list_id)
    })
    .await
}
