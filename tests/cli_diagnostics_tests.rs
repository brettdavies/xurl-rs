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

use xurl::store::TokenStore;

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
