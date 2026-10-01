//! `xr media alt-text` and `xr media subtitles` reach the media metadata and
//! subtitle endpoints through the runner: each sends the JSON body the spec
//! requires and prints the typed response as JSON, and the dry run refuses a
//! malformed id before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth1_store, run_in_process as run};

#[tokio::test]
async fn alt_text_posts_the_media_id_and_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/metadata"))
        .and(body_json(serde_json::json!({
            "id": "1585341984679469056",
            "metadata": {"alt_text": {"text": "A dog asleep on a beach towel"}}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "1585341984679469056",
                "associated_metadata": {"alt_text": {"text": "A dog asleep on a beach towel"}}
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "media",
            "alt-text",
            "1585341984679469056",
            "A dog asleep on a beach towel",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["id"], "1585341984679469056");
}

#[tokio::test]
async fn subtitles_add_sends_the_track_with_the_wire_category() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/subtitles"))
        .and(body_json(serde_json::json!({
            "id": "1585341984679469056",
            "media_category": "TweetVideo",
            "subtitles": {
                "id": "1585341984679469057",
                "language_code": "EN",
                "display_name": "English"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "1585341984679469056", "media_category": "TweetVideo"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "media",
            "subtitles",
            "add",
            "1585341984679469056",
            "1585341984679469057",
            "--language",
            "en",
            "--name",
            "English",
            "--category",
            "tweet_video",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["media_category"], "TweetVideo");
}

#[tokio::test]
async fn subtitles_remove_deletes_with_a_json_body() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/2/media/subtitles"))
        .and(body_json(serde_json::json!({
            "id": "1585341984679469056",
            "media_category": "AmplifyVideo",
            "language_code": "EN"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"deleted": true}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "media",
            "subtitles",
            "remove",
            "1585341984679469056",
            "--language",
            "en",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["deleted"], true);
}

#[tokio::test]
async fn a_malformed_media_id_is_refused_without_a_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "media",
            "alt-text",
            "not-an-id",
            "A dog",
            "--output",
            "json",
        ],
    )
    .await;

    assert_ne!(code, 0);
    assert!(stderr.contains("invalid-media-id"), "stderr: {stderr}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
