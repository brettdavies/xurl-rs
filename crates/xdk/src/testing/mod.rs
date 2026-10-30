//! An in-process X API for tests that must not spend credits, built only
//! with the `testing` feature.
//!
//! [`MockX::start`] binds a local server that answers every endpoint the
//! shortcut layer declares with the fixture responses the response types are
//! validated against, so a `read_post` or `search_posts` against it
//! deserializes into the same `Post` a live call would. Every response
//! carries a rate-limit window, so [`Client::last_rate_limit`] reads
//! something too. [`deliver_signed_sample`] posts a signed sample event to a
//! [`webhooks::Receiver`](crate::webhooks::Receiver) under test.
//!
//! Enable the feature in a test profile only, so a release build pulls none
//! of the mock's dependencies:
//!
//! ```sh
//! cargo add xdk-rs --dev --features testing
//! ```
//!
//! ```rust,no_run
//! use xdk::testing::MockX;
//!
//! #[tokio::main]
//! async fn main() -> xdk::Result<()> {
//!     let mock = MockX::start().await;
//!     let client = mock.app_client()?;
//!
//!     let posts = client.search_posts("rust", 10).send().await?;
//!     assert_eq!(posts.data.len(), 2);
//!
//!     let seen = mock.requests().await;
//!     assert_eq!(seen[0].path, "/2/tweets/search/recent");
//!     Ok(())
//! }
//! ```
//!
//! A route the fixtures do not cover, or a failure to rehearse, is stubbed
//! with [`MockX::stub`]; a stub outranks the seeded route for the same path.
//! The fixture file also carries the two failure shapes the API documents,
//! `api_error` (`code` and `message`) and `api_problem` (a problem+json
//! body), so [`MockX::fixture`] hands a real failure body to a stub.

use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::api::auth_matrix::{Endpoint, endpoints};
use crate::api::{Client, ClientBuilder};
use crate::auth::OAuth2Credential;

mod webhooks;

pub use webhooks::deliver_signed_sample;

/// The bearer token [`MockX::app_client`] sends; the mock accepts anything.
pub const MOCK_BEARER_TOKEN: &str = "mock-bearer-token";

const FIXTURES: &str = include_str!("../../tests/fixtures/openapi/example_responses.json");

/// The `x-rate-limit-*` window every seeded response reports.
const RATE_LIMIT: u32 = 450;
const RATE_LIMIT_REMAINING: u32 = 449;
const RATE_LIMIT_WINDOW_SECS: u64 = 900;

/// A seeded route: the declared endpoint, the fixture that answers it, and
/// the status it answers with. The method and the path pattern come from the
/// declaration, so no route spells a path.
struct Route {
    endpoint: Endpoint,
    fixture: &'static str,
    status: u16,
}

/// One entry per declared endpoint, in declaration order.
const ROUTES: &[Route] = &[
    Route {
        endpoint: endpoints::CREATE_POST,
        fixture: "post_single",
        status: 201,
    },
    Route {
        endpoint: endpoints::READ_POST,
        fixture: "post_single",
        status: 200,
    },
    Route {
        endpoint: endpoints::DELETE_POST,
        fixture: "action_deleted",
        status: 200,
    },
    Route {
        endpoint: endpoints::SEARCH_POSTS,
        fixture: "post_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_ME,
        fixture: "user_single",
        status: 200,
    },
    Route {
        endpoint: endpoints::LOOKUP_USER,
        fixture: "user_single",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_TIMELINE,
        fixture: "post_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_MENTIONS,
        fixture: "post_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_FOLLOWERS,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_MY_FOLLOWERS,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_LIKED_POSTS,
        fixture: "post_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_LIST,
        fixture: "list_created",
        status: 201,
    },
    Route {
        endpoint: endpoints::GET_LIST,
        fixture: "list_single",
        status: 200,
    },
    Route {
        endpoint: endpoints::UPDATE_LIST,
        fixture: "list_updated",
        status: 200,
    },
    Route {
        endpoint: endpoints::DELETE_LIST,
        fixture: "action_deleted",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_LIST_POSTS,
        fixture: "post_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_LIST_MEMBERS,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::ADD_LIST_MEMBER,
        fixture: "list_member",
        status: 200,
    },
    Route {
        endpoint: endpoints::REMOVE_LIST_MEMBER,
        fixture: "list_member",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_LIST_FOLLOWERS,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_OWNED_LISTS,
        fixture: "lists",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_FOLLOWED_LISTS,
        fixture: "lists",
        status: 200,
    },
    Route {
        endpoint: endpoints::FOLLOW_LIST,
        fixture: "list_followed",
        status: 200,
    },
    Route {
        endpoint: endpoints::UNFOLLOW_LIST,
        fixture: "list_followed",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_LIST_MEMBERSHIPS,
        fixture: "lists",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_PINNED_LISTS,
        fixture: "lists",
        status: 200,
    },
    Route {
        endpoint: endpoints::PIN_LIST,
        fixture: "list_pinned",
        status: 200,
    },
    Route {
        endpoint: endpoints::UNPIN_LIST,
        fixture: "list_pinned",
        status: 200,
    },
    Route {
        endpoint: endpoints::SEARCH_COMMUNITIES,
        fixture: "community_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_COMMUNITY,
        fixture: "community",
        status: 200,
    },
    Route {
        endpoint: endpoints::SEARCH_NEWS,
        fixture: "news_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_NEWS,
        fixture: "news_story",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_TRENDS,
        fixture: "trends",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_PERSONALIZED_TRENDS,
        fixture: "personalized_trends",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_POST_ANALYTICS,
        fixture: "post_analytics",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_POST_COUNTS,
        fixture: "post_counts",
        status: 200,
    },
    Route {
        endpoint: endpoints::LOOKUP_USERS,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::LOOKUP_USERS_BY_USERNAME,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::SEARCH_USERS,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_USER_POSTS,
        fixture: "post_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::READ_POSTS,
        fixture: "post_lookup",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_LIKING_USERS,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_REPOSTED_BY,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_QUOTE_POSTS,
        fixture: "post_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::HIDE_REPLY,
        fixture: "action_hidden",
        status: 200,
    },
    Route {
        endpoint: endpoints::LIKE_POST,
        fixture: "action_liked",
        status: 200,
    },
    Route {
        endpoint: endpoints::UNLIKE_POST,
        fixture: "action_liked",
        status: 200,
    },
    Route {
        endpoint: endpoints::REPOST,
        fixture: "action_retweeted",
        status: 200,
    },
    Route {
        endpoint: endpoints::UNREPOST,
        fixture: "action_retweeted",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_BOOKMARKS,
        fixture: "post_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::BOOKMARK,
        fixture: "action_bookmarked",
        status: 200,
    },
    Route {
        endpoint: endpoints::UNBOOKMARK,
        fixture: "action_bookmarked",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_BOOKMARK_FOLDERS,
        fixture: "bookmark_folders",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_BOOKMARK_FOLDER,
        fixture: "bookmark_folder",
        status: 201,
    },
    Route {
        endpoint: endpoints::GET_BOOKMARK_FOLDER_POSTS,
        fixture: "bookmark_folder_posts",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_FOLLOWING,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_MY_FOLLOWING,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::FOLLOW_USER,
        fixture: "action_following",
        status: 200,
    },
    Route {
        endpoint: endpoints::UNFOLLOW_USER,
        fixture: "action_following",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_MUTED,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::MUTE_USER,
        fixture: "action_muting",
        status: 200,
    },
    Route {
        endpoint: endpoints::UNMUTE_USER,
        fixture: "action_muting",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_BLOCKED,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::BLOCK_USER,
        fixture: "action_blocking",
        status: 200,
    },
    Route {
        endpoint: endpoints::UNBLOCK_USER,
        fixture: "action_blocking",
        status: 200,
    },
    Route {
        endpoint: endpoints::SEND_DM,
        fixture: "dm_sent",
        status: 201,
    },
    Route {
        endpoint: endpoints::CREATE_DM_CONVERSATION,
        fixture: "dm_sent",
        status: 201,
    },
    Route {
        endpoint: endpoints::SEND_TO_CONVERSATION,
        fixture: "dm_sent",
        status: 201,
    },
    Route {
        endpoint: endpoints::GET_CONVERSATION_EVENTS,
        fixture: "dm_event_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_DM_EVENTS_WITH,
        fixture: "dm_event_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_DM_EVENT,
        fixture: "dm_event",
        status: 200,
    },
    Route {
        endpoint: endpoints::DELETE_DM_EVENT,
        fixture: "action_deleted",
        status: 200,
    },
    Route {
        endpoint: endpoints::BLOCK_DMS,
        fixture: "dm_blocked",
        status: 200,
    },
    Route {
        endpoint: endpoints::UNBLOCK_DMS,
        fixture: "dm_blocked",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_DM_EVENTS,
        fixture: "dm_event_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_USAGE,
        fixture: "usage",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_USAGE_CREDITS,
        fixture: "usage_credits",
        status: 200,
    },
    Route {
        endpoint: endpoints::MEDIA_UPLOAD,
        fixture: "media_upload_init",
        status: 200,
    },
    Route {
        endpoint: endpoints::MEDIA_UPLOAD_STATUS,
        fixture: "media_upload_status",
        status: 200,
    },
    Route {
        endpoint: endpoints::MEDIA_UPLOAD_INITIALIZE,
        fixture: "media_upload_init",
        status: 200,
    },
    Route {
        endpoint: endpoints::MEDIA_UPLOAD_APPEND,
        fixture: "media_upload_append",
        status: 200,
    },
    Route {
        endpoint: endpoints::MEDIA_UPLOAD_FINALIZE,
        fixture: "media_upload_status",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_MEDIA_METADATA,
        fixture: "media_metadata",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_MEDIA_SUBTITLES,
        fixture: "media_subtitles",
        status: 200,
    },
    Route {
        endpoint: endpoints::DELETE_MEDIA_SUBTITLES,
        fixture: "action_deleted",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_CHAT_MODERATORS,
        fixture: "user_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::ADD_CHAT_MODERATOR,
        fixture: "chat_moderators",
        status: 200,
    },
    Route {
        endpoint: endpoints::REMOVE_CHAT_MODERATOR,
        fixture: "chat_moderators",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_CHAT_MUTES,
        fixture: "chat_mutes",
        status: 200,
    },
    Route {
        endpoint: endpoints::ADD_CHAT_MUTE,
        fixture: "broadcast_chat_muted",
        status: 200,
    },
    Route {
        endpoint: endpoints::REMOVE_CHAT_MUTE,
        fixture: "broadcast_chat_muted",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_MEDIA,
        fixture: "media_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_MEDIA_ANALYTICS,
        fixture: "media_analytics",
        status: 200,
    },
    Route {
        endpoint: endpoints::SEARCH_SPACES,
        fixture: "space_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_SPACES,
        fixture: "space_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_SPACES_BY_CREATOR,
        fixture: "space_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_SPACE_POSTS,
        fixture: "post_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_STREAM_RULES,
        fixture: "stream_rules",
        status: 200,
    },
    Route {
        endpoint: endpoints::UPDATE_STREAM_RULES,
        fixture: "stream_rules_added",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_STREAM_RULE_COUNTS,
        fixture: "stream_rule_counts",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_ARTICLE_DRAFT,
        fixture: "article_draft",
        status: 201,
    },
    Route {
        endpoint: endpoints::PUBLISH_ARTICLE,
        fixture: "article_published",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_NOTE,
        fixture: "note_created",
        status: 201,
    },
    Route {
        endpoint: endpoints::EVALUATE_NOTE,
        fixture: "note_evaluation",
        status: 200,
    },
    Route {
        endpoint: endpoints::DELETE_NOTE,
        fixture: "action_deleted",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_NOTES_WRITTEN,
        fixture: "notes_written",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_POSTS_ELIGIBLE_FOR_NOTES,
        fixture: "post_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::LIST_BROADCASTS,
        fixture: "broadcasts",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_BROADCAST,
        fixture: "broadcast",
        status: 200,
    },
    Route {
        endpoint: endpoints::LIST_SCHEDULED_BROADCASTS,
        fixture: "scheduled_broadcasts",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_SCHEDULED_BROADCAST,
        fixture: "scheduled_broadcast",
        status: 201,
    },
    Route {
        endpoint: endpoints::GET_SCHEDULED_BROADCAST,
        fixture: "scheduled_broadcast",
        status: 200,
    },
    Route {
        endpoint: endpoints::UPDATE_SCHEDULED_BROADCAST,
        fixture: "scheduled_broadcast",
        status: 200,
    },
    Route {
        endpoint: endpoints::DELETE_SCHEDULED_BROADCAST,
        fixture: "action_deleted",
        status: 200,
    },
    Route {
        endpoint: endpoints::GO_LIVE,
        fixture: "scheduled_broadcast",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_BROADCAST_CHAT,
        fixture: "broadcast_chat",
        status: 200,
    },
    Route {
        endpoint: endpoints::SEND_BROADCAST_CHAT,
        fixture: "broadcast_chat_sent",
        status: 200,
    },
    Route {
        endpoint: endpoints::MUTE_CHAT_USER,
        fixture: "broadcast_chat_muted",
        status: 200,
    },
    Route {
        endpoint: endpoints::UNMUTE_CHAT_USER,
        fixture: "broadcast_chat_muted",
        status: 200,
    },
    Route {
        endpoint: endpoints::DELETE_CHAT_MESSAGE,
        fixture: "action_deleted",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_STREAM_SOURCES,
        fixture: "stream_sources",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_STREAM_SOURCE,
        fixture: "stream_source",
        status: 201,
    },
    Route {
        endpoint: endpoints::GET_STREAM_SOURCE,
        fixture: "stream_source",
        status: 200,
    },
    Route {
        endpoint: endpoints::RENAME_STREAM_SOURCE,
        fixture: "stream_source",
        status: 200,
    },
    Route {
        endpoint: endpoints::DELETE_STREAM_SOURCE,
        fixture: "stream_source_deleted",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_CALL,
        fixture: "call",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_SCHEDULED_CALLS,
        fixture: "scheduled_calls",
        status: 200,
    },
    Route {
        endpoint: endpoints::SCHEDULE_CALL,
        fixture: "scheduled_call",
        status: 201,
    },
    Route {
        endpoint: endpoints::GET_SCHEDULED_CALL,
        fixture: "scheduled_call",
        status: 200,
    },
    Route {
        endpoint: endpoints::UPDATE_SCHEDULED_CALL,
        fixture: "scheduled_call_updated",
        status: 200,
    },
    Route {
        endpoint: endpoints::CANCEL_SCHEDULED_CALL,
        fixture: "scheduled_call_cancelled",
        status: 200,
    },
    Route {
        endpoint: endpoints::RESPOND_TO_SCHEDULED_CALL,
        fixture: "scheduled_call_rsvp",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_WEBHOOKS,
        fixture: "webhook_list",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_WEBHOOK,
        fixture: "webhook_single",
        status: 200,
    },
    Route {
        endpoint: endpoints::VALIDATE_WEBHOOK,
        fixture: "webhook_validated",
        status: 200,
    },
    Route {
        endpoint: endpoints::DELETE_WEBHOOK,
        fixture: "action_deleted",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_WEBHOOK_REPLAY,
        fixture: "webhook_replay_job",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_ACCOUNT_ACTIVITY_SUBSCRIPTION_COUNT,
        fixture: "account_activity_count",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_ACCOUNT_ACTIVITY_SUBSCRIPTIONS,
        fixture: "account_activity_subscriptions",
        status: 200,
    },
    Route {
        endpoint: endpoints::CHECK_ACCOUNT_ACTIVITY_SUBSCRIPTION,
        fixture: "account_activity_subscribed",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_ACCOUNT_ACTIVITY_SUBSCRIPTION,
        fixture: "account_activity_subscribed",
        status: 200,
    },
    Route {
        endpoint: endpoints::DELETE_ACCOUNT_ACTIVITY_SUBSCRIPTION,
        fixture: "account_activity_unsubscribed",
        status: 200,
    },
    Route {
        endpoint: endpoints::GET_WEBHOOK_STREAM_LINKS,
        fixture: "webhook_stream_links",
        status: 200,
    },
    Route {
        endpoint: endpoints::CREATE_WEBHOOK_STREAM_LINK,
        fixture: "webhook_stream_link_provisioned",
        status: 200,
    },
    Route {
        endpoint: endpoints::DELETE_WEBHOOK_STREAM_LINK,
        fixture: "action_deleted",
        status: 200,
    },
];

/// The regex that matches the paths `path` renders to: a `{username}`
/// segment widens to anything but a slash, every other parameter to digits.
fn pattern(path: &str) -> String {
    let widened: Vec<String> = path
        .split('/')
        .map(|segment| match segment {
            "{username}" => "[^/]+".to_string(),
            s if s.starts_with('{') && s.ends_with('}') => "[0-9]+".to_string(),
            s => s.to_string(),
        })
        .collect();
    format!("^{}$", widened.join("/"))
}

fn fixtures() -> &'static Value {
    static PARSED: OnceLock<Value> = OnceLock::new();
    PARSED.get_or_init(|| {
        serde_json::from_str(FIXTURES).expect("the bundled fixture file is valid JSON")
    })
}

/// A request the mock received, as a test asserts on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Received {
    /// The HTTP method.
    pub method: String,
    /// The path, without the query string.
    pub path: String,
    /// The query string, without the leading `?`.
    pub query: Option<String>,
    /// Every header, in arrival order; the `authorization` header is among them.
    pub headers: Vec<(String, String)>,
    /// The raw body.
    pub body: Vec<u8>,
}

/// A local X API that answers from the crate's fixtures.
///
/// Dropping it stops the server.
pub struct MockX {
    server: MockServer,
}

impl std::fmt::Debug for MockX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockX")
            .field("base_url", &self.server.uri())
            .finish()
    }
}

impl MockX {
    /// Binds a server on a free local port and seeds every fixture route.
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        let reset_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() + RATE_LIMIT_WINDOW_SECS)
            .unwrap_or(RATE_LIMIT_WINDOW_SECS);
        for route in ROUTES {
            let template = ResponseTemplate::new(route.status)
                .set_body_json(fixtures()[route.fixture].clone())
                .insert_header("x-rate-limit-limit", RATE_LIMIT.to_string().as_str())
                .insert_header(
                    "x-rate-limit-remaining",
                    RATE_LIMIT_REMAINING.to_string().as_str(),
                )
                .insert_header("x-rate-limit-reset", reset_at.to_string().as_str());
            Mock::given(method(route.endpoint.method))
                .and(path_regex(pattern(route.endpoint.path)))
                .respond_with(template)
                .mount(&server)
                .await;
        }
        Self { server }
    }

    /// The origin to point a client at, `http://127.0.0.1:<port>`.
    #[must_use]
    pub fn base_url(&self) -> String {
        self.server.uri()
    }

    /// A client builder already pointed at the mock; add the credential.
    pub fn builder(&self) -> ClientBuilder {
        Client::builder().base_url(self.base_url())
    }

    /// A bearer-token client: app-only endpoints such as search and lookups,
    /// refused by the auth matrix for user-context endpoints exactly as a live
    /// bearer would be.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Http`] when the HTTP client cannot be built.
    pub fn app_client(&self) -> crate::Result<Client> {
        self.builder().bearer(MOCK_BEARER_TOKEN).build()
    }

    /// A client holding an `OAuth2` user credential and a bearer token, so
    /// every shortcut passes the auth matrix. The credential never expires,
    /// so no refresh is attempted.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Http`] when the HTTP client cannot be built.
    pub fn user_client(&self) -> crate::Result<Client> {
        self.builder()
            .bearer(MOCK_BEARER_TOKEN)
            .oauth2(OAuth2Credential {
                client_id: "mock-client-id".to_string(),
                client_secret: "mock-client-secret".to_string(),
                access_token: "mock-access-token".to_string(),
                refresh_token: None,
                expires_at: None,
            })
            .build()
    }

    /// Answers `method` requests whose path matches the regex `path_pattern`
    /// with `status` and the JSON `body`, ahead of any seeded route.
    pub async fn stub(&self, method_name: &str, path_pattern: &str, status: u16, body: Value) {
        Mock::given(method(method_name))
            .and(path_regex(path_pattern))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
            .with_priority(1)
            .mount(&self.server)
            .await;
    }

    /// The seeded body for a fixture name such as `post_single`, or `None`.
    #[must_use]
    pub fn fixture(name: &str) -> Option<Value> {
        fixtures().get(name).cloned()
    }

    /// Every request received so far, in arrival order.
    pub async fn requests(&self) -> Vec<Received> {
        self.server
            .received_requests()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|req| Received {
                method: req.method.to_string(),
                path: req.url.path().to_string(),
                query: req.url.query().map(str::to_string),
                headers: req
                    .headers
                    .iter()
                    .map(|(name, value)| {
                        (
                            name.to_string(),
                            String::from_utf8_lossy(value.as_bytes()).into_owned(),
                        )
                    })
                    .collect(),
                body: req.body,
            })
            .collect()
    }
}
