//! A forward sends a verified event to the caller's endpoint with X's bytes,
//! X's signature, and a key derived from the bytes, retries what another
//! attempt can fix within a bounded budget, and refuses a target it could
//! not send to before it sends anything.
//!
//! Every target is a local mock or a local socket; nothing here reaches X.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::webhooks::{
    Attempt, Event, ForwardFailure, OAUTH2_SIGNATURE_HEADER, Receiver, ReceiverConfig, RetryBudget,
    SignedSample, SigningSecrets, UrlTarget,
};

const CLIENT_SECRET: &str = "client-secret";

fn secrets() -> SigningSecrets {
    SigningSecrets {
        oauth2_client_secret: Some(CLIENT_SECRET.to_string()),
        oauth1_consumer_secret: None,
    }
}

/// A signed event, as a `Receiver` yields one from a delivery.
async fn signed_event() -> Event {
    let sample = SignedSample::new(&secrets()).expect("a sample");
    let config = ReceiverConfig::new("127.0.0.1:0".parse().expect("an address"), secrets());
    let mut receiver = Receiver::bind(config, CancellationToken::new())
        .await
        .expect("the receiver binds");
    let mut post = reqwest::Client::new()
        .post(format!("http://{}/webhook", receiver.local_addr()))
        .body(sample.body);
    for (name, value) in sample.headers() {
        post = post.header(name, value);
    }
    post.send().await.expect("the receiver answers");
    receiver.next_event().await.expect("the event")
}

/// A budget whose waits are short enough for a test: 10, 20, then 40 ms.
fn quick(attempt_timeout: Duration) -> RetryBudget {
    RetryBudget::new(attempt_timeout).with_first_wait(Duration::from_millis(10))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn header<'a>(request: &'a wiremock::Request, name: &str) -> Option<&'a str> {
    request
        .headers
        .get(name)
        .and_then(|value| value.to_str().ok())
}

async fn target_at(server: &MockServer, headers: &[&str]) -> UrlTarget {
    UrlTarget::new(&format!("{}/hook", server.uri()), headers).expect("a valid target")
}

#[tokio::test]
async fn a_forward_posts_the_event_bytes_with_xs_headers_and_the_callers() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/hook"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let event = signed_event().await;
    let target = target_at(&server, &["X-Team: blue", "Authorization: Bearer abc"]).await;

    let outcome = target.forward(&event, &quick(Duration::from_secs(5))).await;

    assert_eq!(outcome.result, Ok(200));
    assert_eq!(outcome.attempts, 1);
    let seen = server.received_requests().await.expect("recorded");
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].body, event.body, "the body is X's bytes");
    let sample = SignedSample::new(&secrets()).expect("a sample");
    assert_eq!(
        header(&seen[0], OAUTH2_SIGNATURE_HEADER),
        Some(sample.signature.as_str())
    );
    assert_eq!(header(&seen[0], "content-type"), Some("application/json"));
    assert_eq!(header(&seen[0], "x-team"), Some("blue"));
    assert_eq!(header(&seen[0], "authorization"), Some("Bearer abc"));
}

#[tokio::test]
async fn the_idempotency_key_is_the_quoted_sha256_of_the_bytes_on_every_attempt() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let event = signed_event().await;
    let target = target_at(&server, &[]).await;

    let outcome = target.forward(&event, &quick(Duration::from_secs(5))).await;

    assert_eq!(outcome.result, Ok(200));
    let expected = format!("\"{}\"", sha256_hex(&event.body));
    assert_eq!(event.sha256(), sha256_hex(&event.body));
    let seen = server.received_requests().await.expect("recorded");
    assert_eq!(seen.len(), 2);
    for request in &seen {
        assert_eq!(
            header(request, "idempotency-key"),
            Some(expected.as_str()),
            "a retry sends the same key"
        );
    }
}

#[tokio::test]
async fn a_5xx_is_retried_until_the_target_answers_2xx() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(2)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let event = signed_event().await;
    let target = target_at(&server, &[]).await;

    let outcome = target.forward(&event, &quick(Duration::from_secs(5))).await;

    assert_eq!(outcome.result, Ok(200));
    assert_eq!(outcome.attempts, 3);
}

#[tokio::test]
async fn a_4xx_other_than_408_and_429_is_final() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(400))
        .mount(&server)
        .await;
    let event = signed_event().await;
    let target = target_at(&server, &[]).await;

    let outcome = target.forward(&event, &quick(Duration::from_secs(5))).await;

    assert_eq!(outcome.result, Err(ForwardFailure::Status(400)));
    assert_eq!(outcome.attempts, 1);
    assert!(outcome.result.unwrap_err().to_string().contains("400"));
}

#[rstest::rstest]
#[case(302)]
#[case(307)]
#[case(308)]
#[tokio::test]
async fn a_redirect_is_a_final_failure_that_follows_nowhere(#[case] status: u16) {
    let elsewhere = MockServer::start().await;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(status)
                .insert_header("location", format!("{}/moved", elsewhere.uri())),
        )
        .mount(&server)
        .await;
    let event = signed_event().await;
    let target = target_at(&server, &[]).await;

    let outcome = target.forward(&event, &quick(Duration::from_secs(5))).await;

    assert_eq!(outcome.result, Err(ForwardFailure::Status(status)));
    assert_eq!(outcome.attempts, 1);
    let shown = format!("{outcome:?} {}", outcome.result.clone().unwrap_err());
    assert!(!shown.contains(&elsewhere.uri()), "{shown}");
    assert!(!shown.contains("/moved"), "{shown}");
    assert_eq!(
        elsewhere.received_requests().await.expect("recorded").len(),
        0,
        "the redirect is not followed"
    );
}

#[tokio::test]
async fn a_204_is_delivered() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    let event = signed_event().await;
    let target = target_at(&server, &[]).await;

    let outcome = target.forward(&event, &quick(Duration::from_secs(5))).await;

    assert_eq!(outcome.result, Ok(204));
}

#[tokio::test]
async fn a_target_that_never_answers_is_cut_at_the_timeout_on_every_attempt() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(30)))
        .mount(&server)
        .await;
    let event = signed_event().await;
    let target = target_at(&server, &[]).await;
    let timeout = Duration::from_millis(200);

    let started = Instant::now();
    let outcome = target.forward(&event, &quick(timeout)).await;

    assert_eq!(outcome.result, Err(ForwardFailure::TimedOut(timeout)));
    assert_eq!(outcome.attempts, 4);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
}

fn closed_port() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a port");
    let address = listener.local_addr().expect("its address");
    drop(listener);
    address
}

#[tokio::test]
async fn a_closed_port_is_retried_and_its_cause_never_names_the_url() {
    let address = closed_port();
    let url = format!("http://{address}/hook");
    let event = signed_event().await;
    let target = UrlTarget::new(&url, &[] as &[&str]).expect("a valid target");

    let outcome = target.forward(&event, &quick(Duration::from_secs(5))).await;

    assert_eq!(outcome.attempts, 4);
    let Err(ForwardFailure::Connection(cause)) = &outcome.result else {
        panic!("a connection failure: {outcome:?}");
    };
    assert!(!cause.is_empty());
    let shown = format!("{outcome:?} {}", outcome.result.clone().unwrap_err());
    assert!(!shown.contains(&url), "{shown}");
    assert!(!shown.contains(&address.to_string()), "{shown}");
}

#[rstest::rstest]
#[case::not_http("ftp://127.0.0.1/hook")]
#[case::not_a_url("127.0.0.1/hook")]
#[case::credentials("http://user:s3cr3t@127.0.0.1:9/hook")]
fn a_url_that_cannot_be_forwarded_to_is_refused_before_any_request(#[case] url: &str) {
    let err = UrlTarget::new(url, &[] as &[&str]).expect_err("refused");
    assert_eq!(err.kind(), "validation", "{err}");
    assert!(!err.to_string().contains("s3cr3t"), "{err}");
}

#[rstest::rstest]
#[case::host("Host: elsewhere.example")]
#[case::length("Content-Length: 1")]
#[case::key("Idempotency-Key: x")]
fn a_header_the_forward_sets_itself_is_refused(#[case] given: &str) {
    let err = UrlTarget::new("http://127.0.0.1:9/hook", &[given]).expect_err("refused");
    assert_eq!(err.kind(), "validation", "{err}");
}

#[rstest::rstest]
#[case::no_colon("X-Token abc")]
#[case::empty_name(": abc")]
#[case::space_in_name("X Token: abc")]
#[case::control_in_value("X-Token: a\u{7}bc")]
fn a_malformed_header_is_refused_by_position_never_by_value(#[case] given: &str) {
    let err = UrlTarget::new("http://127.0.0.1:9/hook", &["X-Ok: 1", given]).expect_err("refused");
    assert_eq!(err.kind(), "validation", "{err}");
    let message = err.to_string();
    assert!(message.contains("header 2"), "{message}");
    assert!(!message.contains("abc"), "{message}");
}

/// A server that answers 200 and then streams a chunked body that never
/// ends. A forward that read the body to its end would never finish.
async fn endless_body_server() -> SocketAddr {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a port");
    let address = listener.local_addr().expect("its address");
    tokio::spawn(async move {
        let Ok((mut stream, _)) = listener.accept().await else {
            return;
        };
        let mut request = [0_u8; 4096];
        let _ = stream.read(&mut request).await;
        let head = "HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n";
        if stream.write_all(head.as_bytes()).await.is_err() {
            return;
        }
        let chunk = format!("{:x}\r\n{}\r\n", 8192, "x".repeat(8192));
        while stream.write_all(chunk.as_bytes()).await.is_ok() {}
    });
    address
}

#[tokio::test]
async fn a_response_body_is_read_no_further_than_the_cap() {
    let address = endless_body_server().await;
    let event = signed_event().await;
    let target =
        UrlTarget::new(&format!("http://{address}/hook"), &[] as &[&str]).expect("a valid target");

    let started = Instant::now();
    let outcome = target
        .forward(&event, &quick(Duration::from_secs(10)))
        .await;

    assert_eq!(outcome.result, Ok(200));
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the endless body was read past the cap: {:?}",
        started.elapsed()
    );
}

// ── the budget itself, on a paused clock ─────────────────────────────

#[tokio::test(start_paused = true)]
async fn the_default_budget_tries_four_times_waiting_one_two_then_four_seconds() {
    let budget = RetryBudget::new(Duration::from_secs(30));

    let outcome = budget
        .run(|| async { Attempt::<(), _>::Retryable(ForwardFailure::Status(503)) })
        .await;

    assert_eq!(outcome.attempts, 4);
    assert_eq!(outcome.result, Err(ForwardFailure::Status(503)));
    assert_eq!(outcome.elapsed, Duration::from_secs(7));
    assert_eq!(budget.total_wait(), Duration::from_secs(7));
}

#[tokio::test(start_paused = true)]
async fn an_attempt_that_never_finishes_is_cut_at_the_attempt_timeout() {
    let timeout = Duration::from_secs(30);

    let outcome = RetryBudget::new(timeout)
        .run(std::future::pending::<Attempt<(), ForwardFailure>>)
        .await;

    assert_eq!(outcome.attempts, 4);
    assert_eq!(outcome.result, Err(ForwardFailure::TimedOut(timeout)));
    assert_eq!(outcome.elapsed, Duration::from_secs(4 * 30 + 7));
}

#[tokio::test(start_paused = true)]
async fn a_final_failure_ends_the_budget_at_once() {
    let outcome = RetryBudget::new(Duration::from_secs(30))
        .run(|| async { Attempt::<(), _>::Final(ForwardFailure::Status(400)) })
        .await;

    assert_eq!(outcome.attempts, 1);
    assert_eq!(outcome.elapsed, Duration::ZERO);
}
