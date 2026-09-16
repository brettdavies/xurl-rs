//! Tests for the `Error` type: constructors, Display fragments, documentation
//! pointers, and the exit-code mapping.

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
    assert_eq!(format!("{err}"), "connection refused");
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
    assert_eq!(format!("{err}"), "token expired");
}

#[test]
fn test_xurl_error_display_io() {
    let err = Error::Io("file not found".to_string());
    assert_eq!(format!("{err}"), "file not found");
}

#[test]
fn test_xurl_error_display_json() {
    let err = Error::Json("unexpected token".to_string());
    assert_eq!(format!("{err}"), "unexpected token");
}

#[test]
fn test_xurl_error_display_invalid_method() {
    let err = Error::InvalidMethod("FROBNICATE".to_string());
    assert_eq!(format!("{err}"), "invalid HTTP method: FROBNICATE");
}

#[test]
fn test_xurl_error_display_token_store() {
    let err = Error::token_store("corrupt yaml");
    assert_eq!(format!("{err}"), "corrupt yaml");
}

/// A representative of every variant, with lowercase payloads so the check
/// reads the format strings rather than the data they carry.
fn one_of_each_variant() -> Vec<Error> {
    vec![
        Error::Http("connection refused".into()),
        Error::Io("permission denied".into()),
        Error::InvalidMethod("bad method".into()),
        Error::api(500, "server error"),
        Error::validation("missing field"),
        Error::InvalidUrl("ftp://example".into()),
        Error::InvalidPathParam {
            name: "id".into(),
            value: "1/2".into(),
        },
        Error::Internal("missing {id}".into()),
        Error::Json("expected value".into()),
        Error::auth("token expired"),
        Error::token_store("corrupt yaml"),
        mismatch(Some("oauth1"), None, None),
        mismatch(None, Some(&["oauth1"]), None),
        mismatch(None, Some(&[]), Some(&["prod"])),
        mismatch(None, None, None),
    ]
}

fn mismatch(
    requested: Option<&str>,
    available_in_app: Option<&[&str]>,
    other_apps_with_creds: Option<&[&str]>,
) -> Error {
    let strings = |items: &[&str]| items.iter().map(ToString::to_string).collect::<Vec<_>>();
    Error::AuthMethodMismatch {
        endpoint: "/2/users/{id}/likes".into(),
        rendered_url: Some("/2/users/12345/likes".into()),
        method: "GET".into(),
        requested: requested.map(str::to_string),
        supported: vec!["app".into(), "oauth2".into()],
        available_in_app: available_in_app.map(strings),
        app: Some("default".into()),
        other_apps_with_creds: other_apps_with_creds.map(strings),
    }
}

/// Library convention: a Display string is a fragment an embedder can wrap,
/// so it starts lowercase, carries no `Error:` prefix, and ends without a
/// period.
#[test]
fn display_is_a_prefix_free_lowercase_fragment() {
    for err in one_of_each_variant() {
        let msg = err.to_string();
        let first = msg.chars().next().expect("non-empty display");
        assert!(
            !first.is_uppercase(),
            "{err:?} starts with an uppercase letter: {msg}"
        );
        assert!(!msg.contains("Error:"), "{err:?} carries a prefix: {msg}");
        assert!(!msg.ends_with('.'), "{err:?} ends with a period: {msg}");
    }
}

#[test]
fn auth_method_mismatch_display_names_the_request_and_the_accepted_methods() {
    assert_eq!(
        mismatch(Some("oauth1"), None, None).to_string(),
        "oauth1 auth is not accepted at GET /2/users/12345/likes (accepts app, oauth2)"
    );
    assert_eq!(
        mismatch(None, Some(&["oauth1"]), None).to_string(),
        "no stored auth method on app 'default' is accepted at GET /2/users/12345/likes (app has oauth1; endpoint accepts app, oauth2)"
    );
    assert_eq!(
        mismatch(None, Some(&[]), Some(&["prod", "staging"])).to_string(),
        "app 'default' holds no credentials for GET /2/users/12345/likes (other apps with credentials: prod, staging)"
    );
    assert_eq!(
        mismatch(None, None, None).to_string(),
        "auth method is not accepted at GET /2/users/12345/likes"
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
