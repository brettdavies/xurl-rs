//! The `spaces` family: find X Spaces by keyword, by id, or by the accounts
//! that created them, and read the posts shared in one.

use serde_json::json;

use super::{Run, effective_limit, list_paged, send_to_accounts, send_validated};
use crate::cli::failure::CommandResult;
use crate::cli::{CommonFlags, SpacesCommands};
use xdk::api::{Client, shortcuts};

pub(super) async fn run(action: SpacesCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        SpacesCommands::Search {
            query,
            state,
            max_results,
            common,
        } => {
            let ctx = json!({
                "command": "spaces-search",
                "query": query,
                "state": state,
                "max_results": max_results,
            });
            let n = effective_limit(max_results, run.flags.global_limit);
            send_validated(
                run,
                ctx,
                || Ok(()),
                &common,
                |client: &Client| client.search_spaces(&query, state.as_deref(), n),
            )
            .await
        }
        SpacesCommands::Get { space_ids, common } => {
            let ctx = json!({"command": "spaces-get", "space_ids": space_ids});
            let validate = || shortcuts::validate_space_ids(&space_ids);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_spaces(&space_ids)
            })
            .await
        }
        SpacesCommands::ByCreator { accounts, common } => by_creator(run, &accounts, &common).await,
        SpacesCommands::Posts {
            space_id,
            max_results,
            common,
        } => {
            let ctx = json!({"command": "spaces-posts", "space_id": space_id});
            let validate = || shortcuts::validate_space_id(&space_id);
            list_paged(run, ctx, validate, max_results, &common, |client, n| {
                client.get_space_posts(&space_id, n)
            })
            .await
        }
    }
}

/// `xr spaces by-creator`: numeric ids go out as they are, and handles
/// resolve to ids first, every one of them in one lookup.
async fn by_creator(run: Run<'_>, accounts: &[String], common: &CommonFlags) -> CommandResult<()> {
    let ctx = json!({"command": "spaces-by-creator", "accounts": accounts});
    let validate = || shortcuts::validate_accounts(accounts);
    send_to_accounts(run, ctx, validate, accounts, common, |client, ids| {
        client.get_spaces_by_creator(ids)
    })
    .await
}
