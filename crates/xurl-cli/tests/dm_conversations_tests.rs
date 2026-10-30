//! The direct-message conversation commands: `xr conversations` starts a
//! group, sends into a conversation, and reads a conversation's history or
//! the one with a user; `xr dm-events` reads or removes one message; and
//! `xr dm-block` and `xr dm-unblock` put the named user's id in the path.
//! Inputs the spec bounds are refused before any request, and removing a
//! message asks first.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const CONVERSATION: &str = "1580705921830768643-1580705921830768644";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn error_envelope(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr")
}

async fn answer_lookup(server: &MockServer, username: &str, id: &str) {
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", username))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": id, "name": username, "username": username}]
        })))
        .expect(1)
        .mount(server)
        .await;
}

fn sent(conversation: &str) -> ResponseTemplate {
    ResponseTemplate::new(201).set_body_json(serde_json::json!({
        "data": {"dm_conversation_id": conversation, "dm_event_id": "1580705921830768647"}
    }))
}

#[tokio::test]
async fn start_resolves_every_handle_in_one_lookup_and_creates_a_group() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "alice,bob"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"id": "11", "name": "Alice", "username": "alice"},
                {"id": "12", "name": "Bob", "username": "Bob"}
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/dm_conversations"))
        .and(body_json(serde_json::json!({
            "conversation_type": "Group",
            "participant_ids": ["11", "12", "22"],
            "message": {"text": "hi"}
        })))
        .respond_with(sent("1580705921830768650"))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "conversations",
            "start",
            "@alice",
            "bob",
            "22",
            "--text",
            "hi",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(
        json_out(&stdout)["data"]["dm_conversation_id"],
        "1580705921830768650"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 2);
}

#[tokio::test]
async fn start_refuses_a_group_outside_two_to_forty_nine_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let fifty: Vec<String> = (1..=50).map(|n| n.to_string()).collect();

    for participants in [&fifty[..1], &fifty[..]] {
        let args: Vec<&str> = ["--dry-run", "--output", "json", "conversations", "start"]
            .into_iter()
            .chain(participants.iter().map(String::as_str))
            .chain(["--text", "hi"])
            .collect();
        let (code, stdout, stderr) = run(&store, &server.uri(), &args).await;

        assert_eq!(code, 0, "{} participants: {stderr}", participants.len());
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false);
        assert_eq!(body["rule"], "invalid-participant-count");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn send_posts_the_text_into_the_conversation() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/2/dm_conversations/{CONVERSATION}/messages")))
        .and(body_json(serde_json::json!({"text": "hello"})))
        .respond_with(sent(CONVERSATION))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "conversations",
            "send",
            CONVERSATION,
            "hello",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn history_reads_the_conversations_events() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/2/dm_conversations/{CONVERSATION}/dm_events"
        )))
        .and(query_param("max_results", "30"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "1580705921830768647", "event_type": "MessageCreate", "text": "hello"}]
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
            "conversations",
            "history",
            CONVERSATION,
            "-n",
            "30",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["text"], "hello");
}

#[tokio::test]
async fn with_resolves_the_user_then_reads_that_conversation() {
    let server = MockServer::start().await;
    answer_lookup(&server, "alice", "11").await;
    Mock::given(method("GET"))
        .and(path("/2/dm_conversations/with/11/dm_events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "1580705921830768647", "event_type": "MessageCreate", "text": "hi"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["conversations", "with", "@alice", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn ids_outside_the_spec_patterns_are_refused_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    for (args, reason) in [
        (
            &["conversations", "history", "abc"][..],
            "invalid-conversation-id",
        ),
        (
            &["conversations", "send", "123", "hi"][..],
            "invalid-conversation-id",
        ),
        (&["dm-events", "get", "dm-1"][..], "invalid-dm-event-id"),
    ] {
        let argv: Vec<&str> = ["--dry-run", "--output", "json"]
            .iter()
            .chain(args)
            .copied()
            .collect();
        let (code, stdout, stderr) = run(&store, &server.uri(), &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{args:?}");
        assert_eq!(body["rule"], reason, "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn dm_events_get_reads_one_message() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/dm_events/1580705921830768647"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "1580705921830768647", "event_type": "MessageCreate", "text": "hello"}
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
            "dm-events",
            "get",
            "1580705921830768647",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["text"], "hello");
}

#[tokio::test]
async fn dm_events_remove_asks_first_and_deletes_with_force() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/2/dm_events/1580705921830768647"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
        )
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
            "dm-events",
            "remove",
            "1580705921830768647",
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
            "dm-events",
            "remove",
            "1580705921830768647",
            "--force",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["deleted"], true);
}

#[tokio::test]
async fn dm_block_and_unblock_put_the_named_users_id_in_the_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "alice"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "11", "name": "alice", "username": "alice"}]
        })))
        .expect(2)
        .mount(&server)
        .await;
    for (verb, blocked) in [("block", true), ("unblock", false)] {
        Mock::given(method("POST"))
            .and(path(format!("/2/users/11/dm/{verb}")))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"blocked": blocked}})),
            )
            .expect(1)
            .mount(&server)
            .await;
    }
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    for (command, blocked) in [("dm-block", true), ("dm-unblock", false)] {
        let (code, stdout, stderr) = run(
            &store,
            &server.uri(),
            &[command, "@alice", "--output", "json"],
        )
        .await;
        assert_eq!(code, 0, "{command}: {stderr}");
        assert_eq!(json_out(&stdout)["data"]["blocked"], blocked, "{command}");
    }
}
