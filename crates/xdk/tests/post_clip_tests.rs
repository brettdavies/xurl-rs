//! A clip post sends the `clip` object the spec defines beside the text, with
//! a post URL resolved to its id and no media; the validators refuse a post id
//! or a range the spec's request schema would not take, and leave the clip's
//! length and the source post to X.

use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::shortcuts::{PostClip, validate_clip_post_id, validate_post_clip};
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

fn clip(post_id: &str, start_seconds: u32, end_seconds: u32) -> PostClip {
    PostClip {
        post_id: post_id.to_string(),
        start_seconds,
        end_seconds,
    }
}

async fn answer_with_body(server: &MockServer, body: serde_json::Value) {
    Mock::given(method("POST"))
        .and(path("/2/tweets"))
        .and(body_json(body))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(json!({"data": {"id": "55555", "text": "Worth watching"}})),
        )
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn create_clip_post_sends_the_clip_beside_the_text() {
    let server = MockServer::start().await;
    answer_with_body(
        &server,
        json!({
            "text": "Worth watching",
            "clip": {"post_id": "1585341984679469056", "start_seconds": 30, "end_seconds": 60}
        }),
    )
    .await;

    let resp = user_client(server.uri())
        .create_clip_post("Worth watching", &clip("1585341984679469056", 30, 60))
        .send()
        .await
        .expect("the clip post is sent");

    assert_eq!(resp.data.id, "55555");
}

#[tokio::test]
async fn create_clip_post_resolves_a_post_url_to_its_id() {
    let server = MockServer::start().await;
    answer_with_body(
        &server,
        json!({
            "text": "Worth watching",
            "clip": {"post_id": "1585341984679469056", "start_seconds": 0, "end_seconds": 5}
        }),
    )
    .await;

    user_client(server.uri())
        .create_clip_post(
            "Worth watching",
            &clip("https://x.com/alice/status/1585341984679469056", 0, 5),
        )
        .send()
        .await
        .expect("the clip post is sent");
}

#[tokio::test]
async fn create_post_still_sends_no_clip() {
    let server = MockServer::start().await;
    answer_with_body(&server, json!({"text": "Worth watching"})).await;

    user_client(server.uri())
        .create_post("Worth watching", &[])
        .send()
        .await
        .expect("the post is sent");
}

#[test]
fn a_clip_of_a_numeric_post_id_that_ends_after_it_starts_validates() {
    assert_eq!(
        validate_post_clip(&clip("1585341984679469056", 0, 1)),
        Ok(())
    );
    assert_eq!(
        validate_post_clip(&clip(
            "https://x.com/alice/status/1585341984679469056",
            30,
            60
        )),
        Ok(())
    );
}

#[test]
fn a_clip_post_id_that_is_not_one_to_nineteen_digits_is_refused() {
    for id in ["", "abc", "12a", "https://x.com/alice", &"9".repeat(20)] {
        assert_eq!(
            validate_post_clip(&clip(id, 0, 10)),
            Err("invalid-clip-post-id"),
            "{id:?}"
        );
        assert_eq!(
            validate_clip_post_id(id),
            Err("invalid-clip-post-id"),
            "{id:?}"
        );
    }
}

#[test]
fn a_clip_that_does_not_end_after_it_starts_is_refused() {
    for (start, end) in [(0, 0), (30, 30), (60, 30)] {
        assert_eq!(
            validate_post_clip(&clip("1585341984679469056", start, end)),
            Err("invalid-clip-range"),
            "{start}-{end}"
        );
    }
}

#[test]
fn the_clips_length_and_source_are_left_to_x() {
    assert_eq!(
        validate_post_clip(&clip("1585341984679469056", 0, 1)),
        Ok(())
    );
    assert_eq!(
        validate_post_clip(&clip("1585341984679469056", 0, 3600)),
        Ok(())
    );
}
