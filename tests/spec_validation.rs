//! Spec-as-test validation — verifies typed response structs can deserialize
//! example API responses derived from the X API v2 documentation.
//!
//! When the X API adds fields, unknown fields land silently in `extra` (R8).
//! When the X API changes a field type or removes a required field, these
//! tests fail with a clear message naming the type and field.

use serde_json::Value;

use xurl::api::response::types::{
    ApiResponse, BlockingResult, BookmarkedResult, DeletedResult, DmEvent, FollowingResult,
    LikedResult, MediaUploadResponse, MutingResult, RetweetedResult, Tweet, UsageData, User,
};

/// Loads the cached example responses fixture.
fn load_examples() -> Value {
    let json_str =
        std::fs::read_to_string("tests/fixtures/openapi/example_responses.json").unwrap();
    serde_json::from_str(&json_str).unwrap()
}

#[test]
fn spec_tweet_single() {
    let examples = load_examples();
    let resp: ApiResponse<Tweet> =
        serde_json::from_value(examples["tweet_single"].clone()).unwrap();
    assert_eq!(resp.data.id, "1346889436626259968");
    assert!(resp.data.created_at.is_some());
    assert!(resp.data.public_metrics.is_some());
    assert!(resp.includes.is_some());
}

#[test]
fn spec_tweet_list() {
    let examples = load_examples();
    let resp: ApiResponse<Vec<Tweet>> =
        serde_json::from_value(examples["tweet_list"].clone()).unwrap();
    assert_eq!(resp.data.len(), 2);
    assert_eq!(resp.data[0].id, "1346889436626259968");
    assert!(resp.meta.is_some());
    assert_eq!(resp.meta.unwrap().result_count, Some(2));
}

#[test]
fn spec_user_single() {
    let examples = load_examples();
    let resp: ApiResponse<User> = serde_json::from_value(examples["user_single"].clone()).unwrap();
    assert_eq!(resp.data.id, "2244994945");
    assert_eq!(resp.data.username, "TwitterDev");
    assert!(resp.data.description.is_some());
    assert!(resp.data.public_metrics.is_some());
}

#[test]
fn spec_action_liked() {
    let examples = load_examples();
    let resp: ApiResponse<LikedResult> =
        serde_json::from_value(examples["action_liked"].clone()).unwrap();
    assert!(resp.data.liked);
}

#[test]
fn spec_action_following() {
    let examples = load_examples();
    let resp: ApiResponse<FollowingResult> =
        serde_json::from_value(examples["action_following"].clone()).unwrap();
    assert!(resp.data.following);
    // Extra field "pending_follow" should be captured
    assert_eq!(resp.data.extra["pending_follow"], false);
}

#[test]
fn spec_action_deleted() {
    let examples = load_examples();
    let resp: ApiResponse<DeletedResult> =
        serde_json::from_value(examples["action_deleted"].clone()).unwrap();
    assert!(resp.data.deleted);
}

#[test]
fn spec_action_retweeted() {
    let examples = load_examples();
    let resp: ApiResponse<RetweetedResult> =
        serde_json::from_value(examples["action_retweeted"].clone()).unwrap();
    assert!(resp.data.retweeted);
}

#[test]
fn spec_action_bookmarked() {
    let examples = load_examples();
    let resp: ApiResponse<BookmarkedResult> =
        serde_json::from_value(examples["action_bookmarked"].clone()).unwrap();
    assert!(resp.data.bookmarked);
}

#[test]
fn spec_action_blocking() {
    let examples = load_examples();
    let resp: ApiResponse<BlockingResult> =
        serde_json::from_value(examples["action_blocking"].clone()).unwrap();
    assert!(resp.data.blocking);
}

#[test]
fn spec_action_muting() {
    let examples = load_examples();
    let resp: ApiResponse<MutingResult> =
        serde_json::from_value(examples["action_muting"].clone()).unwrap();
    assert!(resp.data.muting);
}

#[test]
fn spec_dm_event() {
    let examples = load_examples();
    let resp: ApiResponse<DmEvent> = serde_json::from_value(examples["dm_event"].clone()).unwrap();
    assert_eq!(resp.data.id, "1580705921830768647");
    assert_eq!(resp.data.text.as_deref(), Some("hello there"));
    assert!(resp.data.sender_id.is_some());
}

#[test]
fn spec_dm_event_list() {
    let examples = load_examples();
    let resp: ApiResponse<Vec<DmEvent>> =
        serde_json::from_value(examples["dm_event_list"].clone()).unwrap();
    assert_eq!(resp.data.len(), 1);
    assert!(resp.includes.is_some());
}

#[test]
fn spec_media_upload_init() {
    let examples = load_examples();
    let resp: ApiResponse<MediaUploadResponse> =
        serde_json::from_value(examples["media_upload_init"].clone()).unwrap();
    assert_eq!(resp.data.id, "1455952740635586562");
    assert!(resp.data.media_key.is_some());
    assert!(resp.data.expires_after_secs.is_some());
}

#[test]
fn spec_media_upload_status() {
    let examples = load_examples();
    let resp: ApiResponse<MediaUploadResponse> =
        serde_json::from_value(examples["media_upload_status"].clone()).unwrap();
    let info = resp.data.processing_info.unwrap();
    assert_eq!(info.state, "in_progress");
    assert_eq!(info.check_after_secs, Some(5));
    assert_eq!(info.progress_percent, Some(45));
}

#[test]
fn spec_usage() {
    let examples = load_examples();
    let resp: ApiResponse<UsageData> = serde_json::from_value(examples["usage"].clone()).unwrap();
    assert_eq!(resp.data.project_cap.as_deref(), Some("2000000"));
    assert_eq!(resp.data.cap_reset_day, Some(19));
    assert!(resp.data.daily_project_usage.is_some());
    assert!(resp.data.daily_client_app_usage.is_some());
}
