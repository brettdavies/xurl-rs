//! API shortcut functions — high-level X API v2 operations.
//!
//! Each function maps to one of the 27 shortcut commands, building the
//! appropriate endpoint target and request body. Paths are spec-shaped
//! templates (e.g. `/2/users/{id}/likes`) so the auth-matrix validator
//! can key on the same string the build-time codegen ingests.

use std::collections::HashMap;

use serde::Serialize;

use super::request::{ApiClient, CallOptions, RequestTarget};
use super::response::types::{
    ApiResponse, BookmarkedResult, DeletedResult, DmEvent, FollowingResult, LikedResult,
    MutingResult, Post, RetweetedResult, UsageData, User, deserialize_response,
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

// ── Write-op validators (U7) ─────────────────────────────────────────
//
// Each validator returns `Ok(())` when the inputs would be accepted by the
// API and `Err(reason)` with a kebab-case reason otherwise. Callers compose
// these into the canonical dry-run envelope without issuing HTTP.

/// X API post body length budget. The platform rejects > 280 chars.
pub const POST_BODY_MAX_CHARS: usize = 280;

/// X API media attachment cap per post.
pub const POST_MEDIA_MAX: usize = 4;

/// Validates a post / reply / quote body.
///
/// # Errors
/// Returns the kebab-case reason: `empty-body`, `body-too-long`.
pub fn validate_post_body(text: &str) -> std::result::Result<(), &'static str> {
    if text.is_empty() {
        return Err("empty-body");
    }
    if text.chars().count() > POST_BODY_MAX_CHARS {
        return Err("body-too-long");
    }
    Ok(())
}

/// Validates a media-attachment list for a post.
///
/// # Errors
/// Returns `too-many-attachments` when more than [`POST_MEDIA_MAX`] are passed.
pub fn validate_media_attachments(ids: &[String]) -> std::result::Result<(), &'static str> {
    if ids.len() > POST_MEDIA_MAX {
        return Err("too-many-attachments");
    }
    Ok(())
}

/// Validates a DM body.
///
/// # Errors
/// Returns `empty-body` for an empty string. (DM length is enforced server-side.)
pub fn validate_dm_body(text: &str) -> std::result::Result<(), &'static str> {
    if text.is_empty() {
        return Err("empty-body");
    }
    Ok(())
}

/// Validates a username target (strips a leading `@` first).
///
/// # Errors
/// Returns `empty-username` when the trimmed input is empty.
pub fn validate_target_username(input: &str) -> std::result::Result<(), &'static str> {
    if resolve_username(input).is_empty() {
        return Err("empty-username");
    }
    Ok(())
}

/// Validates a post identifier (URL or ID).
///
/// # Errors
/// Returns `empty-post-id` when the resolver yields the empty string.
pub fn validate_post_id(input: &str) -> std::result::Result<(), &'static str> {
    if resolve_post_id(input).is_empty() {
        return Err("empty-post-id");
    }
    Ok(())
}

// ── Shortcut methods on ApiClient ───────────────────────────────────

/// Returns a base query vec for a paginated shortcut.
///
/// When the caller threaded a cursor into [`CallOptions::pagination_token`],
/// emits `[("pagination_token", token)]`; otherwise empty. The percent-
/// encoding happens at URL render time in `build_url_for_target`.
fn build_query(opts: &CallOptions) -> Vec<(String, String)> {
    if opts.pagination_token.is_empty() {
        Vec::new()
    } else {
        vec![(
            "pagination_token".to_string(),
            opts.pagination_token.clone(),
        )]
    }
}

impl ApiClient {
    /// Creates a new post.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use xurl::api::{ApiClient, CallOptions};
    /// use xurl::auth::Auth;
    /// use xurl::config::Config;
    ///
    /// let cfg = Config::new();
    /// let auth = Auth::new(&cfg);
    /// let mut client = ApiClient::new(&cfg, auth);
    ///
    /// let resp = client.create_post("hello from xurl", &[], &CallOptions::default())?;
    /// println!("created post id={}", resp.data.id);
    /// # Ok::<(), xurl::error::XurlError>(())
    /// ```
    pub fn create_post(
        &mut self,
        text: &str,
        media_ids: &[String],
        opts: &CallOptions,
    ) -> Result<ApiResponse<Post>> {
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
        req.target = RequestTarget::Template {
            path: "/2/tweets".to_string(),
            path_params: HashMap::new(),
            query: Vec::new(),
        };
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
    ) -> Result<ApiResponse<Post>> {
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
        req.target = RequestTarget::Template {
            path: "/2/tweets".to_string(),
            path_params: HashMap::new(),
            query: Vec::new(),
        };
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
    ) -> Result<ApiResponse<Post>> {
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
        req.target = RequestTarget::Template {
            path: "/2/tweets".to_string(),
            path_params: HashMap::new(),
            query: Vec::new(),
        };
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
        req.target = RequestTarget::Template {
            path: "/2/tweets/{id}".to_string(),
            path_params: HashMap::from([("id".to_string(), post_id)]),
            query: Vec::new(),
        };
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Reads a single post with expansions.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    pub fn read_post(&mut self, post_id: &str, opts: &CallOptions) -> Result<ApiResponse<Post>> {
        let post_id = resolve_post_id(post_id);
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.target = RequestTarget::Template {
            path: "/2/tweets/{id}".to_string(),
            path_params: HashMap::from([("id".to_string(), post_id)]),
            query: vec![
                (
                    "post.fields".to_string(),
                    "created_at,public_metrics,conversation_id,entities,attachments".to_string(),
                ),
                (
                    "expansions".to_string(),
                    "author_id,in_reply_to_user_id,referenced_posts".to_string(),
                ),
                (
                    "user.fields".to_string(),
                    "username,name,verified".to_string(),
                ),
            ],
        };
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Searches recent posts.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use xurl::api::{ApiClient, CallOptions};
    /// use xurl::auth::Auth;
    /// use xurl::config::Config;
    ///
    /// let cfg = Config::new();
    /// let auth = Auth::new(&cfg);
    /// let mut client = ApiClient::new(&cfg, auth);
    ///
    /// let resp = client.search_posts("rustlang", 25, &CallOptions::default())?;
    /// for post in &resp.data {
    ///     println!("{}: {}", post.id, post.text);
    /// }
    /// # Ok::<(), xurl::error::XurlError>(())
    /// ```
    pub fn search_posts(
        &mut self,
        query: &str,
        max_results: i32,
        opts: &CallOptions,
    ) -> Result<ApiResponse<Vec<Post>>> {
        let max_results = max_results.clamp(10, 100);

        let mut q = vec![
            ("query".to_string(), query.to_string()),
            ("max_results".to_string(), max_results.to_string()),
            (
                "post.fields".to_string(),
                "created_at,public_metrics,conversation_id,entities".to_string(),
            ),
            ("expansions".to_string(), "author_id".to_string()),
            (
                "user.fields".to_string(),
                "username,name,verified".to_string(),
            ),
        ];
        q.extend(build_query(opts));

        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.target = RequestTarget::Template {
            path: "/2/tweets/search/recent".to_string(),
            path_params: HashMap::new(),
            query: q,
        };
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }

    /// Fetches the authenticated user's profile.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use xurl::api::{ApiClient, CallOptions};
    /// use xurl::auth::Auth;
    /// use xurl::config::Config;
    ///
    /// let cfg = Config::new();
    /// let auth = Auth::new(&cfg);
    /// let mut client = ApiClient::new(&cfg, auth);
    ///
    /// let resp = client.get_me(&CallOptions::default())?;
    /// println!("@{} ({})", resp.data.username, resp.data.id);
    /// # Ok::<(), xurl::error::XurlError>(())
    /// ```
    pub fn get_me(&mut self, opts: &CallOptions) -> Result<ApiResponse<User>> {
        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.target = RequestTarget::Template {
            path: "/2/users/me".to_string(),
            path_params: HashMap::new(),
            query: vec![(
                "user.fields".to_string(),
                "created_at,description,public_metrics,verified,profile_image_url".to_string(),
            )],
        };
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
        req.target = RequestTarget::Template {
            path: "/2/users/by/username/{username}".to_string(),
            path_params: HashMap::from([("username".to_string(), username)]),
            query: vec![(
                "user.fields".to_string(),
                "created_at,description,public_metrics,verified,profile_image_url".to_string(),
            )],
        };
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
    ) -> Result<ApiResponse<Vec<Post>>> {
        let mut q = vec![
            ("max_results".to_string(), max_results.to_string()),
            (
                "post.fields".to_string(),
                "created_at,public_metrics,conversation_id,entities".to_string(),
            ),
            ("expansions".to_string(), "author_id".to_string()),
            ("user.fields".to_string(), "username,name".to_string()),
        ];
        q.extend(build_query(opts));

        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/timelines/reverse_chronological".to_string(),
            path_params: HashMap::from([("id".to_string(), user_id.to_string())]),
            query: q,
        };
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
    ) -> Result<ApiResponse<Vec<Post>>> {
        let mut q = vec![
            ("max_results".to_string(), max_results.to_string()),
            (
                "post.fields".to_string(),
                "created_at,public_metrics,conversation_id,entities".to_string(),
            ),
            ("expansions".to_string(), "author_id".to_string()),
            ("user.fields".to_string(), "username,name".to_string()),
        ];
        q.extend(build_query(opts));

        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/mentions".to_string(),
            path_params: HashMap::from([("id".to_string(), user_id.to_string())]),
            query: q,
        };
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
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/likes".to_string(),
            path_params: HashMap::from([("id".to_string(), user_id.to_string())]),
            query: Vec::new(),
        };
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
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/likes/{tweet_id}".to_string(),
            path_params: HashMap::from([
                ("id".to_string(), user_id.to_string()),
                ("tweet_id".to_string(), post_id),
            ]),
            query: Vec::new(),
        };
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
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/retweets".to_string(),
            path_params: HashMap::from([("id".to_string(), user_id.to_string())]),
            query: Vec::new(),
        };
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
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/retweets/{source_tweet_id}".to_string(),
            path_params: HashMap::from([
                ("id".to_string(), user_id.to_string()),
                ("source_tweet_id".to_string(), post_id),
            ]),
            query: Vec::new(),
        };
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
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/bookmarks".to_string(),
            path_params: HashMap::from([("id".to_string(), user_id.to_string())]),
            query: Vec::new(),
        };
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
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/bookmarks/{tweet_id}".to_string(),
            path_params: HashMap::from([
                ("id".to_string(), user_id.to_string()),
                ("tweet_id".to_string(), post_id),
            ]),
            query: Vec::new(),
        };
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
    ) -> Result<ApiResponse<Vec<Post>>> {
        let mut q = vec![
            ("max_results".to_string(), max_results.to_string()),
            (
                "post.fields".to_string(),
                "created_at,public_metrics,entities".to_string(),
            ),
            ("expansions".to_string(), "author_id".to_string()),
            ("user.fields".to_string(), "username,name".to_string()),
        ];
        q.extend(build_query(opts));

        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/bookmarks".to_string(),
            path_params: HashMap::from([("id".to_string(), user_id.to_string())]),
            query: q,
        };
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
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/following".to_string(),
            path_params: HashMap::from([("id".to_string(), source_user_id.to_string())]),
            query: Vec::new(),
        };
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
        req.target = RequestTarget::Template {
            path: "/2/users/{source_user_id}/following/{target_user_id}".to_string(),
            path_params: HashMap::from([
                ("source_user_id".to_string(), source_user_id.to_string()),
                ("target_user_id".to_string(), target_user_id.to_string()),
            ]),
            query: Vec::new(),
        };
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
        let mut q = vec![
            ("max_results".to_string(), max_results.to_string()),
            (
                "user.fields".to_string(),
                "created_at,description,public_metrics,verified".to_string(),
            ),
        ];
        q.extend(build_query(opts));

        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/following".to_string(),
            path_params: HashMap::from([("id".to_string(), user_id.to_string())]),
            query: q,
        };
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
        let mut q = vec![
            ("max_results".to_string(), max_results.to_string()),
            (
                "user.fields".to_string(),
                "created_at,description,public_metrics,verified".to_string(),
            ),
        ];
        q.extend(build_query(opts));

        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/followers".to_string(),
            path_params: HashMap::from([("id".to_string(), user_id.to_string())]),
            query: q,
        };
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
        req.target = RequestTarget::Template {
            path: "/2/dm_conversations/with/{participant_id}/messages".to_string(),
            path_params: HashMap::from([(
                "participant_id".to_string(),
                participant_id.to_string(),
            )]),
            query: Vec::new(),
        };
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
        let mut q = vec![
            ("max_results".to_string(), max_results.to_string()),
            (
                "dm_event.fields".to_string(),
                "created_at,dm_conversation_id,text".to_string(),
            ),
            ("expansions".to_string(), "sender_id".to_string()),
            ("user.fields".to_string(), "username,name".to_string()),
        ];
        q.extend(build_query(opts));

        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.target = RequestTarget::Template {
            path: "/2/dm_events".to_string(),
            path_params: HashMap::new(),
            query: q,
        };
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
    ) -> Result<ApiResponse<Vec<Post>>> {
        let mut q = vec![
            ("max_results".to_string(), max_results.to_string()),
            (
                "post.fields".to_string(),
                "created_at,public_metrics,entities".to_string(),
            ),
            ("expansions".to_string(), "author_id".to_string()),
            ("user.fields".to_string(), "username,name".to_string()),
        ];
        q.extend(build_query(opts));

        let mut req = opts.to_request_options();
        req.method = "GET".to_string();
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/liked_tweets".to_string(),
            path_params: HashMap::from([("id".to_string(), user_id.to_string())]),
            query: q,
        };
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
        req.target = RequestTarget::Template {
            path: "/2/users/{id}/muting".to_string(),
            path_params: HashMap::from([("id".to_string(), source_user_id.to_string())]),
            query: Vec::new(),
        };
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
        req.target = RequestTarget::Template {
            path: "/2/usage/tweets".to_string(),
            path_params: HashMap::new(),
            query: vec![(
                "usage.fields".to_string(),
                "daily_project_usage,daily_client_app_usage".to_string(),
            )],
        };
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
        req.target = RequestTarget::Template {
            path: "/2/users/{source_user_id}/muting/{target_user_id}".to_string(),
            path_params: HashMap::from([
                ("source_user_id".to_string(), source_user_id.to_string()),
                ("target_user_id".to_string(), target_user_id.to_string()),
            ]),
            query: Vec::new(),
        };
        req.data.clear();

        deserialize_response(self.send_request(&req)?)
    }
}
