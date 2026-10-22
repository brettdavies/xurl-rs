//! API shortcut functions — high-level X API v2 operations.
//!
//! Each method maps to one shortcut command, building the appropriate
//! endpoint target and request body into a [`Call`] the caller configures
//! and sends. Paths are spec-shaped templates (e.g.
//! `/2/users/{id}/likes`) so the auth-matrix validator can key on the same
//! string the build-time codegen ingests.

use std::collections::HashMap;

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::auth_matrix::endpoints;
use super::request::{Call, Client, RequestOptions, RequestTarget};
use super::response::types::{
    ApiResponse, BlockingResult, BookmarkedResult, ChatModeratorsResult, DeletedResult, DmEvent,
    DmSentResult, FollowingResult, LikedResult, MediaMetadataResult, MediaSubtitlesResult,
    MutingResult, Post, RepostedResult, UsageCreditsData, UsageData, User, Webhook,
    WebhookReplayJob, WebhookValidation,
};

// ── Request body types ───────────────────────────────────────────────

#[derive(Serialize)]
struct WebhookBody {
    url: String,
}

#[derive(Serialize)]
struct WebhookReplayBody {
    webhook_id: String,
    from_date: String,
    to_date: String,
}

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

#[derive(Serialize)]
struct ChatModeratorBody {
    user_id: String,
}

#[derive(Serialize)]
struct MediaMetadataBody {
    id: String,
    metadata: MediaMetadata,
}

#[derive(Serialize)]
struct MediaMetadata {
    alt_text: AltText,
}

#[derive(Serialize)]
struct AltText {
    text: String,
}

#[derive(Serialize)]
struct AddSubtitlesBody {
    id: String,
    media_category: &'static str,
    subtitles: SubtitleTrack,
}

#[derive(Serialize)]
struct SubtitleTrack {
    id: String,
    language_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
}

#[derive(Serialize)]
struct RemoveSubtitlesBody {
    id: String,
    media_category: &'static str,
    language_code: String,
}

/// The upload category of a video that carries subtitles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCategory {
    /// A video uploaded with `media_category=amplify_video`.
    AmplifyVideo,
    /// A video uploaded with `media_category=tweet_video`.
    TweetVideo,
}

impl VideoCategory {
    /// Every category, in the order help text lists them.
    pub const ALL: [Self; 2] = [Self::AmplifyVideo, Self::TweetVideo];

    /// The `media_category` the video was uploaded with, such as
    /// `amplify_video`.
    #[must_use]
    pub const fn upload_name(self) -> &'static str {
        match self {
            Self::AmplifyVideo => "amplify_video",
            Self::TweetVideo => "tweet_video",
        }
    }

    /// The category from its upload name, the inverse of
    /// [`Self::upload_name`].
    #[must_use]
    pub fn from_upload_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.upload_name() == name)
    }

    /// The form the subtitles endpoints take, such as `AmplifyVideo`.
    const fn wire_name(self) -> &'static str {
        match self {
            Self::AmplifyVideo => "AmplifyVideo",
            Self::TweetVideo => "TweetVideo",
        }
    }
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

// ── Write-op validators ─────────────────────────────────────────
//
// Each validator returns `Ok(())` when the inputs would be accepted by the
// API and `Err(reason)` with a kebab-case reason otherwise. Callers compose
// these into the canonical dry-run envelope without issuing HTTP.

/// The post length X documents for a standard account.
#[deprecated(
    note = "X sets a post's length limit per account, so `validate_post_body` no longer checks it"
)]
pub const POST_BODY_MAX_CHARS: usize = 280;

/// X API media attachment cap per post.
pub const POST_MEDIA_MAX: usize = 4;

/// Validates a post / reply / quote body.
///
/// Length is left to X: the limit belongs to the account (a Premium account
/// posts past 280 characters) and X counts a URL as 23, so a character count
/// taken here refuses posts X accepts.
///
/// # Errors
/// Returns the kebab-case reason `empty-body`.
pub fn validate_post_body(text: &str) -> std::result::Result<(), &'static str> {
    if text.is_empty() {
        return Err("empty-body");
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

/// X API alt text length budget, the spec's `maxLength` on `alt_text.text`.
pub const ALT_TEXT_MAX_CHARS: usize = 1000;

/// Validates a media id: one to nineteen ASCII digits, the spec's pattern.
///
/// # Errors
/// Returns `invalid-media-id` for anything else, the empty string included.
pub fn validate_media_id(input: &str) -> std::result::Result<(), &'static str> {
    let id = input.trim();
    if id.is_empty() || id.len() > 19 || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid-media-id");
    }
    Ok(())
}

/// Longest webhook URL X accepts, the spec's `maxLength` on the `url` of a
/// webhook registration.
pub const WEBHOOK_URL_MAX_CHARS: usize = 200;

/// Validates a webhook id: one to nineteen ASCII digits, the spec's pattern.
///
/// # Errors
/// Returns `invalid-webhook-id` for anything else, the empty string included.
pub fn validate_webhook_id(input: &str) -> std::result::Result<(), &'static str> {
    if input.is_empty() || input.len() > 19 || !input.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid-webhook-id");
    }
    Ok(())
}

/// Validates a webhook URL against the spec's length bounds.
///
/// # Errors
/// Returns the kebab-case reason: `empty-webhook-url`, `webhook-url-too-long`.
pub fn validate_webhook_url(url: &str) -> std::result::Result<(), &'static str> {
    if url.is_empty() {
        return Err("empty-webhook-url");
    }
    if url.chars().count() > WEBHOOK_URL_MAX_CHARS {
        return Err("webhook-url-too-long");
    }
    Ok(())
}

/// Validates one bound of a webhook replay window: twelve ASCII digits,
/// `yyyymmddhhmm` in UTC, the spec's pattern.
///
/// # Errors
/// Returns `invalid-replay-time` for anything else.
pub fn validate_replay_time(input: &str) -> std::result::Result<(), &'static str> {
    if input.len() != 12 || !input.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid-replay-time");
    }
    Ok(())
}

/// Validates the alt text for a media id.
///
/// # Errors
/// Returns the kebab-case reason: `empty-alt-text`, `alt-text-too-long`.
pub fn validate_alt_text(text: &str) -> std::result::Result<(), &'static str> {
    if text.trim().is_empty() {
        return Err("empty-alt-text");
    }
    if text.chars().count() > ALT_TEXT_MAX_CHARS {
        return Err("alt-text-too-long");
    }
    Ok(())
}

/// Validates a subtitle language code: two ASCII letters, in either case.
///
/// # Errors
/// Returns `invalid-language-code` for anything else.
pub fn validate_language_code(code: &str) -> std::result::Result<(), &'static str> {
    let code = code.trim();
    if code.len() != 2 || !code.bytes().all(|b| b.is_ascii_alphabetic()) {
        return Err("invalid-language-code");
    }
    Ok(())
}

// ── Request construction ─────────────────────────────────────────────

fn template(
    path: &str,
    path_params: HashMap<String, String>,
    query: Vec<(String, String)>,
) -> RequestTarget {
    RequestTarget::Template {
        path: path.to_string(),
        path_params,
        query,
    }
}

fn request(method: &str, target: RequestTarget, data: String) -> RequestOptions {
    RequestOptions {
        method: method.to_string(),
        target,
        data,
        ..Default::default()
    }
}

fn id_param(user_id: &str) -> HashMap<String, String> {
    HashMap::from([("id".to_string(), user_id.to_string())])
}

fn id_and(user_id: &str, key: &str, value: String) -> HashMap<String, String> {
    HashMap::from([
        ("id".to_string(), user_id.to_string()),
        (key.to_string(), value),
    ])
}

fn source_and_target(source_user_id: &str, target_user_id: &str) -> HashMap<String, String> {
    HashMap::from([
        ("source_user_id".to_string(), source_user_id.to_string()),
        ("target_user_id".to_string(), target_user_id.to_string()),
    ])
}

fn target_user_body(target_user_id: &str) -> String {
    format!(r#"{{"target_user_id":"{target_user_id}"}}"#)
}

fn post_id_body(post_id: &str) -> String {
    format!(r#"{{"tweet_id":"{post_id}"}}"#)
}

impl Client {
    fn call<T: DeserializeOwned>(&self, method: &str, target: RequestTarget) -> Call<T> {
        Call::new(self, request(method, target, String::new()))
    }

    fn call_with_body<T: DeserializeOwned>(
        &self,
        method: &str,
        target: RequestTarget,
        data: String,
    ) -> Call<T> {
        Call::new(self, request(method, target, data))
    }

    /// A POST whose body is `body` serialized as JSON.
    fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        target: RequestTarget,
        body: &B,
    ) -> Call<T> {
        self.json_call("POST", target, body)
    }

    /// A `method` call whose body is `body` serialized as JSON; a body that
    /// cannot be serialized surfaces when the call is sent.
    fn json_call<T: DeserializeOwned, B: Serialize>(
        &self,
        method: &str,
        target: RequestTarget,
        body: &B,
    ) -> Call<T> {
        match serde_json::to_string(body) {
            Ok(data) => self.call_with_body(method, target, data),
            Err(e) => Call::failed(self, e.into()),
        }
    }
}

// ── Shortcut methods on Client ───────────────────────────────────────

impl Client {
    /// Creates a new post.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use xdk::api::Client;
    /// use xdk::auth::OAuth2Credential;
    ///
    /// # async fn run(credential: OAuth2Credential) -> xdk::Result<()> {
    /// let client = Client::builder().oauth2(credential).build()?;
    /// let resp = client.create_post("hello from xdk", &[]).send().await?;
    /// println!("created post id={}", resp.data.id);
    /// # Ok(()) }
    /// ```
    pub fn create_post(&self, text: &str, media_ids: &[String]) -> Call<ApiResponse<Post>> {
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
        self.post_json(
            template(endpoints::CREATE_POST.path, HashMap::new(), Vec::new()),
            &body,
        )
    }

    /// Replies to an existing post.
    pub fn reply_to_post(
        &self,
        post_id: &str,
        text: &str,
        media_ids: &[String],
    ) -> Call<ApiResponse<Post>> {
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
        self.post_json(
            template(endpoints::CREATE_POST.path, HashMap::new(), Vec::new()),
            &body,
        )
    }

    /// Quotes an existing post.
    pub fn quote_post(&self, post_id: &str, text: &str) -> Call<ApiResponse<Post>> {
        let post_id = resolve_post_id(post_id);
        let body = PostBody {
            text: text.to_string(),
            reply: None,
            quote: Some(post_id),
            media: None,
        };
        self.post_json(
            template(endpoints::CREATE_POST.path, HashMap::new(), Vec::new()),
            &body,
        )
    }

    /// Deletes a post.
    pub fn delete_post(&self, post_id: &str) -> Call<ApiResponse<DeletedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call(
            endpoints::DELETE_POST.method,
            template(
                endpoints::DELETE_POST.path,
                HashMap::from([("id".to_string(), post_id)]),
                Vec::new(),
            ),
        )
    }

    /// Reads a single post with expansions.
    pub fn read_post(&self, post_id: &str) -> Call<ApiResponse<Post>> {
        let post_id = resolve_post_id(post_id);
        self.call(
            endpoints::READ_POST.method,
            template(
                endpoints::READ_POST.path,
                HashMap::from([("id".to_string(), post_id)]),
                vec![
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,conversation_id,entities,attachments"
                            .to_string(),
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
            ),
        )
    }

    /// Searches recent posts.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use xdk::api::Client;
    ///
    /// # async fn run() -> xdk::Result<()> {
    /// let client = Client::builder().bearer("app-only-token").build()?;
    /// let resp = client.search_posts("rustlang", 25).send().await?;
    /// for post in &resp.data {
    ///     println!("{}: {}", post.id, post.text);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn search_posts(&self, query: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        let max_results = max_results.clamp(10, 100);
        self.call(
            endpoints::SEARCH_POSTS.method,
            template(
                endpoints::SEARCH_POSTS.path,
                HashMap::new(),
                vec![
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
                ],
            ),
        )
        .paginated()
    }

    /// Fetches the authenticated user's profile.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use xdk::api::Client;
    /// use xdk::auth::OAuth2Credential;
    ///
    /// # async fn run(credential: OAuth2Credential) -> xdk::Result<()> {
    /// let client = Client::builder().oauth2(credential).build()?;
    /// let resp = client.get_me().send().await?;
    /// println!("@{} ({})", resp.data.username, resp.data.id);
    /// # Ok(()) }
    /// ```
    pub fn get_me(&self) -> Call<ApiResponse<User>> {
        self.call(
            endpoints::GET_ME.method,
            template(
                endpoints::GET_ME.path,
                HashMap::new(),
                vec![(
                    "user.fields".to_string(),
                    "created_at,description,public_metrics,verified,profile_image_url".to_string(),
                )],
            ),
        )
    }

    /// Looks up a user by username.
    pub fn lookup_user(&self, username: &str) -> Call<ApiResponse<User>> {
        let username = resolve_username(username);
        self.call(
            endpoints::LOOKUP_USER.method,
            template(
                endpoints::LOOKUP_USER.path,
                HashMap::from([("username".to_string(), username)]),
                vec![(
                    "user.fields".to_string(),
                    "created_at,description,public_metrics,verified,profile_image_url".to_string(),
                )],
            ),
        )
    }

    /// Fetches the home timeline.
    pub fn get_timeline(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        self.call(
            endpoints::GET_TIMELINE.method,
            template(
                endpoints::GET_TIMELINE.path,
                id_param(user_id),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,conversation_id,entities".to_string(),
                    ),
                    ("expansions".to_string(), "author_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Fetches recent mentions.
    ///
    /// X accepts 5 to 100 results for this endpoint; `max_results` is brought
    /// into that range.
    pub fn get_mentions(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        let max_results = max_results.clamp(5, 100);
        self.call(
            endpoints::GET_MENTIONS.method,
            template(
                endpoints::GET_MENTIONS.path,
                id_param(user_id),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,conversation_id,entities".to_string(),
                    ),
                    ("expansions".to_string(), "author_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Likes a post.
    pub fn like_post(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<LikedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call_with_body(
            endpoints::LIKE_POST.method,
            template(endpoints::LIKE_POST.path, id_param(user_id), Vec::new()),
            post_id_body(&post_id),
        )
    }

    /// Unlikes a post.
    pub fn unlike_post(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<LikedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call(
            endpoints::UNLIKE_POST.method,
            template(
                endpoints::UNLIKE_POST.path,
                id_and(user_id, "tweet_id", post_id),
                Vec::new(),
            ),
        )
    }

    /// Reposts a post.
    pub fn repost(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<RepostedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call_with_body(
            endpoints::REPOST.method,
            template(endpoints::REPOST.path, id_param(user_id), Vec::new()),
            post_id_body(&post_id),
        )
    }

    /// Removes a repost.
    pub fn unrepost(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<RepostedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call(
            endpoints::UNREPOST.method,
            template(
                endpoints::UNREPOST.path,
                id_and(user_id, "source_tweet_id", post_id),
                Vec::new(),
            ),
        )
    }

    /// Bookmarks a post.
    pub fn bookmark(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<BookmarkedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call_with_body(
            endpoints::BOOKMARK.method,
            template(endpoints::BOOKMARK.path, id_param(user_id), Vec::new()),
            post_id_body(&post_id),
        )
    }

    /// Removes a bookmark.
    pub fn unbookmark(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<BookmarkedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call(
            endpoints::UNBOOKMARK.method,
            template(
                endpoints::UNBOOKMARK.path,
                id_and(user_id, "tweet_id", post_id),
                Vec::new(),
            ),
        )
    }

    /// Fetches bookmarks.
    pub fn get_bookmarks(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        self.call(
            endpoints::GET_BOOKMARKS.method,
            template(
                endpoints::GET_BOOKMARKS.path,
                id_param(user_id),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,entities".to_string(),
                    ),
                    ("expansions".to_string(), "author_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Follows a user.
    pub fn follow_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<FollowingResult>> {
        self.call_with_body(
            endpoints::FOLLOW_USER.method,
            template(
                endpoints::FOLLOW_USER.path,
                id_param(source_user_id),
                Vec::new(),
            ),
            target_user_body(target_user_id),
        )
    }

    /// Unfollows a user.
    pub fn unfollow_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<FollowingResult>> {
        self.call(
            endpoints::UNFOLLOW_USER.method,
            template(
                endpoints::UNFOLLOW_USER.path,
                source_and_target(source_user_id, target_user_id),
                Vec::new(),
            ),
        )
    }

    /// Fetches users that a given user follows.
    pub fn get_following(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        self.get_user_list(endpoints::GET_FOLLOWING.path, user_id, max_results)
    }

    /// Fetches followers of a given user.
    pub fn get_followers(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        self.get_user_list(endpoints::GET_FOLLOWERS.path, user_id, max_results)
    }

    /// Sends a direct message; the response names the conversation and the
    /// event the message created.
    pub fn send_dm(&self, participant_id: &str, text: &str) -> Call<ApiResponse<DmSentResult>> {
        let body = serde_json::json!({"text": text});
        self.post_json(
            template(
                endpoints::SEND_DM.path,
                HashMap::from([("participant_id".to_string(), participant_id.to_string())]),
                Vec::new(),
            ),
            &body,
        )
    }

    /// Fetches recent DM events.
    pub fn get_dm_events(&self, max_results: i32) -> Call<ApiResponse<Vec<DmEvent>>> {
        self.call(
            endpoints::GET_DM_EVENTS.method,
            template(
                endpoints::GET_DM_EVENTS.path,
                HashMap::new(),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "dm_event.fields".to_string(),
                        "created_at,dm_conversation_id,text".to_string(),
                    ),
                    ("expansions".to_string(), "sender_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Fetches posts liked by a user.
    ///
    /// X accepts 5 to 100 results for this endpoint; `max_results` is brought
    /// into that range.
    pub fn get_liked_posts(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        let max_results = max_results.clamp(5, 100);
        self.call(
            endpoints::GET_LIKED_POSTS.method,
            template(
                endpoints::GET_LIKED_POSTS.path,
                id_param(user_id),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,entities".to_string(),
                    ),
                    ("expansions".to_string(), "author_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Mutes a user.
    pub fn mute_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<MutingResult>> {
        self.call_with_body(
            endpoints::MUTE_USER.method,
            template(
                endpoints::MUTE_USER.path,
                id_param(source_user_id),
                Vec::new(),
            ),
            target_user_body(target_user_id),
        )
    }

    /// Fetches API usage data (post caps, daily breakdowns).
    pub fn get_usage(&self) -> Call<ApiResponse<UsageData>> {
        self.call(
            endpoints::GET_USAGE.method,
            template(
                endpoints::GET_USAGE.path,
                HashMap::new(),
                vec![(
                    "usage.fields".to_string(),
                    "daily_project_usage,daily_client_app_usage".to_string(),
                )],
            ),
        )
    }

    /// Fetches credits-based usage for the project.
    pub fn get_usage_credits(&self) -> Call<ApiResponse<UsageCreditsData>> {
        self.call(
            endpoints::GET_USAGE_CREDITS.method,
            template(endpoints::GET_USAGE_CREDITS.path, HashMap::new(), vec![]),
        )
    }

    /// Unmutes a user.
    pub fn unmute_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<MutingResult>> {
        self.call(
            endpoints::UNMUTE_USER.method,
            template(
                endpoints::UNMUTE_USER.path,
                source_and_target(source_user_id, target_user_id),
                Vec::new(),
            ),
        )
    }

    /// Lists the users the authenticated user has muted.
    pub fn get_muted(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        self.get_user_list(endpoints::GET_MUTED.path, user_id, max_results)
    }

    /// Blocks a user.
    pub fn block_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<BlockingResult>> {
        self.call_with_body(
            endpoints::BLOCK_USER.method,
            template(
                endpoints::BLOCK_USER.path,
                id_param(source_user_id),
                Vec::new(),
            ),
            target_user_body(target_user_id),
        )
    }

    /// Unblocks a user.
    pub fn unblock_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<BlockingResult>> {
        self.call(
            endpoints::UNBLOCK_USER.method,
            template(
                endpoints::UNBLOCK_USER.path,
                source_and_target(source_user_id, target_user_id),
                Vec::new(),
            ),
        )
    }

    /// Lists the users the authenticated user has blocked.
    pub fn get_blocked(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        self.get_user_list(endpoints::GET_BLOCKED.path, user_id, max_results)
    }

    // ── Media metadata ───────────────────────────────────────────────

    /// Sets the alt text X shows for `media_id`, an image or video the
    /// authenticated user uploaded.
    pub fn set_media_alt_text(
        &self,
        media_id: &str,
        text: &str,
    ) -> Call<ApiResponse<MediaMetadataResult>> {
        self.json_call(
            endpoints::CREATE_MEDIA_METADATA.method,
            template(
                endpoints::CREATE_MEDIA_METADATA.path,
                HashMap::new(),
                Vec::new(),
            ),
            &MediaMetadataBody {
                id: media_id.trim().to_string(),
                metadata: MediaMetadata {
                    alt_text: AltText {
                        text: text.to_string(),
                    },
                },
            },
        )
    }

    /// Adds the subtitle track uploaded as `subtitles_id` (media category
    /// `subtitles`) to the video `video_id`, under the two-letter
    /// `language_code` and an optional `display_name`.
    pub fn add_media_subtitles(
        &self,
        video_id: &str,
        category: VideoCategory,
        subtitles_id: &str,
        language_code: &str,
        display_name: Option<&str>,
    ) -> Call<ApiResponse<MediaSubtitlesResult>> {
        self.json_call(
            endpoints::CREATE_MEDIA_SUBTITLES.method,
            template(
                endpoints::CREATE_MEDIA_SUBTITLES.path,
                HashMap::new(),
                Vec::new(),
            ),
            &AddSubtitlesBody {
                id: video_id.trim().to_string(),
                media_category: category.wire_name(),
                subtitles: SubtitleTrack {
                    id: subtitles_id.trim().to_string(),
                    language_code: language_code.trim().to_ascii_uppercase(),
                    display_name: display_name.map(str::to_string),
                },
            },
        )
    }

    /// Removes the `language_code` subtitle track from the video
    /// `video_id`.
    pub fn remove_media_subtitles(
        &self,
        video_id: &str,
        category: VideoCategory,
        language_code: &str,
    ) -> Call<ApiResponse<DeletedResult>> {
        self.json_call(
            endpoints::DELETE_MEDIA_SUBTITLES.method,
            template(
                endpoints::DELETE_MEDIA_SUBTITLES.path,
                HashMap::new(),
                Vec::new(),
            ),
            &RemoveSubtitlesBody {
                id: video_id.trim().to_string(),
                media_category: category.wire_name(),
                language_code: language_code.trim().to_ascii_uppercase(),
            },
        )
    }

    // ── Broadcasts ───────────────────────────────────────────────────

    /// Lists the users who moderate the authenticated user's broadcast chats.
    pub fn get_chat_moderators(&self) -> Call<ApiResponse<Vec<User>>> {
        self.call(
            endpoints::GET_CHAT_MODERATORS.method,
            template(
                endpoints::GET_CHAT_MODERATORS.path,
                HashMap::new(),
                vec![(
                    "user.fields".to_string(),
                    "created_at,description,public_metrics,verified".to_string(),
                )],
            ),
        )
    }

    /// Makes `user_id` a moderator of the authenticated user's broadcast
    /// chats; the response carries the whole moderator set.
    pub fn add_chat_moderator(&self, user_id: &str) -> Call<ApiResponse<ChatModeratorsResult>> {
        self.post_json(
            template(
                endpoints::ADD_CHAT_MODERATOR.path,
                HashMap::new(),
                Vec::new(),
            ),
            &ChatModeratorBody {
                user_id: user_id.to_string(),
            },
        )
    }

    /// Removes `user_id` from the moderators of the authenticated user's
    /// broadcast chats; the response carries the remaining set.
    pub fn remove_chat_moderator(&self, user_id: &str) -> Call<ApiResponse<ChatModeratorsResult>> {
        self.call(
            endpoints::REMOVE_CHAT_MODERATOR.method,
            template(
                endpoints::REMOVE_CHAT_MODERATOR.path,
                HashMap::from([("user_id".to_string(), user_id.to_string())]),
                Vec::new(),
            ),
        )
    }

    // ── Webhooks ────────────────────────────────────────────────────

    /// Lists the webhooks registered for the app.
    pub fn get_webhooks(&self) -> Call<ApiResponse<Vec<Webhook>>> {
        self.call(
            endpoints::GET_WEBHOOKS.method,
            template(endpoints::GET_WEBHOOKS.path, HashMap::new(), Vec::new()),
        )
    }

    /// Registers `url` as a webhook. X sends its CRC check to the URL before
    /// it answers, so the request succeeds only while a receiver is reachable
    /// there.
    pub fn create_webhook(&self, url: &str) -> Call<ApiResponse<Webhook>> {
        self.post_json(
            template(endpoints::CREATE_WEBHOOK.path, HashMap::new(), Vec::new()),
            &WebhookBody {
                url: url.to_string(),
            },
        )
    }

    /// Asks X to send its CRC check to a webhook again; the response says
    /// whether the URL answered it.
    pub fn validate_webhook(&self, webhook_id: &str) -> Call<ApiResponse<WebhookValidation>> {
        self.call(
            endpoints::VALIDATE_WEBHOOK.method,
            template(
                endpoints::VALIDATE_WEBHOOK.path,
                HashMap::from([("webhook_id".to_string(), webhook_id.to_string())]),
                Vec::new(),
            ),
        )
    }

    /// Deletes a webhook. Its subscriptions stop delivering.
    pub fn delete_webhook(&self, webhook_id: &str) -> Call<ApiResponse<DeletedResult>> {
        self.call(
            endpoints::DELETE_WEBHOOK.method,
            template(
                endpoints::DELETE_WEBHOOK.path,
                HashMap::from([("webhook_id".to_string(), webhook_id.to_string())]),
                Vec::new(),
            ),
        )
    }

    /// Starts a job that delivers the events between `from_date` and
    /// `to_date` to a webhook again. Both are twelve digits, `yyyymmddhhmm`
    /// in UTC, which is the form the endpoint takes.
    pub fn create_webhook_replay(
        &self,
        webhook_id: &str,
        from_date: &str,
        to_date: &str,
    ) -> Call<ApiResponse<WebhookReplayJob>> {
        self.post_json(
            template(
                endpoints::CREATE_WEBHOOK_REPLAY.path,
                HashMap::new(),
                Vec::new(),
            ),
            &WebhookReplayBody {
                webhook_id: webhook_id.to_string(),
                from_date: from_date.to_string(),
                to_date: to_date.to_string(),
            },
        )
    }

    /// Shared GET for the user-list endpoints keyed on a single `{id}`.
    fn get_user_list(
        &self,
        path: &str,
        user_id: &str,
        max_results: i32,
    ) -> Call<ApiResponse<Vec<User>>> {
        self.call(
            "GET",
            template(
                path,
                id_param(user_id),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "user.fields".to_string(),
                        "created_at,description,public_metrics,verified".to_string(),
                    ),
                ],
            ),
        )
        .paginated()
    }
}
