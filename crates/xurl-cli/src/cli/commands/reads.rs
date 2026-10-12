//! The commands that read posts and profiles: `read`, `search`, `whoami`,
//! `user`, `timeline`, and `mentions`.

use super::{Run, effective_limit, list_for_user, make_client, print_typed, with_flags};
use crate::cli::Commands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;
use xdk::error::Error;

pub(super) async fn run(cmd: Commands, run: Run<'_>) -> CommandResult<()> {
    match cmd {
        Commands::Read { post_id, common } => {
            let Run {
                cfg,
                auth,
                out,
                stdout,
                ..
            } = run;
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.read_post(&post_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Search {
            query,
            max_results,
            common,
        } => {
            let Run {
                cfg,
                auth,
                flags,
                out,
                stdout,
                ..
            } = run;
            let n = effective_limit(max_results, flags.global_limit);
            let client = make_client(cfg, auth)?;
            let call = client.search_posts(&query, n);
            let response = with_flags(call, &common, flags.cursor.as_deref())
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Whoami { refresh, common } => {
            let Run {
                cfg,
                auth,
                out,
                stdout,
                ..
            } = run;

            // When --refresh is not set and we have a cached identity, return it
            // without making an API call
            if !refresh {
                if let Some(token) = auth.token_store.get_first_oauth2_token() {
                    if let Some(oauth2) = &token.oauth2 {
                        if let (Some(user_id), Some(username)) = (
                            &oauth2.user_id,
                            auth.token_store.get_oauth2_usernames().first(),
                        ) {
                            // Build a minimal User response from cache
                            let cached_user = serde_json::json!({
                                "data": {
                                    "id": user_id,
                                    "username": username,
                                    "name": oauth2.display_name.as_deref().unwrap_or(""),
                                },
                                "source": "cache"
                            });
                            out.print_success(stdout, &cached_user);
                            return Ok(());
                        }
                    }
                }
            }

            // Either --refresh was set or cache is unavailable; make the API call
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.get_me(), &common, None).send().await?;

            // Inject source field to indicate it came from API
            let value = serde_json::to_value(&response)
                .map_err(|e| Error::validation(format!("failed to serialize response: {e}")))?;
            let mut value_map = value.as_object().cloned().unwrap_or_default();
            value_map.insert(
                "source".to_string(),
                serde_json::Value::String("api".to_string()),
            );

            out.print_success(stdout, &serde_json::Value::Object(value_map));
        }
        Commands::User {
            target_username,
            common,
        } => {
            let Run {
                cfg,
                auth,
                out,
                stdout,
                ..
            } = run;
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.lookup_user(&target_username), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Timeline {
            max_results,
            common,
        } => list_for_user(run, max_results, None, &common, Client::get_timeline).await?,
        Commands::Mentions {
            max_results,
            common,
        } => list_for_user(run, max_results, None, &common, Client::get_mentions).await?,
        _ => unreachable!("run_subcommand routes only the read commands here"),
    }
    Ok(())
}
