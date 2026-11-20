//! The Amplify shortcuts send the requests the spec gives their endpoints and
//! decode their responses; the offline validators hold the spec's patterns on
//! the ids and the promotion end.

use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::shortcuts::{AmplifyPromotion, validate_amplify_promotion, validate_program_id};
use xdk::auth::OAuth2Credential;

const PROGRAM: &str = "prog_2026-launch";

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

fn program(state: &str) -> serde_json::Value {
    json!({
        "id": PROGRAM,
        "name": "Launch pre-rolls",
        "state": state,
        "advertiser_ids": ["1234567890"],
        "publisher_ids": []
    })
}

#[tokio::test]
async fn the_reads_hit_their_paths_and_decode() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/media/amplify/programs"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": [program("active")]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/2/media/amplify/programs/{PROGRAM}")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": program("inactive")})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/2/media/amplify/programs/{PROGRAM}/posts")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [
            {"post_id": "1346889436626259968", "added_at": "2026-11-02T16:00:00Z"}
        ]})))
        .expect(1)
        .mount(&server)
        .await;
    let client = user_client(server.uri());

    let listed = client.get_amplify_programs().send().await.expect("listed");
    let one = client
        .get_amplify_program(PROGRAM)
        .send()
        .await
        .expect("read");
    let posts = client
        .get_amplify_program_posts(PROGRAM)
        .send()
        .await
        .expect("posts");

    assert_eq!(listed.data[0].state, "active");
    assert_eq!(one.data.state, "inactive");
    assert_eq!(posts.data[0].post_id, "1346889436626259968");
}

#[tokio::test]
async fn add_post_sends_the_post_id_and_only_the_fields_it_was_given() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/amplify/posts"))
        .and(body_json(json!({
            "post_id": "1346889436626259968",
            "program_ids": [PROGRAM, "other"],
            "promotion_ends_at": "2026-12-01T00:00:00Z"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"added": true}})))
        .expect(1)
        .mount(&server)
        .await;
    let promotion = AmplifyPromotion {
        program_ids: vec![PROGRAM.into(), "other".into()],
        promotion_ends_at: Some("2026-12-01T00:00:00Z".into()),
        ..AmplifyPromotion::default()
    };

    let resp = user_client(server.uri())
        .add_post_to_amplify_programs("https://x.com/jack/status/1346889436626259968", &promotion)
        .send()
        .await
        .expect("added");

    assert!(resp.data.added);
}

#[test]
fn a_program_id_is_the_specs_pattern() {
    for id in ["a", "prog_2026-launch", &"a".repeat(64)] {
        assert_eq!(validate_program_id(id), Ok(()), "{id:?}");
    }
    for id in ["", "has space", "dot.ted", "slash/ed", &"a".repeat(65)] {
        assert_eq!(validate_program_id(id), Err("invalid-program-id"), "{id:?}");
    }
}

#[test]
fn a_promotion_is_held_to_the_specs_patterns() {
    let promotion = |ids: &[&str], advertisers: Option<&[&str]>, end: Option<&str>| {
        validate_amplify_promotion(&AmplifyPromotion {
            program_ids: ids.iter().map(ToString::to_string).collect(),
            blocked_advertiser_ids: advertisers
                .map(|ids| ids.iter().map(ToString::to_string).collect()),
            promotion_ends_at: end.map(ToString::to_string),
        })
    };

    assert_eq!(promotion(&["a"], None, None), Ok(()));
    assert_eq!(
        promotion(&["a"], Some(&["123"]), Some("2026-12-01T00:00:00+02:00")),
        Ok(())
    );
    assert_eq!(promotion(&["a b"], None, None), Err("invalid-program-id"));
    assert_eq!(
        promotion(&["a"], Some(&["123", "x1"]), None),
        Err("invalid-advertiser-id")
    );
    assert_eq!(
        promotion(&["a"], None, Some("next week")),
        Err("invalid-promotion-end")
    );
    assert_eq!(
        promotion(&["a"], None, Some("2026-12-01")),
        Err("invalid-promotion-end")
    );
}
