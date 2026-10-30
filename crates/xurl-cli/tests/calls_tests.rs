//! `xr calls` reaches the call endpoints through the runner: each verb sends
//! the request the spec gives its endpoint and prints the typed response, a
//! `cancel` asks before it cancels, and a time or a change set the spec
//! refuses is refused before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth1_store, run_in_process as run};

const CALL_ID: &str = "8a6f1c2e-call";
const SCHEDULED_ID: &str = "1849302018273645";
const START: &str = "2026-10-23T15:00:00Z";

fn scheduled_call() -> serde_json::Value {
    serde_json::json!({
        "id": SCHEDULED_ID,
        "call_id": CALL_ID,
        "title": "Standup",
        "start_time": START,
        "status": "scheduled",
        "invitees": [{"id": "783214", "response": "none"}]
    })
}

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn error_envelope(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr")
}

#[tokio::test]
async fn get_reads_the_call() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/calls/{CALL_ID}")))
        .and(query_param("expansions", "host_id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": CALL_ID, "title": "Standup", "participant_count": 3}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["calls", "get", CALL_ID, "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["participant_count"], 3);
}

#[tokio::test]
async fn scheduled_list_sends_the_window_and_the_count() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/calls/scheduled"))
        .and(query_param("start_time", START))
        .and(query_param("end_time", "2026-10-30T15:00:00Z"))
        .and(query_param("max_results", "50"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": [scheduled_call()]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "calls",
            "scheduled",
            "list",
            "--start",
            START,
            "--end",
            "2026-10-30T15:00:00Z",
            "-n",
            "50",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["id"], SCHEDULED_ID);
}

/// The page size comes from `-n`, then the global `--limit`, then 20, and a
/// dry run reports the size the real run sends.
#[rstest::rstest]
#[case::neither(&[], 20)]
#[case::limit(&["--limit", "50"], 50)]
#[case::n_over_limit(&["-n", "30", "--limit", "50"], 30)]
#[tokio::test]
async fn scheduled_list_takes_its_page_size_from_n_then_limit(
    #[case] flags: &[&str],
    #[case] size: i32,
) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/calls/scheduled"))
        .and(query_param("max_results", size.to_string()))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": [scheduled_call()]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let mut args = vec!["calls", "scheduled", "list", "--output", "json"];
    args.extend_from_slice(flags);

    let (code, _stdout, stderr) = run(&store, &server.uri(), &args).await;
    assert_eq!(code, 0, "stderr: {stderr}");

    let mut dry = vec!["--dry-run"];
    dry.extend_from_slice(&args);
    let (code, stdout, stderr) = run(&store, &server.uri(), &dry).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["max_results"], size, "{stdout}");
}

#[tokio::test]
async fn scheduled_add_posts_the_call_with_its_invitees() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/calls/scheduled"))
        .and(body_json(serde_json::json!({
            "start_time": START,
            "title": "Standup",
            "duration_minutes": 15,
            "invitee_ids": ["783214", "2244994945"],
            "recurrence": "FREQ=WEEKLY;BYDAY=MO",
            "time_zone": "America/Chicago",
            "notify_invitees": false
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({"data": scheduled_call()})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "calls",
            "scheduled",
            "add",
            "--start",
            START,
            "--title",
            "Standup",
            "--duration",
            "15",
            "--invite",
            "783214",
            "--invite",
            "2244994945",
            "--recurrence",
            "FREQ=WEEKLY;BYDAY=MO",
            "--time-zone",
            "America/Chicago",
            "--no-notify",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["call_id"], CALL_ID);
}

#[tokio::test]
async fn scheduled_add_refuses_a_start_that_is_not_rfc_3339_before_any_request() {
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
            "calls",
            "scheduled",
            "add",
            "--start",
            "2026-10-23 15:00",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "calls-scheduled-add");
    assert_eq!(body["would_succeed"], false);
    assert_eq!(body["reason"], "invalid-call-time");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn scheduled_get_reads_one_scheduled_call() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/calls/scheduled/{SCHEDULED_ID}")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": scheduled_call()})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "calls",
            "scheduled",
            "get",
            SCHEDULED_ID,
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["invitees"][0]["response"], "none");
}

#[tokio::test]
async fn scheduled_update_puts_the_changes_for_the_named_occurrences() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/calls/scheduled/{SCHEDULED_ID}")))
        .and(body_json(serde_json::json!({
            "start_time": "2026-10-23T16:00:00Z",
            "scope": "this_and_following",
            "occurrence_start_time": START
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"updated": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "calls",
            "scheduled",
            "update",
            SCHEDULED_ID,
            "--start",
            "2026-10-23T16:00:00Z",
            "--scope",
            "this-and-following",
            "--occurrence",
            START,
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["updated"], true);
}

#[tokio::test]
async fn scheduled_update_that_changes_nothing_is_refused_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "calls",
            "scheduled",
            "update",
            SCHEDULED_ID,
            "--scope",
            "this",
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "validation");
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains("no-call-changes")),
        "{envelope}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn scheduled_cancel_without_force_asks_first_and_cancels_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--no-interactive",
            "--output",
            "json",
            "calls",
            "scheduled",
            "cancel",
            SCHEDULED_ID,
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "confirmation-required");
    assert!(
        envelope["next_step"]["template"]
            .as_str()
            .is_some_and(|t| t.contains("--force")),
        "{envelope}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn scheduled_cancel_with_force_names_the_occurrence_in_the_query() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/calls/scheduled/{SCHEDULED_ID}")))
        .and(query_param("scope", "this"))
        .and(query_param("occurrence_start_time", START))
        .and(query_param("notify_invitees", "false"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"cancelled": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "calls",
            "scheduled",
            "cancel",
            SCHEDULED_ID,
            "--scope",
            "this",
            "--occurrence",
            START,
            "--no-notify",
            "--force",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["cancelled"], true);
}

#[tokio::test]
async fn scheduled_rsvp_posts_the_answer() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/2/calls/scheduled/{SCHEDULED_ID}/rsvp")))
        .and(body_json(serde_json::json!({"response": "declined"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"response": "declined"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "calls",
            "scheduled",
            "rsvp",
            SCHEDULED_ID,
            "declined",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["response"], "declined");
}

/// The lookup that answers `jack`: one account, id 12.
async fn answer_jack(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "jack"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "12", "name": "Jack", "username": "jack"}]
        })))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn scheduled_add_looks_each_handle_invitee_up_and_keeps_the_rest_of_the_call() {
    let server = MockServer::start().await;
    answer_jack(&server).await;
    Mock::given(method("POST"))
        .and(path("/2/calls/scheduled"))
        .and(body_json(serde_json::json!({
            "start_time": START,
            "invitee_ids": ["12", "2244994945"],
            "notify_invitees": false
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({"data": scheduled_call()})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "calls",
            "scheduled",
            "add",
            "--start",
            START,
            "--invite",
            "@jack",
            "--invite",
            "2244994945",
            "--no-notify",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn scheduled_update_replaces_the_invite_list_with_the_handles_ids() {
    let server = MockServer::start().await;
    answer_jack(&server).await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/calls/scheduled/{SCHEDULED_ID}")))
        .and(body_json(serde_json::json!({"invitee_ids": ["12"]})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"updated": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "calls",
            "scheduled",
            "update",
            SCHEDULED_ID,
            "--invite",
            "jack",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn a_handle_invitee_x_does_not_know_fails_before_the_call_is_scheduled() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "12", "name": "Jack", "username": "jack"}],
            "errors": [{"value": "nobody_here", "detail": "Could not find user with usernames: [nobody_here].", "title": "Not Found Error", "resource_type": "user", "parameter": "usernames", "resource_id": "nobody_here", "type": "https://api.x.com/2/problems/resource-not-found"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "calls",
            "scheduled",
            "add",
            "--start",
            START,
            "--invite",
            "@jack",
            "--invite",
            "@nobody_here",
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(stderr.contains("nobody_here"), "{stderr}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 1);
}

#[tokio::test]
async fn a_handle_invitee_under_dry_run_is_named_as_given_and_nothing_is_sent() {
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
            "calls",
            "scheduled",
            "add",
            "--start",
            START,
            "--invite",
            "@jack",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let answer = json_out(&stdout);
    assert_eq!(answer["would_succeed"], true, "{answer}");
    assert_eq!(
        answer["invitee_ids"],
        serde_json::json!(["@jack"]),
        "{answer}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn invitees_outside_the_spec_are_refused_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let mut too_many: Vec<String> = (1..=13).map(|n| format!("@user{n}")).collect();
    too_many.extend((1..=13).map(|n| n.to_string()));

    for (invitees, reason) in [
        (vec!["bad-handle!".to_string()], "invalid-username"),
        (vec!["12345678901234567890".to_string()], "invalid-user-id"),
        (too_many, "too-many-invitees"),
    ] {
        let mut args = vec![
            "--output",
            "json",
            "calls",
            "scheduled",
            "add",
            "--start",
            START,
        ];
        for invitee in &invitees {
            args.extend(["--invite", invitee.as_str()]);
        }
        let (code, _stdout, stderr) = run(&store, &server.uri(), &args).await;

        assert_eq!(code, 1, "{reason}: {stderr}");
        assert_eq!(error_envelope(&stderr)["message"], reason, "{stderr}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
