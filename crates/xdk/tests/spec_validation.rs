//! Spec-as-test validation — verifies typed response structs can deserialize
//! example API responses derived from the X API v2 documentation, and that
//! every such fixture is the shape the vendored spec gives the endpoint it
//! answers.
//!
//! When the X API adds fields, unknown fields land silently in `extra` (R8).
//! When the X API changes a field type or removes a required field, these
//! tests fail with a clear message naming the type and field. When the spec
//! changes a field's type inside a schema that still exists, the fixture
//! walk at the bottom fails naming the fixture, the endpoint, and the field.

mod common;

use std::collections::BTreeMap;

use common::{ReplyKind, check, fixture_schema, load_spec};
use serde_json::Value;
use xdk::api::auth_matrix::{Endpoint, endpoints};

use xdk::api::response::types::{
    ApiError, ApiResponse, BlockingResult, BookmarkedResult, ChatModeratorsResult, DeletedResult,
    DmEvent, DmSentResult, FollowingResult, LikedResult, MediaUploadResponse, MutingResult, Post,
    RepostedResult, UsageCreditsData, UsageData, User,
};

/// Loads the cached example responses fixture.
fn load_examples() -> Value {
    let json_str =
        std::fs::read_to_string("tests/fixtures/openapi/example_responses.json").unwrap();
    serde_json::from_str(&json_str).unwrap()
}

#[test]
fn spec_post_single() {
    let examples = load_examples();
    let resp: ApiResponse<Post> = serde_json::from_value(examples["post_single"].clone()).unwrap();
    assert_eq!(resp.data.id, "1346889436626259968");
    assert!(resp.data.created_at.is_some());
    assert!(resp.data.public_metrics.is_some());
    assert!(resp.includes.is_some());
}

#[test]
fn spec_post_list() {
    let examples = load_examples();
    let resp: ApiResponse<Vec<Post>> =
        serde_json::from_value(examples["post_list"].clone()).unwrap();
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
fn spec_user_single_wire_reads_tweet_count_into_post_count() {
    let examples = load_examples();
    let resp: ApiResponse<User> = serde_json::from_value(examples["user_single_wire"].clone())
        .expect("wire-shaped user response must deserialize");
    let metrics = resp
        .data
        .public_metrics
        .as_ref()
        .expect("user.public_metrics must be present");
    assert_eq!(metrics.post_count, 30972);
    assert!(
        !metrics.extra.contains_key("tweet_count"),
        "tweet_count must be consumed by post_count, not kept in extra"
    );
    assert_eq!(metrics.extra["like_count"], 40451);
    assert_eq!(metrics.extra["media_count"], 2974);

    let out = serde_json::to_value(&resp).expect("typed response must serialize");
    let out_metrics = &out["data"]["public_metrics"];
    assert_eq!(out_metrics["post_count"], 30972);
    assert!(out_metrics.get("tweet_count").is_none());
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
    let resp: ApiResponse<RepostedResult> =
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
fn spec_dm_sent() {
    let examples = load_examples();
    let resp: ApiResponse<DmSentResult> =
        serde_json::from_value(examples["dm_sent"].clone()).unwrap();
    assert_eq!(resp.data.dm_event_id, "1580705921830768647");
    assert_eq!(resp.data.dm_conversation_id, "1580705921830768643");
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
fn spec_media_upload_append() {
    // The library does not type the append reply; `upload_chunks` sends the
    // next chunk on any 2xx. The fixture exists so the mock answers the phase
    // the way the spec documents it.
    let examples = load_examples();
    assert!(examples["media_upload_append"]["data"]["expires_at"].is_i64());
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

#[test]
fn spec_usage_credits() {
    let examples = load_examples();
    let resp: ApiResponse<UsageCreditsData> =
        serde_json::from_value(examples["usage_credits"].clone()).unwrap();
    assert_eq!(resp.data.total_balance, Some(12.5));
    assert_eq!(resp.data.prepaid_balance, Some(10.0));
    assert_eq!(resp.data.free_balance, Some(2.5));
    assert!(resp.data.free_grants.is_some());
}

#[test]
fn spec_api_error() {
    let examples = load_examples();
    let err: ApiError = serde_json::from_value(examples["api_error"].clone()).unwrap();
    assert_eq!(err.message.as_deref(), Some("Invalid or expired token."));
    assert_eq!(err.extra["code"], 89);
}

#[test]
fn spec_api_problem() {
    let examples = load_examples();
    let err: ApiError = serde_json::from_value(examples["api_problem"].clone()).unwrap();
    assert_eq!(err.title.as_deref(), Some("Recipient Not Messageable"));
    assert!(
        err.detail
            .as_deref()
            .is_some_and(|d| d.contains("Direct Messages"))
    );
    assert_eq!(
        err.r#type.as_deref(),
        Some("https://api.x.com/2/problems/recipient-not-messageable")
    );
    assert_eq!(err.extra["status"], 403);
}

#[test]
fn spec_user_list() {
    let examples = load_examples();
    let resp: ApiResponse<Vec<User>> =
        serde_json::from_value(examples["user_list"].clone()).unwrap();
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].username, "TwitterDev");
}

#[test]
fn spec_chat_moderators() {
    let examples = load_examples();
    let resp: ApiResponse<ChatModeratorsResult> =
        serde_json::from_value(examples["chat_moderators"].clone()).unwrap();
    assert_eq!(resp.data.moderator_user_ids, vec!["2244994945"]);
}

#[test]
fn every_fixture_is_exercised_by_a_validation_test() {
    let source_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/spec_validation.rs");
    let source = std::fs::read_to_string(&source_path).expect("this test file is readable");
    let examples = load_examples();
    let untested: Vec<&String> = examples
        .as_object()
        .expect("the fixture file is a JSON object")
        .keys()
        .filter(|key| *key != "description")
        .filter(|key| !source.contains(&format!("examples[\"{key}\"]")))
        .collect();
    assert!(
        untested.is_empty(),
        "these fixtures in tests/fixtures/openapi/example_responses.json have no validation test: {untested:?}\n\
         Cause: a fixture was added to the file without a `spec_*` test that deserializes \
         `examples[\"<key>\"]` into its response type.\n\
         Fix: add the test to crates/xdk/tests/spec_validation.rs, or remove the fixture."
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Every fixture is the shape the spec gives the endpoint it answers
// ═══════════════════════════════════════════════════════════════════════════

/// Which declared endpoint each fixture answers, by the fixture's key, and
/// which of that endpoint's replies it is.
const FIXTURE_ENDPOINTS: &[(&str, Endpoint, ReplyKind)] = &[
    ("post_single", endpoints::READ_POST, ReplyKind::Success),
    ("post_list", endpoints::SEARCH_POSTS, ReplyKind::Success),
    ("user_single", endpoints::GET_ME, ReplyKind::Success),
    ("user_single_wire", endpoints::GET_ME, ReplyKind::Success),
    ("action_liked", endpoints::LIKE_POST, ReplyKind::Success),
    (
        "action_following",
        endpoints::FOLLOW_USER,
        ReplyKind::Success,
    ),
    ("action_deleted", endpoints::DELETE_POST, ReplyKind::Success),
    ("action_retweeted", endpoints::REPOST, ReplyKind::Success),
    ("action_bookmarked", endpoints::BOOKMARK, ReplyKind::Success),
    ("action_blocking", endpoints::BLOCK_USER, ReplyKind::Success),
    ("action_muting", endpoints::MUTE_USER, ReplyKind::Success),
    ("dm_sent", endpoints::SEND_DM, ReplyKind::Success),
    (
        "dm_event_list",
        endpoints::GET_DM_EVENTS,
        ReplyKind::Success,
    ),
    (
        "media_upload_init",
        endpoints::MEDIA_UPLOAD_INITIALIZE,
        ReplyKind::Success,
    ),
    (
        "media_upload_append",
        endpoints::MEDIA_UPLOAD_APPEND,
        ReplyKind::Success,
    ),
    (
        "media_upload_status",
        endpoints::MEDIA_UPLOAD_STATUS,
        ReplyKind::Success,
    ),
    ("usage", endpoints::GET_USAGE, ReplyKind::Success),
    (
        "usage_credits",
        endpoints::GET_USAGE_CREDITS,
        ReplyKind::Success,
    ),
    ("user_list", endpoints::GET_FOLLOWERS, ReplyKind::Success),
    (
        "chat_moderators",
        endpoints::ADD_CHAT_MODERATOR,
        ReplyKind::Success,
    ),
    ("api_error", endpoints::GET_ME, ReplyKind::Error),
    ("api_problem", endpoints::SEND_DM, ReplyKind::Problem),
];

/// Fixtures that carry what X really sends where the vendored spec says
/// otherwise, each with the reason. The walk requires every entry to keep
/// failing validation, so an exemption cannot outlive the drift it names.
const SPEC_EXEMPT_FIXTURES: &[(&str, &str)] = &[(
    "user_single_wire",
    "X sends `tweet_count` where spec 2.168 names `post_count`; the fixture carries the wire \
         shape and `UserPublicMetrics` reads either \
         (https://github.com/brettdavies/xurl-rs/pull/118)",
)];

#[test]
fn every_fixture_validates_against_its_endpoint_schema() {
    let spec = load_spec();
    let examples = load_examples();
    let endpoints_by_fixture: BTreeMap<&str, (&Endpoint, ReplyKind)> = FIXTURE_ENDPOINTS
        .iter()
        .map(|(key, ep, kind)| (*key, (ep, *kind)))
        .collect();
    let exempt: BTreeMap<&str, &str> = SPEC_EXEMPT_FIXTURES.iter().copied().collect();

    let mut unmapped = Vec::new();
    let mut failures = Vec::new();
    let mut stale_exemptions = Vec::new();
    for (key, fixture) in examples
        .as_object()
        .expect("the fixture file is a JSON object")
    {
        if key == "description" {
            continue;
        }
        let Some((endpoint, kind)) = endpoints_by_fixture.get(key.as_str()) else {
            unmapped.push(key.clone());
            continue;
        };
        let mut errors = Vec::new();
        check(
            fixture,
            fixture_schema(&spec, endpoint, *kind),
            &spec,
            key,
            &mut errors,
        );
        match (exempt.get(key.as_str()), errors.is_empty()) {
            (Some(_), true) => stale_exemptions.push(key.clone()),
            (Some(_), false) | (None, true) => {}
            (None, false) => failures.push(format!(
                "{key} ({} {}):\n    {}",
                endpoint.method,
                endpoint.path,
                errors.join("\n    ")
            )),
        }
    }

    assert!(
        unmapped.is_empty(),
        "these fixtures name no endpoint, so nothing checks them against the spec: {unmapped:?}\n\
         Cause: a fixture was added to tests/fixtures/openapi/example_responses.json without a \
         FIXTURE_ENDPOINTS row.\n\
         Fix: in crates/xdk/tests/spec_validation.rs, map the fixture to the endpoint constant \
         whose response it represents."
    );
    assert!(
        failures.is_empty(),
        "these fixtures are not the shape the vendored spec gives their endpoint:\n{}\n\
         Cause: the fixture was written by hand and disagrees with the spec, or the spec was \
         refreshed and the shape changed.\n\
         Fix: correct the fixture in tests/fixtures/openapi/example_responses.json; if X really \
         sends this shape, add the fixture to SPEC_EXEMPT_FIXTURES in \
         crates/xdk/tests/spec_validation.rs with the reason and a link. The spec refreshes \
         with scripts/refresh-x-openapi.sh.",
        failures.join("\n")
    );
    assert!(
        stale_exemptions.is_empty(),
        "these fixtures are exempt from spec validation but now pass it: {stale_exemptions:?}\n\
         Cause: the drift the exemption named is gone from the spec or the fixture.\n\
         Fix: drop each from SPEC_EXEMPT_FIXTURES in crates/xdk/tests/spec_validation.rs."
    );
}

#[test]
fn fixture_endpoint_map_names_only_fixtures_that_exist() {
    let examples = load_examples();
    let stale: Vec<&str> = FIXTURE_ENDPOINTS
        .iter()
        .map(|(key, _, _)| *key)
        .filter(|key| examples.get(key).is_none())
        .collect();
    assert!(
        stale.is_empty(),
        "FIXTURE_ENDPOINTS maps fixtures the file no longer carries: {stale:?}\n\
         Cause: the fixture was removed or renamed while its mapping stayed.\n\
         Fix: drop each stale row from FIXTURE_ENDPOINTS in crates/xdk/tests/spec_validation.rs."
    );
}
