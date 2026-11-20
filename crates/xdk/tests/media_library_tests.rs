//! The media library shortcuts send the requests the spec gives their
//! endpoints and decode their responses; the offline validators hold the
//! spec's bounds on a library item's details.

use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::shortcuts::{
    LIBRARY_DESCRIPTION_MAX_CHARS, LIBRARY_TITLE_MAX_CHARS, LIBRARY_VIDEO_CATEGORIES_MAX,
    LibraryItemDetails, LibraryMonetization, validate_country_code, validate_language_tag,
    validate_library_addition, validate_library_changes, validate_library_details,
};
use xdk::auth::OAuth2Credential;

const MEDIA_KEY: &str = "7_1146654567674912770";

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
async fn add_media_to_library_sends_the_key_and_only_the_details_it_was_given() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/library"))
        .and(body_json(json!({
            "media_key": MEDIA_KEY,
            "title": "Launch day",
            "monetization": {
                "enabled": true,
                "video_category_ids": [4],
                "allowed_countries": ["US", "CA"]
            }
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(json!({"data": {"media_key": MEDIA_KEY}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let details = LibraryItemDetails {
        title: Some("Launch day".into()),
        monetization: Some(LibraryMonetization {
            enabled: Some(true),
            video_category_ids: Some(vec![4]),
            allowed_countries: Some(vec!["US".into(), "CA".into()]),
            ..LibraryMonetization::default()
        }),
        ..LibraryItemDetails::default()
    };

    let resp = user_client(server.uri())
        .add_media_to_library(MEDIA_KEY, &details)
        .send()
        .await
        .expect("added");

    assert_eq!(resp.data.media_key, MEDIA_KEY);
}

#[tokio::test]
async fn update_media_in_library_sends_only_the_fields_it_was_given() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/media/library/{MEDIA_KEY}")))
        .and(body_json(json!({"name": "Renamed"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"updated": true}})))
        .expect(1)
        .mount(&server)
        .await;
    let details = LibraryItemDetails {
        name: Some("Renamed".into()),
        ..LibraryItemDetails::default()
    };

    let resp = user_client(server.uri())
        .update_media_in_library(MEDIA_KEY, &details)
        .send()
        .await
        .expect("updated");

    assert!(resp.data.updated);
}

#[tokio::test]
async fn remove_media_from_library_sends_the_empty_object_the_spec_requires() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/media/library/{MEDIA_KEY}")))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"deleted": true}})))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .remove_media_from_library(MEDIA_KEY)
        .send()
        .await
        .expect("removed");

    assert!(resp.data.deleted);
}

#[tokio::test]
async fn get_media_library_categories_sends_the_language_only_when_given() {
    let server = MockServer::start().await;
    let answer = json!({"data": {
        "video_categories": [{"id": 1, "description": "Notícias"}],
        "advertiser_categories": []
    }});
    Mock::given(method("GET"))
        .and(path("/2/media/library/categories"))
        .and(query_param("lang", "pt-BR"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&answer))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/media/library/categories"))
        .and(query_param_is_missing("lang"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&answer))
        .expect(1)
        .mount(&server)
        .await;
    let client = user_client(server.uri());

    let localized = client
        .get_media_library_categories(Some("pt-BR"))
        .send()
        .await
        .expect("fetched");
    client
        .get_media_library_categories(None)
        .send()
        .await
        .expect("fetched");

    assert_eq!(localized.data.video_categories[0].description, "Notícias");
}

#[test]
fn details_are_held_to_the_specs_bounds() {
    let titled = |title: String| LibraryItemDetails {
        title: Some(title),
        ..LibraryItemDetails::default()
    };
    let described = |text: String| LibraryItemDetails {
        description: Some(text),
        ..LibraryItemDetails::default()
    };
    let monetized = |monetization: LibraryMonetization| LibraryItemDetails {
        monetization: Some(monetization),
        ..LibraryItemDetails::default()
    };

    assert_eq!(
        validate_library_details(&titled("a".repeat(LIBRARY_TITLE_MAX_CHARS + 1))),
        Err("library-title-too-long")
    );
    assert_eq!(
        validate_library_details(&described("a".repeat(LIBRARY_DESCRIPTION_MAX_CHARS + 1))),
        Err("library-description-too-long")
    );
    assert_eq!(
        validate_library_details(&monetized(LibraryMonetization {
            video_category_ids: Some(vec![1; LIBRARY_VIDEO_CATEGORIES_MAX + 1]),
            ..LibraryMonetization::default()
        })),
        Err("too-many-video-categories")
    );
    for code in ["USA", "U", "U1", ""] {
        assert_eq!(
            validate_library_details(&monetized(LibraryMonetization {
                allowed_countries: Some(vec!["US".into(), code.into()]),
                ..LibraryMonetization::default()
            })),
            Err("invalid-country-code"),
            "{code:?}"
        );
    }
    assert_eq!(
        validate_library_details(&titled("a".repeat(LIBRARY_TITLE_MAX_CHARS))),
        Ok(())
    );
    assert_eq!(
        validate_library_details(&monetized(LibraryMonetization {
            allowed_countries: Some(vec!["us".into(), "CA".into()]),
            ..LibraryMonetization::default()
        })),
        Ok(())
    );
}

#[test]
fn a_change_with_no_field_is_refused_and_any_field_is_not() {
    assert_eq!(
        validate_library_changes(&LibraryItemDetails::default()),
        Err("no-library-changes")
    );
    assert_eq!(
        validate_library_changes(&LibraryItemDetails {
            label: Some("promo".into()),
            ..LibraryItemDetails::default()
        }),
        Ok(())
    );
}

#[test]
fn a_language_tag_is_the_specs_pattern() {
    for tag in ["en", "pt-BR", "zh-Hant-TW", "fil", "es-419"] {
        assert_eq!(validate_language_tag(tag), Ok(()), "{tag:?}");
    }
    for tag in [
        "",
        "e",
        "english",
        "en-",
        "en-b",
        "en_US",
        "-en",
        "en-toolongsub",
    ] {
        assert_eq!(
            validate_language_tag(tag),
            Err("invalid-language-tag"),
            "{tag:?}"
        );
    }
}

#[test]
fn an_addition_takes_one_category_and_needs_a_category_or_a_program_for_ads() {
    let monetized = |monetization: LibraryMonetization| LibraryItemDetails {
        monetization: Some(monetization),
        ..LibraryItemDetails::default()
    };

    assert_eq!(
        validate_library_addition(&monetized(LibraryMonetization {
            video_category_ids: Some(vec![1, 2]),
            ..LibraryMonetization::default()
        })),
        Err("too-many-video-categories")
    );
    assert_eq!(
        validate_library_details(&monetized(LibraryMonetization {
            video_category_ids: Some(vec![1, 2]),
            ..LibraryMonetization::default()
        })),
        Ok(()),
        "a change keeps the structured bound of two"
    );
    assert_eq!(
        validate_library_addition(&monetized(LibraryMonetization {
            enabled: Some(true),
            ..LibraryMonetization::default()
        })),
        Err("monetization-needs-category")
    );
    for monetization in [
        LibraryMonetization {
            enabled: Some(true),
            video_category_ids: Some(vec![4]),
            ..LibraryMonetization::default()
        },
        LibraryMonetization {
            enabled: Some(true),
            program_id: Some("prog".into()),
            ..LibraryMonetization::default()
        },
        LibraryMonetization {
            enabled: Some(false),
            ..LibraryMonetization::default()
        },
    ] {
        assert_eq!(
            validate_library_addition(&monetized(monetization.clone())),
            Ok(()),
            "{monetization:?}"
        );
    }
    assert_eq!(
        validate_library_addition(&LibraryItemDetails::default()),
        Ok(())
    );
}

#[test]
fn a_country_code_is_two_letters() {
    assert_eq!(validate_country_code("us"), Ok(()));
    for code in ["", "U", "USA", "U1", "\u{e9}e"] {
        assert_eq!(
            validate_country_code(code),
            Err("invalid-country-code"),
            "{code:?}"
        );
    }
}
