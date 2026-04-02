/// Typed API response structs for X API v2 endpoints.
///
/// Every response struct includes `#[serde(flatten)] extra: BTreeMap<String, Value>`
/// for forward compatibility — unknown API fields are captured during deserialization
/// and re-emitted during serialization.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Generic wrapper ─────────────────────────────────────────────────

/// Standard X API v2 response envelope.
///
/// Single-item endpoints use `ApiResponse<Tweet>`, list endpoints use
/// `ApiResponse<Vec<Tweet>>`. Serde handles both shapes transparently.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiResponse<T: Default> {
    pub data: T,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub includes: Option<Includes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ApiError>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Expanded objects included alongside the primary data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Includes {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<User>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tweets: Option<Vec<Tweet>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Pagination and result count metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_token: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Partial error returned alongside valid data in 200 responses.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiError {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── Tweet ───────────────────────────────────────────────────────────

/// A tweet object from the X API v2.
///
/// Required fields: `id`, `text` (always present in API responses).
/// Optional fields depend on which `tweet.fields` the caller requests.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tweet {
    pub id: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_reply_to_user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_metrics: Option<TweetPublicMetrics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referenced_tweets: Option<Vec<ReferencedTweet>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entities: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Public engagement metrics for a tweet.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TweetPublicMetrics {
    #[serde(default)]
    pub retweet_count: u64,
    #[serde(default)]
    pub reply_count: u64,
    #[serde(default)]
    pub like_count: u64,
    #[serde(default)]
    pub quote_count: u64,
    #[serde(default)]
    pub bookmark_count: u64,
    #[serde(default)]
    pub impression_count: u64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// A referenced tweet (reply-to, quote, retweet).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReferencedTweet {
    pub id: String,
    pub r#type: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── User ────────────────────────────────────────────────────────────

/// A user object from the X API v2.
///
/// Required fields: `id`, `name`, `username`.
/// Optional fields depend on which `user.fields` the caller requests.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub id: String,
    pub name: String,
    pub username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_metrics: Option<UserPublicMetrics>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Public engagement metrics for a user.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPublicMetrics {
    #[serde(default)]
    pub followers_count: u64,
    #[serde(default)]
    pub following_count: u64,
    #[serde(default)]
    pub tweet_count: u64,
    #[serde(default)]
    pub listed_count: u64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── DM ──────────────────────────────────────────────────────────────

/// A direct message event from the X API v2.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DmEvent {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dm_conversation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── Action confirmations ────────────────────────────────────────────

/// Confirmation for like/unlike actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LikedResult {
    pub liked: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for follow/unfollow actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FollowingResult {
    pub following: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for delete actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeletedResult {
    pub deleted: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for repost/unrepost actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetweetedResult {
    pub retweeted: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for bookmark/unbookmark actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookmarkedResult {
    pub bookmarked: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for block/unblock actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockingResult {
    pub blocking: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Confirmation for mute/unmute actions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MutingResult {
    pub muting: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── Media ───────────────────────────────────────────────────────────

/// Response from media upload INIT and FINALIZE steps.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaUploadResponse {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processing_info: Option<MediaProcessingInfo>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Media processing status returned during upload polling.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaProcessingInfo {
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_after_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

// ── Usage ───────────────────────────────────────────────────────────

/// API usage data from the /2/usage/tweets endpoint.
///
/// All fields are optional because the shape varies based on query params
/// and the data is deeply nested with mixed types (strings for numbers).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_cap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_usage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cap_reset_day: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_project_usage: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_client_app_usage: Option<Value>,
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
    // the raw JSON as an API error, matching pre-migration behavior.
    if let Some(obj) = value.as_object() {
        if !obj.contains_key("data") && obj.contains_key("errors") {
            return Err(crate::error::XurlError::Api(value.to_string()));
        }
    }
    Ok(serde_json::from_value(value)?)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // ── Happy path ──────────────────────────────────────────────────

    #[test]
    fn deserialize_single_tweet() {
        let json = json!({
            "data": {
                "id": "123",
                "text": "Hello world",
                "created_at": "2026-01-01T00:00:00.000Z",
                "public_metrics": {
                    "retweet_count": 5,
                    "reply_count": 2,
                    "like_count": 10,
                    "quote_count": 1,
                    "bookmark_count": 0,
                    "impression_count": 100
                }
            }
        });
        let resp: ApiResponse<Tweet> = serde_json::from_value(json).unwrap();
        assert_eq!(resp.data.id, "123");
        assert_eq!(resp.data.text, "Hello world");
        assert_eq!(
            resp.data.created_at.as_deref(),
            Some("2026-01-01T00:00:00.000Z")
        );
        let metrics = resp.data.public_metrics.unwrap();
        assert_eq!(metrics.like_count, 10);
        assert_eq!(metrics.impression_count, 100);
    }

    #[test]
    fn deserialize_tweet_list() {
        let json = json!({
            "data": [
                {"id": "1", "text": "first"},
                {"id": "2", "text": "second"}
            ],
            "meta": {"result_count": 2}
        });
        let resp: ApiResponse<Vec<Tweet>> = serde_json::from_value(json).unwrap();
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].id, "1");
        assert_eq!(resp.data[1].text, "second");
        assert_eq!(resp.meta.unwrap().result_count, Some(2));
    }

    #[test]
    fn deserialize_action_liked() {
        let json = json!({"data": {"liked": true}});
        let resp: ApiResponse<LikedResult> = serde_json::from_value(json).unwrap();
        assert!(resp.data.liked);
    }

    #[test]
    fn deserialize_with_includes_and_meta() {
        let json = json!({
            "data": [{"id": "1", "text": "tweet"}],
            "includes": {
                "users": [{"id": "42", "name": "Bot", "username": "bot"}]
            },
            "meta": {"result_count": 1, "next_token": "abc123"}
        });
        let resp: ApiResponse<Vec<Tweet>> = serde_json::from_value(json).unwrap();
        let includes = resp.includes.unwrap();
        let users = includes.users.unwrap();
        assert_eq!(users[0].id, "42");
        assert_eq!(resp.meta.unwrap().next_token.as_deref(), Some("abc123"));
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
                    "tweet_count": 1000,
                    "listed_count": 5
                }
            }
        });
        let resp: ApiResponse<User> = serde_json::from_value(json).unwrap();
        assert_eq!(resp.data.id, "42");
        assert_eq!(resp.data.username, "testuser");
        assert_eq!(resp.data.verified, Some(true));
        let metrics = resp.data.public_metrics.unwrap();
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
        let resp: ApiResponse<DmEvent> = serde_json::from_value(json).unwrap();
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
        let resp: ApiResponse<UsageData> = serde_json::from_value(json).unwrap();
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
        let resp: ApiResponse<MediaUploadResponse> = serde_json::from_value(json).unwrap();
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
        let resp: ApiResponse<MediaUploadResponse> = serde_json::from_value(json).unwrap();
        let info = resp.data.processing_info.unwrap();
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
        let resp: ApiResponse<Tweet> = serde_json::from_value(json).unwrap();
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
        let resp: ApiResponse<Tweet> = serde_json::from_value(json).unwrap();
        let serialized = serde_json::to_value(&resp).unwrap();
        assert_eq!(serialized["data"]["new_field"], 42);
        assert_eq!(serialized["top_extra"], "value");
    }

    #[test]
    fn nested_unknown_fields_both_captured() {
        let json = json!({
            "data": {
                "id": "123",
                "text": "hello",
                "tweet_extra": "a",
                "public_metrics": {
                    "retweet_count": 0,
                    "reply_count": 0,
                    "like_count": 0,
                    "quote_count": 0,
                    "bookmark_count": 0,
                    "impression_count": 0,
                    "metrics_extra": "b"
                }
            }
        });
        let resp: ApiResponse<Tweet> = serde_json::from_value(json).unwrap();
        assert_eq!(resp.data.extra["tweet_extra"], "a");
        let metrics = resp.data.public_metrics.unwrap();
        assert_eq!(metrics.extra["metrics_extra"], "b");
    }

    #[test]
    fn extra_is_empty_when_no_unknown_fields() {
        let json = json!({"data": {"id": "1", "text": "hi"}});
        let resp: ApiResponse<Tweet> = serde_json::from_value(json).unwrap();
        assert!(resp.extra.is_empty());
        assert!(resp.data.extra.is_empty());
        // Verify serialization produces no extra keys
        let out = serde_json::to_value(&resp).unwrap();
        let data = out["data"].as_object().unwrap();
        assert!(!data.contains_key("extra"));
    }

    #[test]
    fn missing_optional_fields_are_none() {
        let json = json!({"data": {"id": "1", "text": "minimal"}});
        let resp: ApiResponse<Tweet> = serde_json::from_value(json).unwrap();
        assert!(resp.data.created_at.is_none());
        assert!(resp.data.public_metrics.is_none());
        assert!(resp.data.author_id.is_none());
        assert!(resp.includes.is_none());
        assert!(resp.meta.is_none());
    }

    #[test]
    fn default_produces_valid_structs() {
        let tweet = Tweet {
            id: "test".into(),
            text: "hello".into(),
            ..Default::default()
        };
        assert_eq!(tweet.id, "test");
        assert!(tweet.created_at.is_none());

        let user = User {
            id: "42".into(),
            name: "Bot".into(),
            username: "bot".into(),
            ..Default::default()
        };
        assert_eq!(user.username, "bot");

        let _resp: ApiResponse<Tweet> = ApiResponse {
            data: tweet,
            ..Default::default()
        };
    }

    #[test]
    fn all_action_types_default_and_deserialize() {
        // Verify all 7 action types work
        let _: LikedResult = Default::default();
        let _: FollowingResult = Default::default();
        let _: DeletedResult = Default::default();
        let _: RetweetedResult = Default::default();
        let _: BookmarkedResult = Default::default();
        let _: BlockingResult = Default::default();
        let _: MutingResult = Default::default();

        for (field, ty) in [
            ("liked", "LikedResult"),
            ("following", "FollowingResult"),
            ("deleted", "DeletedResult"),
            ("retweeted", "RetweetedResult"),
            ("bookmarked", "BookmarkedResult"),
            ("blocking", "BlockingResult"),
            ("muting", "MutingResult"),
        ] {
            let json = json!({"data": {field: true}});
            // Verify they all parse — use a match to dispatch
            match ty {
                "LikedResult" => {
                    let r: ApiResponse<LikedResult> = serde_json::from_value(json).unwrap();
                    assert!(r.data.liked);
                }
                "FollowingResult" => {
                    let r: ApiResponse<FollowingResult> = serde_json::from_value(json).unwrap();
                    assert!(r.data.following);
                }
                "DeletedResult" => {
                    let r: ApiResponse<DeletedResult> = serde_json::from_value(json).unwrap();
                    assert!(r.data.deleted);
                }
                "RetweetedResult" => {
                    let r: ApiResponse<RetweetedResult> = serde_json::from_value(json).unwrap();
                    assert!(r.data.retweeted);
                }
                "BookmarkedResult" => {
                    let r: ApiResponse<BookmarkedResult> = serde_json::from_value(json).unwrap();
                    assert!(r.data.bookmarked);
                }
                "BlockingResult" => {
                    let r: ApiResponse<BlockingResult> = serde_json::from_value(json).unwrap();
                    assert!(r.data.blocking);
                }
                "MutingResult" => {
                    let r: ApiResponse<MutingResult> = serde_json::from_value(json).unwrap();
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
        let result = serde_json::from_value::<ApiResponse<Tweet>>(json);
        assert!(result.is_err());
    }

    #[test]
    fn round_trip_serialize_deserialize() {
        let tweet = Tweet {
            id: "456".into(),
            text: "round trip".into(),
            created_at: Some("2026-01-01T00:00:00Z".into()),
            ..Default::default()
        };
        let resp = ApiResponse {
            data: tweet,
            ..Default::default()
        };
        let value = serde_json::to_value(&resp).unwrap();
        let back: ApiResponse<Tweet> = serde_json::from_value(value).unwrap();
        assert_eq!(back.data.id, "456");
        assert_eq!(back.data.text, "round trip");
    }

    #[test]
    fn deserialize_helper_rejects_empty_object() {
        let value = json!({});
        let result = deserialize_response::<Tweet>(value);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("empty response body"), "Got: {err}");
    }

    // ── Red team / adversarial ──────────────────────────────────────

    #[test]
    fn adversarial_array_where_object_expected() {
        // data is an array but we expect a single Tweet
        let json = json!({"data": [{"id": "1", "text": "oops"}]});
        let result = serde_json::from_value::<ApiResponse<Tweet>>(json);
        assert!(result.is_err(), "Should fail: array where object expected");
    }

    #[test]
    fn adversarial_string_where_object_expected() {
        let json = json!({"data": "not an object"});
        let result = serde_json::from_value::<ApiResponse<Tweet>>(json);
        assert!(result.is_err(), "Should fail: string where object expected");
    }

    #[test]
    fn adversarial_null_data_field() {
        let json = json!({"data": null});
        let result = serde_json::from_value::<ApiResponse<Tweet>>(json);
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
                    "retweet_count": 0,
                    "reply_count": 0,
                    "quote_count": 0,
                    "bookmark_count": 0,
                    "impression_count": 0
                }
            }
        });
        // This should succeed — 99_999_999_999_999 fits in u64
        let resp: ApiResponse<Tweet> = serde_json::from_value(json).unwrap();
        assert_eq!(
            resp.data.public_metrics.unwrap().like_count,
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
                    "retweet_count": 0,
                    "reply_count": 0,
                    "quote_count": 0,
                    "bookmark_count": 0,
                    "impression_count": 0
                }
            }
        });
        let result = serde_json::from_value::<ApiResponse<Tweet>>(json);
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
        let resp: ApiResponse<Tweet> = serde_json::from_value(json).unwrap();
        assert!(resp.extra.contains_key("extra_field"));
    }

    #[test]
    fn adversarial_empty_string_required_fields() {
        // Empty strings are valid String values — consumer must validate semantics
        let json = json!({"data": {"id": "", "text": ""}});
        let resp: ApiResponse<Tweet> = serde_json::from_value(json).unwrap();
        assert_eq!(resp.data.id, "");
        assert_eq!(resp.data.text, "");
    }

    #[test]
    fn adversarial_errors_field_no_data_raw_serde() {
        // Raw serde (not deserialize_response) — should fail on missing data
        let json = json!({"errors": [{"message": "forbidden"}]});
        let result = serde_json::from_value::<ApiResponse<Tweet>>(json);
        assert!(result.is_err(), "Should fail: no data field");
    }

    #[test]
    fn errors_only_response_returns_api_error() {
        // X API v2 returns {"errors": [...]} with no "data" on 200 for not-found resources.
        // deserialize_response should return XurlError::Api with the raw JSON.
        let json = json!({
            "errors": [{
                "detail": "Could not find tweet with id: [123].",
                "title": "Not Found Error",
                "resource_type": "tweet",
                "type": "https://api.twitter.com/2/problems/resource-not-found"
            }]
        });
        let result = deserialize_response::<Tweet>(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_api(), "Expected API error, got: {err}");
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
        let resp: ApiResponse<LikedResult> = serde_json::from_value(json).unwrap();
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
        let resp: ApiResponse<Tweet> = serde_json::from_value(json).unwrap();
        assert_eq!(resp.data.id, "123");
        let errors = resp.errors.unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message.as_deref(), Some("some field unavailable"));
    }

    #[test]
    fn adversarial_huge_array_in_data() {
        // Large but valid list
        let tweets: Vec<Value> = (0..1000)
            .map(|i| json!({"id": i.to_string(), "text": format!("tweet {i}")}))
            .collect();
        let json = json!({"data": tweets});
        let resp: ApiResponse<Vec<Tweet>> = serde_json::from_value(json).unwrap();
        assert_eq!(resp.data.len(), 1000);
    }

    #[test]
    fn adversarial_completely_wrong_shape() {
        // Total garbage for data
        let json = json!({"data": 42});
        let result = serde_json::from_value::<ApiResponse<Tweet>>(json);
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
        let resp: ApiResponse<Tweet> = serde_json::from_value(json).unwrap();
        let error = &resp.errors.unwrap()[0];
        assert_eq!(error.extra["new_error_field"], "surprise");
    }
}
