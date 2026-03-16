//! Tests for Config loading and environment variable handling.

use xurl::config::Config;

#[test]
fn test_config_defaults() {
    // Clear any env vars that might interfere
    for key in &["CLIENT_ID", "CLIENT_SECRET", "REDIRECT_URI", "AUTH_URL", "TOKEN_URL", "API_BASE_URL", "INFO_URL"] {
        unsafe { std::env::remove_var(key); }
    }

    let cfg = Config::new();

    assert_eq!(cfg.client_id, "", "Default client_id should be empty");
    assert_eq!(cfg.client_secret, "", "Default client_secret should be empty");
    assert_eq!(cfg.redirect_uri, "http://localhost:8080/callback");
    assert_eq!(cfg.auth_url, "https://x.com/i/oauth2/authorize");
    assert_eq!(cfg.token_url, "https://api.x.com/2/oauth2/token");
    assert_eq!(cfg.api_base_url, "https://api.x.com");
    assert!(cfg.info_url.contains("/2/users/me"));
    assert_eq!(cfg.app_name, "");
}

#[test]
fn test_config_from_env_client_id() {
    unsafe { std::env::set_var("CLIENT_ID", "env-test-id"); }
    let cfg = Config::new();
    assert_eq!(cfg.client_id, "env-test-id");
    unsafe { std::env::remove_var("CLIENT_ID"); }
}

#[test]
fn test_config_from_env_client_secret() {
    unsafe { std::env::set_var("CLIENT_SECRET", "env-test-secret"); }
    let cfg = Config::new();
    assert_eq!(cfg.client_secret, "env-test-secret");
    unsafe { std::env::remove_var("CLIENT_SECRET"); }
}

#[test]
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
fn test_config_from_env_api_base_url() {
    unsafe { std::env::set_var("API_BASE_URL", "https://custom.api.example.com"); }
    let cfg = Config::new();
    assert_eq!(cfg.api_base_url, "https://custom.api.example.com");
    // info_url should be derived from api_base_url
    assert!(cfg.info_url.starts_with("https://custom.api.example.com"));
    unsafe { std::env::remove_var("API_BASE_URL"); }
}

#[test]
fn test_config_default_trait() {
    // Config implements Default (which calls new())
    let cfg = Config::default();
    assert!(!cfg.redirect_uri.is_empty());
    assert!(!cfg.auth_url.is_empty());
}
