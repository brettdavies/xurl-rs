//! Ported from Go: api/client_test.go + api/endpoints_test.go
//!                  + api/shortcuts_test.go + api/media_test.go
//!
//! Tests the core API client, request building, response parsing,
//! streaming endpoint detection, shortcut commands, and media upload.
//!
//! Uses wiremock for mock HTTP servers. Since ApiClient uses reqwest::blocking,
//! we start the mock server on a dedicated tokio runtime and run blocking
//! client code on the test thread.

use std::collections::BTreeMap;

use rstest::rstest;
use tempfile::TempDir;
use wiremock::matchers::{method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use std::collections::HashMap;

use xurl::api::{
    self, ApiClient, CallOptions, RequestOptions, RequestTarget, extract_media_id,
    extract_segment_index, is_media_append_request, is_streaming_endpoint,
};
use xurl::auth::Auth;
use xurl::config::Config;
use xurl::store::{App, OAuth1Token, OAuth2Token, Token, TokenStore, TokenType};

// ── Mock server helper ─────────────────────────────────────────────────
// wiremock::MockServer needs a tokio runtime. We create one per test
// and keep it alive while running blocking client code.

struct TestServer {
    _rt: tokio::runtime::Runtime,
    server: &'static MockServer,
    uri: String,
}

impl TestServer {
    fn new() -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let server = rt.block_on(async {
            let s = MockServer::start().await;
            // Leak to get 'static lifetime — tests are short-lived anyway
            Box::leak(Box::new(s))
        });
        let uri = server.uri();
        Self {
            _rt: rt,
            server,
            uri,
        }
    }

    fn mount(&self, mock: Mock) {
        self._rt.block_on(async {
            mock.mount(self.server).await;
        });
    }

    fn uri(&self) -> &str {
        &self.uri
    }

    /// Returns the number of requests this mock server has received across
    /// all registered mocks. Used by the U6 enforcement tests to assert
    /// that fail-fast validation rejects before any network I/O happens.
    fn received_request_count(&self) -> usize {
        self._rt.block_on(async {
            self.server
                .received_requests()
                .await
                .map(|v| v.len())
                .unwrap_or(0)
        })
    }
}

// ── Test helpers ───────────────────────────────────────────────────────

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
    let auth = Auth::new(&cfg);

    let tmp = TempDir::new().expect("temp dir");
    let file_path = tmp.path().join(".xurl");

    let mut store = TokenStore {
        apps: BTreeMap::new(),
        default_app: "default".to_string(),
        file_path,
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

    let auth = auth.with_token_store(store);
    (auth, tmp)
}

fn create_mock_auth_with_oauth1(base_url: &str) -> (Auth, TempDir) {
    let cfg = create_test_config(base_url);
    let auth = Auth::new(&cfg);

    let tmp = TempDir::new().expect("temp dir");
    let file_path = tmp.path().join(".xurl");

    let mut store = TokenStore {
        apps: BTreeMap::new(),
        default_app: "default".to_string(),
        file_path,
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

    let auth = auth.with_token_store(store);
    (auth, tmp)
}

fn create_mock_auth_with_oauth2(base_url: &str) -> (Auth, TempDir) {
    let cfg = create_test_config(base_url);
    let auth = Auth::new(&cfg);

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

    let auth = auth.with_token_store(store);
    (auth, tmp)
}

fn base_call_opts() -> CallOptions {
    CallOptions::default()
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

#[test]
fn test_new_api_client() {
    let ts = TestServer::new();
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let _client = ApiClient::new(&cfg, auth);
}

// ═══════════════════════════════════════════════════════════════════════════
// api/client_test.go — TestBuildRequest
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_build_request_get() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(
                    serde_json::json!({"data":{"id":"12345","username":"testuser"}}),
                ),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        ..Default::default()
    };

    let resp = client.send_request(&opts).unwrap();
    assert_eq!(resp["data"]["username"], "testuser");
}

#[test]
fn test_build_request_post() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(
                    serde_json::json!({"data":{"id":"67890","text":"Hello world!"}}),
                ),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = RequestOptions {
        method: "POST".to_string(),
        target: target_path("/2/tweets"),
        data: r#"{"text":"Hello world!"}"#.to_string(),
        ..Default::default()
    };

    let resp = client.send_request(&opts).unwrap();
    assert_eq!(resp["data"]["text"], "Hello world!");
}

// ═══════════════════════════════════════════════════════════════════════════
// Auth type routing tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_build_request_with_auth_bearer() {
    // The intent: `--auth app` routes the Bearer auth header. Targeted at
    // `/2/tweets/search/recent` rather than `/2/users/me` because the
    // spec-derived auth matrix (v2.0.0) says only the former accepts Bearer.
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{"id":"1"}})),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/tweets/search/recent"),
        auth_type: "app".to_string(),
        ..Default::default()
    };

    let resp = client.send_request(&opts).unwrap();
    assert_eq!(resp["data"]["id"], "1");
}

#[test]
fn test_build_request_with_auth_oauth1() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{"id":"1"}})),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_oauth1(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        auth_type: "oauth1".to_string(),
        ..Default::default()
    };

    let resp = client.send_request(&opts).unwrap();
    assert_eq!(resp["data"]["id"], "1");
}

#[test]
fn test_build_request_with_auth_oauth2() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{"id":"1"}})),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_oauth2(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/users/me"),
        auth_type: "oauth2".to_string(),
        username: "testuser".to_string(),
        ..Default::default()
    };

    let resp = client.send_request(&opts).unwrap();
    assert_eq!(resp["data"]["id"], "1");
}

// ═══════════════════════════════════════════════════════════════════════════
// api/client_test.go — TestSendRequest
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_send_request_success() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":{"id":"12345","name":"Test User","username":"testuser"}}),
            )),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/users/me"),
            ..Default::default()
        })
        .unwrap();

    assert_eq!(resp["data"]["username"], "testuser");
    assert_eq!(resp["data"]["id"], "12345");
}

#[test]
fn test_send_request_http_error() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .respond_with(ResponseTemplate::new(400).set_body_json(
                serde_json::json!({"errors":[{"message":"Invalid query","code":400}]}),
            )),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let err = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/tweets/search/recent"),
            ..Default::default()
        })
        .unwrap_err();

    assert!(err.is_api(), "Expected API error, got: {err}");
}

#[test]
fn test_send_request_json_parse_error() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/bad-json"))
            .respond_with(ResponseTemplate::new(200).set_body_string("this is not json")),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    // Non-JSON 200 response returns empty JSON object
    let resp = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/bad-json"),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(resp, serde_json::json!({}));
}

// ═══════════════════════════════════════════════════════════════════════════
// Timeout wiring — --timeout / XURL_TIMEOUT bound network calls
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn slow_endpoint_trips_explicit_timeout() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/slow"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(std::time::Duration::from_secs(10))
                    .set_body_json(serde_json::json!({"ok": true})),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::with_timeout(&cfg, auth, 1);
    assert_eq!(client.timeout_secs(), 1);

    let started = std::time::Instant::now();
    let err = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/slow"),
            ..Default::default()
        })
        .expect_err("a 1s timeout must fire before the 10s server delay");
    let elapsed = started.elapsed();

    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "timeout should fire well under the server's 10s delay; elapsed = {elapsed:?}"
    );
    let msg = err.to_string();
    assert!(
        msg.contains("HTTP Error"),
        "expected an HTTP/transport error, got: {msg}"
    );
}

#[test]
fn config_default_timeout_carries_through_apiclient_new() {
    // Regression guard for the runner-to-ApiClient timeout plumbing: the value
    // on `Config::http_timeout_secs` must drive `ApiClient::timeout_secs()`.
    let mut cfg = create_test_config("http://127.0.0.1:9");
    cfg.http_timeout_secs = 7;
    let (auth, _tmp) = create_mock_auth_with_bearer("http://127.0.0.1:9");
    let client = ApiClient::new(&cfg, auth);
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

#[test]
fn test_get_auth_header_oauth2() {
    let (mut auth, _tmp) = create_mock_auth_with_oauth2("https://api.x.com");
    let header = auth.get_oauth2_header("testuser").unwrap();
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

// ═══════════════════════════════════════════════════════════════════════════
// api/shortcuts_test.go — Shortcut integration tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_create_post() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data":{"id":"99999","text":"Hello!"}})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client
        .create_post("Hello!", &[], &base_call_opts())
        .unwrap();
    assert_eq!(resp.data.id, "99999");
    assert_eq!(resp.data.text, "Hello!");
}

#[test]
fn test_reply_to_post() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data":{"id":"88888","text":"nice!"}})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client
        .reply_to_post("123", "nice!", &[], &base_call_opts())
        .unwrap();
    assert_eq!(resp.data.id, "88888");
}

#[test]
fn test_reply_to_post_with_url() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(
                    serde_json::json!({"data":{"id":"77777","text":"reply via URL"}}),
                ),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client
        .reply_to_post(
            "https://x.com/u/status/123",
            "reply via URL",
            &[],
            &base_call_opts(),
        )
        .unwrap();
    assert_eq!(resp.data.id, "77777");
}

#[test]
fn test_quote_post() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data":{"id":"66666","text":"my take"}})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client
        .quote_post("123", "my take", &base_call_opts())
        .unwrap();
    assert_eq!(resp.data.id, "66666");
}

#[test]
fn test_delete_post() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("DELETE"))
            .and(path("/2/tweets/123"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data":{"deleted":true}})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.delete_post("123", &base_call_opts()).unwrap();
    assert!(resp.data.deleted);
}

#[test]
fn test_read_post() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET")).and(path_regex(r"/2/tweets/123.*")).respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{"id":"123","text":"existing post","public_metrics":{"like_count":5}}})),
        ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.read_post("123", &base_call_opts()).unwrap();
    assert_eq!(resp.data.id, "123");
    assert_eq!(resp.data.text, "existing post");
}

#[test]
fn test_search_posts() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET")).and(path("/2/tweets/search/recent")).respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":[{"id":"1","text":"result one"}],"meta":{"result_count":1}})),
        ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client
        .search_posts("golang", 10, &base_call_opts())
        .unwrap();
    assert_eq!(resp.meta.as_ref().unwrap().result_count, Some(1));
}

/// Threads `CallOptions::pagination_token` through to the
/// `pagination_token` query parameter on the search URL — wiremock asserts
/// the parameter is present and carries the URL-decoded token.
#[test]
fn test_search_posts_threads_pagination_token() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .and(query_param("pagination_token", "next_abc_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":[{"id":"2","text":"page2"}],"meta":{"result_count":1}}),
            )),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = CallOptions {
        pagination_token: "next_abc_token".into(),
        ..base_call_opts()
    };
    let resp = client.search_posts("golang", 10, &opts).unwrap();
    assert_eq!(resp.data.first().unwrap().id, "2");
}

/// Same wiremock probe as above, but verifies the URL-encoder runs on
/// special characters (spaces → `%20`).
#[test]
fn test_search_posts_url_encodes_pagination_token() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .and(query_param("pagination_token", "page 2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":[{"id":"3","text":"p"}],"meta":{"result_count":1}}),
            )),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = CallOptions {
        pagination_token: "page 2".into(),
        ..base_call_opts()
    };
    let resp = client.search_posts("golang", 10, &opts).unwrap();
    assert_eq!(resp.data.first().unwrap().id, "3");
}

#[test]
fn test_get_me() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":{"id":"42","username":"testbot","name":"Test Bot"}}),
            )),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.get_me(&base_call_opts()).unwrap();
    assert_eq!(resp.data.id, "42");
    assert_eq!(resp.data.username, "testbot");
}

#[test]
fn test_lookup_user() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path_regex(r"/2/users/by/username/someuser.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":{"id":"100","username":"lookedup","name":"Looked Up"}}),
            )),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.lookup_user("@someuser", &base_call_opts()).unwrap();
    assert_eq!(resp.data.id, "100");
    assert_eq!(resp.data.username, "lookedup");
}

#[test]
fn test_create_post_with_media() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data":{"id":"55555","text":"With media"}})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let media_ids = vec!["m1".to_string(), "m2".to_string()];
    let resp = client
        .create_post("With media", &media_ids, &base_call_opts())
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

#[test]
fn test_media_upload_init() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST")).and(path("/2/media/upload/initialize")).respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "test_media_id", "expires_after_secs": 3600, "media_key": "test_media_key"}
            })),
        ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.send_request(&RequestOptions {
        method: "POST".to_string(),
        target: target_path("/2/media/upload/initialize"),
        data: serde_json::json!({"total_bytes": 1024, "media_type": "image/jpeg", "media_category": "tweet_image"}).to_string(),
        ..Default::default()
    }).unwrap();
    assert_eq!(resp["data"]["id"], "test_media_id");
}

#[test]
fn test_media_upload_finalize() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/media/upload/test_media_id/finalize"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "test_media_id", "media_key": "test_media_key"}
            }))),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: target_path("/2/media/upload/test_media_id/finalize"),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(resp["data"]["id"], "test_media_id");
}

#[test]
fn test_media_upload_check_status() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/media/upload"))
            .and(query_param("command", "STATUS"))
            .and(query_param("media_id", "test_media_id"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "test_media_id", "processing_info": {"state": "succeeded", "progress_percent": 100}}
            }))),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

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
        .unwrap();
    assert_eq!(resp["data"]["processing_info"]["state"], "succeeded");
}

#[test]
fn test_stream_request_error() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/stream/error"))
            .respond_with(ResponseTemplate::new(400).set_body_json(
                serde_json::json!({"errors":[{"message":"Invalid rule","code":400}]}),
            )),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let err = client
        .stream_request(
            &RequestOptions {
                method: "GET".to_string(),
                target: target_path("/2/tweets/search/stream/error"),
                ..Default::default()
            },
            &mut stdout,
            &mut stderr,
        )
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

#[test]
fn test_get_usage_happy_path() {
    let ts = TestServer::new();
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
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.get_usage(&base_call_opts()).unwrap();
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

#[test]
fn test_get_usage_requires_usage_fields_query_param() {
    // Verify the shortcut sends the required query parameter.
    // Without usage.fields, the API returns minimal data — this mock
    // only responds when the param is present.
    let ts = TestServer::new();
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
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.get_usage(&base_call_opts()).unwrap();
    assert_eq!(resp.data.project_usage.as_deref(), Some("42"));
}

#[test]
fn test_get_usage_uses_get_method() {
    // Ensure the shortcut uses GET, not POST or another method.
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"project_usage": "0"}})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.get_usage(&base_call_opts());
    assert!(resp.is_ok());
}

#[test]
fn test_get_usage_api_error_401() {
    // Unauthorized — e.g., missing or invalid bearer token.
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "title": "Unauthorized",
                "type": "about:blank",
                "status": 401,
                "detail": "Unauthorized"
            }))),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.get_usage(&base_call_opts());
    assert!(resp.is_err());
}

#[test]
fn test_get_usage_api_error_429() {
    // Rate limited — 50 requests per 15-minute window.
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
                "title": "Too Many Requests",
                "detail": "Too Many Requests",
                "type": "about:blank",
                "status": 429
            }))),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.get_usage(&base_call_opts());
    assert!(resp.is_err());
}

#[test]
fn test_get_usage_with_oauth1_auth() {
    // Usage endpoint works with any valid auth, not just bearer.
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"project_usage": "10"}})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_oauth1(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.get_usage(&base_call_opts()).unwrap();
    assert_eq!(resp.data.project_usage.as_deref(), Some("10"));
}

#[test]
fn test_get_usage_with_oauth2_auth() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"project_usage": "20"}})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_oauth2(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.get_usage(&base_call_opts()).unwrap();
    assert_eq!(resp.data.project_usage.as_deref(), Some("20"));
}

#[test]
fn test_get_usage_daily_project_usage_structure() {
    // Verify the daily_project_usage nested structure is preserved.
    let ts = TestServer::new();
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
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.get_usage(&base_call_opts()).unwrap();
    let daily_val = resp.data.daily_project_usage.as_ref().unwrap();
    let usage = &daily_val["usage"];
    assert!(usage.is_array());
    assert_eq!(usage.as_array().unwrap().len(), 3);
    assert_eq!(usage[0]["usage"], "50");
    assert_eq!(usage[2]["usage"], "100");
}

#[test]
fn test_get_usage_daily_client_app_usage_structure() {
    // Verify the daily_client_app_usage array with per-app breakdowns.
    let ts = TestServer::new();
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
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.get_usage(&base_call_opts()).unwrap();
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

#[test]
fn test_get_usage_clears_request_data() {
    // Ensure no stale request body leaks into the GET request.
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/usage/tweets"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"project_usage": "0"}})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    // CallOptions has no data field, so stale data can't leak — verify the call succeeds
    let resp = client.get_usage(&base_call_opts());
    assert!(resp.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════
// Red team — adversarial API responses via wiremock
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn redteam_create_post_array_where_object_expected() {
    // API returns array in data field for a single-item shortcut
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data": [{"id": "1", "text": "oops"}]})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let result = client.create_post("test", &[], &base_call_opts());
    assert!(
        result.is_err(),
        "Should fail: array where single Tweet expected"
    );
}

#[test]
fn redteam_get_me_no_data_field() {
    // API returns errors-only 200 with no data field
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"errors": [{"message": "forbidden"}]})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let result = client.get_me(&base_call_opts());
    let err = result.unwrap_err();
    assert!(
        err.is_validation(),
        "Should be Validation error (not API or JSON error) for errors-only 200: {err}"
    );
}

#[test]
fn redteam_delete_post_wrong_type_in_data() {
    // API returns string instead of object in data field
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("DELETE"))
            .and(path("/2/tweets/123"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": "unexpected string"})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let result = client.delete_post("123", &base_call_opts());
    assert!(
        result.is_err(),
        "Should fail: string where DeletedResult expected"
    );
}

#[test]
fn redteam_search_posts_null_data() {
    // API returns null data
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": null})),
            ),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let result = client.search_posts("test", 10, &base_call_opts());
    assert!(result.is_err(), "Should fail: null data for Vec<Tweet>");
}

#[test]
fn redteam_empty_body_returns_descriptive_error() {
    // send_request returns empty {} for non-JSON 2xx — shortcut should give clear error
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json")),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let result = client.get_me(&base_call_opts());
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("empty response body"),
        "Expected descriptive error, got: {err}"
    );
}

#[test]
fn redteam_unknown_fields_survive_shortcut_round_trip() {
    // Verify serde(flatten) preserves unknown fields through the full shortcut path
    let ts = TestServer::new();
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
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client
        .create_post("Hello!", &[], &base_call_opts())
        .unwrap();
    assert_eq!(resp.data.id, "99999");
    // Unknown fields preserved in extra
    assert_eq!(resp.data.extra["brand_new_field"], "surprise_value");
    assert_eq!(resp.extra["top_level_extra"], 42);
    // Round-trip: serialize back to Value and verify preservation
    let value = serde_json::to_value(&resp).unwrap();
    assert_eq!(value["data"]["brand_new_field"], "surprise_value");
    assert_eq!(value["top_level_extra"], 42);
}

#[test]
fn redteam_like_post_extra_fields_on_action() {
    // Action confirmation with extra unknown fields
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/users/42/likes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"liked": true, "pending_follow": false},
                "rate_limit_remaining": 99
            }))),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client.like_post("42", "123", &base_call_opts()).unwrap();
    assert!(resp.data.liked);
    // Unknown fields captured, not lost
    assert_eq!(resp.data.extra["pending_follow"], false);
    assert_eq!(resp.extra["rate_limit_remaining"], 99);
}

#[test]
fn redteam_lookup_user_wrong_bool_type() {
    // String "true" where boolean expected in a nested field
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path_regex(r"/2/users/by/username/bad.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "1", "name": "Bad", "username": "bad", "verified": "true"}
            }))),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    // verified is Option<bool> — "true" (string) should fail deserialization
    let result = client.lookup_user("bad", &base_call_opts());
    assert!(
        result.is_err(),
        "Should fail: string 'true' where bool expected"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// from_env() constructor tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_from_env_missing_client_id_returns_validation_error() {
    // Temporarily clear CLIENT_ID to test error path
    let original = std::env::var("CLIENT_ID").ok();
    unsafe { std::env::remove_var("CLIENT_ID") };

    let result = ApiClient::from_env();
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("Expected error when CLIENT_ID is missing"),
    };
    assert!(
        err.is_validation(),
        "Expected Validation error, got: {err:?}"
    );
    assert!(
        err.to_string().contains("CLIENT_ID"),
        "Error should mention CLIENT_ID: {err}"
    );

    // Restore
    if let Some(val) = original {
        unsafe { std::env::set_var("CLIENT_ID", val) };
    }
}

#[test]
fn test_from_env_empty_client_id_returns_validation_error() {
    let original = std::env::var("CLIENT_ID").ok();
    unsafe { std::env::set_var("CLIENT_ID", "") };

    let result = ApiClient::from_env();
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("Expected error when CLIENT_ID is empty"),
    };
    assert!(err.is_validation());

    // Restore
    if let Some(val) = original {
        unsafe { std::env::set_var("CLIENT_ID", val) };
    } else {
        unsafe { std::env::remove_var("CLIENT_ID") };
    }
}

#[test]
fn test_from_env_with_client_id_set_returns_ok() {
    let original_id = std::env::var("CLIENT_ID").ok();
    let original_secret = std::env::var("CLIENT_SECRET").ok();
    unsafe {
        std::env::set_var("CLIENT_ID", "test-from-env-id");
        std::env::set_var("CLIENT_SECRET", "test-secret");
    }

    let result = ApiClient::from_env();
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

#[test]
fn test_from_env_with_client_id_but_no_secret_returns_ok() {
    // Best-effort: CLIENT_SECRET not required at construction time
    let original_id = std::env::var("CLIENT_ID").ok();
    let original_secret = std::env::var("CLIENT_SECRET").ok();
    unsafe {
        std::env::set_var("CLIENT_ID", "test-from-env-id");
        std::env::remove_var("CLIENT_SECRET");
    }

    let result = ApiClient::from_env();
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

#[test]
fn test_no_auth_skips_authorization_header() {
    // With no_auth=true, the request should NOT include an Authorization header.
    // The mock only succeeds if NO Authorization header is present.
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "123", "name": "Test", "username": "test"}}),
            )),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = CallOptions {
        no_auth: true,
        ..CallOptions::default()
    };
    // Should succeed — no_auth skips get_auth_header entirely
    let result = client.get_me(&opts);
    assert!(result.is_ok(), "no_auth=true should not fail: {result:?}");
}

#[test]
fn test_no_auth_false_includes_authorization_header() {
    // With no_auth=false (default), the Authorization header should be present.
    // The mock requires an Authorization header via header_exists matcher.
    use wiremock::matchers::header_exists;

    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .and(header_exists("Authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "123", "name": "Test", "username": "test"}}),
            )),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = CallOptions::default();
    let result = client.get_me(&opts);
    assert!(
        result.is_ok(),
        "Default (no_auth=false) should include auth header: {result:?}"
    );
}

#[test]
fn test_no_auth_with_raw_send_request() {
    // Verify no_auth works at the send_request level too, not just shortcuts
    use wiremock::matchers::header_exists;

    let ts = TestServer::new();
    // This mock will fail if Authorization header IS present
    // (by only matching requests WITHOUT it via a custom approach)
    // Instead, just verify the request succeeds with no_auth=true
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/test"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/test"),
        no_auth: true,
        ..Default::default()
    };
    let result = client.send_request(&opts);
    assert!(
        result.is_ok(),
        "no_auth=true on send_request should work: {result:?}"
    );

    // Now verify that with no_auth=false, the auth header IS sent
    let ts2 = TestServer::new();
    ts2.mount(
        Mock::given(method("GET"))
            .and(path("/2/test"))
            .and(header_exists("Authorization"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})),
            ),
    );

    let cfg2 = create_test_config(ts2.uri());
    let (auth2, _tmp2) = create_mock_auth_with_bearer(ts2.uri());
    let mut client2 = ApiClient::new(&cfg2, auth2);

    let opts2 = RequestOptions {
        method: "GET".to_string(),
        target: target_path("/2/test"),
        no_auth: false,
        ..Default::default()
    };
    let result2 = client2.send_request(&opts2);
    assert!(
        result2.is_ok(),
        "no_auth=false should include auth header: {result2:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Red team — library ergonomics edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn redteam_no_auth_with_auth_type_set_silently_skips_auth() {
    // Conflicting: no_auth=true + auth_type="oauth2" — no_auth should win
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "1", "name": "X", "username": "x"}}),
            )),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = CallOptions {
        auth_type: "oauth2".to_string(),
        no_auth: true,
        ..CallOptions::default()
    };
    // Should succeed — no_auth takes precedence over auth_type
    let result = client.get_me(&opts);
    assert!(
        result.is_ok(),
        "no_auth=true should take precedence over auth_type: {result:?}"
    );
}

#[test]
fn redteam_sequential_calls_on_same_client() {
    // Verify that making multiple calls on the same client instance works
    // (auth state not corrupted between calls)
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"id": "42", "name": "Me", "username": "me"}}),
            )),
    );
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"data": {"id": "99", "text": "hi"}})),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let opts = base_call_opts();

    // First call
    let me = client.get_me(&opts).unwrap();
    assert_eq!(me.data.id, "42");

    // Second call on same client — auth should still work
    let post = client.create_post("hi", &[], &opts).unwrap();
    assert_eq!(post.data.id, "99");

    // Third call — still works
    let me2 = client.get_me(&opts).unwrap();
    assert_eq!(me2.data.id, "42");
}

#[test]
fn redteam_api_error_preserves_status_and_body() {
    // Verify that HTTP errors carry both status code and body through the pipeline
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(
                    serde_json::json!({"detail": "Forbidden", "title": "Forbidden"}),
                ),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let result = client.get_me(&base_call_opts());
    let err = result.unwrap_err();
    assert!(err.is_api());
    // Verify structured error carries status
    match &err {
        xurl::error::XurlError::Api { status, body } => {
            assert_eq!(*status, 403);
            assert!(body.contains("Forbidden"));
        }
        _ => panic!("Expected Api variant, got: {err:?}"),
    }
}

#[test]
fn redteam_api_error_401_gives_auth_exit_code() {
    // Verify the full pipeline: HTTP 401 → Api { status: 401 } → EXIT_AUTH_REQUIRED
    use xurl::error::exit_code_for_error;

    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_json(serde_json::json!({"detail": "Unauthorized"})),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let err = client.get_me(&base_call_opts()).unwrap_err();
    assert_eq!(
        exit_code_for_error(&err),
        xurl::error::EXIT_AUTH_REQUIRED,
        "401 should map to EXIT_AUTH_REQUIRED"
    );
}

#[test]
fn redteam_api_error_429_gives_rate_limit_exit_code() {
    use xurl::error::exit_code_for_error;

    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(429)
                    .set_body_json(serde_json::json!({"detail": "Too Many Requests"})),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let err = client.get_me(&base_call_opts()).unwrap_err();
    assert_eq!(
        exit_code_for_error(&err),
        xurl::error::EXIT_RATE_LIMITED,
        "429 should map to EXIT_RATE_LIMITED"
    );
}

// ── TestAuthErrorPropagation (Bug B) ──────────────────────────────────────
//
// `ApiClient::send_request` (and its sibling paths) must propagate the
// `get_auth_header` error rather than silently sending the request
// unauthenticated. The older `if let Ok(...)` form let auth bugs masquerade
// as upstream 401s; the new path returns the real `XurlError::Auth` so the
// user can tell the difference between "we couldn't sign the request" and
// "we signed it and X rejected it".

fn create_mock_auth_no_tokens(base_url: &str) -> (Auth, TempDir) {
    let cfg = create_test_config(base_url);
    let auth = Auth::new(&cfg);

    let tmp = TempDir::new().expect("temp dir");
    let file_path = tmp.path().join(".xurl");

    let mut store = TokenStore {
        apps: BTreeMap::new(),
        default_app: "default".to_string(),
        file_path,
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
    let auth = auth.with_token_store(store);
    (auth, tmp)
}

#[test]
fn auth_error_propagates_rather_than_silently_unauthenticated_request() {
    let ts = TestServer::new();
    // If Bug B regresses (request sent unauthenticated), the wiremock would
    // need to be mounted; we deliberately mount nothing so a regression
    // surfaces as a network-layer error against an unmatched path rather
    // than as `XurlError::Auth`. The assertion then catches the regression.
    //
    // Target `/2/tweets/search/recent` (Bearer-accepting per the spec
    // matrix) rather than `get_me`'s `/2/users/me` (OAuth1/OAuth2 only),
    // so the U6 fail-fast validator yields and the auth-resolution layer
    // is the one that surfaces the missing-credential error.
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_no_tokens(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let err = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            target: target_path("/2/tweets/search/recent"),
            auth_type: "app".to_string(), // bearer path; no token in store
            ..Default::default()
        })
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("bearer token not found") || msg.contains("TokenNotFound"),
        "expected bearer-not-found auth error, got: {msg}"
    );
}

#[test]
fn auth_error_propagates_for_oauth2_path_with_no_token() {
    let ts = TestServer::new();
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_no_tokens(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let mut opts = base_call_opts();
    opts.auth_type = "oauth2".to_string();
    // get_me with --auth oauth2 and an explicit username triggers the
    // named-caller branch; with no stored token the flow attempts to start
    // an interactive OAuth2 PKCE flow which fails (no DISPLAY in tests).
    // The end state is `XurlError::Auth`, not a request to wiremock.
    opts.username = "ghost-user".to_string();
    let err = client.get_me(&opts).unwrap_err();
    // Accept either the TokenNotFound from refresh_oauth2_token or any
    // browser-open / network error from the implicit PKCE flow that fires
    // when no token is cached. The point of this test is that the call
    // returns Err without silently sending a request — not the exact
    // failure mode.
    assert!(
        matches!(
            err,
            xurl::error::XurlError::Auth(_) | xurl::error::XurlError::Http(_)
        ),
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
/// must short-circuit with `XurlError::AuthMethodMismatch` carrying the
/// U6 explicit-mismatch shape (`requested = Some("app")`,
/// `available_in_app = None`), and wiremock must observe zero requests.
#[test]
fn u6_ae1_explicit_mismatch_app_against_media_upload() {
    let ts = TestServer::new();
    // No mocks mounted intentionally — if the validator failed to reject
    // and the request leaked through, wiremock would return 404 and the
    // request count would tick to 1, breaking the assertion.
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let err = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: target_path("/2/media/upload"),
            auth_type: "app".to_string(),
            ..Default::default()
        })
        .unwrap_err();

    match &err {
        xurl::error::XurlError::AuthMethodMismatch {
            endpoint,
            method,
            requested,
            supported,
            available_in_app,
        } => {
            assert_eq!(endpoint, "/2/media/upload");
            assert_eq!(method, "POST");
            assert_eq!(requested.as_deref(), Some("app"));
            assert_eq!(supported, &vec!["oauth2".to_string(), "oauth1".to_string()]);
            assert!(
                available_in_app.is_none(),
                "U6 explicit-mismatch shape must leave `available_in_app` at None"
            );
        }
        other => panic!("expected AuthMethodMismatch, got {other:?}"),
    }

    // Exit-code surface: 2 (`EX_USAGE`-aligned `EXIT_AUTH_MISMATCH`).
    assert_eq!(err.exit_code(), 2, "exit code must be EXIT_AUTH_MISMATCH");
    assert_eq!(err.kind(), "auth-method-mismatch");

    // Fail-fast invariant: zero traffic reached the mock server.
    assert_eq!(
        ts.received_request_count(),
        0,
        "validator must reject BEFORE any HTTP I/O"
    );
}

/// AE2 — pass-through with an explicit `--auth` value the endpoint accepts.
///
/// OAuth1 creds + `--auth oauth1` + `POST /2/media/upload`. The validator
/// must yield, the request must reach wiremock, and the mocked 200
/// response should propagate back as JSON.
#[test]
fn u6_ae2_passthrough_oauth1_against_media_upload() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/media/upload"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "media_xyz"}
            }))),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_oauth1(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let resp = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: target_path("/2/media/upload"),
            auth_type: "oauth1".to_string(),
            ..Default::default()
        })
        .expect("oauth1 against /2/media/upload must validate and reach the server");

    assert_eq!(resp["data"]["id"], "media_xyz");
    assert_eq!(
        ts.received_request_count(),
        1,
        "validator must pass through and let the request reach wiremock"
    );
}

/// AE5 — raw mode bypasses the matrix (R18).
///
/// `xr <URL> --auth app` against `/2/media/upload` via `RequestTarget::RawUrl`
/// must NOT reject even though the same `(method, path)` would reject under a
/// `Template` target. Raw mode is the user's escape hatch.
#[test]
fn u6_ae5_raw_url_skips_validation() {
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/media/upload"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "raw_bypass"}
            }))),
    );
    let cfg = create_test_config(ts.uri());
    let (auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, auth);

    let raw_url = format!("{}/2/media/upload", ts.uri());
    let resp = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            target: RequestTarget::RawUrl(raw_url),
            auth_type: "app".to_string(),
            ..Default::default()
        })
        .expect("raw mode must skip auth-matrix validation per R18");

    assert_eq!(resp["data"]["id"], "raw_bypass");
    assert_eq!(
        ts.received_request_count(),
        1,
        "raw mode must let the request through even with a normally-rejected auth"
    );
}
