//! Every endpoint the shortcut layer declares answers from the testing mock.
//!
//! The probe starts a mock, renders each declared path with a numeric token
//! in every parameter, sends the declared method, and treats the server's
//! unmatched-route reply as an unanswered endpoint. Probing a running mock
//! rather than reading its route table catches a pattern that never matches
//! the path a shortcut really sends.

use xdk::api::auth_matrix::SHORTCUT_TEMPLATES;
use xdk::testing::MockX;

/// `path` with every `{parameter}` replaced by a token both `[0-9]+` and
/// `[^/]+` accept.
fn materialize(path: &str) -> String {
    let mut out = String::new();
    let mut rest = path;
    while let Some(open) = rest.find('{') {
        let close = rest[open..].find('}').expect("balanced braces") + open;
        out.push_str(&rest[..open]);
        out.push_str("123");
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

#[tokio::test]
async fn every_declared_endpoint_answers_from_the_mock() {
    let mock = MockX::start().await;
    let http = reqwest::Client::new();
    let mut unanswered = Vec::new();
    for (method, path) in SHORTCUT_TEMPLATES {
        let url = format!("{}{}", mock.base_url(), materialize(path));
        let response = http
            .request(method.parse().expect("a standard method"), &url)
            .send()
            .await
            .unwrap_or_else(|e| panic!("{method} {path}: the mock did not answer at all: {e}"));
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            unanswered.push(format!("{method} {path}"));
        }
    }
    assert!(
        unanswered.is_empty(),
        "the testing mock leaves these declared endpoints unanswered:\n{}\n\
         Cause: SHORTCUT_TEMPLATES in crates/xdk/build.rs declares the endpoint, but ROUTES in \
         crates/xdk/src/testing/mod.rs has no route for it, or its pattern does not match the \
         path the shortcut sends.\n\
         Fix: add a Route naming the endpoint constant and the fixture that answers it.",
        unanswered.join("\n")
    );
}

#[test]
fn materialize_fills_every_parameter() {
    assert_eq!(
        materialize("/2/users/{source_user_id}/following/{target_user_id}"),
        "/2/users/123/following/123"
    );
    assert_eq!(materialize("/2/users/me"), "/2/users/me");
}
