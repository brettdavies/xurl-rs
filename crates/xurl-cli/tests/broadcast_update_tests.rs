//! `xr broadcasts update` changes a broadcast's title or who can chat through
//! `PUT /2/broadcasts/{id}`: it sends only the fields its flags name, refuses
//! an update that changes nothing or breaks a bound before any request, and
//! never reaches the live-session endpoints.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth1_store, run_in_process as run};

const BROADCAST_ID: &str = "1YqKDqWqdPLGV";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn updated(title: &str, chat_option: u8) -> serde_json::Value {
    serde_json::json!({"data": {
        "id": BROADCAST_ID,
        "broadcast_id": BROADCAST_ID,
        "state": "RUNNING",
        "share_url": "https://x.com/i/broadcasts/1YqKDqWqdPLGV",
        "title": title,
        "chat_option": chat_option
    }})
}

#[tokio::test]
async fn update_puts_the_title_and_the_chat_option() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/broadcasts/{BROADCAST_ID}")))
        .and(body_json(
            serde_json::json!({"title": "Second take", "chat_option": 3}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(updated("Second take", 3)))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "update",
            BROADCAST_ID,
            "--title",
            "Second take",
            "--chat-option",
            "3",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"]["title"], "Second take");
    assert_eq!(body["data"]["chat_option"], 3);
}

#[tokio::test]
async fn update_sends_only_the_flags_it_was_given() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/broadcasts/{BROADCAST_ID}")))
        .and(body_json(serde_json::json!({"chat_option": 0})))
        .respond_with(ResponseTemplate::new(200).set_body_json(updated("Kept", 0)))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "update",
            BROADCAST_ID,
            "--chat-option",
            "0",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn update_refuses_what_changes_nothing_or_breaks_a_bound_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    for (args, rule) in [
        (vec![BROADCAST_ID], "no-broadcast-changes"),
        (vec![BROADCAST_ID, "--title", "  "], "empty-broadcast-title"),
        (
            vec![BROADCAST_ID, "--chat-option", "7"],
            "invalid-chat-option",
        ),
        (
            vec!["not a broadcast id", "--title", "x"],
            "invalid-broadcast-id",
        ),
    ] {
        let mut argv = vec!["--dry-run", "--output", "json", "broadcasts", "update"];
        argv.extend(&args);
        let (code, stdout, stderr) = run(&store, &server.uri(), &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{args:?}");
        assert_eq!(body["rule"], rule, "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn update_under_dry_run_sends_nothing_and_names_the_credential() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "broadcasts",
            "update",
            BROADCAST_ID,
            "--title",
            "Second take",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "broadcasts-update");
    assert_eq!(body["would_succeed"], true);
    assert_eq!(body["auth"]["scheme"], "oauth1");
    assert_eq!(body["title"], "Second take");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn update_does_not_take_a_state_or_a_chat_message() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    for flag in ["--state", "--end", "--message"] {
        let (code, _stdout, stderr) = run(
            &store,
            &server.uri(),
            &[
                "--output",
                "json",
                "broadcasts",
                "update",
                BROADCAST_ID,
                flag,
                "x",
            ],
        )
        .await;
        assert_eq!(code, 2, "{flag}: {stderr}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_refusal_names_the_input_and_a_dry_run_leaves_out_the_flags_not_given() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    for (args, rule, value) in [
        (
            vec![BROADCAST_ID, "--chat-option", "7"],
            "invalid-chat-option",
            "7",
        ),
        (
            vec!["not a broadcast id", "--title", "x"],
            "invalid-broadcast-id",
            "not a broadcast id",
        ),
    ] {
        let mut argv = vec!["--dry-run", "--output", "json", "broadcasts", "update"];
        argv.extend(&args);
        let (code, stdout, stderr) = run(&store, &server.uri(), &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["rule"], rule, "{args:?}");
        assert_eq!(body["value"], value, "{args:?}");
    }

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "broadcasts",
            "update",
            BROADCAST_ID,
            "--title",
            "Second take",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(json_out(&stdout).get("chat_option").is_none(), "{stdout}");
}
