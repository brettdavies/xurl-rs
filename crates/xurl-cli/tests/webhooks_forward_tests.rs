//! `xr webhooks listen --forward` POSTs every verified event to the caller's
//! URL after X has its answer, in the order X delivered them, and reports
//! each delivery on stderr while stdout keeps one line per event. A URL or
//! header it could not send to is refused before anything binds, and a
//! shutdown drains what is queued under one deadline.
//!
//! Every forward target is a local mock; nothing here reaches X.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::listen::{
    EVENT, EVENT_UNDER_CLIENT_SECRET, OAUTH2_HEADER, crc_token_from, deliver, free_port, listen_as,
    listen_while, post, stderr_records,
};
use common::{oauth1_store, run_in_process as run};

fn header<'a>(request: &'a wiremock::Request, name: &str) -> Option<&'a str> {
    request
        .headers
        .get(name)
        .and_then(|value| value.to_str().ok())
}

async fn answering(status: u16) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/hook"))
        .respond_with(ResponseTemplate::new(status))
        .mount(&server)
        .await;
    server
}

fn hook(server: &MockServer) -> String {
    format!("{}/hook", server.uri())
}

fn port_is_free(port: u16) -> bool {
    std::net::TcpStream::connect(("127.0.0.1", port)).is_err()
}

#[tokio::test]
async fn a_forward_delivers_the_signed_event_byte_for_byte_before_listen_exits() {
    let server = answering(200).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();
    let url = hook(&server);

    let ((code, stdout, stderr), ()) = listen_while(
        &store,
        port,
        &["--max-events", "1", "--forward", &url],
        async {
            crc_token_from(port).await;
            let status = deliver(port, Some((OAUTH2_HEADER, EVENT_UNDER_CLIENT_SECRET))).await;
            assert_eq!(status, 200);
        },
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let seen = server.received_requests().await.expect("recorded");
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].body, EVENT.as_bytes(), "X's bytes");
    assert_eq!(
        header(&seen[0], OAUTH2_HEADER),
        Some(EVENT_UNDER_CLIENT_SECRET)
    );
    assert_eq!(stdout.lines().count(), 2, "stdout is unchanged: {stdout}");
    assert_eq!(stdout.lines().nth(1), Some(EVENT));
    let records = stderr_records(&stderr);
    assert_eq!(records.len(), 1, "one record per delivery: {stderr}");
    let record = &records[0];
    assert_eq!(record["status"], "forwarded", "{record}");
    assert_eq!(record["target"], "url");
    assert_eq!(record["attempts"], 1);
    assert_eq!(record["http_status"], 200);
    assert!(record["elapsed_ms"].is_u64(), "{record}");
    assert!(
        record["received_at"]
            .as_str()
            .is_some_and(|at| at.ends_with('Z')),
        "{record}"
    );
    let key = header(&seen[0], "idempotency-key").expect("the key");
    assert_eq!(
        record["event_sha256"].as_str(),
        Some(key.trim_matches('"')),
        "the record and the forward name the event by the same key"
    );
}

#[tokio::test]
async fn events_reach_the_target_in_the_order_x_delivered_them() {
    let server = answering(200).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();
    let url = hook(&server);
    let flags = ["--allow-unsigned", "--max-events", "3", "--forward", &url];

    let ((code, _stdout, stderr), ()) = listen_while(&store, port, &flags, async {
        crc_token_from(port).await;
        for n in 1..=3 {
            assert_eq!(post(port, &format!(r#"{{"n":{n}}}"#), &[]).await, 200);
        }
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let bodies: Vec<String> = server
        .received_requests()
        .await
        .expect("recorded")
        .iter()
        .map(|request| String::from_utf8_lossy(&request.body).into_owned())
        .collect();
    assert_eq!(bodies, [r#"{"n":1}"#, r#"{"n":2}"#, r#"{"n":3}"#]);
}

#[cfg(unix)]
#[tokio::test]
async fn the_printed_test_command_reaches_the_target_signed() {
    use common::listen::{listen_tapped, next_record};

    let server = answering(200).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();
    let url = hook(&server);

    let (code, stderr, ()) = listen_tapped(
        &store,
        port,
        &["--max-events", "1", "--forward", &url],
        |mut lines| async move {
            let listening = next_record(&mut lines).await;
            let command = listening["test_command"]
                .as_str()
                .expect("a test command")
                .to_string();
            let shell = tokio::task::spawn_blocking(move || {
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .output()
                    .expect("sh runs")
            })
            .await
            .expect("the shell finishes");
            assert!(shell.status.success(), "{shell:?}");
            next_record(&mut lines).await;
        },
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let seen = server.received_requests().await.expect("recorded");
    assert_eq!(seen.len(), 1);
    let body: serde_json::Value = serde_json::from_slice(&seen[0].body).expect("JSON");
    assert_eq!(body["xr_test_event"], true);
    assert!(
        header(&seen[0], OAUTH2_HEADER).is_some_and(|value| value.starts_with("sha256=")),
        "the sample arrives signed"
    );
}

/// A target that answers 500 four times and then 200: the first event's
/// forward fails after its budget, the second is delivered, and neither
/// changes what X is answered, what stdout holds, or how `listen` exits.
#[tokio::test]
async fn a_failed_forward_is_reported_on_stderr_and_changes_nothing_else() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(4)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();
    let url = hook(&server);
    let flags = ["--allow-unsigned", "--max-events", "2", "--forward", &url];

    let ((code, stdout, stderr), ()) = listen_as(&store, "jsonl", port, &flags, async {
        crc_token_from(port).await;
        assert_eq!(
            deliver(port, Some((OAUTH2_HEADER, EVENT_UNDER_CLIENT_SECRET))).await,
            200
        );
        assert_eq!(post(port, r#"{"n":2}"#, &[]).await, 200);
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "the listening line and one per event: {stdout}"
    );
    assert_eq!(lines[1], EVENT);
    assert_eq!(lines[2], r#"{"n":2}"#);

    let records = stderr_records(&stderr);
    assert_eq!(records.len(), 2, "{stderr}");
    let failed = &records[0];
    assert_eq!(failed["status"], "forward-failed", "{failed}");
    assert_eq!(failed["target"], "url");
    assert_eq!(failed["attempts"], 4);
    assert!(
        failed["cause"]
            .as_str()
            .is_some_and(|cause| cause.contains("500")),
        "{failed}"
    );
    assert_eq!(failed["side_effects"], "printed, not delivered");
    let seen = server.received_requests().await.expect("recorded");
    assert_eq!(seen.len(), 5);
    let key = header(&seen[0], "idempotency-key").expect("the key");
    assert_eq!(failed["event_sha256"].as_str(), Some(key.trim_matches('"')));
    assert_eq!(failed["next_step"]["action"], "run-command");
    assert!(failed["next_step"].get("command").is_none(), "{failed}");
    let template = failed["next_step"]["template"]
        .as_str()
        .expect("a template");
    let window =
        regex::Regex::new(r"^xr webhooks replay <WEBHOOK_ID> --from (\d{12}) --to (\d{12})$")
            .unwrap();
    let found = window.captures(template).expect("the replay template");
    let received_at = failed["received_at"].as_str().expect("the arrival time");
    let minute: String = received_at[..16]
        .chars()
        .filter(char::is_ascii_digit)
        .collect();
    assert_eq!(&found[1], minute, "from is the arrival minute: {template}");
    assert!(found[2] > found[1], "to is the minute after: {template}");

    let forwarded = &records[1];
    assert_eq!(forwarded["status"], "forwarded", "{forwarded}");
    assert_eq!(forwarded["http_status"], 200);

    let mock = server.uri();
    let mock_address = mock.trim_start_matches("http://");
    assert!(
        !stderr.contains(mock_address),
        "no record names the URL: {stderr}"
    );
}

#[tokio::test]
async fn an_unsigned_event_is_forwarded_with_no_signature_and_its_own_content_type() {
    let server = answering(200).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();
    let url = hook(&server);
    let flags = ["--allow-unsigned", "--max-events", "1", "--forward", &url];

    let ((code, _stdout, stderr), ()) = listen_while(&store, port, &flags, async {
        crc_token_from(port).await;
        let status = post(
            port,
            "plain text",
            &[("content-type", "text/plain; charset=utf-8")],
        )
        .await;
        assert_eq!(status, 200);
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let seen = server.received_requests().await.expect("recorded");
    assert_eq!(seen.len(), 1);
    assert_eq!(
        header(&seen[0], "content-type"),
        Some("text/plain; charset=utf-8")
    );
    assert_eq!(header(&seen[0], OAUTH2_HEADER), None);
    assert_eq!(header(&seen[0], "x-twitter-webhooks-signature"), None);
}

#[tokio::test]
async fn a_header_without_a_forward_target_is_a_usage_error() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        "http://127.0.0.1:9",
        &["--output", "json", "webhooks", "listen", "-H", "X: y"],
    )
    .await;

    assert_eq!(code, 2, "stderr: {stderr}");
    let envelope: serde_json::Value = serde_json::from_str(stderr.trim()).expect("an envelope");
    assert_eq!(envelope["reason"], "invalid-args");
}

#[rstest::rstest]
#[case::not_http("ftp://127.0.0.1/hook")]
#[case::credentials("https://user:s3cr3t@127.0.0.1:9/x")]
#[tokio::test]
async fn a_forward_url_it_cannot_send_to_is_refused_before_anything_binds(#[case] url: &str) {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();

    let (code, stdout, stderr) = run(
        &store,
        "http://127.0.0.1:9",
        &[
            "--output",
            "json",
            "webhooks",
            "listen",
            "--port",
            &port.to_string(),
            "--forward",
            url,
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    assert_eq!(stdout, "", "nothing bound, so no listening line");
    let envelope: serde_json::Value = serde_json::from_str(stderr.trim()).expect("an envelope");
    assert_eq!(envelope["reason"], "validation", "{envelope}");
    assert_eq!(envelope["next_step"]["action"], "show-help");
    assert!(!stderr.contains("s3cr3t"), "{stderr}");
    if url.contains('@') {
        assert!(
            envelope["message"]
                .as_str()
                .is_some_and(|m| m.contains("-H")),
            "the refusal names -H: {envelope}"
        );
    }
    assert!(port_is_free(port));
}

#[tokio::test]
async fn a_malformed_header_is_refused_by_position_and_never_echoed() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();

    let (code, stdout, stderr) = run(
        &store,
        "http://127.0.0.1:9",
        &[
            "--output",
            "json",
            "webhooks",
            "listen",
            "--port",
            &port.to_string(),
            "--forward",
            "http://127.0.0.1:9/hook",
            "-H",
            "Authorization Bearer abc",
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope: serde_json::Value = serde_json::from_str(stderr.trim()).expect("an envelope");
    assert_eq!(envelope["reason"], "validation", "{envelope}");
    let message = envelope["message"].as_str().expect("a message");
    assert!(
        message.contains("-H") && message.contains("header 1"),
        "{message}"
    );
    assert!(
        !stdout.contains("abc") && !stderr.contains("abc"),
        "{stderr}"
    );
    assert!(port_is_free(port));
}

#[tokio::test]
async fn a_dry_run_names_the_target_and_the_header_names_and_binds_nothing() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let port = free_port();

    let (code, stdout, stderr) = run(
        &store,
        "http://127.0.0.1:9",
        &[
            "--dry-run",
            "--output",
            "json",
            "webhooks",
            "listen",
            "--port",
            &port.to_string(),
            "--forward",
            "http://127.0.0.1:9/hook",
            "-H",
            "Authorization: Bearer abc",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON");
    assert_eq!(body["status"], "dry_run");
    assert_eq!(body["would_succeed"], true);
    assert_eq!(body["forward"]["url"], "http://127.0.0.1:9/hook");
    assert_eq!(
        body["forward"]["headers"],
        serde_json::json!(["Authorization"])
    );
    assert_eq!(body["forward"]["timeout_secs"], 30);
    assert!(
        !stdout.contains("abc") && !stderr.contains("abc"),
        "{stdout}"
    );
    assert!(port_is_free(port));
}

// ── shutdown, in a real process ──────────────────────────────────────

#[cfg(unix)]
mod shutdown {
    use std::time::{Duration, Instant};

    use super::*;
    use common::listen::ListenProcess;

    async fn never_answering() -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(60)))
            .mount(&server)
            .await;
        server
    }

    async fn deliver_three(port: u16) {
        for n in 1..=3 {
            assert_eq!(post(port, &format!(r#"{{"n":{n}}}"#), &[]).await, 200);
        }
    }

    fn incomplete(stderr: &str) -> serde_json::Value {
        stderr_records(stderr)
            .into_iter()
            .find(|record| record["status"] == "forward-incomplete")
            .unwrap_or_else(|| panic!("no forward-incomplete record: {stderr}"))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn sigint_delivers_what_is_queued_behind_a_slow_target() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(500)))
            .mount(&server)
            .await;
        let tmp = TempDir::new().expect("tempdir");
        let store = oauth1_store(tmp.path());
        let port = free_port();
        let url = hook(&server);
        let listen = ListenProcess::start(&store, port, &["--allow-unsigned", "--forward", &url]);

        assert_eq!(post(port, r#"{"n":1}"#, &[]).await, 200);
        assert_eq!(post(port, r#"{"n":2}"#, &[]).await, 200);
        listen.next_line();
        listen.next_line();
        listen.interrupt();
        let (status, stderr) = listen.wait(Duration::from_secs(10));

        assert_eq!(status.code(), Some(0), "stderr: {stderr}");
        let forwarded = stderr_records(&stderr)
            .iter()
            .filter(|record| record["status"] == "forwarded")
            .count();
        assert_eq!(forwarded, 2, "{stderr}");
        assert_eq!(server.received_requests().await.expect("recorded").len(), 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn sigint_ends_the_drain_at_its_deadline_and_counts_what_was_left() {
        let server = never_answering().await;
        let tmp = TempDir::new().expect("tempdir");
        let store = oauth1_store(tmp.path());
        let port = free_port();
        let url = hook(&server);
        let flags = [
            "--allow-unsigned",
            "--forward",
            &url,
            "--forward-timeout",
            "1",
        ];
        let listen = ListenProcess::start(&store, port, &flags);

        deliver_three(port).await;
        for _ in 0..3 {
            listen.next_line();
        }
        let stopped = Instant::now();
        listen.interrupt();
        let (status, stderr) = listen.wait(Duration::from_secs(20));

        let took = stopped.elapsed();
        assert_eq!(status.code(), Some(0), "stderr: {stderr}");
        assert!(
            took >= Duration::from_secs(7) && took < Duration::from_secs(11),
            "the drain runs one attempt timeout plus 7 s of waits: {took:?}"
        );
        let record = incomplete(&stderr);
        assert_eq!(record["target"], "url");
        assert_eq!(record["undelivered"], 3, "{record}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_second_sigint_ends_the_drain_at_once() {
        let server = never_answering().await;
        let tmp = TempDir::new().expect("tempdir");
        let store = oauth1_store(tmp.path());
        let port = free_port();
        let url = hook(&server);
        let flags = [
            "--allow-unsigned",
            "--forward",
            &url,
            "--forward-timeout",
            "1",
        ];
        let listen = ListenProcess::start(&store, port, &flags);

        deliver_three(port).await;
        for _ in 0..3 {
            listen.next_line();
        }
        let stopped = Instant::now();
        listen.interrupt();
        tokio::time::sleep(Duration::from_millis(500)).await;
        listen.interrupt();
        let (status, stderr) = listen.wait(Duration::from_secs(20));

        assert!(
            stopped.elapsed() < Duration::from_secs(3),
            "the second signal ends the drain: {:?}",
            stopped.elapsed()
        );
        assert_eq!(status.code(), Some(0), "stderr: {stderr}");
        assert_eq!(incomplete(&stderr)["undelivered"], 3);
    }
}
