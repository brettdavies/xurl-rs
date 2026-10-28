//! `xr profile`: the caller's display name, bio, website, and location,
//! and their profile image and banner.

use serde_json::json;

use super::{Run, request_context, send_confirmed, send_validated};
use crate::cli::failure::CommandResult;
use crate::cli::{ProfileBannerCommands, ProfileCommands, ProfileImageCommands};
use xdk::api::Client;
use xdk::api::shortcuts::{self, ProfileChanges};

pub(super) async fn run(action: ProfileCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        ProfileCommands::Update {
            name,
            description,
            url,
            location,
            common,
        } => {
            let changes = ProfileChanges {
                name,
                description,
                url,
                location,
            };
            let ctx = request_context("profile-update", &changes);
            let validate = || shortcuts::validate_profile_changes(&changes);
            send_validated(run, ctx, validate, &common, |client| {
                client.update_profile(&changes)
            })
            .await
        }
        ProfileCommands::Image {
            action: ProfileImageCommands::Set { media_id, common },
        } => {
            let ctx = json!({"command": "profile-image-set", "media_id": media_id});
            let validate = || shortcuts::validate_media_id(&media_id);
            send_validated(run, ctx, validate, &common, |client| {
                client.update_profile_image(&media_id)
            })
            .await
        }
        ProfileCommands::Banner { action } => banner(action, run).await,
    }
}

async fn banner(action: ProfileBannerCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        ProfileBannerCommands::Set {
            media_id,
            crop,
            common,
        } => {
            let ctx = json!({"command": "profile-banner-set", "media_id": media_id, "crop": crop});
            let validate = || {
                shortcuts::validate_media_id(&media_id)?;
                crop.as_ref()
                    .map_or(Ok(()), shortcuts::validate_banner_crop)
            };
            send_validated(run, ctx, validate, &common, |client| {
                client.update_profile_banner(&media_id, crop)
            })
            .await
        }
        ProfileBannerCommands::Remove { force, common } => {
            let ctx = json!({"command": "profile-banner-remove"});
            send_confirmed(
                run,
                ctx,
                "Remove your profile banner?",
                force,
                || Ok(()),
                &common,
                Client::remove_profile_banner,
            )
            .await
        }
    }
}
