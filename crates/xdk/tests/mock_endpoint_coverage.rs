//! Every endpoint the shortcut layer declares answers from the testing mock.
//!
//! The probe starts a mock, renders each declared path with a numeric token
//! in every parameter, sends the declared method, and treats the server's
//! unmatched-route reply as an unanswered endpoint. Probing a running mock
//! rather than reading its route table catches a pattern that never matches
//! the path a shortcut really sends.

mod common;

use common::{Reply, check, load_spec, response_schema};
use xdk::api::auth_matrix::{Endpoint, SHORTCUT_TEMPLATES};
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

#[tokio::test]
async fn every_mock_reply_is_the_shape_the_spec_gives_its_endpoint() {
    let spec = load_spec();
    let mock = MockX::start().await;
    let http = reqwest::Client::new();
    let mut failures = Vec::new();
    for (method, path) in SHORTCUT_TEMPLATES {
        let endpoint = Endpoint { method, path };
        let url = format!("{}{}", mock.base_url(), materialize(path));
        let response = http
            .request(method.parse().expect("a standard method"), &url)
            .send()
            .await
            .unwrap_or_else(|e| panic!("{method} {path}: the mock did not answer: {e}"));
        let status = response.status().as_u16();
        let body = response.text().await.expect("a readable body");
        let mut errors = Vec::new();
        match response_schema(&spec, &endpoint, status) {
            Reply::Undeclared => errors.push(format!(
                "answers {status}, a status the spec does not declare for this endpoint"
            )),
            Reply::NoJson if !body.trim().is_empty() => {
                errors.push(format!(
                    "answers {status} with a body the spec gives no JSON schema"
                ));
            }
            Reply::NoJson => {}
            Reply::Json(schema) => match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(value) => check(&value, schema, &spec, "reply", &mut errors),
                Err(e) => errors.push(format!(
                    "answers {status} with a body that is not JSON: {e}"
                )),
            },
        }
        if !errors.is_empty() {
            failures.push(format!("{method} {path}:\n    {}", errors.join("\n    ")));
        }
    }
    assert!(
        failures.is_empty(),
        "the testing mock answers these declared endpoints with a reply the spec does not give them:\n{}\n\
         Cause: the Route in crates/xdk/src/testing/mod.rs names a fixture or status the spec does not \
         document for that endpoint, or the fixture drifted from the spec.\n\
         Fix: point the Route at a fixture shaped to the endpoint's own response schema (add one to \
         tests/fixtures/openapi/example_responses.json with its spec_* test and FIXTURE_ENDPOINTS row), \
         and answer with a status the spec declares.",
        failures.join("\n")
    );
}
