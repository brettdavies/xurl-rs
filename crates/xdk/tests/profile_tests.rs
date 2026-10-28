//! The profile shortcuts send the requests the spec gives `/2/users/me` and
//! its image and banner, with only the fields a change sets; the offline
//! validator holds the spec's bounds on each field.

use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::shortcuts::{
    BannerCrop, ProfileChanges, validate_banner_crop, validate_profile_changes,
};
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
async fn update_profile_puts_only_the_fields_the_change_sets() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/2/users/me"))
        .and(body_json(json!({"description": "Building xr.", "url": ""})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"updated": true}})))
        .expect(1)
        .mount(&server)
        .await;
    let changes = ProfileChanges {
        description: Some("Building xr.".into()),
        url: Some(String::new()),
        ..ProfileChanges::default()
    };

    let resp = user_client(server.uri())
        .update_profile(&changes)
        .send()
        .await
        .expect("updated");

    assert!(resp.data.updated);
}

#[tokio::test]
async fn update_profile_image_puts_the_media_id() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/2/users/me/profile_image"))
        .and(body_json(json!({"media_id": "1146654567674912769"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"updated": true}})))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .update_profile_image("1146654567674912769")
        .send()
        .await
        .expect("updated");

    assert!(resp.data.updated);
}

#[tokio::test]
async fn update_profile_banner_puts_the_media_id_and_the_crop_when_given() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/2/users/me/profile_banner"))
        .and(body_json(json!({
            "media_id": "1146654567674912769",
            "crop": {"width": 1500, "height": 500, "offset_left": 0, "offset_top": 120}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"updated": true}})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/2/users/me/profile_banner"))
        .and(body_json(json!({"media_id": "1146654567674912770"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"updated": true}})))
        .expect(1)
        .mount(&server)
        .await;
    let client = user_client(server.uri());
    let crop = BannerCrop {
        width: 1500,
        height: 500,
        offset_left: 0,
        offset_top: 120,
    };

    let cropped = client
        .update_profile_banner("1146654567674912769", Some(crop))
        .send()
        .await
        .expect("cropped banner set");
    let whole = client
        .update_profile_banner("1146654567674912770", None)
        .send()
        .await
        .expect("whole banner set");

    assert!(cropped.data.updated && whole.data.updated);
}

#[tokio::test]
async fn remove_profile_banner_deletes_it() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/2/users/me/profile_banner"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"deleted": true}})))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .remove_profile_banner()
        .send()
        .await
        .expect("removed");

    assert!(resp.data.deleted);
}

#[test]
fn a_profile_change_must_set_a_field_within_the_spec_bounds() {
    let set = |f: fn(&mut ProfileChanges)| {
        let mut changes = ProfileChanges::default();
        f(&mut changes);
        validate_profile_changes(&changes)
    };

    assert_eq!(
        validate_profile_changes(&ProfileChanges::default()),
        Err("no-profile-changes")
    );
    assert_eq!(set(|c| c.name = Some("X Dev".into())), Ok(()));
    assert_eq!(
        set(|c| c.name = Some(" ".into())),
        Err("empty-profile-name")
    );
    assert_eq!(
        set(|c| c.name = Some("n".repeat(51))),
        Err("profile-name-too-long")
    );
    assert_eq!(set(|c| c.description = Some(String::new())), Ok(()));
    assert_eq!(
        set(|c| c.description = Some("é".repeat(161))),
        Err("profile-description-too-long")
    );
    assert_eq!(set(|c| c.description = Some("é".repeat(160))), Ok(()));
    assert_eq!(
        set(|c| c.url = Some("u".repeat(101))),
        Err("profile-url-too-long")
    );
    assert_eq!(
        set(|c| c.location = Some("l".repeat(31))),
        Err("profile-location-too-long")
    );
}

#[test]
fn a_banner_crop_has_a_size_within_the_spec_bounds() {
    let crop = |width, height| BannerCrop {
        width,
        height,
        offset_left: 0,
        offset_top: 0,
    };

    assert_eq!(validate_banner_crop(&crop(1500, 500)), Ok(()));
    assert_eq!(
        validate_banner_crop(&crop(0, 500)),
        Err("invalid-banner-crop")
    );
    assert_eq!(
        validate_banner_crop(&crop(1500, 0)),
        Err("invalid-banner-crop")
    );
    assert_eq!(
        validate_banner_crop(&crop(u32::MAX, 500)),
        Err("invalid-banner-crop")
    );
}
