//! Ported from Go: auth/auth_test.go (266 LOC)
//!
//! Tests authentication logic: OAuth1 signing, OAuth2 token flow,
//! bearer tokens, credential resolution priority, nonce/timestamp generation,
//! URL encoding, code verifier/challenge generation.

use std::collections::BTreeMap;

use rstest::rstest;
use tempfile::TempDir;

use xurl::auth::Auth;
use xurl::auth::oauth1::{encode, generate_nonce, generate_timestamp};
use xurl::auth::oauth2::{generate_code_verifier_and_challenge, get_oauth2_scopes};
use xurl::config::Config;
use xurl::store::{App, TokenStore};

// ── Test helpers ───────────────────────────────────────────────────────────

fn test_config() -> Config {
    // `Config` has `pub(crate)` resolver fields that external test code cannot
    // name in a struct literal (or fill via `..Config::new()` from an external
    // crate); assign the public fields after `Config::new()`. The resolver
    // fields are overwritten by `Auth::new_with_store_path` anyway.
    let mut cfg = Config::new();
    cfg.client_id = "test-client-id".to_string();
    cfg.client_secret = "test-client-secret".to_string();
    cfg.redirect_uri = "http://localhost:8080/callback".to_string();
    cfg.auth_url = "https://x.com/i/oauth2/authorize".to_string();
    cfg.token_url = "https://api.x.com/2/oauth2/token".to_string();
    cfg.api_base_url = "https://api.x.com".to_string();
    cfg.info_url = "https://api.x.com/2/users/me".to_string();
    cfg.app_name = String::new();
    cfg
}

fn empty_config() -> Config {
    let mut cfg = Config::new();
    cfg.client_id = String::new();
    cfg.client_secret = String::new();
    cfg.redirect_uri = String::new();
    cfg.auth_url = String::new();
    cfg.token_url = String::new();
    cfg.api_base_url = String::new();
    cfg.info_url = String::new();
    cfg.app_name = String::new();
    cfg
}

fn create_temp_token_store() -> (TokenStore, TempDir) {
    let tmp = TempDir::new().expect("Failed to create temp directory");
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
            oauth1_token: None,
            bearer_token: None,
            unnamed_oauth2_token: None,
        },
    );

    (store, tmp)
}

// ── TestNewAuth ────────────────────────────────────────────────────────────

#[test]
fn test_new_auth() {
    let cfg = test_config();
    let auth = Auth::new(&cfg);

    // token_store() now returns &TokenStore directly (always valid)
    let _ = auth.token_store();
}

// ── TestWithTokenStore ─────────────────────────────────────────────────────

#[test]
fn test_with_token_store() {
    let cfg = test_config();
    let auth = Auth::new(&cfg);

    let (token_store, _tmp) = create_temp_token_store();
    let new_auth = auth.with_token_store(token_store);

    let _ = new_auth.token_store();
}

// ── TestBearerToken ────────────────────────────────────────────────────────

#[test]
fn test_bearer_token_no_token() {
    let cfg = empty_config();
    let auth = Auth::new(&cfg);
    let (token_store, _tmp) = create_temp_token_store();
    let auth = auth.with_token_store(token_store);

    // Test with no bearer token
    let result = auth.get_bearer_token_header();
    assert!(
        result.is_err(),
        "Expected error when no bearer token is set"
    );
}

#[test]
fn test_bearer_token_with_token() {
    let cfg = empty_config();
    let auth = Auth::new(&cfg);
    let (mut token_store, _tmp) = create_temp_token_store();

    token_store
        .save_bearer_token("test-bearer-token")
        .expect("Failed to save bearer token");

    let auth = auth.with_token_store(token_store);

    let header = auth
        .get_bearer_token_header()
        .expect("Failed to get bearer token");
    assert_eq!(header, "Bearer test-bearer-token");
}

// ── TestGenerateNonce ──────────────────────────────────────────────────────

#[test]
fn test_generate_nonce() {
    let nonce1 = generate_nonce();
    let nonce2 = generate_nonce();

    assert!(!nonce1.is_empty(), "Expected non-empty nonce");
    assert_ne!(nonce1, nonce2, "Expected different nonces");
}

// ── TestGenerateTimestamp ──────────────────────────────────────────────────

#[test]
fn test_generate_timestamp() {
    let timestamp = generate_timestamp();

    assert!(!timestamp.is_empty(), "Expected non-empty timestamp");

    for c in timestamp.chars() {
        assert!(
            c.is_ascii_digit(),
            "Expected timestamp to contain only digits, got {timestamp}"
        );
    }
}

// ── TestEncode ─────────────────────────────────────────────────────────────

#[rstest]
#[case("abc", "abc")]
#[case("a b c", "a+b+c")]
#[case("a+b+c", "a%2Bb%2Bc")]
#[case("a/b/c", "a%2Fb%2Fc")]
#[case("a?b=c", "a%3Fb%3Dc")]
#[case("a&b=c", "a%26b%3Dc")]
fn test_encode(#[case] input: &str, #[case] expected: &str) {
    let result = encode(input);
    assert_eq!(
        result, expected,
        "encode({input:?}) should return {expected:?}"
    );
}

// ── TestGenerateCodeVerifierAndChallenge ────────────────────────────────────

#[test]
fn test_generate_code_verifier_and_challenge() {
    let (verifier, challenge) = generate_code_verifier_and_challenge();

    assert!(!verifier.is_empty(), "Expected non-empty verifier");
    assert!(!challenge.is_empty(), "Expected non-empty challenge");
    assert_ne!(
        verifier, challenge,
        "Expected verifier and challenge to be different"
    );
}

// ── TestGetOAuth2Scopes ────────────────────────────────────────────────────

#[test]
fn test_get_oauth2_scopes() {
    let scopes = get_oauth2_scopes();

    assert!(!scopes.is_empty(), "Expected non-empty scopes");
    assert!(
        scopes.contains(&"tweet.read"),
        "Expected 'tweet.read' scope"
    );
    assert!(
        scopes.contains(&"users.read"),
        "Expected 'users.read' scope"
    );
}

// ── TestCredentialResolutionPriority ────────────────────────────────────────

#[test]
fn test_env_vars_take_priority_over_store() {
    let (mut token_store, _tmp) = create_temp_token_store();

    token_store.apps.get_mut("default").unwrap().client_id = "store-id".to_string();
    token_store.apps.get_mut("default").unwrap().client_secret = "store-secret".to_string();
    token_store.save_bearer_token("x").unwrap(); // force save

    let mut cfg = empty_config();
    cfg.client_id = "env-id".to_string();
    cfg.client_secret = "env-secret".to_string();
    let auth = Auth::new(&cfg).with_token_store(token_store);
    assert_eq!(auth.client_id(), "env-id");
    assert_eq!(auth.client_secret(), "env-secret");
}

#[test]
fn test_store_used_when_env_vars_empty() {
    let (mut token_store, _tmp) = create_temp_token_store();

    token_store.apps.get_mut("default").unwrap().client_id = "store-id".to_string();
    token_store.apps.get_mut("default").unwrap().client_secret = "store-secret".to_string();
    token_store.save_bearer_token("x").unwrap();

    // When env vars are empty, should fall back to the store's app credentials
    let app = token_store.resolve_app("");
    assert_eq!(app.client_id, "store-id");
    assert_eq!(app.client_secret, "store-secret");
}

// ── TestWithAppName ────────────────────────────────────────────────────────

#[test]
fn test_with_app_name() {
    let (mut token_store, _tmp) = create_temp_token_store();

    // Add a second app with different credentials
    token_store
        .add_app("other", "other-id", "other-secret")
        .unwrap();

    let cfg = empty_config();
    let mut auth = Auth::new(&cfg).with_token_store(token_store);

    // Initially no app override
    assert!(auth.client_id().is_empty());

    // Set app name — should pick up other app's credentials
    auth.with_app_name("other");
    assert_eq!(auth.client_id(), "other-id");
    assert_eq!(auth.client_secret(), "other-secret");
}

#[test]
fn test_with_app_name_nonexistent() {
    let (token_store, _tmp) = create_temp_token_store();

    let cfg = empty_config();
    let mut auth = Auth::new(&cfg).with_token_store(token_store);

    // Setting a nonexistent app name should not panic
    auth.with_app_name("doesnt-exist");
    assert!(auth.client_id().is_empty());
}

// ── TestOAuth1HeaderWithTokenStore ─────────────────────────────────────────

#[test]
fn test_oauth1_header_no_token_fails() {
    let (token_store, _tmp) = create_temp_token_store();

    let cfg = empty_config();
    let auth = Auth::new(&cfg).with_token_store(token_store);

    // No OAuth1 token — should fail
    let result = auth.get_oauth1_header("GET", "https://api.x.com/2/users/me", None);
    assert!(result.is_err());
}

#[test]
fn test_oauth1_header_with_token_succeeds() {
    let (mut token_store, _tmp) = create_temp_token_store();

    token_store
        .save_oauth1_tokens("at", "ts", "ck", "cs")
        .unwrap();

    let cfg = empty_config();
    let auth = Auth::new(&cfg).with_token_store(token_store);

    let header = auth
        .get_oauth1_header("GET", "https://api.x.com/2/users/me", None)
        .expect("Should succeed with OAuth1 token");
    assert!(header.contains("OAuth "));
    assert!(header.contains("oauth_consumer_key"));
}

// ── TestGetOAuth2HeaderNoToken ─────────────────────────────────────────────

#[test]
fn test_get_oauth2_header_no_token() {
    let (token_store, _tmp) = create_temp_token_store();

    // Verify that looking up a nonexistent user returns None
    let token = token_store.get_oauth2_token("nobody");
    assert!(token.is_none());
}

// ── Edge cases NOT covered in Go tests ─────────────────────────────────────

#[test]
fn test_nonce_length() {
    let nonce = generate_nonce();
    // Nonce should be non-empty
    assert!(!nonce.is_empty(), "Nonce should not be empty");
}

#[test]
fn test_timestamp_is_recent() {
    let timestamp = generate_timestamp();
    let ts: u64 = timestamp.parse().expect("Timestamp should be numeric");

    // Should be a Unix timestamp (seconds since epoch)
    // In 2026, this should be > 1_700_000_000
    assert!(ts > 1_700_000_000, "Timestamp seems too old: {ts}");
    assert!(
        ts < 2_000_000_000,
        "Timestamp seems too far in the future: {ts}"
    );
}

#[rstest]
#[case("", "")]
#[case("hello world", "hello+world")]
#[case("100%", "100%25")]
fn test_encode_edge_cases(#[case] input: &str, #[case] expected: &str) {
    let result = encode(input);
    assert_eq!(result, expected);
}

#[test]
fn test_oauth1_header_format() {
    let (mut token_store, _tmp) = create_temp_token_store();
    token_store
        .save_oauth1_tokens(
            "access-token",
            "token-secret",
            "consumer-key",
            "consumer-secret",
        )
        .unwrap();

    let cfg = empty_config();
    let auth = Auth::new(&cfg).with_token_store(token_store);

    let header = auth
        .get_oauth1_header("POST", "https://api.x.com/2/tweets", None)
        .unwrap();

    // Validate OAuth1 header contains required parameters
    assert!(header.starts_with("OAuth "));
    assert!(header.contains("oauth_consumer_key"));
    assert!(header.contains("oauth_nonce"));
    assert!(header.contains("oauth_signature"));
    assert!(header.contains("oauth_signature_method"));
    assert!(header.contains("oauth_timestamp"));
    assert!(header.contains("oauth_token"));
    assert!(header.contains("oauth_version"));
}

// ── Explicit store-path injection (U3) ─────────────────────────────────────
//
// `Auth::new_with_store_path` honours an explicit `TempDir` path so library
// tests need no `HOME` / `XDG_CONFIG_HOME` env-var mutation. Parallel-safe.

#[test]
fn test_new_with_store_path_honors_explicit_path() {
    let tmp = TempDir::new().expect("Failed to create temp directory");
    let store_path = tmp.path().join(".xurl");

    let cfg = test_config();
    let mut auth = Auth::new_with_store_path(&cfg, &store_path);

    auth.token_store
        .save_bearer_token("explicit-path-bearer")
        .expect("Failed to save bearer token");

    assert!(
        store_path.exists(),
        "Expected token store file at {store_path:?} after save"
    );

    let header = auth
        .get_bearer_token_header()
        .expect("Failed to read back bearer header");
    assert_eq!(header, "Bearer explicit-path-bearer");

    let reopened = Auth::new_with_store_path(&cfg, &store_path);
    let reopened_header = reopened
        .get_bearer_token_header()
        .expect("Failed to read bearer token from reopened store");
    assert_eq!(reopened_header, "Bearer explicit-path-bearer");
}

// ── U3: redirect URI single-source-of-truth on owned Config ───────────────
//
// `Auth::new_with_store_path` runs the three-level resolver (env > app-stored
// > built-in default) and writes the resolved value back into the owned
// `Config`. `Auth::with_app_name` re-runs the resolver. `Auth::redirect_uri()`
// returns the resolved value. Tests use `#[serial]` for the env-var leg
// because `REDIRECT_URI` is process-wide.

fn write_store_with_redirect_uri(path: &std::path::Path, app: &str, uri: &str) {
    let yaml = format!(
        "apps:\n  {app}:\n    client_id: ''\n    client_secret: ''\n    redirect_uri: '{uri}'\n    oauth2_tokens: {{}}\ndefault_app: {app}\n"
    );
    std::fs::write(path, yaml).expect("write tempdir store");
}

fn write_two_app_store(path: &std::path::Path, app_a: &str, uri_a: &str, app_b: &str, uri_b: &str) {
    let yaml = format!(
        "apps:\n  {app_a}:\n    client_id: ''\n    client_secret: ''\n    redirect_uri: '{uri_a}'\n    oauth2_tokens: {{}}\n  {app_b}:\n    client_id: ''\n    client_secret: ''\n    redirect_uri: '{uri_b}'\n    oauth2_tokens: {{}}\ndefault_app: {app_a}\n"
    );
    std::fs::write(path, yaml).expect("write tempdir store");
}

#[test]
#[serial_test::serial]
fn test_redirect_uri_env_wins_via_new_with_store_path() {
    let tmp = TempDir::new().expect("temp dir");
    let store_path = tmp.path().join(".xurl");
    write_store_with_redirect_uri(&store_path, "default", "http://localhost:7777/cb");

    let cfg = empty_config();
    unsafe {
        std::env::set_var("REDIRECT_URI", "https://example.com/cb");
    }
    let auth = Auth::new_with_store_path(&cfg, &store_path);
    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    assert_eq!(auth.redirect_uri(), "https://example.com/cb");
}

#[test]
#[serial_test::serial]
fn test_redirect_uri_stored_wins_when_env_unset() {
    let tmp = TempDir::new().expect("temp dir");
    let store_path = tmp.path().join(".xurl");
    write_store_with_redirect_uri(&store_path, "default", "http://localhost:9090/cb");

    let cfg = empty_config();
    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }
    let auth = Auth::new_with_store_path(&cfg, &store_path);

    assert_eq!(auth.redirect_uri(), "http://localhost:9090/cb");
}

#[test]
#[serial_test::serial]
fn test_redirect_uri_falls_back_to_default_when_no_env_and_no_stored() {
    let tmp = TempDir::new().expect("temp dir");
    let store_path = tmp.path().join(".xurl");
    // Store with the default app but no stored redirect_uri (empty).
    let yaml = "apps:\n  default:\n    client_id: ''\n    client_secret: ''\n    oauth2_tokens: {}\ndefault_app: default\n";
    std::fs::write(&store_path, yaml).expect("write store");

    let cfg = empty_config();
    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }
    let auth = Auth::new_with_store_path(&cfg, &store_path);

    assert_eq!(auth.redirect_uri(), "http://localhost:8080/callback");
}

#[test]
#[serial_test::serial]
fn test_with_app_name_re_resolves_per_app_stored_uri() {
    let tmp = TempDir::new().expect("temp dir");
    let store_path = tmp.path().join(".xurl");
    write_two_app_store(
        &store_path,
        "alpha",
        "http://localhost:7001/cb",
        "beta",
        "http://localhost:7002/cb",
    );

    let cfg = empty_config();
    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }
    let mut auth = Auth::new_with_store_path(&cfg, &store_path);

    // Default app is "alpha" — resolver picks alpha's stored URI.
    assert_eq!(auth.redirect_uri(), "http://localhost:7001/cb");

    // Switch to beta; resolver re-runs and returns beta's stored URI.
    auth.with_app_name("beta");
    assert_eq!(auth.redirect_uri(), "http://localhost:7002/cb");

    // Switch back to alpha; resolver re-runs again.
    auth.with_app_name("alpha");
    assert_eq!(auth.redirect_uri(), "http://localhost:7001/cb");
}

#[test]
#[serial_test::serial]
fn test_with_app_name_env_override_survives_app_switch() {
    let tmp = TempDir::new().expect("temp dir");
    let store_path = tmp.path().join(".xurl");
    write_two_app_store(
        &store_path,
        "alpha",
        "http://localhost:7001/cb",
        "beta",
        "http://localhost:7002/cb",
    );

    let cfg = empty_config();
    unsafe {
        std::env::set_var("REDIRECT_URI", "https://envvar.example.com/cb");
    }
    let mut auth = Auth::new_with_store_path(&cfg, &store_path);

    // Env wins for the default app.
    assert_eq!(auth.redirect_uri(), "https://envvar.example.com/cb");

    // Env still wins after switching apps — KTD3 forbids the credential's
    // "preserve if non-empty" pattern; the resolver itself enforces env
    // precedence each time.
    auth.with_app_name("beta");
    let still_env_after_switch = auth.redirect_uri().to_string();

    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    assert_eq!(still_env_after_switch, "https://envvar.example.com/cb");
}
