//! `xr news search` and `xr news get` search X's news stories and read one
//! by id. A story id the spec's pattern rejects is refused before any
//! request, and the posts a story clusters print under the spec's
//! `cluster_posts_results` however X spells the key.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const STORY_ID: &str = "1850000000000000000";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

#[tokio::test]
async fn search_sends_the_query_page_size_and_age() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/news/search"))
        .and(query_param("query", "ai"))
        .and(query_param("max_results", "5"))
        .and(query_param("max_age_hours", "24"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": STORY_ID, "name": "A story", "category": "Technology"}],
            "meta": {"result_count": 1}
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
            "news",
            "search",
            "ai",
            "-n",
            "5",
            "--max-age-hours",
            "24",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["name"], "A story");
}

#[tokio::test]
async fn get_reads_one_story_and_its_clustered_posts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/news/{STORY_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": STORY_ID,
                "name": "A story",
                "cluster_tweets_results": [{"tweet_id": "1585341984679469056"}]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["news", "get", STORY_ID, "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(
        body["data"]["cluster_posts_results"][0]["post_id"],
        "1585341984679469056"
    );
}

#[tokio::test]
async fn get_refuses_an_id_outside_the_spec_pattern_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    for id in ["", "story-1", "12345678901234567890"] {
        let (code, stdout, stderr) = run(
            &store,
            &server.uri(),
            &["--dry-run", "--output", "json", "news", "get", id],
        )
        .await;

        assert_eq!(code, 0, "{id:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{id:?}");
        assert_eq!(body["rule"], "invalid-news-id", "{id:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
