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

use xurl::api::{
    self, ApiClient, RequestOptions, extract_media_id, extract_segment_index,
    is_media_append_request, is_streaming_endpoint,
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
}

// ── Test helpers ───────────────────────────────────────────────────────

fn create_test_config(base_url: &str) -> Config {
    Config {
        client_id: "test-client-id".to_string(),
        client_secret: "test-client-secret".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
        auth_url: "https://x.com/i/oauth2/authorize".to_string(),
        token_url: "https://api.x.com/2/oauth2/token".to_string(),
        api_base_url: base_url.to_string(),
        info_url: format!("{base_url}/2/users/me"),
        app_name: String::new(),
    }
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
            oauth2_tokens: BTreeMap::new(),
            oauth1_token: None,
            bearer_token: Some(Token {
                token_type: TokenType::Bearer,
                bearer: Some("test-bearer-token".to_string()),
                oauth2: None,
                oauth1: None,
            }),
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
        oauth2_tokens: BTreeMap::new(),
        oauth1_token: None,
        bearer_token: None,
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

fn base_opts() -> RequestOptions {
    RequestOptions {
        verbose: false,
        ..Default::default()
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let _client = ApiClient::new(&cfg, &mut auth);
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let opts = RequestOptions {
        method: "GET".to_string(),
        endpoint: "/2/users/me".to_string(),
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let opts = RequestOptions {
        method: "POST".to_string(),
        endpoint: "/2/tweets".to_string(),
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
    let ts = TestServer::new();
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{"id":"1"}})),
            ),
    );

    let cfg = create_test_config(ts.uri());
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let opts = RequestOptions {
        method: "GET".to_string(),
        endpoint: "/2/users/me".to_string(),
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
    let (mut auth, _tmp) = create_mock_auth_with_oauth1(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let opts = RequestOptions {
        method: "GET".to_string(),
        endpoint: "/2/users/me".to_string(),
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
    let (mut auth, _tmp) = create_mock_auth_with_oauth2(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let opts = RequestOptions {
        method: "GET".to_string(),
        endpoint: "/2/users/me".to_string(),
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            endpoint: "/2/users/me".to_string(),
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let err = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            endpoint: "/2/tweets/search/recent".to_string(),
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    // Non-JSON 200 response returns empty JSON object
    let resp = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            endpoint: "/2/bad-json".to_string(),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(resp, serde_json::json!({}));
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::create_post(&mut client, "Hello!", &[], &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::reply_to_post(&mut client, "123", "nice!", &[], &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::reply_to_post(
        &mut client,
        "https://x.com/u/status/123",
        "reply via URL",
        &[],
        &base_opts(),
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::quote_post(&mut client, "123", "my take", &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::delete_post(&mut client, "123", &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::read_post(&mut client, "123", &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::search_posts(&mut client, "golang", 10, &base_opts()).unwrap();
    assert_eq!(resp.meta.as_ref().unwrap().result_count, Some(1));
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::get_me(&mut client, &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::lookup_user(&mut client, "@someuser", &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let media_ids = vec!["m1".to_string(), "m2".to_string()];
    let resp = api::create_post(&mut client, "With media", &media_ids, &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = client.send_request(&RequestOptions {
        method: "POST".to_string(),
        endpoint: "/2/media/upload/initialize".to_string(),
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = client
        .send_request(&RequestOptions {
            method: "POST".to_string(),
            endpoint: "/2/media/upload/test_media_id/finalize".to_string(),
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = client
        .send_request(&RequestOptions {
            method: "GET".to_string(),
            endpoint: "/2/media/upload?command=STATUS&media_id=test_media_id".to_string(),
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let err = client
        .stream_request(&RequestOptions {
            method: "GET".to_string(),
            endpoint: "/2/tweets/search/stream/error".to_string(),
            ..Default::default()
        })
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::get_usage(&mut client, &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::get_usage(&mut client, &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::get_usage(&mut client, &base_opts());
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::get_usage(&mut client, &base_opts());
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::get_usage(&mut client, &base_opts());
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
    let (mut auth, _tmp) = create_mock_auth_with_oauth1(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::get_usage(&mut client, &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_oauth2(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::get_usage(&mut client, &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::get_usage(&mut client, &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::get_usage(&mut client, &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    // Pass opts with stale data to verify it gets cleared
    let mut opts = base_opts();
    opts.data = r#"{"stale":"body"}"#.to_string();
    let resp = api::get_usage(&mut client, &opts);
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let result = api::create_post(&mut client, "test", &[], &base_opts());
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let result = api::get_me(&mut client, &base_opts());
    let err = result.unwrap_err();
    assert!(
        err.is_api(),
        "Should be API error (not JSON error) for errors-only 200: {err}"
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let result = api::delete_post(&mut client, "123", &base_opts());
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let result = api::search_posts(&mut client, "test", 10, &base_opts());
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let result = api::get_me(&mut client, &base_opts());
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::create_post(&mut client, "Hello!", &[], &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    let resp = api::like_post(&mut client, "42", "123", &base_opts()).unwrap();
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
    let (mut auth, _tmp) = create_mock_auth_with_bearer(ts.uri());
    let mut client = ApiClient::new(&cfg, &mut auth);

    // verified is Option<bool> — "true" (string) should fail deserialization
    let result = api::lookup_user(&mut client, "bad", &base_opts());
    assert!(
        result.is_err(),
        "Should fail: string 'true' where bool expected"
    );
}
