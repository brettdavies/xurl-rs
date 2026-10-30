//! `xr broadcasts` beyond moderators and sources: the caller's broadcasts,
//! scheduled broadcasts (whose RFC 3339 times go out as the millisecond
//! strings X takes), and a broadcast's chat. Cancelling a scheduled broadcast
//! and removing a chat message ask first; inputs the spec bounds are refused
//! before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const BROADCAST: &str = "1YqKDqWqdPLGV";
const SCHEDULED: &str = "1850000000000000001";
const START: &str = "2026-11-02T16:00:00Z";
const START_MS: &str = "1793635200000";
const END: &str = "2026-11-02T17:00:00Z";
const END_MS: &str = "1793638800000";

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

fn scheduled() -> serde_json::Value {
    serde_json::json!({
        "data": {
            "scheduled_broadcast_id": SCHEDULED,
            "source_id": "abc123",
            "scheduled_start_ms": START_MS,
            "state": "Scheduled"
        }
    })
}

#[tokio::test]
async fn list_and_get_read_the_callers_broadcasts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/broadcasts"))
        .and(query_param("max_results", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": BROADCAST, "title": "Launch", "state": "ENDED"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/2/broadcasts/{BROADCAST}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": BROADCAST, "title": "Launch", "total_watched": "1200"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(&server, &["broadcasts", "list", "-n", "20"]).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["title"], "Launch");

    let (code, stdout, stderr) = run_json(&server, &["broadcasts", "get", BROADCAST]).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["total_watched"], "1200");
}

#[tokio::test]
async fn scheduled_add_sends_its_times_as_milliseconds() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/broadcasts/scheduled"))
        .and(body_json(serde_json::json!({
            "source_id": "abc123",
            "scheduled_start_ms": START_MS,
            "scheduled_end_ms": END_MS,
            "title": "Launch"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(scheduled()))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "broadcasts",
            "scheduled",
            "add",
            "--source",
            "abc123",
            "--start",
            START,
            "--end",
            END,
            "--title",
            "Launch",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(
        json_out(&stdout)["data"]["scheduled_broadcast_id"],
        SCHEDULED
    );
}

#[tokio::test]
async fn scheduled_update_names_the_broadcast_and_both_times() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/broadcasts/scheduled/{SCHEDULED}")))
        .and(body_json(serde_json::json!({
            "scheduled_broadcast_id": SCHEDULED,
            "scheduled_start_ms": START_MS,
            "scheduled_end_ms": END_MS
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(scheduled()))
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(
        &server,
        &[
            "broadcasts",
            "scheduled",
            "update",
            SCHEDULED,
            "--start",
            START,
            "--end",
            END,
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn cancelling_asks_first() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/broadcasts/scheduled/{SCHEDULED}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(
        &server,
        &[
            "--no-interactive",
            "broadcasts",
            "scheduled",
            "cancel",
            SCHEDULED,
        ],
    )
    .await;
    assert_eq!(code, 1, "stderr: {stderr}");
    assert_eq!(error_envelope(&stderr)["reason"], "confirmation-required");

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "--no-interactive",
            "broadcasts",
            "scheduled",
            "cancel",
            SCHEDULED,
            "--force",
        ],
    )
    .await;
    assert_eq!(code, 0, "--force: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["deleted"], true);
}

/// X publishes every schedule `xr` makes at its start time, so the flag
/// that held one back for a manual start is refused before any request.
#[tokio::test]
async fn scheduling_for_a_manual_start_is_refused() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/broadcasts/scheduled"))
        .respond_with(ResponseTemplate::new(201).set_body_json(scheduled()))
        .expect(0)
        .mount(&server)
        .await;

    for verb in ["add", "update"] {
        let mut args = vec!["broadcasts", "scheduled", verb];
        if verb == "update" {
            args.push(SCHEDULED);
        } else {
            args.extend(["--source", "abc123"]);
        }
        args.extend(["--start", START, "--end", END, "--manual-publish"]);
        let (code, _stdout, stderr) = run_json(&server, &args).await;
        assert_eq!(code, 2, "{verb}: {stderr}");
        assert_eq!(error_envelope(&stderr)["reason"], "invalid-args", "{verb}");
    }
}

#[tokio::test]
async fn chat_send_history_and_remove_reach_the_broadcasts_chat() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/2/broadcasts/{BROADCAST}/chat")))
        .and(body_json(
            serde_json::json!({"text": "Welcome!", "reply_to": "1700000000000"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"success": true, "timestamp": "1700000000001"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/2/broadcasts/{BROADCAST}/chat")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "1700000000001", "text": "Welcome!", "author_username": "alice"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!(
            "/2/broadcasts/{BROADCAST}/chat/1700000000001"
        )))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "broadcasts",
            "chat",
            "send",
            BROADCAST,
            "Welcome!",
            "--reply-to",
            "1700000000000",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["success"], true);

    let (code, stdout, stderr) =
        run_json(&server, &["broadcasts", "chat", "history", BROADCAST]).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["author_username"], "alice");

    let (code, _stdout, stderr) = run_json(
        &server,
        &[
            "--no-interactive",
            "broadcasts",
            "chat",
            "remove",
            BROADCAST,
            "1700000000001",
            "--force",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn chat_mute_resolves_the_user_and_sends_the_expiry_in_milliseconds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "troll"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "99", "name": "troll", "username": "troll"}]
        })))
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/2/broadcasts/{BROADCAST}/chat/mutes")))
        .and(body_json(
            serde_json::json!({"user_id": "99", "end_at_ms": END_MS}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"muted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/broadcasts/{BROADCAST}/chat/mutes/99")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"muted": false}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "broadcasts",
            "chat",
            "mute",
            BROADCAST,
            "@troll",
            "--until",
            END,
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["muted"], true);

    let (code, _stdout, stderr) = run_json(
        &server,
        &["broadcasts", "chat", "unmute", BROADCAST, "@troll"],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn inputs_outside_the_spec_bounds_are_refused_before_any_request() {
    let server = MockServer::start().await;
    let long_message = "x".repeat(141);

    for (args, reason) in [
        (
            vec!["broadcasts", "chat", "send", BROADCAST, &long_message],
            "chat-message-too-long",
        ),
        (
            vec!["broadcasts", "chat", "send", BROADCAST, ""],
            "empty-body",
        ),
        (
            vec!["broadcasts", "get", "not-an-id!"],
            "invalid-broadcast-id",
        ),
        (
            vec!["broadcasts", "scheduled", "get", BROADCAST],
            "invalid-scheduled-broadcast-id",
        ),
        (
            vec!["broadcasts", "chat", "remove", BROADCAST, "msg-1"],
            "invalid-chat-message-id",
        ),
        (
            vec![
                "broadcasts",
                "scheduled",
                "add",
                "--source",
                "abc123",
                "--start",
                END,
                "--end",
                START,
            ],
            "invalid-time-window",
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
