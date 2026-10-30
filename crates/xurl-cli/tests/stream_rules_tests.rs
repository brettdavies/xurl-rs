//! `xr stream rules` manages the filtered stream's rules: list them, add
//! one, remove some, and count them against the caps. The endpoints take
//! the app's bearer token only. `--validate-only` asks X to check a change
//! without saving it, a real request; `--dry-run` sends nothing.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const RULES: &str = "/2/tweets/search/stream/rules";

/// An `OAuth2` login with a bearer token on the same app.
fn bearer_store(dir: &Path) -> PathBuf {
    let store = oauth2_store(dir);
    let mut ts = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.save_bearer_token_for_app("myapp", "BEARER-TOKEN")
        .expect("save bearer");
    store
}

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn error_envelope(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr")
}

async fn run_json(server: &MockServer, args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());
    let mut argv = vec!["--output", "json"];
    argv.extend(args);
    run(&store, &server.uri(), &argv).await
}

fn created() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "data": [{"id": "1585341984679469056", "value": "from:jack", "tag": "j"}],
        "meta": {
            "sent": "2026-10-26T20:00:00.000Z",
            "summary": {"created": 1, "not_created": 0, "valid": 1, "invalid": 0}
        }
    }))
}

#[tokio::test]
async fn list_reads_the_rules_under_the_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(RULES))
        .and(query_param("max_results", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "1585341984679469056", "value": "from:jack", "tag": "j"}],
            "meta": {"result_count": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(&server, &["stream", "rules", "list", "-n", "50"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["value"], "from:jack");
    let requests = server.received_requests().await.expect("recorded");
    let auth = requests[0].headers.get("authorization").expect("auth");
    assert_eq!(auth.to_str().expect("ascii"), "Bearer BEARER-TOKEN");
}

#[tokio::test]
async fn add_sends_the_value_and_tag_and_saves_it() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(RULES))
        .and(query_param_is_missing("dry_run"))
        .and(body_json(serde_json::json!({
            "add": [{"value": "from:jack", "tag": "j"}]
        })))
        .respond_with(created())
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &["stream", "rules", "add", "from:jack", "--tag", "j"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["id"], "1585341984679469056");
}

#[tokio::test]
async fn validate_only_asks_x_to_check_without_saving() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(RULES))
        .and(query_param("dry_run", "true"))
        .and(body_json(
            serde_json::json!({"add": [{"value": "from:jack"}]}),
        ))
        .respond_with(created())
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &["stream", "rules", "add", "from:jack", "--validate-only"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["meta"]["summary"]["valid"], 1);
}

#[tokio::test]
async fn remove_asks_first_and_deletes_the_ids_with_force() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(RULES))
        .and(query_param_is_missing("dry_run"))
        .and(body_json(
            serde_json::json!({"delete": {"ids": ["1", "2"]}}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": {
                "sent": "2026-10-26T20:00:00.000Z",
                "summary": {"deleted": 2, "not_deleted": 0}
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(
        &server,
        &["--no-interactive", "stream", "rules", "remove", "1", "2"],
    )
    .await;
    assert_eq!(code, 1, "stderr: {stderr}");
    assert_eq!(error_envelope(&stderr)["reason"], "confirmation-required");

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "--no-interactive",
            "stream",
            "rules",
            "remove",
            "1",
            "2",
            "--force",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["meta"]["summary"]["deleted"], 2);
}

#[tokio::test]
async fn a_removal_x_only_checks_needs_no_confirmation() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(RULES))
        .and(query_param("dry_run", "true"))
        .and(body_json(serde_json::json!({"delete": {"ids": ["1"]}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": {"summary": {"deleted": 1, "not_deleted": 0}}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(
        &server,
        &[
            "--no-interactive",
            "stream",
            "rules",
            "remove",
            "1",
            "--validate-only",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn count_reads_the_rules_against_the_caps() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{RULES}/counts")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "cap_per_client_app": "1000",
                "cap_per_project": "1000",
                "project_rules_count": "3",
                "client_app_rules_count": {"client_app_id": "29", "rule_count": 3}
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(&server, &["stream", "rules", "count"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(
        json_out(&stdout)["data"]["client_app_rules_count"]["rule_count"],
        3
    );
}

#[tokio::test]
async fn a_user_login_alone_is_an_auth_mismatch() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "stream", "rules", "list"],
    )
    .await;

    assert_eq!(code, 2, "stderr: {stderr}");
    assert_eq!(error_envelope(&stderr)["reason"], "auth-method-mismatch");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn dry_run_sends_nothing_even_with_validate_only() {
    let server = MockServer::start().await;

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "--dry-run",
            "stream",
            "rules",
            "add",
            "from:jack",
            "--validate-only",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["would_succeed"], true);
    assert_eq!(body["validate_only"], true);
    assert_eq!(body["auth"]["scheme"], "app");

    let (code, stdout, stderr) = run_json(
        &server,
        &["--dry-run", "stream", "rules", "remove", "1", "rule-1"],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["rule"], "invalid-rule-id");

    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
