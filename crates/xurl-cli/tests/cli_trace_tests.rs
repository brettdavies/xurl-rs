//! `xr --trace` puts `X-B3-Flags: 1` on the wire, and a shortcut's flags
//! reach the request through the call builder.

use tempfile::TempDir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use xdk::config::EnvOverrides;
use xdk::store::TokenStore;
use xurl::cli;

fn me_body() -> serde_json::Value {
    serde_json::json!({"data": {"id": "111", "username": "self", "name": "Self"}})
}

/// A store whose default app carries an `OAuth1` token, the scheme
/// `/2/users/me` accepts.
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

async fn run(store: &std::path::Path, base_url: &str, args: &[&str]) -> (i32, String) {
    let overrides = EnvOverrides {
        api_base_url: Some(base_url.to_string()),
        ..EnvOverrides::default()
    };
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code =
        cli::runner::run_with_overrides(args, &mut stdout, &mut stderr, store, &overrides).await;
    (code, String::from_utf8_lossy(&stderr).into_owned())
}

#[tokio::test]
async fn trace_flag_sends_the_x_b3_flags_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .and(header("X-B3-Flags", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(me_body()))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(&tmp);

    let (code, stderr) = run(&store, &server.uri(), &["xr", "whoami", "--trace"]).await;
    assert_eq!(code, 0, "whoami --trace failed: {stderr}");
}

#[tokio::test]
async fn without_the_trace_flag_no_x_b3_flags_header_is_sent() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .and(header("X-B3-Flags", "1"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(me_body()))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(&tmp);

    let (code, stderr) = run(&store, &server.uri(), &["xr", "whoami"]).await;
    assert_eq!(code, 0, "whoami failed: {stderr}");
}
