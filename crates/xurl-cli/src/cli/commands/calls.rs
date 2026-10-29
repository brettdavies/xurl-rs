//! The `calls` family: an X call, and the calls on the caller's calendar.

use serde::Serialize;
use serde_json::{Value, json};

use super::{Run, send_confirmed, send_to_accounts, send_validated, stand_in_ids};
use crate::cli::failure::CommandResult;
use crate::cli::{
    CallAnswer, CallDetailsArgs, CallScope, CallsCommands, OccurrenceArgs,
    SCHEDULED_CALLS_PAGE_SIZE, ScheduledCallsCommands,
};
use xdk::api::shortcuts::{
    self, NewScheduledCall, OccurrenceScope, Occurrences, RsvpAnswer, ScheduledCallChanges,
};

pub(super) async fn run(target: CallsCommands, run: Run<'_>) -> CommandResult<()> {
    match target {
        CallsCommands::Get { call_id, common } => {
            let ctx = json!({"command": "calls-get", "call_id": call_id});
            send_validated(
                run,
                ctx,
                || Ok(()),
                &common,
                |client| client.get_call(&call_id),
            )
            .await
        }
        CallsCommands::Scheduled { action } => scheduled(action, run).await,
    }
}

async fn scheduled(action: ScheduledCallsCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        ScheduledCallsCommands::List {
            start,
            end,
            max_results,
            common,
        } => {
            let max_results = max_results
                .or(run.flags.global_limit)
                .unwrap_or(SCHEDULED_CALLS_PAGE_SIZE);
            let ctx = json!({
                "command": "calls-scheduled-list",
                "start_time": start,
                "end_time": end,
                "max_results": max_results,
            });
            let validate = || {
                [&start, &end]
                    .into_iter()
                    .flatten()
                    .try_for_each(|time| shortcuts::validate_call_time(time))
            };
            send_validated(run, ctx, validate, &common, |client| {
                client.get_scheduled_calls(start.as_deref(), end.as_deref(), max_results)
            })
            .await
        }
        ScheduledCallsCommands::Get {
            scheduled_call_id,
            common,
        } => {
            let ctx = json!({
                "command": "calls-scheduled-get",
                "scheduled_call_id": scheduled_call_id,
            });
            send_validated(
                run,
                ctx,
                || Ok(()),
                &common,
                |client| client.get_scheduled_call(&scheduled_call_id),
            )
            .await
        }
        ScheduledCallsCommands::Add {
            start,
            details,
            common,
        } => {
            let call = new_call(start, details);
            let ctx = context("calls-scheduled-add", None, &call);
            let invitees = call.invitee_ids.clone();
            let inviting = |ids: &[String]| NewScheduledCall {
                invitee_ids: ids.to_vec(),
                ..call.clone()
            };
            let validate = || {
                shortcuts::validate_new_scheduled_call(&inviting(&stand_in_ids(&invitees)))?;
                validate_invitees(&invitees)
            };
            send_to_accounts(run, ctx, validate, &invitees, &common, |client, ids| {
                client.schedule_call(&inviting(ids))
            })
            .await
        }
        ScheduledCallsCommands::Update {
            scheduled_call_id,
            start,
            details,
            occurrences,
            common,
        } => {
            let changes = changes(start, details, occurrences);
            let ctx = context("calls-scheduled-update", Some(&scheduled_call_id), &changes);
            let invitees = changes.invitee_ids.clone().unwrap_or_default();
            let inviting = |ids: &[String]| ScheduledCallChanges {
                invitee_ids: changes.invitee_ids.as_ref().map(|_| ids.to_vec()),
                ..changes.clone()
            };
            let validate = || {
                shortcuts::validate_scheduled_call_changes(&inviting(&stand_in_ids(&invitees)))?;
                validate_invitees(&invitees)
            };
            send_to_accounts(run, ctx, validate, &invitees, &common, |client, ids| {
                client.update_scheduled_call(&scheduled_call_id, &inviting(ids))
            })
            .await
        }
        ScheduledCallsCommands::Cancel {
            scheduled_call_id,
            occurrences,
            no_notify,
            force,
            common,
        } => {
            let occurrences = occurrences_of(occurrences);
            let notify = no_notify.then_some(false);
            let mut ctx = context(
                "calls-scheduled-cancel",
                Some(&scheduled_call_id),
                &occurrences,
            );
            if let Some(notify) = notify {
                ctx["notify_invitees"] = json!(notify);
            }
            let prompt = format!("Cancel scheduled call {scheduled_call_id}?");
            send_confirmed(
                run,
                ctx,
                &prompt,
                force,
                || shortcuts::validate_occurrences(&occurrences),
                &common,
                |client| client.cancel_scheduled_call(&scheduled_call_id, &occurrences, notify),
            )
            .await
        }
        ScheduledCallsCommands::Rsvp {
            scheduled_call_id,
            answer,
            occurrences,
            common,
        } => {
            let answer = rsvp_answer(answer);
            let occurrences = occurrences_of(occurrences);
            let mut ctx = context(
                "calls-scheduled-rsvp",
                Some(&scheduled_call_id),
                &occurrences,
            );
            ctx["response"] = json!(answer.wire_name());
            send_validated(
                run,
                ctx,
                || shortcuts::validate_occurrences(&occurrences),
                &common,
                |client| client.respond_to_scheduled_call(&scheduled_call_id, answer, &occurrences),
            )
            .await
        }
    }
}

/// Checks the form of each invitee, a handle or a numeric user id; the
/// call's own validator checks how many there are and each id, with every
/// handle standing in for the id its lookup returns.
fn validate_invitees(invitees: &[String]) -> Result<(), &'static str> {
    if invitees.is_empty() {
        return Ok(());
    }
    shortcuts::validate_accounts(invitees)
}

/// A dry run's context: the command, the scheduled call it names, and each
/// field the request sends, as the request sends it.
fn context(command: &str, scheduled_call_id: Option<&str>, request: &impl Serialize) -> Value {
    let mut ctx = json!({"command": command});
    if let Some(id) = scheduled_call_id {
        ctx["scheduled_call_id"] = json!(id);
    }
    if let (Some(ctx), Ok(Value::Object(fields))) =
        (ctx.as_object_mut(), serde_json::to_value(request))
    {
        ctx.extend(fields);
    }
    ctx
}

fn new_call(start: String, details: CallDetailsArgs) -> NewScheduledCall {
    NewScheduledCall {
        start_time: start,
        title: details.title,
        description: details.description,
        duration_minutes: details.duration,
        invitee_ids: details.invite,
        recurrence: details.recurrence,
        time_zone: details.time_zone,
        notify_invitees: details.no_notify.then_some(false),
    }
}

fn changes(
    start: Option<String>,
    details: CallDetailsArgs,
    occurrences: OccurrenceArgs,
) -> ScheduledCallChanges {
    ScheduledCallChanges {
        start_time: start,
        title: details.title,
        description: details.description,
        duration_minutes: details.duration,
        invitee_ids: (!details.invite.is_empty()).then_some(details.invite),
        recurrence: details.recurrence,
        time_zone: details.time_zone,
        notify_invitees: details.no_notify.then_some(false),
        occurrences: occurrences_of(occurrences),
    }
}

fn occurrences_of(args: OccurrenceArgs) -> Occurrences {
    Occurrences {
        scope: args.scope.map(|scope| match scope {
            CallScope::All => OccurrenceScope::All,
            CallScope::This => OccurrenceScope::This,
            CallScope::ThisAndFollowing => OccurrenceScope::ThisAndFollowing,
        }),
        occurrence_start_time: args.occurrence,
    }
}

const fn rsvp_answer(answer: CallAnswer) -> RsvpAnswer {
    match answer {
        CallAnswer::Accepted => RsvpAnswer::Accepted,
        CallAnswer::Declined => RsvpAnswer::Declined,
        CallAnswer::Tentative => RsvpAnswer::Tentative,
    }
}
