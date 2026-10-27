//! `xr media get` and `xr media analytics`: uploads by media key, and their
//! video metrics over a time window.

use serde_json::json;

use super::{Run, send_validated};
use crate::cli::CommonFlags;
use crate::cli::failure::CommandResult;
use xdk::api::{Client, shortcuts};

/// The time window `media analytics` reads.
pub(super) struct Window<'a> {
    pub start: &'a str,
    pub end: &'a str,
    pub granularity: Option<&'a str>,
}

pub(super) async fn get(
    run: Run<'_>,
    media_keys: &[String],
    common: &CommonFlags,
) -> CommandResult<()> {
    let ctx = json!({"command": "media-get", "media_keys": media_keys});
    let validate = || shortcuts::validate_media_keys(media_keys);
    send_validated(run, ctx, validate, common, |client: &Client| {
        client.get_media(media_keys)
    })
    .await
}

pub(super) async fn analytics(
    run: Run<'_>,
    media_keys: &[String],
    window: Window<'_>,
    common: &CommonFlags,
) -> CommandResult<()> {
    let ctx = json!({
        "command": "media-analytics",
        "media_keys": media_keys,
        "start": window.start,
        "end": window.end,
        "granularity": window.granularity,
    });
    let validate = || {
        shortcuts::validate_media_keys(media_keys)?;
        shortcuts::validate_time_window(Some(window.start), Some(window.end))
    };
    send_validated(run, ctx, validate, common, |client: &Client| {
        client.get_media_analytics(media_keys, window.start, window.end, window.granularity)
    })
    .await
}
