//! The commands that read posts and profiles: `read`, `search`, `whoami`,
//! `user`, `timeline`, and `mentions`.

use super::{Run, effective_limit, list_for_user, make_client, print_typed, with_flags};
use crate::cli::Commands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;

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
        Commands::Whoami { common } => {
            let Run {
                cfg,
                auth,
                out,
                stdout,
                ..
            } = run;
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.get_me(), &common, None).send().await?;
            print_typed(out, stdout, &response)?;
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
