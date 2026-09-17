//! Integration tests for the OAuth2 PKCE flow's listener-before-browser
//! ordering.
//!
//! The contract under test: `run_oauth2_flow` MUST bind the callback listener
//! and enter its accept loop BEFORE invoking `browser_opener`. The recording
//! opener observes this by connecting back to the listener as its first
//! action; if the listener were not yet bound the connect would fail and the
//! opener would propagate the error. The opener also delivers the canonical
//! `/callback?code=...&state=...` request that drives the flow to completion,
//! so the test exercises the full path end-to-end via a wiremock token endpoint.
//!
//! **Regression check**: intentionally inverting the bind/open order in
//! `src/auth/oauth2.rs` (calling `browser_opener` before
//! `wait_for_callback_with`) causes the opener's TCP connect to fail because
//! the listener socket is not yet bound. This is the failure mode that locks
//! in the ordering correctness.

use std::io::{Read, Write};
use std::net::{TcpListener as StdTcpListener, TcpStream as StdTcpStream};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use tokio_util::sync::CancellationToken;
use xdk::auth::Auth;
use xdk::auth::oauth2::run_oauth2_flow;
use xdk::config::Config;

// ── Recording opener shared state ─────────────────────────────────────────
//
// The browser-opener signature is `fn(&str) -> io::Result<()>` — a bare
// function pointer, no closure captures. Tests park observable state in
// a `OnceLock<Mutex<...>>` and the opener function reads/writes it.
//
// One static recorder serves both tests because tests run serially under
// `cargo test` *unless* they are in different test binaries; this file's
// tests share state and must run serially.

struct Recorder {
    /// `Instant::now()` captured the moment the opener is invoked.
    opener_called_at: Option<Instant>,
    /// The auth URL passed by `run_oauth2_flow`.
    auth_url: Option<String>,
    /// The address the opener should connect back to with the callback.
    callback_target: Option<String>,
    /// The state nonce extracted from the auth URL by the opener.
    captured_state: Option<String>,
    /// Connect-attempt outcome (Ok=connected, Err=listener wasn't bound).
    connect_result: Option<Result<(), String>>,
}

impl Recorder {
    const fn new() -> Self {
        Self {
            opener_called_at: None,
            auth_url: None,
            callback_target: None,
            captured_state: None,
            connect_result: None,
        }
    }
}

fn recorder() -> &'static Mutex<Recorder> {
    static R: OnceLock<Mutex<Recorder>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(Recorder::new()))
}

fn reset_recorder(callback_target: &str) {
    let mut r = recorder().lock().unwrap();
    *r = Recorder::new();
    r.callback_target = Some(callback_target.to_string());
}

// ── Recording opener (function pointer) ───────────────────────────────────
//
// Captures the call instant, extracts `state` from the auth URL, then
// connects to the listener with a full callback request. The connect
// succeeds only if the listener is already bound — this is the
// listener-before-browser assertion in observable form.

fn recording_opener(url: &str) -> std::io::Result<()> {
    let now = Instant::now();
    let (callback_target, state) = {
        let mut r = recorder().lock().unwrap();
        r.opener_called_at = Some(now);
        r.auth_url = Some(url.to_string());

        let parsed = url::Url::parse(url).expect("opener received valid auth URL");
        let state = parsed
            .query_pairs()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.to_string())
            .expect("auth URL contains state");
        r.captured_state = Some(state.clone());
        let target = r
            .callback_target
            .clone()
            .expect("test set callback_target before flow");
        (target, state)
    };

    // Try to connect; ANY failure here proves the listener wasn't bound
    // before the opener fired.
    let connect_outcome = (|| -> std::io::Result<()> {
        let mut stream = StdTcpStream::connect(&callback_target)?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        let req = format!(
            "GET /callback?code=AUTHCODE&state={state} HTTP/1.1\r\nHost: {callback_target}\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(req.as_bytes())?;
        let mut buf = String::new();
        let _ = stream.read_to_string(&mut buf);
        Ok(())
    })();

    {
        let mut r = recorder().lock().unwrap();
        r.connect_result = Some(
            connect_outcome
                .as_ref()
                .copied()
                .map_err(ToString::to_string),
        );
    }

    connect_outcome
}

// ── Test scaffolding ──────────────────────────────────────────────────────

fn pick_free_port() -> u16 {
    let l = StdTcpListener::bind("127.0.0.1:0").expect("bind 127.0.0.1:0");
    let port = l.local_addr().unwrap().port();
    drop(l);
    port
}

fn test_config(token_url: &str, info_url: &str, redirect_uri: &str) -> Config {
    let mut cfg = Config::new();
    cfg.client_id = "test-client-id".to_string();
    cfg.client_secret = "test-client-secret".to_string();
    cfg.redirect_uri = redirect_uri.to_string();
    cfg.auth_url = "https://x.com/i/oauth2/authorize".to_string();
    cfg.token_url = token_url.to_string();
    cfg.api_base_url = "https://example.invalid".to_string();
    cfg.info_url = info_url.to_string();
    cfg.app_name = String::new();
    cfg
}

fn test_auth(cfg: Config, tmp: &TempDir, redirect_uri: &str) -> Auth {
    let store_path = tmp.path().join(".xurl");

    // Pre-stage the store with the test app's redirect_uri so the resolver
    // (run by Auth::new_with_store_path) returns it instead of the legacy
    // default. Tests bypass the validator by writing YAML directly because
    // the validator is exercised in U1/U2 tests.
    let yaml = format!(
        "apps:\n  default:\n    client_id: 'test-client-id'\n    client_secret: 'test-client-secret'\n    redirect_uri: '{redirect_uri}'\n    oauth2_tokens: {{}}\ndefault_app: default\n"
    );
    std::fs::write(&store_path, yaml).expect("write tempdir store");

    Auth::new_with_store_path(&cfg, &store_path)
}

// ── The pivotal ordering test ─────────────────────────────────────────────

#[tokio::test]
async fn listener_bound_before_browser_opener_invoked() {
    let port = pick_free_port();
    let callback_target = format!("127.0.0.1:{port}");
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    reset_recorder(&callback_target);

    // Wiremock token + userinfo endpoints, driven from a dedicated runtime.
    let server = MockServer::start().await;
    let token_url = format!("{}/2/oauth2/token", server.uri());
    let info_url = format!("{}/2/users/me", server.uri());

    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "ACCESS-TOKEN",
            "refresh_token": "REFRESH-TOKEN",
            "expires_in": 7200,
            "token_type": "bearer"
        })))
        .mount(&server)
        .await;

    let tmp = TempDir::new().unwrap();
    let cfg = test_config(&token_url, &info_url, &redirect_uri);
    let mut auth = test_auth(cfg, &tmp, &redirect_uri);

    let flow_started_at = Instant::now();
    let token = run_oauth2_flow(
        &mut auth,
        &reqwest::Client::new(),
        "testuser",
        CancellationToken::new(),
        recording_opener,
    )
    .await
    .expect("flow completes");
    assert_eq!(token, "ACCESS-TOKEN");

    let r = recorder().lock().unwrap();

    // The opener was invoked.
    let opener_at = r
        .opener_called_at
        .expect("opener was called by run_oauth2_flow");
    assert!(opener_at >= flow_started_at, "opener time monotonicity");

    // The opener's TCP connect succeeded — proves the listener was bound
    // and accepting BEFORE the opener fired. This is the listener-before-
    // browser ordering, observable.
    let connect = r
        .connect_result
        .as_ref()
        .expect("opener recorded a connect outcome");
    assert!(
        connect.is_ok(),
        "listener must be bound before opener fires; got connect error: {connect:?}"
    );

    // The opener saw the same `state` nonce that drives the listener — the
    // flow accepted the recorded request and returned the code.
    assert!(r.captured_state.is_some());

    // The token store was updated with the new access token (full flow).
    let token = auth
        .token_store()
        .get_oauth2_token("testuser")
        .expect("token saved")
        .clone();
    assert_eq!(
        token.oauth2.as_ref().expect("oauth2 present").access_token,
        "ACCESS-TOKEN"
    );
}

// ══════════════════════════════════════════════════════════════════════════
// Headless step 2, driven through the CLI entrypoint
// ══════════════════════════════════════════════════════════════════════════
//
// The listener test above drives `run_oauth2_flow` directly. These drive
// `xr auth oauth2 --no-browser --step 2` through `run_with_overrides`, which
// is the path a headless operator and an agent actually take: the pending
// state written by step 1 is on disk, the redirect URL arrives as a flag, and
// the token endpoint answers over the network. Every case mounts its mock with
// `.expect(1)`, so a test that stopped reaching the exchange fails on drop
// rather than passing vacuously.

const PENDING_STATE: &str = "TEST-STATE-NONCE";
const PENDING_VERIFIER: &str = "test-code-verifier-01234567890123456789012345678901234567";

/// A store holding one app with the credentials the pending state names.
fn seed_store(store_path: &std::path::Path) {
    std::fs::write(
        store_path,
        "apps:\n  default:\n    client_id: 'test-client-id'\n    client_secret: 'test-client-secret'\n    oauth2_tokens: {}\ndefault_app: default\n",
    )
    .expect("write tempdir store");
}

/// The pending state step 1 leaves beside the store, owned by `default`.
fn seed_pending(store_path: &std::path::Path) -> std::path::PathBuf {
    seed_pending_for_app(store_path, "default")
}

/// A store holding two apps that share a client id, so the only thing telling
/// them apart is the name the runtime context resolves to.
fn seed_store_two_apps(store_path: &std::path::Path) {
    std::fs::write(
        store_path,
        "apps:\n  default:\n    client_id: 'test-client-id'\n    client_secret: 'test-client-secret'\n    oauth2_tokens: {}\n  work:\n    client_id: 'test-client-id'\n    client_secret: 'test-client-secret'\n    oauth2_tokens: {}\ndefault_app: default\n",
    )
    .expect("write tempdir store");
}

/// The pending state step 1 leaves beside the store, owned by `app_name`.
fn seed_pending_for_app(store_path: &std::path::Path, app_name: &str) -> std::path::PathBuf {
    let path = xdk::auth::pending::pending_path_for_store(store_path);
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs();
    let state = xdk::auth::pending::PendingOAuth2State {
        code_verifier: PENDING_VERIFIER.to_string(),
        state: PENDING_STATE.to_string(),
        client_id: "test-client-id".to_string(),
        app_name: app_name.to_string(),
        created_at,
    };
    xdk::auth::pending::save(&state, &path).expect("seed pending state");
    path
}

fn step2_overrides(token_url: &str, info_url: &str) -> xdk::config::EnvOverrides {
    xdk::config::EnvOverrides {
        client_id: Some("test-client-id".to_string()),
        client_secret: Some("test-client-secret".to_string()),
        token_url: Some(token_url.to_string()),
        info_url: Some(info_url.to_string()),
        ..xdk::config::EnvOverrides::default()
    }
}

/// The redirect URL a browser hands back, carrying the pending nonce.
fn redirect_url() -> String {
    format!("http://localhost:8080/callback?code=AUTHCODE&state={PENDING_STATE}")
}

async fn run_cli(
    store: &std::path::Path,
    overrides: &xdk::config::EnvOverrides,
    args: &[&str],
) -> (i32, String, String) {
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code =
        xurl::cli::runner::run_with_overrides(args, &mut stdout, &mut stderr, store, overrides)
            .await;
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

fn token_mock(body: serde_json::Value, status: u16) -> Mock {
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .expect(1)
}

fn ok_token_body() -> serde_json::Value {
    serde_json::json!({
        "access_token": "ACCESS-TOKEN",
        "refresh_token": "REFRESH-TOKEN",
        "expires_in": 7200,
        "token_type": "bearer"
    })
}

#[tokio::test]
async fn step2_exchanges_the_code_and_saves_the_token() {
    let server = MockServer::start().await;
    token_mock(ok_token_body(), 200).mount(&server).await;

    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let pending = seed_pending(&store);

    let overrides = step2_overrides(
        &format!("{}/2/oauth2/token", server.uri()),
        &format!("{}/2/users/me", server.uri()),
    );
    let (code, stdout, stderr) = run_cli(
        &store,
        &overrides,
        &[
            "xr",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
            "--auth-url",
            &redirect_url(),
            "alice",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("OAuth2 authentication successful"),
        "stdout: {stdout}"
    );

    let saved = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    let token = saved.get_oauth2_token("alice").expect("token saved");
    assert_eq!(
        token.oauth2.as_ref().expect("oauth2 present").access_token,
        "ACCESS-TOKEN"
    );
    assert!(
        !pending.exists(),
        "a completed exchange deletes the pending state"
    );
}

#[tokio::test]
async fn step2_resolves_the_username_when_the_positional_is_absent() {
    let server = MockServer::start().await;
    token_mock(ok_token_body(), 200).mount(&server).await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "1", "username": "discovered", "name": "Discovered"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    seed_pending(&store);

    let overrides = step2_overrides(
        &format!("{}/2/oauth2/token", server.uri()),
        &format!("{}/2/users/me", server.uri()),
    );
    let (code, _stdout, stderr) = run_cli(
        &store,
        &overrides,
        &[
            "xr",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
            "--auth-url",
            &redirect_url(),
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let saved = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    assert!(
        saved.get_oauth2_token("discovered").is_some(),
        "the token lands under the username the info endpoint returned"
    );
}

#[tokio::test]
async fn step2_reports_success_under_output_json() {
    let server = MockServer::start().await;
    token_mock(ok_token_body(), 200).mount(&server).await;

    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    seed_pending(&store);

    let overrides = step2_overrides(
        &format!("{}/2/oauth2/token", server.uri()),
        &format!("{}/2/users/me", server.uri()),
    );
    let (code, stdout, stderr) = run_cli(
        &store,
        &overrides,
        &[
            "xr",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
            "--auth-url",
            &redirect_url(),
            "alice",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout is one JSON document");
    assert!(
        v["message"]
            .as_str()
            .is_some_and(|m| m.contains("OAuth2 authentication successful")),
        "envelope: {v}"
    );
    assert!(
        !stdout.contains('\u{1b}'),
        "no escape sequence reaches the structured rendering: {stdout:?}"
    );
}

#[tokio::test]
async fn step2_keeps_the_pending_state_when_the_token_endpoint_fails() {
    let server = MockServer::start().await;
    token_mock(
        serde_json::json!({"error": "server_error", "error_description": "upstream failure"}),
        500,
    )
    .mount(&server)
    .await;

    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let pending = seed_pending(&store);

    let overrides = step2_overrides(
        &format!("{}/2/oauth2/token", server.uri()),
        &format!("{}/2/users/me", server.uri()),
    );
    let (code, _stdout, stderr) = run_cli(
        &store,
        &overrides,
        &[
            "xr",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
            "--auth-url",
            &redirect_url(),
            "alice",
        ],
    )
    .await;

    // A failed exchange is an `Error::Auth`, so it carries
    // `EXIT_AUTH_REQUIRED` whatever the upstream status was.
    assert_eq!(code, xdk::error::EXIT_AUTH_REQUIRED, "stderr: {stderr}");
    assert!(
        stderr.contains("TokenExchangeError"),
        "the error names the exchange: {stderr}"
    );
    assert!(
        pending.exists(),
        "the pending state survives a failed exchange so step 2 can be retried"
    );
    let saved = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    assert!(
        saved.get_oauth2_token("alice").is_none(),
        "no token is written when the exchange fails"
    );
}

#[tokio::test]
async fn step2_saves_the_token_on_the_app_the_runtime_context_names() {
    let server = MockServer::start().await;
    token_mock(ok_token_body(), 200).mount(&server).await;

    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store_two_apps(&store);
    let pending = seed_pending_for_app(&store, "work");

    let overrides = step2_overrides(
        &format!("{}/2/oauth2/token", server.uri()),
        &format!("{}/2/users/me", server.uri()),
    );
    let (code, _stdout, stderr) = run_cli(
        &store,
        &overrides,
        &[
            "xr",
            "--app",
            "work",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
            "--auth-url",
            &redirect_url(),
            "alice",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");

    let saved = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    let on_work = saved
        .get_oauth2_token_for_app("work", "alice")
        .expect("the token lands on the app --app named");
    assert_eq!(
        on_work
            .oauth2
            .as_ref()
            .expect("oauth2 present")
            .access_token,
        "ACCESS-TOKEN"
    );
    // The default app is the one a routing slip would write to, so its absence
    // is what makes this test more than a restatement of the single-app case.
    assert!(
        saved.get_oauth2_token_for_app("default", "alice").is_none(),
        "the default app must not receive the token"
    );
    assert!(
        !pending.exists(),
        "a completed exchange deletes the pending state"
    );
}
