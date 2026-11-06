//! A command whose `OAuth2` login has expired refreshes it first, and a failed
//! refresh reports what happened. A refresh X did not process is the failure
//! any other request to X reports, and the same command finishes once X
//! answers. A refresh X refused is `auth-required`, with the step that gets a
//! working login: a new sign-in for a refused refresh token, and the app's own
//! credentials for a refused client. Nothing a failed refresh touches is
//! written: the stored pair stays as it was.

mod common;

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::config::EnvOverrides;
use xdk::store::TokenStore;

use common::CLOSED;

const APP: &str = "myapp";
const LOGIN: &str = "alice";
const STORED_REFRESH_TOKEN: &str = "old-rt";

/// A store whose default app holds `alice`'s expired `OAuth2` login, so the
/// next request refreshes it before it sends.
fn expired_login(dir: &Path) -> PathBuf {
    let store = dir.join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app(APP, "CLIENT-ID", "CLIENT-SECRET")
        .expect("add_app");
    ts.save_oauth2_token_for_app(APP, LOGIN, "old-at", STORED_REFRESH_TOKEN, 0)
        .expect("save the expired login");
    ts.set_default_app(APP).expect("set_default_app");
    let _ = ts.remove_app("default");
    store
}

/// `xr <args>` in process, with the token endpoint at `token_url` and the API
/// at a closed port, which the run never reaches when the refresh fails.
async fn run(store: &Path, token_url: &str, args: &[&str]) -> (i32, String, String) {
    let overrides = EnvOverrides {
        token_url: Some(token_url.to_string()),
        api_base_url: Some(CLOSED.to_string()),
        info_url: Some(format!("{CLOSED}/2/users/me")),
        ..EnvOverrides::default()
    };
    let mut argv = vec!["xr"];
    argv.extend_from_slice(args);
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code =
        xurl::cli::runner::run_with_overrides(argv, &mut stdout, &mut stderr, store, &overrides)
            .await;
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

/// `xr --output json whoami -u alice` refreshing against `token_url`: its exit
/// code and error envelope, after checking the stored pair is untouched.
async fn whoami_refreshing_at(token_url: &str) -> (i32, Value) {
    let tmp = TempDir::new().unwrap();
    let store = expired_login(tmp.path());
    let (code, stdout, stderr) = run(
        &store,
        token_url,
        &["--output", "json", "whoami", "-u", LOGIN],
    )
    .await;
    assert_eq!(stdout, "", "a failed refresh sends nothing: {stdout}");
    assert_refresh_token_kept(&store);
    let envelope = serde_json::from_str(stderr.trim()).expect("one JSON envelope");
    (code, envelope)
}

async fn whoami_answered_with(response: ResponseTemplate) -> (i32, Value) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(response)
        .expect(1)
        .mount(&server)
        .await;
    whoami_refreshing_at(&format!("{}/2/oauth2/token", server.uri())).await
}

fn assert_refresh_token_kept(store: &Path) {
    let ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    let kept = ts
        .get_oauth2_token_for_app(APP, LOGIN)
        .and_then(|token| token.oauth2.as_ref())
        .map(|oauth2| oauth2.refresh_token.clone());
    assert_eq!(
        kept.as_deref(),
        Some(STORED_REFRESH_TOKEN),
        "a failed refresh keeps the stored pair"
    );
}

fn oauth_error(status: u16, error: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(json!({
        "error": error,
        "error_description": "as the token endpoint put it",
    }))
}

fn assert_reason(code: i32, envelope: &Value, reason: &str, exit_code: i32) {
    assert_eq!(code, exit_code, "{envelope}");
    assert_eq!(envelope["reason"], reason, "{envelope}");
    assert_eq!(envelope["exit_code"], exit_code, "{envelope}");
}

/// X's body is the message of an answer the token endpoint did not process,
/// as on any request, so a warning says the answer was to the refresh.
fn assert_warns_of_the_refresh(envelope: &Value) {
    let warnings = envelope["warnings"].as_array().cloned().unwrap_or_default();
    assert!(
        warnings.iter().any(|warning| {
            warning["target"] == "xdk::auth"
                && warning["message"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("refresh")
        }),
        "a warning names the refresh: {envelope}"
    );
}

fn assert_names_the_refresh(envelope: &Value) {
    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains("RefreshTokenError"),
        "the message says the refresh failed: {envelope}"
    );
}

#[tokio::test]
async fn a_refresh_with_no_answer_is_a_network_error_that_names_its_cause() {
    let (code, envelope) = whoami_refreshing_at(&format!("{CLOSED}/2/oauth2/token")).await;

    assert_reason(
        code,
        &envelope,
        "network-error",
        xdk::error::EXIT_NETWORK_ERROR,
    );

    assert_names_the_refresh(&envelope);
    assert_eq!(
        envelope["next_step"],
        json!({"action": "retry"}),
        "{envelope}"
    );
    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains("os error"),
        "the message names the system's cause, which tells a refused connection from a \
         timeout: {envelope}"
    );
}

/// A server error is the same answer whatever its body holds; a proxy's error
/// page is the usual one that holds no JSON.
#[tokio::test]
async fn a_refresh_answered_with_a_server_error_is_a_server_error_and_is_sent_again() {
    for (why, response) in [
        ("a 503", oauth_error(503, "server_error")),
        (
            "a 502 with no JSON",
            ResponseTemplate::new(502).set_body_string("<html>Bad Gateway</html>"),
        ),
    ] {
        let (code, envelope) = whoami_answered_with(response).await;
        assert_reason(
            code,
            &envelope,
            "server-error",
            xdk::error::EXIT_GENERAL_ERROR,
        );
        assert_eq!(
            envelope["next_step"]["action"], "retry",
            "{why}: {envelope}"
        );
        assert_warns_of_the_refresh(&envelope);
    }
}

#[tokio::test]
async fn a_refresh_rate_limited_is_rate_limited_as_any_other_request_is() {
    let reset_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs()
        + 600;
    let (code, envelope) = whoami_answered_with(
        oauth_error(429, "too_many_requests")
            .insert_header("x-rate-limit-reset", reset_at.to_string()),
    )
    .await;

    assert_reason(
        code,
        &envelope,
        "rate-limited",
        xdk::error::EXIT_RATE_LIMITED,
    );
    assert_eq!(
        envelope["next_step"]["action"], "wait-and-retry",
        "{envelope}"
    );
    assert!(envelope["retry_after_secs"].is_u64(), "{envelope}");
    assert_warns_of_the_refresh(&envelope);
}

/// A text `warning:` line goes to the process's own stderr, which the
/// in-process runner's writer never sees, so this test spawns the built `xr`.
#[tokio::test(flavor = "multi_thread")]
async fn text_output_says_a_server_error_answered_the_refresh() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(502).set_body_string("<html>Bad Gateway</html>"))
        .mount(&server)
        .await;
    let tmp = TempDir::new().unwrap();
    let store = expired_login(tmp.path());
    let token_url = format!("{}/2/oauth2/token", server.uri());

    let output = tokio::task::spawn_blocking(move || {
        common::xr_with_store(&store)
            .env("TOKEN_URL", &token_url)
            .env("API_BASE_URL", CLOSED)
            .env("INFO_URL", format!("{CLOSED}/2/users/me"))
            .args(["whoami", "-u", LOGIN])
            .output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");
    let code = output.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(code, xdk::error::EXIT_GENERAL_ERROR, "stderr: {stderr}");
    assert!(
        stderr
            .lines()
            .any(|line| line.starts_with("warning: ") && line.contains("refresh")),
        "{stderr}"
    );
}

/// The step signs the login in again on the app the command ran against,
/// which need not be the default.
#[tokio::test]
async fn a_refused_refresh_on_another_app_signs_in_on_that_app() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(oauth_error(400, "invalid_grant"))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().unwrap();
    let store = expired_login(tmp.path());
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("work", "WORK-CLIENT-ID", "WORK-SECRET")
        .expect("add a second app");
    ts.save_oauth2_token_for_app("work", LOGIN, "work-at", "work-rt", 0)
        .expect("save the expired login on it");

    let (code, _stdout, stderr) = run(
        &store,
        &format!("{}/2/oauth2/token", server.uri()),
        &["--output", "json", "--app", "work", "whoami", "-u", LOGIN],
    )
    .await;

    let envelope: Value = serde_json::from_str(stderr.trim()).expect("one JSON envelope");
    assert_reason(
        code,
        &envelope,
        "auth-required",
        xdk::error::EXIT_AUTH_REQUIRED,
    );
    assert_eq!(
        envelope["next_step"]["command"], "xr auth oauth2 alice --no-browser --step 1 --app work",
        "{envelope}"
    );
    let ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    let kept = ts
        .get_oauth2_token_for_app("work", LOGIN)
        .and_then(|token| token.oauth2.as_ref())
        .map(|oauth2| oauth2.refresh_token.clone());
    assert_eq!(kept.as_deref(), Some("work-rt"), "the stored pair stays");
}

/// A refresh's 429 is the rate limit any request meets, so
/// `--wait-on-rate-limit` waits out the reset it names and sends the refresh
/// again, inside the same run.
#[tokio::test]
async fn wait_on_rate_limit_waits_out_a_refresh_s_429() {
    let reset_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs()
        + 1;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(
            oauth_error(429, "too_many_requests")
                .insert_header("x-rate-limit-reset", reset_at.to_string()),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "new-at",
            "refresh_token": "new-rt",
            "expires_in": 7200,
        })))
        .mount(&server)
        .await;
    let tmp = TempDir::new().unwrap();
    let store = expired_login(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &format!("{}/2/oauth2/token", server.uri()),
        &[
            "--output",
            "json",
            "--wait-on-rate-limit",
            "whoami",
            "-u",
            LOGIN,
        ],
    )
    .await;

    assert_eq!(
        code,
        xdk::error::EXIT_NETWORK_ERROR,
        "the refresh succeeds after the wait, and the run fails only reaching the API at a \
         closed port: {stderr}"
    );
    let ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    let refreshed = ts
        .get_oauth2_token_for_app(APP, LOGIN)
        .and_then(|token| token.oauth2.as_ref())
        .map(|oauth2| oauth2.refresh_token.clone());
    assert_eq!(refreshed.as_deref(), Some("new-rt"), "{stderr}");
}

/// The same command refreshes once X answers: a failed refresh spent nothing.
#[tokio::test]
async fn a_command_whose_refresh_x_could_not_answer_refreshes_when_run_again() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(oauth_error(503, "server_error"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "new-at",
            "refresh_token": "new-rt",
            "expires_in": 7200,
        })))
        .mount(&server)
        .await;
    let tmp = TempDir::new().unwrap();
    let store = expired_login(tmp.path());
    let token_url = format!("{}/2/oauth2/token", server.uri());

    let (code, _stdout, stderr) = run(
        &store,
        &token_url,
        &["--output", "json", "whoami", "-u", LOGIN],
    )
    .await;
    assert_eq!(code, xdk::error::EXIT_GENERAL_ERROR, "stderr: {stderr}");
    assert_refresh_token_kept(&store);

    let (code, _stdout, stderr) = run(
        &store,
        &token_url,
        &["--output", "json", "whoami", "-u", LOGIN],
    )
    .await;
    assert_eq!(
        code,
        xdk::error::EXIT_NETWORK_ERROR,
        "the second run refreshes, then fails only reaching the API at a closed port: {stderr}"
    );
    let ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    let refreshed = ts
        .get_oauth2_token_for_app(APP, LOGIN)
        .and_then(|token| token.oauth2.as_ref())
        .map(|oauth2| oauth2.refresh_token.clone());
    assert_eq!(
        refreshed.as_deref(),
        Some("new-rt"),
        "the second run stored the new pair"
    );
}

/// Only a new sign-in gets a new refresh token, so the step is the sign-in
/// that stores the login under the name the command asked for.
#[tokio::test]
async fn a_refused_refresh_token_signs_the_login_in_again() {
    for (why, response) in [
        ("invalid_grant", oauth_error(400, "invalid_grant")),
        (
            "a 400 with no JSON",
            ResponseTemplate::new(400).set_body_string("<html>Bad Request</html>"),
        ),
        ("invalid_grant on a 403", oauth_error(403, "invalid_grant")),
        (
            "a 200 with no token",
            ResponseTemplate::new(200).set_body_json(json!({"token_type": "bearer"})),
        ),
        (
            "a 200 with no JSON",
            ResponseTemplate::new(200).set_body_string("<html>"),
        ),
    ] {
        let (code, envelope) = whoami_answered_with(response).await;
        assert_reason(
            code,
            &envelope,
            "auth-required",
            xdk::error::EXIT_AUTH_REQUIRED,
        );
        assert_names_the_refresh(&envelope);
        assert_eq!(
            envelope["next_step"]["action"], "sign-in",
            "{why}: {envelope}"
        );
        assert_eq!(
            envelope["next_step"]["command"], "xr auth oauth2 alice --no-browser --step 1",
            "{why}: {envelope}"
        );
    }
}

/// RFC 6749 section 5.2 refuses with a 400 or a 401, or with an `error`
/// object. Any other answer came from something that did not process the
/// refresh, such as a proxy or a token URL with the wrong path, so it reports
/// what any request to X would, and the refresh token stays good.
#[tokio::test]
async fn a_refresh_answered_with_no_oauth_refusal_reports_its_status() {
    for (why, response, reason, exit_code, action) in [
        (
            "a 408 with no JSON",
            ResponseTemplate::new(408).set_body_string("<html>Request Timeout</html>"),
            "api-error",
            xdk::error::EXIT_GENERAL_ERROR,
            Some("retry"),
        ),
        (
            "a 404 with no JSON",
            ResponseTemplate::new(404).set_body_string("<html>Not Found</html>"),
            "not-found",
            xdk::error::EXIT_NOT_FOUND,
            Some("fix-input"),
        ),
        (
            "a 403 with no JSON",
            ResponseTemplate::new(403).set_body_string("<html>Forbidden</html>"),
            "forbidden",
            xdk::error::EXIT_GENERAL_ERROR,
            None,
        ),
    ] {
        let (code, envelope) = whoami_answered_with(response).await;
        assert_reason(code, &envelope, reason, exit_code);
        assert_eq!(
            envelope["next_step"]["action"].as_str(),
            action,
            "{why}: {envelope}"
        );
        assert!(
            envelope["next_step"]["command"].is_null(),
            "{why}: no sign-in: {envelope}"
        );
        assert_warns_of_the_refresh(&envelope);
    }
}

/// Another sign-in would end at the same refusal, so the step sends nobody to
/// the browser: the message names what to fix.
#[tokio::test]
async fn a_refused_client_asks_for_the_apps_credentials_to_be_fixed() {
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
        ("a 401 with no body", ResponseTemplate::new(401)),
    ] {
        let (code, envelope) = whoami_answered_with(response).await;
        assert_reason(
            code,
            &envelope,
            "auth-required",
            xdk::error::EXIT_AUTH_REQUIRED,
        );
        assert_names_the_refresh(&envelope);
        assert_eq!(
            envelope["next_step"],
            json!({"action": "fix-input"}),
            "{why}: {envelope}"
        );
        assert!(
            !envelope["message"]
                .as_str()
                .unwrap_or_default()
                .contains("unknown"),
            "an answer that names no error is not given one: {why}: {envelope}"
        );
    }
}

/// With no `-u`, the refresh runs for the app's default login, and the step
/// signs that login in again: a sign-in that stored some other name would
/// leave the refused login where the next run finds it.
#[tokio::test]
async fn a_refused_refresh_with_no_login_named_signs_in_the_login_it_ran_for() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(oauth_error(400, "invalid_grant"))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().unwrap();
    let store = expired_login(tmp.path());
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.save_oauth2_token_for_app(APP, "bob", "bob-at", "bob-rt", 0)
        .expect("save a second expired login");
    ts.set_default_app_and_user(APP, "bob")
        .expect("make bob the default login");

    let (code, _stdout, stderr) = run(
        &store,
        &format!("{}/2/oauth2/token", server.uri()),
        &["--output", "json", "whoami"],
    )
    .await;

    let envelope: Value = serde_json::from_str(stderr.trim()).expect("one JSON envelope");
    assert_reason(
        code,
        &envelope,
        "auth-required",
        xdk::error::EXIT_AUTH_REQUIRED,
    );
    assert_eq!(
        envelope["next_step"]["command"], "xr auth oauth2 bob --no-browser --step 1",
        "{envelope}"
    );
}

/// A success whose body broke off is no answer to read: the same command
/// can run again. A refresh token X did spend then answers with the refusal
/// that signs in again.
#[tokio::test]
async fn a_refresh_whose_success_breaks_off_is_a_network_error() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let port = listener.local_addr().expect("its address").port();
    let stub = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("the refresh connects");
        let mut request = Vec::new();
        let mut buf = [0u8; 4096];
        while !request.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = stream.read(&mut buf).expect("read the request");
            if n == 0 {
                break;
            }
            request.extend_from_slice(&buf[..n]);
        }
        stream
            .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 100\r\n\r\n{\"access")
            .expect("answer in part");
    });

    let (code, envelope) =
        whoami_refreshing_at(&format!("http://127.0.0.1:{port}/2/oauth2/token")).await;
    stub.join().expect("the stub ends");

    assert_reason(
        code,
        &envelope,
        "network-error",
        xdk::error::EXIT_NETWORK_ERROR,
    );
    assert_names_the_refresh(&envelope);
    assert_eq!(
        envelope["next_step"],
        json!({"action": "retry"}),
        "{envelope}"
    );
}

#[tokio::test]
async fn a_token_url_that_is_not_a_url_names_it() {
    let (code, envelope) = whoami_refreshing_at("not-a-url").await;

    assert_reason(
        code,
        &envelope,
        "invalid-url",
        xdk::error::EXIT_GENERAL_ERROR,
    );
    let message = envelope["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("TOKEN_URL") && message.contains("'not-a-url'"),
        "the URL is quoted, so an empty one still shows: {envelope}"
    );
}

/// An answer that ends before its body did is no answer.
#[tokio::test]
async fn a_refresh_whose_error_answer_breaks_off_is_a_network_error_that_names_the_refresh() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let port = listener.local_addr().expect("its address").port();
    let stub = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("the refresh connects");
        let mut request = Vec::new();
        let mut buf = [0u8; 4096];
        while !request.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = stream.read(&mut buf).expect("read the request");
            if n == 0 {
                break;
            }
            request.extend_from_slice(&buf[..n]);
        }
        stream
            .write_all(b"HTTP/1.1 503 Service Unavailable\r\ncontent-length: 100\r\n\r\npartial")
            .expect("answer in part");
    });

    let (code, envelope) =
        whoami_refreshing_at(&format!("http://127.0.0.1:{port}/2/oauth2/token")).await;
    stub.join().expect("the stub ends");

    assert_reason(
        code,
        &envelope,
        "network-error",
        xdk::error::EXIT_NETWORK_ERROR,
    );

    assert_names_the_refresh(&envelope);
}

#[tokio::test]
async fn text_output_names_the_step_for_a_refused_refresh() {
    for (response, line) in [
        (
            oauth_error(400, "invalid_grant"),
            "X refused the stored login's refresh token. Sign in again. Run: xr auth oauth2 alice",
        ),
        (
            oauth_error(401, "invalid_client"),
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
        let store = expired_login(tmp.path());

        let (code, _stdout, stderr) = run(
            &store,
            &format!("{}/2/oauth2/token", server.uri()),
            &["whoami", "-u", LOGIN],
        )
        .await;

        assert_eq!(code, xdk::error::EXIT_AUTH_REQUIRED, "stderr: {stderr}");
        assert!(stderr.contains(line), "{line}: {stderr}");
    }
}
