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
