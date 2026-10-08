//! The `usage` family: the account's API usage, and its credit breakdown.

use serde_json::json;

use super::{DryRun, Run, make_client, send_or_report, with_flags};
use crate::cli::failure::CommandResult;
use crate::cli::{CommonFlags, UsageCommands};

pub(super) async fn run(
    target: Option<UsageCommands>,
    common: &CommonFlags,
    run: Run<'_>,
) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let dry_run = |command: &str| {
        flags
            .dry_run
            .then(|| DryRun::new(json!({"command": command})))
    };
    let client = make_client(cfg, auth)?;
    match target {
        Some(UsageCommands::Credits { common }) => {
            let call = with_flags(client.get_usage_credits(), &common, None);
            send_or_report(out, stdout, dry_run("usage-credits"), call).await?;
        }
        None => {
            let call = with_flags(client.get_usage(), common, None);
            send_or_report(out, stdout, dry_run("usage"), call).await?;
        }
    }
    Ok(())
}
