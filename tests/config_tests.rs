//! Tests for Config loading and environment variable handling.
//!
//! Most tests here inject values through `EnvOverrides` and run in parallel.
//! The ones proving `Config::new`'s own contract read the process, so they
//! mutate the environment and stay `#[serial]`; `tests/env_mutation_guard.rs`
//! is the allowlist that keeps that set from growing by accident.

use serial_test::serial;
use xurl::config::{Config, EnvOverrides};

#[test]
#[serial]
fn test_config_defaults() {
    // Clear any env vars that might interfere
    for key in &[
        "CLIENT_ID",
        "CLIENT_SECRET",
        "REDIRECT_URI",
        "AUTH_URL",
        "TOKEN_URL",
        "API_BASE_URL",
        "INFO_URL",
    ] {
        unsafe {
            std::env::remove_var(key);
        }
    }

    let cfg = Config::new();

    assert_eq!(cfg.client_id, "", "Default client_id should be empty");
    assert_eq!(
        cfg.client_secret, "",
        "Default client_secret should be empty"
    );
    assert_eq!(cfg.redirect_uri, "http://localhost:8080/callback");
    assert_eq!(cfg.auth_url, "https://x.com/i/oauth2/authorize");
    assert_eq!(cfg.token_url, "https://api.x.com/2/oauth2/token");
    assert_eq!(cfg.api_base_url, "https://api.x.com");
    assert!(cfg.info_url.contains("/2/users/me"));
    assert_eq!(cfg.app_name, "");
}

#[test]
#[serial]
fn test_config_from_env_client_id() {
    unsafe {
        std::env::set_var("CLIENT_ID", "env-test-id");
    }
    let cfg = Config::new();
    assert_eq!(cfg.client_id, "env-test-id");
    unsafe {
        std::env::remove_var("CLIENT_ID");
    }
}

#[test]
#[serial]
fn test_config_from_env_client_secret() {
    unsafe {
        std::env::set_var("CLIENT_SECRET", "env-test-secret");
    }
    let cfg = Config::new();
    assert_eq!(cfg.client_secret, "env-test-secret");
    unsafe {
        std::env::remove_var("CLIENT_SECRET");
    }
}

#[test]
#[serial]
fn test_config_from_env_all() {
    unsafe {
        std::env::set_var("CLIENT_ID", "all-id");
        std::env::set_var("CLIENT_SECRET", "all-secret");
        std::env::set_var("API_BASE_URL", "https://test.example.com");
    }

    let cfg = Config::new();
    assert_eq!(cfg.client_id, "all-id");
    assert_eq!(cfg.client_secret, "all-secret");
    assert_eq!(cfg.api_base_url, "https://test.example.com");

    unsafe {
        std::env::remove_var("CLIENT_ID");
        std::env::remove_var("CLIENT_SECRET");
        std::env::remove_var("API_BASE_URL");
    }
}

#[test]
#[serial]
fn test_config_from_env_api_base_url() {
    unsafe {
        std::env::set_var("API_BASE_URL", "https://custom.api.example.com");
    }
    let cfg = Config::new();
    assert_eq!(cfg.api_base_url, "https://custom.api.example.com");
    // info_url should be derived from api_base_url
    assert!(cfg.info_url.starts_with("https://custom.api.example.com"));
    unsafe {
        std::env::remove_var("API_BASE_URL");
    }
}

#[test]
#[serial]
fn test_config_default_trait() {
    // Config implements Default (which calls new())
    let cfg = Config::default();
    assert!(!cfg.redirect_uri.is_empty());
    assert!(!cfg.auth_url.is_empty());
}

// Resolver thin-wrapper precedence tests live inline in src/config/mod.rs
// because `ResolvedRedirectUri` is `pub(crate)` per KTD9 and integration-test
// crates cannot reach it. The validator-only tests below use the public API.

// ── Config::from_overrides ──────────────────────────────────────────────────

#[test]
fn test_from_overrides_empty_yields_builtin_defaults() {
    let cfg = Config::from_overrides(&EnvOverrides::default());

    assert_eq!(cfg.client_id, "");
    assert_eq!(cfg.client_secret, "");
    assert_eq!(cfg.redirect_uri, "http://localhost:8080/callback");
    assert_eq!(cfg.auth_url, "https://x.com/i/oauth2/authorize");
    assert_eq!(cfg.token_url, "https://api.x.com/2/oauth2/token");
    assert_eq!(cfg.api_base_url, "https://api.x.com");
    assert_eq!(cfg.info_url, "https://api.x.com/2/users/me");
    assert_eq!(cfg.app_name, "");
}

#[test]
fn test_from_overrides_info_url_derives_from_supplied_api_base() {
    let cfg = Config::from_overrides(&EnvOverrides {
        api_base_url: Some("http://127.0.0.1:9999".into()),
        ..EnvOverrides::default()
    });

    assert_eq!(cfg.api_base_url, "http://127.0.0.1:9999");
    assert_eq!(
        cfg.info_url, "http://127.0.0.1:9999/2/users/me",
        "info_url derives from the supplied base when not set explicitly"
    );
}

#[test]
fn test_from_overrides_explicit_info_url_wins_over_derivation() {
    let cfg = Config::from_overrides(&EnvOverrides {
        api_base_url: Some("http://127.0.0.1:9999".into()),
        info_url: Some("http://elsewhere.test/me".into()),
        ..EnvOverrides::default()
    });

    assert_eq!(cfg.info_url, "http://elsewhere.test/me");
}

#[test]
fn test_from_overrides_carries_every_supplied_value() {
    let cfg = Config::from_overrides(&EnvOverrides {
        client_id: Some("CID".into()),
        client_secret: Some("SECRET".into()),
        redirect_uri: Some("https://example.com/cb".into()),
        auth_url: Some("https://auth.test/authorize".into()),
        token_url: Some("https://auth.test/token".into()),
        api_base_url: Some("https://api.test".into()),
        info_url: Some("https://api.test/me".into()),
        bearer_token: None,
        output: None,
        home: None,
        token_store: None,
    });

    assert_eq!(cfg.client_id, "CID");
    assert_eq!(cfg.client_secret, "SECRET");
    assert_eq!(cfg.redirect_uri, "https://example.com/cb");
    assert_eq!(cfg.auth_url, "https://auth.test/authorize");
    assert_eq!(cfg.token_url, "https://auth.test/token");
    assert_eq!(cfg.api_base_url, "https://api.test");
    assert_eq!(cfg.info_url, "https://api.test/me");
}

// Edge proof: the process-reading path and the injected path agree. This is
// the only assertion here that needs the process environment, so it stays
// serial while the value-level tests above run in parallel.
#[test]
#[serial]
fn test_new_matches_from_overrides_for_every_value() {
    for key in &[
        "CLIENT_ID",
        "CLIENT_SECRET",
        "REDIRECT_URI",
        "AUTH_URL",
        "TOKEN_URL",
        "API_BASE_URL",
        "INFO_URL",
    ] {
        unsafe {
            std::env::remove_var(key);
        }
    }
    unsafe {
        std::env::set_var("CLIENT_ID", "EDGE-CID");
        std::env::set_var("API_BASE_URL", "http://127.0.0.1:8123");
        std::env::set_var("REDIRECT_URI", "https://edge.example/cb");
    }

    let from_process = Config::new();
    let from_injection = Config::from_overrides(&EnvOverrides::from_env());

    unsafe {
        std::env::remove_var("CLIENT_ID");
        std::env::remove_var("API_BASE_URL");
        std::env::remove_var("REDIRECT_URI");
    }

    assert_eq!(from_process.client_id, from_injection.client_id);
    assert_eq!(from_process.client_secret, from_injection.client_secret);
    assert_eq!(from_process.redirect_uri, from_injection.redirect_uri);
    assert_eq!(from_process.auth_url, from_injection.auth_url);
    assert_eq!(from_process.token_url, from_injection.token_url);
    assert_eq!(from_process.api_base_url, from_injection.api_base_url);
    assert_eq!(from_process.info_url, from_injection.info_url);
    assert_eq!(from_process.client_id, "EDGE-CID");
    assert_eq!(from_process.info_url, "http://127.0.0.1:8123/2/users/me");
}

// ── validate_redirect_uri ───────────────────────────────────────────────────

#[test]
fn test_validate_redirect_uri_accepts_https() {
    let url = Config::validate_redirect_uri("https://example.com/cb").expect("https accepted");
    assert_eq!(url.scheme(), "https");
}

#[test]
fn test_validate_redirect_uri_accepts_http_localhost() {
    Config::validate_redirect_uri("http://localhost:9090/cb").expect("localhost accepted");
}

#[test]
fn test_validate_redirect_uri_accepts_http_127_0_0_1() {
    Config::validate_redirect_uri("http://127.0.0.1:9090/cb").expect("127.0.0.1 accepted");
}

#[test]
fn test_validate_redirect_uri_accepts_http_ipv6_loopback() {
    Config::validate_redirect_uri("http://[::1]:9090/cb").expect("[::1] accepted");
}

#[test]
fn test_validate_redirect_uri_rejects_http_remote() {
    let err = Config::validate_redirect_uri("http://example.com/cb");
    assert!(err.is_err(), "http+remote should be rejected");
}

#[test]
fn test_validate_redirect_uri_rejects_ftp() {
    let err = Config::validate_redirect_uri("ftp://localhost/cb");
    assert!(err.is_err(), "ftp scheme should be rejected");
}

#[test]
fn test_validate_redirect_uri_rejects_unparseable() {
    let err = Config::validate_redirect_uri("not-a-url");
    assert!(err.is_err(), "unparseable URI should be rejected");
}

#[test]
fn test_validate_redirect_uri_rejects_file_scheme() {
    let err = Config::validate_redirect_uri("file:///etc/passwd");
    assert!(err.is_err(), "file scheme should be rejected");
}
