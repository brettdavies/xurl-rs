//! `xr bookmark-folders` reaches the bookmark-folder endpoints through the
//! runner: each verb resolves the caller, sends the request the spec gives
//! its endpoint, and prints the typed response, and a folder id or name the
//! spec refuses is refused before any request. Creating a folder takes an
//! `OAuth2` login; the reads also take `OAuth1`.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth1_store, oauth2_store, run_in_process as run};

const MY_ID: &str = "2244994945";
const FOLDER_ID: &str = "1146654567674912769";

async fn answer_me(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": MY_ID, "name": "Alice", "username": "alice"}
        })))
        .mount(server)
        .await;
}

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

#[tokio::test]
async fn list_prints_the_callers_folders() {
    let server = MockServer::start().await;
    answer_me(&server).await;
    Mock::given(method("GET"))
        .and(path(format!("/2/users/{MY_ID}/bookmarks/folders")))
        .and(query_param("max_results", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": FOLDER_ID, "name": "Rust"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["bookmark-folders", "list", "-n", "20", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["id"], FOLDER_ID);
    assert_eq!(body["data"][0]["name"], "Rust");
}

#[tokio::test]
async fn posts_lists_the_posts_in_the_folder() {
    let server = MockServer::start().await;
    answer_me(&server).await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/2/users/{MY_ID}/bookmarks/folders/{FOLDER_ID}"
        )))
        .and(query_param("max_results", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "1346889436626259968"}, {"id": "1460323737035677698"}]
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
            "bookmark-folders",
            "posts",
            FOLDER_ID,
            "-n",
            "50",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][1]["id"], "1460323737035677698");
}

#[tokio::test]
async fn add_posts_the_name() {
    let server = MockServer::start().await;
    answer_me(&server).await;
    Mock::given(method("POST"))
        .and(path(format!("/2/users/{MY_ID}/bookmarks/folders")))
        .and(body_json(serde_json::json!({"name": "Reading list"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": FOLDER_ID, "name": "Reading list"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "bookmark-folders",
            "add",
            "Reading list",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["id"], FOLDER_ID);
}

#[tokio::test]
async fn add_refuses_a_name_past_25_characters_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "bookmark-folders",
            "add",
            "A name that runs past the limit",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "bookmark-folders-add");
    assert_eq!(body["would_succeed"], false);
    assert_eq!(body["reason"], "folder-name-too-long");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn posts_refuses_a_folder_id_that_is_not_digits_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "bookmark-folders", "posts", "rust"],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "validation");
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains("invalid-folder-id")),
        "{envelope}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn posts_under_dry_run_names_the_folder_and_sends_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "bookmark-folders",
            "posts",
            FOLDER_ID,
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "bookmark-folders-posts");
    assert_eq!(body["folder_id"], FOLDER_ID);
    assert_eq!(body["would_succeed"], true);
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
