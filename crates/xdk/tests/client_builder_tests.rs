//! A client built from a credential held in code, with no store file and no
//! environment variable, performs reads and hands rotated OAuth2 tokens to
//! the registered hook.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use tempfile::TempDir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

use xdk::Error;
use xdk::api::Client;
use xdk::api::auth_matrix::WireScheme;
use xdk::auth::{BoxError, OAuth1Credential, OAuth2Credential, OnTokenRefreshed};
use xdk::store::TokenStore;

fn me_body() -> serde_json::Value {
    serde_json::json!({"data": {"id": "1", "name": "Alice", "username": "alice"}})
}

fn search_body() -> serde_json::Value {
    serde_json::json!({"data": [{"id": "7", "text": "hello"}]})
}

async fn mount_me(server: &MockServer, bearer: &str) {
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .and(header("Authorization", format!("Bearer {bearer}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(me_body()))
        .mount(server)
        .await;
}

async fn mount_refresh(server: &MockServer, expected_calls: u64) {
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "new-at",
            "refresh_token": "new-rt",
            "expires_in": 7200
        })))
        .expect(expected_calls)
        .mount(server)
        .await;
}

async fn received(server: &MockServer) -> Vec<Request> {
    server.received_requests().await.unwrap_or_default()
}

fn header_value(request: &Request, name: &str) -> Option<String> {
    request
        .headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

fn oauth2(expires_at: Option<SystemTime>) -> OAuth2Credential {
    OAuth2Credential {
        client_id: "cid".to_string(),
        client_secret: "csec".to_string(),
        access_token: "old-at".to_string(),
        refresh_token: Some("old-rt".to_string()),
        expires_at,
    }
}

fn expired_oauth2() -> OAuth2Credential {
    oauth2(Some(SystemTime::now() - Duration::from_secs(60)))
}

fn live_oauth2() -> OAuth2Credential {
    oauth2(Some(SystemTime::now() + Duration::from_secs(3600)))
}

fn oauth1() -> OAuth1Credential {
    OAuth1Credential {
        consumer_key: "ck".to_string(),
        consumer_secret: "cs".to_string(),
        access_token: "at".to_string(),
        token_secret: "ts".to_string(),
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Records every credential the client hands it.
#[derive(Clone, Default)]
struct Recorder {
    seen: Arc<Mutex<Vec<OAuth2Credential>>>,
}

impl OnTokenRefreshed for Recorder {
    fn on_token_refreshed<'a>(
        &'a self,
        credential: &'a OAuth2Credential,
    ) -> Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'a>> {
        Box::pin(async move {
            self.seen.lock().unwrap().push(credential.clone());
            Ok(())
        })
    }
}

/// Fails every persistence attempt the way a full disk would.
struct FailingSink;

impl OnTokenRefreshed for FailingSink {
    fn on_token_refreshed<'a>(
        &'a self,
        _credential: &'a OAuth2Credential,
    ) -> Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'a>> {
        Box::pin(async { Err("disk full".into()) })
    }
}

// ── Credentials in code ────────────────────────────────────────────────

#[tokio::test]
async fn a_bearer_token_in_code_performs_a_read() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .and(header("Authorization", "Bearer app-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_body()))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .bearer("app-token")
        .base_url(server.uri())
        .build()
        .expect("a bearer token is a complete credential");

    let posts = client
        .search_posts("hello", 10)
        .send()
        .await
        .expect("the read completes");
    assert_eq!(posts.data[0].id, "7");
}

#[tokio::test]
async fn an_oauth2_pair_in_code_performs_a_read() {
    let server = MockServer::start().await;
    mount_me(&server, "old-at").await;

    let client = Client::builder()
        .oauth2(live_oauth2())
        .base_url(server.uri())
        .build()
        .unwrap();

    let me = client.get_me().send().await.expect("the read completes");
    assert_eq!(me.data.username, "alice");
}

#[tokio::test]
async fn oauth1_credentials_in_code_sign_the_read() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(me_body()))
        .mount(&server)
        .await;

    let client = Client::builder()
        .oauth1(oauth1())
        .base_url(server.uri())
        .build()
        .unwrap();

    client.get_me().send().await.expect("the read completes");

    let requests = received(&server).await;
    let authorization = header_value(&requests[0], "Authorization").expect("signed");
    assert!(
        authorization.starts_with("OAuth ") && authorization.contains("oauth_consumer_key=\"ck\""),
        "an OAuth1 signature carries the consumer key: {authorization}"
    );
}

#[tokio::test]
async fn an_explicit_scheme_on_the_call_overrides_the_preference_order() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_body()))
        .mount(&server)
        .await;

    let client = Client::builder()
        .bearer("app-token")
        .oauth2(live_oauth2())
        .base_url(server.uri())
        .build()
        .unwrap();

    client.search_posts("hello", 10).send().await.unwrap();
    client
        .search_posts("hello", 10)
        .auth(WireScheme::App)
        .send()
        .await
        .unwrap();

    let requests = received(&server).await;
    assert_eq!(
        header_value(&requests[0], "Authorization").as_deref(),
        Some("Bearer old-at"),
        "OAuth2 wins the auto-detect order"
    );
    assert_eq!(
        header_value(&requests[1], "Authorization").as_deref(),
        Some("Bearer app-token"),
        "an explicit scheme is honored"
    );
}

#[tokio::test]
async fn a_builder_with_no_credential_is_a_validation_error() {
    let err = Client::builder().build().expect_err("nothing to send as");
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
}

// ── Refresh and the hook ───────────────────────────────────────────────

#[tokio::test]
async fn a_refresh_delivers_the_rotated_pair_and_expiry_to_the_hook() {
    let server = MockServer::start().await;
    mount_refresh(&server, 1).await;
    mount_me(&server, "new-at").await;
    let recorder = Recorder::default();

    let client = Client::builder()
        .oauth2(expired_oauth2())
        .on_token_refreshed(recorder.clone())
        .base_url(server.uri())
        .token_url(format!("{}/2/oauth2/token", server.uri()))
        .build()
        .unwrap();

    let before = SystemTime::now();
    client
        .get_me()
        .send()
        .await
        .expect("the refreshed read completes");

    let seen = recorder.seen.lock().unwrap();
    assert_eq!(seen.len(), 1, "one rotation, one delivery");
    let rotated = &seen[0];
    assert_eq!(rotated.access_token, "new-at");
    assert_eq!(rotated.refresh_token.as_deref(), Some("new-rt"));
    assert_eq!(rotated.client_id, "cid");
    assert_eq!(rotated.client_secret, "csec");
    let expires_at = rotated.expires_at.expect("the response carried expires_in");
    let lower = before + Duration::from_secs(7200);
    let upper = SystemTime::now() + Duration::from_secs(7200);
    assert!(
        expires_at >= lower && expires_at <= upper,
        "expiry is now plus expires_in"
    );
}

#[tokio::test]
async fn the_store_backed_hook_persists_the_rotated_pair() {
    let server = MockServer::start().await;
    mount_refresh(&server, 1).await;
    mount_me(&server, "new-at").await;

    let tmp = TempDir::new().unwrap();
    let store_path = tmp.path().join(".xurl");
    let mut store = TokenStore::new_with_path(store_path.to_str().unwrap());
    store.add_app("main", "cid", "csec").unwrap();
    store
        .save_oauth2_token_for_app("main", "alice", "old-at", "old-rt", 0)
        .unwrap();

    let client = Client::builder()
        .oauth2(expired_oauth2())
        .on_token_refreshed(store)
        .base_url(server.uri())
        .token_url(format!("{}/2/oauth2/token", server.uri()))
        .build()
        .unwrap();

    client
        .get_me()
        .send()
        .await
        .expect("the refreshed read completes");

    let reloaded = TokenStore::new_with_path(store_path.to_str().unwrap());
    let token = reloaded
        .get_oauth2_token_for_app("main", "alice")
        .and_then(|t| t.oauth2.clone())
        .expect("alice's token survives the rotation");
    assert_eq!(token.access_token, "new-at");
    assert_eq!(token.refresh_token, "new-rt");
    assert!(
        token.expiration_time > now_secs() + 7000,
        "the stored expiry is the rotated one: {}",
        token.expiration_time
    );
}

#[tokio::test]
async fn a_failing_hook_fails_the_request_and_keeps_the_new_token() {
    let server = MockServer::start().await;
    mount_refresh(&server, 1).await;
    mount_me(&server, "new-at").await;

    let client = Client::builder()
        .oauth2(expired_oauth2())
        .on_token_refreshed(FailingSink)
        .base_url(server.uri())
        .token_url(format!("{}/2/oauth2/token", server.uri()))
        .build()
        .unwrap();

    let err = client
        .get_me()
        .send()
        .await
        .expect_err("the triggering request fails");
    match &err {
        Error::TokenStore(message) => assert!(message.contains("disk full"), "{message}"),
        other => panic!("expected Error::TokenStore, got {other:?}"),
    }

    client
        .get_me()
        .send()
        .await
        .expect("a retry uses the installed token without a second refresh");
}

#[tokio::test]
async fn an_expired_token_without_a_refresh_token_is_an_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(me_body()))
        .expect(0)
        .mount(&server)
        .await;

    let mut credential = expired_oauth2();
    credential.refresh_token = None;
    let client = Client::builder()
        .oauth2(credential)
        .base_url(server.uri())
        .build()
        .unwrap();

    let err = client.get_me().send().await.expect_err("nothing to send");
    assert!(matches!(err, Error::Auth(_)), "{err:?}");
}

#[tokio::test]
async fn concurrent_calls_on_an_expired_token_refresh_once() {
    let server = MockServer::start().await;
    mount_refresh(&server, 1).await;
    mount_me(&server, "new-at").await;

    let client = Client::builder()
        .oauth2(expired_oauth2())
        .base_url(server.uri())
        .token_url(format!("{}/2/oauth2/token", server.uri()))
        .build()
        .unwrap();

    let (a, b, c) = tokio::join!(
        client.get_me().send(),
        client.get_me().send(),
        client.get_me().send()
    );
    assert!(a.is_ok() && b.is_ok() && c.is_ok(), "{a:?} {b:?} {c:?}");
}

// ── Per-call options ───────────────────────────────────────────────────

#[tokio::test]
async fn a_user_supplied_x_b3_flags_header_wins_over_trace_on_the_call() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(me_body()))
        .mount(&server)
        .await;

    let client = Client::builder()
        .oauth2(live_oauth2())
        .base_url(server.uri())
        .build()
        .unwrap();

    client
        .get_me()
        .trace(true)
        .header("X-B3-Flags", "0")
        .send()
        .await
        .expect("request must succeed with user-supplied X-B3-Flags + trace");

    let requests = received(&server).await;
    let flags: Vec<String> = requests[0]
        .headers
        .get_all("X-B3-Flags")
        .iter()
        .filter_map(|v| v.to_str().ok().map(str::to_owned))
        .collect();
    assert_eq!(
        flags,
        vec!["0".to_string()],
        "user-supplied X-B3-Flags must win over trace=true append"
    );
}

#[tokio::test]
async fn trace_on_the_call_sends_the_x_b3_flags_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .and(header("X-B3-Flags", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(me_body()))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .oauth2(live_oauth2())
        .base_url(server.uri())
        .build()
        .unwrap();

    client.get_me().trace(true).send().await.unwrap();
}

#[tokio::test]
async fn a_pagination_token_reaches_list_calls_only() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_body()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(me_body()))
        .mount(&server)
        .await;

    let client = Client::builder()
        .oauth2(live_oauth2())
        .base_url(server.uri())
        .build()
        .unwrap();

    client
        .search_posts("hello", 10)
        .pagination_token("page-2")
        .send()
        .await
        .unwrap();
    client
        .get_me()
        .pagination_token("page-2")
        .send()
        .await
        .unwrap();

    let requests = received(&server).await;
    assert!(
        requests[0]
            .url
            .query()
            .unwrap_or("")
            .ends_with("pagination_token=page-2"),
        "a list call carries the cursor last: {}",
        requests[0].url
    );
    assert!(
        requests[1]
            .url
            .query()
            .is_none_or(|q| !q.contains("pagination_token")),
        "a single-item call ignores the cursor: {}",
        requests[1].url
    );
}

#[tokio::test]
async fn a_call_timeout_overrides_the_client_bound() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(me_body())
                .set_delay(Duration::from_millis(400)),
        )
        .mount(&server)
        .await;

    let client = Client::builder()
        .oauth2(live_oauth2())
        .base_url(server.uri())
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let err = client
        .get_me()
        .timeout(Duration::from_millis(50))
        .send()
        .await
        .expect_err("the per-call bound fires first");
    assert!(matches!(err, Error::Http(_)), "{err:?}");
}

// ── Review follow-ups: hook scoping, unauthenticated calls, cursor breadth ──

#[tokio::test]
async fn the_scoped_store_hook_persists_into_the_named_app_not_the_default() {
    let server = MockServer::start().await;
    mount_refresh(&server, 1).await;
    mount_me(&server, "new-at").await;

    let tmp = TempDir::new().unwrap();
    let store_path = tmp.path().join(".xurl");
    let mut store = TokenStore::new_with_path(store_path.to_str().unwrap());
    store.add_app("main", "cid", "csec").unwrap();
    store
        .save_oauth2_token_for_app("main", "alice", "main-at", "main-rt", 0)
        .unwrap();
    store.add_app("other", "cid", "csec").unwrap();
    store
        .save_oauth2_token_for_app("other", "bob", "old-at", "old-rt", 0)
        .unwrap();
    store.set_default_app("main").unwrap();

    let client = Client::builder()
        .oauth2(expired_oauth2())
        .on_token_refreshed(store.refresh_hook_for("other"))
        .base_url(server.uri())
        .token_url(format!("{}/2/oauth2/token", server.uri()))
        .build()
        .unwrap();

    client
        .get_me()
        .send()
        .await
        .expect("the refreshed read completes");

    let reloaded = TokenStore::new_with_path(store_path.to_str().unwrap());
    let bob = reloaded
        .get_oauth2_token_for_app("other", "bob")
        .and_then(|t| t.oauth2.clone())
        .expect("bob's token survives");
    assert_eq!(
        bob.access_token, "new-at",
        "the named app received the rotation"
    );
    let alice = reloaded
        .get_oauth2_token_for_app("main", "alice")
        .and_then(|t| t.oauth2.clone())
        .expect("alice's token survives");
    assert_eq!(
        alice.access_token, "main-at",
        "the default app is untouched"
    );
}

#[tokio::test]
async fn no_auth_on_a_call_sends_no_authorization_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(me_body()))
        .mount(&server)
        .await;

    let client = Client::builder()
        .oauth2(live_oauth2())
        .base_url(server.uri())
        .build()
        .unwrap();

    client.get_me().no_auth(true).send().await.unwrap();

    let requests = received(&server).await;
    assert!(
        header_value(&requests[0], "Authorization").is_none(),
        "an unauthenticated call carries no Authorization header"
    );
}

/// Every list shortcut sends the cursor; every single-item shortcut drops
/// it. A new list endpoint that forgets `.paginated()` fails here.
#[tokio::test]
async fn the_pagination_token_reaches_every_list_call_and_no_other() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .mount(&server)
        .await;
    let client = Client::builder()
        .oauth2(live_oauth2())
        .base_url(server.uri())
        .build()
        .unwrap();

    macro_rules! probe {
        ($name:literal, $call:expr) => {{
            // Decoding may fail against the catch-all body; only the wire matters.
            let _ = $call.pagination_token("PROBE").send().await;
            let last = received(&server).await.pop().expect("a request was sent");
            (
                $name,
                last.url
                    .query()
                    .is_some_and(|q| q.ends_with("pagination_token=PROBE")),
            )
        }};
    }

    let lists = [
        probe!("search_posts", client.search_posts("q", 10)),
        probe!("get_timeline", client.get_timeline("1", 10)),
        probe!("get_mentions", client.get_mentions("1", 10)),
        probe!("get_bookmarks", client.get_bookmarks("1", 10)),
        probe!("get_liked_posts", client.get_liked_posts("1", 10)),
        probe!("get_dm_events", client.get_dm_events(10)),
        probe!("get_following", client.get_following("1", 10)),
        probe!("get_followers", client.get_followers("1", 10)),
        probe!("get_muted", client.get_muted("1", 10)),
        probe!("get_blocked", client.get_blocked("1", 10)),
    ];
    let singles = [
        probe!("get_me", client.get_me()),
        probe!("lookup_user", client.lookup_user("alice")),
        probe!("read_post", client.read_post("1")),
        probe!("get_usage", client.get_usage()),
        probe!("get_usage_credits", client.get_usage_credits()),
    ];

    let missing: Vec<_> = lists
        .iter()
        .filter(|(_, sent)| !sent)
        .map(|(n, _)| n)
        .collect();
    let leaked: Vec<_> = singles
        .iter()
        .filter(|(_, sent)| *sent)
        .map(|(n, _)| n)
        .collect();
    assert!(
        missing.is_empty(),
        "list calls that dropped the cursor: {missing:?}"
    );
    assert!(
        leaked.is_empty(),
        "single-item calls that sent a cursor: {leaked:?}"
    );
}

#[tokio::test]
async fn builder_user_agent_replaces_the_library_default() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/2/tweets/search/recent"))
        .and(wiremock::matchers::header("User-Agent", "embedder/1.2.3"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = xdk::api::Client::builder()
        .bearer("app-only-token")
        .base_url(server.uri())
        .user_agent("embedder/1.2.3")
        .build()
        .expect("client builds");
    client
        .search_posts("rust", 10)
        .send()
        .await
        .expect("the mock matched the embedder's User-Agent");
}

#[tokio::test]
async fn the_client_remembers_the_last_rate_limit_window() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/2/tweets/search/recent"))
        .respond_with(
            wiremock::ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": []}))
                .insert_header("x-rate-limit-limit", "450")
                .insert_header("x-rate-limit-remaining", "449")
                .insert_header("x-rate-limit-reset", "1758067200"),
        )
        .mount(&server)
        .await;

    let client = xdk::api::Client::builder()
        .bearer("app-only-token")
        .base_url(server.uri())
        .build()
        .expect("client builds");
    assert!(
        client.last_rate_limit().is_none(),
        "no window before the first response"
    );
    client
        .search_posts("rust", 10)
        .send()
        .await
        .expect("mock read");
    let window = client
        .last_rate_limit()
        .expect("the response headers were recorded");
    assert_eq!(window.limit, Some(450));
    assert_eq!(window.remaining, Some(449));
    assert_eq!(window.reset_at, Some(1_758_067_200));
}
