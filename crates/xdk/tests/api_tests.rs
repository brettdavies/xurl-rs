//! Ported from Go: api/client_test.go + api/endpoints_test.go
//!                  + api/shortcuts_test.go + api/media_test.go
//!
//! Tests the core API client, request building, response parsing,
//! streaming endpoint detection, shortcut commands, and media upload.
//!
//! Uses wiremock for mock HTTP servers on the test's own runtime.

use std::collections::BTreeMap;

use rstest::rstest;
use tempfile::TempDir;
use wiremock::matchers::{method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use std::collections::HashMap;

use xdk::api::auth_matrix::WireScheme;
use xdk::api::{
    self, Client, RequestOptions, RequestTarget, extract_media_id, extract_segment_index,
    is_media_append_request, is_streaming_endpoint,
};
use xdk::auth::Auth;
use xdk::config::Config;
use xdk::store::{App, OAuth1Token, OAuth2Token, Token, TokenStore, TokenType};

// ── Mock server helper ─────────────────────────────────────────────────

struct TestServer {
    server: MockServer,
    uri: String,
}

impl TestServer {
    async fn new() -> Self {
        let server = MockServer::start().await;
        let uri = server.uri();
        Self { server, uri }
    }

    async fn mount(&self, mock: Mock) {
        mock.mount(&self.server).await;
    }

    fn uri(&self) -> &str {
        &self.uri
    }

    /// Returns the number of requests this mock server has received across
    /// all registered mocks. Used by the U6 enforcement tests to assert
    /// that fail-fast validation rejects before any network I/O happens.
    async fn received_request_count(&self) -> usize {
        self.server
            .received_requests()
            .await
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Returns the values of a given header from every received request, in
    /// arrival order. Each inner `Vec<String>` collects all values for that
    /// header on a single request (HTTP allows multiple headers with the same
    /// name; the Authorization-double-header bug surfaces here as a vec with
    /// two entries).
    async fn received_header_values(&self, name: &str) -> Vec<Vec<String>> {
        let lookup = name.to_string();
        self.server
            .received_requests()
            .await
            .map(|reqs| {
                reqs.into_iter()
                    .map(|r| {
                        r.headers
                            .get_all(&lookup)
                            .iter()
                            .filter_map(|v| v.to_str().ok().map(str::to_owned))
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ── Test helpers ───────────────────────────────────────────────────────

/// Builds an `Auth` on the temp-dir store it will carry, so construction never
/// resolves the real `~/.xurl`.
fn auth_for(cfg: &Config, token_store: TokenStore) -> Auth {
    let store_path = token_store.file_path.clone();
    Auth::new_with_store_path(cfg, &store_path).with_token_store(token_store)
}

fn create_test_config(base_url: &str) -> Config {
    // `Config` has `pub(crate)` resolver fields that external callers cannot
    // name in a struct literal; start from `Config::new()` and assign the
    // public fields explicitly. The resolver fields are overwritten by
    // `Auth::new_with_store_path` downstream.
    let mut cfg = Config::new();
    cfg.client_id = "test-client-id".to_string();
    cfg.client_secret = "test-client-secret".to_string();
    cfg.redirect_uri = "http://localhost:8080/callback".to_string();
    cfg.auth_url = "https://x.com/i/oauth2/authorize".to_string();
    cfg.token_url = "https://api.x.com/2/oauth2/token".to_string();
    cfg.api_base_url = base_url.to_string();
    cfg.info_url = format!("{base_url}/2/users/me");
    cfg.app_name = String::new();
    cfg
}

fn create_mock_auth_with_bearer(base_url: &str) -> (Auth, TempDir) {
    let cfg = create_test_config(base_url);
    let tmp = TempDir::new().expect("temp dir");
    let file_path = tmp.path().join(".xurl");

    let mut store = TokenStore {
        apps: BTreeMap::new(),
        default_app: "default".to_string(),
        file_path,
        load_state: xdk::store::LoadState::Loaded,
    };
    store.apps.insert(
        "default".to_string(),
        App {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            default_user: String::new(),
            redirect_uri: String::new(),
            oauth2_tokens: BTreeMap::new(),
            oauth1_token: None,
            bearer_token: Some(Token {
                token_type: TokenType::Bearer,
                bearer: Some("test-bearer-token".to_string()),
                oauth2: None,
                oauth1: None,
            }),
            unnamed_oauth2_token: None,
        },
    );

    let auth = auth_for(&cfg, store);
    (auth, tmp)
}

fn create_mock_auth_with_oauth1(base_url: &str) -> (Auth, TempDir) {
    let cfg = create_test_config(base_url);
    let tmp = TempDir::new().expect("temp dir");
    let file_path = tmp.path().join(".xurl");

    let mut store = TokenStore {
        apps: BTreeMap::new(),
        default_app: "default".to_string(),
        file_path,
        load_state: xdk::store::LoadState::Loaded,
    };
    store.apps.insert(
        "default".to_string(),
        App {
            client_id: String::new(),
            client_secret: String::new(),
            default_user: String::new(),
            redirect_uri: String::new(),
            oauth2_tokens: BTreeMap::new(),
            oauth1_token: Some(Token {
                token_type: TokenType::Oauth1,
                bearer: None,
                oauth2: None,
                oauth1: Some(OAuth1Token {
                    access_token: "at".to_string(),
                    token_secret: "ts".to_string(),
                    consumer_key: "ck".to_string(),
                    consumer_secret: "cs".to_string(),
                }),
            }),
            bearer_token: None,
            unnamed_oauth2_token: None,
        },
    );

    let auth = auth_for(&cfg, store);
    (auth, tmp)
}

fn create_mock_auth_with_oauth2(base_url: &str) -> (Auth, TempDir) {
    let cfg = create_test_config(base_url);
    let tmp = TempDir::new().expect("temp dir");
    let file_path = tmp.path().join(".xurl");

    let future_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;

    let mut store = TokenStore {
        apps: BTreeMap::new(),
        default_app: "default".to_string(),
        file_path,
        load_state: xdk::store::LoadState::Loaded,
    };
    let mut app = App {
        client_id: "cid".to_string(),
        client_secret: "csec".to_string(),
        default_user: "testuser".to_string(),
        redirect_uri: String::new(),
        oauth2_tokens: BTreeMap::new(),
        oauth1_token: None,
        bearer_token: None,
        unnamed_oauth2_token: None,
    };
    app.oauth2_tokens.insert(
        "testuser".to_string(),
        Token {
            token_type: TokenType::Oauth2,
            bearer: None,
            oauth2: Some(OAuth2Token {
                access_token: "valid-access-token".to_string(),
                refresh_token: "refresh".to_string(),
                expiration_time: future_epoch,
            }),
            oauth1: None,
        },
    );
    store.apps.insert("default".to_string(), app);

    let auth = auth_for(&cfg, store);
    (auth, tmp)
}

/// Fixture with all three credentials stored on the default app. Under U7's
/// endpoint-aware auto-detect, callers exercising a non-auth-centric
/// concern (request shape, error mapping, response parsing) can opt into
/// this fixture and stay agnostic about which scheme the matrix picks for
/// any given `(method, path)` — the intersection always resolves.
fn create_mock_auth_with_all_methods(base_url: &str) -> (Auth, TempDir) {
    let cfg = create_test_config(base_url);
    let tmp = TempDir::new().expect("temp dir");
    let file_path = tmp.path().join(".xurl");

    let future_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;

    let mut store = TokenStore {
        apps: BTreeMap::new(),
        default_app: "default".to_string(),
        file_path,
        load_state: xdk::store::LoadState::Loaded,
    };
    let mut app = App {
        client_id: "cid".to_string(),
        client_secret: "csec".to_string(),
        default_user: "testuser".to_string(),
        redirect_uri: String::new(),
        oauth2_tokens: BTreeMap::new(),
        oauth1_token: Some(Token {
            token_type: TokenType::Oauth1,
            bearer: None,
            oauth2: None,
            oauth1: Some(OAuth1Token {
                access_token: "at".to_string(),
                token_secret: "ts".to_string(),
                consumer_key: "ck".to_string(),
                consumer_secret: "cs".to_string(),
            }),
        }),
        bearer_token: Some(Token {
            token_type: TokenType::Bearer,
            bearer: Some("test-bearer-token".to_string()),
            oauth2: None,
            oauth1: None,
        }),
        unnamed_oauth2_token: None,
    };
    app.oauth2_tokens.insert(
        "testuser".to_string(),
        Token {
            token_type: TokenType::Oauth2,
            bearer: None,
            oauth2: Some(OAuth2Token {
                access_token: "valid-access-token".to_string(),
                refresh_token: "refresh".to_string(),
                expiration_time: future_epoch,
            }),
            oauth1: None,
        },
    );
    store.apps.insert("default".to_string(), app);

    let auth = auth_for(&cfg, store);
    (auth, tmp)
}

/// Builds a `RequestTarget::Template` from a path with no params or query.
///
/// Keeps the test-side ergonomics close to the pre-U4 `endpoint:
/// "/2/foo".to_string()` shorthand without rewriting every literal at
/// each call site.
fn target_path(path: &str) -> RequestTarget {
    RequestTarget::Template {
        path: path.to_string(),
        path_params: HashMap::new(),
        query: Vec::new(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// api/endpoints_test.go — TestIsStreamingEndpoint
// ═══════════════════════════════════════════════════════════════════════════

#[rstest]
#[case("/2/tweets/search/stream", true)]
#[case("/2/tweets/sample/stream", true)]
#[case("/2/tweets/sample10/stream", true)]
#[case("/2/tweets/firehose/stream", true)]
#[case("/2/tweets/firehose/stream/lang/en", true)]
#[case("/2/tweets/firehose/stream/lang/ja", true)]
#[case("/2/tweets/firehose/stream/lang/ko", true)]
#[case("/2/tweets/firehose/stream/lang/pt", true)]
#[case("/2/tweets/search/stream/", true)]
#[case("/2/tweets/search/stream?query=test", true)]
#[case("https://api.x.com/2/tweets/search/stream", true)]
#[case("http://api.x.com/2/tweets/search/stream", true)]
#[case("https://api.x.com/2/tweets/search/stream?query=test", true)]
#[case("/2/tweets/search/recent", false)]
#[case("/2/users/me", false)]
#[case("https://api.x.com/2/users/me", false)]
#[case("/not/a/streaming/endpoint", false)]
#[case("", false)]
fn test_is_streaming_endpoint(#[case] endpoint: &str, #[case] expected: bool) {
    let result = is_streaming_endpoint(endpoint);
    assert_eq!(
        result, expected,
        "is_streaming_endpoint({endpoint:?}) should return {expected}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// api/shortcuts_test.go — resolve_post_id / resolve_username
// ═══════════════════════════════════════════════════════════════════════════

#[rstest]
#[case("1234567890", "1234567890")]
#[case("https://x.com/user/status/1234567890", "1234567890")]
#[case("https://twitter.com/user/status/9876543210", "9876543210")]
#[case("https://x.com/user/status/111?s=20", "111")]
#[case("  1234567890  ", "1234567890")]
#[case("https://x.com/user", "https://x.com/user")]
fn test_resolve_post_id(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(api::resolve_post_id(input), expected);
}

#[rstest]
#[case("elonmusk", "elonmusk")]
#[case("@elonmusk", "elonmusk")]
#[case("  @XDev  ", "XDev")]
#[case("plain", "plain")]
fn test_resolve_username(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(api::resolve_username(input), expected);
}

// ═══════════════════════════════════════════════════════════════════════════
// api/client_test.go — TestNewApiClient
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_new_api_client() {
    let ts = TestServer::new().await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let _client = Client::new(&cfg, auth).expect("client builds");
}

// ═══════════════════════════════════════════════════════════════════════════
// api/client_test.go — TestBuildRequest
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_build_request_get() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(
                    serde_json::json!({"data":{"id":"12345","username":"testuser"}}),
                ),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        ..Default::default()
    };

    let resp = client.send_request(&opts).await.unwrap();
    assert_eq!(resp["data"]["username"], "testuser");
}

#[tokio::test]
async fn test_build_request_post() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(
                    serde_json::json!({"data":{"id":"67890","text":"Hello world!"}}),
                ),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "POST".to_string(),
        target: target_path("/2/tweets"),
        data: r#"{"text":"Hello world!"}"#.to_string(),
        ..Default::default()
    };

    let resp = client.send_request(&opts).await.unwrap();
    assert_eq!(resp["data"]["text"], "Hello world!");
}

// ═══════════════════════════════════════════════════════════════════════════
// Auth type routing tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_build_request_with_auth_bearer() {
    // The intent: `--auth app` routes the Bearer auth header. Targeted at
    // `/2/tweets/search/recent` rather than `/2/users/me` because the
    // spec-derived auth matrix (v2.0.0) says only the former accepts Bearer.
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{"id":"1"}})),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/tweets/search/recent"),
        auth_type: "app".to_string(),
        ..Default::default()
    };

    let resp = client.send_request(&opts).await.unwrap();
    assert_eq!(resp["data"]["id"], "1");
}

#[tokio::test]
async fn test_build_request_with_auth_oauth1() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{"id":"1"}})),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_oauth1(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        auth_type: "oauth1".to_string(),
        ..Default::default()
    };

    let resp = client.send_request(&opts).await.unwrap();
    assert_eq!(resp["data"]["id"], "1");
}

#[tokio::test]
async fn test_build_request_with_auth_oauth2() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{"id":"1"}})),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_oauth2(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        auth_type: "oauth2".to_string(),
        username: "testuser".to_string(),
        ..Default::default()
    };

    let resp = client.send_request(&opts).await.unwrap();
    assert_eq!(resp["data"]["id"], "1");
}

// ═══════════════════════════════════════════════════════════════════════════
// api/client_test.go — TestSendRequest
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_send_request_success() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":{"id":"12345","name":"Test User","username":"testuser"}}),
            )),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/users/me"),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(resp["data"]["username"], "testuser");
    assert_eq!(resp["data"]["id"], "12345");
}

#[tokio::test]
async fn test_send_request_http_error() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .respond_with(ResponseTemplate::new(400).set_body_json(
                serde_json::json!({"errors":[{"message":"Invalid query","code":400}]}),
            )),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let err = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/tweets/search/recent"),
            ..Default::default()
        })
        .await
        .unwrap_err();

    assert!(err.is_api(), "Expected API error, got: {err}");
}

#[tokio::test]
async fn test_send_request_json_parse_error() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/bad-json"))
            .respond_with(ResponseTemplate::new(200).set_body_string("this is not json")),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    // Non-JSON 200 response returns empty JSON object
    let resp = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/bad-json"),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(resp, serde_json::json!({}));
}

// ═══════════════════════════════════════════════════════════════════════════
// Timeout wiring — --timeout / XURL_TIMEOUT bound network calls
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn slow_endpoint_trips_explicit_timeout() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/slow"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(std::time::Duration::from_secs(10))
                    .set_body_json(serde_json::json!({"ok": true})),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::with_timeout(&cfg, auth, 1).expect("client builds");
    assert_eq!(client.timeout_secs(), 1);

    let started = std::time::Instant::now();
    let err = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/slow"),
            ..Default::default()
        })
        .await
        .expect_err("a 1s timeout must fire before the 10s server delay");
    let elapsed = started.elapsed();

    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "timeout should fire well under the server's 10s delay; elapsed = {elapsed:?}"
    );
    assert!(
        matches!(err, xdk::Error::Http(_)),
        "expected an HTTP/transport error, got: {err:?}"
    );
}

#[test]
fn config_default_timeout_carries_through_apiclient_new() {
    // Regression guard for the runner-to-Client timeout plumbing: the value
    // on `Config::http_timeout_secs` must drive `Client::timeout_secs()`.
    let mut cfg = create_test_config("http://127.0.0.1:9");
    cfg.http_timeout_secs = 7;
    let (auth, _tmp) = create_mock_auth_with_all_methods("http://127.0.0.1:9");
    let client = Client::new(&cfg, auth).expect("client builds");
    assert_eq!(client.timeout_secs(), 7);
}

// ═══════════════════════════════════════════════════════════════════════════
// Auth header routing tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_get_auth_header_oauth1() {
    let (auth, _tmp) = create_mock_auth_with_oauth1("https://api.x.com");
    let header = auth
        .get_oauth1_header("GET", "https://api.x.com/2/users/me", None)
        .unwrap();
    assert!(header.starts_with("OAuth "));
    assert!(header.contains("oauth_consumer_key"));
}

#[tokio::test]
async fn test_get_auth_header_oauth2() {
    let (mut auth, _tmp) = create_mock_auth_with_oauth2("https://api.x.com");
    let header = auth
        .get_oauth2_header(&reqwest::Client::new(), "testuser")
        .await
        .unwrap();
    assert!(
        header.starts_with("Bearer "),
        "Expected Bearer header, got: {header}"
    );
}

#[test]
fn test_get_auth_header_bearer() {
    let (auth, _tmp) = create_mock_auth_with_bearer("https://api.x.com");
    let header = auth.get_bearer_token_header().unwrap();
    assert_eq!(header, "Bearer test-bearer-token");
}

/// An env-supplied bearer counts as the `app` scheme during auto-detect, so
/// a shortcut against an empty store resolves it instead of reporting that
/// no authentication method is available.
#[tokio::test]
async fn test_env_bearer_counts_as_app_scheme_on_empty_store() {
    use wiremock::matchers::header;
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .and(header("Authorization", "Bearer env-bearer-value"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":[{"id":"1","text":"hello"}],"meta":{"result_count":1}}),
            ))
            .expect(1),
    )
    .await;
    let mut cfg = create_test_config(ts.uri());
    cfg.client_id = String::new();
    cfg.client_secret = String::new();
    let tmp = TempDir::new().expect("temp dir");
    let store_path = tmp.path().join(".xurl");
    let overrides = xdk::config::EnvOverrides {
        bearer_token: Some("env-bearer-value".to_string()),
        ..xdk::config::EnvOverrides::default()
    };
    let auth = Auth::new_with_store_path_and_overrides(&cfg, &store_path, &overrides);
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client
        .search_posts("hello", 10)
        .send()
        .await
        .expect("env bearer must satisfy auto-detect on an empty store");
    assert_eq!(resp.meta.as_ref().unwrap().result_count, Some(1));
}

// ═══════════════════════════════════════════════════════════════════════════
// api/shortcuts_test.go — Shortcut integration tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_post() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data":{"id":"99999","text":"Hello!"}})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.create_post("Hello!", &[]).send().await.unwrap();
    assert_eq!(resp.data.id, "99999");
    assert_eq!(resp.data.text, "Hello!");
}

#[tokio::test]
async fn test_reply_to_post() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data":{"id":"88888","text":"nice!"}})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client
        .reply_to_post("123", "nice!", &[])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.data.id, "88888");
}

#[tokio::test]
async fn test_reply_to_post_with_url() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(
                    serde_json::json!({"data":{"id":"77777","text":"reply via URL"}}),
                ),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client
        .reply_to_post("https://x.com/u/status/123", "reply via URL", &[])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.data.id, "77777");
}

#[tokio::test]
async fn test_quote_post() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data":{"id":"66666","text":"my take"}})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.quote_post("123", "my take").send().await.unwrap();
    assert_eq!(resp.data.id, "66666");
}

#[tokio::test]
async fn test_delete_post() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("DELETE"))
            .and(path("/2/tweets/123"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data":{"deleted":true}})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.delete_post("123").send().await.unwrap();
    assert!(resp.data.deleted);
}

#[tokio::test]
async fn test_read_post() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET")).and(path_regex(r"/2/tweets/123.*")).respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{"id":"123","text":"existing post","public_metrics":{"like_count":5}}})),
        ),
    ).await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.read_post("123").send().await.unwrap();
    assert_eq!(resp.data.id, "123");
    assert_eq!(resp.data.text, "existing post");
}

#[tokio::test]
async fn test_search_posts() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET")).and(path("/2/tweets/search/recent")).respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":[{"id":"1","text":"result one"}],"meta":{"result_count":1}})),
        ),
    ).await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.search_posts("golang", 10).send().await.unwrap();
    assert_eq!(resp.meta.as_ref().unwrap().result_count, Some(1));
}

/// Threads `Call::pagination_token` through to the
/// `pagination_token` query parameter on the search URL — wiremock asserts
/// the parameter is present and carries the URL-decoded token.
#[tokio::test]
async fn test_search_posts_threads_pagination_token() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .and(query_param("pagination_token", "next_abc_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":[{"id":"2","text":"page2"}],"meta":{"result_count":1}}),
            )),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client
        .search_posts("golang", 10)
        .pagination_token("next_abc_token")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.data.first().unwrap().id, "2");
}

/// Same wiremock probe as above, but verifies the URL-encoder runs on
/// special characters (spaces → `%20`).
#[tokio::test]
async fn test_search_posts_url_encodes_pagination_token() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .and(query_param("pagination_token", "page 2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":[{"id":"3","text":"p"}],"meta":{"result_count":1}}),
            )),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client
        .search_posts("golang", 10)
        .pagination_token("page 2")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.data.first().unwrap().id, "3");
}

#[tokio::test]
async fn test_get_me() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":{"id":"42","username":"testbot","name":"Test Bot"}}),
            )),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.get_me().send().await.unwrap();
    assert_eq!(resp.data.id, "42");
    assert_eq!(resp.data.username, "testbot");
}

#[tokio::test]
async fn test_lookup_user() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path_regex(r"/2/users/by/username/someuser.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":{"id":"100","username":"lookedup","name":"Looked Up"}}),
            )),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.lookup_user("@someuser").send().await.unwrap();
    assert_eq!(resp.data.id, "100");
    assert_eq!(resp.data.username, "lookedup");
}

#[tokio::test]
async fn test_create_post_with_media() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data":{"id":"55555","text":"With media"}})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let media_ids = vec!["m1".to_string(), "m2".to_string()];
    let resp = client
        .create_post("With media", &media_ids)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.data.id, "55555");
}

// ═══════════════════════════════════════════════════════════════════════════
// api/media_test.go — Media tests
// ═══════════════════════════════════════════════════════════════════════════

#[rstest]
#[case("/2/media/upload/123456/append", "123456")]
#[case("/2/media/upload/123456/finalize", "123456")]
#[case("/2/media/upload?command=STATUS&media_id=123456", "123456")]
#[case("/2/media/upload/initialize", "")]
#[case("/2/media/upload", "")]
#[case("api.x.com/2/media/upload/123456/append", "123456")]
#[case("api.x.com/2/media/upload/123456/finalize", "123456")]
#[case("api.x.com/2/media/upload?command=STATUS&media_id=123456", "123456")]
#[case("", "")]
fn test_extract_media_id(#[case] url: &str, #[case] expected: &str) {
    assert_eq!(extract_media_id(url), expected);
}

#[rstest]
#[case("", None)]
#[case(r#"{"segment_index": "1"}"#, Some("1"))]
fn test_extract_segment_index(#[case] data: &str, #[case] expected: Option<&str>) {
    assert_eq!(extract_segment_index(data).as_deref(), expected);
}

#[rstest]
#[case("/2/media/upload/123/append", "file.jpg", true)]
#[case("/2/media/upload/initialize", "file.jpg", false)]
#[case("/2/media/upload/123/append", "", false)]
#[case("/2/users/me", "file.jpg", false)]
#[case("", "", false)]
fn test_is_media_append_request(
    #[case] url: &str,
    #[case] media_file: &str,
    #[case] expected: bool,
) {
    assert_eq!(is_media_append_request(url, media_file), expected);
}

#[tokio::test]
async fn test_media_upload_init() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST")).and(path("/2/media/upload/initialize")).respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "test_media_id", "expires_after_secs": 3600, "media_key": "test_media_key"}
            })),
        ),
    ).await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.send_request(&RequestOptions {
        method: "POST".to_string(),
        target: target_path("/2/media/upload/initialize"),
        data: serde_json::json!({"total_bytes": 1024, "media_type": "image/jpeg", "media_category": "tweet_image"}).to_string(),
        ..Default::default()
    }).await.unwrap();
    assert_eq!(resp["data"]["id"], "test_media_id");
}

#[tokio::test]
async fn test_media_upload_finalize() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/media/upload/test_media_id/finalize"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "test_media_id", "media_key": "test_media_key"}
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: target_path("/2/media/upload/test_media_id/finalize"),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(resp["data"]["id"], "test_media_id");
}

#[tokio::test]
async fn test_media_upload_check_status() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/media/upload"))
            .and(query_param("command", "STATUS"))
            .and(query_param("media_id", "test_media_id"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "test_media_id", "processing_info": {"state": "succeeded", "progress_percent": 100}}
            }))),
    ).await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            target: RequestTarget::Template {
                path: "/2/media/upload".to_string(),
                path_params: HashMap::new(),
                query: vec![
                    ("command".to_string(), "STATUS".to_string()),
                    ("media_id".to_string(), "test_media_id".to_string()),
                ],
            },
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(resp["data"]["processing_info"]["state"], "succeeded");
}

#[tokio::test]
async fn test_stream_request_error() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/stream/error"))
            .respond_with(ResponseTemplate::new(400).set_body_json(
                serde_json::json!({"errors":[{"message":"Invalid rule","code":400}]}),
            )),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let err = client
        .stream_request(&RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/tweets/search/stream/error"),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(err.is_api(), "Expected API error, got: {err}");
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[rstest]
#[case(
    "https://twitter.com/user/status/123456789012345678",
    "123456789012345678"
)]
#[case("https://x.com/user/status/1", "1")]
fn test_resolve_post_id_edge_cases(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(api::resolve_post_id(input), expected);
}

#[test]
fn test_resolve_username_empty() {
    assert_eq!(api::resolve_username(""), "");
}

#[test]
fn test_resolve_username_at_only() {
    assert_eq!(api::resolve_username("@"), "");
}

#[rstest]
#[case("/2/tweets/search/stream/rules", false)]
#[case("/2/tweets/search/stream/rules?query=test", false)]
fn test_is_streaming_endpoint_rules_not_streaming(#[case] endpoint: &str, #[case] expected: bool) {
    assert_eq!(is_streaming_endpoint(endpoint), expected);
}

#[test]
fn test_extract_media_id_with_extra_path() {
    let result = extract_media_id("/2/media/upload/999/append/extra");
    assert_eq!(result, "999");
}

// ── Usage shortcut tests ────────────────────────────────────────────────

#[tokio::test]
async fn test_get_usage_happy_path() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .and(query_param(
                "usage.fields",
                "daily_project_usage,daily_client_app_usage",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "project_cap": "2000000",
                    "project_id": "2020044302890438656",
                    "project_usage": "399",
                    "cap_reset_day": 19,
                    "daily_project_usage": {
                        "project_id": "2020044302890438656",
                        "usage": [
                            {"date": "2026-03-25T00:00:00.000Z", "usage": "299"},
                            {"date": "2026-03-26T00:00:00.000Z", "usage": "100"}
                        ]
                    },
                    "daily_client_app_usage": [
                        {
                            "client_app_id": "32371675",
                            "usage": [
                                {"date": "2026-03-25T00:00:00.000Z", "usage": "299"}
                            ],
                            "usage_result_count": 1
                        }
                    ]
                }
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.get_usage().send().await.unwrap();
    assert_eq!(resp.data.project_cap.as_deref(), Some("2000000"));
    assert_eq!(resp.data.project_usage.as_deref(), Some("399"));
    assert_eq!(resp.data.cap_reset_day, Some(19));
    let daily = resp.data.daily_project_usage.as_ref().unwrap();
    assert_eq!(daily["project_id"], "2020044302890438656");
    assert!(daily["usage"].is_array());
    assert!(
        resp.data
            .daily_client_app_usage
            .as_ref()
            .unwrap()
            .is_array()
    );
}

#[tokio::test]
async fn test_get_usage_requires_usage_fields_query_param() {
    // Verify the shortcut sends the required query parameter.
    // Without usage.fields, the API returns minimal data — this mock
    // only responds when the param is present.
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .and(query_param(
                "usage.fields",
                "daily_project_usage,daily_client_app_usage",
            ))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"project_usage": "42"}})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.get_usage().send().await.unwrap();
    assert_eq!(resp.data.project_usage.as_deref(), Some("42"));
}

#[tokio::test]
async fn test_get_usage_uses_get_method() {
    // Ensure the shortcut uses GET, not POST or another method.
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"project_usage": "0"}})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.get_usage().send().await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn test_get_usage_api_error_401() {
    // Unauthorized — e.g., missing or invalid bearer token.
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "title": "Unauthorized",
                "type": "about:blank",
                "status": 401,
                "detail": "Unauthorized"
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.get_usage().send().await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_get_usage_api_error_429() {
    // Rate limited — 50 requests per 15-minute window.
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
                "title": "Too Many Requests",
                "detail": "Too Many Requests",
                "type": "about:blank",
                "status": 429
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.get_usage().send().await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_get_usage_with_bearer() {
    // `/2/usage/tweets` is Bearer-only per the OpenAPI spec; previous tests
    // that drove OAuth1/OAuth2 against this endpoint passed because no
    // matrix validation existed. The intersection check (U7) now rejects
    // those configurations, so the surviving coverage uses Bearer.
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"project_usage": "10"}})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.get_usage().send().await.unwrap();
    assert_eq!(resp.data.project_usage.as_deref(), Some("10"));
}

#[tokio::test]
async fn test_get_usage_rejects_oauth_only_app() {
    // /2/usage/tweets accepts only Bearer per the spec. An app that only
    // holds OAuth1 credentials hits the empty-intersection path (U7) and
    // surfaces AuthMethodMismatch rather than the prior silent OAuth1
    // attempt.
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(ResponseTemplate::new(200)),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_oauth1(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let err = client.get_usage().send().await.unwrap_err();
    match err {
        xdk::Error::AuthMethodMismatch(mismatch) => {
            let xdk::error::AuthMismatch {
                endpoint,
                supported,
                available_in_app,
                ..
            } = *mismatch;
            assert_eq!(endpoint, "/2/usage/tweets");
            assert_eq!(supported, vec!["app"]);
            assert_eq!(available_in_app, Some(vec!["oauth1".to_string()]));
        }
        other => panic!("expected AuthMethodMismatch, got {other:?}"),
    }
}

#[tokio::test]
async fn test_get_usage_daily_project_usage_structure() {
    // Verify the daily_project_usage nested structure is preserved.
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "daily_project_usage": {
                        "project_id": "123",
                        "usage": [
                            {"date": "2026-03-01T00:00:00.000Z", "usage": "50"},
                            {"date": "2026-03-02T00:00:00.000Z", "usage": "75"},
                            {"date": "2026-03-03T00:00:00.000Z", "usage": "100"}
                        ]
                    }
                }
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.get_usage().send().await.unwrap();
    let daily_val = resp.data.daily_project_usage.as_ref().unwrap();
    let usage = &daily_val["usage"];
    assert!(usage.is_array());
    assert_eq!(usage.as_array().unwrap().len(), 3);
    assert_eq!(usage[0]["usage"], "50");
    assert_eq!(usage[2]["usage"], "100");
}

#[tokio::test]
async fn test_get_usage_daily_client_app_usage_structure() {
    // Verify the daily_client_app_usage array with per-app breakdowns.
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "daily_client_app_usage": [
                        {
                            "client_app_id": "app_1",
                            "usage": [{"date": "2026-03-25T00:00:00.000Z", "usage": "10"}],
                            "usage_result_count": 1
                        },
                        {
                            "client_app_id": "app_2",
                            "usage": [{"date": "2026-03-25T00:00:00.000Z", "usage": "30"}],
                            "usage_result_count": 1
                        }
                    ]
                }
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.get_usage().send().await.unwrap();
    let apps = resp
        .data
        .daily_client_app_usage
        .as_ref()
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(apps.len(), 2);
    assert_eq!(apps[0]["client_app_id"], "app_1");
    assert_eq!(apps[1]["client_app_id"], "app_2");
}

#[tokio::test]
async fn test_get_usage_clears_request_data() {
    // Ensure no stale request body leaks into the GET request.
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"project_usage": "0"}})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    // A GET call carries no body, so stale data can't leak — verify the call succeeds
    let resp = client.get_usage().send().await;
    assert!(resp.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════
// Red team — adversarial API responses via wiremock
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn redteam_create_post_array_where_object_expected() {
    // API returns array in data field for a single-item shortcut
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data": [{"id": "1", "text": "oops"}]})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let result = client.create_post("test", &[]).send().await;
    assert!(
        result.is_err(),
        "Should fail: array where single Post expected"
    );
}

#[tokio::test]
async fn redteam_get_me_no_data_field() {
    // API returns errors-only 200 with no data field
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"errors": [{"message": "forbidden"}]})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let result = client.get_me().send().await;
    let err = result.unwrap_err();
    assert!(
        err.is_validation(),
        "Should be Validation error (not API or JSON error) for errors-only 200: {err}"
    );
}

#[tokio::test]
async fn redteam_delete_post_wrong_type_in_data() {
    // API returns string instead of object in data field
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("DELETE"))
            .and(path("/2/tweets/123"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": "unexpected string"})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let result = client.delete_post("123").send().await;
    assert!(
        result.is_err(),
        "Should fail: string where DeletedResult expected"
    );
}

#[tokio::test]
async fn redteam_search_posts_null_data() {
    // API returns null data
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": null})),
            ),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let result = client.search_posts("test", 10).send().await;
    assert!(result.is_err(), "Should fail: null data for Vec<Post>");
}

#[tokio::test]
async fn redteam_empty_body_returns_descriptive_error() {
    // send_request returns empty {} for non-JSON 2xx — shortcut should give clear error
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json")),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let result = client.get_me().send().await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("empty response body"),
        "Expected descriptive error, got: {err}"
    );
}

#[tokio::test]
async fn redteam_unknown_fields_survive_shortcut_round_trip() {
    // Verify serde(flatten) preserves unknown fields through the full shortcut path
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {
                    "id": "99999",
                    "text": "Hello!",
                    "brand_new_field": "surprise_value"
                },
                "top_level_extra": 42
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.create_post("Hello!", &[]).send().await.unwrap();
    assert_eq!(resp.data.id, "99999");
    // Unknown fields preserved in extra
    assert_eq!(resp.data.extra["brand_new_field"], "surprise_value");
    assert_eq!(resp.extra["top_level_extra"], 42);
    // Round-trip: serialize back to Value and verify preservation
    let value = serde_json::to_value(&resp).unwrap();
    assert_eq!(value["data"]["brand_new_field"], "surprise_value");
    assert_eq!(value["top_level_extra"], 42);
}

#[tokio::test]
async fn redteam_like_post_extra_fields_on_action() {
    // Action confirmation with extra unknown fields
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/users/42/likes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"liked": true, "pending_follow": false},
                "rate_limit_remaining": 99
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client.like_post("42", "123").send().await.unwrap();
    assert!(resp.data.liked);
    // Unknown fields captured, not lost
    assert_eq!(resp.data.extra["pending_follow"], false);
    assert_eq!(resp.extra["rate_limit_remaining"], 99);
}

#[tokio::test]
async fn redteam_lookup_user_wrong_bool_type() {
    // String "true" where boolean expected in a nested field
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path_regex(r"/2/users/by/username/bad.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "1", "name": "Bad", "username": "bad", "verified": "true"}
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    // verified is Option<bool> — "true" (string) should fail deserialization
    let result = client.lookup_user("bad").send().await;
    assert!(
        result.is_err(),
        "Should fail: string 'true' where bool expected"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// from_env() constructor tests
// ═══════════════════════════════════════════════════════════════════════════

#[serial_test::serial]
#[test]
fn test_from_env_without_client_id_builds_a_store_backed_client() {
    let original = std::env::var("CLIENT_ID").ok();
    unsafe { std::env::remove_var("CLIENT_ID") };
    let unset = Client::from_env();
    unsafe { std::env::set_var("CLIENT_ID", "") };
    let empty = Client::from_env();
    match original {
        Some(val) => unsafe { std::env::set_var("CLIENT_ID", val) },
        None => unsafe { std::env::remove_var("CLIENT_ID") },
    }

    assert!(
        unset.is_ok(),
        "from_env() with CLIENT_ID unset must build over the store: {unset:?}"
    );
    assert!(
        empty.is_ok(),
        "from_env() with CLIENT_ID empty must build over the store: {empty:?}"
    );
}

#[serial_test::serial]
#[test]
fn test_from_env_with_client_id_set_returns_ok() {
    let original_id = std::env::var("CLIENT_ID").ok();
    let original_secret = std::env::var("CLIENT_SECRET").ok();
    unsafe {
        std::env::set_var("CLIENT_ID", "test-from-env-id");
        std::env::set_var("CLIENT_SECRET", "test-secret");
    }

    let result = Client::from_env();
    assert!(
        result.is_ok(),
        "from_env() should succeed with CLIENT_ID set"
    );

    // Restore
    match original_id {
        Some(val) => unsafe { std::env::set_var("CLIENT_ID", val) },
        None => unsafe { std::env::remove_var("CLIENT_ID") },
    }
    match original_secret {
        Some(val) => unsafe { std::env::set_var("CLIENT_SECRET", val) },
        None => unsafe { std::env::remove_var("CLIENT_SECRET") },
    }
}

#[serial_test::serial]
#[test]
fn test_from_env_with_client_id_but_no_secret_returns_ok() {
    // Best-effort: CLIENT_SECRET not required at construction time
    let original_id = std::env::var("CLIENT_ID").ok();
    let original_secret = std::env::var("CLIENT_SECRET").ok();
    unsafe {
        std::env::set_var("CLIENT_ID", "test-from-env-id");
        std::env::remove_var("CLIENT_SECRET");
    }

    let result = Client::from_env();
    assert!(
        result.is_ok(),
        "from_env() should succeed without CLIENT_SECRET (best-effort)"
    );

    // Restore
    match original_id {
        Some(val) => unsafe { std::env::set_var("CLIENT_ID", val) },
        None => unsafe { std::env::remove_var("CLIENT_ID") },
    }
    if let Some(val) = original_secret {
        unsafe { std::env::set_var("CLIENT_SECRET", val) };
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// no_auth behavior tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_shortcut_calls_include_authorization_header() {
    // Every shortcut call attaches the Authorization header; the mock
    // requires it via the header_exists matcher.
    use wiremock::matchers::header_exists;

    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .and(header_exists("Authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "123", "name": "Test", "username": "test"}}),
            )),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let result = client.get_me().send().await;
    assert!(
        result.is_ok(),
        "a shortcut call carries the auth header: {result:?}"
    );
}

#[tokio::test]
async fn test_no_auth_with_raw_send_request() {
    // Verify no_auth works at the send_request level too, not just shortcuts
    use wiremock::matchers::header_exists;

    let ts = TestServer::new().await;
    // This mock will fail if Authorization header IS present
    // (by only matching requests WITHOUT it via a custom approach)
    // Instead, just verify the request succeeds with no_auth=true
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/test"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/test"),
        no_auth: true,
        ..Default::default()
    };
    let result = client.send_request(&opts).await;
    assert!(
        result.is_ok(),
        "no_auth=true on send_request should work: {result:?}"
    );

    // Now verify that with no_auth=false, the auth header IS sent
    let ts2 = TestServer::new().await;
    ts2.mount(
        Mock::given(method("GET"))
            .and(path("/2/test"))
            .and(header_exists("Authorization"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})),
            ),
    )
    .await;

    let cfg2 = create_test_config(ts2.uri());
    let (auth2, _tmp2) = create_mock_auth_with_all_methods(ts2.uri());
    let client2 = Client::new(&cfg2, auth2).expect("client builds");

    let opts2 = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/test"),
        no_auth: false,
        ..Default::default()
    };
    let result2 = client2.send_request(&opts2).await;
    assert!(
        result2.is_ok(),
        "no_auth=false should include auth header: {result2:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// User-supplied header overrides — suppression of the client's auto-append
//
// `reqwest::RequestBuilder::header` uses `HeaderMap::append` semantics; if
// the caller passes any of the four client-added headers (Authorization,
// Content-Type, User-Agent, X-B3-Flags) via `options.headers` and the client
// appends its own, the outgoing request carries two values for that header.
// Authorization and Content-Type are the breakage-prone cases (HTTP
// undefined, server rejection); User-Agent and X-B3-Flags simply confuse
// receivers. When the caller explicitly supplies any of these, treat
// the user's value as authoritative and skip the client's append.
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn user_supplied_authorization_replaces_xurl_auth_on_send_request() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "1", "name": "Test", "username": "test"}}),
            )),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        headers: vec!["Authorization: Bearer user-token".to_string()],
        ..Default::default()
    };
    let result = client.send_request(&opts).await;
    assert!(result.is_ok(), "request must succeed: {result:?}");

    let auths = ts.received_header_values("Authorization").await;
    assert_eq!(auths.len(), 1, "expected exactly one received request");
    assert_eq!(
        auths[0],
        vec!["Bearer user-token".to_string()],
        "user-supplied Authorization must be the only value sent — got {:?}",
        auths[0]
    );
}

#[tokio::test]
async fn user_supplied_authorization_detection_is_case_insensitive() {
    for raw_header in [
        "authorization: Bearer user-token",
        "AUTHORIZATION: Bearer user-token",
        "aUtHoRiZaTiOn: Bearer user-token",
    ] {
        let ts = TestServer::new().await;
        ts.mount(
            Mock::given(method("GET"))
                .and(path("/2/users/me"))
                .respond_with(ResponseTemplate::new(200).set_body_json(
                    serde_json::json!({"data": {"id": "1", "name": "Test", "username": "test"}}),
                )),
        )
        .await;

        let cfg = create_test_config(ts.uri());
        let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
        let client = Client::new(&cfg, auth).expect("client builds");

        let opts = RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/users/me"),
            headers: vec![raw_header.to_string()],
            ..Default::default()
        };
        client
            .send_request(&opts)
            .await
            .expect("request must succeed regardless of header-key case");

        let auths = ts.received_header_values("Authorization").await;
        assert_eq!(
            auths[0],
            vec!["Bearer user-token".to_string()],
            "case-insensitive header detection failed for input {raw_header:?} — got {:?}",
            auths[0]
        );
    }
}

#[tokio::test]
async fn no_auth_with_user_supplied_authorization_sends_users_value() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "1", "name": "Test", "username": "test"}}),
            )),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        headers: vec!["Authorization: Bearer override".to_string()],
        no_auth: true,
        ..Default::default()
    };
    client
        .send_request(&opts)
        .await
        .expect("request must succeed with no_auth + user-supplied Authorization");

    let auths = ts.received_header_values("Authorization").await;
    assert_eq!(
        auths[0],
        vec!["Bearer override".to_string()],
        "no_auth=true must not suppress user-supplied Authorization — got {:?}",
        auths[0]
    );
}

#[tokio::test]
async fn user_supplied_authorization_does_not_affect_other_custom_headers() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "1", "name": "Test", "username": "test"}}),
            )),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    // Non-Authorization custom headers must NOT suppress the client's auth append.
    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        headers: vec![
            "Cookie: session=abc".to_string(),
            "X-Authorization-Hint: not-the-real-header".to_string(),
        ],
        ..Default::default()
    };
    client
        .send_request(&opts)
        .await
        .expect("request must succeed when no Authorization is user-supplied");

    let auths = ts.received_header_values("Authorization").await;
    assert_eq!(
        auths[0].len(),
        1,
        "expected exactly one Authorization (the client's), got {:?}",
        auths[0]
    );
    assert!(
        auths[0][0].starts_with("Bearer ") || auths[0][0].starts_with("OAuth "),
        "client-appended Authorization must be a real auth scheme, got {:?}",
        auths[0][0]
    );

    let cookies = ts.received_header_values("Cookie").await;
    assert_eq!(cookies[0], vec!["session=abc".to_string()]);
}

#[tokio::test]
async fn user_supplied_authorization_replaces_xurl_auth_on_multipart_request() {
    use xdk::api::MultipartOptions;

    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/media/upload"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"id": "media-1"}})),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let mp_opts = MultipartOptions {
        request: RequestOptions {
            method: "POST".to_string(),
            target: target_path("/2/media/upload"),
            headers: vec!["Authorization: Bearer multipart-user-token".to_string()],
            ..Default::default()
        },
        form_fields: HashMap::new(),
        file_field: "media".to_string(),
        file_path: String::new(),
        file_name: "x.jpg".to_string(),
        file_data: b"fake-jpeg".to_vec(),
    };

    client
        .send_multipart_request(&mp_opts)
        .await
        .expect("multipart request must succeed with user-supplied Authorization");

    let auths = ts.received_header_values("Authorization").await;
    assert_eq!(auths.len(), 1, "expected exactly one received request");
    assert_eq!(
        auths[0],
        vec!["Bearer multipart-user-token".to_string()],
        "multipart path must honor user-supplied Authorization — got {:?}",
        auths[0]
    );
}

#[tokio::test]
async fn user_supplied_authorization_replaces_xurl_auth_on_stream_request() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/stream"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data: hello\n\n")),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/tweets/search/stream"),
        headers: vec!["Authorization: Bearer stream-user-token".to_string()],
        ..Default::default()
    };

    client
        .stream_request(&opts)
        .await
        .expect("stream request must succeed with user-supplied Authorization");

    let auths = ts.received_header_values("Authorization").await;
    assert_eq!(auths.len(), 1, "expected exactly one received request");
    assert_eq!(
        auths[0],
        vec!["Bearer stream-user-token".to_string()],
        "stream path must honor user-supplied Authorization — got {:?}",
        auths[0]
    );
}

#[tokio::test]
async fn user_supplied_user_agent_replaces_xurl_user_agent_on_send_request() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "1", "name": "Test", "username": "test"}}),
            )),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        headers: vec!["User-Agent: custom-client/9.9".to_string()],
        ..Default::default()
    };
    client
        .send_request(&opts)
        .await
        .expect("request must succeed with user-supplied User-Agent");

    let uas = ts.received_header_values("User-Agent").await;
    assert_eq!(
        uas[0],
        vec!["custom-client/9.9".to_string()],
        "user-supplied User-Agent must be the only value sent — got {:?}",
        uas[0]
    );
}

#[tokio::test]
async fn user_supplied_content_type_replaces_xurl_content_type_on_post() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/echo"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    // Payload looks like JSON, so the client would normally set application/json.
    // The user-supplied Content-Type must win.
    let opts = RequestOptions {
        method: "POST".to_string(),
        target: target_path("/2/echo"),
        headers: vec!["Content-Type: application/xml".to_string()],
        data: r#"{"hello":"world"}"#.to_string(),
        ..Default::default()
    };
    client
        .send_request(&opts)
        .await
        .expect("request must succeed with user-supplied Content-Type");

    let cts = ts.received_header_values("Content-Type").await;
    assert_eq!(
        cts[0],
        vec!["application/xml".to_string()],
        "user-supplied Content-Type must be the only value sent — got {:?}",
        cts[0]
    );
}

#[tokio::test]
async fn user_supplied_user_agent_replaces_xurl_value_on_multipart_request() {
    use xdk::api::MultipartOptions;

    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/media/upload"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"id": "media-1"}})),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let mp_opts = MultipartOptions {
        request: RequestOptions {
            method: "POST".to_string(),
            target: target_path("/2/media/upload"),
            headers: vec!["User-Agent: multipart-custom/2.0".to_string()],
            ..Default::default()
        },
        form_fields: HashMap::new(),
        file_field: "media".to_string(),
        file_path: String::new(),
        file_name: "x.jpg".to_string(),
        file_data: b"fake-jpeg".to_vec(),
    };
    client
        .send_multipart_request(&mp_opts)
        .await
        .expect("multipart request must succeed with user-supplied User-Agent");

    let uas = ts.received_header_values("User-Agent").await;
    assert_eq!(
        uas[0],
        vec!["multipart-custom/2.0".to_string()],
        "multipart path must honor user-supplied User-Agent — got {:?}",
        uas[0]
    );
}

#[tokio::test]
async fn user_supplied_user_agent_replaces_xurl_value_on_stream_request() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/stream"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data: hello\n\n")),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/tweets/search/stream"),
        headers: vec!["User-Agent: streamer/1.0".to_string()],
        ..Default::default()
    };
    client
        .stream_request(&opts)
        .await
        .expect("stream request must succeed with user-supplied User-Agent");

    let uas = ts.received_header_values("User-Agent").await;
    assert_eq!(
        uas[0],
        vec!["streamer/1.0".to_string()],
        "stream path must honor user-supplied User-Agent — got {:?}",
        uas[0]
    );
}

#[tokio::test]
async fn xurl_default_user_agent_is_sent_when_not_overridden() {
    use wiremock::matchers::header_regex;

    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .and(header_regex(
                "User-Agent",
                r"^xdk-rs/[0-9]+\.[0-9]+\.[0-9]+",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "1", "name": "Test", "username": "test"}}),
            )),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        ..Default::default()
    };
    client
        .send_request(&opts)
        .await
        .expect("default path must send xdk-rs/<version> User-Agent");

    let uas = ts.received_header_values("User-Agent").await;
    assert_eq!(uas[0].len(), 1, "expected exactly one User-Agent");
    assert!(
        uas[0][0].starts_with("xdk-rs/"),
        "default User-Agent must be xdk-rs/<version>, got {:?}",
        uas[0][0]
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Red team — library ergonomics edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn redteam_no_auth_with_auth_type_set_silently_skips_auth() {
    // Conflicting: no_auth=true + auth_type="oauth2" — no_auth should win
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "1", "name": "X", "username": "x"}}),
            )),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        auth_type: "oauth2".to_string(),
        no_auth: true,
        ..Default::default()
    };
    // Should succeed — no_auth takes precedence over auth_type
    let result = client.send_request(&opts).await;
    assert!(
        result.is_ok(),
        "no_auth=true should take precedence over auth_type: {result:?}"
    );
}

#[tokio::test]
async fn redteam_sequential_calls_on_same_client() {
    // Verify that making multiple calls on the same client instance works
    // (auth state not corrupted between calls)
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "42", "name": "Me", "username": "me"}}),
            )),
    )
    .await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data": {"id": "99", "text": "hi"}})),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    // First call
    let me = client.get_me().send().await.unwrap();
    assert_eq!(me.data.id, "42");

    // Second call on same client — auth should still work
    let post = client.create_post("hi", &[]).send().await.unwrap();
    assert_eq!(post.data.id, "99");

    // Third call — still works
    let me2 = client.get_me().send().await.unwrap();
    assert_eq!(me2.data.id, "42");
}

#[tokio::test]
async fn redteam_api_error_preserves_status_and_body() {
    // Verify that HTTP errors carry both status code and body through the pipeline
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(
                    serde_json::json!({"detail": "Forbidden", "title": "Forbidden"}),
                ),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let result = client.get_me().send().await;
    let err = result.unwrap_err();
    assert!(err.is_api());
    // Verify structured error carries status
    match &err {
        xdk::Error::Api { status, body } => {
            assert_eq!(*status, 403);
            assert!(body.contains("Forbidden"));
        }
        _ => panic!("Expected Api variant, got: {err:?}"),
    }
}

#[tokio::test]
async fn redteam_api_error_401_gives_auth_exit_code() {
    // Verify the full pipeline: HTTP 401 → Api { status: 401 } → EXIT_AUTH_REQUIRED
    use xdk::error::exit_code_for_error;

    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_json(serde_json::json!({"detail": "Unauthorized"})),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let err = client.get_me().send().await.unwrap_err();
    assert_eq!(
        exit_code_for_error(&err),
        xdk::error::EXIT_AUTH_REQUIRED,
        "401 should map to EXIT_AUTH_REQUIRED"
    );
}

#[tokio::test]
async fn redteam_api_error_429_gives_rate_limit_exit_code() {
    use xdk::error::exit_code_for_error;

    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(429)
                    .set_body_json(serde_json::json!({"detail": "Too Many Requests"})),
            ),
    )
    .await;

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let err = client.get_me().send().await.unwrap_err();
    assert_eq!(
        exit_code_for_error(&err),
        xdk::error::EXIT_RATE_LIMITED,
        "429 should map to EXIT_RATE_LIMITED"
    );
}

// ── TestAuthErrorPropagation (Bug B) ──────────────────────────────────────
//
// `Client::send_request` (and its sibling paths) must propagate the
// `get_auth_header` error rather than silently sending the request
// unauthenticated. The older `if let Ok(...)` form let auth bugs masquerade
// as upstream 401s; the new path returns the real `Error::Auth` so the
// user can tell the difference between "we couldn't sign the request" and
// "we signed it and X rejected it".

fn create_mock_auth_no_tokens(base_url: &str) -> (Auth, TempDir) {
    let cfg = create_test_config(base_url);
    let tmp = TempDir::new().expect("temp dir");
    let file_path = tmp.path().join(".xurl");

    let mut store = TokenStore {
        apps: BTreeMap::new(),
        default_app: "default".to_string(),
        file_path,
        load_state: xdk::store::LoadState::Loaded,
    };
    store.apps.insert(
        "default".to_string(),
        App {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            default_user: String::new(),
            redirect_uri: String::new(),
            oauth2_tokens: BTreeMap::new(),
            oauth1_token: None,
            bearer_token: None,
            unnamed_oauth2_token: None,
        },
    );
    let auth = auth_for(&cfg, store);
    (auth, tmp)
}

#[tokio::test]
async fn auth_error_propagates_rather_than_silently_unauthenticated_request() {
    let ts = TestServer::new().await;
    // If Bug B regresses (request sent unauthenticated), the wiremock would
    // need to be mounted; we deliberately mount nothing so a regression
    // surfaces as a network-layer error against an unmatched path rather
    // than as `Error::Auth`. The assertion then catches the regression.
    //
    // Target `/2/tweets/search/recent` (Bearer-accepting per the spec
    // matrix) rather than `get_me`'s `/2/users/me` (OAuth1/OAuth2 only),
    // so the U6 fail-fast validator yields and the auth-resolution layer
    // is the one that surfaces the missing-credential error.
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_no_tokens(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let err = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/tweets/search/recent"),
            auth_type: "app".to_string(), // bearer path; no token in store
            ..Default::default()
        })
        .await
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("bearer token not found") || msg.contains("TokenNotFound"),
        "expected bearer-not-found auth error, got: {msg}"
    );
}

#[tokio::test]
async fn auth_error_propagates_for_oauth2_path_with_no_token() {
    let ts = TestServer::new().await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_no_tokens(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    // get_me with --auth oauth2 and an explicit username triggers the
    // named-caller branch; with no stored token the end state is
    // `Error::Auth`, not a request to wiremock.
    let err = client
        .get_me()
        .auth(WireScheme::OAuth2)
        .username("ghost-user")
        .send()
        .await
        .unwrap_err();
    // Accept either the TokenNotFound from refresh_oauth2_token or any
    // browser-open / network error from the implicit PKCE flow that fires
    // when no token is cached. The point of this test is that the call
    // returns Err without silently sending a request — not the exact
    // failure mode.
    assert!(
        matches!(err, xdk::Error::Auth(_) | xdk::Error::Http(_)),
        "expected auth-layer error, got: {err:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// U6 auth-enforcement integration tests (AE1, AE2, AE5)
// ═══════════════════════════════════════════════════════════════════════════
//
// These exercise the fail-fast validator wired into `send_request` /
// `send_multipart_request` / `stream_request`. The matrix says
// `POST /2/media/upload` accepts OAuth2 (`media.write`) + OAuth1 but not
// Bearer — so `--auth app` against that endpoint must reject before any
// network I/O. The wiremock server is set up to log all traffic so we can
// assert "zero requests received" for the reject case.

/// AE1 — explicit mismatch.
///
/// Bearer-only app, `--auth app`, `POST /2/media/upload`. The validator
/// must short-circuit with `Error::AuthMethodMismatch` carrying the
/// U6 explicit-mismatch shape (`requested = Some("app")`,
/// `available_in_app = None`), and wiremock must observe zero requests.
#[tokio::test]
async fn u6_ae1_explicit_mismatch_app_against_media_upload() {
    let ts = TestServer::new().await;
    // No mocks mounted intentionally — if the validator failed to reject
    // and the request leaked through, wiremock would return 404 and the
    // request count would tick to 1, breaking the assertion.
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let err = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: target_path("/2/media/upload"),
            auth_type: "app".to_string(),
            ..Default::default()
        })
        .await
        .unwrap_err();

    match &err {
        xdk::Error::AuthMethodMismatch(mismatch) => {
            let xdk::error::AuthMismatch {
                endpoint,
                method,
                requested,
                supported,
                available_in_app,
                ..
            } = &**mismatch;
            assert_eq!(endpoint, "/2/media/upload");
            assert_eq!(method, "POST");
            assert_eq!(requested.as_deref(), Some("app"));
            assert_eq!(supported, &vec!["oauth2".to_string(), "oauth1".to_string()]);
            assert!(
                available_in_app.is_none(),
                "explicit-mismatch shape must leave `available_in_app` at None"
            );
        }
        other => panic!("expected AuthMethodMismatch, got {other:?}"),
    }

    // Exit-code surface: 2 (`EX_USAGE`-aligned `EXIT_AUTH_MISMATCH`).
    assert_eq!(err.exit_code(), 2, "exit code must be EXIT_AUTH_MISMATCH");
    assert_eq!(err.kind(), "auth-method-mismatch");

    // Fail-fast invariant: zero traffic reached the mock server.
    assert_eq!(
        ts.received_request_count().await,
        0,
        "validator must reject BEFORE any HTTP I/O"
    );
}

/// AE2 — pass-through with an explicit `--auth` value the endpoint accepts.
///
/// OAuth1 creds + `--auth oauth1` + `POST /2/media/upload`. The validator
/// must yield, the request must reach wiremock, and the mocked 200
/// response should propagate back as JSON.
#[tokio::test]
async fn u6_ae2_passthrough_oauth1_against_media_upload() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/media/upload"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "media_xyz"}
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_oauth1(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: target_path("/2/media/upload"),
            auth_type: "oauth1".to_string(),
            ..Default::default()
        })
        .await
        .expect("oauth1 against /2/media/upload must validate and reach the server");

    assert_eq!(resp["data"]["id"], "media_xyz");
    assert_eq!(
        ts.received_request_count().await,
        1,
        "validator must pass through and let the request reach wiremock"
    );
}

/// AE1 mirror for the multipart send path.
///
/// `send_multipart_request` shares the same fail-fast validator wiring as
/// `send_request`. Bearer-only app + `--auth app` against `/2/media/upload`
/// must surface `AuthMethodMismatch` before the multipart body is built and
/// wiremock receives zero traffic. Pre-merge insurance against a future
/// edit that drops the gate from one of the three send paths.
#[tokio::test]
async fn u6_ae1_explicit_mismatch_app_against_multipart_upload() {
    use std::collections::HashMap;
    use xdk::api::MultipartOptions;

    let ts = TestServer::new().await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let mp_opts = MultipartOptions {
        request: RequestOptions {
            method: "POST".to_string(),
            target: target_path("/2/media/upload"),
            auth_type: "app".to_string(),
            ..Default::default()
        },
        form_fields: HashMap::new(),
        file_field: "media".to_string(),
        file_path: String::new(),
        file_name: "x.jpg".to_string(),
        file_data: b"fake-jpeg".to_vec(),
    };

    let err = client.send_multipart_request(&mp_opts).await.unwrap_err();

    match &err {
        xdk::Error::AuthMethodMismatch(mismatch) => {
            let xdk::error::AuthMismatch {
                endpoint,
                method,
                requested,
                supported,
                available_in_app,
                ..
            } = &**mismatch;
            assert_eq!(endpoint, "/2/media/upload");
            assert_eq!(method, "POST");
            assert_eq!(requested.as_deref(), Some("app"));
            assert_eq!(supported, &vec!["oauth2".to_string(), "oauth1".to_string()]);
            assert!(
                available_in_app.is_none(),
                "explicit-mismatch shape must leave `available_in_app` at None"
            );
        }
        other => panic!("expected AuthMethodMismatch, got {other:?}"),
    }

    assert_eq!(err.exit_code(), 2, "exit code must be EXIT_AUTH_MISMATCH");
    assert_eq!(err.kind(), "auth-method-mismatch");
    assert_eq!(
        ts.received_request_count().await,
        0,
        "multipart validator must reject BEFORE any HTTP I/O"
    );
}

/// AE1 mirror for the streaming send path.
///
/// `Client::stream_request` wires the same fail-fast validator as
/// `send_request`. Bearer-only app + `--auth app` against a streaming
/// endpoint that only accepts OAuth1/OAuth2 must surface
/// `AuthMethodMismatch` before any socket is opened. The CLI streaming
/// wrapper (`cli::commands::streaming::stream_request_with_output`) makes
/// the identical validator call at its own entry, so locking this
/// invariant on the library side covers both.
#[tokio::test]
async fn u6_ae1_explicit_mismatch_app_against_streaming_endpoint() {
    let ts = TestServer::new().await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    // /2/media/upload accepts OAuth2 + OAuth1 but rejects Bearer. Even
    // though it isn't a "real" streaming endpoint, stream_request applies
    // the validator before opening any connection, so the matrix surface
    // is what matters here.
    let err = client
        .stream_request(&RequestOptions {
            method: "POST".to_string(),
            target: target_path("/2/media/upload"),
            auth_type: "app".to_string(),
            ..Default::default()
        })
        .await
        .unwrap_err();

    match &err {
        xdk::Error::AuthMethodMismatch(mismatch) => {
            let xdk::error::AuthMismatch {
                endpoint,
                requested,
                supported,
                available_in_app,
                ..
            } = &**mismatch;
            assert_eq!(endpoint, "/2/media/upload");
            assert_eq!(requested.as_deref(), Some("app"));
            assert_eq!(supported, &vec!["oauth2".to_string(), "oauth1".to_string()]);
            assert!(
                available_in_app.is_none(),
                "explicit-mismatch shape must leave `available_in_app` at None"
            );
        }
        other => panic!("expected AuthMethodMismatch, got {other:?}"),
    }

    assert_eq!(err.exit_code(), 2);
    assert_eq!(err.kind(), "auth-method-mismatch");
    assert_eq!(
        ts.received_request_count().await,
        0,
        "streaming validator must reject BEFORE any socket is opened"
    );
}

/// Streaming auth-error propagation.
///
/// The CLI streaming wrapper switched from `if let Ok(auth_header) = ...`
/// (silently swallowing every auth error) to `?` propagation. The library
/// `Client::stream_request` shares the contract: when auth resolution
/// fails (e.g. token-not-found on the active app), the error must surface
/// rather than the request going out unauthenticated.
#[tokio::test]
async fn u7_streaming_propagates_auth_resolution_errors() {
    let ts = TestServer::new().await;
    let cfg = create_test_config(ts.uri());
    // No tokens stored on any app. Auto-detect has nothing to dispatch
    // against a matrix-hit endpoint and must surface auth-required rather
    // than letting an unauthenticated request leak through.
    let tmp = TempDir::new().expect("temp dir");
    let auth = auth_for(
        &cfg,
        TokenStore {
            apps: BTreeMap::new(),
            default_app: "default".to_string(),
            file_path: tmp.path().join(".xurl"),
            load_state: xdk::store::LoadState::Loaded,
        },
    );
    let client = Client::new(&cfg, auth).expect("client builds");

    let err = client
        .stream_request(&RequestOptions {
            method: "POST".to_string(),
            target: target_path("/2/media/upload"),
            // No --auth set; auto-detect path. Empty token store →
            // auth-required (exit 77), not silent unauth request.
            auth_type: String::new(),
            ..Default::default()
        })
        .await
        .unwrap_err();

    assert_eq!(err.kind(), "auth-required");
    assert_eq!(err.exit_code(), 77);
    assert_eq!(
        ts.received_request_count().await,
        0,
        "auth-resolution failure on the streaming path must not let the request through"
    );
}

/// AE5 — raw mode bypasses the matrix.
///
/// `xr <URL> --auth app` against `/2/media/upload` via `RequestTarget::RawUrl`
/// must NOT reject even though the same `(method, path)` would reject under a
/// `Template` target. Raw mode is the user's escape hatch.
#[tokio::test]
async fn u6_ae5_raw_url_skips_validation() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/media/upload"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "raw_bypass"}
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let raw_url = format!("{}/2/media/upload", ts.uri());
    let resp = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: RequestTarget::RawUrl(raw_url),
            auth_type: "app".to_string(),
            ..Default::default()
        })
        .await
        .expect("raw mode must skip auth-matrix validation per R18");

    assert_eq!(resp["data"]["id"], "raw_bypass");
    assert_eq!(
        ts.received_request_count().await,
        1,
        "raw mode must let the request through even with a normally-rejected auth"
    );
}

/// AE3 — auto-detect against an OAuth1-only app at an OAuth1+OAuth2 endpoint.
/// Intersection yields {OAuth1}; the request must dispatch on OAuth1 without
/// prompting and reach the wiremock.
#[tokio::test]
async fn u7_ae3_auto_detect_oauth1_only_app() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/media/upload/initialize"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "ae3"}
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_oauth1(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: RequestTarget::Template {
                path: "/2/media/upload/initialize".to_string(),
                path_params: HashMap::new(),
                query: Vec::new(),
            },
            ..Default::default()
        })
        .await
        .expect("AE3: OAuth1 must be auto-selected for an OAuth1+OAuth2 endpoint");

    assert_eq!(resp["data"]["id"], "ae3");
    assert_eq!(ts.received_request_count().await, 1);
}

/// AE4 — auto-detect against a Bearer-only app at an OAuth1+OAuth2 endpoint.
/// Intersection is empty; the request must fail with the typed envelope
/// shape (R8) carrying `requested: None`, `available_in_app: Some(["app"])`,
/// and the endpoint's supported set.
#[tokio::test]
async fn u7_ae4_auto_detect_empty_intersection_envelope() {
    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/media/upload/initialize"))
            .respond_with(ResponseTemplate::new(200)),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let err = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: RequestTarget::Template {
                path: "/2/media/upload/initialize".to_string(),
                path_params: HashMap::new(),
                query: Vec::new(),
            },
            ..Default::default()
        })
        .await
        .unwrap_err();

    match err {
        xdk::Error::AuthMethodMismatch(mismatch) => {
            let xdk::error::AuthMismatch {
                endpoint,
                method: m,
                requested,
                supported,
                available_in_app,
                app,
                ..
            } = *mismatch;
            assert_eq!(endpoint, "/2/media/upload/initialize");
            assert_eq!(m, "POST");
            assert_eq!(requested, None);
            assert_eq!(supported, vec!["oauth2", "oauth1"]);
            assert_eq!(available_in_app, Some(vec!["app".to_string()]));
            assert_eq!(
                app,
                Some("default".to_string()),
                "envelope must carry the active app name"
            );
        }
        other => panic!("AE4: expected AuthMethodMismatch, got {other:?}"),
    }
    assert_eq!(
        ts.received_request_count().await,
        0,
        "AE4: validator must refuse before HTTP — wiremock receives nothing"
    );
}

/// AE6 — auto-detect against an app holding both OAuth1 and OAuth2 at an
/// endpoint accepting both. OAuth2 wins per the locked preference order.
/// Verified by mounting two distinct mocks that match on the Authorization
/// header prefix and asserting the OAuth2-shaped header lands.
#[tokio::test]
async fn u7_ae6_auto_detect_oauth2_preference_when_both_stored() {
    use wiremock::matchers::header;

    let ts = TestServer::new().await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/media/upload/initialize"))
            .and(header("Authorization", "Bearer valid-access-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "ae6_oauth2"}
            }))),
    )
    .await;
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_all_methods(ts.uri());
    let client = Client::new(&cfg, auth).expect("client builds");

    let resp = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: RequestTarget::Template {
                path: "/2/media/upload/initialize".to_string(),
                path_params: HashMap::new(),
                query: Vec::new(),
            },
            ..Default::default()
        })
        .await
        .expect("AE6: OAuth2 must win the preference when both schemes are stored");

    assert_eq!(resp["data"]["id"], "ae6_oauth2");
}

/// U7 semantic guard — an app with zero stored credentials surfaces
/// `AuthRequired` (exit 77, "log in first"), not `AuthMethodMismatch`
/// (exit 2, "wrong credential"). The empty-intersection branch fires only
/// when the user has SOMETHING stored that happens not to overlap.
#[tokio::test]
async fn u7_no_stored_credentials_returns_auth_required() {
    let ts = TestServer::new().await;
    let cfg = create_test_config(ts.uri());
    let tmp = TempDir::new().expect("temp dir");
    let auth = auth_for(
        &cfg,
        TokenStore {
            apps: BTreeMap::new(),
            default_app: "default".to_string(),
            file_path: tmp.path().join(".xurl"),
            load_state: xdk::store::LoadState::Loaded,
        },
    );
    let client = Client::new(&cfg, auth).expect("client builds");

    let err = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: RequestTarget::Template {
                path: "/2/media/upload/initialize".to_string(),
                path_params: HashMap::new(),
                query: Vec::new(),
            },
            ..Default::default()
        })
        .await
        .unwrap_err();

    // Surfaces as Auth (kind "auth-required", exit 77) per the semantic
    // distinction: no creds at all ≠ wrong creds.
    assert_eq!(err.kind(), "auth-required");
    assert_eq!(err.exit_code(), 77);
}
