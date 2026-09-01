//! Ported from Go: auth/auth_test.go (266 LOC)
//!
//! Tests authentication logic: OAuth1 signing, OAuth2 token flow,
//! bearer tokens, credential resolution priority, nonce/timestamp generation,
//! URL encoding, code verifier/challenge generation.

use std::collections::BTreeMap;

use rstest::rstest;
use tempfile::TempDir;

use xurl::auth::oauth1::{encode, generate_nonce, generate_timestamp};
use xurl::auth::oauth2::{generate_code_verifier_and_challenge, get_oauth2_scopes};
use xurl::auth::{Auth, resolve_bearer_token};
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

#[serial_test::parallel]
#[test]
fn test_new_auth() {
    let cfg = test_config();
    let auth = Auth::new(&cfg);

    // token_store() now returns &TokenStore directly (always valid)
    let _ = auth.token_store();
}

// ── TestWithTokenStore ─────────────────────────────────────────────────────

#[serial_test::parallel]
#[test]
fn test_with_token_store() {
    let cfg = test_config();
    let auth = Auth::new(&cfg);

    let (token_store, _tmp) = create_temp_token_store();
    let new_auth = auth.with_token_store(token_store);

    let _ = new_auth.token_store();
}

// ── TestBearerToken ────────────────────────────────────────────────────────

#[serial_test::parallel]
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

#[serial_test::parallel]
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

#[serial_test::parallel]
#[test]
fn test_generate_nonce() {
    let nonce1 = generate_nonce();
    let nonce2 = generate_nonce();

    assert!(!nonce1.is_empty(), "Expected non-empty nonce");
    assert_ne!(nonce1, nonce2, "Expected different nonces");
}

// ── TestGenerateTimestamp ──────────────────────────────────────────────────

#[serial_test::parallel]
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

#[serial_test::parallel]
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

#[serial_test::parallel]
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

#[serial_test::parallel]
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

#[serial_test::parallel]
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

#[serial_test::parallel]
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

#[serial_test::parallel]
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

#[serial_test::parallel]
#[test]
fn test_oauth1_header_no_token_fails() {
    let (token_store, _tmp) = create_temp_token_store();

    let cfg = empty_config();
    let auth = Auth::new(&cfg).with_token_store(token_store);

    // No OAuth1 token — should fail
    let result = auth.get_oauth1_header("GET", "https://api.x.com/2/users/me", None);
    assert!(result.is_err());
}

#[serial_test::parallel]
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

#[serial_test::parallel]
#[test]
fn test_get_oauth2_header_no_token() {
    let (token_store, _tmp) = create_temp_token_store();

    // Verify that looking up a nonexistent user returns None
    let token = token_store.get_oauth2_token("nobody");
    assert!(token.is_none());
}

// ── Edge cases NOT covered in Go tests ─────────────────────────────────────

#[serial_test::parallel]
#[test]
fn test_nonce_length() {
    let nonce = generate_nonce();
    // Nonce should be non-empty
    assert!(!nonce.is_empty(), "Nonce should not be empty");
}

#[serial_test::parallel]
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

#[serial_test::parallel]
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

#[serial_test::parallel]
#[test]
fn test_new_with_store_path_honors_explicit_path() {
    let tmp = TempDir::new().expect("Failed to create temp directory");
    let store_path = tmp.path().join(".xurl");

    let cfg = test_config();
    let mut auth = Auth::new_with_store_path_and_overrides(
        &cfg,
        &store_path,
        &xurl::config::EnvOverrides::default(),
    );

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
    // ALLOWLISTED ENV MUTATION (see tests/env_mutation_guard.rs).
    //
    // This is the proof that `new_with_store_path` reads the process at all.
    // Every other redirect-URI test here injects through
    // `new_with_store_path_and_overrides`; if this one injected too, nothing
    // would cover the shim.
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
    let auth = Auth::new_with_store_path_and_overrides(
        &cfg,
        &store_path,
        &xurl::config::EnvOverrides::default(),
    );

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
    let auth = Auth::new_with_store_path_and_overrides(
        &cfg,
        &store_path,
        &xurl::config::EnvOverrides::default(),
    );

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
    let mut auth = Auth::new_with_store_path_and_overrides(
        &cfg,
        &store_path,
        &xurl::config::EnvOverrides::default(),
    );

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
    let overrides = xurl::config::EnvOverrides {
        redirect_uri: Some("https://envvar.example.com/cb".into()),
        ..xurl::config::EnvOverrides::default()
    };
    let mut auth = Auth::new_with_store_path_and_overrides(&cfg, &store_path, &overrides);

    // Env wins for the default app.
    assert_eq!(auth.redirect_uri(), "https://envvar.example.com/cb");

    // Env still wins after switching apps — KTD3 forbids the credential's
    // "preserve if non-empty" pattern; the resolver itself enforces env
    // precedence each time.
    auth.with_app_name("beta");
    let still_env_after_switch = auth.redirect_uri().to_string();

    assert_eq!(still_env_after_switch, "https://envvar.example.com/cb");
}

// ── TestResolveBearerToken (env fallback for bearer auth) ──────────────────
//
// `resolve_bearer_token` is the pure resolver factored out of
// `Auth::get_bearer_token_header`. The unit tests exercise every precedence
// branch without touching the process environment, per the project's
// no-env-mutation test-isolation policy. One integration-style test below
// (`get_bearer_token_header_reads_real_env`) explicitly sets the env var
// behind an `unsafe { set_var }` guard to verify the production wrapper.

#[serial_test::parallel]
#[test]
fn resolve_bearer_returns_env_when_set_and_store_empty() {
    let (token_store, _tmp) = create_temp_token_store();
    let header = resolve_bearer_token(Some("env-only-bearer".to_string()), &token_store, "")
        .expect("env token should resolve");
    assert_eq!(header, "Bearer env-only-bearer");
}

#[serial_test::parallel]
#[test]
fn resolve_bearer_env_overrides_stored_bearer() {
    let (mut token_store, _tmp) = create_temp_token_store();
    token_store
        .save_bearer_token("stored-bearer")
        .expect("save_bearer_token must succeed");

    let header = resolve_bearer_token(Some("env-wins".to_string()), &token_store, "")
        .expect("env token should resolve");
    assert_eq!(
        header, "Bearer env-wins",
        "env-supplied bearer must win over stored bearer"
    );
}

#[serial_test::parallel]
#[test]
fn resolve_bearer_falls_back_to_store_when_env_empty() {
    let (mut token_store, _tmp) = create_temp_token_store();
    token_store
        .save_bearer_token("stored-bearer")
        .expect("save_bearer_token must succeed");

    let header = resolve_bearer_token(Some(String::new()), &token_store, "")
        .expect("empty env should fall through to store");
    assert_eq!(
        header, "Bearer stored-bearer",
        "Some(\"\") env must be treated as unset and fall through"
    );
}

#[serial_test::parallel]
#[test]
fn resolve_bearer_falls_back_to_store_when_env_unset() {
    let (mut token_store, _tmp) = create_temp_token_store();
    token_store
        .save_bearer_token("stored-bearer")
        .expect("save_bearer_token must succeed");

    let header = resolve_bearer_token(None, &token_store, "").expect("store should resolve");
    assert_eq!(header, "Bearer stored-bearer");
}

#[serial_test::parallel]
#[test]
fn resolve_bearer_errors_when_neither_set() {
    let (token_store, _tmp) = create_temp_token_store();
    let err =
        resolve_bearer_token(None, &token_store, "").expect_err("no env, no store should error");
    assert!(
        err.to_string().contains("bearer token not found"),
        "expected TokenNotFound message, got: {err}"
    );
}

#[serial_test::parallel]
#[test]
fn resolve_bearer_errors_when_env_empty_and_store_empty() {
    let (token_store, _tmp) = create_temp_token_store();
    let err = resolve_bearer_token(Some(String::new()), &token_store, "")
        .expect_err("empty env + empty store should error");
    assert!(err.to_string().contains("bearer token not found"));
}

#[serial_test::serial]
#[test]
fn get_bearer_token_header_reads_real_env() {
    // Production wrapper covers the path that pulls from `std::env`. Mutates
    // env behind an `unsafe` guard to mirror the existing
    // `test_env_redirect_uri_wins_over_app_stored` pattern. The var name is
    // unique enough that parallel-test contamination is unlikely; the cleanup
    // is unconditional so a panic mid-test still restores process state.
    let cfg = empty_config();
    let (token_store, _tmp) = create_temp_token_store();

    // The process read happens when `Auth` is constructed, not when the
    // header is requested, so the variable is exported first. A run resolves
    // its environment once at the entrypoint and carries it, which is what
    // makes every other test in this file able to inject instead of export.
    unsafe {
        std::env::set_var("XURL_BEARER_TOKEN", "integration-env-bearer");
    }
    let auth = Auth::new(&cfg).with_token_store(token_store);
    let header_with_env = auth.get_bearer_token_header();
    unsafe {
        std::env::remove_var("XURL_BEARER_TOKEN");
    }

    let header = header_with_env.expect("env-supplied bearer should resolve");
    assert_eq!(header, "Bearer integration-env-bearer");
}

// ── TestWithAppName (client_id env precedence across app switches) ────────
//
// The `client_id_from_env` / `client_secret_from_env` flags on `Auth` track
// where credentials originally came from so a later `with_app_name` switch
// honors env precedence correctly. Without the flags, the older
// "preserve if non-empty" check could not distinguish env-supplied values
// from values loaded off the previous app's store entry, so subsequent
// `--app NAME` switches silently re-used the previous app's stored
// client_id (the user-facing bug surfaced during v1.3.0 preflight smoke).

fn token_store_with_two_apps(
    default_id: &str,
    second_name: &str,
    second_id: &str,
) -> (TokenStore, TempDir) {
    let (mut ts, tmp) = create_temp_token_store();
    ts.apps
        .get_mut("default")
        .expect("default app must exist")
        .client_id = default_id.to_string();
    ts.apps
        .get_mut("default")
        .expect("default app must exist")
        .client_secret = format!("{default_id}-secret");
    ts.apps.insert(
        second_name.to_string(),
        App {
            client_id: second_id.to_string(),
            client_secret: format!("{second_id}-secret"),
            default_user: String::new(),
            redirect_uri: String::new(),
            oauth2_tokens: BTreeMap::new(),
            oauth1_token: None,
            bearer_token: None,
            unnamed_oauth2_token: None,
        },
    );
    (ts, tmp)
}

#[serial_test::parallel]
#[test]
fn with_app_name_loads_new_app_client_id_when_no_env() {
    // cfg.client_id empty => Auth picks default app's "default-id" from store.
    // After `with_app_name("bird-dev")`, client_id must be re-resolved to
    // "bird-id", not silently retained as "default-id".
    let cfg = empty_config();
    let (token_store, _tmp) = token_store_with_two_apps("default-id", "bird-dev", "bird-id");
    let mut auth = Auth::new(&cfg).with_token_store(token_store);

    assert_eq!(
        auth.client_id(),
        "default-id",
        "Auth must load default app's id"
    );

    auth.with_app_name("bird-dev");
    assert_eq!(
        auth.client_id(),
        "bird-id",
        "with_app_name must re-resolve client_id from the new app's store entry"
    );
}

#[serial_test::parallel]
#[test]
fn with_app_name_loads_new_app_client_secret_when_no_env() {
    let cfg = empty_config();
    let (token_store, _tmp) = token_store_with_two_apps("default-id", "bird-dev", "bird-id");
    let mut auth = Auth::new(&cfg).with_token_store(token_store);

    auth.with_app_name("bird-dev");
    assert_eq!(auth.client_secret(), "bird-id-secret");
}

#[serial_test::parallel]
#[test]
fn with_app_name_preserves_env_supplied_client_id_across_switches() {
    // cfg.client_id non-empty => env-supplied. After switching to bird-dev,
    // the env value must remain even though bird-dev's stored value is
    // different. The old "preserve if non-empty" check happened to satisfy
    // this case, but the explicit flag makes the invariant load-bearing.
    let mut cfg = empty_config();
    cfg.client_id = "from-env".to_string();
    cfg.client_secret = "secret-from-env".to_string();

    let (token_store, _tmp) = token_store_with_two_apps("default-id", "bird-dev", "bird-id");
    let mut auth = Auth::new(&cfg).with_token_store(token_store);

    assert_eq!(auth.client_id(), "from-env");
    assert_eq!(auth.client_secret(), "secret-from-env");

    auth.with_app_name("bird-dev");
    assert_eq!(
        auth.client_id(),
        "from-env",
        "env-supplied client_id must survive a `with_app_name` switch"
    );
    assert_eq!(
        auth.client_secret(),
        "secret-from-env",
        "env-supplied client_secret must survive a `with_app_name` switch"
    );
}

#[serial_test::parallel]
#[test]
fn with_app_name_back_to_default_re_resolves_from_default_app() {
    // Round-trip: default => bird-dev => default. Without the env flag,
    // step three would retain bird-dev's "bird-id" because of the old
    // "if empty" check. The fix re-resolves correctly.
    let cfg = empty_config();
    let (token_store, _tmp) = token_store_with_two_apps("default-id", "bird-dev", "bird-id");
    let mut auth = Auth::new(&cfg).with_token_store(token_store);

    auth.with_app_name("bird-dev");
    assert_eq!(auth.client_id(), "bird-id");

    auth.with_app_name("default");
    assert_eq!(
        auth.client_id(),
        "default-id",
        "switching back to default must re-resolve to default's stored id"
    );
}

// ── TestRefreshOauth2TokenAppContext (Bug A) ──────────────────────────────
//
// `oauth2::refresh_oauth2_token` must use the active app context when
// looking up the cached token. The older `get_first_oauth2_token()` call
// (no arg) resolved to the empty-string app name, which `resolve_app` falls
// back to the default app for — so a token freshly minted under a named
// app via `xr auth oauth2 --app NAME` was invisible to the subsequent
// refresh path, and the request went out without a token. The integration
// surface for this bug fires only when the token is still valid and the
// refresh becomes a fast no-op return of the cached access_token.

#[serial_test::parallel]
#[test]
fn refresh_finds_named_app_token_when_active_app_set() {
    use xurl::auth::oauth2::refresh_oauth2_token;

    let (mut store, _tmp) = create_temp_token_store();
    store
        .add_app("bird-dev", "id", "secret")
        .expect("add bird-dev");
    // Stash a token under bird-dev with a far-future expiration so the
    // refresh path returns the cached value directly (no HTTP).
    let far_future = 9_999_999_999;
    store
        .save_oauth2_token_for_app(
            "bird-dev",
            "alice",
            "alice-access",
            "alice-refresh",
            far_future,
        )
        .expect("save");

    // Default app stays uninitialized; the bug's symptom was the refresh
    // path resolving lookups to the default app and missing the bird-dev
    // token entirely.
    let cfg = empty_config();
    let mut auth = Auth::new(&cfg).with_token_store(store);
    auth.with_app_name("bird-dev");

    let token =
        refresh_oauth2_token(&mut auth, "alice").expect("refresh must find named app token");
    assert_eq!(
        token, "alice-access",
        "refresh must read alice's token from bird-dev, not the empty default"
    );
}

#[serial_test::parallel]
#[test]
fn refresh_finds_first_token_in_named_app_when_username_empty() {
    use xurl::auth::oauth2::refresh_oauth2_token;

    let (mut store, _tmp) = create_temp_token_store();
    store
        .add_app("bird-dev", "id", "secret")
        .expect("add bird-dev");
    let far_future = 9_999_999_999;
    store
        .save_oauth2_token_for_app("bird-dev", "user1", "u1-token", "u1-ref", far_future)
        .expect("save");

    let cfg = empty_config();
    let mut auth = Auth::new(&cfg).with_token_store(store);
    auth.with_app_name("bird-dev");

    // Empty username = "use the first token in the active app". The fix
    // routes this lookup through bird-dev instead of falling back to the
    // empty default app.
    let token = refresh_oauth2_token(&mut auth, "").expect("refresh with empty username");
    assert_eq!(token, "u1-token");
}

#[serial_test::parallel]
#[test]
fn refresh_falls_back_to_unnamed_slot_within_active_app() {
    use xurl::auth::oauth2::refresh_oauth2_token;

    let (mut store, _tmp) = create_temp_token_store();
    store
        .add_app("bird-dev", "id", "secret")
        .expect("add bird-dev");
    let far_future = 9_999_999_999;
    // Salvage-state: no named users, only the unnamed slot has a token.
    store
        .save_oauth2_token_unnamed_for_app("bird-dev", "unnamed-tok", "unnamed-ref", far_future)
        .expect("save unnamed");

    let cfg = empty_config();
    let mut auth = Auth::new(&cfg).with_token_store(store);
    auth.with_app_name("bird-dev");

    let token = refresh_oauth2_token(&mut auth, "").expect("refresh must fall back to unnamed");
    assert_eq!(token, "unnamed-tok");
}

// ── TestMultiAppCredentialRouting (Bugs D, E, F) ──────────────────────────
//
// Multi-app isolation: each app should own its own OAuth1, OAuth2, and
// bearer credentials, and `Auth`'s `--app NAME`-driven `app_name` field
// must scope every read and write. The legacy code paths used the
// store's no-arg accessors, which `resolve_app("")` redirected to the
// default app, so `--app NAME --auth <method>` silently fell back to the
// default app for OAuth1, bearer, and auto-detect.
//
// The OAuth2 refresh-path fix in `refresh_finds_named_app_token_when_active_app_set`
// lives above; these tests cover the parallel cases for OAuth1, bearer,
// and the request layer's auto-detect.

#[serial_test::parallel]
#[test]
fn bearer_resolution_targets_active_app_not_default() {
    use xurl::auth::resolve_bearer_token;

    let (mut store, _tmp) = create_temp_token_store();
    store.add_app("alpha", "a-id", "a-secret").unwrap();
    store.add_app("beta", "b-id", "b-secret").unwrap();
    store
        .save_bearer_token_for_app("alpha", "alpha-bearer")
        .unwrap();
    store
        .save_bearer_token_for_app("beta", "beta-bearer")
        .unwrap();

    // Default app stays uninitialized; the legacy resolver would have
    // ignored the `app_name` argument and resolved to default, returning
    // TokenNotFound here.
    let alpha_header = resolve_bearer_token(None, &store, "alpha").expect("alpha header");
    assert_eq!(alpha_header, "Bearer alpha-bearer");

    let beta_header = resolve_bearer_token(None, &store, "beta").expect("beta header");
    assert_eq!(beta_header, "Bearer beta-bearer");
}

#[serial_test::parallel]
#[test]
fn bearer_env_var_overrides_active_app_store() {
    use xurl::auth::resolve_bearer_token;

    let (mut store, _tmp) = create_temp_token_store();
    store.add_app("alpha", "id", "secret").unwrap();
    store
        .save_bearer_token_for_app("alpha", "alpha-bearer")
        .unwrap();

    // Env wins over any app's stored bearer regardless of app_name scope.
    let header = resolve_bearer_token(Some("env-wins".to_string()), &store, "alpha")
        .expect("env should override");
    assert_eq!(header, "Bearer env-wins");
}

#[serial_test::parallel]
#[test]
fn oauth1_header_routes_to_active_app() {
    let (mut store, _tmp) = create_temp_token_store();
    store.add_app("alpha", "a-id", "a-secret").unwrap();
    store
        .save_oauth1_tokens_for_app(
            "alpha",
            "alpha-access-tok",
            "alpha-secret",
            "alpha-consumer-key",
            "alpha-consumer-secret",
        )
        .unwrap();

    let cfg = empty_config();
    let mut auth = Auth::new(&cfg).with_token_store(store);
    auth.with_app_name("alpha");

    // OAuth1 header construction reaches into the active app's stored
    // token; pre-fix it would have hit the empty default app and errored.
    let header = auth
        .get_oauth1_header("GET", "https://api.x.com/2/users/me", None)
        .expect("alpha must have OAuth1");
    assert!(
        header.starts_with("OAuth "),
        "expected OAuth1 header prefix, got {header}"
    );
    assert!(
        header.contains("alpha-consumer-key"),
        "expected consumer-key threaded into header, got {header}"
    );
}

#[serial_test::parallel]
#[test]
fn oauth1_header_errors_when_active_app_has_no_token() {
    let (mut store, _tmp) = create_temp_token_store();
    store.add_app("alpha", "a-id", "a-secret").unwrap();
    // alpha has no OAuth1 tokens; default also has none. Active app is alpha.
    let cfg = empty_config();
    let mut auth = Auth::new(&cfg).with_token_store(store);
    auth.with_app_name("alpha");

    let err = auth
        .get_oauth1_header("GET", "https://api.x.com/2/users/me", None)
        .expect_err("should error rather than fall back to default");
    assert!(
        err.to_string().contains("OAuth1 token not found"),
        "expected TokenNotFound on the active app, got {err}"
    );
}

#[serial_test::parallel]
#[test]
fn bearer_header_via_auth_routes_to_active_app() {
    let (mut store, _tmp) = create_temp_token_store();
    store.add_app("alpha", "a-id", "a-secret").unwrap();
    store.add_app("beta", "b-id", "b-secret").unwrap();
    store
        .save_bearer_token_for_app("alpha", "alpha-bearer")
        .unwrap();
    store
        .save_bearer_token_for_app("beta", "beta-bearer")
        .unwrap();

    let cfg = empty_config();
    let mut auth = Auth::new(&cfg).with_token_store(store);
    auth.with_app_name("alpha");
    assert_eq!(
        auth.get_bearer_token_header().expect("alpha bearer"),
        "Bearer alpha-bearer"
    );

    auth.with_app_name("beta");
    assert_eq!(
        auth.get_bearer_token_header().expect("beta bearer"),
        "Bearer beta-bearer"
    );
}

#[serial_test::parallel]
#[test]
fn switching_apps_at_runtime_resolves_each_apps_oauth2_token() {
    use xurl::auth::oauth2::refresh_oauth2_token;

    let (mut store, _tmp) = create_temp_token_store();
    store.add_app("alpha", "a-id", "a-secret").unwrap();
    store.add_app("beta", "b-id", "b-secret").unwrap();
    let far_future = 9_999_999_999;
    store
        .save_oauth2_token_for_app("alpha", "alice", "alpha-tok", "alpha-ref", far_future)
        .unwrap();
    store
        .save_oauth2_token_for_app("beta", "bob", "beta-tok", "beta-ref", far_future)
        .unwrap();

    let cfg = empty_config();
    let mut auth = Auth::new(&cfg).with_token_store(store);

    auth.with_app_name("alpha");
    assert_eq!(
        refresh_oauth2_token(&mut auth, "alice").expect("alpha/alice"),
        "alpha-tok"
    );

    auth.with_app_name("beta");
    assert_eq!(
        refresh_oauth2_token(&mut auth, "bob").expect("beta/bob"),
        "beta-tok"
    );

    // Cross-check: alice does NOT exist in beta. Without the fix, the
    // refresh path would have resolved to default (no tokens) instead of
    // failing in beta — both arrive at "not found", but for the right
    // reason now.
    auth.with_app_name("beta");
    let err = refresh_oauth2_token(&mut auth, "alice").expect_err("alice not in beta");
    assert!(err.to_string().contains("oauth2 token not found"));
}
