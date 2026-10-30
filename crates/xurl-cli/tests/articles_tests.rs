//! `xr articles draft` turns a plain-text body, from a file or stdin, into
//! the content blocks X's Articles API takes and saves a draft; `xr articles
//! publish` publishes one, asking first. An empty title or body, and an
//! article id outside the spec's pattern, are refused before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const ARTICLE_ID: &str = "1850000000000000003";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn error_envelope(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr")
}

fn draft_request() -> serde_json::Value {
    serde_json::json!({
        "title": "Why Rust",
        "content_state": {
            "blocks": [
                {"text": "Why Rust", "type": "header-one"},
                {"text": "It is fast and it is safe.", "type": "unstyled"},
                {"text": "No garbage collector", "type": "unordered-list-item"},
                {"text": "No data races", "type": "unordered-list-item"}
            ],
            "entities": []
        }
    })
}

async fn answer_draft(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/2/articles/draft"))
        .and(body_json(draft_request()))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": ARTICLE_ID, "title": "Why Rust"}
        })))
        .expect(1)
        .mount(server)
        .await;
}

const BODY: &str =
    "# Why Rust\n\nIt is fast\nand it is safe.\n\n- No garbage collector\n- No data races\n";

#[tokio::test]
async fn draft_turns_the_body_file_into_blocks() {
    let server = MockServer::start().await;
    answer_draft(&server).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let body = tmp.path().join("article.md");
    std::fs::write(&body, BODY).expect("write the body");

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "articles",
            "draft",
            "--title",
            "Why Rust",
            "--body-file",
            body.to_str().expect("utf-8 path"),
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["id"], ARTICLE_ID);
}

#[tokio::test]
async fn draft_reads_the_body_from_stdin_given_a_dash() {
    let server = MockServer::start().await;
    answer_draft(&server).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let base_url = server.uri();

    let output = tokio::task::spawn_blocking(move || {
        common::xr_with_store(&store)
            .env("API_BASE_URL", &base_url)
            .args([
                "articles",
                "draft",
                "--title",
                "Why Rust",
                "--body-file",
                "-",
                "--output",
                "json",
            ])
            .write_stdin(BODY)
            .output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "stderr: {stderr}");
}

#[tokio::test]
async fn publish_asks_first_and_publishes_with_force() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/2/articles/{ARTICLE_ID}/publish")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"post_id": "1585341984679469056"}
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
            "--no-interactive",
            "--output",
            "json",
            "articles",
            "publish",
            ARTICLE_ID,
        ],
    )
    .await;
    assert_eq!(code, 1, "stderr: {stderr}");
    assert_eq!(error_envelope(&stderr)["reason"], "confirmation-required");

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--no-interactive",
            "--output",
            "json",
            "articles",
            "publish",
            ARTICLE_ID,
            "--force",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["post_id"], "1585341984679469056");
}

#[tokio::test]
async fn inputs_outside_the_spec_are_refused_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let body = tmp.path().join("article.md");
    std::fs::write(&body, BODY).expect("write the body");
    let blank = tmp.path().join("blank.md");
    std::fs::write(&blank, "\n\n   \n").expect("write the blank body");
    let body = body.to_str().expect("utf-8 path");
    let blank = blank.to_str().expect("utf-8 path");

    for (args, reason) in [
        (
            vec!["articles", "draft", "--title", "", "--body-file", body],
            "empty-article-title",
        ),
        (
            vec![
                "articles",
                "draft",
                "--title",
                "Why Rust",
                "--body-file",
                blank,
            ],
            "empty-article-body",
        ),
        (vec!["articles", "publish", "draft-1"], "invalid-article-id"),
    ] {
        let mut argv = vec!["--dry-run", "--output", "json"];
        argv.extend(&args);
        let (code, stdout, stderr) = run(&store, &server.uri(), &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{args:?}");
        assert_eq!(body["rule"], reason, "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
