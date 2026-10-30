//! `xr broadcasts chat`: a broadcast's chat history.

use serde_json::json;

use super::{Run, list_paged};
use crate::cli::BroadcastChatCommands;
use crate::cli::failure::CommandResult;
use xdk::api::shortcuts;

pub(super) async fn run(action: BroadcastChatCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        BroadcastChatCommands::History {
            broadcast_id,
            max_results,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-chat-history", "broadcast_id": broadcast_id});
            let validate = || shortcuts::validate_broadcast_id(&broadcast_id);
            list_paged(run, ctx, validate, max_results, &common, |client, n| {
                client.get_broadcast_chat(&broadcast_id, n)
            })
            .await
        }
    }
}
