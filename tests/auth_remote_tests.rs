//! Integration tests for the headless (`--no-browser`) OAuth2 PKCE flow.
//!
//! Tests the two-step no-browser flow: step 1 generates the auth URL and persists
//! PKCE state; step 2 accepts the redirect URL, exchanges the code for a token,
//! and saves it to the token store.

mod common;

use std::collections::BTreeMap;

use tempfile::TempDir;
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use xurl::auth::Auth;
use xurl::auth::pending;
use xurl::config::Config;
use xurl::store::{App, TokenStore};

// ── Test helpers ───────────────────────────────────────────────────────

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
    cfg.token_url = format!("{base_url}/2/oauth2/token");
    cfg.api_base_url = base_url.to_string();
    cfg.info_url = format!("{base_url}/2/users/me");
    cfg.app_name = String::new();
    cfg
}

fn create_test_auth(base_url: &str, tmp: &TempDir) -> Auth {
    let cfg = create_test_config(base_url);

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

    auth_for(&cfg, store)
}

// ── Step 1 tests ──────────────────────────────────────────────────────

#[test]
fn step1_creates_pending_file_and_returns_auth_url() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    assert!(!pending_path.exists());

    let auth_url = auth.remote_oauth2_step1(&pending_path).unwrap();

    // Auth URL should contain all OAuth2 parameters
    let parsed = Url::parse(&auth_url).unwrap();
    let params: BTreeMap<String, String> = parsed
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    assert_eq!(params.get("response_type").unwrap(), "code");
    assert_eq!(params.get("client_id").unwrap(), "test-client-id");
    assert!(params.contains_key("state"));
    assert!(params.contains_key("code_challenge"));
    assert_eq!(params.get("code_challenge_method").unwrap(), "S256");

    // Pending file should exist
    assert!(pending_path.exists());

    // Pending state should be loadable and contain correct data
    let state = pending::load(&pending_path).unwrap();
    assert_eq!(state.client_id, "test-client-id");
    assert!(!state.code_verifier.is_empty());
    assert!(!state.state.is_empty());
}

#[test]
fn step1_overwrites_existing_pending_file() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    // Run step 1 twice
    let url1 = auth.remote_oauth2_step1(&pending_path).unwrap();
    let state1 = pending::load(&pending_path).unwrap();

    let url2 = auth.remote_oauth2_step1(&pending_path).unwrap();
    let state2 = pending::load(&pending_path).unwrap();

    // Second run should produce different state/verifier
    assert_ne!(url1, url2);
    assert_ne!(state1.state, state2.state);
    assert_ne!(state1.code_verifier, state2.code_verifier);
}

// ── Step 2 tests ──────────────────────────────────────────────────────

#[test]
fn step2_happy_path_exchanges_code_and_saves_token() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    // Step 1
    auth.remote_oauth2_step1(&pending_path).unwrap();
    let state = pending::load(&pending_path).unwrap();

    // Mock token exchange endpoint
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "remote-access-token",
                "refresh_token": "remote-refresh-token",
                "expires_in": 7200,
                "token_type": "bearer"
            }))),
    );

    // Mock /2/users/me for username resolution
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "12345", "username": "remoteuser"}
            }))),
    );

    // Build a fake redirect URL with the correct state (properly encoded)
    let mut redirect = Url::parse("http://localhost:8080/callback").unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("code", "test-auth-code")
        .append_pair("state", &state.state);
    let redirect_url = redirect.to_string();

    // Step 2
    let access_token = auth
        .remote_oauth2_step2(&redirect_url, "", &pending_path)
        .unwrap();

    assert_eq!(access_token, "remote-access-token");

    // Pending file should be deleted
    assert!(!pending_path.exists());

    // Token should be saved in the store
    let token = auth.token_store().get_oauth2_token("remoteuser");
    assert!(token.is_some());
    let oauth2 = token.unwrap().oauth2.as_ref().unwrap();
    assert_eq!(oauth2.access_token, "remote-access-token");
    assert_eq!(oauth2.refresh_token, "remote-refresh-token");
}

#[test]
fn step2_with_explicit_username_skips_resolution() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();
    let state = pending::load(&pending_path).unwrap();

    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "explicit-token",
                "refresh_token": "explicit-refresh",
                "expires_in": 7200
            }))),
    );

    let mut redirect = Url::parse("http://localhost:8080/callback").unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("code", "test-code")
        .append_pair("state", &state.state);
    let redirect_url = redirect.to_string();

    let access_token = auth
        .remote_oauth2_step2(&redirect_url, "explicituser", &pending_path)
        .unwrap();

    assert_eq!(access_token, "explicit-token");
    assert!(
        auth.token_store()
            .get_oauth2_token("explicituser")
            .is_some()
    );
}

#[test]
fn step2_without_prior_step1_returns_error() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    let err = auth
        .remote_oauth2_step2(
            "http://localhost:8080/callback?code=abc&state=xyz",
            "",
            &pending_path,
        )
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("PendingStateNotFound"),
        "Expected PendingStateNotFound error, got: {msg}"
    );
}

#[test]
fn step2_state_mismatch_returns_error_and_preserves_pending() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();

    let redirect_url = "http://localhost:8080/callback?code=abc&state=wrong-state";

    let err = auth
        .remote_oauth2_step2(redirect_url, "", &pending_path)
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("StateMismatch"),
        "Expected StateMismatch error, got: {msg}"
    );

    // Pending file should be preserved for retry
    assert!(pending_path.exists());
}

#[test]
fn step2_missing_code_parameter_returns_error() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();
    let state = pending::load(&pending_path).unwrap();

    // URL with state but no code
    let mut redirect = Url::parse("http://localhost:8080/callback").unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("state", &state.state);
    let redirect_url = redirect.to_string();

    let err = auth
        .remote_oauth2_step2(&redirect_url, "", &pending_path)
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("MissingCode"),
        "Expected MissingCode error, got: {msg}"
    );

    // Pending file preserved
    assert!(pending_path.exists());
}

#[test]
fn step2_client_id_mismatch_returns_error() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let pending_path = tmp.path().join(".xurl.pending");

    // Create auth with one client_id and do step 1
    let auth1 = create_test_auth(ts.uri(), &tmp);
    auth1.remote_oauth2_step1(&pending_path).unwrap();
    let state = pending::load(&pending_path).unwrap();

    // Create auth with a DIFFERENT client_id
    let mut cfg2 = Config::new();
    cfg2.client_id = "different-client-id".to_string();
    cfg2.client_secret = "test-client-secret".to_string();
    cfg2.redirect_uri = "http://localhost:8080/callback".to_string();
    cfg2.auth_url = "https://x.com/i/oauth2/authorize".to_string();
    cfg2.token_url = format!("{}/2/oauth2/token", ts.uri());
    cfg2.api_base_url = ts.uri().to_string();
    cfg2.info_url = format!("{}/2/users/me", ts.uri());
    cfg2.app_name = String::new();
    let file_path2 = tmp.path().join(".xurl2");
    let mut store2 = TokenStore {
        apps: BTreeMap::new(),
        default_app: "default".to_string(),
        file_path: file_path2,
    };
    store2.apps.insert(
        "default".to_string(),
        App {
            client_id: "different-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            default_user: String::new(),
            redirect_uri: String::new(),
            oauth2_tokens: BTreeMap::new(),
            oauth1_token: None,
            bearer_token: None,
            unnamed_oauth2_token: None,
        },
    );
    let mut auth2 = auth_for(&cfg2, store2);

    let mut redirect = Url::parse("http://localhost:8080/callback").unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("code", "abc")
        .append_pair("state", &state.state);
    let redirect_url = redirect.to_string();

    let err = auth2
        .remote_oauth2_step2(&redirect_url, "", &pending_path)
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("AppMismatch"),
        "Expected AppMismatch error, got: {msg}"
    );

    // Pending file preserved
    assert!(pending_path.exists());
}

#[test]
fn step2_expired_pending_returns_ttl_error() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    // Manually create a pending file with an old created_at
    let old_state = pending::PendingOAuth2State {
        code_verifier: "verifier".to_string(),
        state: "state".to_string(),
        client_id: "test-client-id".to_string(),
        app_name: String::new(),
        created_at: 0, // epoch = 1970, definitely expired
    };
    pending::save(&old_state, &pending_path).unwrap();

    let err = auth
        .remote_oauth2_step2(
            "http://localhost:8080/callback?code=abc&state=state",
            "",
            &pending_path,
        )
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("PendingStateExpired"),
        "Expected PendingStateExpired error, got: {msg}"
    );

    // Expired file should be cleaned up by load()
    assert!(!pending_path.exists());
}

#[test]
fn step2_failed_exchange_preserves_pending_for_retry() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();
    let state = pending::load(&pending_path).unwrap();

    // Mock a 400 error (expired/revoked code)
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "Authorization code expired"
            }))),
    );

    let mut redirect = Url::parse("http://localhost:8080/callback").unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("code", "expired-code")
        .append_pair("state", &state.state);
    let redirect_url = redirect.to_string();

    let err = auth
        .remote_oauth2_step2(&redirect_url, "", &pending_path)
        .unwrap_err();

    // Should be a token exchange error
    let msg = err.to_string();
    assert!(
        msg.contains("TokenExchangeError"),
        "Expected TokenExchangeError, got: {msg}"
    );

    // Pending file should be preserved for retry
    assert!(pending_path.exists());
}

#[test]
fn step1_auth_url_contains_redirect_uri() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    let auth_url = auth.remote_oauth2_step1(&pending_path).unwrap();
    assert!(auth_url.contains("redirect_uri="));
    assert!(auth_url.contains("localhost"));
}

#[test]
fn full_round_trip_token_matches_interactive_format() {
    // Verify the token saved by remote flow is identical in structure
    // to what the interactive flow would produce.
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();
    let state = pending::load(&pending_path).unwrap();

    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "at-123",
                "refresh_token": "rt-456",
                "expires_in": 7200
            }))),
    );

    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "1", "username": "testuser"}
            }))),
    );

    let mut redirect = Url::parse("http://localhost:8080/callback").unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("code", "c")
        .append_pair("state", &state.state);
    let redirect_url = redirect.to_string();

    auth.remote_oauth2_step2(&redirect_url, "", &pending_path)
        .unwrap();

    let token = auth
        .token_store()
        .get_oauth2_token("testuser")
        .expect("token should exist");
    let oauth2 = token.oauth2.as_ref().expect("oauth2 payload");

    assert_eq!(oauth2.access_token, "at-123");
    assert_eq!(oauth2.refresh_token, "rt-456");
    assert!(oauth2.expiration_time > 0);
}

// ── Adversarial / Red Team Tests ──────────────────────────────────────

#[test]
fn step2_empty_redirect_url_returns_parse_error() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();

    let err = auth.remote_oauth2_step2("", "", &pending_path).unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("MissingCode") || msg.contains("failed to parse"),
        "Expected parse error for empty URL, got: {msg}"
    );
    // Pending file preserved for retry
    assert!(pending_path.exists());
}

#[test]
fn step2_garbage_redirect_url_returns_parse_error() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();

    let err = auth
        .remote_oauth2_step2("not a url at all!!!", "", &pending_path)
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("InvalidRedirectURL"),
        "Expected InvalidRedirectURL parse error, got: {msg}"
    );
    assert!(pending_path.exists());
}

#[test]
fn step2_redirect_with_error_param_returns_missing_code() {
    // When the user denies authorization, Twitter redirects with ?error=access_denied
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();

    let err = auth
        .remote_oauth2_step2(
            "http://localhost:8080/callback?error=access_denied&error_description=The+user+denied+the+request",
            "",
            &pending_path,
        )
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("MissingState"),
        "Expected MissingState when user denies auth (no state param), got: {msg}"
    );
    assert!(pending_path.exists());
}

#[test]
fn step2_token_exchange_200_but_no_access_token() {
    // Twitter returns 200 but with an error body (no access_token field)
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();
    let state = pending::load(&pending_path).unwrap();

    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "Value passed for the authorization code was invalid."
            }))),
    );

    let mut redirect = Url::parse("http://localhost:8080/callback").unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("code", "bad-code")
        .append_pair("state", &state.state);
    let redirect_url = redirect.to_string();

    let err = auth
        .remote_oauth2_step2(&redirect_url, "", &pending_path)
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("TokenExchangeError") && msg.contains("no access_token"),
        "Expected no access_token error, got: {msg}"
    );
    // Pending preserved for retry
    assert!(pending_path.exists());
}

#[test]
fn step2_username_resolution_failure_saves_unnamed_and_consumes_pending() {
    // KTD7: token exchange succeeds + /2/users/me fails + empty caller ->
    // token persists in the unnamed slot, run_remote_step2 reaches the
    // normal pending::delete success branch, and the access token surfaces
    // to the caller as Ok.
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();
    let state = pending::load(&pending_path).unwrap();

    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "good-token",
                "refresh_token": "good-refresh",
                "expires_in": 7200
            }))),
    );

    // /2/users/me returns an error
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "title": "Unauthorized",
                "status": 401
            }))),
    );

    let mut redirect = Url::parse("http://localhost:8080/callback").unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("code", "good-code")
        .append_pair("state", &state.state);
    let redirect_url = redirect.to_string();

    let access_token = auth
        .remote_oauth2_step2(&redirect_url, "", &pending_path)
        .expect("empty-caller + /me-failure path must return Ok with the new access token");
    assert_eq!(access_token, "good-token");

    let token = auth
        .token_store
        .get_oauth2_token_unnamed_for_app("default")
        .expect("unnamed slot populated after /me failure");
    let oauth2 = token.oauth2.as_ref().unwrap();
    assert_eq!(oauth2.access_token, "good-token");
    assert_eq!(oauth2.refresh_token, "good-refresh");

    // Pending file is consumed by run_remote_step2's normal success path
    // because exchange_code_for_token no longer propagates the /me error.
    assert!(
        !pending_path.exists(),
        "pending file should be deleted on the Ok path"
    );
}

#[test]
fn exchange_code_for_token_nonempty_username_skips_me_and_saves_named() {
    // KTD7: when the CLI threads a `USERNAME` positional through to
    // exchange_code_for_token, the function saves under that username and
    // never consults /2/users/me.
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();
    let state = pending::load(&pending_path).unwrap();

    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "alice-access",
                "refresh_token": "alice-refresh",
                "expires_in": 7200
            }))),
    );
    // Deliberately do NOT mock /2/users/me; any call to it returns 404 from
    // wiremock's default unmatched handler, which would propagate as Err
    // and fail this test if the empty-username branch were taken.

    let mut redirect = Url::parse("http://localhost:8080/callback").unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("code", "good-code")
        .append_pair("state", &state.state);
    let redirect_url = redirect.to_string();

    let access_token = auth
        .remote_oauth2_step2(&redirect_url, "alice", &pending_path)
        .expect("non-empty username path must succeed without /me");
    assert_eq!(access_token, "alice-access");

    let token = auth
        .token_store
        .get_oauth2_token("alice")
        .expect("token saved under supplied username");
    let oauth2 = token.oauth2.as_ref().unwrap();
    assert_eq!(oauth2.access_token, "alice-access");
    assert_eq!(oauth2.refresh_token, "alice-refresh");

    // Unnamed slot must remain empty on the named path.
    assert!(
        auth.token_store
            .get_oauth2_token_unnamed_for_app("default")
            .is_none(),
        "non-empty username path must not write to the unnamed slot"
    );
}

#[test]
fn test_exchange_code_for_token_empty_username_me_ok_saves_named() {
    // KTD7: empty caller + /me Ok -> token saved under the discovered name;
    // unnamed slot untouched.
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();
    let state = pending::load(&pending_path).unwrap();

    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "discovered-access",
                "refresh_token": "discovered-refresh",
                "expires_in": 7200
            }))),
    );
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "42", "username": "discovered"}
            }))),
    );

    let mut redirect = Url::parse("http://localhost:8080/callback").unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("code", "good-code")
        .append_pair("state", &state.state);
    let redirect_url = redirect.to_string();

    let access_token = auth
        .remote_oauth2_step2(&redirect_url, "", &pending_path)
        .expect("empty caller + /me Ok must succeed");
    assert_eq!(access_token, "discovered-access");

    let token = auth
        .token_store
        .get_oauth2_token("discovered")
        .expect("token saved under /me-discovered username");
    let oauth2 = token.oauth2.as_ref().unwrap();
    assert_eq!(oauth2.access_token, "discovered-access");
    assert_eq!(oauth2.refresh_token, "discovered-refresh");

    assert!(
        auth.token_store
            .get_oauth2_token_unnamed_for_app("default")
            .is_none(),
        "/me-success path must not touch the unnamed slot"
    );
}

#[test]
fn test_exchange_code_for_token_empty_username_me_failure_saves_unnamed() {
    // KTD7: empty caller + /me Err -> token saved in the unnamed slot,
    // named map untouched, function returns Ok with the access token.
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();
    let state = pending::load(&pending_path).unwrap();

    let named_before: Vec<String> = auth.token_store.get_oauth2_usernames();

    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "salvage-access",
                "refresh_token": "salvage-refresh",
                "expires_in": 7200
            }))),
    );
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
                "title": "Server Error",
                "status": 500
            }))),
    );

    let mut redirect = Url::parse("http://localhost:8080/callback").unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("code", "good-code")
        .append_pair("state", &state.state);
    let redirect_url = redirect.to_string();

    let access_token = auth
        .remote_oauth2_step2(&redirect_url, "", &pending_path)
        .expect("empty caller + /me Err must still return Ok with the access token");
    assert_eq!(access_token, "salvage-access");

    let token = auth
        .token_store
        .get_oauth2_token_unnamed_for_app("default")
        .expect("unnamed slot populated when /me fails");
    let oauth2 = token.oauth2.as_ref().unwrap();
    assert_eq!(oauth2.access_token, "salvage-access");
    assert_eq!(oauth2.refresh_token, "salvage-refresh");

    let named_after: Vec<String> = auth.token_store.get_oauth2_usernames();
    assert_eq!(
        named_before, named_after,
        "named map keys unchanged when /me fails"
    );
}

// ── CLI E2E tests ─────────────────────────────────────────────────────

#[test]
fn cli_no_browser_without_step_auto_engages_step1() {
    // U9: `--no-browser` (no `--step`) now auto-promotes to step 1 — emits
    // the authorize URL and exits 0 rather than rejecting at usage time.
    // The exact envelope shape is covered by the JSON-mode tests in
    // `cli_tests.rs`; this test asserts the exit code only so it survives
    // text-mode rendering changes. `XURL_TOKEN_STORE` points the child at a
    // tempdir store so the pending state lands beside it.
    let tmp = TempDir::new().expect("tempdir");
    let output = common::xr_with_store(&tmp.path().join(".xurl"))
        .args(["auth", "oauth2", "--no-browser"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "expected success; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_step_without_no_browser_fails() {
    let output = common::xr()
        .args(["auth", "oauth2", "--step", "1"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--no-browser"),
        "Expected --no-browser required error, got: {stderr}"
    );
}

#[test]
fn cli_step_3_rejected_by_value_parser() {
    let output = common::xr()
        .args(["auth", "oauth2", "--no-browser", "--step", "3"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not in 1..=2"),
        "Expected range error, got: {stderr}"
    );
}

#[test]
fn cli_step2_without_auth_url_fails() {
    let output = common::xr()
        .args(["auth", "oauth2", "--no-browser", "--step", "2"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--auth-url") || stderr.contains("auth-url"),
        "Expected --auth-url required error, got: {stderr}"
    );
}

#[test]
fn step2_redirect_url_with_code_but_no_state() {
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);
    let pending_path = tmp.path().join(".xurl.pending");

    auth.remote_oauth2_step1(&pending_path).unwrap();

    // URL has a code parameter but no state parameter
    let redirect_url = "http://localhost:8080/callback?code=abc";

    let err = auth
        .remote_oauth2_step2(redirect_url, "", &pending_path)
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("MissingState"),
        "Expected MissingState error, got: {msg}"
    );

    // Pending file preserved for retry
    assert!(pending_path.exists());
}

#[test]
fn cli_step1_with_auth_url_rejected() {
    let output = common::xr()
        .args([
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "1",
            "--auth-url",
            "http://example.com",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--auth-url") && stderr.contains("step 2"),
        "Expected error about --auth-url only for step 2, got: {stderr}"
    );
}

// ── Refresh resilience + get_oauth2_header precedence (U2) ────────────

/// Seeds the test auth's token store with an `OAuth2` token under `username`
/// with `expiration_time = 0` so the next `refresh_oauth2_token` call fires
/// the actual refresh POST instead of short-circuiting on the cached token.
fn seed_expired_named_oauth2(auth: &mut Auth, username: &str) {
    auth.token_store
        .save_oauth2_token(username, "old-access", "old-refresh", 0)
        .unwrap();
}

#[test]
fn refresh_with_caller_supplied_username_skips_fetch_username() {
    // KTD2: non-empty caller username -> save under that name, never call /me.
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);

    seed_expired_named_oauth2(&mut auth, "alice");

    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "alice-new-access",
                "refresh_token": "alice-new-refresh",
                "expires_in": 7200
            }))),
    );
    // Deliberately do NOT mock /2/users/me. Any call to it returns 404 from
    // wiremock's default unmatched handler, which would propagate as Err
    // before this PR; we assert it is never called.

    let access_token = auth.refresh_oauth2_token("alice").unwrap();
    assert_eq!(access_token, "alice-new-access");

    let token = auth
        .token_store
        .get_oauth2_token("alice")
        .expect("alice token present after refresh");
    let oauth2 = token.oauth2.as_ref().unwrap();
    assert_eq!(oauth2.access_token, "alice-new-access");
    assert_eq!(oauth2.refresh_token, "alice-new-refresh");

    // Unnamed slot untouched.
    assert!(
        auth.token_store
            .get_oauth2_token_unnamed_for_app("default")
            .is_none(),
        "named-caller refresh must not write to the unnamed slot"
    );
}

#[test]
fn refresh_with_empty_caller_and_me_ok_saves_named() {
    // KTD2: empty caller + /me Ok -> save under discovered username.
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);

    // Seed with a placeholder username so `get_first_oauth2_token` returns
    // it during the refresh entry lookup.
    seed_expired_named_oauth2(&mut auth, "placeholder");

    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "discovered-access",
                "refresh_token": "discovered-refresh",
                "expires_in": 7200
            }))),
    );
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "999", "username": "discovered"}
            }))),
    );

    let access_token = auth.refresh_oauth2_token("").unwrap();
    assert_eq!(access_token, "discovered-access");

    let token = auth
        .token_store
        .get_oauth2_token("discovered")
        .expect("discovered token present");
    let oauth2 = token.oauth2.as_ref().unwrap();
    assert_eq!(oauth2.access_token, "discovered-access");
    assert_eq!(oauth2.refresh_token, "discovered-refresh");

    assert!(
        auth.token_store
            .get_oauth2_token_unnamed_for_app("default")
            .is_none(),
        "/me-success path must not touch the unnamed slot"
    );
}

#[test]
fn refresh_with_empty_caller_and_me_failure_saves_unnamed() {
    // KTD2: empty caller + /me Err -> save into unnamed slot, return Ok.
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);

    seed_expired_named_oauth2(&mut auth, "placeholder");
    let named_before: Vec<String> = auth.token_store.get_oauth2_usernames();

    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "salvage-access",
                "refresh_token": "salvage-refresh",
                "expires_in": 7200
            }))),
    );
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
                "title": "Server Error",
                "status": 500
            }))),
    );

    let access_token = auth.refresh_oauth2_token("").unwrap();
    assert_eq!(access_token, "salvage-access");

    let token = auth
        .token_store
        .get_oauth2_token_unnamed_for_app("default")
        .expect("unnamed slot populated after /me failure");
    let oauth2 = token.oauth2.as_ref().unwrap();
    assert_eq!(oauth2.access_token, "salvage-access");
    assert_eq!(oauth2.refresh_token, "salvage-refresh");

    // Named map shape is unchanged: same keys, same (stale) values.
    let named_after: Vec<String> = auth.token_store.get_oauth2_usernames();
    assert_eq!(
        named_before, named_after,
        "named map keys unchanged when /me fails"
    );
    let stale = auth.token_store.get_oauth2_token("placeholder").unwrap();
    let stale_oauth2 = stale.oauth2.as_ref().unwrap();
    assert_eq!(
        stale_oauth2.access_token, "old-access",
        "placeholder's named token untouched"
    );
}

#[test]
fn get_oauth2_header_named_caller_never_uses_unnamed_slot() {
    // KTD5: named caller never falls through to the unnamed slot.
    // We seed ONLY the unnamed slot and a fresh-token mock so any successful
    // return would have to come from either named-by-name (miss),
    // get_first_oauth2_token_for_app (miss; named map is empty AND default_user
    // is empty), or the unnamed slot (forbidden for named callers).
    // With all three paths empty/forbidden, the function must fall through
    // to `oauth2_flow`, which will attempt to open a browser and fail in the
    // test environment. We assert the call returns an Err whose message does
    // NOT include the unnamed token's "salvage-bearer" string.
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);

    auth.token_store
        .save_oauth2_token_unnamed_for_app(
            "default",
            "salvage-bearer",
            "salvage-refresh",
            u64::MAX, // not expired
        )
        .unwrap();

    let result = auth.get_oauth2_header("alice");
    let err = result.expect_err(
        "named caller with no named token must NOT return the unnamed slot's Bearer; \
         it must trigger the OAuth2 flow (which fails in the test env)",
    );
    let msg = err.to_string();
    assert!(
        !msg.contains("salvage-bearer"),
        "named caller leaked the unnamed slot's access token: {msg}"
    );

    // Unnamed slot untouched.
    let still = auth
        .token_store
        .get_oauth2_token_unnamed_for_app("default")
        .expect("unnamed slot still populated");
    assert_eq!(
        still.oauth2.as_ref().unwrap().access_token,
        "salvage-bearer"
    );
    let _ = ts;
}

#[test]
fn get_oauth2_header_empty_caller_with_default_user_returns_default_user_token_not_unnamed() {
    // KTD5: empty caller + default_user + unnamed populated -> default_user wins.
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);

    auth.token_store
        .save_oauth2_token("alice", "alice-bearer", "alice-refresh", u64::MAX)
        .unwrap();
    auth.token_store
        .set_default_user("default", "alice")
        .unwrap();
    auth.token_store
        .save_oauth2_token_unnamed_for_app("default", "unnamed-bearer", "unnamed-refresh", u64::MAX)
        .unwrap();

    let header = auth.get_oauth2_header("").unwrap();
    assert_eq!(
        header, "Bearer alice-bearer",
        "default_user must outrank the unnamed slot"
    );
    let _ = ts;
}

#[test]
fn get_oauth2_header_empty_caller_no_named_uses_unnamed() {
    // KTD5: empty caller + no named token + no default_user + unnamed populated
    // -> return the unnamed slot's Bearer.
    let ts = TestServer::new();
    let tmp = TempDir::new().unwrap();
    let mut auth = create_test_auth(ts.uri(), &tmp);

    auth.token_store
        .save_oauth2_token_unnamed_for_app(
            "default",
            "only-unnamed-bearer",
            "only-unnamed-refresh",
            u64::MAX,
        )
        .unwrap();

    let header = auth.get_oauth2_header("").unwrap();
    assert_eq!(
        header, "Bearer only-unnamed-bearer",
        "empty-caller falls back to unnamed when no named tokens exist"
    );
    let _ = ts;
}
