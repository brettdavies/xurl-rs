//! The post draft shortcuts send the requests the spec gives their endpoints
//! and decode their responses.

use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::shortcuts::validate_draft_id;
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
async fn create_post_draft_sends_only_the_text_it_was_given() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/posts/drafts"))
        .and(body_json(json!({"text": "Remember to post this"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"data": {"id": "1850000000000000013", "text": "Remember to post this"}}),
        ))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .create_post_draft("Remember to post this", &[], None)
        .send()
        .await
        .expect("saved");

    assert_eq!(resp.data.id, "1850000000000000013");
}

#[tokio::test]
async fn create_post_draft_sends_its_media_and_the_post_it_replies_to() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/posts/drafts"))
        .and(body_json(json!({
            "text": "Photos from the launch",
            "media": {"media_ids": ["1850000000000000020", "1850000000000000021"]},
            "reply": {"in_reply_to_tweet_id": "1585341984679469056"}
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"data": {"id": "1850000000000000014", "text": "Photos from the launch"}}),
        ))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .create_post_draft(
            "Photos from the launch",
            &[
                "1850000000000000020".to_string(),
                "1850000000000000021".to_string(),
            ],
            Some("https://x.com/jack/status/1585341984679469056"),
        )
        .send()
        .await
        .expect("saved");

    assert_eq!(resp.data.id, "1850000000000000014");
}

#[tokio::test]
async fn get_post_drafts_sends_the_page_size_and_the_cursor() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/posts/drafts"))
        .and(query_param("max_results", "5"))
        .and(query_param("pagination_token", "1850000000000000012"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"id": "1850000000000000011", "text": "Older draft"}],
            "meta": {"result_count": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .get_post_drafts(5)
        .pagination_token("1850000000000000012")
        .send()
        .await
        .expect("read");

    assert_eq!(resp.data[0].text, "Older draft");
}

#[tokio::test]
async fn delete_post_draft_names_the_draft_in_the_path() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/2/posts/drafts/1850000000000000011"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"deleted": true}})))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .delete_post_draft("1850000000000000011")
        .send()
        .await
        .expect("deleted");

    assert!(resp.data.deleted);
}

#[test]
fn a_draft_id_is_one_to_nineteen_digits() {
    assert_eq!(validate_draft_id("1850000000000000011"), Ok(()));
    assert_eq!(validate_draft_id("1"), Ok(()));
    for bad in ["", "draft-1", "12a", "18500000000000000110"] {
        assert_eq!(validate_draft_id(bad), Err("invalid-draft-id"), "{bad:?}");
    }
}
