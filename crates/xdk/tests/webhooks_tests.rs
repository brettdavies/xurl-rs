//! The webhook shortcuts send the requests the spec gives their endpoints
//! and decode their responses; the offline validators hold the spec's
//! patterns; a client with no bearer is refused a replay before any request
//! leaves.

use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::shortcuts::{
    WEBHOOK_URL_MAX_CHARS, validate_replay_time, validate_user_id, validate_webhook_id,
    validate_webhook_url,
};
use xdk::auth::OAuth2Credential;

const WEBHOOK_ID: &str = "1146654567674912769";

fn app_client(base_url: String) -> Client {
    Client::builder()
        .base_url(base_url)
        .bearer("app-token")
        .build()
        .expect("client builds")
}

fn user_client(base_url: String) -> Client {
    Client::builder()
        .base_url(base_url)
        .oauth2(OAuth2Credential {
            client_id: "client-id".into(),
            client_secret: "client-secret".into(),
            access_token: "user-at".into(),
            refresh_token: None,
            expires_at: None,
        })
        .build()
        .expect("client builds")
}

fn webhook() -> serde_json::Value {
    json!({
        "id": WEBHOOK_ID,
        "url": "https://example.com/webhooks/x",
        "valid": true,
        "created_at": "2026-01-15T12:00:00.000Z"
    })
}

#[tokio::test]
async fn get_webhooks_lists_what_the_app_registered() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/webhooks"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data": [webhook()], "meta": {"result_count": 1}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = app_client(server.uri())
        .get_webhooks()
        .send()
        .await
        .expect("webhooks are listed");

    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].id.as_deref(), Some(WEBHOOK_ID));
    assert_eq!(result.data[0].valid, Some(true));
}

#[tokio::test]
async fn create_webhook_posts_the_url() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/webhooks"))
        .and(body_json(json!({"url": "https://example.com/webhooks/x"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": webhook()})))
        .expect(1)
        .mount(&server)
        .await;

    let result = app_client(server.uri())
        .create_webhook("https://example.com/webhooks/x")
        .send()
        .await
        .expect("the webhook is registered");

    assert_eq!(result.data.id.as_deref(), Some(WEBHOOK_ID));
}

#[tokio::test]
async fn validate_webhook_puts_to_the_webhook() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/webhooks/{WEBHOOK_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"valid": true}})))
        .expect(1)
        .mount(&server)
        .await;

    let result = user_client(server.uri())
        .validate_webhook(WEBHOOK_ID)
        .send()
        .await
        .expect("the check is sent again");

    assert!(result.data.valid);
}

#[tokio::test]
async fn delete_webhook_deletes_the_webhook() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/webhooks/{WEBHOOK_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"deleted": true}})))
        .expect(1)
        .mount(&server)
        .await;

    let result = app_client(server.uri())
        .delete_webhook(WEBHOOK_ID)
        .send()
        .await
        .expect("the webhook is deleted");

    assert!(result.data.deleted);
}

#[tokio::test]
async fn create_webhook_replay_posts_the_webhook_and_the_window() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/webhooks/replay"))
        .and(body_json(json!({
            "webhook_id": WEBHOOK_ID,
            "from_date": "202601150000",
            "to_date": "202601151200"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"job_id": "1915510368169844736", "created_at": "2026-01-15T12:00:00.000Z"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = app_client(server.uri())
        .create_webhook_replay(WEBHOOK_ID, "202601150000", "202601151200")
        .send()
        .await
        .expect("the replay job is created");

    assert_eq!(result.data.job_id, "1915510368169844736");
}

#[tokio::test]
async fn a_client_with_no_bearer_is_refused_a_replay_before_any_request() {
    let server = MockServer::start().await;

    let err = user_client(server.uri())
        .create_webhook_replay(WEBHOOK_ID, "202601150000", "202601151200")
        .send()
        .await
        .expect_err("the endpoint takes a bearer only");

    assert_eq!(err.kind(), "auth-method-mismatch", "{err}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[test]
fn validate_webhook_id_holds_the_specs_pattern() {
    assert_eq!(validate_webhook_id(WEBHOOK_ID), Ok(()));
    assert_eq!(validate_webhook_id("7"), Ok(()));
    assert_eq!(validate_webhook_id(""), Err("invalid-webhook-id"));
    assert_eq!(validate_webhook_id("12a"), Err("invalid-webhook-id"));
    assert_eq!(
        validate_webhook_id(&"9".repeat(20)),
        Err("invalid-webhook-id")
    );
}

#[test]
fn validate_webhook_url_holds_the_specs_length_bounds() {
    assert_eq!(validate_webhook_url("https://example.com/hook"), Ok(()));
    assert_eq!(validate_webhook_url(""), Err("empty-webhook-url"));
    let longest = format!(
        "https://example.com/{}",
        "a".repeat(WEBHOOK_URL_MAX_CHARS - 20)
    );
    assert_eq!(longest.chars().count(), WEBHOOK_URL_MAX_CHARS);
    assert_eq!(validate_webhook_url(&longest), Ok(()));
    assert_eq!(
        validate_webhook_url(&format!("{longest}a")),
        Err("webhook-url-too-long")
    );
}

#[test]
fn validate_replay_time_takes_twelve_digits() {
    assert_eq!(validate_replay_time("202601151200"), Ok(()));
    assert_eq!(validate_replay_time(""), Err("invalid-replay-time"));
    assert_eq!(
        validate_replay_time("2026-01-15T12:00"),
        Err("invalid-replay-time")
    );
    assert_eq!(
        validate_replay_time("20260115120"),
        Err("invalid-replay-time")
    );
    assert_eq!(
        validate_replay_time("2026011512000"),
        Err("invalid-replay-time")
    );
}

// ── Account Activity subscriptions ───────────────────────────────────

const USER_ID: &str = "2244994945";

#[tokio::test]
async fn create_account_activity_subscription_posts_an_empty_object_as_the_user() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/2/account_activity/webhooks/{WEBHOOK_ID}/subscriptions/all"
        )))
        .and(body_json(json!({})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": {"subscribed": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = user_client(server.uri())
        .create_account_activity_subscription(WEBHOOK_ID)
        .send()
        .await
        .expect("the account is subscribed");

    assert!(result.data.subscribed);
}

#[tokio::test]
async fn delete_account_activity_subscription_names_the_webhook_and_the_user() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!(
            "/2/account_activity/webhooks/{WEBHOOK_ID}/subscriptions/{USER_ID}/all"
        )))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": {"subscribed": false}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = app_client(server.uri())
        .delete_account_activity_subscription(WEBHOOK_ID, USER_ID)
        .send()
        .await
        .expect("the subscription ends");

    assert!(!result.data.subscribed);
}

#[tokio::test]
async fn get_account_activity_subscriptions_lists_the_subscribed_accounts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/2/account_activity/webhooks/{WEBHOOK_ID}/subscriptions/all/list"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
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

    let result = app_client(server.uri())
        .get_account_activity_subscriptions(WEBHOOK_ID)
        .send()
        .await
        .expect("subscriptions are listed");

    let subscriptions = result.data.subscriptions.expect("subscriptions");
    assert_eq!(subscriptions[0].user_id.as_deref(), Some(USER_ID));
}

#[tokio::test]
async fn a_bearer_only_client_is_refused_a_subscription_before_any_request() {
    let server = MockServer::start().await;

    let err = app_client(server.uri())
        .create_account_activity_subscription(WEBHOOK_ID)
        .send()
        .await
        .expect_err("subscribing takes a user login");

    assert_eq!(err.kind(), "auth-method-mismatch", "{err}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[test]
fn validate_user_id_holds_the_specs_pattern() {
    assert_eq!(validate_user_id(USER_ID), Ok(()));
    assert_eq!(validate_user_id(""), Err("invalid-user-id"));
    assert_eq!(validate_user_id("@someone"), Err("invalid-user-id"));
    assert_eq!(validate_user_id(&"9".repeat(20)), Err("invalid-user-id"));
}

// ── Filtered-stream links ────────────────────────────────────────────

#[tokio::test]
async fn create_webhook_stream_link_posts_to_the_webhook() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/2/tweets/search/webhooks/{WEBHOOK_ID}")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": {"provisioned": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = app_client(server.uri())
        .create_webhook_stream_link(WEBHOOK_ID)
        .send()
        .await
        .expect("the stream is linked");

    assert!(result.data.provisioned);
}

#[tokio::test]
async fn get_webhook_stream_links_lists_where_the_stream_delivers() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/webhooks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"webhook_id": WEBHOOK_ID, "instance_id": "1", "fields": ["created_at"]}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = app_client(server.uri())
        .get_webhook_stream_links()
        .send()
        .await
        .expect("links are listed");

    assert_eq!(result.data[0].webhook_id.as_deref(), Some(WEBHOOK_ID));
}

#[tokio::test]
async fn delete_webhook_stream_link_deletes_the_link() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/tweets/search/webhooks/{WEBHOOK_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"deleted": true}})))
        .expect(1)
        .mount(&server)
        .await;

    let result = app_client(server.uri())
        .delete_webhook_stream_link(WEBHOOK_ID)
        .send()
        .await
        .expect("the link is deleted");

    assert!(result.data.deleted);
}
