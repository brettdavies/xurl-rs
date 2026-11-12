//! `xr post --clip` shares a clip of another post's video: it sends the
//! `clip` object beside the text, resolves a post URL to its id, refuses a
//! post id or a range the spec's schema would not take before any request,
//! answers `--dry-run` with the rule it would break, and takes all three
//! flags or none and no media.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const SOURCE_ID: &str = "1585341984679469056";

fn json_of(text: &str) -> serde_json::Value {
    serde_json::from_str(text.trim()).expect("JSON")
}

async fn refuse_me_lookups(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(server)
        .await;
}

async fn answer_clip_post(server: &MockServer, start_seconds: u64, end_seconds: u64) {
    Mock::given(method("POST"))
        .and(path("/2/tweets"))
        .and(body_json(serde_json::json!({
            "text": "Worth watching",
            "clip": {
                "post_id": SOURCE_ID,
                "start_seconds": start_seconds,
                "end_seconds": end_seconds,
            }
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(
                serde_json::json!({"data": {"id": "55555", "text": "Worth watching"}}),
            ),
        )
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn post_with_a_clip_sends_the_clip_beside_the_text() {
    let server = MockServer::start().await;
    refuse_me_lookups(&server).await;
    answer_clip_post(&server, 30, 60).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "post",
            "Worth watching",
            "--clip",
            SOURCE_ID,
            "--clip-start",
            "30",
            "--clip-end",
            "60",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_of(&stdout)["data"]["id"], "55555");
}

#[tokio::test]
async fn post_with_a_clip_resolves_a_post_url_to_its_id() {
    let server = MockServer::start().await;
    refuse_me_lookups(&server).await;
    answer_clip_post(&server, 0, 5).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let url = format!("https://x.com/alice/status/{SOURCE_ID}");

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "post",
            "Worth watching",
            "--clip",
            &url,
            "--clip-start",
            "0",
            "--clip-end",
            "5",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn a_dry_run_names_the_clip_and_sends_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "post",
            "Worth watching",
            "--clip",
            SOURCE_ID,
            "--clip-start",
            "30",
            "--clip-end",
            "60",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let v = json_of(&stdout);
    assert_eq!(v["would_succeed"], true, "{v}");
    assert_eq!(
        v["clip"],
        serde_json::json!({"post_id": SOURCE_ID, "start_seconds": 30, "end_seconds": 60}),
        "{v}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_dry_run_of_a_post_without_a_clip_carries_no_clip_key() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--dry-run", "--output", "json", "post", "Plain"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(json_of(&stdout).get("clip").is_none(), "{stdout}");
}

#[tokio::test]
async fn a_dry_run_reports_the_rule_a_bad_clip_breaks() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    for (id, start, end, rule, sentence) in [
        (
            "not-an-id",
            "0",
            "10",
            "invalid-clip-post-id",
            "A clip's post id is 1 to 19 digits, or a post URL.",
        ),
        (
            SOURCE_ID,
            "30",
            "30",
            "invalid-clip-range",
            "A clip ends after it starts: --clip-end is greater than --clip-start.",
        ),
    ] {
        let (code, stdout, stderr) = run(
            &store,
            &server.uri(),
            &[
                "--dry-run",
                "--output",
                "json",
                "post",
                "Worth watching",
                "--clip",
                id,
                "--clip-start",
                start,
                "--clip-end",
                end,
            ],
        )
        .await;

        assert_eq!(code, 0, "{rule}: {stderr}");
        let v = json_of(&stdout);
        assert_eq!(v["would_succeed"], false, "{v}");
        assert_eq!(v["reason"], "validation", "{v}");
        assert_eq!(v["rule"], rule, "{v}");
        assert_eq!(v["message"], sentence, "{v}");
        assert_eq!(v["exit_code"], 1, "{v}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_bad_clip_is_refused_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "post",
            "Worth watching",
            "--clip",
            SOURCE_ID,
            "--clip-start",
            "60",
            "--clip-end",
            "30",
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope = json_of(&stderr);
    assert_eq!(envelope["reason"], "validation", "{envelope}");
    assert_eq!(envelope["rule"], "invalid-clip-range", "{envelope}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn the_clip_flags_come_together_and_exclude_media() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    for args in [
        &["--clip", SOURCE_ID][..],
        &["--clip", SOURCE_ID, "--clip-start", "0"][..],
        &["--clip", SOURCE_ID, "--clip-end", "10"][..],
        &["--clip-start", "0", "--clip-end", "10"][..],
        &["--clip-start", "0"][..],
        &["--clip-end", "10"][..],
        &[
            "--clip",
            SOURCE_ID,
            "--clip-start",
            "0",
            "--clip-end",
            "10",
            "--media-id",
            "5",
        ][..],
        &["--clip", SOURCE_ID, "--clip-start=-1", "--clip-end", "10"][..],
        &[
            "--clip",
            SOURCE_ID,
            "--clip-start",
            "0",
            "--clip-end",
            "ten",
        ][..],
    ] {
        let mut argv = vec!["--output", "json", "post", "Worth watching"];
        argv.extend(args);
        let (code, _stdout, stderr) = run(&store, &server.uri(), &argv).await;
        assert_eq!(code, 2, "{args:?}: {stderr}");
        let envelope = json_of(&stderr);
        assert_eq!(envelope["reason"], "invalid-args", "{args:?}");
        let message = envelope["message"].as_str().expect("a message");
        assert!(
            !message.contains("unexpected argument"),
            "{args:?}: a flag the command knows, refused for how it is combined: {message}"
        );
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
