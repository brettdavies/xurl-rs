//! `Client::upload_media` runs INIT, APPEND, and FINALIZE with the media
//! type and category inferred from the file, and reports the media id.

use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::auth::OAuth2Credential;

fn user_client(base_url: String) -> Client {
    Client::builder()
        .base_url(base_url)
        .oauth2(OAuth2Credential {
            client_id: "client-id".into(),
            client_secret: "client-secret".into(),
            access_token: "user-at".into(),
            refresh_token: None,
            expires_at: None,
        })
        .build()
        .expect("client builds")
}

async fn mount_phases(server: &MockServer, media_id: &str) {
    let body = serde_json::json!({
        "data": {"id": media_id, "media_key": format!("3_{media_id}"), "expires_after_secs": 86400}
    });
    Mock::given(method("POST"))
        .and(path("/2/media/upload/initialize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/2/media/upload/{media_id}/append")))
        .respond_with(ResponseTemplate::new(204))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/2/media/upload/{media_id}/finalize")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(server)
        .await;
}

#[tokio::test]
async fn upload_media_infers_the_type_and_category_and_returns_the_media_id() {
    let server = MockServer::start().await;
    mount_phases(&server, "710511363345354753").await;
    let tmp = TempDir::new().expect("tempdir");
    let file = tmp.path().join("photo.png");
    std::fs::write(&file, b"not really a png").expect("write");

    let outcome = user_client(server.uri())
        .upload_media(&file)
        .send()
        .await
        .expect("upload succeeds");

    assert_eq!(outcome.media_id(), "710511363345354753");
    assert!(outcome.processing.is_none(), "an image is not awaited");
    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(requests.len(), 3, "INIT, APPEND, FINALIZE");
    let init: serde_json::Value = serde_json::from_slice(&requests[0].body).expect("INIT is JSON");
    assert_eq!(init["media_type"], "image/png");
    assert_eq!(init["media_category"], "tweet_image");
    assert_eq!(init["total_bytes"], 16);
    assert_eq!(
        requests[1].url.path(),
        "/2/media/upload/710511363345354753/append"
    );
    assert_eq!(
        requests[2].url.path(),
        "/2/media/upload/710511363345354753/finalize"
    );
}

#[tokio::test]
async fn upload_media_takes_an_explicit_type_and_category_over_the_inferred_ones() {
    let server = MockServer::start().await;
    mount_phases(&server, "42").await;
    let tmp = TempDir::new().expect("tempdir");
    let file = tmp.path().join("frame.bin");
    std::fs::write(&file, b"gif bytes").expect("write");

    user_client(server.uri())
        .upload_media(&file)
        .media_type("image/gif")
        .category("dm_gif")
        .send()
        .await
        .expect("upload succeeds");

    let requests = server.received_requests().await.expect("requests recorded");
    let init: serde_json::Value = serde_json::from_slice(&requests[0].body).expect("INIT is JSON");
    assert_eq!(init["media_type"], "image/gif");
    assert_eq!(init["media_category"], "dm_gif");
}

#[tokio::test]
async fn upload_media_rejects_an_unknown_extension_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let file = tmp.path().join("notes.txt");
    std::fs::write(&file, b"x").expect("write");

    let err = user_client(server.uri())
        .upload_media(&file)
        .send()
        .await
        .expect_err("no media type to infer");

    assert!(err.is_validation(), "{err:?}");
    assert!(err.to_string().contains("media_type"), "{err}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

// ── A wait on media processing always ends ─────────────────────────────

use std::time::Duration;

use wiremock::matchers::query_param;
use xdk::Error;
use xdk::api::execute_media_status;
use xdk::error::NextAction;

/// The STATUS endpoint for media `m1`, answering `data`.
fn status_mock(data: serde_json::Value) -> Mock {
    Mock::given(method("GET"))
        .and(path("/2/media/upload"))
        .and(query_param("command", "STATUS"))
        .and(query_param("media_id", "m1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": data })))
}

fn in_progress() -> serde_json::Value {
    serde_json::json!({
        "id": "m1",
        "processing_info": {"state": "in_progress", "check_after_secs": 1, "progress_percent": 10}
    })
}

async fn status_calls(server: &MockServer) -> usize {
    server
        .received_requests()
        .await
        .expect("requests recorded")
        .iter()
        .filter(|request| request.method == wiremock::http::Method::GET)
        .count()
}

#[tokio::test]
async fn a_status_without_processing_info_ends_the_wait_after_one_call() {
    let server = MockServer::start().await;
    status_mock(serde_json::json!({"id": "m1", "media_key": "3_m1"}))
        .mount(&server)
        .await;
    let client = user_client(server.uri());

    let status = tokio::time::timeout(
        Duration::from_secs(5),
        execute_media_status(
            "m1",
            "",
            "",
            Some(Duration::from_secs(30)),
            false,
            &[],
            &client,
        ),
    )
    .await
    .expect("the wait ends without a deadline's help")
    .expect("the status reads");

    assert!(status.data.processing_info.is_none());
    assert_eq!(status_calls(&server).await, 1);
}

#[tokio::test]
async fn a_wait_past_its_deadline_is_a_processing_timeout_carrying_the_id_and_the_wait() {
    let server = MockServer::start().await;
    status_mock(in_progress()).mount(&server).await;
    let client = user_client(server.uri());

    let error = execute_media_status(
        "m1",
        "",
        "",
        Some(Duration::from_secs(2)),
        false,
        &[],
        &client,
    )
    .await
    .expect_err("the job never finishes");

    let Error::ProcessingTimeout { media_id, waited } = &error else {
        panic!("expected a processing timeout, got {error:?}");
    };
    assert_eq!(media_id, "m1");
    assert_eq!(*waited, Duration::from_secs(2));
    assert_eq!(error.kind(), "processing-timeout");
    assert_eq!(error.exit_code(), 1);
    assert_eq!(error.next_action(), Some(NextAction::ResumeWait));
    let calls = status_calls(&server).await;
    assert!(
        (1..=3).contains(&calls),
        "a two-second deadline at one check a second allows at most three status calls, saw {calls}"
    );
}

#[tokio::test]
async fn a_failed_job_is_a_validation_error_not_a_timeout() {
    let server = MockServer::start().await;
    status_mock(serde_json::json!({"id": "m1", "processing_info": {"state": "failed"}}))
        .mount(&server)
        .await;
    let client = user_client(server.uri());

    let error = execute_media_status(
        "m1",
        "",
        "",
        Some(Duration::from_secs(30)),
        false,
        &[],
        &client,
    )
    .await
    .expect_err("processing failed");

    assert!(error.is_validation(), "got {error:?}");
    assert_eq!(error.to_string(), "media processing failed");
}

#[tokio::test]
async fn a_status_read_without_a_wait_returns_the_job_as_it_stands() {
    let server = MockServer::start().await;
    status_mock(in_progress()).mount(&server).await;
    let client = user_client(server.uri());

    let status = execute_media_status("m1", "", "", None, false, &[], &client)
        .await
        .expect("the status reads");

    assert_eq!(
        status
            .data
            .processing_info
            .as_ref()
            .map(|info| info.state.as_str()),
        Some("in_progress")
    );
    assert_eq!(status_calls(&server).await, 1);
}

/// An upload whose video outlasts the deadline still completed at FINALIZE:
/// the outcome carries the media id and the timeout beside it.
#[tokio::test]
async fn an_upload_reports_a_processing_timeout_beside_its_media_id() {
    let server = MockServer::start().await;
    mount_phases(&server, "m1").await;
    status_mock(in_progress()).mount(&server).await;
    let tmp = TempDir::new().expect("tempdir");
    let file = tmp.path().join("clip.mp4");
    std::fs::write(&file, b"not really a video").expect("write");

    let outcome = user_client(server.uri())
        .upload_media(&file)
        .processing_deadline(Duration::from_secs(1))
        .send()
        .await
        .expect("the upload itself completes");

    assert_eq!(outcome.media_id(), "m1");
    assert!(
        matches!(
            &outcome.processing,
            Some(Err(Error::ProcessingTimeout { media_id, waited }))
                if media_id == "m1" && *waited == Duration::from_secs(1)
        ),
        "got {:?}",
        outcome.processing
    );
}
