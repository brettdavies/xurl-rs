//! The chat moderator shortcuts send the three calls the spec defines and
//! decode their responses; a bearer-only client is refused before any
//! request leaves.

use serde_json::json;
use wiremock::matchers::{body_json, method, path};
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

#[tokio::test]
async fn get_chat_moderators_lists_the_users_with_their_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/broadcasts/chat/moderators"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"id": "2244994945", "name": "X Dev", "username": "XDevelopers"}],
            "meta": {"result_count": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let moderators = user_client(server.uri())
        .get_chat_moderators()
        .send()
        .await
        .expect("list succeeds");

    assert_eq!(moderators.data.len(), 1);
    assert_eq!(moderators.data[0].username, "XDevelopers");
    let requests = server.received_requests().await.expect("recorded");
    assert!(
        requests[0]
            .url
            .query()
            .unwrap_or("")
            .contains("user.fields="),
        "the list asks for user fields: {:?}",
        requests[0].url
    );
}

#[tokio::test]
async fn add_chat_moderator_posts_the_user_id_and_returns_the_moderator_set() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/broadcasts/chat/moderators"))
        .and(body_json(json!({"user_id": "2244994945"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"moderator_user_ids": ["2244994945"]}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = user_client(server.uri())
        .add_chat_moderator("2244994945")
        .send()
        .await
        .expect("add succeeds");

    assert_eq!(
        result.data.moderator_user_ids,
        vec!["2244994945".to_string()]
    );
}

#[tokio::test]
async fn remove_chat_moderator_deletes_by_user_id() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/2/broadcasts/chat/moderators/2244994945"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"moderator_user_ids": []}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = user_client(server.uri())
        .remove_chat_moderator("2244994945")
        .send()
        .await
        .expect("remove succeeds");

    assert!(result.data.moderator_user_ids.is_empty());
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
        .get_chat_moderators()
        .send()
        .await
        .expect_err("the endpoint accepts no app-only scheme");

    assert_eq!(err.kind(), "auth-method-mismatch", "{err}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
