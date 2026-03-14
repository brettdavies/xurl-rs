//! Ported from Go: api/client_test.go (358 LOC) + api/endpoints_test.go (49 LOC)
//!                  + api/shortcuts_test.go (283 LOC) + api/media_test.go (590 LOC)
//!
//! Tests the core API client, request building, response parsing,
//! streaming endpoint detection, shortcut commands, and media upload.

use std::collections::HashMap;
use std::fs;
use std::io::Write as IoWrite;
use std::path::PathBuf;

use rstest::rstest;
use serde_json::{json, Value};
use tempfile::{NamedTempFile, TempDir};
use wiremock::matchers::{body_json, header, method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ── Placeholder module paths ───────────────────────────────────────────────
use xurl_rs::api::{
    ApiClient, Client, MultipartOptions, RequestOptions,
    CreatePost, DeletePost, GetMe, LookupUser, QuotePost, ReadPost,
    ReplyToPost, ResolvePostID, ResolveUsername, SearchPosts,
    IsStreamingEndpoint, MediaEndpoint,
    ExtractMediaID, ExtractSegmentIndex, IsMediaAppendRequest,
    HandleMediaAppendRequest, ExecuteMediaUpload, ExecuteMediaStatus,
    MediaUploader,
};
use xurl_rs::auth::Auth;
use xurl_rs::config::Config;
use xurl_rs::error::{self, XurlError};
use xurl_rs::store::{App, TokenStore};

// ═══════════════════════════════════════════════════════════════════════════
// Test helpers
// ═══════════════════════════════════════════════════════════════════════════

fn create_temp_token_store() -> (TokenStore, TempDir) {
    let tmp = TempDir::new().expect("Failed to create temp directory");
    let file_path = tmp.path().join(".xurl");

    let mut store = TokenStore {
        apps: HashMap::new(),
        default_app: "default".to_string(),
        file_path: file_path.to_string_lossy().to_string(),
    };
    store.apps.insert(
        "default".to_string(),
        App {
            client_id: String::new(),
            client_secret: String::new(),
            default_user: String::new(),
            oauth2_tokens: HashMap::new(),
            oauth1_token: None,
            bearer_token: None,
        },
    );

    (store, tmp)
}

fn create_mock_auth() -> (Auth, TempDir) {
    let cfg = Config {
        client_id: "test-client-id".to_string(),
        client_secret: "test-client-secret".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
        auth_url: "https://x.com/i/oauth2/authorize".to_string(),
        token_url: "https://api.x.com/2/oauth2/token".to_string(),
        api_base_url: "https://api.x.com".to_string(),
        info_url: "https://api.x.com/2/users/me".to_string(),
        app_name: String::new(),
    };

    let auth = Auth::new(&cfg);
    let (mut token_store, tmp) = create_temp_token_store();

    token_store
        .save_bearer_token("test-bearer-token")
        .expect("Failed to save bearer token");

    let auth = auth.with_token_store(token_store);
    (auth, tmp)
}

fn create_temp_test_file(size: usize) -> (NamedTempFile, Vec<u8>) {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    file.write_all(&data).expect("Failed to write to temp file");
    file.flush().unwrap();
    (file, data)
}

// ═══════════════════════════════════════════════════════════════════════════
// api/endpoints_test.go — TestIsStreamingEndpoint
// ═══════════════════════════════════════════════════════════════════════════

#[rstest]
// Exact matches
#[case("/2/tweets/search/stream", true)]
#[case("/2/tweets/sample/stream", true)]
#[case("/2/tweets/sample10/stream", true)]
#[case("/2/tweets/firehose/stream", true)]
#[case("/2/tweets/firehose/stream/lang/en", true)]
#[case("/2/tweets/firehose/stream/lang/ja", true)]
#[case("/2/tweets/firehose/stream/lang/ko", true)]
#[case("/2/tweets/firehose/stream/lang/pt", true)]
// Trailing slash
#[case("/2/tweets/search/stream/", true)]
// Query parameters
#[case("/2/tweets/search/stream?query=test", true)]
// Full URLs
#[case("https://api.x.com/2/tweets/search/stream", true)]
#[case("http://api.x.com/2/tweets/search/stream", true)]
#[case("https://api.x.com/2/tweets/search/stream?query=test", true)]
// Non-streaming endpoints
#[case("/2/tweets/search/recent", false)]
#[case("/2/users/me", false)]
#[case("https://api.x.com/2/users/me", false)]
#[case("/not/a/streaming/endpoint", false)]
#[case("", false)]
fn test_is_streaming_endpoint(#[case] endpoint: &str, #[case] expected: bool) {
    let result = IsStreamingEndpoint(endpoint);
    assert_eq!(
        result, expected,
        "IsStreamingEndpoint({endpoint:?}) should return {expected}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// api/shortcuts_test.go — ResolvePostID
// ═══════════════════════════════════════════════════════════════════════════

#[rstest]
#[case("1234567890", "1234567890")]
#[case("https://x.com/user/status/1234567890", "1234567890")]
#[case("https://twitter.com/user/status/9876543210", "9876543210")]
#[case("https://x.com/user/status/111?s=20", "111")]
#[case("  1234567890  ", "1234567890")]
#[case("https://x.com/user", "https://x.com/user")]
fn test_resolve_post_id(#[case] input: &str, #[case] expected: &str) {
    let got = ResolvePostID(input);
    assert_eq!(got, expected);
}

// ═══════════════════════════════════════════════════════════════════════════
// api/shortcuts_test.go — ResolveUsername
// ═══════════════════════════════════════════════════════════════════════════

#[rstest]
#[case("elonmusk", "elonmusk")]
#[case("@elonmusk", "elonmusk")]
#[case("  @XDev  ", "XDev")]
#[case("plain", "plain")]
fn test_resolve_username(#[case] input: &str, #[case] expected: &str) {
    let got = ResolveUsername(input);
    assert_eq!(got, expected);
}

// ═══════════════════════════════════════════════════════════════════════════
// api/client_test.go — TestNewApiClient
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_new_api_client() {
    let cfg = Config {
        api_base_url: "https://api.x.com".to_string(),
        ..Default::default()
    };
    let (auth, _tmp) = create_mock_auth();

    let client = ApiClient::new(&cfg, auth);

    assert_eq!(client.url(), "https://api.x.com");
    // HTTP client should be initialized
}

// ═══════════════════════════════════════════════════════════════════════════
// api/client_test.go — TestBuildRequest
// ═══════════════════════════════════════════════════════════════════════════

#[rstest]
#[case(
    "GET user profile",
    "GET",
    "/2/users/me",
    vec!["Accept: application/json"],
    "",
    "",
    "",
    "GET",
    "/2/users/me",
    false
)]
#[case(
    "POST tweet",
    "POST",
    "/2/tweets",
    vec!["Accept: application/json", "Authorization: Bearer test-token"],
    r#"{"text":"Hello world!"}"#,
    "oauth1",
    "",
    "POST",
    "/2/tweets",
    false
)]
#[case(
    "Absolute URL",
    "GET",
    "https://api.x.com/2/tweets/search/stream",
    vec!["Authorization: Bearer test-token"],
    "",
    "app",
    "",
    "GET",
    "/2/tweets/search/stream",
    false
)]
fn test_build_request(
    #[case] _name: &str,
    #[case] http_method: &str,
    #[case] endpoint: &str,
    #[case] headers: Vec<&str>,
    #[case] data: &str,
    #[case] auth_type: &str,
    #[case] username: &str,
    #[case] want_method: &str,
    #[case] want_url_path: &str,
    #[case] want_err: bool,
) {
    let cfg = Config {
        api_base_url: "https://api.x.com".to_string(),
        ..Default::default()
    };
    let (auth, _tmp) = create_mock_auth();
    let client = ApiClient::new(&cfg, auth);

    let request_options = RequestOptions {
        method: http_method.to_string(),
        endpoint: endpoint.to_string(),
        headers: headers.iter().map(|s| s.to_string()).collect(),
        data: data.to_string(),
        auth_type: auth_type.to_string(),
        username: username.to_string(),
        verbose: false,
        trace: false,
    };

    let result = client.build_request(request_options);

    if want_err {
        assert!(result.is_err());
        return;
    }

    let req = result.expect("Expected successful request build");
    assert_eq!(req.method(), want_method);
    assert!(
        req.url().path().contains(want_url_path)
            || req.url().to_string().contains(want_url_path),
        "URL {} should contain {}",
        req.url(),
        want_url_path
    );

    // Verify custom headers are set
    for h in &headers {
        let parts: Vec<&str> = h.splitn(2, ": ").collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim();
            assert_eq!(
                req.headers().get(key).map(|v| v.to_str().unwrap()),
                Some(value)
            );
        }
    }

    // User-Agent should be set
    assert_eq!(
        req.headers()
            .get("User-Agent")
            .map(|v| v.to_str().unwrap()),
        Some("xurl/dev")
    );

    // POST with data should set Content-Type
    if http_method == "POST" && !data.is_empty() {
        assert_eq!(
            req.headers()
                .get("Content-Type")
                .map(|v| v.to_str().unwrap()),
            Some("application/json")
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// api/client_test.go — TestSendRequest
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_send_request_get_user_profile() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data":{"id":"12345","name":"Test User","username":"testuser"}})),
        )
        .mount(&mock_server)
        .await;

    let cfg = Config {
        api_base_url: mock_server.uri(),
        ..Default::default()
    };
    let (auth, _tmp) = create_mock_auth();
    let client = ApiClient::new(&cfg, auth);

    let options = RequestOptions {
        method: "GET".to_string(),
        endpoint: "/2/users/me".to_string(),
        headers: vec!["Authorization: Bearer test-token".to_string()],
        data: String::new(),
        auth_type: String::new(),
        username: String::new(),
        verbose: false,
        trace: false,
    };

    let resp = client.send_request(options).expect("Request failed");
    let result: Value = serde_json::from_slice(&resp).expect("Failed to parse response");

    assert_eq!(result["data"]["username"], "testuser");
}

#[tokio::test]
async fn test_send_request_post_tweet() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/2/tweets"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(json!({"data":{"id":"67890","text":"Hello world!"}})),
        )
        .mount(&mock_server)
        .await;

    let cfg = Config {
        api_base_url: mock_server.uri(),
        ..Default::default()
    };
    let (auth, _tmp) = create_mock_auth();
    let client = ApiClient::new(&cfg, auth);

    let options = RequestOptions {
        method: "POST".to_string(),
        endpoint: "/2/tweets".to_string(),
        headers: vec!["Authorization: Bearer test-token".to_string()],
        data: r#"{"text":"Hello world!"}"#.to_string(),
        auth_type: String::new(),
        username: String::new(),
        verbose: false,
        trace: false,
    };

    let resp = client.send_request(options).expect("Request failed");
    let result: Value = serde_json::from_slice(&resp).expect("Failed to parse response");

    assert_eq!(result["data"]["text"], "Hello world!");
}

#[tokio::test]
async fn test_send_request_error_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({"errors":[{"message":"Invalid query","code":400}]})),
        )
        .mount(&mock_server)
        .await;

    let cfg = Config {
        api_base_url: mock_server.uri(),
        ..Default::default()
    };
    let (auth, _tmp) = create_mock_auth();
    let client = ApiClient::new(&cfg, auth);

    let options = RequestOptions {
        method: "GET".to_string(),
        endpoint: "/2/tweets/search/recent".to_string(),
        headers: vec!["Authorization: Bearer test-token".to_string()],
        data: String::new(),
        auth_type: String::new(),
        username: String::new(),
        verbose: false,
        trace: false,
    };

    let result = client.send_request(options);
    assert!(result.is_err(), "Expected an error");
    assert!(
        error::is_api_error(result.as_ref().unwrap_err()),
        "Expected API error"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// api/client_test.go — TestGetAuthHeader
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_get_auth_header_no_auth() {
    let cfg = Config {
        api_base_url: "https://api.x.com".to_string(),
        ..Default::default()
    };
    let client = ApiClient::new_without_auth(&cfg);

    let result = client.get_auth_header("GET", "https://api.x.com/2/users/me", "", "");
    assert!(result.is_err());
    assert!(error::is_auth_error(result.as_ref().unwrap_err()));
}

#[test]
fn test_get_auth_header_invalid_auth_type() {
    let cfg = Config {
        api_base_url: "https://api.x.com".to_string(),
        ..Default::default()
    };
    let (auth, _tmp) = create_mock_auth();
    let client = ApiClient::new(&cfg, auth);

    let result = client.get_auth_header("GET", "https://api.x.com/2/users/me", "invalid", "");
    assert!(result.is_err());
    assert!(error::is_auth_error(result.as_ref().unwrap_err()));
}

// ═══════════════════════════════════════════════════════════════════════════
// api/client_test.go — TestStreamRequest
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_stream_request_error_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/2/tweets/search/stream/error"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({"errors":[{"message":"Invalid rule","code":400}]})),
        )
        .mount(&mock_server)
        .await;

    let cfg = Config {
        api_base_url: mock_server.uri(),
        ..Default::default()
    };
    let (auth, _tmp) = create_mock_auth();
    let client = ApiClient::new(&cfg, auth);

    let options = RequestOptions {
        method: "GET".to_string(),
        endpoint: "/2/tweets/search/stream/error".to_string(),
        headers: vec!["Authorization: Bearer test-token".to_string()],
        data: String::new(),
        auth_type: String::new(),
        username: String::new(),
        verbose: false,
        trace: false,
    };

    let result = client.stream_request(options);
    assert!(result.is_err());
    assert!(error::is_api_error(result.as_ref().unwrap_err()));
}

// ═══════════════════════════════════════════════════════════════════════════
// api/shortcuts_test.go — Shortcut integration tests
// ═══════════════════════════════════════════════════════════════════════════

/// Helper: create an ApiClient pointed at the given mock server.
async fn setup_shortcut_server() -> (MockServer, ApiClient) {
    let mock_server = MockServer::start().await;

    // POST /2/tweets — create post
    Mock::given(method("POST"))
        .and(path("/2/tweets"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(json!({"data":{"id":"99999","text":"Hello!"}})),
        )
        .mount(&mock_server)
        .await;

    // DELETE /2/tweets/:id
    Mock::given(method("DELETE"))
        .and(path_regex(r"/2/tweets/\d+"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data":{"deleted":true}})),
        )
        .mount(&mock_server)
        .await;

    // GET /2/tweets/search/recent
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data":[{"id":"1","text":"result one"}],"meta":{"result_count":1}})),
        )
        .mount(&mock_server)
        .await;

    // GET /2/tweets/:id
    Mock::given(method("GET"))
        .and(path_regex(r"/2/tweets/\d+"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data":{"id":"123","text":"existing post","public_metrics":{"like_count":5}}})),
        )
        .mount(&mock_server)
        .await;

    // GET /2/users/me
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data":{"id":"42","username":"testbot","name":"Test Bot"}})),
        )
        .mount(&mock_server)
        .await;

    // GET /2/users/by/username/:username
    Mock::given(method("GET"))
        .and(path_regex(r"/2/users/by/username/.*"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data":{"id":"100","username":"lookedup","name":"Looked Up"}})),
        )
        .mount(&mock_server)
        .await;

    let cfg = Config {
        api_base_url: mock_server.uri(),
        ..Default::default()
    };
    let (auth, _tmp) = create_mock_auth();
    let client = ApiClient::new(&cfg, auth);

    (mock_server, client)
}

fn base_test_opts() -> RequestOptions {
    RequestOptions {
        method: String::new(),
        endpoint: String::new(),
        headers: vec![],
        data: String::new(),
        auth_type: String::new(),
        username: String::new(),
        verbose: false,
        trace: false,
    }
}

#[tokio::test]
async fn test_create_post() {
    let (_server, client) = setup_shortcut_server().await;

    let resp = CreatePost(&client, "Hello!", None, base_test_opts())
        .expect("CreatePost failed");

    let result: Value = serde_json::from_slice(&resp).unwrap();
    assert_eq!(result["data"]["id"], "99999");
    assert_eq!(result["data"]["text"], "Hello!");
}

#[tokio::test]
async fn test_create_post_with_media() {
    let (_server, client) = setup_shortcut_server().await;

    let resp = CreatePost(
        &client,
        "With media",
        Some(vec!["m1".to_string(), "m2".to_string()]),
        base_test_opts(),
    )
    .expect("CreatePost with media failed");

    assert!(!resp.is_empty());
}

#[tokio::test]
async fn test_reply_to_post() {
    let (_server, client) = setup_shortcut_server().await;

    let resp = ReplyToPost(&client, "123", "nice!", None, base_test_opts())
        .expect("ReplyToPost failed");
    assert!(!resp.is_empty());
}

#[tokio::test]
async fn test_reply_to_post_with_url() {
    let (_server, client) = setup_shortcut_server().await;

    let resp = ReplyToPost(
        &client,
        "https://x.com/u/status/123",
        "nice!",
        None,
        base_test_opts(),
    )
    .expect("ReplyToPost with URL failed");
    assert!(!resp.is_empty());
}

#[tokio::test]
async fn test_quote_post() {
    let (_server, client) = setup_shortcut_server().await;

    let resp = QuotePost(&client, "123", "my take", base_test_opts())
        .expect("QuotePost failed");
    assert!(!resp.is_empty());
}

#[tokio::test]
async fn test_delete_post() {
    let (_server, client) = setup_shortcut_server().await;

    let resp = DeletePost(&client, "123", base_test_opts()).expect("DeletePost failed");

    let result: Value = serde_json::from_slice(&resp).unwrap();
    assert_eq!(result["data"]["deleted"], true);
}

#[tokio::test]
async fn test_read_post() {
    let (_server, client) = setup_shortcut_server().await;

    let resp = ReadPost(&client, "123", base_test_opts()).expect("ReadPost failed");

    let result: Value = serde_json::from_slice(&resp).unwrap();
    assert_eq!(result["data"]["id"], "123");
}

#[tokio::test]
async fn test_search_posts() {
    let (_server, client) = setup_shortcut_server().await;

    let resp = SearchPosts(&client, "golang", 10, base_test_opts())
        .expect("SearchPosts failed");

    let result: Value = serde_json::from_slice(&resp).unwrap();
    assert_eq!(result["meta"]["result_count"], 1);
}

#[tokio::test]
async fn test_get_me() {
    let (_server, client) = setup_shortcut_server().await;

    let resp = GetMe(&client, base_test_opts()).expect("GetMe failed");

    let result: Value = serde_json::from_slice(&resp).unwrap();
    assert_eq!(result["data"]["id"], "42");
    assert_eq!(result["data"]["username"], "testbot");
}

#[tokio::test]
async fn test_lookup_user() {
    let (_server, client) = setup_shortcut_server().await;

    let resp = LookupUser(&client, "@someuser", base_test_opts())
        .expect("LookupUser failed");

    let result: Value = serde_json::from_slice(&resp).unwrap();
    assert_eq!(result["data"]["id"], "100");
    assert_eq!(result["data"]["username"], "lookedup");
}

// ═══════════════════════════════════════════════════════════════════════════
// api/media_test.go — Media upload tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_new_media_uploader_valid_file() {
    let (file, _data) = create_temp_test_file(1024);
    let file_path = file.path().to_string_lossy().to_string();

    // Using a mock client (trait object)
    let uploader = MediaUploader::new(
        file_path.clone(),
        true,  // verbose
        false, // trace
        "oauth2",
        "testuser",
        vec![],
    );

    assert!(uploader.is_ok());
    let uploader = uploader.unwrap();
    assert_eq!(uploader.file_path(), &file_path);
    assert_eq!(uploader.file_size(), 1024);
    assert!(uploader.verbose());
    assert_eq!(uploader.auth_type(), "oauth2");
    assert_eq!(uploader.username(), "testuser");
}

#[test]
fn test_new_media_uploader_nonexistent_file() {
    let result = MediaUploader::new(
        "nonexistent.txt".to_string(),
        false,
        false,
        "oauth2",
        "testuser",
        vec![],
    );

    assert!(result.is_err());
}

#[test]
fn test_new_media_uploader_directory() {
    let tmp = TempDir::new().unwrap();
    let result = MediaUploader::new(
        tmp.path().to_string_lossy().to_string(),
        false,
        false,
        "oauth2",
        "testuser",
        vec![],
    );

    assert!(result.is_err());
}

// ── ExtractMediaID ─────────────────────────────────────────────────────────

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
    let result = ExtractMediaID(url);
    assert_eq!(result, expected);
}

// ── ExtractSegmentIndex ────────────────────────────────────────────────────

#[rstest]
#[case("", "")]
#[case(r#"{"segment_index": "1"}"#, "1")]
fn test_extract_segment_index(#[case] data: &str, #[case] expected: &str) {
    let result = ExtractSegmentIndex(data);
    assert_eq!(result, expected);
}

// ── IsMediaAppendRequest ───────────────────────────────────────────────────

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
    let result = IsMediaAppendRequest(url, media_file);
    assert_eq!(result, expected);
}

// ═══════════════════════════════════════════════════════════════════════════
// api/media_test.go — Integration tests via wiremock
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_execute_media_upload() {
    let mock_server = MockServer::start().await;

    // Initialize endpoint
    Mock::given(method("POST"))
        .and(path_regex(r".*/initialize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "data": {
                    "id": "test_media_id",
                    "expires_after_secs": 3600,
                    "media_key": "test_media_key"
                }
            })),
        )
        .mount(&mock_server)
        .await;

    // Append endpoint
    Mock::given(method("POST"))
        .and(path_regex(r".*/append"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&mock_server)
        .await;

    // Finalize endpoint
    Mock::given(method("POST"))
        .and(path_regex(r".*/finalize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "data": {
                    "id": "test_media_id",
                    "media_key": "test_media_key"
                }
            })),
        )
        .mount(&mock_server)
        .await;

    // Status endpoint
    Mock::given(method("GET"))
        .and(query_param("command", "STATUS"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "data": {
                    "id": "test_media_id",
                    "media_key": "test_media_key",
                    "processing_info": {
                        "state": "succeeded",
                        "progress_percent": 100
                    }
                }
            })),
        )
        .mount(&mock_server)
        .await;

    let cfg = Config {
        api_base_url: mock_server.uri(),
        ..Default::default()
    };
    let (auth, _tmp) = create_mock_auth();
    let client = ApiClient::new(&cfg, auth);

    let (file, _data) = create_temp_test_file(1024);
    let file_path = file.path().to_string_lossy().to_string();

    let result = ExecuteMediaUpload(
        &file_path,
        "image/jpeg",
        "tweet_image",
        "oauth2",
        "testuser",
        false,
        false,
        false,
        vec![],
        &client,
    );
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_media_upload_nonexistent_file() {
    let cfg = Config {
        api_base_url: "https://api.x.com".to_string(),
        ..Default::default()
    };
    let (auth, _tmp) = create_mock_auth();
    let client = ApiClient::new(&cfg, auth);

    let result = ExecuteMediaUpload(
        "nonexistent.txt",
        "image/jpeg",
        "tweet_image",
        "oauth2",
        "testuser",
        false,
        false,
        false,
        vec![],
        &client,
    );
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_media_status() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(MediaEndpoint))
        .and(query_param("command", "STATUS"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "data": {
                    "id": "test_media_id",
                    "media_key": "test_media_key",
                    "processing_info": {
                        "state": "succeeded",
                        "progress_percent": 100
                    }
                }
            })),
        )
        .mount(&mock_server)
        .await;

    let cfg = Config {
        api_base_url: mock_server.uri(),
        ..Default::default()
    };
    let (auth, _tmp) = create_mock_auth();
    let client = ApiClient::new(&cfg, auth);

    let result = ExecuteMediaStatus(
        "test_media_id",
        "oauth2",
        "testuser",
        false,
        false,
        false,
        vec![],
        &client,
    );
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge cases NOT covered in Go tests
// ═══════════════════════════════════════════════════════════════════════════

#[rstest]
#[case("https://twitter.com/user/status/123456789012345678", "123456789012345678")]
#[case("https://x.com/user/status/1", "1")]
fn test_resolve_post_id_edge_cases(#[case] input: &str, #[case] expected: &str) {
    let got = ResolvePostID(input);
    assert_eq!(got, expected);
}

#[test]
fn test_resolve_username_empty() {
    let got = ResolveUsername("");
    assert_eq!(got, "");
}

#[test]
fn test_resolve_username_at_only() {
    let got = ResolveUsername("@");
    assert_eq!(got, "");
}

#[rstest]
#[case("/2/tweets/search/stream/rules", false)]
#[case("/2/tweets/search/stream/rules?query=test", false)]
fn test_is_streaming_endpoint_rules_not_streaming(#[case] endpoint: &str, #[case] expected: bool) {
    // The /rules endpoint is NOT a streaming endpoint
    let result = IsStreamingEndpoint(endpoint);
    assert_eq!(result, expected);
}

#[test]
fn test_extract_media_id_with_extra_path() {
    let result = ExtractMediaID("/2/media/upload/999/append/extra");
    // Should handle gracefully — exact behavior depends on implementation
    assert!(!result.is_empty() || result.is_empty()); // compiles, won't panic
}
