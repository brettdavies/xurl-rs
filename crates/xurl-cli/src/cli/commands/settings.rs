//! `xr settings`: the caller's account settings.

use serde_json::json;

use super::{Run, request_context, send_validated};
use crate::cli::failure::CommandResult;
use crate::cli::{AllowDmsFrom, AllowMediaTagging, SettingsArgs, SettingsCommands};
use xdk::api::Client;
use xdk::api::shortcuts::{self, DmsFrom, MediaTagging, QuietHours, SettingsChanges};

pub(super) async fn run(action: SettingsCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        SettingsCommands::Get { common } => {
            let ctx = json!({"command": "settings-get"});
            send_validated(run, ctx, || Ok(()), &common, Client::get_settings).await
        }
        SettingsCommands::Update { changes, common } => {
            let changes = settings_changes(changes);
            let ctx = request_context("settings-update", &changes);
            let validate = || shortcuts::validate_settings_changes(&changes);
            send_validated(run, ctx, validate, &common, |client| {
                client.update_settings(&changes)
            })
            .await
        }
    }
}

fn settings_changes(args: SettingsArgs) -> SettingsChanges {
    let quiet_hours = QuietHours {
        enabled: args.quiet_hours_enabled,
        start_hour: args.quiet_hours_start,
        end_hour: args.quiet_hours_end,
    };
    SettingsChanges {
        allow_dms_from: args.allow_dms_from.map(|who| match who {
            AllowDmsFrom::All => DmsFrom::All,
            AllowDmsFrom::Following => DmsFrom::Following,
            AllowDmsFrom::Verified => DmsFrom::Verified,
        }),
        allow_media_tagging: args.allow_media_tagging.map(|who| match who {
            AllowMediaTagging::All => MediaTagging::All,
            AllowMediaTagging::Following => MediaTagging::Following,
            AllowMediaTagging::None => MediaTagging::None,
        }),
        always_allow_dms_from_subscribers: args.always_allow_dms_from_subscribers,
        discoverable_by_email: args.discoverable_by_email,
        discoverable_by_phone: args.discoverable_by_phone,
        display_sensitive_media: args.display_sensitive_media,
        language: args.language,
        location_tagging_enabled: args.location_tagging_enabled,
        protected: args.protected,
        quiet_hours: (!quiet_hours.is_empty()).then_some(quiet_hours),
        time_zone: args.time_zone,
        use_cookie_personalization: args.use_cookie_personalization,
    }
}
