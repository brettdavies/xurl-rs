/// API shortcut functions — high-level X API v2 operations.
///
/// Each function maps to one of the 29 shortcut commands, building the
/// appropriate endpoint URL and request body.
use serde::Serialize;

use super::request::{ApiClient, CallOptions};
use super::response::types::{
    ApiResponse, BlockingResult, BookmarkedResult, DeletedResult, DmEvent, FollowingResult,
    LikedResult, MutingResult, RetweetedResult, Tweet, UsageData, User, deserialize_response,
};
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

// ── Shortcut methods on ApiClient ───────────────────────────────────

impl ApiClient {
    /// Creates a new post.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn create_post(
        &mut self,
        text: &str,
        media_ids: &[String],
        opts: &CallOptions,
    ) -> Result<ApiResponse<Tweet>> {
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
        let mut req = opts.to_request_options();
        req.method = "POST".to_string();
        req.endpoint = "/2/tweets".to_string();
        req.data = data;

        deserialize_response(self.send_request(&req)?)
    }

    /// Replies to an existing post.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn reply_to_post(
        &mut self,
        post_id: &str,
        text: &str,
        media_ids: &[String],
        opts: &CallOptions,
    ) -> Result<ApiResponse<Tweet>> {
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
        let mut req = opts.to_request_options();
        req.method = "POST".to_string();
        req.endpoint = "/2/tweets".to_string();
        req.data = data;

        deserialize_response(self.send_request(&req)?)
    }

    /// Quotes an existing post.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn quote_post(
        &mut self,
        post_id: &str,
        text: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<Tweet>> {
        let post_id = resolve_post_id(post_id);
        let body = PostBody {
            text: text.to_string(),
            reply: None,
            quote: Some(post_id),
            media: None,
        };

        let data = serde_json::to_string(&body)?;
        let mut req = opts.to_request_options();
        req.method = "POST".to_string();
        req.endpoint = "/2/tweets".to_string();
        req.data = data;

        deserialize_response(self.send_request(&req)?)
    }

    /// Deletes a post.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn delete_post(
        &mut self,
        post_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<DeletedResult>> {
        let post_id = resolve_post_id(post_id);
        let mut req = opts.to_request_options();
        req.method = "DELETE".to_string();
        req.endpoint = format!("/2/tweets/{post_id}");
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Reads a single post with expansions.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn read_post(&mut self, post_id: &str, opts: &CallOptions) -> Result<ApiResponse<Tweet>> {
        let post_id = resolve_post_id(post_id);
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint = format!(
            "/2/tweets/{post_id}?tweet.fields=created_at,public_metrics,conversation_id,in_reply_to_user_id,referenced_tweets,entities,attachments&expansions=author_id,referenced_tweets.id&user.fields=username,name,verified"
        );
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Searches recent posts.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn search_posts(
        &mut self,
        query: &str,
        max_results: i32,
        opts: &CallOptions,
    ) -> Result<ApiResponse<Vec<Tweet>>> {
        let q = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
        let max_results = max_results.clamp(10, 100);

        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint = format!(
            "/2/tweets/search/recent?query={q}&max_results={max_results}&tweet.fields=created_at,public_metrics,conversation_id,entities&expansions=author_id&user.fields=username,name,verified"
        );
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Fetches the authenticated user's profile.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn get_me(&mut self, opts: &CallOptions) -> Result<ApiResponse<User>> {
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint =
            "/2/users/me?user.fields=created_at,description,public_metrics,verified,profile_image_url"
                .to_string();
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Looks up a user by username.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn lookup_user(&mut self, username: &str, opts: &CallOptions) -> Result<ApiResponse<User>> {
        let username = resolve_username(username);
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint = format!(
            "/2/users/by/username/{username}?user.fields=created_at,description,public_metrics,verified,profile_image_url"
        );
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Fetches the home timeline.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn get_timeline(
        &mut self,
        user_id: &str,
        max_results: i32,
        opts: &CallOptions,
    ) -> Result<ApiResponse<Vec<Tweet>>> {
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint = format!(
            "/2/users/{user_id}/timelines/reverse_chronological?max_results={max_results}&tweet.fields=created_at,public_metrics,conversation_id,entities&expansions=author_id&user.fields=username,name"
        );
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Fetches recent mentions.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn get_mentions(
        &mut self,
        user_id: &str,
        max_results: i32,
        opts: &CallOptions,
    ) -> Result<ApiResponse<Vec<Tweet>>> {
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint = format!(
            "/2/users/{user_id}/mentions?max_results={max_results}&tweet.fields=created_at,public_metrics,conversation_id,entities&expansions=author_id&user.fields=username,name"
        );
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Likes a post.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn like_post(
        &mut self,
        user_id: &str,
        post_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<LikedResult>> {
        let post_id = resolve_post_id(post_id);
        let mut req = opts.to_request_options();
        req.method = "POST".to_string();
        req.endpoint = format!("/2/users/{user_id}/likes");
        req.data = format!(r#"{{"tweet_id":"{post_id}"}}"#);

        deserialize_response(self.send_request(&req)?)
    }

    /// Unlikes a post.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn unlike_post(
        &mut self,
        user_id: &str,
        post_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<LikedResult>> {
        let post_id = resolve_post_id(post_id);
        let mut req = opts.to_request_options();
        req.method = "DELETE".to_string();
        req.endpoint = format!("/2/users/{user_id}/likes/{post_id}");
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Reposts a post.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn repost(
        &mut self,
        user_id: &str,
        post_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<RetweetedResult>> {
        let post_id = resolve_post_id(post_id);
        let mut req = opts.to_request_options();
        req.method = "POST".to_string();
        req.endpoint = format!("/2/users/{user_id}/retweets");
        req.data = format!(r#"{{"tweet_id":"{post_id}"}}"#);

        deserialize_response(self.send_request(&req)?)
    }

    /// Removes a repost.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn unrepost(
        &mut self,
        user_id: &str,
        post_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<RetweetedResult>> {
        let post_id = resolve_post_id(post_id);
        let mut req = opts.to_request_options();
        req.method = "DELETE".to_string();
        req.endpoint = format!("/2/users/{user_id}/retweets/{post_id}");
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Bookmarks a post.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn bookmark(
        &mut self,
        user_id: &str,
        post_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<BookmarkedResult>> {
        let post_id = resolve_post_id(post_id);
        let mut req = opts.to_request_options();
        req.method = "POST".to_string();
        req.endpoint = format!("/2/users/{user_id}/bookmarks");
        req.data = format!(r#"{{"tweet_id":"{post_id}"}}"#);

        deserialize_response(self.send_request(&req)?)
    }

    /// Removes a bookmark.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn unbookmark(
        &mut self,
        user_id: &str,
        post_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<BookmarkedResult>> {
        let post_id = resolve_post_id(post_id);
        let mut req = opts.to_request_options();
        req.method = "DELETE".to_string();
        req.endpoint = format!("/2/users/{user_id}/bookmarks/{post_id}");
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Fetches bookmarks.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn get_bookmarks(
        &mut self,
        user_id: &str,
        max_results: i32,
        opts: &CallOptions,
    ) -> Result<ApiResponse<Vec<Tweet>>> {
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint = format!(
            "/2/users/{user_id}/bookmarks?max_results={max_results}&tweet.fields=created_at,public_metrics,entities&expansions=author_id&user.fields=username,name"
        );
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Follows a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn follow_user(
        &mut self,
        source_user_id: &str,
        target_user_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<FollowingResult>> {
        let mut req = opts.to_request_options();
        req.method = "POST".to_string();
        req.endpoint = format!("/2/users/{source_user_id}/following");
        req.data = format!(r#"{{"target_user_id":"{target_user_id}"}}"#);

        deserialize_response(self.send_request(&req)?)
    }

    /// Unfollows a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn unfollow_user(
        &mut self,
        source_user_id: &str,
        target_user_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<FollowingResult>> {
        let mut req = opts.to_request_options();
        req.method = "DELETE".to_string();
        req.endpoint = format!("/2/users/{source_user_id}/following/{target_user_id}");
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Fetches users that a given user follows.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn get_following(
        &mut self,
        user_id: &str,
        max_results: i32,
        opts: &CallOptions,
    ) -> Result<ApiResponse<Vec<User>>> {
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint = format!(
            "/2/users/{user_id}/following?max_results={max_results}&user.fields=created_at,description,public_metrics,verified"
        );
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Fetches followers of a given user.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn get_followers(
        &mut self,
        user_id: &str,
        max_results: i32,
        opts: &CallOptions,
    ) -> Result<ApiResponse<Vec<User>>> {
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint = format!(
            "/2/users/{user_id}/followers?max_results={max_results}&user.fields=created_at,description,public_metrics,verified"
        );
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Sends a direct message.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn send_dm(
        &mut self,
        participant_id: &str,
        text: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<DmEvent>> {
        let body = serde_json::json!({"text": text});
        let mut req = opts.to_request_options();
        req.method = "POST".to_string();
        req.endpoint = format!("/2/dm_conversations/with/{participant_id}/messages");
        req.data = serde_json::to_string(&body)?;

        deserialize_response(self.send_request(&req)?)
    }

    /// Fetches recent DM events.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn get_dm_events(
        &mut self,
        max_results: i32,
        opts: &CallOptions,
    ) -> Result<ApiResponse<Vec<DmEvent>>> {
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint = format!(
            "/2/dm_events?max_results={max_results}&dm_event.fields=created_at,dm_conversation_id,sender_id,text&expansions=sender_id&user.fields=username,name"
        );
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Fetches posts liked by a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn get_liked_posts(
        &mut self,
        user_id: &str,
        max_results: i32,
        opts: &CallOptions,
    ) -> Result<ApiResponse<Vec<Tweet>>> {
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint = format!(
            "/2/users/{user_id}/liked_tweets?max_results={max_results}&tweet.fields=created_at,public_metrics,entities&expansions=author_id&user.fields=username,name"
        );
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Blocks a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn block_user(
        &mut self,
        source_user_id: &str,
        target_user_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<BlockingResult>> {
        let mut req = opts.to_request_options();
        req.method = "POST".to_string();
        req.endpoint = format!("/2/users/{source_user_id}/blocking");
        req.data = format!(r#"{{"target_user_id":"{target_user_id}"}}"#);

        deserialize_response(self.send_request(&req)?)
    }

    /// Unblocks a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn unblock_user(
        &mut self,
        source_user_id: &str,
        target_user_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<BlockingResult>> {
        let mut req = opts.to_request_options();
        req.method = "DELETE".to_string();
        req.endpoint = format!("/2/users/{source_user_id}/blocking/{target_user_id}");
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Mutes a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn mute_user(
        &mut self,
        source_user_id: &str,
        target_user_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<MutingResult>> {
        let mut req = opts.to_request_options();
        req.method = "POST".to_string();
        req.endpoint = format!("/2/users/{source_user_id}/muting");
        req.data = format!(r#"{{"target_user_id":"{target_user_id}"}}"#);

        deserialize_response(self.send_request(&req)?)
    }

    /// Fetches API usage data (tweet caps, daily breakdowns).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn get_usage(&mut self, opts: &CallOptions) -> Result<ApiResponse<UsageData>> {
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.endpoint =
            "/2/usage/tweets?usage.fields=daily_project_usage,daily_client_app_usage".to_string();
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Unmutes a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn unmute_user(
        &mut self,
        source_user_id: &str,
        target_user_id: &str,
        opts: &CallOptions,
    ) -> Result<ApiResponse<MutingResult>> {
        let mut req = opts.to_request_options();
        req.method = "DELETE".to_string();
        req.endpoint = format!("/2/users/{source_user_id}/muting/{target_user_id}");
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }
}
