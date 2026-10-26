//! `xr broadcasts chat`: a broadcast's chat, sending into it, removing a
//! message, and muting someone.

use serde_json::json;

use super::{Run, act_on_user_with, list_paged, send_confirmed, send_validated};
use crate::cli::BroadcastChatCommands;
use crate::cli::failure::CommandResult;
use xdk::api::{Client, shortcuts};

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
        BroadcastChatCommands::Send {
            broadcast_id,
            text,
            reply_to,
            common,
        } => {
            let ctx = json!({
                "command": "broadcasts-chat-send",
                "broadcast_id": broadcast_id,
                "body": text,
                "reply_to": reply_to,
            });
            let validate = || {
                shortcuts::validate_broadcast_id(&broadcast_id)?;
                shortcuts::validate_chat_message(&text)?;
                reply_to
                    .as_deref()
                    .map_or(Ok(()), shortcuts::validate_chat_message_id)
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.send_broadcast_chat(&broadcast_id, &text, reply_to.as_deref())
            })
            .await
        }
        BroadcastChatCommands::Remove {
            broadcast_id,
            message_id,
            force,
            common,
        } => {
            let ctx = json!({
                "command": "broadcasts-chat-remove",
                "broadcast_id": broadcast_id,
                "message_id": message_id,
            });
            let prompt = format!("Remove chat message {message_id}?");
            let validate = || {
                shortcuts::validate_broadcast_id(&broadcast_id)?;
                shortcuts::validate_chat_message_id(&message_id)
            };
            send_confirmed(run, ctx, &prompt, force, validate, &common, |client| {
                client.delete_chat_message(&broadcast_id, &message_id)
            })
            .await
        }
        BroadcastChatCommands::Mute {
            broadcast_id,
            target_username,
            until,
            message,
            common,
        } => {
            let ctx = json!({
                "command": "broadcasts-chat-mute",
                "broadcast_id": broadcast_id,
                "until": until,
                "message_id": message,
            });
            let validate = || {
                shortcuts::validate_broadcast_id(&broadcast_id)?;
                shortcuts::validate_time_window(until.as_deref(), None)?;
                message
                    .as_deref()
                    .map_or(Ok(()), shortcuts::validate_chat_message_id)
            };
            act_on_user_with(
                run,
                ctx,
                validate,
                &target_username,
                &common,
                |client, id| {
                    client.mute_chat_user(&broadcast_id, id, until.as_deref(), message.as_deref())
                },
            )
            .await
        }
        BroadcastChatCommands::Unmute {
            broadcast_id,
            target_username,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-chat-unmute", "broadcast_id": broadcast_id});
            let validate = || shortcuts::validate_broadcast_id(&broadcast_id);
            act_on_user_with(
                run,
                ctx,
                validate,
                &target_username,
                &common,
                |client, id| client.unmute_chat_user(&broadcast_id, id),
            )
            .await
        }
    }
}
