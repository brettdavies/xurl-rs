//! `xr trends place` and `xr trends personal` read the trends for a place,
//! by its Yahoo WOEID, and the caller's personalized trends. A WOEID that
//! is not a positive integer, or a `--max` outside 1 to 50, is refused by
//! the parser; the personalized trends take a user login only.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn bearer_store(dir: &Path) -> PathBuf {
    let store = dir.join(".xurl");
    let mut ts = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_bearer_token_for_app("myapp", "BEARER-TOKEN")
        .expect("save bearer");
    ts.set_default_app("myapp").expect("set_default_app");
    store
}

#[tokio::test]
async fn place_reads_the_trends_for_a_woeid() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/trends/by/woeid/1"))
        .and(query_param("max_trends", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"trend_name": "#rustlang", "tweet_count": 1200}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["trends", "place", "1", "--max", "10", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["trend_name"], "#rustlang");
    assert_eq!(body["data"][0]["tweet_count"], 1200);
}

#[tokio::test]
async fn personal_reads_the_callers_personalized_trends() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/personalized_trends"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "trend_name": "Rust 2026",
                "category": "Technology",
                "post_count": "12.5K posts",
                "trending_since": "Trending now"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["trends", "personal", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["category"], "Technology");
    assert_eq!(body["data"][0]["post_count"], "12.5K posts");
}

#[tokio::test]
async fn place_refuses_a_woeid_or_max_the_spec_does_not_take() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    for args in [
        &["trends", "place", "0"][..],
        &["trends", "place", "london"][..],
        &["trends", "place", "1", "--max", "51"][..],
    ] {
        let argv: Vec<&str> = ["--output", "json"].iter().chain(args).copied().collect();
        let (code, _stdout, stderr) = run(&store, &server.uri(), &argv).await;
        assert_eq!(code, 2, "{args:?}: {stderr}");
        let envelope: serde_json::Value =
            serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
        assert_eq!(envelope["reason"], "invalid-args", "{args:?}");
        assert!(
            envelope["message"]
                .as_str()
                .is_some_and(|m| m.contains("invalid value")),
            "{args:?}: {envelope}"
        );
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn personal_with_only_a_bearer_is_an_auth_mismatch() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "trends", "personal"],
    )
    .await;

    assert_ne!(code, 0, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "auth-method-mismatch");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
