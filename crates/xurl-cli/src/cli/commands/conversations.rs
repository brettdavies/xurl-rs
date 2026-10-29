//! The `conversations` family (start a group, send into a conversation, and
//! read one) and the `dm-events` family (read or delete one message).

use serde_json::json;

use super::{Run, list_owned_by, list_paged, send_confirmed, send_to_accounts, send_validated};
use crate::cli::failure::CommandResult;
use crate::cli::{CommonFlags, ConversationsCommands, DmEventsCommands};
use xdk::api::{Client, shortcuts};

pub(super) async fn run(action: ConversationsCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        ConversationsCommands::Start {
            participants,
            text,
            common,
        } => start(run, &participants, &text, &common).await,
        ConversationsCommands::Send {
            conversation_id,
            text,
            common,
        } => {
            let ctx = json!({
                "command": "conversations-send",
                "conversation_id": conversation_id,
                "body": text,
            });
            let validate = || {
                shortcuts::validate_conversation_id(&conversation_id)?;
                shortcuts::validate_dm_body(&text)
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.send_to_conversation(&conversation_id, &text)
            })
            .await
        }
        ConversationsCommands::History {
            conversation_id,
            max_results,
            common,
        } => {
            let ctx = json!({
                "command": "conversations-history",
                "conversation_id": conversation_id,
            });
            let validate = || shortcuts::validate_conversation_id(&conversation_id);
            list_paged(run, ctx, validate, max_results, &common, |client, n| {
                client.get_conversation_events(&conversation_id, n)
            })
            .await
        }
        ConversationsCommands::With {
            target_username,
            max_results,
            common,
        } => {
            let ctx = json!({"command": "conversations-with"});
            let validate = || shortcuts::validate_target_username(&target_username);
            let of = Some(target_username.as_str());
            let shortcut = Client::get_dm_events_with;
            list_owned_by(run, ctx, validate, max_results, of, &common, shortcut).await
        }
    }
}

pub(super) async fn dm_events(action: DmEventsCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        DmEventsCommands::Get { event_id, common } => {
            let ctx = json!({"command": "dm-events-get", "event_id": event_id});
            let validate = || shortcuts::validate_dm_event_id(&event_id);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_dm_event(&event_id)
            })
            .await
        }
        DmEventsCommands::Remove {
            event_id,
            force,
            common,
        } => {
            let ctx = json!({"command": "dm-events-remove", "event_id": event_id});
            let prompt = format!("Delete direct message {event_id}?");
            let validate = || shortcuts::validate_dm_event_id(&event_id);
            send_confirmed(run, ctx, &prompt, force, validate, &common, |client| {
                client.delete_dm_event(&event_id)
            })
            .await
        }
    }
}

/// Starts a group conversation. A participant given as a numeric user id
/// goes out as it is; every handle is looked up first, all in one request.
async fn start(
    run: Run<'_>,
    participants: &[String],
    text: &str,
    common: &CommonFlags,
) -> CommandResult<()> {
    let ctx = json!({
        "command": "conversations-start",
        "participants": participants,
        "body": text,
    });
    let validate = || {
        shortcuts::validate_participant_count(participants.len())?;
        shortcuts::validate_accounts(participants)?;
        shortcuts::validate_dm_body(text)
    };
    send_to_accounts(run, ctx, validate, participants, common, |client, ids| {
        client.create_dm_conversation(ids, text)
    })
    .await
}
