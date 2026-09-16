//! Tests for the Error type system and exit code mapping.

use xurl::error::{
    EXIT_AUTH_MISMATCH, EXIT_AUTH_REQUIRED, EXIT_GENERAL_ERROR, EXIT_NETWORK_ERROR, EXIT_NOT_FOUND,
    EXIT_RATE_LIMITED, EXIT_USAGE_ERROR, Error, exit_code_for_error,
};

#[test]
fn test_xurl_error_http_is_not_api() {
    let err = Error::Http("connection refused".to_string());
    assert!(!err.is_api(), "Http error should not be is_api()");
}

#[test]
fn test_xurl_error_api_is_api() {
    let err = Error::api(404, r#"{"errors":[{"message":"Not Found"}]}"#);
    assert!(err.is_api(), "Api error should be is_api()");
}

#[test]
fn test_xurl_error_validation_is_validation() {
    let err = Error::validation("bad input");
    assert!(
        err.is_validation(),
        "Validation error should be is_validation()"
    );
    assert!(!err.is_api(), "Validation error should not be is_api()");
}

#[test]
fn test_xurl_error_api_is_not_validation() {
    let err = Error::api(400, "bad request");
    assert!(
        !err.is_validation(),
        "Api error should not be is_validation()"
    );
}

#[test]
fn test_xurl_error_auth_is_not_api() {
    let err = Error::auth("token expired");
    assert!(!err.is_api(), "Auth error should not be is_api()");
}

#[test]
fn test_xurl_error_io_is_not_api() {
    let err = Error::Io("file not found".to_string());
    assert!(!err.is_api());
}

#[test]
fn test_xurl_error_json_is_not_api() {
    let err = Error::Json("invalid json".to_string());
    assert!(!err.is_api());
}

#[test]
fn test_xurl_error_token_store_is_not_api() {
    let err = Error::token_store("store corrupted");
    assert!(!err.is_api());
}

#[test]
fn test_xurl_error_display_http() {
    let err = Error::Http("connection refused".to_string());
    let msg = format!("{err}");
    assert!(
        msg.contains("HTTP Error"),
        "Expected 'HTTP Error' in: {msg}"
    );
    assert!(msg.contains("connection refused"));
}

#[test]
fn test_xurl_error_display_api() {
    let err = Error::api(400, "bad request");
    let msg = format!("{err}");
    // Display shows body only, not status
    assert_eq!(msg, "bad request", "Expected body-only display, got: {msg}");
}

#[test]
fn test_xurl_error_display_validation() {
    let err = Error::validation("bad input");
    let msg = format!("{err}");
    assert_eq!(msg, "bad input", "Expected message display, got: {msg}");
}

#[test]
fn test_xurl_error_api_constructor() {
    let err = Error::api(401, "unauthorized");
    match &err {
        Error::Api { status, body } => {
            assert_eq!(*status, 401);
            assert_eq!(body, "unauthorized");
        }
        _ => panic!("Expected Api variant, got: {err:?}"),
    }
}

#[test]
fn test_xurl_error_display_auth() {
    let err = Error::auth("token expired");
    let msg = format!("{err}");
    assert!(
        msg.contains("Auth Error"),
        "Expected 'Auth Error' in: {msg}"
    );
    assert!(msg.contains("token expired"));
}

#[test]
fn test_xurl_error_display_io() {
    let err = Error::Io("file not found".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("IO Error"), "Expected 'IO Error' in: {msg}");
}

#[test]
fn test_xurl_error_display_json() {
    let err = Error::Json("unexpected token".to_string());
    let msg = format!("{err}");
    assert!(
        msg.contains("JSON Error"),
        "Expected 'JSON Error' in: {msg}"
    );
}

#[test]
fn test_xurl_error_display_invalid_method() {
    let err = Error::InvalidMethod("FROBNICATE".to_string());
    let msg = format!("{err}");
    assert!(
        msg.contains("Invalid Method"),
        "Expected 'Invalid Method' in: {msg}"
    );
    assert!(msg.contains("FROBNICATE"));
}

#[test]
fn test_xurl_error_display_token_store() {
    let err = Error::token_store("corrupt yaml");
    let msg = format!("{err}");
    assert!(
        msg.contains("Token Store Error"),
        "Expected 'Token Store Error' in: {msg}"
    );
}

#[test]
fn test_xurl_error_auth_with_cause() {
    let err = Error::auth_with_cause("NetworkError", &"timeout");
    let msg = format!("{err}");
    assert!(msg.contains("NetworkError"));
    assert!(msg.contains("timeout"));
}

#[test]
fn test_xurl_error_from_reqwest() {
    // Create a reqwest error by trying to build an invalid request
    let result = reqwest::blocking::Client::new().get("not-a-url").send();
    if let Err(reqwest_err) = result {
        let xurl_err: Error = reqwest_err.into();
        assert!(matches!(xurl_err, Error::Http(_)));
    }
}

#[test]
fn test_xurl_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
    let xurl_err: Error = io_err.into();
    assert!(matches!(xurl_err, Error::Io(_)));
    assert!(format!("{xurl_err}").contains("gone"));
}

#[test]
fn test_xurl_error_from_serde_json() {
    let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let xurl_err: Error = json_err.into();
    assert!(matches!(xurl_err, Error::Json(_)));
}

// ── exit_code_for_error tests ──────────────────────────────────────

#[test]
fn test_exit_code_api_401() {
    assert_eq!(
        exit_code_for_error(&Error::api(401, "unauthorized")),
        EXIT_AUTH_REQUIRED
    );
}

#[test]
fn test_exit_code_api_429() {
    assert_eq!(
        exit_code_for_error(&Error::api(429, "rate limited")),
        EXIT_RATE_LIMITED
    );
}

#[test]
fn test_exit_code_api_404() {
    assert_eq!(
        exit_code_for_error(&Error::api(404, "not found")),
        EXIT_NOT_FOUND
    );
}

#[test]
fn test_exit_code_api_500() {
    assert_eq!(
        exit_code_for_error(&Error::api(500, "server error")),
        EXIT_GENERAL_ERROR
    );
}

#[test]
fn test_exit_code_api_403() {
    assert_eq!(
        exit_code_for_error(&Error::api(403, "forbidden")),
        EXIT_GENERAL_ERROR
    );
}

#[test]
fn test_exit_code_validation() {
    assert_eq!(
        exit_code_for_error(&Error::validation("bad input")),
        EXIT_GENERAL_ERROR
    );
}

#[test]
fn test_exit_code_auth() {
    assert_eq!(
        exit_code_for_error(&Error::auth("expired")),
        EXIT_AUTH_REQUIRED
    );
}

#[test]
fn test_exit_code_token_store() {
    assert_eq!(
        exit_code_for_error(&Error::token_store("corrupt")),
        EXIT_AUTH_REQUIRED
    );
}

#[test]
fn test_exit_code_io() {
    assert_eq!(
        exit_code_for_error(&Error::Io("timeout".into())),
        EXIT_NETWORK_ERROR
    );
}

#[test]
fn test_exit_code_http_401_string() {
    assert_eq!(
        exit_code_for_error(&Error::Http("401 Unauthorized".into())),
        EXIT_AUTH_REQUIRED
    );
}

#[test]
fn test_exit_code_http_generic() {
    assert_eq!(
        exit_code_for_error(&Error::Http("connection refused".into())),
        EXIT_GENERAL_ERROR
    );
}

/// The usage code the runner returns for an invalid invocation and for a word
/// that names no command. It shares its number with the auth-mismatch code,
/// which the documented exit-code table states as one row.
#[test]
fn test_exit_usage_error_is_ex_usage() {
    assert_eq!(EXIT_USAGE_ERROR, 2);
    assert_eq!(EXIT_USAGE_ERROR, EXIT_AUTH_MISMATCH);
}
