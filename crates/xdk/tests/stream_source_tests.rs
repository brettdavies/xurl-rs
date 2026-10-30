//! The broadcast stream-source shortcuts send the requests the spec gives
//! their endpoints and decode their responses; the offline validators hold
//! the spec's bounds on a source's name and region.

use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::shortcuts::{validate_source_name, validate_source_region};
use xdk::auth::OAuth2Credential;

const SOURCE_ID: &str = "1x2y3z4w5v6u";

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

/// A source as X describes one.
fn source(name: &str) -> serde_json::Value {
    json!({
        "id": SOURCE_ID,
        "name": name,
        "region": "pdxa",
        "rtmp_url": "rtmp://va.pscp.tv:80/x",
        "rtmps_url": "rtmps://va.pscp.tv:443/x",
        "stream_key": SOURCE_ID,
        "is_stream_active": false,
        "created_at_ms": "1761148800000"
    })
}

#[tokio::test]
async fn get_stream_sources_lists_the_accounts_sources() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/sources"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data": [source("Studio A")], "meta": {"result_count": 1}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .get_stream_sources()
        .send()
        .await
        .expect("listed");

    assert_eq!(resp.data.len(), 1);
    let listed = &resp.data[0];
    assert_eq!(listed.id, SOURCE_ID);
    assert_eq!(listed.name.as_deref(), Some("Studio A"));
    assert_eq!(listed.region.as_deref(), Some("pdxa"));
    assert_eq!(listed.stream_key.as_deref(), Some(SOURCE_ID));
    assert_eq!(listed.created_at_ms.as_deref(), Some("1761148800000"));
    assert_eq!(listed.is_stream_active, Some(false));
}

#[tokio::test]
async fn create_stream_source_posts_the_name_and_region() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/sources"))
        .and(body_json(
            json!({"name": "Studio A", "region": "us-east-1"}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"data": source("Studio A")})))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .create_stream_source("Studio A", "us-east-1")
        .send()
        .await
        .expect("created");

    assert_eq!(resp.data.id, SOURCE_ID);
    assert_eq!(
        resp.data.rtmps_url.as_deref(),
        Some("rtmps://va.pscp.tv:443/x")
    );
}

#[tokio::test]
async fn get_stream_source_gets_the_source() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/sources/{SOURCE_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": source("Studio A")})))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .get_stream_source(SOURCE_ID)
        .send()
        .await
        .expect("fetched");

    assert_eq!(resp.data.name.as_deref(), Some("Studio A"));
}

#[tokio::test]
async fn rename_stream_source_puts_the_new_name() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/sources/{SOURCE_ID}")))
        .and(body_json(json!({"name": "Studio B"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": source("Studio B")})))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .rename_stream_source(SOURCE_ID, "Studio B")
        .send()
        .await
        .expect("renamed");

    assert_eq!(resp.data.name.as_deref(), Some("Studio B"));
}

#[tokio::test]
async fn delete_stream_source_deletes_and_reads_the_answer() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/sources/{SOURCE_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"deleted": true}})))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .delete_stream_source(SOURCE_ID)
        .send()
        .await
        .expect("deleted");

    assert!(resp.data.deleted);
}

#[test]
fn a_source_name_and_region_must_not_be_empty() {
    assert_eq!(validate_source_name("Studio A"), Ok(()));
    assert_eq!(validate_source_name(""), Err("empty-source-name"));
    assert_eq!(validate_source_name("   "), Err("empty-source-name"));
    assert_eq!(validate_source_region("eu-west-1"), Ok(()));
    assert_eq!(validate_source_region(""), Err("empty-source-region"));
}
