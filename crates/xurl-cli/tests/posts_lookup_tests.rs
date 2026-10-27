//! `xr read-posts` reads up to 100 posts in one request: each argument is a
//! post id or URL, the ids go out comma-separated, and X's `errors` array,
//! naming the ids it could not return, prints beside the posts it could.
//! More than 100 ids are refused before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

#[tokio::test]
async fn read_posts_sends_every_id_in_one_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets"))
        .and(query_param("ids", "1,2,3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"id": "1", "text": "one"},
                {"id": "2", "text": "two"},
                {"id": "3", "text": "three"}
            ]
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
            "read-posts",
            "1",
            "https://x.com/alice/status/2",
            "3",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"].as_array().map(Vec::len), Some(3));
    assert_eq!(body["data"][2]["text"], "three");
}

#[tokio::test]
async fn read_posts_prints_the_ids_x_could_not_return_beside_the_posts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "1", "text": "one"}],
            "errors": [{
                "value": "2",
                "detail": "Could not find tweet with ids: [2].",
                "title": "Not Found Error",
                "resource_type": "tweet",
                "parameter": "ids",
                "resource_id": "2",
                "type": "https://api.x.com/2/problems/resource-not-found"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["read-posts", "1", "2", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["id"], "1");
    assert_eq!(body["errors"][0]["resource_id"], "2");
}

#[tokio::test]
async fn read_posts_takes_100_ids_and_refuses_101_before_any_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let ids: Vec<String> = (1..=101).map(|n| n.to_string()).collect();
    let args = |count: usize| -> Vec<&str> {
        ["--output", "json", "read-posts"]
            .into_iter()
            .chain(ids[..count].iter().map(String::as_str))
            .collect()
    };

    let (code, _stdout, stderr) = run(&store, &server.uri(), &args(100)).await;
    assert_eq!(code, 0, "stderr: {stderr}");

    let (code, _stdout, stderr) = run(&store, &server.uri(), &args(101)).await;
    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "validation");
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains("too-many-ids")),
        "{envelope}"
    );
}

#[tokio::test]
async fn read_posts_refuses_an_empty_id_among_the_others() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--dry-run", "--output", "json", "read-posts", "1", ""],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "read-posts");
    assert_eq!(body["would_succeed"], false);
    assert_eq!(body["reason"], "empty-post-id");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
