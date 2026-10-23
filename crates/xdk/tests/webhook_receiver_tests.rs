//! The webhook receiver answers X's CRC check with the documented response
//! token, yields an event only when its signature verifies, and refuses
//! everything else with the status X's documentation gives.
//!
//! Every expected signature below was computed outside this crate
//! (`hmac.new(key, message, sha256)` in Python, base64-encoded), so the tests
//! hold the receiver to the algorithm and not to its own implementation.

use std::net::SocketAddr;
use std::time::{Duration, SystemTime};

use tokio_util::sync::CancellationToken;
use xdk::error::{Error, NextAction};
use xdk::webhooks::{Event, Receiver, ReceiverConfig, Signature, SigningSecrets};

const CLIENT_SECRET: &str = "client-secret";
const CONSUMER_SECRET: &str = "consumer-secret";
const CRC_TOKEN: &str = "challenge_string";
const CRC_UNDER_CLIENT_SECRET: &str = "sha256=bACRGErecWu2jj2NpzrZglSUmdYwRiKKEr4qwY8EfB0=";
const CRC_UNDER_CONSUMER_SECRET: &str = "sha256=S/mG8E9n5SjoGuCqHguvP8K8uuu5ogCPF0dOpIdyZlk=";
const BODY: &str = r#"{"for_user_id":"2244994945","follow_events":[]}"#;
const BODY_UNDER_CLIENT_SECRET: &str = "sha256=5Bg4ljGblYRlSUh1r3Q/d52TsG7GmNV53cCzrrquwcw=";
const BODY_UNDER_CONSUMER_SECRET: &str = "sha256=hbnk9KfP6HMZYlRvN1bQQ62z6zAyI3oqWMeXXAoQztY=";
const OAUTH2_HEADER: &str = "X-Twitter-Webhooks-Signature-OAuth2";
const LEGACY_HEADER: &str = "X-Twitter-Webhooks-Signature";

fn loopback() -> SocketAddr {
    "127.0.0.1:0".parse().expect("an address")
}

fn both_secrets() -> SigningSecrets {
    SigningSecrets {
        oauth2_client_secret: Some(CLIENT_SECRET.to_string()),
        oauth1_consumer_secret: Some(CONSUMER_SECRET.to_string()),
    }
}

async fn receiver(secrets: SigningSecrets) -> Receiver {
    Receiver::bind(
        ReceiverConfig::new(loopback(), secrets),
        CancellationToken::new(),
    )
    .await
    .expect("the receiver binds")
}

fn url(receiver: &Receiver, path_and_query: &str) -> String {
    format!("http://{}{path_and_query}", receiver.local_addr())
}

async fn nothing_was_yielded(receiver: &mut Receiver) -> bool {
    tokio::time::timeout(Duration::from_millis(100), receiver.next_event())
        .await
        .is_err()
}

fn header<'a>(event: &'a Event, name: &str) -> Option<&'a str> {
    event
        .headers
        .iter()
        .find(|(held, _)| held.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

#[tokio::test]
async fn the_crc_check_is_answered_with_the_client_secrets_token() {
    let receiver = receiver(both_secrets()).await;

    let response = reqwest::get(url(&receiver, &format!("/webhook?crc_token={CRC_TOKEN}")))
        .await
        .expect("the receiver answers");

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.expect("a JSON body");
    assert_eq!(body["response_token"], CRC_UNDER_CLIENT_SECRET);
}

#[tokio::test]
async fn an_app_with_only_a_consumer_secret_answers_the_crc_check_with_it() {
    let receiver = receiver(SigningSecrets {
        oauth2_client_secret: None,
        oauth1_consumer_secret: Some(CONSUMER_SECRET.to_string()),
    })
    .await;

    let response = reqwest::get(url(&receiver, &format!("/webhook?crc_token={CRC_TOKEN}")))
        .await
        .expect("the receiver answers");

    let body: serde_json::Value = response.json().await.expect("a JSON body");
    assert_eq!(body["response_token"], CRC_UNDER_CONSUMER_SECRET);
}

#[tokio::test]
async fn a_crc_check_without_a_token_is_a_bad_request() {
    let receiver = receiver(both_secrets()).await;

    let response = reqwest::get(url(&receiver, "/webhook"))
        .await
        .expect("the receiver answers");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn an_event_signed_with_the_client_secret_is_yielded() {
    let mut receiver = receiver(both_secrets()).await;

    let response = reqwest::Client::new()
        .post(url(&receiver, "/webhook"))
        .header(OAUTH2_HEADER, BODY_UNDER_CLIENT_SECRET)
        .body(BODY)
        .send()
        .await
        .expect("the receiver answers");

    assert_eq!(response.status(), 200);
    let event = receiver.next_event().await.expect("an event");
    assert_eq!(event.body, BODY.as_bytes());
    assert_eq!(event.signature, Signature::OAuth2);
}

#[tokio::test]
async fn an_event_signed_with_the_consumer_secret_is_yielded() {
    let mut receiver = receiver(both_secrets()).await;

    let response = reqwest::Client::new()
        .post(url(&receiver, "/webhook"))
        .header(LEGACY_HEADER, BODY_UNDER_CONSUMER_SECRET)
        .body(BODY)
        .send()
        .await
        .expect("the receiver answers");

    assert_eq!(response.status(), 200);
    let event = receiver.next_event().await.expect("an event");
    assert_eq!(event.body, BODY.as_bytes());
    assert_eq!(event.signature, Signature::OAuth1);
}

#[tokio::test]
async fn an_event_holds_its_signature_header_and_content_type_as_sent() {
    let mut receiver = receiver(both_secrets()).await;

    reqwest::Client::new()
        .post(url(&receiver, "/webhook"))
        .header(OAUTH2_HEADER, BODY_UNDER_CLIENT_SECRET)
        .header("Content-Type", "application/json; charset=utf-8")
        .body(BODY)
        .send()
        .await
        .expect("the receiver answers");

    let event = receiver.next_event().await.expect("an event");
    assert_eq!(
        header(&event, OAUTH2_HEADER),
        Some(BODY_UNDER_CLIENT_SECRET)
    );
    assert_eq!(
        header(&event, "Content-Type"),
        Some("application/json; charset=utf-8")
    );
    assert_eq!(header(&event, LEGACY_HEADER), None);
}

#[tokio::test]
async fn a_legacy_signed_event_holds_the_legacy_header_only() {
    let mut receiver = receiver(both_secrets()).await;

    reqwest::Client::new()
        .post(url(&receiver, "/webhook"))
        .header(LEGACY_HEADER, BODY_UNDER_CONSUMER_SECRET)
        .body(BODY)
        .send()
        .await
        .expect("the receiver answers");

    let event = receiver.next_event().await.expect("an event");
    assert_eq!(
        header(&event, LEGACY_HEADER),
        Some(BODY_UNDER_CONSUMER_SECRET)
    );
    assert_eq!(header(&event, OAUTH2_HEADER), None);
}

#[tokio::test]
async fn an_event_records_when_it_arrived() {
    let mut receiver = receiver(both_secrets()).await;

    let before = SystemTime::now();
    reqwest::Client::new()
        .post(url(&receiver, "/webhook"))
        .header(OAUTH2_HEADER, BODY_UNDER_CLIENT_SECRET)
        .body(BODY)
        .send()
        .await
        .expect("the receiver answers");
    let after = SystemTime::now();

    let event = receiver.next_event().await.expect("an event");
    assert!(
        before <= event.received_at && event.received_at <= after,
        "{:?} lies outside {before:?}..={after:?}",
        event.received_at
    );
}

#[tokio::test]
async fn the_oauth2_header_decides_when_both_are_sent() {
    let mut receiver = receiver(both_secrets()).await;

    let response = reqwest::Client::new()
        .post(url(&receiver, "/webhook"))
        .header(OAUTH2_HEADER, "sha256=AAAA")
        .header(LEGACY_HEADER, BODY_UNDER_CONSUMER_SECRET)
        .body(BODY)
        .send()
        .await
        .expect("the receiver answers");

    assert_eq!(response.status(), 401);
    assert!(nothing_was_yielded(&mut receiver).await);
}

#[tokio::test]
async fn an_event_whose_body_was_changed_is_refused() {
    let mut receiver = receiver(both_secrets()).await;

    let response = reqwest::Client::new()
        .post(url(&receiver, "/webhook"))
        .header(OAUTH2_HEADER, BODY_UNDER_CLIENT_SECRET)
        .body(BODY.replace("2244994945", "1"))
        .send()
        .await
        .expect("the receiver answers");

    assert_eq!(response.status(), 401);
    assert!(nothing_was_yielded(&mut receiver).await);
}

#[tokio::test]
async fn an_event_with_no_signature_is_refused() {
    let mut receiver = receiver(both_secrets()).await;

    let response = reqwest::Client::new()
        .post(url(&receiver, "/webhook"))
        .body(BODY)
        .send()
        .await
        .expect("the receiver answers");

    assert_eq!(response.status(), 401);
    assert!(nothing_was_yielded(&mut receiver).await);
}

#[tokio::test]
async fn an_unsigned_event_is_yielded_only_when_the_caller_allows_it() {
    let mut config = ReceiverConfig::new(loopback(), both_secrets());
    config.allow_unsigned = true;
    let mut receiver = Receiver::bind(config, CancellationToken::new())
        .await
        .expect("the receiver binds");

    let response = reqwest::Client::new()
        .post(url(&receiver, "/webhook"))
        .body(BODY)
        .send()
        .await
        .expect("the receiver answers");

    assert_eq!(response.status(), 200);
    let event = receiver.next_event().await.expect("an event");
    assert_eq!(event.signature, Signature::Unsigned);
}

#[tokio::test]
async fn a_wrong_signature_is_refused_even_when_unsigned_events_are_allowed() {
    let mut config = ReceiverConfig::new(loopback(), both_secrets());
    config.allow_unsigned = true;
    let mut receiver = Receiver::bind(config, CancellationToken::new())
        .await
        .expect("the receiver binds");

    let response = reqwest::Client::new()
        .post(url(&receiver, "/webhook"))
        .header(OAUTH2_HEADER, "sha256=AAAA")
        .body(BODY)
        .send()
        .await
        .expect("the receiver answers");

    assert_eq!(response.status(), 401);
    assert!(nothing_was_yielded(&mut receiver).await);
}

#[tokio::test]
async fn another_path_is_not_found_and_another_method_is_not_allowed() {
    let receiver = receiver(both_secrets()).await;
    let client = reqwest::Client::new();

    let elsewhere = client
        .get(url(&receiver, &format!("/other?crc_token={CRC_TOKEN}")))
        .send()
        .await
        .expect("the receiver answers");
    let put = client
        .put(url(&receiver, "/webhook"))
        .send()
        .await
        .expect("the receiver answers");

    assert_eq!(elsewhere.status(), 404);
    assert_eq!(put.status(), 405);
}

#[tokio::test]
async fn a_body_over_the_cap_is_refused() {
    let mut config = ReceiverConfig::new(loopback(), both_secrets());
    config.allow_unsigned = true;
    config.max_body_bytes = 16;
    let mut receiver = Receiver::bind(config, CancellationToken::new())
        .await
        .expect("the receiver binds");

    let response = reqwest::Client::new()
        .post(url(&receiver, "/webhook"))
        .body(BODY)
        .send()
        .await
        .expect("the receiver answers");

    assert_eq!(response.status(), 413);
    assert!(nothing_was_yielded(&mut receiver).await);
}

#[tokio::test]
async fn a_receiver_with_no_secret_does_not_bind() {
    let err = Receiver::bind(
        ReceiverConfig::new(loopback(), SigningSecrets::default()),
        CancellationToken::new(),
    )
    .await
    .expect_err("the CRC check cannot be answered without a secret");

    assert_eq!(err.kind(), "validation", "{err}");
}

#[tokio::test]
async fn binding_a_port_another_listener_holds_is_address_in_use() {
    let held = std::net::TcpListener::bind(loopback()).expect("a port to hold");
    let address = held.local_addr().expect("its address");

    let err = Receiver::bind(
        ReceiverConfig::new(address, both_secrets()),
        CancellationToken::new(),
    )
    .await
    .expect_err("the port is taken");

    assert_eq!(err.kind(), "address-in-use", "{err:?}");
    assert!(
        matches!(&err, Error::AddressInUse { address: named, .. } if *named == address),
        "the error names the held address: {err:?}"
    );
    assert_eq!(err.exit_code(), 5);
    assert_eq!(err.next_action(), Some(NextAction::FixInput));
    assert_eq!(err.docs_url(), None);
    let os = std::error::Error::source(&err)
        .and_then(|source| source.downcast_ref::<std::io::Error>())
        .expect("the OS error is the source");
    assert_eq!(os.kind(), std::io::ErrorKind::AddrInUse);
}

#[tokio::test]
async fn binding_an_address_the_host_does_not_have_stays_an_io_error() {
    // 192.0.2.0/24 is TEST-NET-1 (RFC 5737), assigned to no host.
    let err = Receiver::bind(
        ReceiverConfig::new("192.0.2.1:0".parse().expect("an address"), both_secrets()),
        CancellationToken::new(),
    )
    .await
    .expect_err("no interface holds that address");

    assert_eq!(err.kind(), "io", "{err:?}");
}

#[tokio::test]
async fn cancelling_the_token_ends_the_event_stream() {
    let cancel = CancellationToken::new();
    let mut receiver = Receiver::bind(
        ReceiverConfig::new(loopback(), both_secrets()),
        cancel.clone(),
    )
    .await
    .expect("the receiver binds");

    cancel.cancel();

    assert!(receiver.next_event().await.is_none());
}

#[test]
fn the_secrets_do_not_print() {
    let shown = format!("{:?}", both_secrets());
    assert!(!shown.contains(CLIENT_SECRET), "{shown}");
    assert!(!shown.contains(CONSUMER_SECRET), "{shown}");
}

#[tokio::test]
async fn a_held_connection_does_not_starve_the_receiver_forever() {
    let mut config = ReceiverConfig::new(loopback(), both_secrets());
    config.max_connections = 1;
    let receiver = Receiver::bind(config, CancellationToken::new())
        .await
        .expect("the receiver binds");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(300))
        .build()
        .expect("a client");
    let crc = url(&receiver, &format!("/webhook?crc_token={CRC_TOKEN}"));

    // One connection that sends nothing takes the only slot.
    let idle = tokio::net::TcpStream::connect(receiver.local_addr())
        .await
        .expect("the receiver accepts");
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        client.get(&crc).send().await.is_err(),
        "a second connection waits while the slot is taken"
    );

    drop(idle);
    let answered = client
        .get(&crc)
        .send()
        .await
        .expect("the slot is free again");
    assert_eq!(answered.status(), 200);
}
