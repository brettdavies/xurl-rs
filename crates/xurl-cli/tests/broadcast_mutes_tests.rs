//! `xr broadcasts mutes` reaches the chat-mute endpoints that span every
//! chat of a broadcaster: `list` reads who is muted with each expiry, `add`
//! resolves the handle and sends the expiry in milliseconds, `remove` names
//! the host's broadcast when a moderator gives one, and inputs the spec
//! bounds are refused before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const BROADCAST: &str = "1YqKDqWqdPLGV";
const END: &str = "2026-11-02T17:00:00Z";
const END_MS: &str = "1793638800000";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

async fn run_json(server: &MockServer, args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let mut argv = args.to_vec();
    argv.extend(["--output", "json"]);
    run(&store, &server.uri(), &argv).await
}

async fn mount_troll(server: &MockServer, times: u64) {
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "troll"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "99", "name": "troll", "username": "troll"}]
        })))
        .expect(times)
        .mount(server)
        .await;
}

#[tokio::test]
async fn list_reads_the_muted_users_with_each_expiry() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/broadcasts/chat/mutes"))
        .and(query_param("max_results", "50"))
        .and(query_param("broadcast_chat_mute.fields", "end_at_ms"))
        .and(query_param("expansions", "user_id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"user_id": "99", "end_at_ms": END_MS}, {"user_id": "100"}],
            "includes": {"users": [{"id": "99", "name": "troll", "username": "troll"}]},
            "meta": {"result_count": 2}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) =
        run_json(&server, &["broadcasts", "mutes", "list", "-n", "50"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["user_id"], "99");
    assert_eq!(body["data"][0]["end_at_ms"], END_MS);
    assert!(
        body["data"][1].get("end_at_ms").is_none(),
        "a mute with no end prints no expiry: {body}"
    );
    assert_eq!(body["includes"]["users"][0]["username"], "troll");
}

#[tokio::test]
async fn add_resolves_the_user_and_sends_the_expiry_in_milliseconds() {
    let server = MockServer::start().await;
    mount_troll(&server, 1).await;
    Mock::given(method("POST"))
        .and(path("/2/broadcasts/chat/mutes"))
        .and(body_json(
            serde_json::json!({"user_id": "99", "end_at_ms": END_MS}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"muted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &["broadcasts", "mutes", "add", "@troll", "--until", END],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["muted"], true);
}

#[tokio::test]
async fn a_moderator_names_the_hosts_broadcast() {
    let server = MockServer::start().await;
    mount_troll(&server, 2).await;
    Mock::given(method("POST"))
        .and(path("/2/broadcasts/chat/mutes"))
        .and(body_json(serde_json::json!({
            "user_id": "99",
            "broadcast_id": BROADCAST
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"muted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/2/broadcasts/chat/mutes/99"))
        .and(query_param("broadcast_id", BROADCAST))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"muted": false}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(
        &server,
        &[
            "broadcasts",
            "mutes",
            "add",
            "@troll",
            "--broadcast",
            BROADCAST,
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "broadcasts",
            "mutes",
            "remove",
            "@troll",
            "--broadcast",
            BROADCAST,
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["muted"], false);
}

#[tokio::test]
async fn remove_without_a_broadcast_unmutes_in_the_callers_own_chats() {
    let server = MockServer::start().await;
    mount_troll(&server, 1).await;
    Mock::given(method("DELETE"))
        .and(path("/2/broadcasts/chat/mutes/99"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"muted": false}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) =
        run_json(&server, &["broadcasts", "mutes", "remove", "@troll"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let requests = server.received_requests().await.expect("recorded");
    let delete = requests
        .iter()
        .find(|r| r.method.as_str() == "DELETE")
        .expect("the unmute was sent");
    assert_eq!(delete.url.query(), None, "no broadcast_id: {}", delete.url);
}

#[tokio::test]
async fn inputs_outside_the_spec_bounds_are_refused_before_any_request() {
    let server = MockServer::start().await;

    for (args, rule) in [
        (
            vec![
                "broadcasts",
                "mutes",
                "add",
                "@troll",
                "--until",
                "tomorrow",
            ],
            "invalid-time-window",
        ),
        (
            vec![
                "broadcasts",
                "mutes",
                "add",
                "@troll",
                "--broadcast",
                "not-an-id!",
            ],
            "invalid-broadcast-id",
        ),
        (
            vec![
                "broadcasts",
                "mutes",
                "remove",
                "@troll",
                "--broadcast",
                "not-an-id!",
            ],
            "invalid-broadcast-id",
        ),
    ] {
        let mut argv = vec!["--dry-run"];
        argv.extend(&args);
        let (code, stdout, stderr) = run_json(&server, &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{args:?}");
        assert_eq!(body["reason"], "validation", "{args:?}");
        assert_eq!(body["rule"], rule, "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_mute_removes_no_chat_message() {
    let server = MockServer::start().await;

    let (code, _stdout, stderr) = run_json(
        &server,
        &[
            "broadcasts",
            "mutes",
            "add",
            "@troll",
            "--broadcast",
            BROADCAST,
            "--message",
            "1850000000000000077",
        ],
    )
    .await;

    assert_eq!(code, 2, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "invalid-args", "stderr: {stderr}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
