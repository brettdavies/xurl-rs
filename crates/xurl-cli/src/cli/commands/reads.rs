//! The commands that read posts and profiles: `read`, `posts`,
//! `users`, `user-posts`, `search`, `search-users`, `search-counts`,
//! `whoami`, `user`, `timeline`, `mentions`, who engaged with a post
//! (`likers`, `reposters`, and `quotes`), and how the caller's posts did
//! (`analytics`).

use serde_json::json;

use super::{
    DryRun, Run, dry_run_or_validate, effective_limit, list_for_post, list_for_user, list_owned_by,
    list_paged, make_client, print_typed, send_or_report, send_validated, with_flags,
};
use crate::cli::Commands;
use crate::cli::failure::CommandResult;
use xdk::api::{AuthPreflight, Client, shortcuts};

pub(super) async fn run(cmd: Commands, run: Run<'_>) -> CommandResult<()> {
    match cmd {
        Commands::Read { post_id, common } => {
            let Run {
                cfg,
                auth,
                flags,
                out,
                stdout,
                ..
            } = run;
            let dry_run = flags
                .dry_run
                .then(|| DryRun::new(json!({"command": "read", "post_id": post_id})));
            let client = make_client(cfg, auth)?;
            let call = with_flags(client.read_post(&post_id), &common, None);
            send_or_report(out, stdout, dry_run, call).await?;
        }
        Commands::Posts { post_ids, common } => {
            let ctx = json!({"command": "posts", "post_ids": post_ids});
            let validator = || {
                shortcuts::validate_lookup_ids(&post_ids)?;
                post_ids
                    .iter()
                    .try_for_each(|id| shortcuts::validate_post_id(id))
            };
            send_validated(run, ctx, validator, &common, |client: &Client| {
                client.read_posts(&post_ids)
            })
            .await?;
        }
        Commands::Users { accounts, common } => {
            let ctx = json!({"command": "users", "accounts": accounts});
            let validate = || shortcuts::validate_user_lookup(&accounts);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                if accounts.iter().all(|a| shortcuts::is_user_id_form(a)) {
                    client.lookup_users_by_id(&accounts)
                } else {
                    client.lookup_users_by_username(&accounts)
                }
            })
            .await?;
        }
        Commands::UserPosts {
            target_username,
            max_results,
            common,
        } => {
            let ctx = json!({"command": "user-posts"});
            let validator = || shortcuts::validate_target_username(&target_username);
            let of = Some(target_username.as_str());
            let shortcut = Client::get_user_posts;
            list_owned_by(run, ctx, validator, max_results, of, &common, shortcut).await?;
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
            let dry_run = flags.dry_run.then(|| {
                let mut ctx = json!({"command": "search", "query": query});
                if let Some(n) = max_results {
                    ctx["max_results"] = json!(n);
                }
                DryRun::new(ctx)
            });
            let n = effective_limit(max_results, flags.global_limit);
            let client = make_client(cfg, auth)?;
            let call = with_flags(
                client.search_posts(&query, n),
                &common,
                flags.cursor.as_deref(),
            );
            send_or_report(out, stdout, dry_run, call).await?;
        }
        Commands::SearchUsers {
            query,
            max_results,
            common,
        } => {
            let ctx = json!({"command": "search-users", "query": query});
            let validator = || shortcuts::validate_user_search_query(&query);
            list_paged(run, ctx, validator, max_results, &common, |client, n| {
                client.search_users(&query, n)
            })
            .await?;
        }
        Commands::SearchCounts {
            query,
            granularity,
            start,
            end,
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
            let ctx = json!({
                "command": "search-counts",
                "query": query,
                "granularity": granularity,
                "start": start,
                "end": end,
            });
            let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
                shortcuts::validate_time_window(start.as_deref(), end.as_deref())
            })?;
            let client = make_client(cfg, auth)?;
            let call = client.count_recent_posts(
                &query,
                granularity.as_deref(),
                start.as_deref(),
                end.as_deref(),
            );
            let call = with_flags(call, &common, flags.cursor.as_deref());
            send_or_report(out, stdout, dry_run, call).await?;
        }
        Commands::Analytics {
            post_ids,
            start,
            end,
            granularity,
            common,
        } => {
            let ctx = json!({
                "command": "analytics",
                "post_ids": post_ids,
                "start": start,
                "end": end,
                "granularity": granularity,
            });
            let validator = || {
                shortcuts::validate_lookup_ids(&post_ids)?;
                post_ids
                    .iter()
                    .try_for_each(|id| shortcuts::validate_post_id(id))?;
                shortcuts::validate_time_window(Some(&start), Some(&end))
            };
            send_validated(run, ctx, validator, &common, |client: &Client| {
                client.post_analytics(&post_ids, &start, &end, granularity.as_deref())
            })
            .await?;
        }
        Commands::Whoami { cached, common } => {
            let Run {
                cfg,
                auth,
                flags,
                out,
                stdout,
                ..
            } = run;
            let client = make_client(cfg, auth)?;
            let call = with_flags(client.get_me(), &common, None);
            if flags.dry_run {
                let credential = call.auth_preflight().await;
                let mut ctx = json!({"command": "whoami"});
                if cached {
                    let stored = matches!(
                        &credential,
                        Ok(Some(found)) if stored_identity(found).is_some()
                    );
                    ctx["cached"] = json!(true);
                    ctx["source"] = json!(if stored { "cache" } else { "api" });
                }
                return DryRun::new(ctx).answer(out, stdout, [credential]);
            }
            if cached
                && let Ok(Some(found)) = call.auth_preflight().await
                && let Some(stored) = stored_identity(&found)
            {
                out.print_response(stdout, &stored);
                return Ok(());
            }
            // The one command a person runs to ask who a login is, so its
            // answer is where a login stored without a name gets one.
            let response = call.send_saving_identity().await?;
            if cached {
                let mut answered =
                    serde_json::to_value(&response).map_err(xdk::error::Error::from)?;
                answered["source"] = json!("api");
                out.print_response(stdout, &answered);
            } else {
                print_typed(out, stdout, &response)?;
            }
        }
        Commands::User {
            target_username,
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
            let dry_run = flags.dry_run.then(|| {
                DryRun::new(json!({"command": "user", "target_username": target_username}))
            });
            let client = make_client(cfg, auth)?;
            let call = with_flags(client.lookup_user(&target_username), &common, None);
            send_or_report(out, stdout, dry_run, call).await?;
        }
        Commands::Timeline {
            max_results,
            common,
        } => {
            let shortcut = Client::get_timeline;
            list_for_user(run, "timeline", max_results, None, &common, shortcut).await?;
        }
        Commands::Mentions {
            max_results,
            common,
        } => {
            let shortcut = Client::get_mentions;
            list_for_user(run, "mentions", max_results, None, &common, shortcut).await?;
        }
        Commands::Likers {
            post_id,
            max_results,
            common,
        } => {
            let shortcut = Client::get_liking_users;
            list_for_post(run, "likers", &post_id, max_results, &common, shortcut).await?;
        }
        Commands::Reposters {
            post_id,
            max_results,
            common,
        } => {
            let shortcut = Client::get_reposted_by;
            list_for_post(run, "reposters", &post_id, max_results, &common, shortcut).await?;
        }
        Commands::Quotes {
            post_id,
            max_results,
            common,
        } => {
            let shortcut = Client::get_quote_posts;
            list_for_post(run, "quotes", &post_id, max_results, &common, shortcut).await?;
        }
        _ => unreachable!("run_subcommand routes only the read commands here"),
    }
    Ok(())
}

/// `whoami`'s answer from the store: the account the login belongs to, when
/// the store holds its id, username, and display name, with `source:
/// "cache"`. `None` sends the caller to X for an account the store does not
/// hold whole.
fn stored_identity(found: &AuthPreflight) -> Option<serde_json::Value> {
    let (Some(id), Some(username), Some(name)) =
        (&found.user_id, &found.account_username, &found.display_name)
    else {
        return None;
    };
    Some(json!({
        "data": {"id": id, "name": name, "username": username},
        "source": "cache",
    }))
}
