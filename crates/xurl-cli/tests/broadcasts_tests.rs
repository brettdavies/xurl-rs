//! `xr broadcasts moderators` reaches the chat moderator endpoints through
//! the runner: the list is a GET, add resolves the handle and POSTs the user
//! id, and both print the typed response as JSON.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth1_store, run_in_process as run};

#[tokio::test]
async fn moderators_list_prints_the_users_as_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/broadcasts/chat/moderators"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "2244994945", "name": "Helper", "username": "helper"}],
            "meta": {"result_count": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["broadcasts", "moderators", "list", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"][0]["username"], "helper");
}

#[tokio::test]
async fn moderators_add_resolves_the_handle_and_posts_the_user_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by/username/helper"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "2244994945", "name": "Helper", "username": "helper"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/broadcasts/chat/moderators"))
        .and(body_json(serde_json::json!({"user_id": "2244994945"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"moderator_user_ids": ["2244994945"]}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "moderators",
            "add",
            "@helper",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["moderator_user_ids"][0], "2244994945");
}

#[tokio::test]
async fn moderators_remove_deletes_by_the_resolved_user_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by/username/helper"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "2244994945", "name": "Helper", "username": "helper"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/2/broadcasts/chat/moderators/2244994945"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"moderator_user_ids": []}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "moderators",
            "remove",
            "@helper",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn moderators_add_dry_run_validates_without_a_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, _stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "broadcasts",
            "moderators",
            "add",
            "@helper",
        ],
    )
    .await;

    assert_eq!(code, 0);
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["would_succeed"], true, "{body}");
    assert_eq!(body["command"], "broadcasts-moderators-add", "{body}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
