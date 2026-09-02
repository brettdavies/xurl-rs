//! Typed API response structs for X API v2 endpoints.
//!
//! Every response struct includes `#[serde(flatten)] extra: BTreeMap<String, Value>`
//! for forward compatibility — unknown API fields are captured during deserialization
//! and re-emitted during serialization.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Generic wrapper ─────────────────────────────────────────────────

/// Standard X API v2 response envelope.
///
/// Single-item endpoints use `ApiResponse<Post>`, list endpoints use
/// `ApiResponse<Vec<Post>>`. Serde handles both shapes transparently.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct ApiResponse<T: Default> {
    /// Primary payload — a single object or a `Vec<T>` for list endpoints.
    pub data: T,
    /// Expanded objects referenced by `data` when the caller requested
    /// `expansions=...`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub includes: Option<Includes>,
    /// Pagination and result-count metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
    /// Partial errors returned alongside a 200 response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ApiError>>,
    /// Forward-compatibility bucket — captures unknown top-level fields so a
    /// new spec field round-trips through serialize/deserialize unchanged.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Expanded objects included alongside the primary data.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct Includes {
    /// User objects referenced by `author_id`, `sender_id`, etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<User>>,
    /// Post objects referenced by `referenced_posts`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posts: Option<Vec<Post>>,
    /// Forward-compatibility bucket — captures unknown include keys.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Pagination and result count metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct ResponseMeta {
    /// Total items returned in `data` for list endpoints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_count: Option<u64>,
    /// Opaque cursor for the next page; pass to a follow-up call as
    /// `pagination_token`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    /// Opaque cursor for the previous page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_token: Option<String>,
    /// Forward-compatibility bucket — captures unknown meta fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Partial error returned alongside valid data in 200 responses.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct ApiError {
    /// Short human-readable error description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Error category title (e.g. `"Not Found Error"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Longer human-readable detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// URI identifying the error type (problem-details style).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Forward-compatibility bucket — captures unknown error fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── Post ────────────────────────────────────────────────────────────

/// A post object from the X API v2.
///
/// Required fields: `id`, `text` (always present in API responses).
/// Optional fields depend on which `post.fields` the caller requests.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct Post {
    /// Post identifier (X API snowflake string).
    pub id: String,
    /// Post body text.
    pub text: String,
    /// ISO-8601 timestamp the post was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Author's user ID; resolve via `includes.users` when expanded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_id: Option<String>,
    /// Root post ID of the conversation thread.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    /// User ID this post replies to, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_reply_to_user_id: Option<String>,
    /// Engagement counts (likes, replies, reposts, etc).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_metrics: Option<PostPublicMetrics>,
    /// Posts this one references (reply, quote, repost).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referenced_posts: Option<Vec<ReferencedPost>>,
    /// Parsed entities (URLs, mentions, hashtags) — opaque JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entities: Option<Value>,
    /// Media / poll attachment references — opaque JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Value>,
    /// Forward-compatibility bucket — captures unknown post fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Public engagement metrics for a post.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct PostPublicMetrics {
    /// Repost count.
    #[serde(default)]
    pub repost_count: u64,
    /// Reply count.
    #[serde(default)]
    pub reply_count: u64,
    /// Like count.
    #[serde(default)]
    pub like_count: u64,
    /// Quote-post count.
    #[serde(default)]
    pub quote_count: u64,
    /// Bookmark count.
    #[serde(default)]
    pub bookmark_count: u64,
    /// View count.
    #[serde(default)]
    pub impression_count: u64,
    /// Forward-compatibility bucket — captures unknown metric fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// A referenced post (reply-to, quote, repost).
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct ReferencedPost {
    /// Referenced post identifier.
    pub id: String,
    /// Reference kind — `"replied_to"`, `"quoted"`, or `"retweeted"`.
    pub r#type: String,
    /// Forward-compatibility bucket — captures unknown reference fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── User ────────────────────────────────────────────────────────────

/// A user object from the X API v2.
///
/// Required fields: `id`, `name`, `username`.
/// Optional fields depend on which `user.fields` the caller requests.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct User {
    /// User identifier (X API snowflake string).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Handle without the leading `@`.
    pub username: String,
    /// ISO-8601 account creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Bio / profile description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the account carries a verified badge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    /// Profile image URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_image_url: Option<String>,
    /// Engagement counts (followers, following, posts).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_metrics: Option<UserPublicMetrics>,
    /// Forward-compatibility bucket — captures unknown user fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Public engagement metrics for a user.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct UserPublicMetrics {
    /// Follower count.
    #[serde(default)]
    pub followers_count: u64,
    /// Following count.
    #[serde(default)]
    pub following_count: u64,
    /// Post count for the user.
    #[serde(default)]
    pub post_count: u64,
    /// Number of public lists the user is on.
    #[serde(default)]
    pub listed_count: u64,
    /// Forward-compatibility bucket — captures unknown metric fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── DM ──────────────────────────────────────────────────────────────

/// A direct message event from the X API v2.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct DmEvent {
    /// Event identifier.
    pub id: String,
    /// Message body, for text events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Event kind (e.g. `"MessageCreate"`, `"ParticipantsJoin"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    /// ISO-8601 timestamp the event occurred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Conversation the event belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dm_conversation_id: Option<String>,
    /// Sender's user ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<String>,
    /// Forward-compatibility bucket — captures unknown event fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── Action confirmations ────────────────────────────────────────────

/// Confirmation for like/unlike actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct LikedResult {
    /// Whether the target is now liked.
    pub liked: bool,
    /// Forward-compatibility bucket — captures unknown response fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for follow/unfollow actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct FollowingResult {
    /// Whether the target is now followed.
    pub following: bool,
    /// Forward-compatibility bucket — captures unknown response fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for delete actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct DeletedResult {
    /// Whether the resource was deleted.
    pub deleted: bool,
    /// Forward-compatibility bucket — captures unknown response fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for repost/unrepost actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct RepostedResult {
    /// Whether the target is now reposted.
    pub retweeted: bool,
    /// Forward-compatibility bucket — captures unknown response fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for bookmark/unbookmark actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct BookmarkedResult {
    /// Whether the target is now bookmarked.
    pub bookmarked: bool,
    /// Forward-compatibility bucket — captures unknown response fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for block/unblock actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct BlockingResult {
    /// Whether the target is now blocked.
    pub blocking: bool,
    /// Forward-compatibility bucket — captures unknown response fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for mute/unmute actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct MutingResult {
    /// Whether the target is now muted.
    pub muting: bool,
    /// Forward-compatibility bucket — captures unknown response fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── Media ───────────────────────────────────────────────────────────

/// Response from media upload INIT and FINALIZE steps.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct MediaUploadResponse {
    /// Media identifier — thread this into APPEND, FINALIZE, and STATUS calls.
    pub id: String,
    /// Stable media key surfaced once FINALIZE succeeds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_key: Option<String>,
    /// Seconds until the upload session expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after_secs: Option<u64>,
    /// Server-side processing status for async media (video, GIF).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processing_info: Option<MediaProcessingInfo>,
    /// Forward-compatibility bucket — captures unknown upload fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Media processing status returned during upload polling.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct MediaProcessingInfo {
    /// Processing state — `"pending"`, `"in_progress"`, `"succeeded"`, `"failed"`.
    pub state: String,
    /// Recommended wait before polling again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_after_secs: Option<u64>,
    /// Server-reported processing progress (0..=100).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<u64>,
    /// Error details when `state` is `"failed"` — opaque JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    /// Forward-compatibility bucket — captures unknown status fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── Usage ───────────────────────────────────────────────────────────

/// API usage data from the /2/usage/tweets endpoint.
///
/// All fields are optional because the shape varies based on query params
/// and the data is deeply nested with mixed types (strings for numbers).
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct UsageData {
    /// Project post cap (string-encoded integer).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_cap: Option<String>,
    /// Project identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Project post usage so far this period (string-encoded integer).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_usage: Option<String>,
    /// Day of month the cap resets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cap_reset_day: Option<u64>,
    /// Per-day project usage breakdown — opaque JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_project_usage: Option<Value>,
    /// Per-day client-app usage breakdown — opaque JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_client_app_usage: Option<Value>,
    /// Forward-compatibility bucket — captures unknown usage fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Credits-based usage for the project, from `GET /2/usage/credits`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct UsageCreditsData {
    /// Remaining free credit balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub free_balance: Option<f64>,
    /// Free credit grants applied to the project, opaque JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub free_grants: Option<Value>,
    /// Remaining prepaid credit balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepaid_balance: Option<f64>,
    /// Total remaining credit balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_balance: Option<f64>,
    /// Forward-compatibility bucket — captures unknown credits fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── Deserialization helper ──────────────────────────────────────────

/// Deserializes a `serde_json::Value` into `ApiResponse<T>`.
///
/// Guards against empty `{}` responses (the `send_request()` fallback for
/// non-JSON 2xx bodies) with a descriptive error instead of a cryptic
/// serde deserialization failure.
///
/// # Errors
///
/// Returns `XurlError::Json` if the Value is an empty object or cannot
/// be deserialized into the target type.
pub fn deserialize_response<T: Default + serde::de::DeserializeOwned>(
    value: Value,
) -> crate::error::Result<ApiResponse<T>> {
    if value.as_object().is_some_and(|m| m.is_empty()) {
        return Err(crate::error::XurlError::Json(
            "empty response body — expected JSON with a \"data\" field".to_string(),
        ));
    }
    // X API v2 returns errors-only 200 responses with no `data` field
    // (e.g., {"errors": [{"title": "Not Found Error", ...}]}). Surface
    // the raw JSON as a validation error — these are not HTTP errors
    // (status was 200) but semantic failures from the API.
    if let Some(obj) = value.as_object()
        && !obj.contains_key("data")
        && obj.contains_key("errors")
    {
        return Err(crate::error::XurlError::validation(value.to_string()));
    }
    Ok(serde_json::from_value(value)?)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // ── Happy path ──────────────────────────────────────────────────

    #[test]
    fn deserialize_single_post() {
        let json = json!({
            "data": {
                "id": "123",
                "text": "Hello world",
                "created_at": "2026-01-01T00:00:00.000Z",
                "public_metrics": {
                    "repost_count": 5,
                    "reply_count": 2,
                    "like_count": 10,
                    "quote_count": 1,
                    "bookmark_count": 0,
                    "impression_count": 100
                }
            }
        });
        let resp: ApiResponse<Post> =
            serde_json::from_value(json).expect("Post response must deserialize");
        assert_eq!(resp.data.id, "123");
        assert_eq!(resp.data.text, "Hello world");
        assert_eq!(
            resp.data.created_at.as_deref(),
            Some("2026-01-01T00:00:00.000Z")
        );
        let metrics = resp
            .data
            .public_metrics
            .expect("public_metrics must be present");
        assert_eq!(metrics.like_count, 10);
        assert_eq!(metrics.impression_count, 100);
    }

    #[test]
    fn deserialize_post_list() {
        let json = json!({
            "data": [
                {"id": "1", "text": "first"},
                {"id": "2", "text": "second"}
            ],
            "meta": {"result_count": 2}
        });
        let resp: ApiResponse<Vec<Post>> =
            serde_json::from_value(json).expect("Post list response must deserialize");
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].id, "1");
        assert_eq!(resp.data[1].text, "second");
        assert_eq!(
            resp.meta.expect("meta must be present").result_count,
            Some(2)
        );
    }

    #[test]
    fn deserialize_action_liked() {
        let json = json!({"data": {"liked": true}});
        let resp: ApiResponse<LikedResult> =
            serde_json::from_value(json).expect("LikedResult response must deserialize");
        assert!(resp.data.liked);
    }

    #[test]
    fn deserialize_with_includes_and_meta() {
        let json = json!({
            "data": [{"id": "1", "text": "post"}],
            "includes": {
                "users": [{"id": "42", "name": "Bot", "username": "bot"}]
            },
            "meta": {"result_count": 1, "next_token": "abc123"}
        });
        let resp: ApiResponse<Vec<Post>> =
            serde_json::from_value(json).expect("Post list with includes/meta must deserialize");
        let includes = resp.includes.expect("includes must be present");
        let users = includes.users.expect("includes.users must be present");
        assert_eq!(users[0].id, "42");
        assert_eq!(
            resp.meta
                .expect("meta must be present")
                .next_token
                .as_deref(),
            Some("abc123")
        );
    }

    #[test]
    fn deserialize_user() {
        let json = json!({
            "data": {
                "id": "42",
                "name": "Test User",
                "username": "testuser",
                "verified": true,
                "public_metrics": {
                    "followers_count": 100,
                    "following_count": 50,
                    "post_count": 1000,
                    "listed_count": 5
                }
            }
        });
        let resp: ApiResponse<User> =
            serde_json::from_value(json).expect("User response must deserialize");
        assert_eq!(resp.data.id, "42");
        assert_eq!(resp.data.username, "testuser");
        assert_eq!(resp.data.verified, Some(true));
        let metrics = resp
            .data
            .public_metrics
            .expect("user.public_metrics must be present");
        assert_eq!(metrics.followers_count, 100);
    }

    #[test]
    fn deserialize_dm_event() {
        let json = json!({
            "data": {
                "id": "dm1",
                "text": "hello",
                "event_type": "MessageCreate",
                "dm_conversation_id": "conv1",
                "sender_id": "42"
            }
        });
        let resp: ApiResponse<DmEvent> =
            serde_json::from_value(json).expect("DmEvent response must deserialize");
        assert_eq!(resp.data.id, "dm1");
        assert_eq!(resp.data.text.as_deref(), Some("hello"));
        assert_eq!(resp.data.sender_id.as_deref(), Some("42"));
    }

    #[test]
    fn deserialize_usage_data() {
        let json = json!({
            "data": {
                "project_cap": "2000000",
                "project_id": "123",
                "project_usage": "399",
                "cap_reset_day": 19
            }
        });
        let resp: ApiResponse<UsageData> =
            serde_json::from_value(json).expect("UsageData response must deserialize");
        assert_eq!(resp.data.project_cap.as_deref(), Some("2000000"));
        assert_eq!(resp.data.cap_reset_day, Some(19));
    }

    #[test]
    fn deserialize_media_upload_response() {
        let json = json!({
            "data": {
                "id": "media_123",
                "media_key": "key_456",
                "expires_after_secs": 3600
            }
        });
        let resp: ApiResponse<MediaUploadResponse> =
            serde_json::from_value(json).expect("MediaUploadResponse must deserialize");
        assert_eq!(resp.data.id, "media_123");
        assert_eq!(resp.data.media_key.as_deref(), Some("key_456"));
        assert_eq!(resp.data.expires_after_secs, Some(3600));
    }

    #[test]
    fn deserialize_media_with_processing_info() {
        let json = json!({
            "data": {
                "id": "media_123",
                "processing_info": {
                    "state": "in_progress",
                    "check_after_secs": 5,
                    "progress_percent": 45
                }
            }
        });
        let resp: ApiResponse<MediaUploadResponse> = serde_json::from_value(json)
            .expect("MediaUploadResponse with processing_info must deserialize");
        let info = resp
            .data
            .processing_info
            .expect("processing_info must be present");
        assert_eq!(info.state, "in_progress");
        assert_eq!(info.check_after_secs, Some(5));
        assert_eq!(info.progress_percent, Some(45));
    }

    // ── Edge cases ──────────────────────────────────────────────────

    #[test]
    fn unknown_fields_captured_in_extra() {
        let json = json!({
            "data": {
                "id": "123",
                "text": "hello",
                "brand_new_field": "surprise"
            },
            "top_level_extra": 42
        });
        let resp: ApiResponse<Post> = serde_json::from_value(json)
            .expect("Post response with unknown fields must deserialize");
        assert_eq!(resp.data.extra["brand_new_field"], "surprise");
        assert_eq!(resp.extra["top_level_extra"], 42);
    }

    #[test]
    fn unknown_fields_round_trip() {
        let json = json!({
            "data": {
                "id": "123",
                "text": "hello",
                "new_field": 42
            },
            "top_extra": "value"
        });
        let resp: ApiResponse<Post> =
            serde_json::from_value(json).expect("Post round-trip fixture must deserialize");
        let serialized = serde_json::to_value(&resp).expect("Post response must serialize");
        assert_eq!(serialized["data"]["new_field"], 42);
        assert_eq!(serialized["top_extra"], "value");
    }

    #[test]
    fn nested_unknown_fields_both_captured() {
        let json = json!({
            "data": {
                "id": "123",
                "text": "hello",
                "post_extra": "a",
                "public_metrics": {
                    "repost_count": 0,
                    "reply_count": 0,
                    "like_count": 0,
                    "quote_count": 0,
                    "bookmark_count": 0,
                    "impression_count": 0,
                    "metrics_extra": "b"
                }
            }
        });
        let resp: ApiResponse<Post> = serde_json::from_value(json)
            .expect("Post response with nested unknown fields must deserialize");
        assert_eq!(resp.data.extra["post_extra"], "a");
        let metrics = resp
            .data
            .public_metrics
            .expect("public_metrics must be present");
        assert_eq!(metrics.extra["metrics_extra"], "b");
    }

    #[test]
    fn extra_is_empty_when_no_unknown_fields() {
        let json = json!({"data": {"id": "1", "text": "hi"}});
        let resp: ApiResponse<Post> =
            serde_json::from_value(json).expect("minimal Post response must deserialize");
        assert!(resp.extra.is_empty());
        assert!(resp.data.extra.is_empty());
        // Verify serialization produces no extra keys
        let out = serde_json::to_value(&resp).expect("Post response must serialize");
        let data = out["data"]
            .as_object()
            .expect("serialized data must be a JSON object");
        assert!(!data.contains_key("extra"));
    }

    #[test]
    fn missing_optional_fields_are_none() {
        let json = json!({"data": {"id": "1", "text": "minimal"}});
        let resp: ApiResponse<Post> =
            serde_json::from_value(json).expect("minimal Post must deserialize");
        assert!(resp.data.created_at.is_none());
        assert!(resp.data.public_metrics.is_none());
        assert!(resp.data.author_id.is_none());
        assert!(resp.includes.is_none());
        assert!(resp.meta.is_none());
    }

    #[test]
    fn default_produces_valid_structs() {
        let post = Post {
            id: "test".into(),
            text: "hello".into(),
            ..Default::default()
        };
        assert_eq!(post.id, "test");
        assert!(post.created_at.is_none());

        let user = User {
            id: "42".into(),
            name: "Bot".into(),
            username: "bot".into(),
            ..Default::default()
        };
        assert_eq!(user.username, "bot");

        let _resp: ApiResponse<Post> = ApiResponse {
            data: post,
            ..Default::default()
        };
    }

    #[test]
    fn all_action_types_default_and_deserialize() {
        // Verify all 7 action types work
        let _: LikedResult = Default::default();
        let _: FollowingResult = Default::default();
        let _: DeletedResult = Default::default();
        let _: RepostedResult = Default::default();
        let _: BookmarkedResult = Default::default();
        let _: BlockingResult = Default::default();
        let _: MutingResult = Default::default();

        for (field, ty) in [
            ("liked", "LikedResult"),
            ("following", "FollowingResult"),
            ("deleted", "DeletedResult"),
            ("retweeted", "RepostedResult"),
            ("bookmarked", "BookmarkedResult"),
            ("blocking", "BlockingResult"),
            ("muting", "MutingResult"),
        ] {
            let json = json!({"data": {field: true}});
            // Verify they all parse — use a match to dispatch
            match ty {
                "LikedResult" => {
                    let r: ApiResponse<LikedResult> = serde_json::from_value(json)
                        .expect("LikedResult action response must deserialize");
                    assert!(r.data.liked);
                }
                "FollowingResult" => {
                    let r: ApiResponse<FollowingResult> = serde_json::from_value(json)
                        .expect("FollowingResult action response must deserialize");
                    assert!(r.data.following);
                }
                "DeletedResult" => {
                    let r: ApiResponse<DeletedResult> = serde_json::from_value(json)
                        .expect("DeletedResult action response must deserialize");
                    assert!(r.data.deleted);
                }
                "RepostedResult" => {
                    let r: ApiResponse<RepostedResult> = serde_json::from_value(json)
                        .expect("RepostedResult action response must deserialize");
                    assert!(r.data.retweeted);
                }
                "BookmarkedResult" => {
                    let r: ApiResponse<BookmarkedResult> = serde_json::from_value(json)
                        .expect("BookmarkedResult action response must deserialize");
                    assert!(r.data.bookmarked);
                }
                "BlockingResult" => {
                    let r: ApiResponse<BlockingResult> = serde_json::from_value(json)
                        .expect("BlockingResult action response must deserialize");
                    assert!(r.data.blocking);
                }
                "MutingResult" => {
                    let r: ApiResponse<MutingResult> = serde_json::from_value(json)
                        .expect("MutingResult action response must deserialize");
                    assert!(r.data.muting);
                }
                _ => unreachable!(),
            }
        }
    }

    // ── Error paths ─────────────────────────────────────────────────

    #[test]
    fn invalid_json_missing_required_field() {
        // Missing required `id` field
        let json = json!({"data": {"text": "no id"}});
        let result = serde_json::from_value::<ApiResponse<Post>>(json);
        assert!(result.is_err());
    }

    #[test]
    fn round_trip_serialize_deserialize() {
        let post = Post {
            id: "456".into(),
            text: "round trip".into(),
            created_at: Some("2026-01-01T00:00:00Z".into()),
            ..Default::default()
        };
        let resp = ApiResponse {
            data: post,
            ..Default::default()
        };
        let value = serde_json::to_value(&resp).expect("Post must serialize");
        let back: ApiResponse<Post> =
            serde_json::from_value(value).expect("round-tripped Post must deserialize");
        assert_eq!(back.data.id, "456");
        assert_eq!(back.data.text, "round trip");
    }

    #[test]
    fn deserialize_helper_rejects_empty_object() {
        let value = json!({});
        let result = deserialize_response::<Post>(value);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("empty response body"), "Got: {err}");
    }

    // ── Red team / adversarial ──────────────────────────────────────

    #[test]
    fn adversarial_array_where_object_expected() {
        // data is an array but we expect a single Post
        let json = json!({"data": [{"id": "1", "text": "oops"}]});
        let result = serde_json::from_value::<ApiResponse<Post>>(json);
        assert!(result.is_err(), "Should fail: array where object expected");
    }

    #[test]
    fn adversarial_string_where_object_expected() {
        let json = json!({"data": "not an object"});
        let result = serde_json::from_value::<ApiResponse<Post>>(json);
        assert!(result.is_err(), "Should fail: string where object expected");
    }

    #[test]
    fn adversarial_null_data_field() {
        let json = json!({"data": null});
        let result = serde_json::from_value::<ApiResponse<Post>>(json);
        assert!(result.is_err(), "Should fail: null data");
    }

    #[test]
    fn adversarial_numeric_overflow_u64() {
        // u64::MAX + 1 would overflow — serde should error, not panic
        let json = json!({
            "data": {
                "id": "123",
                "text": "hi",
                "public_metrics": {
                    "like_count": 99_999_999_999_999u64,
                    "repost_count": 0,
                    "reply_count": 0,
                    "quote_count": 0,
                    "bookmark_count": 0,
                    "impression_count": 0
                }
            }
        });
        // This should succeed — 99_999_999_999_999 fits in u64
        let resp: ApiResponse<Post> =
            serde_json::from_value(json).expect("Post with large u64 must deserialize");
        assert_eq!(
            resp.data
                .public_metrics
                .expect("public_metrics must be present")
                .like_count,
            99_999_999_999_999
        );
    }

    #[test]
    fn adversarial_negative_count() {
        // Negative number in a u64 field — should error
        let json = json!({
            "data": {
                "id": "123",
                "text": "hi",
                "public_metrics": {
                    "like_count": -1,
                    "repost_count": 0,
                    "reply_count": 0,
                    "quote_count": 0,
                    "bookmark_count": 0,
                    "impression_count": 0
                }
            }
        });
        let result = serde_json::from_value::<ApiResponse<Post>>(json);
        assert!(result.is_err(), "Should fail: negative u64");
    }

    #[test]
    fn adversarial_deeply_nested_unknown_fields() {
        let json = json!({
            "data": {"id": "123", "text": "hi"},
            "extra_field": {
                "a": {"b": {"c": [1, 2, 3]}}
            }
        });
        let resp: ApiResponse<Post> =
            serde_json::from_value(json).expect("Post with deep unknown fields must deserialize");
        assert!(resp.extra.contains_key("extra_field"));
    }

    #[test]
    fn adversarial_empty_string_required_fields() {
        // Empty strings are valid String values — consumer must validate semantics
        let json = json!({"data": {"id": "", "text": ""}});
        let resp: ApiResponse<Post> = serde_json::from_value(json)
            .expect("Post with empty string required fields must deserialize");
        assert_eq!(resp.data.id, "");
        assert_eq!(resp.data.text, "");
    }

    #[test]
    fn adversarial_errors_field_no_data_raw_serde() {
        // Raw serde (not deserialize_response) — should fail on missing data
        let json = json!({"errors": [{"message": "forbidden"}]});
        let result = serde_json::from_value::<ApiResponse<Post>>(json);
        assert!(result.is_err(), "Should fail: no data field");
    }

    #[test]
    fn errors_only_response_returns_validation_error() {
        // X API v2 returns {"errors": [...]} with no "data" on 200 for not-found resources.
        // deserialize_response should return XurlError::Validation with the raw JSON.
        let json = json!({
            "errors": [{
                "detail": "Could not find tweet with id: [123].",
                "title": "Not Found Error",
                "resource_type": "tweet",
                "type": "https://api.twitter.com/2/problems/resource-not-found"
            }]
        });
        let result = deserialize_response::<Post>(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_validation(), "Expected Validation error, got: {err}");
        let msg = err.to_string();
        assert!(
            msg.contains("Not Found Error"),
            "Error should contain API message: {msg}"
        );
    }

    #[test]
    fn adversarial_extra_top_level_on_action() {
        let json = json!({
            "data": {"liked": true},
            "extra_top_level": "ignored_by_consumer"
        });
        let resp: ApiResponse<LikedResult> = serde_json::from_value(json)
            .expect("LikedResult with extra top-level fields must deserialize");
        assert!(resp.data.liked);
        assert_eq!(resp.extra["extra_top_level"], "ignored_by_consumer");
    }

    #[test]
    fn adversarial_wrong_bool_type_for_action() {
        // String "true" instead of boolean true
        let json = json!({"data": {"liked": "true"}});
        let result = serde_json::from_value::<ApiResponse<LikedResult>>(json);
        assert!(result.is_err(), "Should fail: string instead of bool");
    }

    #[test]
    fn adversarial_partial_errors_with_valid_data() {
        // 200 response with both data and errors (partial failure)
        let json = json!({
            "data": {"id": "123", "text": "partial"},
            "errors": [{"message": "some field unavailable", "title": "Partial Error"}]
        });
        let resp: ApiResponse<Post> = serde_json::from_value(json)
            .expect("Post with partial errors payload must deserialize");
        assert_eq!(resp.data.id, "123");
        let errors = resp.errors.expect("errors field must be present");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message.as_deref(), Some("some field unavailable"));
    }

    #[test]
    fn adversarial_huge_array_in_data() {
        // Large but valid list
        let posts: Vec<Value> = (0..1000)
            .map(|i| json!({"id": i.to_string(), "text": format!("post {i}")}))
            .collect();
        let json = json!({"data": posts});
        let resp: ApiResponse<Vec<Post>> =
            serde_json::from_value(json).expect("1000-element Post list must deserialize");
        assert_eq!(resp.data.len(), 1000);
    }

    #[test]
    fn adversarial_completely_wrong_shape() {
        // Total garbage for data
        let json = json!({"data": 42});
        let result = serde_json::from_value::<ApiResponse<Post>>(json);
        assert!(result.is_err());
    }

    #[test]
    fn adversarial_api_error_with_extra_fields() {
        let json = json!({
            "data": {"id": "1", "text": "ok"},
            "errors": [{
                "message": "oops",
                "new_error_field": "surprise"
            }]
        });
        let resp: ApiResponse<Post> = serde_json::from_value(json)
            .expect("Post with errors containing extra fields must deserialize");
        let error = &resp.errors.expect("errors field must be present")[0];
        assert_eq!(error.extra["new_error_field"], "surprise");
    }
}
