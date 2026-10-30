//! `xr spaces` finds X Spaces: by keyword, by id, by who created them, and
//! the posts shared in one. `by-creator` takes ids and handles in any mix and
//! resolves the handles in one lookup, and a Space id outside the spec's
//! pattern is refused before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const SPACE: &str = "1DXxyRYNejbKM";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn spaces() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "data": [{"id": SPACE, "state": "live", "title": "Rust hour", "participant_count": 40}]
    }))
}

async fn run_json(server: &MockServer, args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let mut argv = args.to_vec();
    argv.extend(["--output", "json"]);
    run(&store, &server.uri(), &argv).await
}

#[tokio::test]
async fn search_sends_the_query_and_state() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/spaces/search"))
        .and(query_param("query", "rust"))
        .and(query_param("state", "live"))
        .respond_with(spaces())
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) =
        run_json(&server, &["spaces", "search", "rust", "--state", "live"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["title"], "Rust hour");
}

#[tokio::test]
async fn get_reads_every_id_in_one_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/spaces"))
        .and(query_param(
            "ids",
            format!("{SPACE},1YpKkgVgQwvGj").as_str(),
        ))
        .respond_with(spaces())
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) =
        run_json(&server, &["spaces", "get", SPACE, "1YpKkgVgQwvGj"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn by_creator_resolves_every_handle_in_one_lookup() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "alice,bob"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"id": "11", "name": "Alice", "username": "alice"},
                {"id": "22", "name": "Bob", "username": "Bob"}
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/spaces/by/creator_ids"))
        .and(query_param("user_ids", "11,22"))
        .respond_with(spaces())
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) =
        run_json(&server, &["spaces", "by-creator", "@alice", "@bob"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn by_creator_takes_ids_and_handles_together() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "bob"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "22", "name": "Bob", "username": "bob"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/spaces/by/creator_ids"))
        .and(query_param("user_ids", "11,22,33"))
        .respond_with(spaces())
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) =
        run_json(&server, &["spaces", "by-creator", "11", "@bob", "33"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn by_creator_sends_ids_alone_without_a_lookup() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/spaces/by/creator_ids"))
        .and(query_param("user_ids", "11,22"))
        .respond_with(spaces())
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(&server, &["spaces", "by-creator", "11", "22"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 1);
}

#[tokio::test]
async fn by_creator_names_a_handle_x_could_not_find() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "11", "name": "Alice", "username": "alice"}],
            "errors": [{"value": "nobody_here", "detail": "Could not find user with usernames: [nobody_here].", "title": "Not Found Error", "resource_type": "user", "parameter": "usernames", "resource_id": "nobody_here", "type": "https://api.x.com/2/problems/resource-not-found"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) =
        run_json(&server, &["spaces", "by-creator", "@alice", "@nobody_here"]).await;

    assert_ne!(code, 0, "stderr: {stderr}");
    assert!(stderr.contains("nobody_here"), "{stderr}");
}

#[tokio::test]
async fn posts_reads_a_spaces_posts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/spaces/{SPACE}/tweets")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "20", "text": "just setting up my twttr"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(&server, &["spaces", "posts", SPACE]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["id"], "20");
}

#[tokio::test]
async fn inputs_outside_the_spec_are_refused_before_any_request() {
    let server = MockServer::start().await;

    for (args, reason) in [
        (vec!["spaces", "get", "not-a-space!"], "invalid-space-id"),
        (
            vec!["spaces", "posts", "1DXxyRYNejbKMxx"],
            "invalid-space-id",
        ),
        (
            vec!["spaces", "by-creator", "11", "@not-a-handle!"],
            "invalid-username",
        ),
    ] {
        let mut argv = vec!["--dry-run"];
        argv.extend(&args);
        let (code, stdout, stderr) = run_json(&server, &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{args:?}");
        assert_eq!(body["rule"], reason, "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
