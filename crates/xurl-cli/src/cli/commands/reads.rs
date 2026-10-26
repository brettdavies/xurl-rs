//! The commands that read posts and profiles: `read`, `search`, `whoami`,
//! `user`, `timeline`, and `mentions`.

use serde_json::json;

use super::{
    DryRun, Run, effective_limit, list_for_user, make_client, print_typed, send_or_report,
    with_flags,
};
use crate::cli::Commands;
use crate::cli::failure::CommandResult;
use xdk::api::{AuthPreflight, Client};

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
