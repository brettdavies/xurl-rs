//! `update_broadcast` sends the request the spec gives `PUT /2/broadcasts/{id}`
//! and decodes the broadcast it answers; the offline validator holds the spec's
//! bounds on a change.

use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::shortcuts::{BroadcastChanges, CHAT_OPTION_MAX, validate_broadcast_changes};
use xdk::auth::OAuth2Credential;

const BROADCAST_ID: &str = "1YqKDqWqdPLGV";

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
async fn update_broadcast_sends_only_the_fields_it_was_given() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/broadcasts/{BROADCAST_ID}")))
        .and(body_json(json!({"title": "Second take", "chat_option": 3})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "id": BROADCAST_ID,
                "broadcast_id": BROADCAST_ID,
                "state": "RUNNING",
                "share_url": "https://x.com/i/broadcasts/1YqKDqWqdPLGV",
                "title": "Second take",
                "chat_option": 3
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let changes = BroadcastChanges {
        title: Some("Second take".into()),
        chat_option: Some(3),
        ..BroadcastChanges::default()
    };

    let resp = user_client(server.uri())
        .update_broadcast(BROADCAST_ID, &changes)
        .send()
        .await
        .expect("updated");

    assert_eq!(resp.data.title.as_deref(), Some("Second take"));
    assert_eq!(resp.data.chat_option, Some(3));
    assert_eq!(resp.data.state, "RUNNING");
}

#[tokio::test]
async fn update_broadcast_sends_an_empty_category_to_remove_it() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/broadcasts/{BROADCAST_ID}")))
        .and(body_json(json!({"live_category_id": ""})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "id": BROADCAST_ID,
                "broadcast_id": BROADCAST_ID,
                "state": "ENDED",
                "share_url": "https://x.com/i/broadcasts/1YqKDqWqdPLGV"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let changes = BroadcastChanges {
        live_category_id: Some(String::new()),
        ..BroadcastChanges::default()
    };

    let resp = user_client(server.uri())
        .update_broadcast(BROADCAST_ID, &changes)
        .send()
        .await
        .expect("updated");

    assert_eq!(resp.data.live_category_id, None);
}

#[test]
fn a_change_is_held_to_the_specs_bounds() {
    let change = |title: Option<&str>, chat: Option<u8>, category: Option<String>| {
        validate_broadcast_changes(&BroadcastChanges {
            title: title.map(str::to_string),
            chat_option: chat,
            live_category_id: category,
        })
    };

    assert_eq!(change(None, None, None), Err("no-broadcast-changes"));
    assert_eq!(change(Some(""), None, None), Err("empty-broadcast-title"));
    assert_eq!(change(Some("  "), None, None), Err("empty-broadcast-title"));
    assert_eq!(
        change(None, Some(CHAT_OPTION_MAX + 1), None),
        Err("invalid-chat-option")
    );
    assert_eq!(change(Some("Take two"), Some(0), None), Ok(()));
    assert_eq!(change(None, Some(CHAT_OPTION_MAX), None), Ok(()));
    assert_eq!(change(None, None, Some(String::new())), Ok(()));
}
