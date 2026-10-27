//! The `communities` family: search X's communities, and read one by id.

use serde_json::json;

use super::{Run, list_paged, send_validated};
use crate::cli::CommunitiesCommands;
use crate::cli::failure::CommandResult;
use xdk::api::{Client, shortcuts};

pub(super) async fn run(action: CommunitiesCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        CommunitiesCommands::Search {
            query,
            max_results,
            common,
        } => {
            let ctx = json!({"command": "communities-search", "query": query});
            list_paged(
                run,
                ctx,
                || Ok(()),
                max_results,
                &common,
                |client, n| client.search_communities(&query, n),
            )
            .await
        }
        CommunitiesCommands::Get {
            community_id,
            common,
        } => {
            let ctx = json!({"command": "communities-get", "community_id": community_id});
            let validate = || shortcuts::validate_community_id(&community_id);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_community(&community_id)
            })
            .await
        }
    }
}
