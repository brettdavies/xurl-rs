//! A stream that a signal ends writes one `cancelled` document on stdout,
//! after the records it already printed, and its `reason` names the signal:
//! `sigint` for Ctrl+C, `sigterm` for a terminate.
//!
//! Only a real process shows it: the handlers belong to the binary, and a
//! mock server cannot hold a response body open, so a socket the test owns
//! sends one record and keeps the connection until `xr` closes it.
#![cfg(unix)]

mod common;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::Stdio;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use rstest::rstest;

const RECORD: &str = r#"{"data":{"id":"1"}}"#;

/// Answers the first request with a chunked body that holds one record and
/// never ends, then waits until the client closes the connection.
fn hold_a_stream_open(listener: TcpListener) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("xr connects");
        let mut head = Vec::new();
        let mut byte = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            socket.read_exact(&mut byte).expect("the request head");
            head.push(byte[0]);
        }
        let line = format!("{RECORD}\n");
        let chunk = format!("{:x}\r\n{line}\r\n", line.len());
        socket
            .write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                     Transfer-Encoding: chunked\r\n\r\n{chunk}"
                )
                .as_bytes(),
            )
            .expect("the response head and first record");
        socket.flush().expect("flush");
        let mut rest = Vec::new();
        let _ = socket.read_to_end(&mut rest);
    })
}

#[rstest]
#[case::ctrl_c("INT", "sigint")]
#[case::terminate("TERM", "sigterm")]
fn a_signal_ends_a_stream_with_a_document_that_names_it(
    #[case] signal: &str,
    #[case] reason: &str,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let origin = format!("http://{}", listener.local_addr().expect("its address"));
    let server = hold_a_stream_open(listener);

    let mut child = common::xr_std()
        .env("API_BASE_URL", &origin)
        .env("XURL_BEARER_TOKEN", "env-bearer-value")
        .args([
            "--output",
            "jsonl",
            "--auth",
            "app",
            "-s",
            "/2/tweets/search/stream",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn xr");
    let stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    let (lines, records) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if lines.send(line).is_err() {
                break;
            }
        }
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text);
        text
    });

    // The handlers register once the stream is being read; a signal sent
    // earlier takes the default disposition and ends the process silently.
    let first = records
        .recv_timeout(Duration::from_secs(10))
        .expect("the record reaches stdout");
    assert_eq!(first, RECORD);

    let sent = std::process::Command::new("kill")
        .args([format!("-{signal}"), child.id().to_string()])
        .status()
        .expect("kill runs");
    assert!(sent.success(), "kill -{signal} failed");

    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("xr did not end within ten seconds of SIG{signal}");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    reader.join().expect("the stdout reader");
    let stderr = stderr_reader.join().expect("the stderr reader");
    server.join().expect("the server thread");

    assert_eq!(status.code(), Some(0), "stderr: {stderr}");
    let rest: Vec<String> = records.try_iter().collect();
    assert_eq!(
        rest.len(),
        1,
        "only the cancelled document follows: {rest:?}"
    );
    let document: serde_json::Value = serde_json::from_str(&rest[0])
        .unwrap_or_else(|e| panic!("a JSON document ({e}): {rest:?}"));
    assert_eq!(
        document,
        serde_json::json!({"status": "cancelled", "reason": reason}),
        "stderr: {stderr}"
    );
}
