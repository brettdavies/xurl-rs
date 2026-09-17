//! An in-process X API for tests that must not spend credits.
//!
//! [`MockX::start`] binds a local server whose routes answer the crate's
//! shortcuts with the fixture responses the response types are validated
//! against, so a `read_post` or `search_posts` against it deserializes into
//! the same `Post` a live call would. Every response carries a rate-limit
//! window, so [`Client::last_rate_limit`] reads something too.
//!
//! Enable the feature in a test profile only, so a release build pulls none
//! of the mock's dependencies:
//!
//! ```toml
//! [dev-dependencies]
//! xdk-rs = { version = "0.1", features = ["testing"] }
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

use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::api::{Client, ClientBuilder};
use crate::auth::OAuth2Credential;

/// The bearer token [`MockX::app_client`] sends; the mock accepts anything.
pub const MOCK_BEARER_TOKEN: &str = "mock-bearer-token";

const FIXTURES: &str = include_str!("../../tests/fixtures/openapi/example_responses.json");

/// The `x-rate-limit-*` window every seeded response reports.
const RATE_LIMIT: u32 = 450;
const RATE_LIMIT_REMAINING: u32 = 449;
const RATE_LIMIT_WINDOW_SECS: u64 = 900;

/// A seeded route: the shortcut's method and path, and which fixture answers.
struct Route {
    method: &'static str,
    path: &'static str,
    fixture: &'static str,
    status: u16,
}

/// One entry per shortcut whose response the fixture file carries; the
/// patterns are the shortcuts' own path templates with their `{id}` segments
/// widened.
const ROUTES: &[Route] = &[
    Route {
        method: "GET",
        path: r"^/2/tweets/[0-9]+$",
        fixture: "post_single",
        status: 200,
    },
    Route {
        method: "GET",
        path: r"^/2/tweets/search/recent$",
        fixture: "post_list",
        status: 200,
    },
    Route {
        method: "POST",
        path: r"^/2/tweets$",
        fixture: "post_single",
        status: 201,
    },
    Route {
        method: "DELETE",
        path: r"^/2/tweets/[0-9]+$",
        fixture: "action_deleted",
        status: 200,
    },
    Route {
        method: "GET",
        path: r"^/2/users/me$",
        fixture: "user_single",
        status: 200,
    },
    Route {
        method: "GET",
        path: r"^/2/users/by/username/[^/]+$",
        fixture: "user_single",
        status: 200,
    },
    Route {
        method: "GET",
        path: r"^/2/users/[0-9]+/timelines/reverse_chronological$",
        fixture: "post_list",
        status: 200,
    },
    Route {
        method: "GET",
        path: r"^/2/users/[0-9]+/mentions$",
        fixture: "post_list",
        status: 200,
    },
    Route {
        method: "GET",
        path: r"^/2/users/[0-9]+/bookmarks$",
        fixture: "post_list",
        status: 200,
    },
    Route {
        method: "GET",
        path: r"^/2/users/[0-9]+/liked_tweets$",
        fixture: "post_list",
        status: 200,
    },
    Route {
        method: "POST",
        path: r"^/2/users/[0-9]+/likes$",
        fixture: "action_liked",
        status: 200,
    },
    Route {
        method: "POST",
        path: r"^/2/users/[0-9]+/retweets$",
        fixture: "action_retweeted",
        status: 200,
    },
    Route {
        method: "POST",
        path: r"^/2/users/[0-9]+/bookmarks$",
        fixture: "action_bookmarked",
        status: 200,
    },
    Route {
        method: "POST",
        path: r"^/2/users/[0-9]+/following$",
        fixture: "action_following",
        status: 200,
    },
    Route {
        method: "POST",
        path: r"^/2/users/[0-9]+/blocking$",
        fixture: "action_blocking",
        status: 200,
    },
    Route {
        method: "POST",
        path: r"^/2/users/[0-9]+/muting$",
        fixture: "action_muting",
        status: 200,
    },
    Route {
        method: "POST",
        path: r"^/2/dm_conversations/with/[0-9]+/messages$",
        fixture: "dm_event",
        status: 201,
    },
    Route {
        method: "GET",
        path: r"^/2/dm_events$",
        fixture: "dm_event_list",
        status: 200,
    },
    Route {
        method: "GET",
        path: r"^/2/usage/tweets$",
        fixture: "usage",
        status: 200,
    },
];

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
            let body = fixtures()[route.fixture].clone();
            Mock::given(method(route.method))
                .and(path_regex(route.path))
                .respond_with(
                    ResponseTemplate::new(route.status)
                        .set_body_json(body)
                        .insert_header("x-rate-limit-limit", RATE_LIMIT.to_string().as_str())
                        .insert_header(
                            "x-rate-limit-remaining",
                            RATE_LIMIT_REMAINING.to_string().as_str(),
                        )
                        .insert_header("x-rate-limit-reset", reset_at.to_string().as_str()),
                )
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
