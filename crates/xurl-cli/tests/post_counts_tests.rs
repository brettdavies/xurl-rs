//! `xr search-counts` reports how many recent posts match a query per time
//! bucket through `GET /2/tweets/counts/recent`, before a search is paid
//! for. `--granularity` takes the spec's three values, and `--start` and
//! `--end` are checked as a time window before any request.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wiremock::matchers::{header, method, path, query_param};
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
async fn search_counts_sends_the_query_and_granularity_and_reads_each_bucket() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/counts/recent"))
        .and(query_param("query", "from:jack"))
        .and(query_param("granularity", "day"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "start": "2026-10-25T00:00:00.000Z",
                "end": "2026-10-26T00:00:00.000Z",
                "tweet_count": 7
            }],
            "meta": {"total_tweet_count": 7}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "search-counts",
            "from:jack",
            "--granularity",
            "day",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["post_count"], 7);
    assert_eq!(body["meta"]["total_post_count"], 7);
}

#[tokio::test]
async fn search_counts_goes_out_with_the_bearer_token_alone() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/counts/recent"))
        .and(header("authorization", "Bearer BEARER-TOKEN"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [],
            "meta": {"total_tweet_count": 0}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["search-counts", "rust", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn search_counts_refuses_a_granularity_outside_the_spec() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "search-counts",
            "rust",
            "--granularity",
            "week",
        ],
    )
    .await;

    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(
        stderr.contains("minute") && stderr.contains("hour") && stderr.contains("day"),
        "{stderr}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn search_counts_refuses_a_window_whose_end_precedes_its_start() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    for (start, end) in [
        ("2026-10-26T00:00:00Z", "2026-10-25T00:00:00Z"),
        ("2026-10-25T00:00:00Z", "2026-10-26T00:00:00"),
        ("2026-10-26T00:00:00Z", "2026-10-26T01:00:00+02:00"),
    ] {
        let (code, stdout, stderr) = run(
            &store,
            &server.uri(),
            &[
                "--dry-run",
                "--output",
                "json",
                "search-counts",
                "rust",
                "--start",
                start,
                "--end",
                end,
            ],
        )
        .await;

        assert_eq!(code, 0, "{start} {end}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{start} {end}");
        assert_eq!(body["reason"], "invalid-time-window", "{start} {end}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
