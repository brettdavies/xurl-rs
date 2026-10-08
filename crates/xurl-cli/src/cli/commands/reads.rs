//! The commands that read posts and profiles: `read`, `search`, `whoami`,
//! `user`, `timeline`, and `mentions`.

use serde_json::json;

use super::{DryRun, Run, effective_limit, list_for_user, make_client, send_or_report, with_flags};
use crate::cli::Commands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;

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
        Commands::Whoami { common } => {
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
                .then(|| DryRun::new(json!({"command": "whoami"})));
            let client = make_client(cfg, auth)?;
            let call = with_flags(client.get_me(), &common, None);
            send_or_report(out, stdout, dry_run, call).await?;
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
