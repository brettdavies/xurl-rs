//! `xr notes` over X's AI Note Writer API: write a Community Note, score a
//! draft, delete a note, and list the posts eligible for one and the notes
//! the caller wrote. `--test` is the spec's `test_mode`, false unless given;
//! deleting asks first; an app X has not enrolled gets the enrollment step.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const POST_ID: &str = "1585341984679469056";
const NOTE_ID: &str = "1850000000000000002";
const TEXT: &str = "The photo is from 2019: https://example.com/source";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn error_envelope(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr")
}

async fn run_json(server: &MockServer, args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let mut argv = args.to_vec();
    argv.extend(["--output", "json"]);
    run(&store, &server.uri(), &argv).await
}

#[tokio::test]
async fn write_sends_the_note_and_test_mode() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/notes"))
        .and(body_json(serde_json::json!({
            "test_mode": true,
            "post_id": POST_ID,
            "info": {
                "text": TEXT,
                "classification": "misinformed_or_potentially_misleading",
                "misleading_tags": ["outdated_information", "missing_important_context"],
                "trustworthy_sources": true
            }
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({"data": {"id": NOTE_ID}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "notes",
            "write",
            &format!("https://x.com/alice/status/{POST_ID}"),
            "--text",
            TEXT,
            "--classification",
            "misinformed_or_potentially_misleading",
            "--tag",
            "outdated_information",
            "--tag",
            "missing_important_context",
            "--trustworthy-sources",
            "--test",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["id"], NOTE_ID);
}

#[tokio::test]
async fn evaluate_scores_a_draft() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/notes/evaluate"))
        .and(body_json(
            serde_json::json!({"post_id": POST_ID, "note_text": TEXT}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"claim_opinion_score": 0.82}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) =
        run_json(&server, &["notes", "evaluate", POST_ID, "--text", TEXT]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["claim_opinion_score"], 0.82);
}

#[tokio::test]
async fn eligible_and_written_send_test_mode_false_unless_asked() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/notes/search/posts_eligible_for_notes"))
        .and(query_param("test_mode", "false"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": POST_ID, "text": "A claim"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/notes/search/notes_written"))
        .and(query_param("test_mode", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": NOTE_ID, "status": "needs_more_ratings", "info": {"text": TEXT, "post_id": POST_ID}}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(&server, &["notes", "eligible"]).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["id"], POST_ID);

    let (code, stdout, stderr) = run_json(&server, &["notes", "written", "--test"]).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["info"]["post_id"], POST_ID);
}

#[tokio::test]
async fn remove_asks_first_and_deletes_with_force() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/notes/{NOTE_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) =
        run_json(&server, &["--no-interactive", "notes", "remove", NOTE_ID]).await;
    assert_eq!(code, 1, "stderr: {stderr}");
    assert_eq!(error_envelope(&stderr)["reason"], "confirmation-required");

    let (code, _stdout, stderr) = run_json(
        &server,
        &["--no-interactive", "notes", "remove", NOTE_ID, "--force"],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn an_app_x_has_not_enrolled_gets_the_enrollment_step() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/notes/search/notes_written"))
        .respond_with(ResponseTemplate::new(403).set_body_string(
            r#"{"title":"Unsupported Authentication","detail":"Your client is not enrolled: client-not-enrolled","status":403}"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(&server, &["notes", "written"]).await;

    assert_ne!(code, 0, "stderr: {stderr}");
    assert_eq!(error_envelope(&stderr)["next_step"]["action"], "enroll-app");
}

#[tokio::test]
async fn inputs_outside_the_spec_are_refused_before_any_request() {
    let server = MockServer::start().await;

    for (args, reason) in [
        (
            vec![
                "notes",
                "write",
                POST_ID,
                "--text",
                "No source here",
                "--classification",
                "not_misleading",
            ],
            "note-needs-source-url",
        ),
        (vec!["notes", "remove", "note-1"], "invalid-note-id"),
        (
            vec!["notes", "evaluate", "", "--text", TEXT],
            "empty-post-id",
        ),
    ] {
        let mut argv = vec!["--dry-run"];
        argv.extend(&args);
        let (code, stdout, stderr) = run_json(&server, &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{args:?}");
        assert_eq!(body["reason"], reason, "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
