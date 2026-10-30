//! `xr broadcasts scheduled`: the caller's scheduled broadcasts, scheduling
//! one, moving it, and cancelling it. X publishes each one at its start time.

use serde_json::json;

use super::{Run, list_paged, send_confirmed, send_validated};
use crate::cli::ScheduledBroadcastsCommands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;
use xdk::api::shortcuts::{self, NewScheduledBroadcast, ScheduledBroadcastChanges};

pub(super) async fn run(action: ScheduledBroadcastsCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        ScheduledBroadcastsCommands::List {
            max_results,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-scheduled-list"});
            let shortcut = Client::list_scheduled_broadcasts;
            list_paged(run, ctx, || Ok(()), max_results, &common, shortcut).await
        }
        ScheduledBroadcastsCommands::Get {
            scheduled_id,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-scheduled-get", "scheduled_id": scheduled_id});
            let validate = || shortcuts::validate_scheduled_broadcast_id(&scheduled_id);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_scheduled_broadcast(&scheduled_id)
            })
            .await
        }
        ScheduledBroadcastsCommands::Add {
            source,
            start,
            end,
            title,
            description,
            replay,
            common,
        } => {
            let broadcast = NewScheduledBroadcast {
                source_id: source,
                start_time: start,
                end_time: end,
                title,
                description,
                available_for_replay: replay.then_some(true),
                manual_publish: None,
            };
            let ctx = json!({
                "command": "broadcasts-scheduled-add",
                "source": broadcast.source_id,
                "start": broadcast.start_time,
                "end": broadcast.end_time,
                "title": broadcast.title,
            });
            let validate = || {
                shortcuts::validate_time_window(
                    Some(&broadcast.start_time),
                    broadcast.end_time.as_deref(),
                )
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.create_scheduled_broadcast(&broadcast)
            })
            .await
        }
        ScheduledBroadcastsCommands::Update {
            scheduled_id,
            start,
            end,
            source,
            title,
            description,
            replay,
            common,
        } => {
            let changes = ScheduledBroadcastChanges {
                start_time: start,
                end_time: end,
                source_id: source,
                title,
                description,
                available_for_replay: replay.then_some(true),
                manual_publish: None,
            };
            let ctx = json!({
                "command": "broadcasts-scheduled-update",
                "scheduled_id": scheduled_id,
                "start": changes.start_time,
                "end": changes.end_time,
            });
            let validate = || {
                shortcuts::validate_scheduled_broadcast_id(&scheduled_id)?;
                shortcuts::validate_time_window(Some(&changes.start_time), Some(&changes.end_time))
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.update_scheduled_broadcast(&scheduled_id, &changes)
            })
            .await
        }
        ScheduledBroadcastsCommands::Cancel {
            scheduled_id,
            force,
            common,
        } => {
            let ctx =
                json!({"command": "broadcasts-scheduled-cancel", "scheduled_id": scheduled_id});
            let prompt = format!("Cancel scheduled broadcast {scheduled_id}?");
            let validate = || shortcuts::validate_scheduled_broadcast_id(&scheduled_id);
            send_confirmed(run, ctx, &prompt, force, validate, &common, |client| {
                client.delete_scheduled_broadcast(&scheduled_id)
            })
            .await
        }
    }
}
