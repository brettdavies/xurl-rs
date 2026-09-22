//! Library diagnostics reach `xr`'s stderr no matter which task emits them:
//! a warning raised inside the spawned token refresh renders exactly like one
//! raised on the dispatch future itself.
//!
//! The renderer writes to the process's own stderr, so the assertion spawns
//! the built binary rather than driving the runner in-process.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use xdk::store::TokenStore;

/// A store whose default app holds an already-expired `OAuth2` token for
/// `alice`, so the first request refreshes before it sends.
fn expired_oauth2_store(tmp: &TempDir) -> std::path::PathBuf {
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_oauth2_token_for_app("myapp", "alice", "old-at", "old-rt", 0)
        .expect("save_oauth2");
    ts.set_default_app("myapp").expect("set_default_app");
    let _ = ts.remove_app("default");
    store
}

/// Runs `xr <args>` against `base_url` with the OAuth2 endpoints pointed at
/// the same mock, off the runtime thread so the mock keeps serving.
async fn run(store: std::path::PathBuf, base_url: String, args: &[&str]) -> (i32, String) {
    let args: Vec<String> = args.iter().map(|a| (*a).to_string()).collect();
    let output = tokio::task::spawn_blocking(move || {
        let mut cmd = common::xr_with_store(&store);
        cmd.env("API_BASE_URL", &base_url)
            .env("TOKEN_URL", format!("{base_url}/2/oauth2/token"))
            .env("INFO_URL", format!("{base_url}/2/users/me"))
            .args(&args)
            .output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn a_refresh_whose_username_lookup_fails_warns_on_stderr() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "new-at",
            "refresh_token": "new-rt",
            "expires_in": 7200
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .and(header("Authorization", "Bearer new-at"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": [{"id": "1", "text": "hi"}]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = expired_oauth2_store(&tmp);

    let (code, stderr) = run(store, server.uri(), &["search", "hi"]).await;

    assert_eq!(code, 0, "search after a refresh failed: {stderr}");
    assert!(
        stderr.contains(
            "warning: refresh succeeded but /2/users/me lookup failed; token stored under unnamed slot"
        ),
        "the refresh warning must reach stderr: {stderr:?}"
    );
}

/// Runs `xr <args>` with a bearer token in the environment and the API
/// pointed at `base_url`, off the runtime thread so the mock keeps serving.
async fn run_with_bearer(base_url: String, args: &[&str]) -> (i32, String) {
    let args: Vec<String> = args.iter().map(|a| (*a).to_string()).collect();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let output = tokio::task::spawn_blocking(move || {
        let mut cmd = common::xr_with_store(&store);
        cmd.env("API_BASE_URL", &base_url)
            .env("XURL_BEARER_TOKEN", "bearer-token")
            .args(&args)
            .output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn a_user_supplied_default_header_is_reported_in_the_binary_s_words() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .and(header("User-Agent", "custom/1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": [{"id": "1", "text": "hi"}]})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let url = format!("{}/2/tweets/search/recent?query=rust", server.uri());
    let (code, stderr) = run_with_bearer(
        server.uri(),
        &[&url, "--verbose", "-H", "User-Agent: custom/1"],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stderr.contains("info: user-supplied User-Agent detected; skipping xurl append"),
        "the advisory keeps its 3.x wording: {stderr}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_rejected_redirect_uri_warns_before_dispatch() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let output = tokio::task::spawn_blocking(move || {
        let mut cmd = common::xr_with_store(&store);
        cmd.env("API_BASE_URL", "http://127.0.0.1:1")
            .env("REDIRECT_URI", "not a url")
            .args(["auth", "status"])
            .output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("REDIRECT_URI"),
        "the rejected override is reported on stderr: {stderr}"
    );
}

/// A store whose default app holds OAuth1 tokens, the scheme `xr post`
/// signs with.
fn oauth1_store(tmp: &TempDir) -> std::path::PathBuf {
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_oauth1_tokens_for_app(
        "myapp",
        "OA1-ACCESS-TOKEN",
        "TOKEN-SECRET",
        "OA1-CONSUMER-KEY",
        "CONSUMER-SECRET",
    )
    .expect("save_oauth1");
    ts.set_default_app("myapp").expect("set_default_app");
    let _ = ts.remove_app("default");
    store
}

/// Runs `xr <args>` with the API pointed at `base_url` and returns the exit
/// code, stdout, and stderr, off the runtime thread so the mock keeps
/// serving.
async fn run_capturing(
    store: std::path::PathBuf,
    base_url: String,
    args: &[&str],
) -> (i32, String, String) {
    let args: Vec<String> = args.iter().map(|a| (*a).to_string()).collect();
    let output = tokio::task::spawn_blocking(move || {
        let mut cmd = common::xr_with_store(&store);
        cmd.env("API_BASE_URL", &base_url).args(&args).output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn verbose_reports_a_legacy_key_x_sent_and_stdout_reads_the_current_name() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/tweets"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": "2101712260468977783", "text": "hi", "edit_history_tweet_ids": [""]}
        })))
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(&tmp);
    let line =
        "info: X sent edit_history_tweet_ids; read as edit_history_post_ids (array, length 1)";

    let (code, stdout, stderr) = run_capturing(
        store.clone(),
        server.uri(),
        &["--verbose", "post", "hi", "--auth", "oauth1"],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stderr.lines().any(|l| l == line),
        "--verbose prints the normalization line: {stderr}"
    );
    assert!(
        stdout.contains("\"edit_history_post_ids\"") && !stdout.contains("edit_history_tweet_ids"),
        "stdout reads the current name: {stdout}"
    );

    for flags in [
        &["post", "hi", "--auth", "oauth1"][..],
        &["--verbose", "--quiet", "post", "hi", "--auth", "oauth1"],
        &[
            "--verbose",
            "--output",
            "json",
            "post",
            "hi",
            "--auth",
            "oauth1",
        ],
        &[
            "--verbose",
            "--output",
            "jsonl",
            "post",
            "hi",
            "--auth",
            "oauth1",
        ],
    ] {
        let (code, _stdout, stderr) = run_capturing(store.clone(), server.uri(), flags).await;
        assert_eq!(code, 0, "{flags:?}: {stderr}");
        assert!(
            !stderr.contains("X sent"),
            "{flags:?} must not print the normalization line: {stderr}"
        );
    }
}
