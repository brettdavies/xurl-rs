//! The `conversations` family (start a group, send into a conversation, and
//! read one) and the `dm-events` family (read or delete one message).

use serde_json::json;

use super::{
    Run, dry_run_or_validate, handles_lookup_call, list_owned_by, list_paged, make_client,
    print_typed, resolve_account_ids, send_confirmed, send_validated, with_flags,
};
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
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({
        "command": "conversations-start",
        "participants": participants,
        "body": text,
    });
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
        shortcuts::validate_participant_count(participants.len())?;
        shortcuts::validate_accounts(participants)?;
        shortcuts::validate_dm_body(text)
    })?;
    let client = make_client(cfg, auth)?;
    if let Some(dry_run) = dry_run {
        let mut credentials = Vec::new();
        if let Some(lookup) = handles_lookup_call(&client, participants, common) {
            credentials.push(lookup.auth_preflight().await);
        }
        let create = client.create_dm_conversation(participants, text);
        credentials.push(with_flags(create, common, None).auth_preflight().await);
        return dry_run.answer(out, stdout, credentials);
    }
    let ids = resolve_account_ids(&client, participants, common).await?;
    let response = with_flags(client.create_dm_conversation(&ids, text), common, None)
        .send()
        .await?;
    print_typed(out, stdout, &response)?;
    Ok(())
}
