//! The `broadcasts` family: the caller's broadcasts, their chat moderators,
//! the people muted in their chats, and the RTMP and RTMPS stream sources
//! the caller broadcasts from, with the ingest region to create one in. The scheduled broadcasts and a broadcast's
//! chat have modules of their own.

use serde_json::json;

use super::{
    DryRun, Run, act_on_user, act_on_user_with, broadcast_chat, broadcast_scheduled, list_paged,
    make_client, send_confirmed, send_or_report, send_validated, with_flags,
};
use crate::cli::failure::CommandResult;
use crate::cli::{BroadcastsCommands, ModeratorsCommands, MutesCommands, SourcesCommands};
use xdk::api::Client;
use xdk::api::shortcuts;

pub(super) async fn run(target: BroadcastsCommands, run: Run<'_>) -> CommandResult<()> {
    match target {
        BroadcastsCommands::Moderators { action } => match action {
            ModeratorsCommands::List { common } => {
                let Run {
                    cfg,
                    auth,
                    flags,
                    out,
                    stdout,
                    ..
                } = run;
                let dry_run = flags
                    .dry_run
                    .then(|| DryRun::new(json!({"command": "broadcasts-moderators-list"})));
                let client = make_client(cfg, auth)?;
                let call = with_flags(client.get_chat_moderators(), &common, None);
                send_or_report(out, stdout, dry_run, call).await?;
            }
            ModeratorsCommands::Add {
                target_username,
                common,
            } => {
                act_on_user(
                    run,
                    "broadcasts-moderators-add",
                    &target_username,
                    &common,
                    Client::add_chat_moderator,
                )
                .await?;
            }
            ModeratorsCommands::Remove {
                target_username,
                common,
            } => {
                act_on_user(
                    run,
                    "broadcasts-moderators-remove",
                    &target_username,
                    &common,
                    Client::remove_chat_moderator,
                )
                .await?;
            }
        },
        BroadcastsCommands::Mutes { action } => mutes(action, run).await?,
        BroadcastsCommands::Sources { action } => sources(action, run).await?,
        BroadcastsCommands::List {
            max_results,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-list"});
            list_paged(
                run,
                ctx,
                || Ok(()),
                max_results,
                &common,
                Client::list_broadcasts,
            )
            .await?;
        }
        BroadcastsCommands::Get {
            broadcast_id,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-get", "broadcast_id": broadcast_id});
            let validate = || shortcuts::validate_broadcast_id(&broadcast_id);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_broadcast(&broadcast_id)
            })
            .await?;
        }
        BroadcastsCommands::End {
            broadcast_id,
            force,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-end", "broadcast_id": broadcast_id});
            let prompt = format!("End broadcast {broadcast_id}?");
            let validate = || shortcuts::validate_broadcast_id(&broadcast_id);
            send_confirmed(run, ctx, &prompt, force, validate, &common, |client| {
                client.end_broadcast(&broadcast_id)
            })
            .await?;
        }
        BroadcastsCommands::Remove {
            broadcast_id,
            force,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-remove", "broadcast_id": broadcast_id});
            let prompt = format!("Delete broadcast {broadcast_id} and its replay?");
            let validate = || shortcuts::validate_broadcast_id(&broadcast_id);
            send_confirmed(run, ctx, &prompt, force, validate, &common, |client| {
                client.delete_broadcast(&broadcast_id)
            })
            .await?;
        }
        BroadcastsCommands::Scheduled { action } => broadcast_scheduled::run(action, run).await?,
        BroadcastsCommands::Chat { action } => broadcast_chat::run(action, run).await?,
    }
    Ok(())
}

async fn mutes(action: MutesCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        MutesCommands::List {
            max_results,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-mutes-list"});
            list_paged(
                run,
                ctx,
                || Ok(()),
                max_results,
                &common,
                Client::get_chat_mutes,
            )
            .await
        }
        MutesCommands::Add {
            target_username,
            until,
            broadcast,
            message,
            common,
        } => {
            let ctx = json!({
                "command": "broadcasts-mutes-add",
                "broadcast_id": broadcast,
                "until": until,
                "message_id": message,
            });
            let validate = || {
                shortcuts::validate_time_window(until.as_deref(), None)?;
                broadcast
                    .as_deref()
                    .map_or(Ok(()), shortcuts::validate_broadcast_id)?;
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
                    client.add_chat_mute(
                        id,
                        broadcast.as_deref(),
                        until.as_deref(),
                        message.as_deref(),
                    )
                },
            )
            .await
        }
        MutesCommands::Remove {
            target_username,
            broadcast,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-mutes-remove", "broadcast_id": broadcast});
            let validate = || {
                broadcast
                    .as_deref()
                    .map_or(Ok(()), shortcuts::validate_broadcast_id)
            };
            act_on_user_with(
                run,
                ctx,
                validate,
                &target_username,
                &common,
                |client, id| client.remove_chat_mute(id, broadcast.as_deref()),
            )
            .await
        }
    }
}

async fn sources(action: SourcesCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        SourcesCommands::List { common } => {
            let ctx = json!({"command": "broadcasts-sources-list"});
            send_validated(run, ctx, || Ok(()), &common, Client::get_stream_sources).await
        }
        SourcesCommands::Region { common } => {
            let ctx = json!({"command": "broadcasts-sources-region"});
            send_validated(run, ctx, || Ok(()), &common, Client::get_region).await
        }
        SourcesCommands::Get { source_id, common } => {
            let ctx = json!({"command": "broadcasts-sources-get", "source_id": source_id});
            send_validated(
                run,
                ctx,
                || Ok(()),
                &common,
                |client| client.get_stream_source(&source_id),
            )
            .await
        }
        SourcesCommands::Add {
            name,
            region,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-sources-add", "name": name, "region": region});
            let validate = || {
                shortcuts::validate_source_name(&name)?;
                shortcuts::validate_source_region(&region)
            };
            send_validated(run, ctx, validate, &common, |client| {
                client.create_stream_source(&name, &region)
            })
            .await
        }
        SourcesCommands::Rename {
            source_id,
            name,
            common,
        } => {
            let ctx = json!({
                "command": "broadcasts-sources-rename",
                "source_id": source_id,
                "name": name,
            });
            let validate = || shortcuts::validate_source_name(&name);
            send_validated(run, ctx, validate, &common, |client| {
                client.rename_stream_source(&source_id, &name)
            })
            .await
        }
        SourcesCommands::Remove {
            source_id,
            force,
            common,
        } => {
            let ctx = json!({"command": "broadcasts-sources-remove", "source_id": source_id});
            let prompt = format!("Delete stream source {source_id}?");
            send_confirmed(
                run,
                ctx,
                &prompt,
                force,
                || Ok(()),
                &common,
                |client| client.delete_stream_source(&source_id),
            )
            .await
        }
    }
}
