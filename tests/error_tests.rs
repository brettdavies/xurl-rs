//! Tests for the XurlError type system.

use xurl::error::XurlError;

#[test]
fn test_xurl_error_http_is_not_api() {
    let err = XurlError::Http("connection refused".to_string());
    assert!(!err.is_api(), "Http error should not be is_api()");
}

#[test]
fn test_xurl_error_api_is_api() {
    let err = XurlError::api(r#"{"errors":[{"message":"Not Found"}]}"#);
    assert!(err.is_api(), "Api error should be is_api()");
}

#[test]
fn test_xurl_error_auth_is_not_api() {
    let err = XurlError::auth("token expired");
    assert!(!err.is_api(), "Auth error should not be is_api()");
}

#[test]
fn test_xurl_error_io_is_not_api() {
    let err = XurlError::Io("file not found".to_string());
    assert!(!err.is_api());
}

#[test]
fn test_xurl_error_json_is_not_api() {
    let err = XurlError::Json("invalid json".to_string());
    assert!(!err.is_api());
}

#[test]
fn test_xurl_error_token_store_is_not_api() {
    let err = XurlError::token_store("store corrupted");
    assert!(!err.is_api());
}

#[test]
fn test_xurl_error_display_http() {
    let err = XurlError::Http("connection refused".to_string());
    let msg = format!("{err}");
    assert!(
        msg.contains("HTTP Error"),
        "Expected 'HTTP Error' in: {msg}"
    );
    assert!(msg.contains("connection refused"));
}

#[test]
fn test_xurl_error_display_api() {
    let err = XurlError::api("bad request");
    let msg = format!("{err}");
    assert!(msg.contains("bad request"), "Expected error body in: {msg}");
}

#[test]
fn test_xurl_error_display_auth() {
    let err = XurlError::auth("token expired");
    let msg = format!("{err}");
    assert!(
        msg.contains("Auth Error"),
        "Expected 'Auth Error' in: {msg}"
    );
    assert!(msg.contains("token expired"));
}

#[test]
fn test_xurl_error_display_io() {
    let err = XurlError::Io("file not found".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("IO Error"), "Expected 'IO Error' in: {msg}");
}

#[test]
fn test_xurl_error_display_json() {
    let err = XurlError::Json("unexpected token".to_string());
    let msg = format!("{err}");
    assert!(
        msg.contains("JSON Error"),
        "Expected 'JSON Error' in: {msg}"
    );
}

#[test]
fn test_xurl_error_display_invalid_method() {
    let err = XurlError::InvalidMethod("FROBNICATE".to_string());
    let msg = format!("{err}");
    assert!(
        msg.contains("Invalid Method"),
        "Expected 'Invalid Method' in: {msg}"
    );
    assert!(msg.contains("FROBNICATE"));
}

#[test]
fn test_xurl_error_display_token_store() {
    let err = XurlError::token_store("corrupt yaml");
    let msg = format!("{err}");
    assert!(
        msg.contains("Token Store Error"),
        "Expected 'Token Store Error' in: {msg}"
    );
}

#[test]
fn test_xurl_error_auth_with_cause() {
    let err = XurlError::auth_with_cause("NetworkError", &"timeout");
    let msg = format!("{err}");
    assert!(msg.contains("NetworkError"));
    assert!(msg.contains("timeout"));
}

#[test]
fn test_xurl_error_from_reqwest() {
    // Create a reqwest error by trying to build an invalid request
    let result = reqwest::blocking::Client::new().get("not-a-url").send();
    if let Err(reqwest_err) = result {
        let xurl_err: XurlError = reqwest_err.into();
        assert!(matches!(xurl_err, XurlError::Http(_)));
    }
}

#[test]
fn test_xurl_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
    let xurl_err: XurlError = io_err.into();
    assert!(matches!(xurl_err, XurlError::Io(_)));
    assert!(format!("{xurl_err}").contains("gone"));
}

#[test]
fn test_xurl_error_from_serde_json() {
    let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let xurl_err: XurlError = json_err.into();
    assert!(matches!(xurl_err, XurlError::Json(_)));
}
