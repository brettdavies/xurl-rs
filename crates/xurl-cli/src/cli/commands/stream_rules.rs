//! The `stream rules` family: list, add, remove, and count the rules that
//! pick which posts the filtered stream delivers.

use serde_json::json;

use super::{Run, list_paged, send_confirmed, send_validated};
use crate::cli::failure::CommandResult;
use crate::cli::{StreamCommands, StreamRulesCommands};
use xdk::api::{Client, shortcuts};

pub(super) async fn run(action: StreamCommands, run: Run<'_>) -> CommandResult<()> {
    let StreamCommands::Rules { action } = action;
    match action {
        StreamRulesCommands::List {
            max_results,
            common,
        } => {
            let ctx = json!({"command": "stream-rules-list"});
            list_paged(
                run,
                ctx,
                || Ok(()),
                max_results,
                &common,
                |client, n| client.get_stream_rules(n),
            )
            .await
        }
        StreamRulesCommands::Add {
            value,
            tag,
            validate_only,
            common,
        } => {
            let ctx = json!({
                "command": "stream-rules-add",
                "value": value,
                "tag": tag,
                "validate_only": validate_only,
            });
            send_validated(
                run,
                ctx,
                || Ok(()),
                &common,
                |client: &Client| client.add_stream_rule(&value, tag.as_deref(), validate_only),
            )
            .await
        }
        StreamRulesCommands::Remove {
            rule_ids,
            force,
            validate_only,
            common,
        } => {
            let ctx = json!({
                "command": "stream-rules-remove",
                "rule_ids": rule_ids,
                "validate_only": validate_only,
            });
            let validate = || shortcuts::validate_rule_ids(&rule_ids);
            let build = |client: &Client| client.remove_stream_rules(&rule_ids, validate_only);
            if validate_only {
                return send_validated(run, ctx, validate, &common, build).await;
            }
            let prompt = match rule_ids.as_slice() {
                [id] => format!("Remove filtered-stream rule {id}?"),
                ids => format!("Remove {} filtered-stream rules?", ids.len()),
            };
            send_confirmed(run, ctx, &prompt, force, validate, &common, build).await
        }
        StreamRulesCommands::Counts { common } => {
            let ctx = json!({"command": "stream-rules-counts"});
            send_validated(
                run,
                ctx,
                || Ok(()),
                &common,
                |client: &Client| client.get_stream_rule_counts(),
            )
            .await
        }
    }
}
