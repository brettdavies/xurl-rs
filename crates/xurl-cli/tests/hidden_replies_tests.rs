//! `xr hide` and `xr unhide` reach the hide-reply endpoint through the
//! runner: each resolves a post URL to its id, sends the hidden flag in the
//! body, and prints the typed response, with no lookup of the caller first.
//! An empty reply id is refused before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const REPLY_ID: &str = "1585341984679469056";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

async fn refuse_me_lookups(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(server)
        .await;
}

async fn answer_hidden(server: &MockServer, hidden: bool) {
    Mock::given(method("PUT"))
        .and(path(format!("/2/tweets/{REPLY_ID}/hidden")))
        .and(body_json(serde_json::json!({"hidden": hidden})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"hidden": hidden}})),
        )
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn hide_sends_hidden_true_for_the_reply() {
    let server = MockServer::start().await;
    refuse_me_lookups(&server).await;
    answer_hidden(&server, true).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["hide", REPLY_ID, "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["hidden"], true);
}

#[tokio::test]
async fn unhide_resolves_a_post_url_and_sends_hidden_false() {
    let server = MockServer::start().await;
    refuse_me_lookups(&server).await;
    answer_hidden(&server, false).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let url = format!("https://x.com/alice/status/{REPLY_ID}");

    let (code, stdout, stderr) =
        run(&store, &server.uri(), &["unhide", &url, "--output", "json"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["hidden"], false);
}

#[tokio::test]
async fn hide_refuses_an_empty_reply_id_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, _stdout, stderr) =
        run(&store, &server.uri(), &["--output", "json", "hide", ""]).await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "validation");
    assert_eq!(envelope["rule"], "empty-post-id", "{envelope}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn unhide_under_dry_run_names_the_reply_and_sends_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--dry-run", "--output", "json", "unhide", REPLY_ID],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "unhide");
    assert_eq!(body["post_id"], REPLY_ID);
    assert_eq!(body["would_succeed"], true);
    assert_eq!(body["auth"]["scheme"], "oauth2");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
