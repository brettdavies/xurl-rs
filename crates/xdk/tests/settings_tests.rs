//! The account-settings shortcuts read every field `/2/users/me/settings`
//! names and send only the settings a change sets; the offline validator
//! holds the spec's bounds on each one.

use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::shortcuts::{
    DmsFrom, MediaTagging, QuietHours, SettingsChanges, validate_settings_changes,
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
async fn get_settings_asks_for_every_field_and_reads_them() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me/settings"))
        .and(query_param(
            "user_settings.fields",
            "allow_dms_from,allow_media_tagging,always_allow_dms_from_subscribers,\
             discoverable_by_email,discoverable_by_phone,display_sensitive_media,id,language,\
             location_tagging_enabled,personalized_trends,protected,quiet_hours,time_zone,\
             use_cookie_personalization",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "id": "2244994945",
                "allow_dms_from": "following",
                "protected": false,
                "language": "en",
                "quiet_hours": {"enabled": true, "start_hour": 22, "end_hour": 7},
                "time_zone": {
                    "name": "Pacific Time (US & Canada)",
                    "tz_name": "America/Los_Angeles",
                    "utc_offset_seconds": -25200
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .get_settings()
        .send()
        .await
        .expect("read");

    let settings = resp.data;
    assert_eq!(settings.allow_dms_from.as_deref(), Some("following"));
    assert_eq!(settings.protected, Some(false));
    let quiet = settings.quiet_hours.expect("quiet hours");
    assert_eq!((quiet.start_hour, quiet.end_hour), (Some(22), Some(7)));
    let zone = settings.time_zone.expect("a time zone");
    assert_eq!(zone.tz_name.as_deref(), Some("America/Los_Angeles"));
    assert_eq!(zone.utc_offset_seconds, Some(-25200));
}

#[tokio::test]
async fn update_settings_puts_only_the_settings_the_change_sets() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/2/users/me/settings"))
        .and(body_json(json!({
            "allow_dms_from": "verified",
            "allow_media_tagging": "none",
            "protected": true,
            "quiet_hours": {"enabled": true, "start_hour": 22},
            "time_zone": {"name": "America/Chicago"}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"updated": true}})))
        .expect(1)
        .mount(&server)
        .await;
    let changes = SettingsChanges {
        allow_dms_from: Some(DmsFrom::Verified),
        allow_media_tagging: Some(MediaTagging::None),
        protected: Some(true),
        quiet_hours: Some(QuietHours {
            enabled: Some(true),
            start_hour: Some(22),
            end_hour: None,
        }),
        time_zone: Some("America/Chicago".into()),
        ..SettingsChanges::default()
    };

    let resp = user_client(server.uri())
        .update_settings(&changes)
        .send()
        .await
        .expect("updated");

    assert!(resp.data.updated);
}

#[test]
fn a_settings_change_must_set_a_setting_within_the_spec_bounds() {
    let set = |f: fn(&mut SettingsChanges)| {
        let mut changes = SettingsChanges::default();
        f(&mut changes);
        validate_settings_changes(&changes)
    };

    assert_eq!(
        validate_settings_changes(&SettingsChanges::default()),
        Err("no-settings-changes")
    );
    assert_eq!(set(|c| c.discoverable_by_email = Some(false)), Ok(()));
    assert_eq!(set(|c| c.language = Some("pt-BR".into())), Ok(()));
    assert_eq!(
        set(|c| c.language = Some("e".into())),
        Err("invalid-settings-language")
    );
    assert_eq!(
        set(|c| c.language = Some("x".repeat(36))),
        Err("invalid-settings-language")
    );
    assert_eq!(
        set(|c| c.time_zone = Some(String::new())),
        Err("invalid-settings-time-zone")
    );
    assert_eq!(
        set(|c| c.time_zone = Some("z".repeat(101))),
        Err("invalid-settings-time-zone")
    );
    assert_eq!(
        set(|c| c.quiet_hours = Some(QuietHours {
            end_hour: Some(24),
            ..QuietHours::default()
        })),
        Err("invalid-quiet-hour")
    );
    assert_eq!(
        set(|c| c.quiet_hours = Some(QuietHours::default())),
        Err("no-settings-changes")
    );
}
