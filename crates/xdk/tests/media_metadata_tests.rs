//! The media alt-text and subtitle shortcuts send the bodies the spec
//! requires and decode their responses; a bearer-only client is refused
//! before any request leaves.

use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::shortcuts::{validate_alt_text, validate_language_code, validate_media_id};
use xdk::api::{Client, VideoCategory};
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

#[tokio::test]
async fn set_media_alt_text_posts_the_id_and_the_alt_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/metadata"))
        .and(body_json(json!({
            "id": "1880028106020515840",
            "metadata": {"alt_text": {"text": "A dog asleep on a towel"}}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "id": "1880028106020515840",
                "associated_metadata": {"alt_text": {"text": "A dog asleep on a towel"}}
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = user_client(server.uri())
        .set_media_alt_text("1880028106020515840", "A dog asleep on a towel")
        .send()
        .await
        .expect("alt text is set");

    assert_eq!(result.data.id, "1880028106020515840");
    assert!(result.data.associated_metadata.is_some());
}

#[tokio::test]
async fn add_media_subtitles_posts_the_track_under_the_wire_category() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/subtitles"))
        .and(body_json(json!({
            "id": "1880028106020515840",
            "media_category": "TweetVideo",
            "subtitles": {
                "id": "1880028217379905536",
                "language_code": "EN",
                "display_name": "English"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"id": "1880028106020515840", "media_category": "TweetVideo"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = user_client(server.uri())
        .add_media_subtitles(
            "1880028106020515840",
            VideoCategory::TweetVideo,
            "1880028217379905536",
            "en",
            Some("English"),
        )
        .send()
        .await
        .expect("subtitles are added");

    assert_eq!(result.data.media_category.as_deref(), Some("TweetVideo"));
}

#[tokio::test]
async fn add_media_subtitles_omits_an_absent_display_name() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/subtitles"))
        .and(body_json(json!({
            "id": "1880028106020515840",
            "media_category": "AmplifyVideo",
            "subtitles": {"id": "1880028217379905536", "language_code": "FR"}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}})))
        .expect(1)
        .mount(&server)
        .await;

    user_client(server.uri())
        .add_media_subtitles(
            "1880028106020515840",
            VideoCategory::AmplifyVideo,
            "1880028217379905536",
            "FR",
            None,
        )
        .send()
        .await
        .expect("subtitles are added");
}

#[tokio::test]
async fn remove_media_subtitles_deletes_with_a_json_body() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/2/media/subtitles"))
        .and(body_json(json!({
            "id": "1880028106020515840",
            "media_category": "AmplifyVideo",
            "language_code": "EN"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"deleted": true}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = user_client(server.uri())
        .remove_media_subtitles("1880028106020515840", VideoCategory::AmplifyVideo, "en")
        .send()
        .await
        .expect("subtitles are removed");

    assert!(result.data.deleted);
}

#[tokio::test]
async fn a_bearer_only_client_is_refused_before_any_request() {
    let server = MockServer::start().await;
    let client = Client::builder()
        .base_url(server.uri())
        .bearer("app-only-token")
        .build()
        .expect("client builds");

    let err = client
        .set_media_alt_text("1880028106020515840", "alt")
        .send()
        .await
        .expect_err("the endpoint accepts no app-only scheme");

    assert_eq!(err.kind(), "auth-method-mismatch", "{err}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[test]
fn video_category_round_trips_its_upload_name() {
    for category in VideoCategory::ALL {
        assert_eq!(
            VideoCategory::from_upload_name(category.upload_name()),
            Some(category)
        );
    }
    assert_eq!(VideoCategory::from_upload_name("tweet_image"), None);
}

#[test]
fn validators_name_the_reason_for_each_bad_input() {
    assert_eq!(validate_media_id("1880028106020515840"), Ok(()));
    assert_eq!(validate_media_id(""), Err("invalid-media-id"));
    assert_eq!(validate_media_id("12a"), Err("invalid-media-id"));
    assert_eq!(validate_media_id(&"9".repeat(20)), Err("invalid-media-id"));

    assert_eq!(validate_alt_text("A dog"), Ok(()));
    assert_eq!(validate_alt_text("  "), Err("empty-alt-text"));
    assert_eq!(validate_alt_text(&"é".repeat(1000)), Ok(()));
    assert_eq!(
        validate_alt_text(&"é".repeat(1001)),
        Err("alt-text-too-long")
    );

    assert_eq!(validate_language_code("en"), Ok(()));
    assert_eq!(validate_language_code("EN"), Ok(()));
    assert_eq!(validate_language_code("eng"), Err("invalid-language-code"));
    assert_eq!(validate_language_code("e1"), Err("invalid-language-code"));
}
