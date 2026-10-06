//! The `usage` family: the account's API usage, and its credit breakdown.

use super::{Run, make_client, print_typed, with_flags};
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
        out,
        stdout,
        ..
    } = run;
    let client = make_client(cfg, auth)?;
    match target {
        Some(UsageCommands::Credits { common }) => {
            let response = with_flags(client.get_usage_credits(), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        None => {
            let response = with_flags(client.get_usage(), common, None).send().await?;
            print_typed(out, stdout, &response)?;
        }
    }
    Ok(())
}
