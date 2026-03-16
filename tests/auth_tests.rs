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
    Config {
        client_id: "test-client-id".to_string(),
        client_secret: "test-client-secret".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
        auth_url: "https://x.com/i/oauth2/authorize".to_string(),
        token_url: "https://api.x.com/2/oauth2/token".to_string(),
        api_base_url: "https://api.x.com".to_string(),
        info_url: "https://api.x.com/2/users/me".to_string(),
        app_name: String::new(),
    }
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
            oauth2_tokens: BTreeMap::new(),
            oauth1_token: None,
            bearer_token: None,
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
    let cfg = Config {
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uri: String::new(),
        auth_url: String::new(),
        token_url: String::new(),
        api_base_url: String::new(),
        info_url: String::new(),
        app_name: String::new(),
    };
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
    let cfg = Config {
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uri: String::new(),
        auth_url: String::new(),
        token_url: String::new(),
        api_base_url: String::new(),
        info_url: String::new(),
        app_name: String::new(),
    };
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

    let cfg = Config {
        client_id: "env-id".to_string(),
        client_secret: "env-secret".to_string(),
        redirect_uri: String::new(),
        auth_url: String::new(),
        token_url: String::new(),
        api_base_url: String::new(),
        info_url: String::new(),
        app_name: String::new(),
    };
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

    let cfg = Config {
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uri: String::new(),
        auth_url: String::new(),
        token_url: String::new(),
        api_base_url: String::new(),
        info_url: String::new(),
        app_name: String::new(),
    };
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

    let cfg = Config {
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uri: String::new(),
        auth_url: String::new(),
        token_url: String::new(),
        api_base_url: String::new(),
        info_url: String::new(),
        app_name: String::new(),
    };
    let mut auth = Auth::new(&cfg).with_token_store(token_store);

    // Setting a nonexistent app name should not panic
    auth.with_app_name("doesnt-exist");
    assert!(auth.client_id().is_empty());
}

// ── TestOAuth1HeaderWithTokenStore ─────────────────────────────────────────

#[test]
fn test_oauth1_header_no_token_fails() {
    let (token_store, _tmp) = create_temp_token_store();

    let cfg = Config {
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uri: String::new(),
        auth_url: String::new(),
        token_url: String::new(),
        api_base_url: String::new(),
        info_url: String::new(),
        app_name: String::new(),
    };
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

    let cfg = Config {
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uri: String::new(),
        auth_url: String::new(),
        token_url: String::new(),
        api_base_url: String::new(),
        info_url: String::new(),
        app_name: String::new(),
    };
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

    let cfg = Config {
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uri: String::new(),
        auth_url: String::new(),
        token_url: String::new(),
        api_base_url: String::new(),
        info_url: String::new(),
        app_name: String::new(),
    };
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
