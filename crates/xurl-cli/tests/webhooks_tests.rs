//! `xr webhooks` reaches the webhook management and Account Activity
//! subscription endpoints through the runner: each verb sends the request the
//! spec gives its endpoint and prints the typed response, a `remove` asks
//! before it deletes, an argument the spec refuses is refused offline, and an
//! endpoint that takes one kind of credential is refused the other.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::listen::{
    CRC_UNDER_CLIENT_SECRET, CRC_UNDER_CONSUMER_SECRET, EVENT, EVENT_UNDER_CLIENT_SECRET,
    EVENT_UNDER_CONSUMER_SECRET, LEGACY_HEADER, OAUTH2_HEADER, crc_token_from, deliver, free_port,
    listen_tapped, listen_while, next_record,
};
use common::{oauth1_store, oauth2_store, run_in_process as run};

const WEBHOOK_ID: &str = "1146654567674912769";

fn webhook() -> serde_json::Value {
    serde_json::json!({
        "id": WEBHOOK_ID,
        "url": "https://example.com/webhooks/x",
        "valid": true,
        "created_at": "2026-01-15T12:00:00.000Z"
    })
}

/// The `OAuth1` store with a bearer token on the same app, for the endpoint
/// that takes a bearer only.
fn bearer_store(dir: &Path) -> PathBuf {
    let store = oauth1_store(dir);
    let mut ts = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.save_bearer_token_for_app("myapp", "BEARER-TOKEN")
        .expect("save bearer");
    store
}

fn error_envelope(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr")
}

#[tokio::test]
async fn list_prints_the_registered_webhooks_as_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/webhooks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [webhook()],
            "meta": {"result_count": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["webhooks", "list", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"][0]["id"], WEBHOOK_ID);
    assert_eq!(body["data"][0]["valid"], true);
}

#[tokio::test]
async fn add_posts_the_url_and_prints_the_webhook() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/webhooks"))
        .and(body_json(
            serde_json::json!({"url": "https://example.com/webhooks/x"}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": webhook()})),
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
            "webhooks",
            "add",
            "https://example.com/webhooks/x",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["id"], WEBHOOK_ID);
}

#[tokio::test]
async fn add_refuses_an_empty_url_without_a_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "webhooks", "add", ""],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "validation");
    assert_eq!(envelope["rule"], "empty-webhook-url", "{envelope}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn validate_puts_to_the_webhook() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/webhooks/{WEBHOOK_ID}")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"valid": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["webhooks", "validate", WEBHOOK_ID, "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["valid"], true);
}

#[tokio::test]
async fn remove_without_force_asks_first_and_deletes_nothing() {
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
            "webhooks",
            "remove",
            WEBHOOK_ID,
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "confirmation-required");
    assert_eq!(envelope["next_step"]["action"], "confirm");
    assert!(
        envelope["next_step"]["template"]
            .as_str()
            .is_some_and(|t| t.contains("--force")),
        "{envelope}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn remove_with_force_deletes_the_webhook() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/webhooks/{WEBHOOK_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
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
            "webhooks", "remove", WEBHOOK_ID, "--force", "--output", "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["deleted"], true);
}

#[tokio::test]
async fn remove_under_dry_run_says_the_real_run_would_ask() {
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
            "webhooks",
            "remove",
            WEBHOOK_ID,
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["status"], "dry_run");
    assert_eq!(body["command"], "webhooks-remove");
    assert_eq!(body["confirmation_required"], true);
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn replay_posts_the_webhook_and_the_window_under_a_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/webhooks/replay"))
        .and(body_json(serde_json::json!({
            "webhook_id": WEBHOOK_ID,
            "from_date": "202601150000",
            "to_date": "202601151200"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"job_id": "1915510368169844736", "created_at": "2026-01-15T12:00:00.000Z"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "webhooks",
            "replay",
            WEBHOOK_ID,
            "--from",
            "202601150000",
            "--to",
            "202601151200",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["job_id"], "1915510368169844736");
}

#[tokio::test]
async fn replay_refuses_a_time_that_is_not_twelve_digits() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "webhooks",
            "replay",
            WEBHOOK_ID,
            "--from",
            "2026-01-15T00:00",
            "--to",
            "202601151200",
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "validation");
    assert_eq!(envelope["rule"], "invalid-replay-time", "{envelope}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn replay_without_a_bearer_is_an_auth_mismatch() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "webhooks",
            "replay",
            WEBHOOK_ID,
            "--from",
            "202601150000",
            "--to",
            "202601151200",
        ],
    )
    .await;

    assert_eq!(code, 2, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "auth-method-mismatch");
    assert_eq!(envelope["supported"], serde_json::json!(["app"]));
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

// ── Account Activity subscriptions ───────────────────────────────────

const USER_ID: &str = "2244994945";

#[tokio::test]
async fn subscriptions_count_prints_the_apps_counts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/account_activity/subscriptions/count"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "account_name": "example-app",
                "provisioned_count": "15",
                "subscriptions_count_all": "2",
                "subscriptions_count_direct_messages": "0"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["webhooks", "subscriptions", "count", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["subscriptions_count_all"], "2");
}

#[tokio::test]
async fn subscriptions_list_prints_the_subscribed_users() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/2/account_activity/webhooks/{WEBHOOK_ID}/subscriptions/all/list"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "application_id": "32371675",
                "webhook_id": WEBHOOK_ID,
                "webhook_url": "https://example.com/webhooks/x",
                "subscriptions": [{"user_id": USER_ID}]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "webhooks",
            "subscriptions",
            "list",
            WEBHOOK_ID,
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["subscriptions"][0]["user_id"], USER_ID);
}

#[tokio::test]
async fn subscriptions_add_subscribes_the_signed_in_account() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/2/account_activity/webhooks/{WEBHOOK_ID}/subscriptions/all"
        )))
        .and(body_json(serde_json::json!({})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"subscribed": true}})),
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
            "webhooks",
            "subscriptions",
            "add",
            WEBHOOK_ID,
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["subscribed"], true);
}

#[tokio::test]
async fn subscriptions_check_asks_whether_the_signed_in_account_is_subscribed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/2/account_activity/webhooks/{WEBHOOK_ID}/subscriptions/all"
        )))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"subscribed": false}})),
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
            "webhooks",
            "subscriptions",
            "check",
            WEBHOOK_ID,
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["subscribed"], false);
}

#[tokio::test]
async fn subscriptions_add_with_only_a_bearer_is_an_auth_mismatch() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_bearer_token_for_app("myapp", "BEARER-TOKEN")
        .expect("save bearer");
    ts.set_default_app("myapp").expect("set_default_app");

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "webhooks",
            "subscriptions",
            "add",
            WEBHOOK_ID,
        ],
    )
    .await;

    assert_ne!(code, 0, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "auth-method-mismatch");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

/// Asking comes first, so a handle is not looked up for a removal the
/// caller has not confirmed.
#[tokio::test]
async fn subscriptions_remove_without_force_asks_first_and_looks_nothing_up() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--no-interactive",
            "--output",
            "json",
            "webhooks",
            "subscriptions",
            "remove",
            WEBHOOK_ID,
            "@jack",
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "confirmation-required");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn subscriptions_remove_looks_a_handle_up_and_unsubscribes_its_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "jack"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "12", "name": "Jack", "username": "jack"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!(
            "/2/account_activity/webhooks/{WEBHOOK_ID}/subscriptions/12/all"
        )))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"subscribed": false}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "webhooks",
            "subscriptions",
            "remove",
            WEBHOOK_ID,
            "@jack",
            "--force",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn subscriptions_remove_of_a_handle_x_does_not_know_deletes_nothing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errors": [{"value": "nobody_here", "detail": "Could not find user with usernames: [nobody_here].", "title": "Not Found Error", "resource_type": "user", "parameter": "usernames", "resource_id": "nobody_here", "type": "https://api.x.com/2/problems/resource-not-found"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "webhooks",
            "subscriptions",
            "remove",
            WEBHOOK_ID,
            "@nobody_here",
            "--force",
        ],
    )
    .await;

    assert_eq!(code, 4, "stderr: {stderr}");
    assert_eq!(error_envelope(&stderr)["reason"], "not-found", "{stderr}");
    assert!(stderr.contains("nobody_here"), "{stderr}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 1);
}

#[tokio::test]
async fn subscriptions_remove_of_a_handle_under_dry_run_sends_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "webhooks",
            "subscriptions",
            "remove",
            WEBHOOK_ID,
            "@jack",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let answer: serde_json::Value = serde_json::from_str(stdout.trim()).expect("an envelope");
    assert_eq!(answer["would_succeed"], true, "{answer}");
    assert_eq!(answer["confirmation_required"], true, "{answer}");
    assert_eq!(answer["user_id"], "@jack", "{answer}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn subscriptions_remove_with_force_unsubscribes_the_user() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!(
            "/2/account_activity/webhooks/{WEBHOOK_ID}/subscriptions/{USER_ID}/all"
        )))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"subscribed": false}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "webhooks",
            "subscriptions",
            "remove",
            WEBHOOK_ID,
            USER_ID,
            "--force",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["subscribed"], false);
}

/// An account is a handle or a numeric user id; one in neither form is
/// refused before any request, a lookup included.
#[tokio::test]
async fn subscriptions_remove_refuses_an_account_in_neither_form() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    for (account, reason) in [
        ("jack.smith", "invalid-username"),
        ("12345678901234567890", "invalid-user-id"),
    ] {
        let (code, _stdout, stderr) = run(
            &store,
            &server.uri(),
            &[
                "--output",
                "json",
                "webhooks",
                "subscriptions",
                "remove",
                WEBHOOK_ID,
                account,
                "--force",
            ],
        )
        .await;

        assert_eq!(code, 1, "{account}: {stderr}");
        let envelope = error_envelope(&stderr);
        assert_eq!(envelope["reason"], "validation", "{envelope}");
        assert_eq!(envelope["rule"], reason, "{envelope}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

// ── Filtered-stream links ────────────────────────────────────────────

#[tokio::test]
async fn stream_links_list_prints_the_webhooks_the_filtered_stream_delivers_to() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/webhooks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "application_id": "32371675",
                "business_user_id": USER_ID,
                "instance_id": "1",
                "webhook_id": WEBHOOK_ID,
                "fields": ["created_at"],
                "created_at": "2026-01-15T12:00:00.000Z"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["webhooks", "stream-links", "list", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"][0]["webhook_id"], WEBHOOK_ID);
}

#[tokio::test]
async fn stream_links_add_links_the_filtered_stream_to_the_webhook() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/2/tweets/search/webhooks/{WEBHOOK_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"provisioned": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "webhooks",
            "stream-links",
            "add",
            WEBHOOK_ID,
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["provisioned"], true);
}

#[tokio::test]
async fn stream_links_remove_without_force_asks_first_and_deletes_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--no-interactive",
            "--output",
            "json",
            "webhooks",
            "stream-links",
            "remove",
            WEBHOOK_ID,
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "confirmation-required");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn stream_links_remove_with_force_unlinks_the_webhook() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/tweets/search/webhooks/{WEBHOOK_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "webhooks",
            "stream-links",
            "remove",
            WEBHOOK_ID,
            "--force",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["deleted"], true);
}

#[tokio::test]
async fn stream_links_need_a_bearer() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "webhooks", "stream-links", "list"],
    )
    .await;

    assert_eq!(code, 2, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "auth-method-mismatch");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

// ── listen ───────────────────────────────────────────────────────────
//
// `oauth1_store` gives the app the client secret `SECRET-VALUE` and the
// consumer secret `CONSUMER-SECRET`. Each expected signature below was
// computed outside this crate (Python's `hmac.new(key, message, sha256)`,
// base64-encoded) for those keys.

#[tokio::test]
async fn listen_answers_the_crc_check_and_prints_a_signed_event() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();

    let ((code, stdout, stderr), ()) = listen_while(&store, port, &["--max-events", "1"], async {
        assert_eq!(crc_token_from(port).await, CRC_UNDER_CLIENT_SECRET);
        let status = deliver(port, Some((OAUTH2_HEADER, EVENT_UNDER_CLIENT_SECRET))).await;
        assert_eq!(status, 200);
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let lines: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("each line is JSON"))
        .collect();
    assert_eq!(lines.len(), 2, "{stdout}");
    assert_eq!(lines[0]["status"], "listening");
    assert_eq!(lines[0]["url"], format!("http://127.0.0.1:{port}/webhook"));
    assert_eq!(lines[0]["secret"], "oauth2");
    assert_eq!(
        stdout.lines().nth(1),
        Some(EVENT),
        "a one-line JSON body prints byte for byte as X sent it"
    );
}

#[tokio::test]
async fn listen_prints_every_body_on_one_line_that_parses() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();
    let flags = ["--allow-unsigned", "--max-events", "2"];

    let ((code, stdout, stderr), ()) = listen_while(&store, port, &flags, async {
        crc_token_from(port).await;
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/webhook");
        for body in ["{\n  \"b\": 1,\n  \"a\": 2\n}", "not json"] {
            let delivered = client
                .post(&url)
                .body(body)
                .send()
                .await
                .expect("the listener answers");
            assert_eq!(delivered.status(), 200);
        }
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "{stdout}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(lines[1]).expect("JSON"),
        serde_json::json!({"a": 2, "b": 1}),
        "a body that spans lines is printed on one"
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(lines[2]).expect("JSON"),
        serde_json::json!("not json"),
        "a body that is not JSON is carried as a JSON string"
    );
}

#[tokio::test]
async fn listen_refuses_an_event_whose_signature_is_wrong() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();

    let ((code, stdout, stderr), ()) = listen_while(&store, port, &["--max-events", "1"], async {
        crc_token_from(port).await;
        assert_eq!(
            deliver(port, Some((OAUTH2_HEADER, "sha256=AAAA"))).await,
            401
        );
        assert_eq!(deliver(port, None).await, 401);
        let status = deliver(port, Some((OAUTH2_HEADER, EVENT_UNDER_CLIENT_SECRET))).await;
        assert_eq!(status, 200);
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(
        stdout.lines().count(),
        2,
        "only the genuine event prints: {stdout}"
    );
}

#[tokio::test]
async fn listen_with_secret_oauth1_signs_with_the_consumer_secret() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();
    let flags = ["--secret", "oauth1", "--max-events", "1"];

    let ((code, stdout, stderr), ()) = listen_while(&store, port, &flags, async {
        assert_eq!(crc_token_from(port).await, CRC_UNDER_CONSUMER_SECRET);
        let status = deliver(port, Some((LEGACY_HEADER, EVENT_UNDER_CONSUMER_SECRET))).await;
        assert_eq!(status, 200);
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let first: serde_json::Value =
        serde_json::from_str(stdout.lines().next().expect("a first line")).expect("JSON");
    assert_eq!(first["secret"], "oauth1");
}

#[tokio::test]
async fn listen_with_allow_unsigned_prints_an_unsigned_event() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();
    let flags = ["--allow-unsigned", "--max-events", "1"];

    let ((code, stdout, stderr), ()) = listen_while(&store, port, &flags, async {
        crc_token_from(port).await;
        assert_eq!(deliver(port, None).await, 200);
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.lines().count(), 2, "{stdout}");
}

#[tokio::test]
async fn listen_under_dry_run_binds_nothing_and_names_the_secret() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();

    let (code, stdout, stderr) = run(
        &store,
        "http://127.0.0.1:9",
        &[
            "--dry-run",
            "--output",
            "json",
            "webhooks",
            "listen",
            "--port",
            &port.to_string(),
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["status"], "dry_run");
    assert_eq!(body["command"], "webhooks-listen");
    assert_eq!(body["would_succeed"], true);
    assert_eq!(body["secret"], "oauth2");
    assert!(
        std::net::TcpStream::connect(("127.0.0.1", port)).is_err(),
        "a dry run must not bind the port"
    );
}

#[tokio::test]
async fn listen_hands_out_a_signed_test_request_that_verifies() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();

    let (code, stderr, (listening, event)) = listen_tapped(
        &store,
        port,
        &["--max-events", "1"],
        |mut lines| async move {
            let listening = next_record(&mut lines).await;
            let request = &listening["test_request"];
            assert_eq!(request["method"], "POST", "{listening}");
            let mut post = reqwest::Client::new()
                .post(request["url"].as_str().expect("a URL"))
                .body(request["body"].as_str().expect("a body").to_string());
            for (name, value) in request["headers"].as_object().expect("headers") {
                post = post.header(name, value.as_str().expect("a header value"));
            }
            let status = post.send().await.expect("the listener answers").status();
            assert_eq!(status, 200, "the sample verifies");
            (listening, next_record(&mut lines).await)
        },
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(listening["status"], "listening");
    assert_eq!(event["xr_test_event"], true, "the sample is the event");
    for record in [&listening["test_command"], &listening["test_request"]] {
        let shown = record.to_string();
        assert!(!shown.contains("SECRET-VALUE"), "{shown}");
        assert!(!shown.contains("CONSUMER-SECRET"), "{shown}");
    }
}

#[cfg(unix)]
#[tokio::test]
async fn listen_hands_out_a_test_command_a_posix_shell_runs() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();

    let (code, stderr, (shell, event)) = listen_tapped(
        &store,
        port,
        &["--max-events", "1"],
        |mut lines| async move {
            let listening = next_record(&mut lines).await;
            let command = listening["test_command"]
                .as_str()
                .expect("a test command")
                .to_string();
            let shell = tokio::task::spawn_blocking(move || {
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .output()
                    .expect("sh runs")
            })
            .await
            .expect("the shell finishes");
            (shell, next_record(&mut lines).await)
        },
    )
    .await;

    assert!(
        shell.status.success(),
        "{:?}: {}",
        shell.status,
        String::from_utf8_lossy(&shell.stderr)
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(event["xr_test_event"], true);
}

#[tokio::test]
async fn listen_in_text_prints_the_test_command_on_stderr() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let held = std::net::TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port = held.local_addr().expect("its address").port();
    drop(held);

    let port_arg = port.to_string();
    let args = [
        "webhooks",
        "listen",
        "--port",
        &port_arg,
        "--max-events",
        "1",
    ];
    let ((code, stdout, stderr), ()) =
        tokio::join!(run(&store, "http://127.0.0.1:9", &args), async {
            crc_token_from(port).await;
            deliver(port, Some((OAUTH2_HEADER, EVENT_UNDER_CLIENT_SECRET))).await;
        });

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(
        stdout.lines().count(),
        1,
        "stdout holds the event only: {stdout}"
    );
    assert!(stderr.contains("curl "), "{stderr}");
    assert!(stderr.contains("xr_test_event"), "{stderr}");
    assert!(!stderr.contains("SECRET-VALUE"), "{stderr}");
}

#[tokio::test]
async fn listen_on_a_held_port_names_it_and_reruns_on_a_free_port() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let held = std::net::TcpListener::bind("127.0.0.1:0").expect("a port to hold");
    let port = held.local_addr().expect("its address").port().to_string();

    let (code, stdout, stderr) = run(
        &store,
        "http://127.0.0.1:9",
        &[
            "--output",
            "json",
            "webhooks",
            "listen",
            "--port",
            &port,
            "--max-events",
            "1",
        ],
    )
    .await;

    assert_eq!(code, 5, "stderr: {stderr}");
    assert_eq!(stdout, "");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "address-in-use", "{envelope}");
    assert_eq!(envelope["exit_code"], 5);
    assert_eq!(
        envelope["next_step"],
        serde_json::json!({
            "action": "fix-input",
            "command": "xr --output json webhooks listen --max-events 1 --port 0"
        })
    );
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains(&format!("127.0.0.1:{port}"))),
        "the message names the address: {envelope}"
    );
}

#[tokio::test]
async fn listen_on_a_held_port_in_text_suggests_a_free_port_or_stopping_the_holder() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let held = std::net::TcpListener::bind("127.0.0.1:0").expect("a port to hold");
    let port = held.local_addr().expect("its address").port().to_string();

    let (code, _stdout, stderr) = run(
        &store,
        "http://127.0.0.1:9",
        &["webhooks", "listen", "--port", &port],
    )
    .await;

    assert_eq!(code, 5, "stderr: {stderr}");
    assert!(stderr.contains(&format!("127.0.0.1:{port}")), "{stderr}");
    assert!(stderr.contains("Stop"), "{stderr}");
    assert!(
        stderr.contains("xr webhooks listen --port 0"),
        "the rerun on a free port: {stderr}"
    );
}

// ── the URL rules X documents ────────────────────────────────────────

#[rstest::rstest]
#[case::plain_http("http://example.com/webhooks/x", "webhook-url-not-https")]
#[case::not_a_url("example.com/webhooks/x", "webhook-url-not-https")]
#[case::a_port("https://example.com:8443/webhooks/x", "webhook-url-has-port")]
#[tokio::test]
async fn add_refuses_a_url_x_would_refuse(#[case] url: &str, #[case] reason: &str) {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "webhooks", "add", url],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "validation");
    assert_eq!(envelope["rule"], reason, "{envelope}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

// ── replay from a checkpoint ─────────────────────────────────────────

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("after the epoch")
        .as_secs()
}

/// A checkpoint file recording `secs`, as `listen --checkpoint` writes one.
fn checkpoint_at(dir: &Path, secs: u64) -> PathBuf {
    let path = dir.join("checkpoint.json");
    std::fs::write(&path, format!(r#"{{"last_event_at": {secs}}}"#)).expect("write");
    path
}

async fn replay_server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/webhooks/replay"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"job_id": "1915510368169844736", "created_at": "2026-01-15T12:00:00.000Z"}
        })))
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn replay_from_a_checkpoint_posts_its_minute_to_the_current_one() {
    let server = replay_server().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());
    let checkpoint_secs = now_secs() - 600;
    let checkpoint = checkpoint_at(tmp.path(), checkpoint_secs);

    let before = now_secs();
    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "webhooks",
            "replay",
            WEBHOOK_ID,
            "--from-checkpoint",
            checkpoint.to_str().expect("utf-8 path"),
            "--output",
            "json",
        ],
    )
    .await;
    let after = now_secs();

    assert_eq!(code, 0, "stderr: {stderr}");
    let seen = server.received_requests().await.expect("recorded");
    assert_eq!(seen.len(), 1);
    assert!(
        seen[0].headers["authorization"]
            .to_str()
            .is_ok_and(|value| value.starts_with("Bearer ")),
        "the replay goes out under the bearer"
    );
    let body: serde_json::Value = serde_json::from_slice(&seen[0].body).expect("JSON");
    assert_eq!(body["webhook_id"], WEBHOOK_ID);
    assert_eq!(body["from_date"], xdk::time::utc_minute(checkpoint_secs));
    let to = body["to_date"].as_str().expect("a to_date");
    assert!(
        to == xdk::time::utc_minute(before) || to == xdk::time::utc_minute(after),
        "to is the current minute: {to}"
    );
    let printed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON");
    assert_eq!(printed["data"]["job_id"], "1915510368169844736");
    assert_eq!(printed["clamped"], false);
    assert!(printed.get("lost_from").is_none(), "{printed}");
}

#[tokio::test]
async fn replay_from_a_checkpoint_under_dry_run_shows_the_window_and_sends_nothing() {
    let server = replay_server().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());
    let checkpoint_secs = now_secs() - 600;
    let checkpoint = checkpoint_at(tmp.path(), checkpoint_secs);

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "webhooks",
            "replay",
            WEBHOOK_ID,
            "--from-checkpoint",
            checkpoint.to_str().expect("utf-8 path"),
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON");
    assert_eq!(body["status"], "dry_run");
    assert_eq!(body["would_succeed"], true);
    assert_eq!(body["from"], xdk::time::utc_minute(checkpoint_secs));
    assert!(body["to"].is_string(), "{body}");
    assert_eq!(body["clamped"], false);
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_checkpoint_older_than_a_day_is_clamped_in_the_dry_run() {
    let server = replay_server().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());
    let checkpoint_secs = now_secs() - 30 * 3600;
    let checkpoint = checkpoint_at(tmp.path(), checkpoint_secs);

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "webhooks",
            "replay",
            WEBHOOK_ID,
            "--from-checkpoint",
            checkpoint.to_str().expect("utf-8 path"),
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON");
    assert_eq!(body["clamped"], true, "{body}");
    assert_eq!(body["lost_from"], xdk::time::utc_minute(checkpoint_secs));
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[rstest::rstest]
#[case::checkpoint_and_from(&["--from-checkpoint", "cp.json", "--from", "202601150000"])]
#[case::from_alone(&["--from", "202601150000"])]
#[tokio::test]
async fn the_window_is_a_checkpoint_or_both_bounds(#[case] window: &[&str]) {
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());
    let mut args = vec!["--output", "json", "webhooks", "replay", WEBHOOK_ID];
    args.extend_from_slice(window);

    let (code, _stdout, stderr) = run(&store, "http://127.0.0.1:9", &args).await;

    assert_eq!(code, 2, "stderr: {stderr}");
    assert_eq!(error_envelope(&stderr)["reason"], "invalid-args");
}

#[tokio::test]
async fn a_missing_checkpoint_is_refused_as_io_and_names_both_fixes() {
    let server = replay_server().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());
    let missing = tmp.path().join("no-checkpoint.json");

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "webhooks",
            "replay",
            WEBHOOK_ID,
            "--from-checkpoint",
            missing.to_str().expect("utf-8 path"),
        ],
    )
    .await;

    assert_eq!(code, 5, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "io", "{envelope}");
    assert_eq!(envelope["next_step"]["action"], "fix-input");
    let message = envelope["message"].as_str().expect("a message");
    assert!(
        message.contains("--from") && message.contains("--to") && message.contains("--checkpoint"),
        "the message names both fixes: {message}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn replay_from_a_checkpoint_without_a_bearer_is_an_auth_mismatch() {
    let server = replay_server().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let checkpoint = checkpoint_at(tmp.path(), now_secs() - 600);

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "webhooks",
            "replay",
            WEBHOOK_ID,
            "--from-checkpoint",
            checkpoint.to_str().expect("utf-8 path"),
        ],
    )
    .await;

    assert_ne!(code, 0, "stderr: {stderr}");
    assert_eq!(error_envelope(&stderr)["reason"], "auth-method-mismatch");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

// ── help states what the validators enforce ──────────────────────────

/// The help of `command` as `xr` prints it.
async fn help_of(command: &[&str]) -> String {
    let tmp = TempDir::new().expect("tempdir");
    let mut args = command.to_vec();
    args.push("--help");
    let (code, stdout, stderr) = run(&tmp.path().join(".xurl"), "http://127.0.0.1:9", &args).await;
    assert_eq!(code, 0, "{stderr}");
    stdout
}

/// `<URL>` names the two rules a URL can break offline beyond being https:
/// it names no port, and it fits the validator's length limit.
#[tokio::test]
async fn the_url_help_states_the_rules_its_validator_enforces() {
    let help = help_of(&["webhooks", "add"]).await;
    let arguments = help
        .split("Arguments:")
        .nth(1)
        .and_then(|rest| rest.split("Options:").next())
        .expect("an Arguments section");
    assert!(arguments.contains("no port"), "{arguments}");
    let limit = format!("{} characters", xdk::api::shortcuts::WEBHOOK_URL_MAX_CHARS);
    assert!(arguments.contains(&limit), "{arguments}");
}

/// Both checkpoint flags name the file's shape, and a file written by hand in
/// that shape drives a replay.
#[tokio::test]
async fn the_checkpoint_help_names_a_shape_the_replay_takes() {
    for command in [&["webhooks", "replay"][..], &["webhooks", "listen"][..]] {
        let help = help_of(command).await;
        assert!(
            help.contains(xdk::webhooks::CHECKPOINT_SHAPE),
            "xr {} --help does not name the checkpoint shape: {help}",
            command.join(" ")
        );
    }

    let server = replay_server().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());
    let checkpoint = tmp.path().join("checkpoint.json");
    let filled =
        xdk::webhooks::CHECKPOINT_SHAPE.replace("<Unix seconds>", &(now_secs() - 600).to_string());
    std::fs::write(&checkpoint, filled).expect("write");
    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "webhooks",
            "replay",
            WEBHOOK_ID,
            "--from-checkpoint",
            checkpoint.to_str().expect("utf-8 path"),
            "--output",
            "json",
        ],
    )
    .await;
    assert_eq!(code, 0, "{stdout} {stderr}");
}

/// The real run and the dry run of `listen` with `extra` arguments, under
/// `store`: each one's exit code and JSON envelope.
async fn listen_both_ways(
    store: &Path,
    extra: &[&str],
) -> ((i32, serde_json::Value), (i32, serde_json::Value)) {
    let mut argv = vec!["--output", "json", "webhooks", "listen", "--port", "0"];
    argv.extend(extra);
    let (code, _stdout, stderr) = run(store, "http://127.0.0.1:9", &argv).await;
    let real = (code, error_envelope(&stderr));
    argv.insert(0, "--dry-run");
    let (code, stdout, stderr) = run(store, "http://127.0.0.1:9", &argv).await;
    assert_eq!(code, 0, "dry run: {stderr}");
    let dry = (
        code,
        serde_json::from_str(stdout.trim()).expect("JSON on stdout"),
    );
    (real, dry)
}

/// `listen` with no secret to verify X's signatures by names the secret to
/// store, and its dry run answers with the same reason, message, and step.
#[tokio::test]
async fn listen_without_a_signing_secret_names_the_secret_to_store() {
    let tmp = TempDir::new().expect("tempdir");
    let no_secret = tmp.path().join("no-secret.yml");
    let mut ts = xdk::store::TokenStore::new_with_path(no_secret.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "").expect("add_app");
    let oauth2_only = oauth2_store(tmp.path());
    let empty = tmp.path().join("empty.yml");

    for (store, extra, action, template) in [
        (
            no_secret.as_path(),
            &[][..],
            "fix-input",
            Some("xr auth apps update myapp --client-secret-file <path>"),
        ),
        (
            oauth2_only.as_path(),
            &["--secret", "oauth1"][..],
            "fix-input",
            Some(
                "xr auth oauth1 --app myapp --consumer-key <key> --consumer-secret-file <path> \
                 --access-token-file <path> --token-secret-file <path>",
            ),
        ),
        (empty.as_path(), &[][..], "register-app", None),
    ] {
        let ((code, real), (_, dry)) = listen_both_ways(store, extra).await;

        assert_eq!(code, 77, "{extra:?}: {real}");
        assert_eq!(real["reason"], "auth-required", "{real}");
        assert_eq!(real["next_step"]["action"], action, "{real}");
        if let Some(template) = template {
            assert_eq!(real["next_step"]["template"], template, "{real}");
        }
        assert!(real["next_step"].get("command").is_none(), "{real}");

        assert_eq!(dry["status"], "dry_run", "{dry}");
        assert_eq!(dry["would_succeed"], false, "{dry}");
        for key in ["reason", "exit_code", "message", "next_step"] {
            assert_eq!(dry[key], real[key], "{extra:?}: the dry run's {key}: {dry}");
        }
    }
}
