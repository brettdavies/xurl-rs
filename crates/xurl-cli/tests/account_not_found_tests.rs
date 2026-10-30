//! A handle X knows no account for is `not-found` (exit 4) with the handle,
//! without its `@`, in `account` and a sentence in `message`, on every path
//! that looks a handle up. X answers a lookup that finds none of its handles
//! with a 200 carrying `errors` and no `data`, and one that finds some with
//! `data` beside `errors`; both name the first handle it did not find, and
//! nothing past the lookup is sent. A dry run looks nothing up.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn error_envelope(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr")
}

/// The entry X puts in `errors` for a handle it found no account for.
fn not_found(handle: &str) -> serde_json::Value {
    serde_json::json!({
        "value": handle,
        "detail": format!("Could not find user with usernames: [{handle}]."),
        "title": "Not Found Error",
        "resource_type": "user",
        "parameter": "usernames",
        "resource_id": handle,
        "type": "https://api.x.com/2/problems/resource-not-found"
    })
}

/// X's answer to a lookup that found no account for any of `handles`.
fn none_found(handles: &[&str]) -> ResponseTemplate {
    let errors: Vec<serde_json::Value> = handles.iter().map(|h| not_found(h)).collect();
    ResponseTemplate::new(200).set_body_json(serde_json::json!({"errors": errors}))
}

/// A server that knows the caller and finds no account for `handles`, by
/// either lookup endpoint.
async fn server_finding_none_of(handles: &[&str]) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "99", "name": "Alice", "username": "alice"}
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", handles.join(",")))
        .respond_with(none_found(handles))
        .mount(&server)
        .await;
    for handle in handles {
        Mock::given(method("GET"))
            .and(path(format!("/2/users/by/username/{handle}")))
            .respond_with(none_found(&[handle]))
            .mount(&server)
            .await;
    }
    server
}

async fn run_json(server: &MockServer, args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let mut argv = vec!["--output", "json"];
    argv.extend(args);
    run(&store, &server.uri(), &argv).await
}

/// Asserts the refusal names `handle` in `account` and in a sentence.
fn assert_names(code: i32, stderr: &str, handle: &str) {
    assert_eq!(code, 4, "stderr: {stderr}");
    let envelope = error_envelope(stderr);
    assert_eq!(envelope["reason"], "not-found", "{envelope}");
    assert_eq!(envelope["account"], handle, "{envelope}");
    assert_eq!(
        envelope["message"],
        format!("X found no account for the handle @{handle}."),
        "{envelope}"
    );
    assert_eq!(envelope["next_step"]["action"], "fix-input", "{envelope}");
}

/// Asserts every request `server` saw was a user lookup: nothing the
/// command meant to do with the account went out.
async fn assert_only_lookups(server: &MockServer) {
    let sent: Vec<String> = server
        .received_requests()
        .await
        .expect("recorded")
        .iter()
        .map(|req| format!("{} {}", req.method, req.url.path()))
        .collect();
    assert!(
        sent.iter().all(|r| r.starts_with("GET /2/users/")),
        "{sent:?}"
    );
}

#[rstest::rstest]
#[case::follow(&["follow", "ghost"])]
#[case::follow_with_at(&["follow", "@ghost"])]
#[case::followers_of(&["followers", "--of", "ghost"])]
#[case::user_posts(&["user-posts", "ghost"])]
#[case::lists_members_add(&["lists", "members", "add", "1146654567674912769", "ghost"])]
#[case::dm(&["dm", "ghost", "hi"])]
#[case::spaces_by_creator(&["spaces", "by-creator", "ghost"])]
#[case::users(&["users", "ghost"])]
#[case::user(&["user", "ghost"])]
#[case::user_with_at(&["user", "@ghost"])]
#[tokio::test]
async fn a_handle_x_finds_no_account_for_is_named_in_account(#[case] args: &[&str]) {
    let server = server_finding_none_of(&["ghost"]).await;

    let (code, _stdout, stderr) = run_json(&server, args).await;

    assert_names(code, &stderr, "ghost");
    assert_only_lookups(&server).await;
}

#[rstest::rstest]
#[case::spaces_by_creator(&["spaces", "by-creator", "12", "ghost", "@phantom"])]
#[case::conversations_start(&["conversations", "start", "ghost", "@phantom", "--text", "hi"])]
#[case::users(&["users", "ghost", "@phantom"])]
#[tokio::test]
async fn several_handles_none_of_which_resolve_name_the_first(#[case] args: &[&str]) {
    let server = server_finding_none_of(&["ghost", "phantom"]).await;

    let (code, _stdout, stderr) = run_json(&server, args).await;

    assert_names(code, &stderr, "ghost");
    assert_only_lookups(&server).await;
}

#[tokio::test]
async fn a_lookup_that_finds_some_handles_names_the_first_it_did_not() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "jack,ghost,phantom"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "12", "name": "Jack", "username": "jack"}],
            "errors": [not_found("ghost"), not_found("phantom")]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(
        &server,
        &["spaces", "by-creator", "@jack", "ghost", "@phantom"],
    )
    .await;

    assert_names(code, &stderr, "ghost");
    assert_only_lookups(&server).await;
}

#[tokio::test]
async fn users_prints_the_accounts_a_lookup_found_beside_those_it_did_not() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "jack,ghost"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "12", "name": "Jack", "username": "jack"}],
            "errors": [not_found("ghost")]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(&server, &["users", "jack", "ghost"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["id"], "12", "{body}");
    assert_eq!(body["errors"][0]["value"], "ghost", "{body}");
}

#[tokio::test]
async fn text_output_names_the_handle_in_a_sentence() {
    let server = server_finding_none_of(&["ghost"]).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, _stdout, stderr) = run(&store, &server.uri(), &["follow", "@ghost"]).await;

    assert_eq!(code, 4, "stderr: {stderr}");
    assert!(
        stderr.contains("X found no account for the handle @ghost."),
        "{stderr}"
    );
    assert!(!stderr.contains("errors"), "X's body leaked: {stderr}");
}

#[rstest::rstest]
#[case::follow(&["follow", "ghost"])]
#[case::spaces_by_creator(&["spaces", "by-creator", "ghost", "phantom"])]
#[case::conversations_start(&["conversations", "start", "ghost", "phantom", "--text", "hi"])]
#[case::user(&["user", "ghost"])]
#[tokio::test]
async fn a_dry_run_looks_no_handle_up(#[case] args: &[&str]) {
    let server = MockServer::start().await;
    let mut argv = vec!["--dry-run"];
    argv.extend(args);

    let (code, stdout, stderr) = run_json(&server, &argv).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let answer = json_out(&stdout);
    assert_eq!(answer["status"], "dry_run", "{answer}");
    assert_eq!(answer["would_succeed"], true, "{answer}");
    assert!(answer.get("account").is_none(), "{answer}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
