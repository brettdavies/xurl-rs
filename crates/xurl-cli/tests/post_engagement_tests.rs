//! `xr likers`, `xr reposters`, and `xr quotes` reach the post-engagement
//! endpoints through the runner: each resolves a post URL to its id, sends
//! the page size and cursor, and prints the typed list, with no lookup of
//! the caller first. The scheme each goes out under is the one the spec
//! gives its endpoint: `liking_users` takes no app token.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const POST_ID: &str = "1460323737035677698";

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

async fn refuse_me_lookups(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(server)
        .await;
}

#[tokio::test]
async fn likers_lists_the_users_who_liked_the_post() {
    let server = MockServer::start().await;
    refuse_me_lookups(&server).await;
    Mock::given(method("GET"))
        .and(path(format!("/2/tweets/{POST_ID}/liking_users")))
        .and(query_param("max_results", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "2244994945", "name": "X Dev", "username": "XDevelopers"}],
            "meta": {"result_count": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let url = format!("https://x.com/alice/status/{POST_ID}");

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["likers", &url, "-n", "50", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["username"], "XDevelopers");
}

#[tokio::test]
async fn quotes_sends_the_cursor_as_the_pagination_token() {
    let server = MockServer::start().await;
    refuse_me_lookups(&server).await;
    Mock::given(method("GET"))
        .and(path(format!("/2/tweets/{POST_ID}/quote_tweets")))
        .and(query_param("pagination_token", "NEXT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "1346889436626259968", "text": "Worth a read."}],
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
        &["--cursor", "NEXT", "quotes", POST_ID, "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["text"], "Worth a read.");
}

#[tokio::test]
async fn reposters_goes_out_with_the_bearer_token_alone() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/tweets/{POST_ID}/retweeted_by")))
        .and(header("authorization", "Bearer BEARER-TOKEN"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "2244994945", "name": "X Dev", "username": "XDevelopers"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["reposters", POST_ID, "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["id"], "2244994945");
}

#[tokio::test]
async fn likers_with_only_a_bearer_is_an_auth_mismatch() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "likers", POST_ID],
    )
    .await;

    assert_ne!(code, 0, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "auth-method-mismatch");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn likers_refuses_an_empty_post_id_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--dry-run", "--output", "json", "likers", ""],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "likers");
    assert_eq!(body["would_succeed"], false);
    assert_eq!(body["reason"], "empty-post-id");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
