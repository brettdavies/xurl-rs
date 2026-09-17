//! `xr media upload --wait` against a mock whose processing fails: the
//! upload itself completed at FINALIZE, so the caller still gets the media id
//! on stdout before the processing error ends the run.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::store::TokenStore;

/// A store whose default app holds a far-future OAuth2 token for `alice`, so
/// no refresh happens before the upload.
fn oauth2_store(tmp: &TempDir) -> std::path::PathBuf {
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_oauth2_token_for_app("myapp", "alice", "user-at", "user-rt", 9_999_999_999)
        .expect("save_oauth2");
    ts.set_default_app("myapp").expect("set_default_app");
    let _ = ts.remove_app("default");
    store
}

#[tokio::test(flavor = "multi_thread")]
async fn a_processing_failure_after_finalize_still_prints_the_media_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/upload/initialize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "777", "media_key": "7_777", "expires_after_secs": 86400}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/media/upload/777/append"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/media/upload/777/finalize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "777", "processing_info": {"state": "pending", "check_after_secs": 1}}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/media/upload"))
        .and(query_param("command", "STATUS"))
        .and(query_param("media_id", "777"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "777", "processing_info": {"state": "failed"}}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(&tmp);
    let video = tmp.path().join("clip.mp4");
    std::fs::write(&video, b"not really a video").expect("write clip");
    let base_url = server.uri();
    let output = tokio::task::spawn_blocking(move || {
        let mut cmd = common::xr_with_store(&store);
        cmd.env("API_BASE_URL", &base_url)
            .args(["media", "upload", video.to_str().expect("utf-8 path")])
            .output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(output.status.code(), Some(0), "processing failed: {stderr}");
    assert!(
        stdout.contains("\"777\""),
        "the FINALIZE envelope with the media id reaches stdout first: {stdout}"
    );
    assert!(
        stderr.contains("media processing failed"),
        "the processing error is reported: {stderr}"
    );
}
