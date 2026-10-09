//! `xr webhooks` reaches the webhook management and Account Activity
//! subscription endpoints through the runner: each verb sends the request the
//! spec gives its endpoint and prints the typed response, a `remove` asks
//! before it deletes, an argument the spec refuses is refused offline, and an
//! endpoint that takes one kind of credential is refused the other.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth1_store, run_in_process as run};

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
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains("empty-webhook-url")),
        "{envelope}"
    );
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
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains("invalid-replay-time")),
        "{envelope}"
    );
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

#[tokio::test]
async fn subscriptions_remove_without_force_asks_first_and_deletes_nothing() {
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
            USER_ID,
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "confirmation-required");
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

#[tokio::test]
async fn subscriptions_remove_refuses_a_user_id_that_is_not_digits() {
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
            "subscriptions",
            "remove",
            WEBHOOK_ID,
            "@someone",
            "--force",
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope = error_envelope(&stderr);
    assert_eq!(envelope["reason"], "validation");
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains("invalid-user-id")),
        "{envelope}"
    );
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

const CRC_TOKEN: &str = "challenge_string";
const CRC_UNDER_CLIENT_SECRET: &str = "sha256=xT/YyTRZde5T1HbCK7K2bpcELYOIMkZVbgYjsESw+AM=";
const CRC_UNDER_CONSUMER_SECRET: &str = "sha256=9CnCWcBcOApA1LHdqkjWbF2DBNdBrnAJPmL5qTUUANQ=";
const EVENT: &str = r#"{"for_user_id":"2244994945","follow_events":[]}"#;
const EVENT_UNDER_CLIENT_SECRET: &str = "sha256=jlSMkj72sqpIWJdl8rdKiEf1zPMRJkvY1VWxVhuPD4c=";
const EVENT_UNDER_CONSUMER_SECRET: &str = "sha256=IdtPMVhr8Hql9OtN7YTX/+gknTMNOui0D4ixytI8bD0=";

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("a loopback port")
        .local_addr()
        .expect("its address")
        .port()
}

/// Runs `xr webhooks listen` on `port` with `extra` flags while `drive` talks
/// to it, and returns both results. The run ends once `--max-events` events
/// have printed, so `drive` has to deliver that many. Both futures share
/// this task: the run holds a `&mut dyn Write` and cannot be spawned.
async fn listen_while<T>(
    store: &Path,
    port: u16,
    extra: &[&str],
    drive: impl std::future::Future<Output = T>,
) -> ((i32, String, String), T) {
    let port = port.to_string();
    let mut args = vec!["--output", "json", "webhooks", "listen", "--port", &port];
    args.extend_from_slice(extra);
    tokio::join!(run(store, "http://127.0.0.1:9", &args), drive)
}

/// The CRC response token the listener on `port` gives, once it is up.
async fn crc_token_from(port: u16) -> String {
    let url = format!("http://127.0.0.1:{port}/webhook?crc_token={CRC_TOKEN}");
    for _ in 0..100 {
        if let Ok(response) = reqwest::get(&url).await {
            let body: serde_json::Value = response.json().await.expect("a JSON body");
            return body["response_token"]
                .as_str()
                .expect("a response token")
                .to_string();
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("the listener never came up on port {port}");
}

/// Posts [`EVENT`] to the listener on `port` under `header`, when one is
/// given, and returns the status it answers with.
async fn deliver(port: u16, header: Option<(&str, &str)>) -> u16 {
    let mut request = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{port}/webhook"))
        .body(EVENT);
    if let Some((name, value)) = header {
        request = request.header(name, value);
    }
    request
        .send()
        .await
        .expect("the listener answers")
        .status()
        .as_u16()
}

const OAUTH2_HEADER: &str = "X-Twitter-Webhooks-Signature-OAuth2";
const LEGACY_HEADER: &str = "X-Twitter-Webhooks-Signature";

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
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains(reason)),
        "{envelope}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
