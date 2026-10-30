//! The shortcuts that act inside a live broadcast, which `xr` gives no
//! command: ending it and sending, removing, muting, and unmuting in its
//! chat. Each sends the request the spec defines and decodes its answer,
//! and the chat validators refuse what the spec bounds.

use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::shortcuts::{
    BROADCAST_CHAT_MAX_CHARS, validate_chat_message, validate_chat_message_id,
};
use xdk::auth::OAuth2Credential;
use xdk::error::Error;

const BROADCAST: &str = "1YqKDqWqdPLGV";
const MESSAGE: &str = "1700000000001";

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
async fn end_broadcast_puts_the_end_state() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/broadcasts/{BROADCAST}/state")))
        .and(body_json(json!({"state": "END"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"id": BROADCAST, "state": "ENDED"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let ended = user_client(server.uri())
        .end_broadcast(BROADCAST)
        .send()
        .await
        .expect("end succeeds");

    assert_eq!(ended.data.id, BROADCAST);
    assert_eq!(ended.data.state, "ENDED");
}

#[tokio::test]
async fn send_broadcast_chat_posts_the_text_and_the_message_it_replies_to() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/2/broadcasts/{BROADCAST}/chat")))
        .and(body_json(
            json!({"text": "Welcome!", "reply_to": "1700000000000"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"success": true, "timestamp": MESSAGE}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/2/broadcasts/{BROADCAST}/chat")))
        .and(body_json(json!({"text": "Hello"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"success": true, "timestamp": "1700000000002"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let client = user_client(server.uri());

    let reply = client
        .send_broadcast_chat(BROADCAST, "Welcome!", Some("1700000000000"))
        .send()
        .await
        .expect("reply succeeds");
    let message = client
        .send_broadcast_chat(BROADCAST, "Hello", None)
        .send()
        .await
        .expect("send succeeds");

    assert!(reply.data.success);
    assert_eq!(reply.data.timestamp, MESSAGE);
    assert_eq!(message.data.timestamp, "1700000000002");
}

#[tokio::test]
async fn delete_chat_message_deletes_the_message_by_id() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/broadcasts/{BROADCAST}/chat/{MESSAGE}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"deleted": true}})))
        .expect(1)
        .mount(&server)
        .await;

    let deleted = user_client(server.uri())
        .delete_chat_message(BROADCAST, MESSAGE)
        .send()
        .await
        .expect("delete succeeds");

    assert!(deleted.data.deleted);
}

#[tokio::test]
async fn mute_chat_user_posts_the_user_the_expiry_in_milliseconds_and_the_message() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/2/broadcasts/{BROADCAST}/chat/mutes")))
        .and(body_json(json!({
            "user_id": "99",
            "end_at_ms": "1793638800000",
            "message_id": MESSAGE
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"muted": true}})))
        .expect(1)
        .mount(&server)
        .await;

    let muted = user_client(server.uri())
        .mute_chat_user(BROADCAST, "99", Some("2026-11-02T17:00:00Z"), Some(MESSAGE))
        .send()
        .await
        .expect("mute succeeds");

    assert!(muted.data.muted);
}

#[tokio::test]
async fn mute_chat_user_refuses_an_expiry_outside_rfc_3339_before_any_request() {
    let server = MockServer::start().await;

    let refused = user_client(server.uri())
        .mute_chat_user(BROADCAST, "99", Some("tomorrow"), None)
        .send()
        .await
        .expect_err("the expiry is refused");

    assert!(
        matches!(&refused, Error::Validation { rule, .. } if *rule == "invalid-time-window"),
        "{refused:?}"
    );
    assert!(
        server
            .received_requests()
            .await
            .expect("recorded")
            .is_empty()
    );
}

#[tokio::test]
async fn unmute_chat_user_deletes_the_users_mute() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/broadcasts/{BROADCAST}/chat/mutes/99")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"muted": false}})))
        .expect(1)
        .mount(&server)
        .await;

    let unmuted = user_client(server.uri())
        .unmute_chat_user(BROADCAST, "99")
        .send()
        .await
        .expect("unmute succeeds");

    assert!(!unmuted.data.muted);
}

#[test]
fn a_chat_message_is_one_to_the_spec_limit_of_characters() {
    assert_eq!(validate_chat_message(""), Err("empty-body"));
    assert_eq!(
        validate_chat_message(&"é".repeat(BROADCAST_CHAT_MAX_CHARS)),
        Ok(())
    );
    assert_eq!(
        validate_chat_message(&"x".repeat(BROADCAST_CHAT_MAX_CHARS + 1)),
        Err("chat-message-too-long")
    );
}

#[test]
fn a_chat_message_id_is_one_to_nineteen_digits() {
    assert_eq!(validate_chat_message_id(MESSAGE), Ok(()));
    assert_eq!(validate_chat_message_id("1234567890123456789"), Ok(()));
    for id in ["", "msg-1", "12345678901234567890"] {
        assert_eq!(
            validate_chat_message_id(id),
            Err("invalid-chat-message-id"),
            "{id:?}"
        );
    }
}
