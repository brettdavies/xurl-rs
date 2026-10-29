//! `xr user-posts` lists the posts a user wrote: the handle resolves to an
//! id once, and the list is read from that user's posts endpoint. An empty
//! handle is refused before any request, and a dry run sends nothing.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const JACK_ID: &str = "12";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

#[tokio::test]
async fn user_posts_resolves_the_handle_then_reads_its_posts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "jack"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": JACK_ID, "name": "jack", "username": "jack"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/2/users/{JACK_ID}/tweets")))
        .and(query_param("max_results", "25"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "20", "text": "just setting up my twttr"}],
            "meta": {"result_count": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["user-posts", "@jack", "-n", "25", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["id"], "20");
}

#[tokio::test]
async fn user_posts_refuses_an_empty_handle_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "user-posts", ""],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "validation");
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains("empty-username")),
        "{envelope}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn user_posts_under_dry_run_names_the_user_and_sends_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--dry-run", "--output", "json", "user-posts", "@jack"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "user-posts");
    assert_eq!(body["of"], "@jack");
    assert_eq!(body["would_succeed"], true);
    assert_eq!(body["auth"]["scheme"], "oauth2");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
