//! `xr media get` reads 1 to 100 uploads by media key in one request, and
//! `xr media analytics` reports their video metrics over a time window. A
//! media key is two numbers joined by an underscore; a key outside that, a
//! window out of order, and more than 100 keys are refused before any
//! request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const START: &str = "2026-10-01T00:00:00Z";
const END: &str = "2026-10-02T00:00:00Z";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

async fn run_json(server: &MockServer, args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let mut argv = args.to_vec();
    argv.extend(["--output", "json"]);
    run(&store, &server.uri(), &argv).await
}

#[tokio::test]
async fn get_reads_every_key_in_one_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/media"))
        .and(query_param(
            "media_keys",
            "3_1146654567674912769,7_1146654567674912770",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"media_key": "3_1146654567674912769", "type": "photo", "width": 1200},
                {"media_key": "7_1146654567674912770", "type": "video", "duration_ms": 46000}
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "media",
            "get",
            "3_1146654567674912769",
            "7_1146654567674912770",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["type"], "photo");
    assert_eq!(body["data"][1]["duration_ms"], 46000);
}

#[tokio::test]
async fn analytics_sends_the_keys_and_window() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/media/analytics"))
        .and(query_param("media_keys", "7_1146654567674912770"))
        .and(query_param("start_time", START))
        .and(query_param("end_time", END))
        .and(query_param("granularity", "total"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"media_key": "7_1146654567674912770", "video_views": 900, "playback_complete": 120}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "media",
            "analytics",
            "7_1146654567674912770",
            "--start",
            START,
            "--end",
            END,
            "--granularity",
            "total",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["video_views"], 900);
}

#[tokio::test]
async fn inputs_outside_the_spec_are_refused_before_any_request() {
    let server = MockServer::start().await;
    let keys: Vec<String> = (1..=101).map(|n| format!("3_{n}")).collect();
    let too_many: Vec<&str> = ["media", "get"]
        .into_iter()
        .chain(keys.iter().map(String::as_str))
        .collect();

    for (args, reason) in [
        (
            vec!["media", "get", "1146654567674912769"],
            "invalid-media-key",
        ),
        (vec!["media", "get", "3_"], "invalid-media-key"),
        (too_many, "too-many-ids"),
        (
            vec!["media", "analytics", "3_1", "--start", END, "--end", START],
            "invalid-time-window",
        ),
    ] {
        let mut argv = vec!["--dry-run"];
        argv.extend(&args);
        let (code, stdout, stderr) = run_json(&server, &argv).await;

        assert_eq!(code, 0, "{:?}: {stderr}", &args[..3]);
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{:?}", &args[..3]);
        assert_eq!(body["reason"], reason, "{:?}", &args[..3]);
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
