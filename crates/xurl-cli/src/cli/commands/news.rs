//! The `news` family: search X's news stories, and read one by id.

use serde_json::json;

use super::{Run, effective_limit, send_validated};
use crate::cli::NewsCommands;
use crate::cli::failure::CommandResult;
use xdk::api::{Client, shortcuts};

pub(super) async fn run(action: NewsCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        NewsCommands::Search {
            query,
            max_results,
            max_age_hours,
            common,
        } => {
            let ctx = json!({
                "command": "news-search",
                "query": query,
                "max_results": max_results,
                "max_age_hours": max_age_hours,
            });
            let n = effective_limit(max_results, run.flags.global_limit);
            send_validated(
                run,
                ctx,
                || Ok(()),
                &common,
                |client: &Client| client.search_news(&query, n, max_age_hours),
            )
            .await
        }
        NewsCommands::Get { story_id, common } => {
            let ctx = json!({"command": "news-get", "story_id": story_id});
            let validate = || shortcuts::validate_news_id(&story_id);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_news(&story_id)
            })
            .await
        }
    }
}
