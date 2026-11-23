//! A stream that a signal ends writes one `cancelled` document on stdout,
//! after the records it already printed, and its `reason` names the signal:
//! `sigint` for Ctrl+C, `sigterm` for a terminate. A signal that arrives
//! before the response headers ends it the same way.
//!
//! Only a real process shows it: the handlers belong to the binary, and a
//! mock server cannot hold a response open, so a socket the test owns sends
//! one record, or nothing, and keeps the connection until `xr` closes it.
#![cfg(unix)]

mod common;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::{Child, ExitStatus, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use rstest::rstest;

const RECORD: &str = r#"{"data":{"id":"1"}}"#;

fn read_request_head(socket: &mut impl Read) {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        socket.read_exact(&mut byte).expect("the request head");
        head.push(byte[0]);
    }
}

/// Answers the first request with a chunked body that holds one record and
/// never ends, then waits until the client closes the connection.
fn hold_a_stream_open(listener: TcpListener) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("xr connects");
        read_request_head(&mut socket);
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

/// Reads the first request and answers nothing, so `xr` waits on the response
/// headers, then waits until the client closes the connection. `requested`
/// fires once the request has been read.
fn hold_the_headers_back(
    listener: TcpListener,
    requested: mpsc::Sender<()>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("xr connects");
        read_request_head(&mut socket);
        requested.send(()).expect("the test is waiting");
        let mut rest = Vec::new();
        let _ = socket.read_to_end(&mut rest);
    })
}

/// A running `xr` on a stream, its stdout lines arriving on `records`.
struct Streaming {
    child: Child,
    records: mpsc::Receiver<String>,
    stdout_reader: std::thread::JoinHandle<()>,
    stderr_reader: std::thread::JoinHandle<String>,
}

fn stream_from(origin: &str) -> Streaming {
    let mut child = common::xr_std()
        .env("API_BASE_URL", origin)
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
    let stdout_reader = std::thread::spawn(move || {
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
    Streaming {
        child,
        records,
        stdout_reader,
        stderr_reader,
    }
}

impl Streaming {
    /// Sends `signal`, waits for `xr` to end, and returns its exit status,
    /// the stdout lines still unread, and its stderr.
    fn end_with(mut self, signal: &str) -> (ExitStatus, Vec<String>, String) {
        let sent = std::process::Command::new("kill")
            .args([format!("-{signal}"), self.child.id().to_string()])
            .status()
            .expect("kill runs");
        assert!(sent.success(), "kill -{signal} failed");

        let deadline = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(status) = self.child.try_wait().expect("try_wait") {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = self.child.kill();
                let _ = self.child.wait();
                panic!("xr did not end within ten seconds of SIG{signal}");
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        self.stdout_reader.join().expect("the stdout reader");
        let stderr = self.stderr_reader.join().expect("the stderr reader");
        (status, self.records.try_iter().collect(), stderr)
    }
}

fn assert_only_the_cancelled_document(
    (status, rest, stderr): (ExitStatus, Vec<String>, String),
    reason: &str,
) {
    assert_eq!(status.code(), Some(0), "stderr: {stderr}");
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
    let streaming = stream_from(&origin);

    // Waiting for the first record puts the signal after the response headers.
    let first = streaming
        .records
        .recv_timeout(Duration::from_secs(10))
        .expect("the record reaches stdout");
    assert_eq!(first, RECORD);

    let ended = streaming.end_with(signal);
    server.join().expect("the server thread");
    assert_only_the_cancelled_document(ended, reason);
}

#[rstest]
#[case::ctrl_c("INT", "sigint")]
#[case::terminate("TERM", "sigterm")]
fn a_signal_before_the_headers_ends_a_stream_with_the_same_document(
    #[case] signal: &str,
    #[case] reason: &str,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let origin = format!("http://{}", listener.local_addr().expect("its address"));
    let (requested, request_read) = mpsc::channel();
    let server = hold_the_headers_back(listener, requested);
    let streaming = stream_from(&origin);

    // The request is out, and no response head has come back.
    request_read
        .recv_timeout(Duration::from_secs(10))
        .expect("xr sends the request");

    let ended = streaming.end_with(signal);
    server.join().expect("the server thread");
    assert_only_the_cancelled_document(ended, reason);
}
