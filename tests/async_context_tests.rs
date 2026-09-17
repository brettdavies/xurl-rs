//! The library is callable from inside a caller's async runtime: a shortcut
//! and a token refresh both complete under `#[tokio::test]` with no
//! `spawn_blocking` and no nested runtime.

use std::collections::BTreeMap;

use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xurl::api::{ApiClient, CallOptions};
use xurl::auth::Auth;
use xurl::config::Config;
use xurl::store::{App, LoadState, OAuth2Token, Token, TokenStore, TokenType};

fn config_for(server: &MockServer) -> Config {
    let mut cfg = Config::new();
    cfg.client_id = "cid".to_string();
    cfg.client_secret = "csec".to_string();
    cfg.redirect_uri = "http://localhost:8080/callback".to_string();
    cfg.auth_url = "https://x.com/i/oauth2/authorize".to_string();
    cfg.token_url = format!("{}/2/oauth2/token", server.uri());
    cfg.api_base_url = server.uri();
    cfg.info_url = format!("{}/2/users/me", server.uri());
    cfg
}

fn store_with_oauth2(tmp: &TempDir, expiration_time: u64) -> TokenStore {
    let mut store = TokenStore {
        apps: BTreeMap::new(),
        default_app: "default".to_string(),
        file_path: tmp.path().join(".xurl"),
        load_state: LoadState::Loaded,
    };
    let mut app = App {
        client_id: "cid".to_string(),
        client_secret: "csec".to_string(),
        default_user: "alice".to_string(),
        redirect_uri: String::new(),
        oauth2_tokens: BTreeMap::new(),
        oauth1_token: None,
        bearer_token: None,
        unnamed_oauth2_token: None,
    };
    app.oauth2_tokens.insert(
        "alice".to_string(),
        Token {
            token_type: TokenType::Oauth2,
            bearer: None,
            oauth2: Some(OAuth2Token {
                access_token: "stale-access-token".to_string(),
                refresh_token: "refresh-1".to_string(),
                expiration_time,
            }),
            oauth1: None,
        },
    );
    store.apps.insert("default".to_string(), app);
    store
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn me_body() -> serde_json::Value {
    serde_json::json!({"data": {"id": "1", "name": "Alice", "username": "alice"}})
}

async fn mount_me(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(me_body()))
        .mount(server)
        .await;
}

#[tokio::test]
async fn a_shortcut_completes_inside_a_caller_runtime() {
    let server = MockServer::start().await;
    mount_me(&server).await;
    let cfg = config_for(&server);
    let tmp = TempDir::new().unwrap();
    let auth = Auth::new_with_store_path(&cfg, &tmp.path().join(".xurl"))
        .with_token_store(store_with_oauth2(&tmp, now_secs() + 3600));
    let client = ApiClient::new(&cfg, auth).expect("client builds");

    let me = client
        .get_me(&CallOptions::default())
        .await
        .expect("the read completes");
    assert_eq!(me.data.username, "alice");
}

#[tokio::test]
async fn a_refresh_completes_inside_a_caller_runtime() {
    let server = MockServer::start().await;
    mount_me(&server).await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "fresh-access-token",
            "refresh_token": "refresh-2",
            "expires_in": 7200,
        })))
        .mount(&server)
        .await;
    let cfg = config_for(&server);
    let tmp = TempDir::new().unwrap();
    let auth = Auth::new_with_store_path(&cfg, &tmp.path().join(".xurl"))
        .with_token_store(store_with_oauth2(&tmp, now_secs() - 10));
    let client = ApiClient::new(&cfg, auth).expect("client builds");

    let me = client
        .get_me(&CallOptions::default())
        .await
        .expect("the refresh and the read complete");
    assert_eq!(me.data.username, "alice");

    let token_requests = server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| r.url.path() == "/2/oauth2/token")
        .count();
    assert_eq!(
        token_requests, 1,
        "exactly one refresh went to the token endpoint"
    );
}

#[tokio::test]
async fn a_refresh_the_server_rejects_is_an_auth_error_with_exit_77() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": "invalid_grant",
            "error_description": "refresh token revoked",
        })))
        .mount(&server)
        .await;
    let cfg = config_for(&server);
    let tmp = TempDir::new().unwrap();
    let auth = Auth::new_with_store_path(&cfg, &tmp.path().join(".xurl"))
        .with_token_store(store_with_oauth2(&tmp, now_secs() - 10));
    let client = ApiClient::new(&cfg, auth).expect("client builds");

    let err = client
        .get_me(&CallOptions::default())
        .await
        .expect_err("a rejected refresh fails the request");
    assert!(matches!(err, xurl::Error::Auth(_)), "got {err:?}");
    assert!(
        err.to_string().starts_with("RefreshTokenError"),
        "got {err}"
    );
    assert_eq!(err.exit_code(), xurl::error::EXIT_AUTH_REQUIRED);
}

#[tokio::test]
async fn a_refresh_against_a_silent_server_gives_up_at_the_configured_timeout() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_secs(10))
                .set_body_json(serde_json::json!({"access_token": "late"})),
        )
        .mount(&server)
        .await;
    let mut cfg = config_for(&server);
    cfg.http_timeout_secs = 1;
    let tmp = TempDir::new().unwrap();
    let auth = Auth::new_with_store_path(&cfg, &tmp.path().join(".xurl"))
        .with_token_store(store_with_oauth2(&tmp, now_secs() - 10));
    let client = ApiClient::new(&cfg, auth).expect("client builds");

    let started = std::time::Instant::now();
    let err = client
        .get_me(&CallOptions::default())
        .await
        .expect_err("the refresh gives up");
    let elapsed = started.elapsed();
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "the refresh must stop at http_timeout_secs, not hang; elapsed {elapsed:?}"
    );
    assert!(matches!(err, xurl::Error::Auth(_)), "got {err:?}");
}

#[tokio::test]
async fn a_media_processing_poll_sleeps_without_blocking_other_tasks() {
    let server = MockServer::start().await;
    let status_path = "/2/media/upload";
    Mock::given(method("GET"))
        .and(path(status_path))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "m1", "processing_info": {"state": "in_progress", "check_after_secs": 1, "progress_percent": 40}}
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(status_path))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "m1", "processing_info": {"state": "succeeded", "progress_percent": 100}}
        })))
        .mount(&server)
        .await;
    let cfg = config_for(&server);
    let tmp = TempDir::new().unwrap();
    let auth = Auth::new_with_store_path(&cfg, &tmp.path().join(".xurl"))
        .with_token_store(store_with_oauth2(&tmp, now_secs() + 3600));
    let client = ApiClient::new(&cfg, auth).expect("client builds");

    // A sibling task keeps ticking while the poll sleeps; a blocking sleep
    // would park the only runtime thread and freeze it.
    let ticks = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let ticker = {
        let ticks = ticks.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                ticks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        })
    };

    let response = xurl::api::execute_media_status("m1", "", "", true, false, &[], &client)
        .await
        .expect("processing completes");
    ticker.abort();

    assert_eq!(
        response
            .data
            .processing_info
            .as_ref()
            .map(|p| p.state.as_str()),
        Some("succeeded")
    );
    let polls = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path() == status_path)
        .count();
    assert_eq!(polls, 2, "one in-progress poll, one succeeded poll");
    assert!(
        ticks.load(std::sync::atomic::Ordering::SeqCst) >= 5,
        "the sibling task kept running during the one-second poll sleep"
    );
}

/// A minimal chunked HTTP/1.1 server that sends one line, then reports
/// whether the client hung up before it sent a second.
async fn slow_line_server() -> (String, tokio::sync::oneshot::Receiver<bool>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0u8; 4096];
        let _ = socket.read(&mut request).await;
        let head = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n";
        socket.write_all(head).await.unwrap();
        let line = b"{\"data\":{\"id\":\"1\"}}\n";
        socket
            .write_all(format!("{:x}\r\n", line.len()).as_bytes())
            .await
            .unwrap();
        socket.write_all(line).await.unwrap();
        socket.write_all(b"\r\n").await.unwrap();
        // Wait for the client to close: a read that returns 0 bytes or an
        // error is the hang-up; a timeout means the connection stayed open.
        let mut probe = [0u8; 16];
        let hung_up = matches!(
            tokio::time::timeout(std::time::Duration::from_secs(5), socket.read(&mut probe)).await,
            Ok(Ok(0)) | Ok(Err(_))
        );
        let _ = tx.send(hung_up);
    });
    (format!("http://{addr}"), rx)
}

#[tokio::test]
async fn dropping_a_stream_mid_response_releases_its_connection() {
    let (base_url, hung_up) = slow_line_server().await;
    let mut cfg = Config::new();
    cfg.api_base_url = base_url;
    let tmp = TempDir::new().unwrap();
    let auth = Auth::new_with_store_path(&cfg, &tmp.path().join(".xurl"));
    let client = ApiClient::new(&cfg, auth).expect("client builds");

    let options = xurl::api::RequestOptions {
        method: "GET".to_string(),
        target: xurl::api::RequestTarget::Template {
            path: "/2/tweets/sample/stream".to_string(),
            path_params: std::collections::HashMap::new(),
            query: Vec::new(),
        },
        no_auth: true,
        ..Default::default()
    };
    let mut lines = client.stream_request(&options).await.expect("stream opens");
    let first = lines
        .next_line()
        .await
        .expect("first line")
        .expect("a line arrived");
    assert_eq!(first, "{\"data\":{\"id\":\"1\"}}");
    drop(lines);

    assert!(
        hung_up.await.unwrap(),
        "the server must see the connection close when the stream is dropped"
    );
}
