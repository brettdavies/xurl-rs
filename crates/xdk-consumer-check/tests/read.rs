//! The read an embedder performs first, against a mock reached through the
//! base URL, plus the two things they touch next: a failure's next action and
//! the refresh hook.

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use xdk::api::Client;
use xdk::auth::OAuth2Credential;
use xdk::error::NextAction;
use xdk_consumer_check::{RememberedPair, describe_failure, recent_posts};

#[tokio::test]
async fn a_bearer_client_reads_typed_posts_and_the_rate_limit_window() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .and(header("User-Agent", "consumer-check/1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "data": [{"id": "1", "text": "hello"}, {"id": "2", "text": "world"}]
                }))
                .insert_header("x-rate-limit-remaining", "41"),
        )
        .mount(&server)
        .await;

    let client = Client::builder()
        .bearer("app-only-token")
        .base_url(server.uri())
        .user_agent("consumer-check/1")
        .build()
        .expect("client builds from a credential in code");
    let (lines, window) = recent_posts(&client, "rust").await.expect("read");
    assert_eq!(lines, vec!["1: hello", "2: world"]);
    assert_eq!(window.and_then(|w| w.remaining), Some(41));
}

#[tokio::test]
async fn a_rate_limited_call_names_its_kind_next_action_and_exit_code() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "title": "Too Many Requests", "detail": "Too Many Requests", "type": "about:blank", "status": 429
        })))
        .mount(&server)
        .await;
    let client = Client::builder()
        .bearer("app-only-token")
        .base_url(server.uri())
        .build()
        .expect("client builds");

    let error = recent_posts(&client, "rust")
        .await
        .expect_err("429 is an error");
    let (kind, next, code) = describe_failure(&error);
    assert_eq!(kind, "rate-limited");
    // A rate limit has no recovery action for the caller to take, only a
    // window to wait out; the pointer is the documentation URL.
    assert_eq!(next, None::<NextAction>);
    assert!(
        error
            .docs_url()
            .is_some_and(|url| url.contains("rate-limits")),
        "docs pointer: {:?}",
        error.docs_url()
    );
    assert_eq!(code, xdk::error::EXIT_RATE_LIMITED);
}

#[tokio::test]
async fn the_refresh_hook_receives_the_rotated_pair() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "fresh-access", "refresh_token": "fresh-refresh",
            "token_type": "bearer", "expires_in": 7200
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .and(header("Authorization", "Bearer fresh-access"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "42", "name": "Embedder", "username": "embedder"}
        })))
        .mount(&server)
        .await;

    let hook = RememberedPair::default();
    let client = Client::builder()
        .oauth2(OAuth2Credential {
            client_id: "client-id".into(),
            client_secret: "client-secret".into(),
            access_token: "stale-access".into(),
            refresh_token: Some("stale-refresh".into()),
            expires_at: Some(std::time::UNIX_EPOCH),
        })
        .on_token_refreshed(hook.clone())
        .base_url(server.uri())
        .token_url(format!("{}/2/oauth2/token", server.uri()))
        .build()
        .expect("client builds");

    let me = client.get_me().send().await.expect("refresh then read");
    assert_eq!(me.data.username, "embedder");
    assert_eq!(
        hook.latest(),
        Some((
            "fresh-access".to_string(),
            Some("fresh-refresh".to_string())
        ))
    );
}
