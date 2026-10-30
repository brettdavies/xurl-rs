//! `xr search-users` finds accounts by keyword through `GET /2/users/search`.
//! That endpoint pages with `next_token` rather than `pagination_token`, takes
//! a user login only, and accepts a query of 1 to 50 letters, digits,
//! underscores, apostrophes, and spaces; a query outside that is refused
//! before any request.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn bearer_store(dir: &Path) -> PathBuf {
    let store = dir.join(".xurl");
    let mut ts = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_bearer_token_for_app("myapp", "BEARER-TOKEN")
        .expect("save bearer");
    ts.set_default_app("myapp").expect("set_default_app");
    store
}

#[tokio::test]
async fn search_users_sends_the_query_and_page_size() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/search"))
        .and(query_param("query", "rust lang"))
        .and(query_param("max_results", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "2244994945", "name": "X Dev", "username": "XDevelopers"}],
            "meta": {"next_token": "NEXT"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["search-users", "rust lang", "-n", "20", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["username"], "XDevelopers");
    assert_eq!(body["meta"]["next_token"], "NEXT");
}

#[tokio::test]
async fn search_users_sends_the_cursor_as_next_token() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/search"))
        .and(query_param("next_token", "NEXT"))
        .and(query_param_is_missing("pagination_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "2244994945", "name": "X Dev", "username": "XDevelopers"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--cursor",
            "NEXT",
            "search-users",
            "rust",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn search_users_with_only_a_bearer_is_an_auth_mismatch() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "search-users", "rust"],
    )
    .await;

    assert_ne!(code, 0, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "auth-method-mismatch");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn search_users_refuses_a_query_outside_the_spec_pattern_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let too_long = "a".repeat(51);

    for query in ["", "rust-lang", too_long.as_str()] {
        let (code, stdout, stderr) = run(
            &store,
            &server.uri(),
            &["--dry-run", "--output", "json", "search-users", query],
        )
        .await;

        assert_eq!(code, 0, "{query:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{query:?}");
        assert_eq!(body["reason"], "invalid-user-query", "{query:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

/// `<QUERY>` states, under Arguments, the rule the validator enforces, with
/// the limit read from the validator's own constant; the examples do not
/// repeat it.
#[tokio::test]
async fn the_query_help_states_the_rule_its_validator_enforces() {
    let tmp = TempDir::new().expect("tempdir");
    let rule = format!(
        "1 to {} letters, digits, underscores, apostrophes, or spaces",
        xdk::api::shortcuts::USER_SEARCH_QUERY_MAX_CHARS
    );
    let (code, stdout, stderr) = run(
        &tmp.path().join(".xurl"),
        "http://127.0.0.1:9",
        &["search-users", "--help"],
    )
    .await;
    assert_eq!(code, 0, "{stderr}");

    let arguments = stdout
        .split("Arguments:")
        .nth(1)
        .and_then(|rest| rest.split("Options:").next())
        .expect("an Arguments section");
    assert!(arguments.contains(&rule), "{arguments}");
    let examples = stdout
        .split("Examples:")
        .nth(1)
        .expect("an Examples section");
    assert!(!examples.contains("letters, digits"), "{examples}");
}
