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
use std::net::TcpStream as StdTcpStream;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use tokio_util::sync::CancellationToken;
use xdk::auth::Auth;
use xdk::auth::oauth2::{run_oauth2_flow, run_oauth2_flow_on};
use xdk::config::{Config, EnvOverrides};

use common::CLOSED;
use common::token_endpoint::{Echo, NoRefusal, TOKEN_ENDPOINT_DOCS, big_page, no_refusal_answers};

mod common;

#[path = "../../xdk/tests/support/port_band.rs"]
mod port_band;

use port_band::{band_port, failed_to_bind};

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

fn test_config(token_url: &str, info_url: &str, redirect_uri: &str) -> Config {
    let mut cfg = Config::from_overrides(&EnvOverrides::default());
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
    // (run when the Auth is built) returns it instead of the built-in
    // default. The YAML is written directly, past the redirect-URI
    // validator, which has tests of its own.
    let yaml = format!(
        "apps:\n  default:\n    client_id: 'test-client-id'\n    client_secret: 'test-client-secret'\n    redirect_uri: '{redirect_uri}'\n    oauth2_tokens: {{}}\ndefault_app: default\n"
    );
    std::fs::write(&store_path, yaml).expect("write tempdir store");

    Auth::new_with_store_path_and_overrides(&cfg, &store_path, &EnvOverrides::default())
}

// ── The pivotal ordering test ─────────────────────────────────────────────

#[tokio::test]
async fn listener_bound_before_browser_opener_invoked() {
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

    let mut completed = None;
    for _ in 0..20 {
        let port = band_port();
        let redirect_uri = format!("http://127.0.0.1:{port}/callback");
        reset_recorder(&format!("127.0.0.1:{port}"));
        let tmp = TempDir::new().unwrap();
        let cfg = test_config(&token_url, &info_url, &redirect_uri);
        let mut auth = test_auth(cfg, &tmp, &redirect_uri);

        let flow_started_at = Instant::now();
        match run_oauth2_flow(
            &mut auth,
            &reqwest::Client::new(),
            "testuser",
            CancellationToken::new(),
            recording_opener,
        )
        .await
        {
            Ok(token) => {
                completed = Some((token, auth, tmp, flow_started_at));
                break;
            }
            // The test names the port before the flow binds it, so another
            // process can hold it; only that bind failing picks again.
            Err(error) if failed_to_bind(&error, port) => {}
            Err(error) => panic!("flow completes: {error}"),
        }
    }
    let (token, auth, _tmp, flow_started_at) =
        completed.expect("a port the flow could bind in 20 picks");
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
    save_flow(store_path, &saved_flow(app_name, "test-client-id"))
}

/// What step 1 saves for `app_name` under `client_id`.
fn saved_flow(app_name: &str, client_id: &str) -> xdk::auth::pending::PendingOAuth2State {
    xdk::auth::pending::PendingOAuth2State {
        code_verifier: PENDING_VERIFIER.to_string(),
        state: PENDING_STATE.to_string(),
        client_id: client_id.to_string(),
        app_name: app_name.to_string(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_secs(),
        scopes: Vec::new(),
    }
}

fn save_flow(
    store: &std::path::Path,
    state: &xdk::auth::pending::PendingOAuth2State,
) -> std::path::PathBuf {
    let path = xdk::auth::pending::pending_path_for_store(store);
    xdk::auth::pending::save(state, &path).expect("seed pending state");
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

    // The endpoint failed, so nothing about the credentials is in question.
    assert_eq!(code, xdk::error::EXIT_GENERAL_ERROR, "stderr: {stderr}");
    assert!(
        stderr.contains(FAILED_EXCHANGE),
        "the error's message is X's answer alone, so a line names the exchange: {stderr}"
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

// ── `--scopes` narrows what a sign-in requests ──────────────────────────

/// Overrides for step 1, which builds a URL and reaches no network.
fn step1_overrides() -> xdk::config::EnvOverrides {
    xdk::config::EnvOverrides {
        client_id: Some("test-client-id".to_string()),
        client_secret: Some("test-client-secret".to_string()),
        ..xdk::config::EnvOverrides::default()
    }
}

/// Runs headless step 1 with `extra` flags and returns the authorize URL.
async fn step1_url(store: &std::path::Path, extra: &[&str]) -> Url {
    let mut args = vec![
        "xr",
        "--output",
        "json",
        "auth",
        "oauth2",
        "--no-browser",
        "--step",
        "1",
    ];
    args.extend_from_slice(extra);
    let (code, stdout, stderr) = run_cli(store, &step1_overrides(), &args).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("a JSON object");
    Url::parse(v["auth_url"].as_str().expect("auth_url")).expect("the authorize URL parses")
}

fn query_param(url: &Url, name: &str) -> String {
    url.query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
        .unwrap_or_else(|| panic!("{url} carries no {name}"))
}

#[tokio::test]
async fn step1_requests_only_the_named_scopes_and_offline_access() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);

    let url = step1_url(&store, &["--scopes", "tweet.read,users.read"]).await;

    assert_eq!(
        query_param(&url, "scope"),
        "tweet.read users.read offline.access"
    );
}

#[tokio::test]
async fn step1_requests_every_scope_without_the_flag() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);

    let url = step1_url(&store, &[]).await;

    assert_eq!(query_param(&url, "scope").split(' ').count(), 26);
}

#[tokio::test]
async fn an_unknown_scope_is_rejected_before_anything_is_written() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);

    let (code, stdout, stderr) = run_cli(
        &store,
        &step1_overrides(),
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "1",
            "--scopes",
            "tweet.reed",
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(stdout.is_empty(), "no URL is printed: {stdout}");
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "validation", "got: {v}");
    let message = v["message"].as_str().expect("message");
    assert!(message.contains("tweet.reed"), "{message}");
    assert!(
        message.contains("tweet.read") && message.contains("offline.access"),
        "the valid scopes are listed: {message}"
    );
    assert!(
        !xdk::auth::pending::pending_path_for_store(&store).exists(),
        "no pending state is written"
    );
}

/// Step 2 takes the scope set from what step 1 saved, so the two steps cannot
/// describe different sign-ins: no flag is needed, and a flag that names a
/// different set is refused before the code is spent.
#[tokio::test]
async fn step2_reuses_the_scopes_step1_saved() {
    let server = MockServer::start().await;
    token_mock(ok_token_body(), 200).mount(&server).await;
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);

    let url = step1_url(&store, &["--scopes", "tweet.read,users.read"]).await;
    let pending_path = xdk::auth::pending::pending_path_for_store(&store);
    let pending = xdk::auth::pending::load(&pending_path).expect("step 1 saved its state");
    assert_eq!(
        pending.scopes,
        ["tweet.read", "users.read", "offline.access"]
    );

    let redirect = format!(
        "http://localhost:8080/callback?code=AUTHCODE&state={}",
        urlencode(&query_param(&url, "state"))
    );
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
            &redirect,
            "--scopes",
            "dm.read",
            "alice",
        ],
    )
    .await;
    assert_eq!(
        code, 77,
        "a different scope set is refused; stderr: {stderr}"
    );
    assert!(
        stderr.contains("dm.read") && stderr.contains("tweet.read"),
        "the refusal names both sets: {stderr}"
    );
    assert!(
        pending_path.exists(),
        "the pending state survives a refusal"
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
            &redirect,
            "alice",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
}

/// Percent-encodes a query value the way a browser's address bar shows it.
fn urlencode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

// ── What the interactive sign-in returns ─────────────────────────────────
//
// `xr` hands the flow the token its signal handler cancels and reads a
// cancelled token as Ctrl-C, so the flow ending for any reason of its own
// must leave that token as it was.

/// Runs the interactive flow on a loopback listener handed over to it, with
/// `opener` as the browser (given the authorize URL and the listener's
/// address) and token and user endpoints nothing listens on. Returns what the
/// flow returned and the token it was handed.
async fn run_interactive(
    opener: impl Fn(&str, &str) -> std::io::Result<()> + Send + Sync + 'static,
) -> (xdk::error::Result<String>, CancellationToken) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let target = listener.local_addr().expect("its address").to_string();
    let redirect_uri = format!("http://{target}/callback");
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(
        &format!("{CLOSED}/2/oauth2/token"),
        &format!("{CLOSED}/2/users/me"),
        &redirect_uri,
    );
    let mut auth = test_auth(cfg, &tmp, &redirect_uri);
    let cancel = CancellationToken::new();
    let returned = run_oauth2_flow_on(
        &mut auth,
        &reqwest::Client::new(),
        "testuser",
        listener,
        cancel.clone(),
        move |url| opener(url, &target),
    )
    .await;
    (returned, cancel)
}

/// Follows the redirect to `target` the way a browser does: one GET carrying
/// a code and `state`.
fn follow_redirect(target: &str, state: &str) -> std::io::Result<()> {
    send_callback(target, &format!("code=AUTHCODE&state={state}"))
}

/// Sends the callback request a browser sends for a redirect carrying
/// `query` to `target`.
fn send_callback(target: &str, query: &str) -> std::io::Result<()> {
    let mut stream = StdTcpStream::connect(target)?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    let request =
        format!("GET /callback?{query} HTTP/1.1\r\nHost: {target}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes())?;
    let mut answer = String::new();
    let _ = stream.read_to_string(&mut answer);
    Ok(())
}

/// The state an authorize URL carries, decoded.
fn state_of(auth_url: &str) -> String {
    query_param(
        &Url::parse(auth_url).expect("the authorize URL parses"),
        "state",
    )
}

/// Whether `state` holds only characters a query string carries unencoded,
/// so it reads back the same whatever encoding the redirect uses.
fn needs_no_encoding(state: &str) -> bool {
    !state.is_empty()
        && state
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

#[tokio::test]
async fn a_refused_callback_returns_its_reason_and_leaves_the_callers_token_alone() {
    let (returned, cancel) =
        run_interactive(|_url, target| follow_redirect(target, "NOT-THE-STATE")).await;

    let refused = returned.expect_err("the listener refuses the state");
    assert!(
        refused.to_string().contains("invalid state parameter"),
        "{refused}"
    );
    assert!(
        !cancel.is_cancelled(),
        "the flow cancelled the caller's token, which xr reads as Ctrl-C"
    );
}

#[tokio::test]
async fn a_failed_browser_open_returns_its_reason_and_leaves_the_callers_token_alone() {
    let (returned, cancel) =
        run_interactive(|_url, _target| Err(std::io::Error::other("no browser"))).await;

    let refused = returned.expect_err("the opener fails");
    assert!(
        refused.to_string().contains("browser-open failed"),
        "{refused}"
    );
    assert!(
        !cancel.is_cancelled(),
        "the flow cancelled the caller's token, which xr reads as Ctrl-C"
    );
}

#[tokio::test]
async fn a_failed_exchange_after_the_callback_leaves_the_callers_token_alone() {
    let (returned, cancel) =
        run_interactive(|url, target| follow_redirect(target, &state_of(url))).await;

    let refused = returned.expect_err("the closed token endpoint fails the exchange");
    assert!(
        !refused.to_string().contains("CallbackError"),
        "the listener took the callback: {refused}"
    );
    assert!(
        matches!(refused, xdk::Error::Http { .. }),
        "no answer is a transport failure on the interactive path too: {refused:?}"
    );
    assert!(
        !cancel.is_cancelled(),
        "the flow cancelled the caller's token, which xr reads as Ctrl-C"
    );
}

/// A denied consent: the redirect carries the provider's `error` with the
/// state and no code (RFC 6749 section 4.1.2.1).
const DENIAL: &str = "error=access_denied&error_description=The+user+denied+access";

#[tokio::test]
async fn a_denied_consent_returns_the_providers_error() {
    let (returned, cancel) = run_interactive(|url, target| {
        send_callback(target, &format!("{DENIAL}&state={}", state_of(url)))
    })
    .await;

    let refused = returned.expect_err("the provider refused").to_string();
    assert!(
        refused.contains("access_denied") && refused.contains("The user denied access"),
        "{refused}"
    );
    assert!(
        !cancel.is_cancelled(),
        "the flow cancelled the caller's token, which xr reads as Ctrl-C"
    );
}

#[tokio::test]
async fn step2_reports_a_denied_consent_with_the_providers_error_and_step_one_again() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let pending = seed_pending(&store);
    let overrides = step2_overrides(
        &format!("{CLOSED}/2/oauth2/token"),
        &format!("{CLOSED}/2/users/me"),
    );
    let denied = format!("http://localhost:8080/callback?{DENIAL}&state={PENDING_STATE}");

    let (code, _stdout, stderr) = run_cli(
        &store,
        &overrides,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
            "--auth-url",
            &denied,
        ],
    )
    .await;

    assert_eq!(code, 77, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("one JSON envelope");
    let message = v["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("access_denied") && message.contains("The user denied access"),
        "envelope: {v}"
    );
    assert_eq!(v["reason"], "auth-required", "envelope: {v}");
    assert_eq!(v["next_step"]["action"], "sign-in", "envelope: {v}");
    assert_eq!(
        v["next_step"]["command"], "xr --output json auth oauth2 --no-browser --step 1",
        "a denied consent starts the sign-in over: {v}"
    );
    assert!(pending.exists(), "a refused step 2 keeps the pending state");
}

/// A step 2 refused on its redirect URL, under `--output`: the refusal's name
/// and the URL's query, one case per way the URL can be wrong.
fn refused_redirects() -> [(&'static str, String); 3] {
    [
        (
            "StateMismatch",
            "code=AUTHCODE&state=ANOTHER-NONCE".to_string(),
        ),
        ("MissingState", "code=AUTHCODE".to_string()),
        ("MissingCode", format!("state={PENDING_STATE}")),
    ]
}

async fn run_refused_step2(tmp: &TempDir, flags: &[&str], query: &str) -> (i32, String) {
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let pending = seed_pending(&store);
    let overrides = step2_overrides(
        &format!("{CLOSED}/2/oauth2/token"),
        &format!("{CLOSED}/2/users/me"),
    );
    let redirect = format!("http://localhost:8080/callback?{query}");
    let mut args = vec!["xr"];
    args.extend_from_slice(flags);
    args.extend_from_slice(&[
        "auth",
        "oauth2",
        "--no-browser",
        "--step",
        "2",
        "--auth-url",
        &redirect,
    ]);

    let (code, _stdout, stderr) = run_cli(&store, &overrides, &args).await;
    assert!(pending.exists(), "a refused step 2 keeps the pending state");
    (code, stderr)
}

#[tokio::test]
async fn step2_refusing_its_redirect_url_starts_the_sign_in_over_at_step_one() {
    for (refusal, query) in refused_redirects() {
        let tmp = TempDir::new().unwrap();
        let (code, stderr) = run_refused_step2(&tmp, &["--output", "json"], &query).await;

        assert_eq!(code, 77, "{refusal}: {stderr}");
        let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("one JSON envelope");
        let message = v["message"].as_str().unwrap_or_default();
        assert!(message.contains(refusal), "{refusal}: {v}");
        assert_eq!(v["reason"], "auth-required", "{refusal}: {v}");
        assert_eq!(v["next_step"]["action"], "sign-in", "{refusal}: {v}");
        assert_eq!(
            v["next_step"]["command"], "xr --output json auth oauth2 --no-browser --step 1",
            "{refusal}: the step drops the URL it was refused on: {v}"
        );
        assert!(v["next_step"]["template"].is_null(), "{refusal}: {v}");
    }
}

#[tokio::test]
async fn step2_refusing_its_redirect_url_says_how_to_start_over_in_text() {
    for (refusal, query) in refused_redirects() {
        let tmp = TempDir::new().unwrap();
        let (code, stderr) = run_refused_step2(&tmp, &[], &query).await;

        assert_eq!(code, 77, "{refusal}: {stderr}");
        assert!(stderr.contains(refusal), "{refusal}: {stderr}");
        assert!(
            stderr.contains("Sign in again. Run: xr auth oauth2 --no-browser --step 1"),
            "{refusal}: {stderr}"
        );
    }
}

/// Step 2 under `--output json`, with the token endpoint at a closed port so a
/// run that reached the exchange would answer differently, and its envelope.
async fn refused_step2(
    store: &std::path::Path,
    before: &[&str],
    auth_url: &str,
    after: &[&str],
) -> serde_json::Value {
    let overrides = step2_overrides(
        &format!("{CLOSED}/2/oauth2/token"),
        &format!("{CLOSED}/2/users/me"),
    );
    let mut args = vec!["xr", "--output", "json"];
    args.extend_from_slice(before);
    args.extend_from_slice(&[
        "auth",
        "oauth2",
        "--no-browser",
        "--step",
        "2",
        "--auth-url",
        auth_url,
    ]);
    args.extend_from_slice(after);
    let (code, _stdout, stderr) = run_cli(store, &overrides, &args).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("one JSON envelope");
    assert_eq!(envelope["reason"], "auth-required", "{envelope}");
    envelope
}

fn assert_step(envelope: &serde_json::Value, action: &str, key: &str, invocation: &str) {
    let step = &envelope["next_step"];
    assert_eq!(step["action"], action, "{envelope}");
    assert_eq!(step[key], invocation, "{envelope}");
    let other = if key == "command" {
        "template"
    } else {
        "command"
    };
    assert!(step[other].is_null(), "never both: {envelope}");
}

const STEP_ONE: &str = "xr --output json auth oauth2 --no-browser --step 1";
const ASK_FOR_THE_URL: &str =
    "xr --output json auth oauth2 --no-browser --step 2 --auth-url <redirect-url>";

#[tokio::test]
async fn step2_with_no_saved_flow_starts_over_at_step_one() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);

    let envelope = refused_step2(&store, &[], &redirect_url(), &[]).await;

    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains("PendingStateNotFound"),
        "{envelope}"
    );
    assert_step(&envelope, "sign-in", "command", STEP_ONE);
}

#[tokio::test]
async fn step2_with_an_expired_flow_starts_over_at_step_one() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let path = save_flow(
        &store,
        &xdk::auth::pending::PendingOAuth2State {
            created_at: 0,
            ..saved_flow("default", "test-client-id")
        },
    );

    let envelope = refused_step2(&store, &[], &redirect_url(), &[]).await;

    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains("PendingStateExpired"),
        "{envelope}"
    );
    assert_step(&envelope, "sign-in", "command", STEP_ONE);
    assert!(
        !path.exists(),
        "an expired flow is gone, so nothing resumes"
    );
}

/// The flow is still good, so the step asks for the URL again rather than for
/// another consent, and does not repeat what it was refused.
#[tokio::test]
async fn step2_refusing_a_url_it_cannot_parse_asks_for_the_url_again() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let pending = seed_pending(&store);

    let envelope = refused_step2(&store, &[], "not a url", &[]).await;

    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains("InvalidRedirectURL"),
        "{envelope}"
    );
    assert_step(&envelope, "fix-input", "template", ASK_FOR_THE_URL);
    assert!(pending.exists(), "a refused step 2 keeps the pending state");
}

/// The consent went to the saved app, so finishing there asks nothing of the
/// person who gave it. The URL holds an authorization code, so the step is a
/// template that leaves it out.
#[tokio::test]
async fn step2_for_another_apps_flow_finishes_under_that_app() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store_two_apps(&store);
    let pending = save_flow(&store, &saved_flow("work", "work-client-id"));

    let envelope = refused_step2(&store, &["--app", "default"], &redirect_url(), &[]).await;

    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains("AppMismatch"),
        "{envelope}"
    );
    assert_step(
        &envelope,
        "fix-input",
        "template",
        "xr --app work --output json auth oauth2 --no-browser --step 2 --auth-url <redirect-url>",
    );
    assert!(!envelope.to_string().contains("AUTHCODE"), "{envelope}");
    assert!(pending.exists(), "a refused step 2 keeps the pending state");
}

/// `--app <saved>` cannot help when the saved app is this one, whose client id
/// has moved on, or one the store no longer holds: the same refusal, or an
/// unknown app. Both need a new flow.
#[tokio::test]
async fn step2_for_a_flow_no_stored_app_can_finish_starts_over_at_step_one() {
    for (saved_app, why) in [("default", "this app"), ("gone", "an app no longer stored")] {
        let tmp = TempDir::new().unwrap();
        let store = tmp.path().join(".xurl");
        seed_store_two_apps(&store);
        save_flow(&store, &saved_flow(saved_app, "another-client-id"));

        let envelope = refused_step2(&store, &[], &redirect_url(), &[]).await;

        assert!(
            envelope["message"]
                .as_str()
                .unwrap_or_default()
                .contains("AppMismatch"),
            "{why}: {envelope}"
        );
        assert_step(&envelope, "sign-in", "command", STEP_ONE);
    }
}

/// The caller named these scopes, and the consent in the URL covers others.
/// Finishing it would store a token that is not what the invocation asked
/// for, so the step keeps the scopes and moves them to step 1, where the flag
/// belongs.
#[tokio::test]
async fn step2_naming_other_scopes_starts_over_at_step_one_with_those_scopes() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let pending = save_flow(
        &store,
        &xdk::auth::pending::PendingOAuth2State {
            scopes: vec!["tweet.read".to_string(), "offline.access".to_string()],
            ..saved_flow("default", "test-client-id")
        },
    );

    let envelope = refused_step2(&store, &[], &redirect_url(), &["--scopes", "dm.read"]).await;

    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains("ScopeMismatch"),
        "{envelope}"
    );
    assert_step(
        &envelope,
        "sign-in",
        "command",
        "xr --output json auth oauth2 --no-browser --step 1 --scopes dm.read",
    );
    assert!(pending.exists(), "a refused step 2 keeps the pending state");
}

/// The step works when run: filling in the template finishes the sign-in, on
/// the app whose flow it was.
#[tokio::test]
async fn the_step_for_another_apps_flow_finishes_the_sign_in_when_run() {
    let server = MockServer::start().await;
    token_mock(ok_token_body(), 200).mount(&server).await;
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    std::fs::write(
        &store,
        "apps:\n  default:\n    client_id: 'test-client-id'\n    client_secret: 'test-client-secret'\n    oauth2_tokens: {}\n  work:\n    client_id: 'work-client-id'\n    client_secret: 'work-client-secret'\n    oauth2_tokens: {}\ndefault_app: default\n",
    )
    .expect("write tempdir store");
    let pending = save_flow(&store, &saved_flow("work", "work-client-id"));
    let overrides = xdk::config::EnvOverrides {
        token_url: Some(format!("{}/2/oauth2/token", server.uri())),
        info_url: Some(format!("{}/2/users/me", server.uri())),
        ..xdk::config::EnvOverrides::default()
    };
    let redirect = redirect_url();

    let (code, _stdout, stderr) = run_cli(
        &store,
        &overrides,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
            "--auth-url",
            &redirect,
            "alice",
        ],
    )
    .await;
    assert_eq!(code, 77, "stderr: {stderr}");
    let refused: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("one JSON envelope");
    let template = refused["next_step"]["template"]
        .as_str()
        .unwrap_or_else(|| panic!("a template: {refused}"));

    let filled = template.replace("<redirect-url>", &redirect);
    let args: Vec<&str> = filled.split(' ').collect();
    let (code, _stdout, stderr) = run_cli(&store, &overrides, &args).await;

    assert_eq!(code, 0, "{filled}: {stderr}");
    let saved = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    assert!(
        saved.get_oauth2_token_for_app("work", "alice").is_some(),
        "the login lands on the app whose flow it was"
    );
    assert!(!pending.exists(), "a finished flow is spent");
}

#[tokio::test]
async fn text_output_names_the_same_steps_for_the_other_step2_refusals() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store_two_apps(&store);
    let overrides = step2_overrides(
        &format!("{CLOSED}/2/oauth2/token"),
        &format!("{CLOSED}/2/users/me"),
    );
    let redirect = redirect_url();
    let args = |auth_url| {
        [
            "xr",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
            "--auth-url",
            auth_url,
        ]
    };

    let (_, _, nothing_saved) = run_cli(&store, &overrides, &args(&redirect)).await;
    assert!(
        nothing_saved.contains("Sign in again. Run: xr auth oauth2 --no-browser --step 1"),
        "{nothing_saved}"
    );

    save_flow(&store, &saved_flow("default", "test-client-id"));
    let (_, _, unparsed) = run_cli(&store, &overrides, &args("not a url")).await;
    assert!(
        unparsed.contains(
            "Fill in the redirect URL and run: xr auth oauth2 --no-browser --step 2 --auth-url <redirect-url>"
        ),
        "{unparsed}"
    );

    save_flow(&store, &saved_flow("work", "work-client-id"));
    let (_, _, elsewhere) = run_cli(&store, &overrides, &args(&redirect)).await;
    assert!(
        elsewhere.contains(
            r#"The pending sign-in belongs to app "work". Fill in the redirect URL and run: xr --app work auth oauth2 --no-browser --step 2 --auth-url <redirect-url>"#
        ),
        "{elsewhere}"
    );
}

const OAUTH2_HELP: &str = "xr auth oauth2 --help";
const FAILED_EXCHANGE: &str = "The token endpoint's answer ended the sign-in.";

/// A refused argument is a usage error, which says nothing about credentials.
fn assert_usage_refusal(code: Option<i32>, stderr: &str, says: &str) {
    assert_eq!(code, Some(xdk::error::EXIT_USAGE_ERROR), "{says}: {stderr}");
    let envelope: serde_json::Value = serde_json::from_str(stderr.trim()).expect("one envelope");
    assert_eq!(envelope["reason"], "invalid-args", "{envelope}");
    assert_eq!(
        envelope["exit_code"],
        xdk::error::EXIT_USAGE_ERROR,
        "{envelope}"
    );
    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains(says),
        "{envelope}"
    );
    assert_step(&envelope, "show-help", "command", OAUTH2_HELP);
}

#[tokio::test]
async fn step2_with_no_auth_url_points_at_the_help() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let overrides = step2_overrides(
        &format!("{CLOSED}/2/oauth2/token"),
        &format!("{CLOSED}/2/users/me"),
    );

    let (code, _stdout, stderr) = run_cli(
        &store,
        &overrides,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
        ],
    )
    .await;

    assert_usage_refusal(Some(code), &stderr, "--auth-url is required");
}

/// An empty `--auth-url` is a missing one, refused before the store or the
/// saved flow is read, as empty stdin is.
#[tokio::test]
async fn step2_with_an_empty_auth_url_points_at_the_help() {
    for (saved, url) in [(true, ""), (false, ""), (true, "  ")] {
        let tmp = TempDir::new().unwrap();
        let store = tmp.path().join(".xurl");
        seed_store(&store);
        let pending = saved.then(|| seed_pending(&store));
        let overrides = step2_overrides(
            &format!("{CLOSED}/2/oauth2/token"),
            &format!("{CLOSED}/2/users/me"),
        );

        let (code, _stdout, stderr) = run_cli(
            &store,
            &overrides,
            &[
                "xr",
                "--output",
                "json",
                "auth",
                "oauth2",
                "--no-browser",
                "--step",
                "2",
                "--auth-url",
                url,
            ],
        )
        .await;

        assert_usage_refusal(Some(code), &stderr, "--auth-url is empty");
        if let Some(pending) = pending {
            assert!(pending.exists(), "the saved flow stays: {url:?}");
        }
    }
}

#[tokio::test]
async fn step1_given_an_auth_url_points_at_the_help() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);

    let (code, _stdout, stderr) = run_cli(
        &store,
        &step1_overrides(),
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "1",
            "--auth-url",
            "http://localhost:8080/callback",
        ],
    )
    .await;

    assert_usage_refusal(Some(code), &stderr, "--auth-url is only used with --step 2");
}

/// A dry run answers at exit 0 on every command, and says what the real run
/// would do: refuse the argument, with the keys its error would carry.
#[tokio::test]
async fn a_dry_run_says_the_real_run_would_refuse_the_argument() {
    for (step, url, says) in [
        ("2", None, "--auth-url is required"),
        ("2", Some(""), "--auth-url is empty"),
        ("2", Some("  "), "--auth-url is empty"),
        (
            "1",
            Some("http://localhost:8080/callback"),
            "--auth-url is only used",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        let store = tmp.path().join(".xurl");
        seed_store(&store);
        let mut args = vec![
            "xr",
            "--output",
            "json",
            "--dry-run",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            step,
        ];
        if let Some(url) = url {
            args.extend_from_slice(&["--auth-url", url]);
        }

        let (code, stdout, stderr) = run_cli(&store, &step1_overrides(), &args).await;

        assert_eq!(code, 0, "stderr: {stderr}");
        assert_eq!(stderr, "", "the answer is on stdout");
        let answer: serde_json::Value = serde_json::from_str(stdout.trim()).expect("one document");
        assert_eq!(answer["status"], "dry_run", "{answer}");
        assert_eq!(answer["would_succeed"], false, "{answer}");
        assert_eq!(
            answer["exit_code"],
            xdk::error::EXIT_USAGE_ERROR,
            "the code the real run ends on: {answer}"
        );
        assert_eq!(answer["reason"], "invalid-args", "{answer}");
        assert!(
            answer["message"]
                .as_str()
                .unwrap_or_default()
                .contains(says),
            "{answer}"
        );
        assert_step(&answer, "show-help", "command", OAUTH2_HELP);
    }
}

#[tokio::test]
async fn a_dry_run_of_a_step_with_its_arguments_right_would_succeed() {
    let redirect = redirect_url();
    for args in [
        &[][..],
        &["--step", "1"][..],
        &["--step", "2", "--auth-url", &redirect],
    ] {
        let tmp = TempDir::new().unwrap();
        let store = tmp.path().join(".xurl");
        seed_store(&store);
        if args.contains(&"2") {
            seed_pending(&store);
        }
        let mut argv = vec![
            "xr",
            "--output",
            "json",
            "--dry-run",
            "auth",
            "oauth2",
            "--no-browser",
        ];
        argv.extend_from_slice(args);

        let (code, stdout, stderr) = run_cli(&store, &step1_overrides(), &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let answer: serde_json::Value = serde_json::from_str(stdout.trim()).expect("one document");
        assert_eq!(answer["would_succeed"], true, "{args:?}: {answer}");
    }
}

/// `--auth-url -` reads the process's stdin, so this spawns the built binary
/// to give it one.
#[test]
fn step2_reading_no_url_from_stdin_points_at_the_help() {
    for (stdin, says) in [
        (&b""[..], "No redirect URL provided on stdin"),
        (&b"  \n"[..], "No redirect URL provided on stdin"),
        (&[0xff, b'\n'][..], "Failed to read redirect URL from stdin"),
    ] {
        let tmp = TempDir::new().unwrap();
        let store = tmp.path().join(".xurl");
        seed_store(&store);

        let output = common::xr_with_store(&store)
            .env("API_BASE_URL", "http://127.0.0.1:1")
            .args([
                "--output",
                "json",
                "auth",
                "oauth2",
                "--no-browser",
                "--step",
                "2",
                "--auth-url",
                "-",
            ])
            .write_stdin(stdin)
            .output()
            .expect("xr runs");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_usage_refusal(output.status.code(), &stderr, says);
    }
}

/// A dry run reads piped stdin as the real run does, so it predicts the same
/// refusal, and well-formed stdin still answers that the run would succeed.
#[test]
fn a_dry_run_reads_piped_stdin_as_the_real_run_does() {
    for (stdin, refusal) in [
        (&b""[..], Some("No redirect URL provided on stdin")),
        (
            &[0xff, b'\n'][..],
            Some("Failed to read redirect URL from stdin"),
        ),
        (
            &b"http://localhost:8080/callback?code=AUTHCODE&state=TEST-STATE-NONCE\n"[..],
            None,
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        let store = tmp.path().join(".xurl");
        seed_store(&store);
        seed_pending(&store);

        let output = common::xr_with_store(&store)
            .env("API_BASE_URL", "http://127.0.0.1:1")
            .args([
                "--output",
                "json",
                "--dry-run",
                "auth",
                "oauth2",
                "--no-browser",
                "--step",
                "2",
                "--auth-url",
                "-",
            ])
            .write_stdin(stdin)
            .output()
            .expect("xr runs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(0), "{refusal:?}: {stdout}");
        let answer: serde_json::Value = serde_json::from_str(stdout.trim()).expect("one document");
        assert_eq!(answer["status"], "dry_run", "{answer}");
        match refusal {
            Some(says) => {
                assert_eq!(answer["would_succeed"], false, "{answer}");
                assert_eq!(
                    answer["exit_code"],
                    xdk::error::EXIT_USAGE_ERROR,
                    "{answer}"
                );
                assert_eq!(answer["reason"], "invalid-args", "{answer}");
                assert!(
                    answer["message"]
                        .as_str()
                        .unwrap_or_default()
                        .contains(says),
                    "{answer}"
                );
                assert_step(&answer, "show-help", "command", OAUTH2_HELP);
            }
            None => assert_eq!(answer["would_succeed"], true, "{answer}"),
        }
    }
}

/// The real run reads stdin only once the client id is there, and checks what
/// it read against the saved flow. A dry run reads it at the same point and
/// refuses for the same reason.
#[test]
fn a_dry_run_reads_piped_stdin_where_the_real_run_does_and_refuses_as_it_does() {
    let another_state = b"http://localhost:8080/callback?code=AUTHCODE&state=ANOTHER-NONCE\n";
    let mut faults = Vec::new();
    for (name, client_id, saved, stdin) in [
        ("no client id", false, true, &b""[..]),
        ("another state", true, true, &another_state[..]),
        (
            "nothing on stdin, ahead of a missing flow",
            true,
            false,
            &b""[..],
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        let store = tmp.path().join(".xurl");
        if client_id {
            seed_store(&store);
        } else {
            seed_store_without_a_client_id(&store);
        }
        if saved {
            if client_id {
                seed_pending(&store);
            } else {
                save_flow(&store, &saved_flow("default", "test-client-id"));
            }
        }
        let run = |dry: bool| {
            let mut command = common::xr_with_store(&store);
            command
                .env("API_BASE_URL", CLOSED)
                .env("TOKEN_URL", format!("{CLOSED}/2/oauth2/token"))
                .env("INFO_URL", format!("{CLOSED}/2/users/me"))
                .args(["--output", "json"]);
            if dry {
                command.arg("--dry-run");
            }
            command
                .args([
                    "auth",
                    "oauth2",
                    "--no-browser",
                    "--step",
                    "2",
                    "--auth-url",
                    "-",
                ])
                .write_stdin(stdin)
                .output()
                .expect("xr runs")
        };

        let dry = run(true);
        let real = run(false);

        let real_code = real.status.code().unwrap_or_default();
        let dry_document = serde_json::from_slice(&dry.stdout).unwrap_or_default();
        let real_document = serde_json::from_slice(&real.stderr).unwrap_or_default();
        if dry.status.code() != Some(0) || real_code == 0 {
            faults.push(format!(
                "{name}: the dry run exits {:?}, the real run {real_code}",
                dry.status.code()
            ));
        }
        faults.extend(dry_run_faults(
            name,
            &dry_document,
            &real_document,
            real_code,
        ));
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}

/// `XURL_DRY_RUN` selects a dry run with no flag in argv, and a step built
/// from argv has to name the flag all the same: followed without the variable,
/// a step without it is a real run.
#[test]
fn a_dry_run_the_environment_selects_names_steps_that_keep_it() {
    let redirect = redirect_url();
    let mut faults = Vec::new();
    for (name, saved, url, action) in [
        ("no saved flow", false, redirect.as_str(), "sign-in"),
        (
            "redirect URL that does not parse",
            true,
            "not a url",
            "fix-input",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        let store = tmp.path().join(".xurl");
        seed_store(&store);
        if saved {
            seed_pending(&store);
        }
        let output = common::xr_with_store(&store)
            .env("API_BASE_URL", CLOSED)
            .env("XURL_DRY_RUN", "1")
            .args([
                "--output",
                "json",
                "auth",
                "oauth2",
                "--no-browser",
                "--step",
                "2",
                "--auth-url",
                url,
            ])
            .output()
            .expect("xr runs");
        let answer: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
        if output.status.code() != Some(0) || answer["status"] != "dry_run" {
            faults.push(format!("{name}: not a dry run: {output:?}"));
        } else if answer["next_step"]["action"] != action {
            faults.push(format!("{name}: the step is not {action}: {answer}"));
        }
        if let Some(step) = rerun_that_signs_in_for_real(&answer) {
            faults.push(format!("{name}: the step drops --dry-run: {step}"));
        }
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}

#[tokio::test]
async fn a_dry_run_in_text_says_the_real_run_would_refuse_the_argument() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);

    let (code, stdout, stderr) = run_cli(
        &store,
        &step1_overrides(),
        &[
            "xr",
            "--dry-run",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("The real run would fail with exit code 2: --auth-url is required"),
        "{stdout}"
    );
}

/// In text the step is the form a person runs, and a dry run's carries
/// `--dry-run` so that following it changes nothing.
#[tokio::test]
async fn a_dry_run_in_text_names_the_sign_in_step_the_real_run_names_under_dry_run() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store_without_a_client_id(&store);
    let overrides = closed_overrides(false);

    let (dry_code, dry_stdout, dry_stderr) =
        run_cli(&store, &overrides, &["xr", "--dry-run", "auth", "oauth2"]).await;
    let (code, _, stderr) = run_cli(&store, &overrides, &["xr", "auth", "oauth2"]).await;

    assert_eq!(dry_code, 0, "stderr: {dry_stderr}");
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(
        stderr.contains("Run: xr auth oauth2 --app work"),
        "{stderr}"
    );
    assert!(
        dry_stdout.contains("Run: xr --dry-run auth oauth2 --app work"),
        "{dry_stdout}"
    );
}

/// The store file stops parsing while the person consents: the code the
/// callback delivers is not exchanged, since the store could not keep the
/// tokens it buys.
#[tokio::test]
async fn the_interactive_flow_does_not_spend_the_code_when_the_store_cannot_keep_its_tokens() {
    let server = MockServer::start().await;
    let token_url = format!("{}/2/oauth2/token", server.uri());
    let info_url = format!("{}/2/users/me", server.uri());
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "ACCESS-TOKEN",
            "refresh_token": "REFRESH-TOKEN",
            "expires_in": 7200
        })))
        .mount(&server)
        .await;

    let mut ended = None;
    for _ in 0..20 {
        let port = band_port();
        let redirect_uri = format!("http://127.0.0.1:{port}/callback");
        let tmp = TempDir::new().unwrap();
        let store_path = tmp.path().join(".xurl");
        let mut auth = test_auth(
            test_config(&token_url, &info_url, &redirect_uri),
            &tmp,
            &redirect_uri,
        );
        let breaking_opener = move |url: &str| -> std::io::Result<()> {
            std::fs::write(&store_path, "apps: [half an edit")?;
            let state = Url::parse(url)
                .expect("the opener receives a valid auth URL")
                .query_pairs()
                .find(|(name, _)| name == "state")
                .map(|(_, value)| value.into_owned())
                .expect("the auth URL carries a state");
            let mut stream = StdTcpStream::connect(("127.0.0.1", port))?;
            stream.set_read_timeout(Some(Duration::from_secs(2)))?;
            write!(
                stream,
                "GET /callback?code=AUTHCODE&state={state} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
            )?;
            let _ = stream.read_to_string(&mut String::new());
            Ok(())
        };
        match run_oauth2_flow(
            &mut auth,
            &reqwest::Client::new(),
            "testuser",
            CancellationToken::new(),
            breaking_opener,
        )
        .await
        {
            Err(error) if failed_to_bind(&error, port) => {}
            outcome => {
                ended = Some(outcome);
                break;
            }
        }
    }
    let outcome = ended.expect("a port the flow could bind in 20 picks");

    let spent = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|request| request.url.path() == "/2/oauth2/token")
        .count();
    assert_eq!(
        spent, 0,
        "the code was exchanged though its tokens could not be stored: {outcome:?}"
    );
    assert!(
        matches!(outcome, Err(xdk::Error::TokenStore { .. })),
        "{outcome:?}"
    );
}

// ── A dry run reaches the verdict the real run reaches ───────────────────

/// Two apps, the target `default` with no client id and `work` with one.
fn seed_store_without_a_client_id(store_path: &std::path::Path) {
    std::fs::write(
        store_path,
        "apps:\n  default:\n    client_id: ''\n    client_secret: ''\n    oauth2_tokens: {}\n  work:\n    client_id: 'work-client-id'\n    client_secret: 'work-client-secret'\n    oauth2_tokens: {}\ndefault_app: default\n",
    )
    .expect("write tempdir store");
}

/// One app, `blank`, with no client id, and no other app to sign in under.
fn seed_store_with_only_a_blank_app(store_path: &std::path::Path) {
    std::fs::write(
        store_path,
        "apps:\n  blank:\n    client_id: ''\n    client_secret: ''\n    oauth2_tokens: {}\ndefault_app: blank\n",
    )
    .expect("write tempdir store");
}

/// The environment of a run whose endpoints are all closed ports, so a run
/// that reached one would fail differently. Without `client_id` the app the
/// store resolves to is the only source of one.
fn closed_overrides(client_id: bool) -> xdk::config::EnvOverrides {
    xdk::config::EnvOverrides {
        client_id: client_id.then(|| "test-client-id".to_string()),
        client_secret: client_id.then(|| "test-client-secret".to_string()),
        token_url: Some(format!("{CLOSED}/2/oauth2/token")),
        info_url: Some(format!("{CLOSED}/2/users/me")),
        api_base_url: Some(CLOSED.to_string()),
        ..xdk::config::EnvOverrides::default()
    }
}

/// A state to run `xr auth oauth2` in, and a word of what its real run says.
struct Verdict {
    name: &'static str,
    /// Whether `CLIENT_ID` is set in the environment.
    client_id: bool,
    seed: Box<dyn Fn(&std::path::Path)>,
    /// Global flags, given before the command.
    before: Vec<&'static str>,
    args: Vec<String>,
    says: &'static str,
}

fn verdict(
    name: &'static str,
    says: &'static str,
    client_id: bool,
    seed: impl Fn(&std::path::Path) + 'static,
    args: &[&str],
) -> Verdict {
    Verdict {
        name,
        client_id,
        seed: Box::new(seed),
        before: Vec::new(),
        args: args.iter().map(ToString::to_string).collect(),
        says,
    }
}

fn flow_at(redirect: &str) -> Vec<&str> {
    vec!["--no-browser", "--step", "2", "--auth-url", redirect]
}

/// Every way the local state, the environment, and the arguments make
/// `xr auth oauth2` fail before it sends anything, in the order the real run
/// checks them.
fn verdicts() -> Vec<Verdict> {
    let redirect = redirect_url();
    let at_redirect = |query: &str| format!("http://localhost:8080/callback?{query}");
    let step1 = ["--no-browser", "--step", "1"];
    let bad_scope = ["--scopes", "tweet.reed"];
    let mut rows = vec![
        verdict(
            "no client id anywhere",
            "no app carries client credentials",
            false,
            |_| {},
            &step1,
        ),
        verdict(
            "no client id, another app has one",
            "app \"default\" has no client credentials; app \"work\" does",
            false,
            seed_store_without_a_client_id,
            &step1,
        ),
        verdict(
            "no client id, ahead of the saved flow",
            "has no client credentials",
            false,
            |store| {
                seed_store_without_a_client_id(store);
                save_flow(store, &saved_flow("default", "test-client-id"));
            },
            &flow_at(&redirect),
        ),
        verdict(
            "unknown scope",
            "tweet.reed is not an OAuth2 scope",
            true,
            seed_store,
            &[&step1[..], &bad_scope[..]].concat(),
        ),
        verdict(
            "unknown scope, ahead of the client id",
            "tweet.reed is not an OAuth2 scope",
            false,
            |_| {},
            &[&step1[..], &bad_scope[..]].concat(),
        ),
        verdict(
            "unknown scope, ahead of a missing --auth-url",
            "tweet.reed is not an OAuth2 scope",
            true,
            seed_store,
            &["--no-browser", "--step", "2", "--scopes", "tweet.reed"],
        ),
        verdict(
            "missing --auth-url, ahead of the client id",
            "--auth-url is required",
            false,
            |_| {},
            &["--no-browser", "--step", "2"],
        ),
        verdict(
            "no saved flow",
            "PendingStateNotFound",
            true,
            seed_store,
            &flow_at(&redirect),
        ),
        verdict(
            "expired flow",
            "PendingStateExpired",
            true,
            |store| {
                seed_store(store);
                save_flow(
                    store,
                    &xdk::auth::pending::PendingOAuth2State {
                        created_at: 0,
                        ..saved_flow("default", "test-client-id")
                    },
                );
            },
            &flow_at(&redirect),
        ),
        verdict(
            "flow that does not parse",
            "PendingStateInvalid",
            true,
            |store| {
                seed_store(store);
                let path = xdk::auth::pending::pending_path_for_store(store);
                std::fs::write(&path, "state: [unclosed\n").unwrap();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                        .unwrap();
                }
            },
            &flow_at(&redirect),
        ),
        verdict(
            "flow saved for another client id",
            "AppMismatch",
            true,
            |store| {
                seed_store(store);
                save_flow(store, &saved_flow("default", "another-client-id"));
            },
            &flow_at(&redirect),
        ),
        verdict(
            "flow saved with other scopes",
            "ScopeMismatch",
            true,
            |store| {
                seed_store(store);
                save_flow(
                    store,
                    &xdk::auth::pending::PendingOAuth2State {
                        scopes: vec!["tweet.read".to_string(), "offline.access".to_string()],
                        ..saved_flow("default", "test-client-id")
                    },
                );
            },
            &[&flow_at(&redirect)[..], &["--scopes", "dm.read"][..]].concat(),
        ),
        verdict(
            "redirect URL that does not parse",
            "InvalidRedirectURL",
            true,
            |store| {
                seed_store(store);
                seed_pending(store);
            },
            &flow_at("not a url"),
        ),
        verdict(
            "redirect URL with another state",
            "StateMismatch",
            true,
            |store| {
                seed_store(store);
                seed_pending(store);
            },
            &flow_at(&at_redirect("code=AUTHCODE&state=ANOTHER-NONCE")),
        ),
        verdict(
            "redirect URL with no state",
            "MissingState",
            true,
            |store| {
                seed_store(store);
                seed_pending(store);
            },
            &flow_at(&at_redirect("code=AUTHCODE")),
        ),
        verdict(
            "redirect URL with no code",
            "MissingCode",
            true,
            |store| {
                seed_store(store);
                seed_pending(store);
            },
            &flow_at(&at_redirect(&format!("state={PENDING_STATE}"))),
        ),
        verdict(
            "redirect URL of a denied consent",
            "access_denied",
            true,
            |store| {
                seed_store(store);
                seed_pending(store);
            },
            &flow_at(&at_redirect(&format!("{DENIAL}&state={PENDING_STATE}"))),
        ),
        verdict(
            "store that does not parse",
            "could not be loaded",
            true,
            |store| {
                std::fs::write(store, "apps: [half an edit").unwrap();
                seed_pending(store);
            },
            &flow_at(&redirect),
        ),
    ];
    rows.push(Verdict {
        before: vec!["--app", "default"],
        ..verdict(
            "flow saved for another app",
            "AppMismatch",
            true,
            |store| {
                seed_store_two_apps(store);
                save_flow(store, &saved_flow("work", "work-client-id"));
            },
            &flow_at(&redirect),
        )
    });
    rows.push(Verdict {
        before: vec!["--app", "blank"],
        ..verdict(
            "app named, with no client id and no other app to sign in under",
            "app \"blank\" has no client credentials.",
            false,
            seed_store_with_only_a_blank_app,
            &step1,
        )
    });
    rows.push(verdict(
        "flow saved for another client id, ahead of a redirect URL that does not parse",
        "AppMismatch",
        true,
        |store| {
            seed_store(store);
            save_flow(store, &saved_flow("default", "another-client-id"));
        },
        &flow_at("not a url"),
    ));
    #[cfg(unix)]
    {
        rows.push(verdict(
            "flow with the wrong mode",
            "PendingStatePermissions",
            true,
            |store| {
                use std::os::unix::fs::PermissionsExt;
                seed_store(store);
                let path = seed_pending(store);
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
            },
            &flow_at(&redirect),
        ));
        rows.push(verdict(
            "flow that is a symlink",
            "symlink",
            true,
            |store| {
                seed_store(store);
                let elsewhere = store.with_file_name("elsewhere");
                std::fs::write(&elsewhere, "x").unwrap();
                std::os::unix::fs::symlink(
                    &elsewhere,
                    xdk::auth::pending::pending_path_for_store(store),
                )
                .unwrap();
            },
            &flow_at(&redirect),
        ));
    }
    rows
}

/// The store file and the flow beside it, as bytes, `None` for a path that
/// holds nothing. A symlink counts as present however it points.
fn on_disk(store: &std::path::Path) -> Vec<Option<Vec<u8>>> {
    let pending = xdk::auth::pending::pending_path_for_store(store);
    [store, pending.as_path()]
        .iter()
        .map(|path| {
            std::fs::symlink_metadata(path)
                .ok()
                .map(|_| std::fs::read(path).unwrap_or_default())
        })
        .collect()
}

/// The keys of a dry-run answer that frame the command and are no part of the
/// error the real run writes.
const DRY_RUN_FRAME: [&str; 6] = [
    "status",
    "would_succeed",
    "command",
    "no_browser",
    "step",
    "username",
];

/// A step that reruns the invocation keeps `--dry-run` in a dry run, and the
/// message that names the step with it, so the two answers agree once that
/// word is dropped.
fn without_dry_run(keys: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(serde_json::Value::String(message)) = keys.get_mut("message") {
        *message = message.replace(" --dry-run", "");
    }
    let Some(step) = keys.get_mut("next_step").and_then(|s| s.as_object_mut()) else {
        return;
    };
    for key in ["command", "template"] {
        if let Some(serde_json::Value::String(text)) = step.get_mut(key) {
            *text = text.replace(" --dry-run", "");
        }
    }
}

/// The step of a dry run's answer that reruns an `xr auth oauth2` invocation
/// without `--dry-run`: following it would sign in for real.
fn rerun_that_signs_in_for_real(dry: &serde_json::Value) -> Option<String> {
    let step = dry.get("next_step")?.as_object()?;
    if matches!(
        step.get("action").and_then(|action| action.as_str()),
        Some("show-help" | "register-app")
    ) {
        return None;
    }
    ["command", "template"].iter().find_map(|key| {
        let text = step.get(*key)?.as_str()?;
        (text.starts_with("xr ") && text.contains(" auth oauth2") && !text.contains(" --dry-run"))
            .then(|| text.to_string())
    })
}

/// What differs between the dry run's answer and the real run's: the dry run
/// refuses, ends where the real run ends, and carries the keys the real
/// run's error carries.
fn dry_run_faults(
    name: &str,
    dry: &serde_json::Value,
    real: &serde_json::Value,
    real_code: i32,
) -> Vec<String> {
    if dry["status"] != "dry_run" || dry["would_succeed"] != false {
        return vec![format!("{name}: the dry run does not refuse: {dry}")];
    }
    let mut faults = Vec::new();
    if dry.to_string().contains("AUTHCODE") {
        faults.push(format!("{name}: the dry run echoes the code: {dry}"));
    }
    if let Some(step) = rerun_that_signs_in_for_real(dry) {
        faults.push(format!("{name}: the step drops --dry-run: {step}"));
    }
    if dry["exit_code"] != real_code || real["exit_code"] != real_code {
        faults.push(format!(
            "{name}: the real run exits {real_code}, the dry run says {}",
            dry["exit_code"]
        ));
    }
    let mut from_dry = dry.as_object().expect("a JSON object").clone();
    for key in DRY_RUN_FRAME {
        from_dry.remove(key);
    }
    let mut from_real = real.as_object().expect("a JSON object").clone();
    from_real.remove("status");
    without_dry_run(&mut from_dry);
    without_dry_run(&mut from_real);
    if from_dry != from_real {
        faults.push(format!(
            "{name}: the keys differ\n  dry:  {dry}\n  real: {real}"
        ));
    }
    faults
}

fn json_document(name: &str, text: &str) -> serde_json::Value {
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("{name}: {e}: {text}"))
}

/// Runs `row` as a dry run and then as the real run on the same state, and
/// lists what the two disagree on.
async fn disagreements(row: Verdict) -> Vec<String> {
    let Verdict {
        name,
        client_id,
        seed,
        before,
        args,
        says,
    } = row;
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed(&store);
    let overrides = closed_overrides(client_id);
    let command = |dry: bool| {
        let mut argv = vec!["xr"];
        argv.extend_from_slice(&before);
        argv.extend_from_slice(&["--output", "json"]);
        if dry {
            argv.push("--dry-run");
        }
        argv.extend_from_slice(&["auth", "oauth2"]);
        argv.extend(args.iter().map(String::as_str));
        argv
    };
    let kept = on_disk(&store);

    let (code, stdout, stderr) = run_cli(&store, &overrides, &command(true)).await;
    let mut faults = Vec::new();
    if code != 0 || !stderr.is_empty() {
        faults.push(format!(
            "{name}: the dry run exits {code} with stderr: {stderr}"
        ));
    }
    if on_disk(&store) != kept {
        faults.push(format!("{name}: the dry run changed a file"));
    }
    let dry = serde_json::from_str::<serde_json::Value>(stdout.trim());

    let (real_code, real_stdout, real_stderr) = run_cli(&store, &overrides, &command(false)).await;
    if real_code == 0 {
        faults.push(format!("{name}: the real run succeeded: {real_stdout}"));
        return faults;
    }
    let real = json_document(name, &real_stderr);
    if !real["message"].as_str().unwrap_or_default().contains(says) {
        faults.push(format!("{name}: the real run says something else: {real}"));
    }
    match dry {
        Ok(dry) => faults.extend(dry_run_faults(name, &dry, &real, real_code)),
        Err(e) => faults.push(format!("{name}: the dry run answers no document ({e})")),
    }
    faults
}

#[tokio::test]
async fn a_dry_run_of_a_sign_in_the_real_run_refuses_says_so_with_the_same_keys() {
    let mut faults = Vec::new();
    for row in verdicts() {
        faults.extend(disagreements(row).await);
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}

/// A dry run of a step 2 that would finish exchanges nothing and removes
/// nothing; the real run on the same state exchanges the code once.
#[tokio::test]
async fn a_dry_run_of_a_step_two_that_would_finish_sends_nothing_and_leaves_the_flow() {
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
    let redirect = redirect_url();
    let argv = |dry: bool| {
        let mut argv = vec!["xr", "--output", "json"];
        if dry {
            argv.push("--dry-run");
        }
        argv.extend_from_slice(&[
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
            "--auth-url",
            &redirect,
            "alice",
        ]);
        argv
    };
    let kept = on_disk(&store);

    let (code, stdout, stderr) = run_cli(&store, &overrides, &argv(true)).await;

    assert_eq!(code, 0, "{stderr}");
    let dry = json_document("dry run", &stdout);
    assert_eq!(dry["status"], "dry_run", "{dry}");
    assert_eq!(dry["would_succeed"], true, "{dry}");
    assert_eq!(dry["exit_code"], 0, "{dry}");
    assert!(
        server
            .received_requests()
            .await
            .unwrap_or_default()
            .is_empty(),
        "a dry run sent a request"
    );
    assert_eq!(on_disk(&store), kept, "the dry run changed a file");

    let (code, stdout, stderr) = run_cli(&store, &overrides, &argv(false)).await;

    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("OAuth2 authentication successful"),
        "{stdout}"
    );
    assert!(!pending.exists(), "the real run finishes the flow");
}

/// The message names the file and what is wrong with it, which is all an
/// `xr` invocation could add, so the step carries no invocation.
#[cfg(unix)]
#[tokio::test]
async fn step2_refusing_the_saved_file_names_it_and_asks_for_it_to_be_fixed() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let path = seed_pending(&store);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    let envelope = refused_step2(&store, &[], &redirect_url(), &[]).await;

    let message = envelope["message"].as_str().unwrap_or_default();
    assert!(message.contains("PendingStatePermissions"), "{envelope}");
    assert!(message.contains(&*path.to_string_lossy()), "{envelope}");
    assert!(message.contains("0644"), "{envelope}");
    assert_eq!(
        envelope["next_step"],
        serde_json::json!({"action": "fix-input"}),
        "{envelope}"
    );

    std::fs::remove_file(&path).unwrap();
    let elsewhere = tmp.path().join("elsewhere");
    std::fs::write(&elsewhere, "x").unwrap();
    std::os::unix::fs::symlink(&elsewhere, &path).unwrap();

    let envelope = refused_step2(&store, &[], &redirect_url(), &[]).await;

    let message = envelope["message"].as_str().unwrap_or_default();
    assert!(message.contains("symlink"), "{envelope}");
    assert!(message.contains(&*path.to_string_lossy()), "{envelope}");
    assert_eq!(
        envelope["next_step"],
        serde_json::json!({"action": "fix-input"}),
        "{envelope}"
    );
}

/// The step works when run: step 1 replaces the file it could not read.
#[tokio::test]
async fn step2_with_a_saved_file_that_does_not_parse_starts_over_and_step_one_replaces_it() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let path = seed_pending(&store);
    std::fs::write(&path, "not: [yaml").unwrap();

    let envelope = refused_step2(&store, &[], &redirect_url(), &[]).await;

    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains("PendingStateInvalid"),
        "{envelope}"
    );
    assert_step(&envelope, "sign-in", "command", STEP_ONE);

    let url = step1_url(&store, &[]).await;
    let replaced = xdk::auth::pending::load(&path).expect("step 1 replaced the file");
    assert_eq!(replaced.state, query_param(&url, "state"));
}

const MARKER_VERIFIER: &str = "MARKER-VERIFIER-NOT-A-SECRET";
const MARKER_STATE: &str = "MARKER-STATE-NOT-A-SECRET";

/// A saved flow that does not parse is named by its path and the position the
/// parser stopped at, and its content is never printed: it holds the PKCE
/// verifier and the state nonce.
#[tokio::test]
async fn a_saved_flow_that_does_not_parse_is_named_and_never_quoted() {
    let broken = [
        (
            "an unclosed sequence below the secrets",
            format!(
                "code_verifier: {MARKER_VERIFIER}\nstate: {MARKER_STATE}\nclient_id: [unclosed\n"
            ),
        ),
        (
            "an unterminated quote on the secret's own line",
            format!("code_verifier: \"{MARKER_VERIFIER}\nstate: {MARKER_STATE}\n"),
        ),
        (
            "a secret where a number belongs",
            format!(
                "code_verifier: {MARKER_VERIFIER}\nstate: {MARKER_STATE}\nclient_id: x\napp_name: x\ncreated_at: {MARKER_VERIFIER}\n"
            ),
        ),
    ];
    let redirect = redirect_url();
    let mut faults = Vec::new();
    for (shape, body) in &broken {
        for dry in [false, true] {
            for output in ["json", "text"] {
                let tmp = TempDir::new().unwrap();
                let store = tmp.path().join(".xurl");
                seed_store(&store);
                let path = seed_pending(&store);
                std::fs::write(&path, body).unwrap();
                let mut args = vec!["xr", "--output", output];
                if dry {
                    args.push("--dry-run");
                }
                args.extend_from_slice(&["auth", "oauth2"]);
                args.extend(flow_at(&redirect));

                let (_code, stdout, stderr) = run_cli(&store, &closed_overrides(true), &args).await;

                let printed = format!("{stdout}{stderr}");
                let run = format!("{shape}, dry={dry}, output={output}");
                for marker in [MARKER_VERIFIER, MARKER_STATE] {
                    if printed.contains(marker) {
                        faults.push(format!("{run}: {marker} printed:\n{printed}"));
                    }
                }
                for (needle, what) in [
                    ("PendingStateInvalid", "the reason"),
                    (&*path.to_string_lossy(), "the file"),
                    ("line ", "the line"),
                    ("column ", "the column"),
                ] {
                    if !printed.contains(needle) {
                        faults.push(format!("{run}: {what} is not named:\n{printed}"));
                    }
                }
            }
        }
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}

/// Step 2 against the token endpoint at `base`: its exit code and envelope.
/// The pending state survives each refusal, so the code is the only thing
/// spent.
async fn step2_at(base: &str) -> (i32, serde_json::Value) {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let pending = seed_pending(&store);
    let overrides = step2_overrides(
        &format!("{base}/2/oauth2/token"),
        &format!("{base}/2/users/me"),
    );

    let (code, _stdout, stderr) = run_cli(
        &store,
        &overrides,
        &[
            "xr",
            "--output",
            "json",
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

    assert!(pending.exists(), "a refused step 2 keeps the pending state");
    let envelope = serde_json::from_str(stderr.trim()).expect("one envelope");
    (code, envelope)
}

async fn step2_answered_with(response: ResponseTemplate) -> (i32, serde_json::Value) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(response)
        .expect(1)
        .mount(&server)
        .await;
    step2_at(&server.uri()).await
}

/// What X refused is the credential in hand, so the failure is `auth-required`.
fn assert_exchange_refused(code: i32, envelope: &serde_json::Value) {
    assert_eq!(code, xdk::error::EXIT_AUTH_REQUIRED, "{envelope}");
    assert_eq!(envelope["reason"], "auth-required", "{envelope}");
    assert_eq!(
        envelope["exit_code"],
        xdk::error::EXIT_AUTH_REQUIRED,
        "{envelope}"
    );
    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains("TokenExchangeError"),
        "{envelope}"
    );
}

fn oauth_error(status: u16, error: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(serde_json::json!({
        "error": error,
        "error_description": "as the token endpoint put it",
    }))
}

/// X answered, and the single-use code is spent or was never good, so sending
/// it again fails the same way. Only a new consent gets a new code.
#[tokio::test]
async fn step2_whose_code_x_would_not_exchange_starts_over_at_step_one() {
    for (why, response) in [
        ("invalid_grant", oauth_error(400, "invalid_grant")),
        (
            "a 400 with no JSON",
            ResponseTemplate::new(400).set_body_string("no"),
        ),
        ("invalid_grant on a 403", oauth_error(403, "invalid_grant")),
        ("invalid_grant on a 404", oauth_error(404, "invalid_grant")),
        (
            "a 200 with no token",
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"token_type": "bearer"})),
        ),
        (
            "a 200 with no JSON",
            ResponseTemplate::new(200).set_body_string("<html>"),
        ),
    ] {
        let (code, envelope) = step2_answered_with(response).await;
        assert_exchange_refused(code, &envelope);
        assert_step(
            &envelope,
            "sign-in",
            "command",
            "xr --output json auth oauth2 --no-browser --step 1 alice",
        );
        assert!(
            !envelope.to_string().contains("AUTHCODE"),
            "{why}: {envelope}"
        );
    }
}

/// X did not process the request, so the code is still good and a new consent
/// would ask a person for nothing. Nothing about the credentials failed, so
/// these report what any other request to X would. A 5xx whose body is not JSON
/// keeps its status: a proxy's error page is the usual one.
#[tokio::test]
async fn step2_x_answered_with_a_server_error_is_a_server_error_and_is_sent_again() {
    for (why, response) in [
        ("a 503", oauth_error(503, "server_error")),
        (
            "a 502 with no JSON",
            ResponseTemplate::new(502).set_body_string("<html>Bad Gateway</html>"),
        ),
    ] {
        let (code, envelope) = step2_answered_with(response).await;
        assert_eq!(code, xdk::error::EXIT_GENERAL_ERROR, "{why}: {envelope}");
        assert_eq!(envelope["reason"], "server-error", "{why}: {envelope}");
        assert_eq!(
            envelope["exit_code"],
            xdk::error::EXIT_GENERAL_ERROR,
            "{why}: {envelope}"
        );
        assert_eq!(
            envelope["next_step"]["action"], "retry",
            "{why}: {envelope}"
        );
        assert!(
            envelope["next_step"]["command"].is_null(),
            "{why}: {envelope}"
        );
    }
}

/// RFC 6749 section 5.2 refuses with a 400 or a 401, or with a JSON body that
/// holds a string `error`. Any other answer came from something that did not
/// process the exchange, such as a proxy or a token URL with the wrong path, so
/// the code is still good and the saved flow stays for the same step 2. A 403
/// points at the token URL and the proxy in front of it, and every such answer
/// is warned of by name, with its status.
#[tokio::test]
async fn step2_answered_with_no_oauth_refusal_reports_its_status() {
    for NoRefusal {
        why,
        response,
        status,
        reason,
        exit_code,
        action,
        docs,
    } in no_refusal_answers()
    {
        let (code, envelope) = step2_answered_with(response).await;
        assert_eq!(code, exit_code, "{why}: {envelope}");
        assert_eq!(envelope["reason"], reason, "{why}: {envelope}");
        assert_eq!(envelope["exit_code"], exit_code, "{why}: {envelope}");
        assert_eq!(envelope["next_step"]["action"], action, "{why}: {envelope}");
        assert_eq!(envelope["next_step"]["docs"], docs, "{why}: {envelope}");
        assert!(
            envelope["next_step"]["command"].is_null(),
            "{why}: no sign-in: {envelope}"
        );
        assert_warns_of_the_exchange(&envelope, status);
    }
}

/// A proxy's block page can repeat the request it refused. What the exchange
/// sent is a credential, so none of it reaches the error.
#[tokio::test]
async fn step2_failure_answer_that_echoes_the_exchange_leaks_none_of_it() {
    for status in [403, 404, 502] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(Echo(status))
            .mount(&server)
            .await;

        let (_code, envelope) = step2_at(&server.uri()).await;

        let text = envelope.to_string();
        for secret in ["AUTHCODE", PENDING_VERIFIER, "test-client-secret"] {
            assert!(!text.contains(secret), "{status} leaked {secret}: {text}");
        }
        assert!(
            envelope["message"]
                .as_str()
                .unwrap_or_default()
                .contains("<redacted>"),
            "{status}: the message shows where it was cut: {envelope}"
        );
    }
}

/// A page of tens of kilobytes would bury the rest of the output, so the error
/// keeps its start and says how much it left out.
#[tokio::test]
async fn step2_failure_answer_of_a_large_page_is_cut_short() {
    for status in [403, 404] {
        let (_code, envelope) =
            step2_answered_with(ResponseTemplate::new(status).set_body_string(big_page())).await;

        let message = envelope["message"].as_str().unwrap_or_default();
        assert!(message.len() < 1_300, "{status}: {} bytes", message.len());
        assert!(message.starts_with("<html>blocked"), "{status}: {message}");
        assert!(message.contains("more bytes"), "{status}: {message}");
    }
}

/// An empty 403 is said to be empty, not printed as `{}`.
#[tokio::test]
async fn step2_with_an_empty_token_endpoint_403_says_so() {
    let (_code, envelope) = step2_answered_with(ResponseTemplate::new(403)).await;

    assert_eq!(
        envelope["message"], "the token endpoint answered 403 with an empty body",
        "{envelope}"
    );
}

/// A warning names the token endpoint and the HTTP status of its answer, since
/// the error prints X's body alone.
fn assert_warns_of_the_exchange(envelope: &serde_json::Value, status: &str) {
    let warnings = envelope["warnings"].as_array().cloned().unwrap_or_default();
    assert!(
        warnings.iter().any(|warning| {
            warning["target"] == "xdk::auth"
                && warning["message"].as_str().is_some_and(|message| {
                    message.contains("token endpoint")
                        && message.contains("code exchange")
                        && message.contains(&format!("HTTP {status}"))
                })
        }),
        "a warning names the token endpoint and HTTP {status}: {envelope}"
    );
}

/// Text output tells a person where to look: the token URL and any proxy in
/// front of it, with the page that covers both.
#[tokio::test]
async fn text_output_points_a_token_endpoint_403_at_the_token_url_and_the_proxy() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(403).set_body_string("<html>Forbidden</html>"))
        .mount(&server)
        .await;
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    seed_pending(&store);
    let overrides = step2_overrides(
        &format!("{}/2/oauth2/token", server.uri()),
        &format!("{CLOSED}/2/users/me"),
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

    assert_eq!(code, xdk::error::EXIT_GENERAL_ERROR, "{stderr}");
    assert!(stderr.contains(FAILED_EXCHANGE), "{stderr}");
    assert!(stderr.contains("TOKEN_URL"), "{stderr}");
    assert!(stderr.contains("proxy"), "{stderr}");
    assert!(stderr.contains(TOKEN_ENDPOINT_DOCS), "{stderr}");
}

/// A text `warning:` line goes to the process's own stderr, which the
/// in-process runner's writer never sees, so this spawns the built `xr`.
#[tokio::test(flavor = "multi_thread")]
async fn text_output_warns_that_the_token_endpoint_answered_the_exchange() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(408).set_body_string("<html>Request Timeout</html>"))
        .mount(&server)
        .await;
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    seed_pending(&store);
    let token_url = format!("{}/2/oauth2/token", server.uri());
    let redirect = redirect_url();

    let output = tokio::task::spawn_blocking(move || {
        common::xr_with_store(&store)
            .env("CLIENT_ID", "test-client-id")
            .env("CLIENT_SECRET", "test-client-secret")
            .env("TOKEN_URL", &token_url)
            .env("API_BASE_URL", CLOSED)
            .env("INFO_URL", format!("{CLOSED}/2/users/me"))
            .args([
                "auth",
                "oauth2",
                "--no-browser",
                "--step",
                "2",
                "--auth-url",
                &redirect,
                "alice",
            ])
            .output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(xdk::error::EXIT_GENERAL_ERROR),
        "{stderr}"
    );
    assert!(
        stderr.lines().any(|line| {
            line.starts_with("warning: ")
                && line.contains("token endpoint")
                && line.contains("code exchange")
                && line.contains("HTTP 408")
        }),
        "{stderr}"
    );
}

#[tokio::test]
async fn step2_with_no_answer_is_a_network_error_and_is_sent_again() {
    let (code, envelope) = step2_at(CLOSED).await;

    assert_eq!(code, xdk::error::EXIT_NETWORK_ERROR, "{envelope}");
    assert_eq!(envelope["reason"], "network-error", "{envelope}");
    assert_eq!(
        envelope["exit_code"],
        xdk::error::EXIT_NETWORK_ERROR,
        "{envelope}"
    );
    assert_eq!(
        envelope["next_step"],
        serde_json::json!({"action": "retry"}),
        "{envelope}"
    );
    let message = envelope["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("TokenExchangeError"),
        "the message still says what failed: {envelope}"
    );
    assert!(
        message.contains("os error"),
        "the message names the system's cause, which tells a refused connection from a \
         timeout: {envelope}"
    );
}

/// The token URL is the caller's to fix, so a `retry` would fail the same way
/// every time.
#[tokio::test]
async fn step2_with_a_token_url_that_is_not_a_url_names_it() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let pending = seed_pending(&store);
    let overrides = step2_overrides("not-a-url", &format!("{CLOSED}/2/users/me"));

    let (code, _stdout, stderr) = run_cli(
        &store,
        &overrides,
        &[
            "xr",
            "--output",
            "json",
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

    let envelope: serde_json::Value = serde_json::from_str(stderr.trim()).expect("one envelope");
    assert_eq!(code, xdk::error::EXIT_GENERAL_ERROR, "{envelope}");
    assert_eq!(envelope["reason"], "invalid-url", "{envelope}");
    assert_step(&envelope, "show-help", "command", OAUTH2_HELP);
    let message = envelope["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("TOKEN_URL") && message.contains("not-a-url"),
        "{envelope}"
    );
    assert!(pending.exists(), "the flow is still good");
}

/// An answer that ends before its body did is no answer, whichever failure
/// status it came with: the same request can go again, the saved flow stays,
/// and the message says what failed.
#[tokio::test]
async fn step2_whose_error_answer_breaks_off_is_a_network_error_that_names_the_exchange() {
    for status_line in [
        "503 Service Unavailable",
        "403 Forbidden",
        "408 Request Timeout",
    ] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let port = listener.local_addr().expect("its address").port();
        let answer = format!("HTTP/1.1 {status_line}\r\ncontent-length: 100\r\n\r\npartial");
        let stub = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("the exchange connects");
            let mut request = Vec::new();
            let mut buf = [0u8; 4096];
            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = stream.read(&mut buf).expect("read the request");
                if n == 0 {
                    break;
                }
                request.extend_from_slice(&buf[..n]);
            }
            stream.write_all(answer.as_bytes()).expect("answer in part");
        });

        let (code, envelope) = step2_at(&format!("http://127.0.0.1:{port}")).await;
        stub.join().expect("the stub ends");

        assert_eq!(
            code,
            xdk::error::EXIT_NETWORK_ERROR,
            "{status_line}: {envelope}"
        );
        assert_eq!(envelope["reason"], "network-error", "{envelope}");
        assert_eq!(envelope["next_step"]["action"], "retry", "{envelope}");
        assert!(
            envelope["message"]
                .as_str()
                .unwrap_or_default()
                .contains("TokenExchangeError"),
            "{envelope}"
        );
        assert_warns_of_the_exchange(&envelope, &status_line[..3]);
    }
}

/// A 429 here answers as a 429 anywhere: a wait when it names its reset, and no
/// step when it does not.
#[tokio::test]
async fn step2_rate_limited_is_rate_limited_as_any_other_request_is() {
    let reset_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs()
        + 600;
    let (code, envelope) = step2_answered_with(
        oauth_error(429, "too_many_requests")
            .insert_header("x-rate-limit-reset", reset_at.to_string()),
    )
    .await;
    assert_eq!(code, xdk::error::EXIT_RATE_LIMITED, "{envelope}");
    assert_eq!(envelope["reason"], "rate-limited", "{envelope}");
    assert_eq!(
        envelope["exit_code"],
        xdk::error::EXIT_RATE_LIMITED,
        "{envelope}"
    );
    assert_eq!(
        envelope["next_step"]["action"], "wait-and-retry",
        "{envelope}"
    );
    assert!(envelope["retry_after_secs"].is_u64(), "{envelope}");

    let (code, envelope) = step2_answered_with(oauth_error(429, "too_many_requests")).await;
    assert_eq!(code, xdk::error::EXIT_RATE_LIMITED, "{envelope}");
    assert_eq!(envelope["reason"], "rate-limited", "{envelope}");
    assert!(envelope["next_step"].is_null(), "{envelope}");
}

/// The step works when run: the same invocation finishes once X can answer.
#[tokio::test]
async fn a_sign_in_x_could_not_answer_finishes_when_sent_again() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(oauth_error(503, "server_error"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    token_mock(ok_token_body(), 200).mount(&server).await;
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);
    let pending = seed_pending(&store);
    let overrides = step2_overrides(
        &format!("{}/2/oauth2/token", server.uri()),
        &format!("{}/2/users/me", server.uri()),
    );
    let redirect = redirect_url();
    let args = [
        "xr",
        "--output",
        "json",
        "auth",
        "oauth2",
        "--no-browser",
        "--step",
        "2",
        "--auth-url",
        &redirect,
        "alice",
    ];

    let (code, _stdout, stderr) = run_cli(&store, &overrides, &args).await;
    assert_eq!(code, xdk::error::EXIT_GENERAL_ERROR, "stderr: {stderr}");

    let (code, _stdout, stderr) = run_cli(&store, &overrides, &args).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    let saved = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    assert!(saved.get_oauth2_token("alice").is_some());
    assert!(!pending.exists(), "a finished flow is spent");
}

/// The client's own credentials were refused. A new consent would end at this
/// same refusal, so the step does not send a person back to the browser.
#[tokio::test]
async fn step2_whose_client_x_refused_asks_for_the_credentials_to_be_fixed() {
    for (why, response) in [
        ("a 401", oauth_error(401, "invalid_client")),
        (
            "invalid_client on a 400",
            oauth_error(400, "invalid_client"),
        ),
        (
            "unauthorized_client",
            oauth_error(400, "unauthorized_client"),
        ),
    ] {
        let (code, envelope) = step2_answered_with(response).await;
        assert_exchange_refused(code, &envelope);
        assert_eq!(
            envelope["next_step"],
            serde_json::json!({"action": "fix-input"}),
            "{why}: {envelope}"
        );
    }
}

#[tokio::test]
async fn text_output_names_the_steps_for_a_failed_exchange() {
    for (response, exit_code, line) in [
        (
            oauth_error(400, "invalid_grant"),
            xdk::error::EXIT_AUTH_REQUIRED,
            "Sign in again. Run: xr auth oauth2 --no-browser --step 1",
        ),
        (
            oauth_error(503, "server_error"),
            xdk::error::EXIT_GENERAL_ERROR,
            "Nothing was wrong with the request. Send it again.",
        ),
        (
            oauth_error(401, "invalid_client"),
            xdk::error::EXIT_AUTH_REQUIRED,
            "X refused the app's client credentials. Check its client id and secret.",
        ),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(response)
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
                "alice",
            ],
        )
        .await;

        assert_eq!(code, exit_code, "stderr: {stderr}");
        assert!(stderr.contains(line), "{line}: {stderr}");
        assert_eq!(
            stderr.contains(FAILED_EXCHANGE),
            exit_code != xdk::error::EXIT_AUTH_REQUIRED,
            "the line names an exchange the endpoint failed, not one it refused: {stderr}"
        );
    }
}

/// `--quiet` drops advice and never the error, and the line is part of it: the
/// error alone is X's body, which says nothing of the sign-in.
#[tokio::test]
async fn quiet_keeps_the_line_that_names_a_failed_exchange() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(502).set_body_string("<html>Bad Gateway</html>"))
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
            "--quiet",
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

    assert_eq!(code, xdk::error::EXIT_GENERAL_ERROR, "stderr: {stderr}");
    assert!(stderr.contains(FAILED_EXCHANGE), "{stderr}");
    assert!(
        !stderr.contains("Send it again"),
        "--quiet still drops the advice: {stderr}"
    );
}

#[tokio::test]
async fn the_interactive_state_needs_no_encoding() {
    let sent = Arc::new(Mutex::new(None::<String>));
    let recorded = Arc::clone(&sent);
    let _ = run_interactive(move |url, target| {
        let state = state_of(url);
        *recorded.lock().unwrap() = Some(state.clone());
        follow_redirect(target, &state)
    })
    .await;

    let state = sent.lock().unwrap().clone().expect("the opener ran");
    assert!(needs_no_encoding(&state), "state {state:?}");
}

#[tokio::test]
async fn the_headless_state_needs_no_encoding() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_store(&store);

    let state = query_param(&step1_url(&store, &[]).await, "state");

    assert!(needs_no_encoding(&state), "state {state:?}");
}
