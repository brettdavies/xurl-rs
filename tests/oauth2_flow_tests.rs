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

use xurl::auth::Auth;
use xurl::auth::oauth2::run_oauth2_flow;
use xurl::config::Config;
use xurl::output::{OutputConfig, OutputFormat};

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

#[test]
fn listener_bound_before_browser_opener_invoked() {
    let port = pick_free_port();
    let callback_target = format!("127.0.0.1:{port}");
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    reset_recorder(&callback_target);

    // Wiremock token + userinfo endpoints, driven from a dedicated runtime.
    let rt = tokio::runtime::Runtime::new().expect("build mock runtime");
    let server = rt.block_on(MockServer::start());
    let token_url = format!("{}/2/oauth2/token", server.uri());
    let info_url = format!("{}/2/users/me", server.uri());

    rt.block_on(
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "ACCESS-TOKEN",
                "refresh_token": "REFRESH-TOKEN",
                "expires_in": 7200,
                "token_type": "bearer"
            })))
            .mount(&server),
    );

    let tmp = TempDir::new().unwrap();
    let cfg = test_config(&token_url, &info_url, &redirect_uri);
    let mut auth = test_auth(cfg, &tmp, &redirect_uri);

    let out = OutputConfig::new(
        OutputFormat::Text,
        false,
        false,
        xurl::cli::ColorChoice::Auto,
    );
    let mut stdout = Vec::<u8>::new();

    let flow_started_at = Instant::now();
    let token = run_oauth2_flow(&mut auth, "testuser", &out, &mut stdout, recording_opener)
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
