//! `xr drafts` lists the caller's post drafts, saves one, and removes one:
//! each sends one request shaped as the spec gives it, and refuses what
//! breaks a rule before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth1_store, run_in_process as run};

const DRAFT_ID: &str = "1850000000000000011";
const POST_ID: &str = "1585341984679469056";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn saved(text: &str) -> serde_json::Value {
    serde_json::json!({"data": {"id": "1850000000000000013", "text": text}})
}

#[tokio::test]
async fn add_saves_the_text_alone() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/posts/drafts"))
        .and(body_json(serde_json::json!({"text": "Ship the changelog"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(saved("Ship the changelog")))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["drafts", "add", "Ship the changelog", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["id"], "1850000000000000013");
}

#[tokio::test]
async fn add_saves_its_media_and_the_post_it_replies_to_by_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/posts/drafts"))
        .and(body_json(serde_json::json!({
            "text": "Launch day photos",
            "media": {"media_ids": ["1850000000000000020", "1850000000000000021"]},
            "reply": {"in_reply_to_tweet_id": POST_ID}
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(saved("Launch day photos")))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let url = format!("https://x.com/jack/status/{POST_ID}");

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "drafts",
            "add",
            "Launch day photos",
            "--media-id",
            "1850000000000000020",
            "--media-id",
            "1850000000000000021",
            "--reply-to",
            &url,
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn list_sends_the_page_size_and_the_cursor() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/posts/drafts"))
        .and(query_param("max_results", "5"))
        .and(query_param("pagination_token", "1850000000000000012"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": DRAFT_ID, "text": "Older draft"}],
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
        &[
            "--cursor",
            "1850000000000000012",
            "drafts",
            "list",
            "-n",
            "5",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["text"], "Older draft");
}

#[tokio::test]
async fn list_sends_no_cursor_when_none_is_given() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/posts/drafts"))
        .and(query_param_is_missing("pagination_token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": [], "meta": {}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["drafts", "list", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn remove_deletes_the_draft_with_force() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/posts/drafts/{DRAFT_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["drafts", "remove", DRAFT_ID, "--force", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["deleted"], true);
}

#[tokio::test]
async fn remove_without_force_sends_nothing_when_it_cannot_ask() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--no-interactive",
            "--output",
            "json",
            "drafts",
            "remove",
            DRAFT_ID,
        ],
    )
    .await;

    assert_ne!(code, 0, "{stdout}");
    let refusal = json_out(&stderr);
    assert_eq!(refusal["reason"], "confirmation-required");
    assert_eq!(refusal["draft_id"], DRAFT_ID);
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_dry_run_names_the_inputs_and_the_credential_and_sends_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    for (args, command) in [
        (
            vec![
                "add",
                "Launch day photos",
                "--media-id",
                "1850000000000000020",
                "--reply-to",
                POST_ID,
            ],
            "drafts-add",
        ),
        (vec!["list", "-n", "5"], "drafts-list"),
        (vec!["remove", DRAFT_ID], "drafts-remove"),
    ] {
        let mut argv = vec!["--dry-run", "--output", "json", "drafts"];
        argv.extend(&args);
        let (code, stdout, stderr) = run(&store, &server.uri(), &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["command"], command, "{args:?}");
        assert_eq!(body["would_succeed"], true, "{args:?}");
        assert_eq!(body["auth"]["scheme"], "oauth1", "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_dry_run_of_add_leaves_out_the_reply_it_was_not_given() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--dry-run", "--output", "json", "drafts", "add", "Hello"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(json_out(&stdout).get("reply_to").is_none(), "{stdout}");
}

#[tokio::test]
async fn a_refused_input_names_its_rule_and_sends_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let too_many: Vec<String> = (1..=21).map(|n| format!("185000000000000{n:04}")).collect();

    let mut cases: Vec<(Vec<&str>, &str, Option<&str>)> = vec![
        (vec!["add", ""], "empty-body", None),
        (
            vec!["add", "Hello", "--reply-to", "not-a-post"],
            "invalid-post-id",
            Some("not-a-post"),
        ),
        (
            vec!["remove", "draft-1"],
            "invalid-draft-id",
            Some("draft-1"),
        ),
        (vec!["remove", ""], "invalid-draft-id", None),
    ];
    let mut crowded = vec!["add", "Hello"];
    for id in &too_many {
        crowded.extend(["--media-id", id]);
    }
    cases.push((crowded, "too-many-attachments", None));

    for (args, rule, value) in cases {
        let mut argv = vec!["--dry-run", "--output", "json", "drafts"];
        argv.extend(&args);
        let (code, stdout, stderr) = run(&store, &server.uri(), &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["rule"], rule, "{args:?}");
        assert_eq!(body["would_succeed"], false, "{args:?}");
        match value {
            Some(value) => assert_eq!(body["value"], value, "{args:?}"),
            None => assert!(body.get("value").is_none(), "{args:?}: {stdout}"),
        }
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
