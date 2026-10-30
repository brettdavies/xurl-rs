//! `xr broadcasts sources` reaches the stream-source endpoints and the
//! ingest region through the runner: each verb sends the request the spec
//! gives its endpoint and prints the typed response, a `remove` asks before
//! it deletes, and an empty name is refused before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth1_store, run_in_process as run};

const SOURCE_ID: &str = "1x2y3z4w5v6u";

fn source(name: &str) -> serde_json::Value {
    serde_json::json!({
        "id": SOURCE_ID,
        "name": name,
        "region": "pdxa",
        "rtmps_url": "rtmps://va.pscp.tv:443/x",
        "stream_key": SOURCE_ID,
        "is_stream_active": false,
        "created_at_ms": "1761148800000"
    })
}

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

#[tokio::test]
async fn sources_list_prints_the_sources_as_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/sources"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"data": [source("Studio A")], "meta": {"result_count": 1}}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["broadcasts", "sources", "list", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["name"], "Studio A");
    assert_eq!(body["data"][0]["stream_key"], SOURCE_ID);
    assert_eq!(body["meta"]["result_count"], 1);
}

#[tokio::test]
async fn sources_region_prints_the_nearest_ingest_region() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/region"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"region": "us-west-2"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["broadcasts", "sources", "region", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["region"], "us-west-2");
}

#[tokio::test]
async fn sources_add_posts_the_name_and_region() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/sources"))
        .and(body_json(
            serde_json::json!({"name": "Studio A", "region": "eu-west-1"}),
        ))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({"data": source("Studio A")})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "sources",
            "add",
            "Studio A",
            "--region",
            "eu-west-1",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["id"], SOURCE_ID);
}

#[tokio::test]
async fn sources_get_reads_one_source() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/sources/{SOURCE_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": source("Studio A")})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "sources",
            "get",
            SOURCE_ID,
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["is_stream_active"], false);
}

#[tokio::test]
async fn sources_update_puts_the_new_name() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/sources/{SOURCE_ID}")))
        .and(body_json(serde_json::json!({"name": "Studio B"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": source("Studio B")})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "sources",
            "update",
            SOURCE_ID,
            "--name",
            "Studio B",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["name"], "Studio B");
}

/// The label is a flag, as `lists update --name` takes it, and `update`
/// sends nothing without one.
#[tokio::test]
async fn sources_update_needs_the_name_flag() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    for args in [
        &["broadcasts", "sources", "update", SOURCE_ID][..],
        &["broadcasts", "sources", "update", SOURCE_ID, "Studio B"][..],
    ] {
        let mut argv = vec!["--output", "json"];
        argv.extend(args);
        let (code, _stdout, stderr) = run(&store, &server.uri(), &argv).await;
        assert_eq!(code, 2, "{args:?}: {stderr}");
        let envelope: serde_json::Value =
            serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
        assert_eq!(envelope["reason"], "invalid-args", "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn sources_remove_without_force_asks_first_and_deletes_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--no-interactive",
            "--output",
            "json",
            "broadcasts",
            "sources",
            "remove",
            SOURCE_ID,
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "confirmation-required");
    assert_eq!(envelope["next_step"]["action"], "confirm");
    assert!(
        envelope["next_step"]["template"]
            .as_str()
            .is_some_and(|t| t.contains("--force")),
        "{envelope}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn sources_remove_with_force_deletes_the_source() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/sources/{SOURCE_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "sources",
            "remove",
            SOURCE_ID,
            "--force",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["deleted"], true);
}

#[tokio::test]
async fn sources_remove_under_dry_run_says_the_real_run_would_ask() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "broadcasts",
            "sources",
            "remove",
            SOURCE_ID,
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "broadcasts-sources-remove");
    assert_eq!(body["source_id"], SOURCE_ID);
    assert_eq!(body["confirmation_required"], true);
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn sources_add_refuses_an_empty_name_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "broadcasts",
            "sources",
            "add",
            " ",
            "--region",
            "us-east-1",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["would_succeed"], false);
    assert_eq!(body["rule"], "empty-source-name");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
