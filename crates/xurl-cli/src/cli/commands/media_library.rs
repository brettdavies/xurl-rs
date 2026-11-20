//! `xr media library`: adding uploads to the media library, changing and
//! removing them, and the categories a monetization names.

use serde_json::{Value, json};

use super::{Run, send_confirmed, send_validated};
use crate::cli::failure::CommandResult;
use crate::cli::{LibraryCommands, LibraryItemArgs};
use xdk::api::Client;
use xdk::api::shortcuts::{
    LibraryItemDetails, LibraryMonetization, validate_language_tag, validate_library_changes,
    validate_library_details, validate_media_key,
};

/// `Some(items)` when there is at least one, so an unset repeated flag is not
/// sent.
fn some_if_any<T>(items: Vec<T>) -> Option<Vec<T>> {
    (!items.is_empty()).then_some(items)
}

impl LibraryItemArgs {
    /// The details the flags name; a monetization setting is carried only when
    /// a monetization flag was given.
    fn details(self) -> LibraryItemDetails {
        let monetization = LibraryMonetization {
            enabled: match (self.monetize, self.no_monetize) {
                (true, _) => Some(true),
                (_, true) => Some(false),
                _ => None,
            },
            video_category_ids: some_if_any(self.video_category),
            program_id: self.program,
            allowed_countries: some_if_any(self.country),
            blocked_advertiser_ids: some_if_any(self.block_advertiser),
            blocked_advertiser_category_ids: some_if_any(self.block_advertiser_category),
        };
        LibraryItemDetails {
            name: self.name,
            label: self.label,
            title: self.title,
            description: self.description,
            monetization: (monetization != LibraryMonetization::default()).then_some(monetization),
        }
    }
}

/// The dry-run context of a command that names `media_key` and its details.
fn item_context(command: &str, media_key: &str, details: &LibraryItemDetails) -> Value {
    let mut ctx = serde_json::to_value(details).unwrap_or_else(|_| json!({}));
    ctx["command"] = json!(command);
    ctx["media_key"] = json!(media_key);
    ctx
}

pub(super) async fn run(action: LibraryCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        LibraryCommands::Add {
            media_key,
            item,
            common,
        } => {
            let details = item.details();
            let ctx = item_context("media-library-add", &media_key, &details);
            let validate = || {
                validate_media_key(&media_key)?;
                validate_library_details(&details)
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.add_media_to_library(&media_key, &details)
            })
            .await
        }
        LibraryCommands::Update {
            media_key,
            item,
            common,
        } => {
            let details = item.details();
            let ctx = item_context("media-library-update", &media_key, &details);
            let validate = || {
                validate_media_key(&media_key)?;
                validate_library_changes(&details)
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.update_media_in_library(&media_key, &details)
            })
            .await
        }
        LibraryCommands::Remove {
            media_key,
            force,
            common,
        } => {
            let ctx = json!({"command": "media-library-remove", "media_key": media_key});
            let prompt = format!("Remove {media_key} from your media library?");
            let validate = || validate_media_key(&media_key);
            send_confirmed(run, ctx, &prompt, force, validate, &common, |client| {
                client.remove_media_from_library(&media_key)
            })
            .await
        }
        LibraryCommands::Categories { lang, common } => {
            let ctx = json!({"command": "media-library-categories", "lang": lang});
            let validate = || lang.as_deref().map_or(Ok(()), validate_language_tag);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_media_library_categories(lang.as_deref())
            })
            .await
        }
    }
}
