//! The direct-message commands: `dm`, `dms`, `dm-block`, and `dm-unblock`.

use serde_json::json;

use super::{
    Run, UNRESOLVED_ID, act_on_user, dry_run_or_validate, list_mine, make_client, print_typed,
    resolve_user_id, user_id_call, with_flags,
};
use crate::cli::Commands;
use crate::cli::failure::CommandResult;
use xdk::api::{Client, shortcuts};

pub(super) async fn run(cmd: Commands, run: Run<'_>) -> CommandResult<()> {
    match cmd {
        Commands::Dm {
            target_username,
            text,
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
                "command": "dm",
                "target_username": target_username,
                "body": text,
            });
            let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
                shortcuts::validate_target_username(&target_username)?;
                shortcuts::validate_dm_body(&text)
            })?;
            let client = make_client(cfg, auth)?;
            if let Some(dry_run) = dry_run {
                let send = client.send_dm(UNRESOLVED_ID, &text);
                let credentials = [
                    user_id_call(&client, &target_username, &common)
                        .auth_preflight()
                        .await,
                    with_flags(send, &common, None).auth_preflight().await,
                ];
                return dry_run.answer(out, stdout, credentials);
            }
            let target_id = resolve_user_id(&client, &target_username, &common).await?;
            let response = with_flags(client.send_dm(&target_id, &text), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Dms {
            max_results,
            common,
        } => list_mine(run, "dms", max_results, &common, Client::get_dm_events).await?,
        Commands::DmBlock {
            target_username,
            common,
        } => {
            act_on_user(
                run,
                "dm-block",
                &target_username,
                &common,
                Client::block_dms,
            )
            .await?
        }
        Commands::DmUnblock {
            target_username,
            common,
        } => {
            let shortcut = Client::unblock_dms;
            act_on_user(run, "dm-unblock", &target_username, &common, shortcut).await?;
        }
        _ => unreachable!("run_subcommand routes only the direct-message commands here"),
    }
    Ok(())
}
