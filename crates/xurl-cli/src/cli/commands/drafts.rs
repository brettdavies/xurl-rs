//! `xr drafts`: listing the caller's post drafts, saving one, and removing
//! one.

use serde_json::json;

use super::{
    Invalid, Run, list_mine, send_confirmed, send_validated, valid_post_id, without_unset,
};
use crate::cli::DraftsCommands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;
use xdk::api::shortcuts::{validate_draft_id, validate_media_attachments, validate_post_body};

/// The refusal of `draft_id` under `rule`, naming it unless it is empty,
/// which has nothing to name.
fn refused_id(rule: &'static str, draft_id: &str) -> Invalid {
    if draft_id.trim().is_empty() {
        Invalid::from(rule)
    } else {
        Invalid::naming(rule, draft_id)
    }
}

pub(super) async fn run(action: DraftsCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        DraftsCommands::List {
            max_results,
            common,
        } => {
            list_mine(
                run,
                "drafts-list",
                max_results,
                &common,
                Client::get_post_drafts,
            )
            .await
        }
        DraftsCommands::Add {
            text,
            media_ids,
            reply_to,
            common,
        } => {
            let ctx = without_unset(json!({
                "command": "drafts-add",
                "body": text,
                "media_ids": media_ids,
                "reply_to": reply_to,
            }));
            let validate = || -> Result<(), Invalid> {
                if let Some(reply_to) = &reply_to {
                    valid_post_id(reply_to)?;
                }
                validate_post_body(&text)?;
                validate_media_attachments(&media_ids)?;
                Ok(())
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.create_post_draft(&text, &media_ids, reply_to.as_deref())
            })
            .await
        }
        DraftsCommands::Remove {
            draft_id,
            force,
            common,
        } => {
            let ctx = json!({"command": "drafts-remove", "draft_id": draft_id});
            let prompt = format!("Delete draft {draft_id}?");
            let validate =
                || validate_draft_id(&draft_id).map_err(|rule| refused_id(rule, &draft_id));
            send_confirmed(run, ctx, &prompt, force, validate, &common, |client| {
                client.delete_post_draft(&draft_id)
            })
            .await
        }
    }
}
