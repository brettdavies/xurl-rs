/// API shortcut functions — high-level X API v2 operations.
///
/// Each function maps to one of the 28 shortcut commands, building the
/// appropriate endpoint URL and request body.
use serde::Serialize;

use super::request::{ApiClient, RequestOptions};
use crate::error::Result;

// ── Request body types ───────────────────────────────────────────────

#[derive(Serialize)]
struct PostBody {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply: Option<PostReply>,
    #[serde(rename = "quote_tweet_id", skip_serializing_if = "Option::is_none")]
    quote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media: Option<PostMedia>,
}

#[derive(Serialize)]
struct PostReply {
    #[serde(rename = "in_reply_to_tweet_id")]
    in_reply_to_post_id: String,
}

#[derive(Serialize)]
struct PostMedia {
    media_ids: Vec<String>,
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Extracts a post ID from a full URL or returns the input as-is.
#[must_use]
pub fn resolve_post_id(input: &str) -> String {
    let input = input.trim();

    if (input.starts_with("http://") || input.starts_with("https://"))
        && let Ok(parsed) = url::Url::parse(input)
    {
        let parts: Vec<&str> = parsed.path().trim_matches('/').split('/').collect();
        for (i, p) in parts.iter().enumerate() {
            if *p == "status" && i + 1 < parts.len() {
                return parts[i + 1].to_string();
            }
        }
    }
    input.to_string()
}

/// Normalizes a username — strips a leading "@" if present.
#[must_use]
pub fn resolve_username(input: &str) -> String {
    input.trim().trim_start_matches('@').to_string()
}

// ── Shortcut executors ───────────────────────────────────────────────

/// Creates a new post.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn create_post(
    client: &mut ApiClient,
    text: &str,
    media_ids: &[String],
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut body = PostBody {
        text: text.to_string(),
        reply: None,
        quote: None,
        media: None,
    };
    if !media_ids.is_empty() {
        body.media = Some(PostMedia {
            media_ids: media_ids.to_vec(),
        });
    }

    let data = serde_json::to_string(&body)?;
    let mut opts = opts.clone();
    opts.method = "POST".to_string();
    opts.endpoint = "/2/tweets".to_string();
    opts.data = data;

    client.send_request(&opts)
}

/// Replies to an existing post.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn reply_to_post(
    client: &mut ApiClient,
    post_id: &str,
    text: &str,
    media_ids: &[String],
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let post_id = resolve_post_id(post_id);
    let mut body = PostBody {
        text: text.to_string(),
        reply: Some(PostReply {
            in_reply_to_post_id: post_id,
        }),
        quote: None,
        media: None,
    };
    if !media_ids.is_empty() {
        body.media = Some(PostMedia {
            media_ids: media_ids.to_vec(),
        });
    }

    let data = serde_json::to_string(&body)?;
    let mut opts = opts.clone();
    opts.method = "POST".to_string();
    opts.endpoint = "/2/tweets".to_string();
    opts.data = data;

    client.send_request(&opts)
}

/// Quotes an existing post.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn quote_post(
    client: &mut ApiClient,
    post_id: &str,
    text: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let post_id = resolve_post_id(post_id);
    let body = PostBody {
        text: text.to_string(),
        reply: None,
        quote: Some(post_id),
        media: None,
    };

    let data = serde_json::to_string(&body)?;
    let mut opts = opts.clone();
    opts.method = "POST".to_string();
    opts.endpoint = "/2/tweets".to_string();
    opts.data = data;

    client.send_request(&opts)
}

/// Deletes a post.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn delete_post(
    client: &mut ApiClient,
    post_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let post_id = resolve_post_id(post_id);
    let mut opts = opts.clone();
    opts.method = "DELETE".to_string();
    opts.endpoint = format!("/2/tweets/{post_id}");
    opts.data.clear();

    client.send_request(&opts)
}

/// Reads a single post with expansions.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn read_post(
    client: &mut ApiClient,
    post_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let post_id = resolve_post_id(post_id);
    let mut opts = opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint = format!(
        "/2/tweets/{post_id}?tweet.fields=created_at,public_metrics,conversation_id,in_reply_to_user_id,referenced_tweets,entities,attachments&expansions=author_id,referenced_tweets.id&user.fields=username,name,verified"
    );
    opts.data.clear();

    client.send_request(&opts)
}

/// Searches recent posts.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn search_posts(
    client: &mut ApiClient,
    query: &str,
    max_results: i32,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let q = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
    let max_results = max_results.clamp(10, 100);

    let mut opts = opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint = format!(
        "/2/tweets/search/recent?query={q}&max_results={max_results}&tweet.fields=created_at,public_metrics,conversation_id,entities&expansions=author_id&user.fields=username,name,verified"
    );
    opts.data.clear();

    client.send_request(&opts)
}

/// Fetches the authenticated user's profile.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn get_me(client: &mut ApiClient, opts: &RequestOptions) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint =
        "/2/users/me?user.fields=created_at,description,public_metrics,verified,profile_image_url"
            .to_string();
    opts.data.clear();

    client.send_request(&opts)
}

/// Looks up a user by username.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn lookup_user(
    client: &mut ApiClient,
    username: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let username = resolve_username(username);
    let mut opts = opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint = format!(
        "/2/users/by/username/{username}?user.fields=created_at,description,public_metrics,verified,profile_image_url"
    );
    opts.data.clear();

    client.send_request(&opts)
}

/// Fetches the home timeline.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn get_timeline(
    client: &mut ApiClient,
    user_id: &str,
    max_results: i32,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint = format!(
        "/2/users/{user_id}/timelines/reverse_chronological?max_results={max_results}&tweet.fields=created_at,public_metrics,conversation_id,entities&expansions=author_id&user.fields=username,name"
    );
    opts.data.clear();

    client.send_request(&opts)
}

/// Fetches recent mentions.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn get_mentions(
    client: &mut ApiClient,
    user_id: &str,
    max_results: i32,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint = format!(
        "/2/users/{user_id}/mentions?max_results={max_results}&tweet.fields=created_at,public_metrics,conversation_id,entities&expansions=author_id&user.fields=username,name"
    );
    opts.data.clear();

    client.send_request(&opts)
}

/// Likes a post.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn like_post(
    client: &mut ApiClient,
    user_id: &str,
    post_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let post_id = resolve_post_id(post_id);
    let mut opts = opts.clone();
    opts.method = "POST".to_string();
    opts.endpoint = format!("/2/users/{user_id}/likes");
    opts.data = format!(r#"{{"tweet_id":"{post_id}"}}"#);

    client.send_request(&opts)
}

/// Unlikes a post.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn unlike_post(
    client: &mut ApiClient,
    user_id: &str,
    post_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let post_id = resolve_post_id(post_id);
    let mut opts = opts.clone();
    opts.method = "DELETE".to_string();
    opts.endpoint = format!("/2/users/{user_id}/likes/{post_id}");
    opts.data.clear();

    client.send_request(&opts)
}

/// Reposts a post.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn repost(
    client: &mut ApiClient,
    user_id: &str,
    post_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let post_id = resolve_post_id(post_id);
    let mut opts = opts.clone();
    opts.method = "POST".to_string();
    opts.endpoint = format!("/2/users/{user_id}/retweets");
    opts.data = format!(r#"{{"tweet_id":"{post_id}"}}"#);

    client.send_request(&opts)
}

/// Removes a repost.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn unrepost(
    client: &mut ApiClient,
    user_id: &str,
    post_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let post_id = resolve_post_id(post_id);
    let mut opts = opts.clone();
    opts.method = "DELETE".to_string();
    opts.endpoint = format!("/2/users/{user_id}/retweets/{post_id}");
    opts.data.clear();

    client.send_request(&opts)
}

/// Bookmarks a post.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn bookmark(
    client: &mut ApiClient,
    user_id: &str,
    post_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let post_id = resolve_post_id(post_id);
    let mut opts = opts.clone();
    opts.method = "POST".to_string();
    opts.endpoint = format!("/2/users/{user_id}/bookmarks");
    opts.data = format!(r#"{{"tweet_id":"{post_id}"}}"#);

    client.send_request(&opts)
}

/// Removes a bookmark.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn unbookmark(
    client: &mut ApiClient,
    user_id: &str,
    post_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let post_id = resolve_post_id(post_id);
    let mut opts = opts.clone();
    opts.method = "DELETE".to_string();
    opts.endpoint = format!("/2/users/{user_id}/bookmarks/{post_id}");
    opts.data.clear();

    client.send_request(&opts)
}

/// Fetches bookmarks.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn get_bookmarks(
    client: &mut ApiClient,
    user_id: &str,
    max_results: i32,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint = format!(
        "/2/users/{user_id}/bookmarks?max_results={max_results}&tweet.fields=created_at,public_metrics,entities&expansions=author_id&user.fields=username,name"
    );
    opts.data.clear();

    client.send_request(&opts)
}

/// Follows a user.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn follow_user(
    client: &mut ApiClient,
    source_user_id: &str,
    target_user_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "POST".to_string();
    opts.endpoint = format!("/2/users/{source_user_id}/following");
    opts.data = format!(r#"{{"target_user_id":"{target_user_id}"}}"#);

    client.send_request(&opts)
}

/// Unfollows a user.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn unfollow_user(
    client: &mut ApiClient,
    source_user_id: &str,
    target_user_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "DELETE".to_string();
    opts.endpoint = format!("/2/users/{source_user_id}/following/{target_user_id}");
    opts.data.clear();

    client.send_request(&opts)
}

/// Fetches users that a given user follows.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn get_following(
    client: &mut ApiClient,
    user_id: &str,
    max_results: i32,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint = format!(
        "/2/users/{user_id}/following?max_results={max_results}&user.fields=created_at,description,public_metrics,verified"
    );
    opts.data.clear();

    client.send_request(&opts)
}

/// Fetches followers of a given user.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn get_followers(
    client: &mut ApiClient,
    user_id: &str,
    max_results: i32,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint = format!(
        "/2/users/{user_id}/followers?max_results={max_results}&user.fields=created_at,description,public_metrics,verified"
    );
    opts.data.clear();

    client.send_request(&opts)
}

/// Sends a direct message.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn send_dm(
    client: &mut ApiClient,
    participant_id: &str,
    text: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let body = serde_json::json!({"text": text});
    let mut opts = opts.clone();
    opts.method = "POST".to_string();
    opts.endpoint = format!("/2/dm_conversations/with/{participant_id}/messages");
    opts.data = serde_json::to_string(&body)?;

    client.send_request(&opts)
}

/// Fetches recent DM events.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn get_dm_events(
    client: &mut ApiClient,
    max_results: i32,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint = format!(
        "/2/dm_events?max_results={max_results}&dm_event.fields=created_at,dm_conversation_id,sender_id,text&expansions=sender_id&user.fields=username,name"
    );
    opts.data.clear();

    client.send_request(&opts)
}

/// Fetches posts liked by a user.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn get_liked_posts(
    client: &mut ApiClient,
    user_id: &str,
    max_results: i32,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint = format!(
        "/2/users/{user_id}/liked_tweets?max_results={max_results}&tweet.fields=created_at,public_metrics,entities&expansions=author_id&user.fields=username,name"
    );
    opts.data.clear();

    client.send_request(&opts)
}

/// Blocks a user.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn block_user(
    client: &mut ApiClient,
    source_user_id: &str,
    target_user_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "POST".to_string();
    opts.endpoint = format!("/2/users/{source_user_id}/blocking");
    opts.data = format!(r#"{{"target_user_id":"{target_user_id}"}}"#);

    client.send_request(&opts)
}

/// Unblocks a user.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn unblock_user(
    client: &mut ApiClient,
    source_user_id: &str,
    target_user_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "DELETE".to_string();
    opts.endpoint = format!("/2/users/{source_user_id}/blocking/{target_user_id}");
    opts.data.clear();

    client.send_request(&opts)
}

/// Mutes a user.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn mute_user(
    client: &mut ApiClient,
    source_user_id: &str,
    target_user_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "POST".to_string();
    opts.endpoint = format!("/2/users/{source_user_id}/muting");
    opts.data = format!(r#"{{"target_user_id":"{target_user_id}"}}"#);

    client.send_request(&opts)
}

/// Unmutes a user.
///
/// # Errors
///
/// Returns an error if the request fails or the API returns an error.
pub fn unmute_user(
    client: &mut ApiClient,
    source_user_id: &str,
    target_user_id: &str,
    opts: &RequestOptions,
) -> Result<serde_json::Value> {
    let mut opts = opts.clone();
    opts.method = "DELETE".to_string();
    opts.endpoint = format!("/2/users/{source_user_id}/muting/{target_user_id}");
    opts.data.clear();

    client.send_request(&opts)
}
