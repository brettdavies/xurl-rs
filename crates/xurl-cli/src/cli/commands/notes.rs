//! The `notes` family over X's AI Note Writer API: write a Community Note,
//! score a draft, delete one, and list the posts eligible for a note and the
//! notes the caller wrote.

use serde_json::json;

use super::{Run, list_paged, send_confirmed, send_validated};
use crate::cli::NotesCommands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;
use xdk::api::shortcuts::{self, NewNote};

pub(super) async fn run(action: NotesCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        NotesCommands::Write {
            post,
            text,
            classification,
            tags,
            trustworthy_sources,
            media_note,
            test,
            common,
        } => {
            let note = NewNote {
                post,
                text,
                classification,
                misleading_tags: tags,
                trustworthy_sources,
                is_media_note: media_note.then_some(true),
            };
            let ctx = json!({
                "command": "notes-write",
                "post": note.post,
                "note": note,
                "test_mode": test,
            });
            let validate = || validate_draft(&note.post, &note.text);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.create_note(&note, test)
            })
            .await
        }
        NotesCommands::Evaluate { post, text, common } => {
            let ctx = json!({"command": "notes-evaluate", "post": post, "text": text});
            let validate = || validate_draft(&post, &text);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.evaluate_note(&post, &text)
            })
            .await
        }
        NotesCommands::Eligible {
            max_results,
            test,
            common,
        } => {
            let ctx = json!({"command": "notes-eligible", "test_mode": test});
            list_paged(
                run,
                ctx,
                || Ok(()),
                max_results,
                &common,
                |client, n| client.get_posts_eligible_for_notes(test, n),
            )
            .await
        }
        NotesCommands::Written {
            max_results,
            test,
            common,
        } => {
            let ctx = json!({"command": "notes-written", "test_mode": test});
            list_paged(
                run,
                ctx,
                || Ok(()),
                max_results,
                &common,
                |client, n| client.get_notes_written(test, n),
            )
            .await
        }
        NotesCommands::Remove {
            note_id,
            force,
            common,
        } => {
            let ctx = json!({"command": "notes-remove", "note_id": note_id});
            let prompt = format!("Delete Community Note {note_id}?");
            let validate = || shortcuts::validate_note_id(&note_id);
            send_confirmed(run, ctx, &prompt, force, validate, &common, |client| {
                client.delete_note(&note_id)
            })
            .await
        }
    }
}

/// Checks a note's post and text, for writing it or scoring the draft: X
/// writes no note without a source URL, so a draft without one is not worth
/// a request to score.
fn validate_draft(post: &str, text: &str) -> Result<(), &'static str> {
    shortcuts::validate_post_id(post)?;
    shortcuts::validate_note_text(text)
}
