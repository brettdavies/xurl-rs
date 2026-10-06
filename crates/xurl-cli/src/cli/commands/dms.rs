//! The direct-message commands: `dm` and `dms`.

use serde_json::json;

use super::{
    Run, dry_run_or_validate, effective_limit, make_client, print_typed, resolve_user_id,
    with_flags,
};
use crate::cli::Commands;
use crate::cli::failure::CommandResult;
use xdk::api::shortcuts;

pub(super) async fn run(cmd: Commands, run: Run<'_>) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    match cmd {
        Commands::Dm {
            target_username,
            text,
            common,
        } => {
            let ctx = json!({
                "command": "dm",
                "target_username": target_username,
                "body": text,
            });
            let proceed = dry_run_or_validate(out, stdout, flags.dry_run, ctx, || {
                shortcuts::validate_target_username(&target_username)?;
                shortcuts::validate_dm_body(&text)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let target_id = resolve_user_id(&client, &target_username, &common).await?;
            let response = with_flags(client.send_dm(&target_id, &text), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Dms {
            max_results,
            common,
        } => {
            let n = effective_limit(max_results, flags.global_limit);
            let client = make_client(cfg, auth)?;
            let call = client.get_dm_events(n);
            let response = with_flags(call, &common, flags.cursor.as_deref())
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        _ => unreachable!("run_subcommand routes only the direct-message commands here"),
    }
    Ok(())
}
