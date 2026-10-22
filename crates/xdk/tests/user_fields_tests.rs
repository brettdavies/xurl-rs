//! The signed-in account's own lookup asks X for `active_follower_count`,
//! which X returns only for the authenticated user, and the typed `User`
//! carries it.

use serde_json::json;
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

#[tokio::test]
async fn get_me_asks_for_and_reads_active_follower_count() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "id": "2244994945",
                "name": "Example",
                "username": "example",
                "active_follower_count": 1234
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let me = user_client(server.uri())
        .get_me()
        .send()
        .await
        .expect("the lookup succeeds");

    assert_eq!(me.data.active_follower_count, Some(1234));
    let sent = server.received_requests().await.expect("recorded");
    let fields: Vec<String> = sent[0]
        .url
        .query_pairs()
        .filter(|(name, _)| name == "user.fields")
        .flat_map(|(_, value)| value.split(',').map(str::to_string).collect::<Vec<_>>())
        .collect();
    assert!(
        fields.iter().any(|field| field == "active_follower_count"),
        "user.fields = {fields:?}"
    );
}

#[tokio::test]
async fn a_lookup_of_another_user_does_not_ask_for_it() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by/username/someone"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"id": "1", "name": "Someone", "username": "someone"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let user = user_client(server.uri())
        .lookup_user("someone")
        .send()
        .await
        .expect("the lookup succeeds");

    assert_eq!(user.data.active_follower_count, None);
    let sent = server.received_requests().await.expect("recorded");
    assert!(
        !sent[0]
            .url
            .query()
            .unwrap_or_default()
            .contains("active_follower_count"),
        "X returns the field only for the signed-in account: {:?}",
        sent[0].url.query()
    );
}
