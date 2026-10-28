//! `xr analytics` reads impressions and engagement for up to 100 of the
//! caller's posts over a time window through `GET /2/tweets/analytics`. Both
//! bounds are required and checked as a window before any request, and the
//! metrics print under the spec's names.

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

#[tokio::test]
async fn analytics_sends_the_ids_and_window_and_prints_the_metrics() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/analytics"))
        .and(query_param("ids", "1,2"))
        .and(query_param("start_time", START))
        .and(query_param("end_time", END))
        .and(query_param("granularity", "daily"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "1",
                    "impressions": 120,
                    "likes": 4,
                    "timestamped_metrics": [{
                        "timestamp": START,
                        "metrics": {"impressions": 120, "likes": 4}
                    }]
                },
                {"id": "2", "impressions": 30}
            ]
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
            "analytics",
            "1",
            "https://x.com/alice/status/2",
            "--start",
            START,
            "--end",
            END,
            "--granularity",
            "daily",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["impressions"], 120);
    assert_eq!(
        body["data"][0]["timestamped_metrics"][0]["metrics"]["likes"],
        4
    );
    assert_eq!(body["data"][1]["impressions"], 30);
}

#[tokio::test]
async fn analytics_refuses_a_window_out_of_order_or_without_a_zone() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    for (start, end) in [(END, START), (START, "2026-10-02T00:00:00")] {
        let (code, stdout, stderr) = run(
            &store,
            &server.uri(),
            &[
                "--dry-run",
                "--output",
                "json",
                "analytics",
                "1",
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

#[tokio::test]
async fn analytics_without_a_start_is_refused_by_the_parser() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "analytics", "1", "--end", END],
    )
    .await;

    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stderr.contains("--start"), "{stderr}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
