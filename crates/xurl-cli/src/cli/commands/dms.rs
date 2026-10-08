//! The direct-message commands: `dm` and `dms`.

use serde_json::json;

use super::{
    DryRun, Run, UNRESOLVED_ID, dry_run_or_validate, effective_limit, make_client, print_typed,
    resolve_user_id, send_or_report, user_id_call, with_flags,
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
                dry_run.answer(out, stdout, credentials);
                return Ok(());
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
        } => {
            let dry_run = flags.dry_run.then(|| {
                let mut ctx = json!({"command": "dms"});
                if let Some(n) = max_results {
                    ctx["max_results"] = json!(n);
                }
                DryRun::new(ctx)
            });
            let n = effective_limit(max_results, flags.global_limit);
            let client = make_client(cfg, auth)?;
            let call = with_flags(client.get_dm_events(n), &common, flags.cursor.as_deref());
            send_or_report(out, stdout, dry_run, call).await?;
        }
        _ => unreachable!("run_subcommand routes only the direct-message commands here"),
    }
    Ok(())
}
