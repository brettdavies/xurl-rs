//! OAuth2 callback listener tests.
//!
//! Drives `xdk::auth::callback::wait_for_callback_with` on a worker thread's own runtime
//! and exercises bind, path-match, state-validation, partial-bind warning,
//! and cancellation behaviour from a client-side `tokio::net::TcpStream`.

use std::io::{Read, Write};
use std::net::{TcpListener as StdTcpListener, TcpStream as StdTcpStream};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use url::Url;

use xdk::auth::callback::wait_for_callback_with;
use xdk::error::Result as XurlResult;

/// How long a test waits for the listener to bind or to return. It is only
/// reached when something is wrong, so it is generous enough for a host
/// loaded with other test binaries.
const WAIT: Duration = Duration::from_secs(30);

/// Drives the listener future to completion on a runtime private to the
/// calling thread, so a test can poke the socket with blocking I/O from its
/// own thread while the listener runs on another.
fn block_on_listener<F: std::future::Future>(listener: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("listener runtime")
        .block_on(listener)
}

// ── Port allocation ───────────────────────────────────────────────────────
//
// The listener binds the port its URI names, so a test picks a free one and
// releases it first. Anything on the host, another test in this binary
// included, can take the port before the listener binds it, so
// `start_listener` picks again when the bind fails. The IPv6 leg is
// best-effort, because some CI environments disable [::1].

fn pick_free_port_ipv4() -> u16 {
    let l = StdTcpListener::bind("127.0.0.1:0").expect("bind 127.0.0.1:0");
    l.local_addr().unwrap().port()
}

fn pick_free_port_ipv6() -> u16 {
    let l = StdTcpListener::bind("[::1]:0").expect("bind [::1]:0");
    l.local_addr().unwrap().port()
}

fn ipv6_available() -> bool {
    StdTcpListener::bind("[::1]:0").is_ok()
}

/// A listener running on its own thread, bound and accepting.
struct Running {
    port: u16,
    result: mpsc::Receiver<XurlResult<String>>,
    thread: thread::JoinHandle<()>,
    warnings: Captured,
}

impl Running {
    /// Waits for the listener to return, and returns what it did.
    fn finish(self) -> XurlResult<String> {
        let result = self
            .result
            .recv_timeout(WAIT)
            .expect("the listener returns");
        self.thread.join().expect("the listener thread");
        result
    }
}

/// What the listener thread writes through `tracing`, kept for the test.
#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Captured {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

/// Starts the listener for `uri_for(port)` on a port `pick` chose and
/// returns once it has bound, so a request sent after this reaches it.
/// `hold` runs first with the port and returns what the test keeps for its
/// duration (a socket occupying one leg), or `None` when that port cannot be
/// set up. A port something else took between the pick and the bind is
/// picked again. Every test sends to 127.0.0.1, so a `localhost` listener
/// whose IPv4 leg did not bind (its partial-bind warning says so) is stopped
/// and started again on another port too.
fn start_listener<H>(
    pick: impl Fn() -> u16,
    uri_for: impl Fn(u16) -> Url,
    hold: impl Fn(u16) -> Option<H>,
    cancel: &CancellationToken,
    on_bound: impl Fn() + Clone + Send + 'static,
) -> (Running, H) {
    const ATTEMPTS: usize = 20;
    for _ in 0..ATTEMPTS {
        let port = pick();
        let Some(held) = hold(port) else {
            continue;
        };
        let uri = uri_for(port);
        let attempt = cancel.child_token();
        let listener_cancel = attempt.clone();
        let on_bound = on_bound.clone();
        let warnings = Captured::default();
        let writer = warnings.clone();
        let (bound_tx, bound_rx) = mpsc::channel();
        let (result_tx, result) = mpsc::channel();
        let thread = thread::spawn(move || {
            let subscriber = tracing_subscriber::fmt()
                .with_ansi(false)
                .with_writer(move || writer.clone())
                .finish();
            let res = tracing::subscriber::with_default(subscriber, || {
                block_on_listener(wait_for_callback_with(
                    &uri,
                    "STATE",
                    listener_cancel,
                    move || {
                        on_bound();
                        let _ = bound_tx.send(());
                    },
                ))
            });
            let _ = result_tx.send(res);
        });
        let running = Running {
            port,
            result,
            thread,
            warnings,
        };
        if bound_rx.recv_timeout(WAIT).is_ok() {
            if !running.warnings.text().contains("failed to bind 127.0.0.1") {
                return (running, held);
            }
            attempt.cancel();
            let _ = running.finish();
            continue;
        }
        let failed = running.finish();
        assert!(
            failed
                .as_ref()
                .is_err_and(|e| e.to_string().contains("could not bind callback listener")),
            "the listener stopped before it bound: {failed:?}"
        );
    }
    panic!("no port the listener could bind in {ATTEMPTS} picks");
}

/// [`start_listener`] on 127.0.0.1 or `localhost`, holding nothing.
fn start_ipv4(uri_for: impl Fn(u16) -> Url, cancel: &CancellationToken) -> Running {
    start_listener(pick_free_port_ipv4, uri_for, |_| Some(()), cancel, || {}).0
}

fn callback_uri(host: &str, path: &str) -> impl Fn(u16) -> Url {
    move |port| Url::parse(&format!("http://{host}:{port}{path}")).unwrap()
}

// ── HTTP helpers ──────────────────────────────────────────────────────────

/// Sends one GET to a listener [`start_listener`] reported bound.
fn send_request(addr: &str, path: &str) -> String {
    let mut stream =
        StdTcpStream::connect(addr).unwrap_or_else(|e| panic!("connect to {addr}: {e}"));
    stream.set_read_timeout(Some(WAIT)).unwrap();
    let req = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).expect("write request");
    let mut buf = String::new();
    let _ = stream.read_to_string(&mut buf);
    buf
}

fn http_status_code(response: &str) -> u16 {
    let first = response.lines().next().unwrap_or("");
    let parts: Vec<&str> = first.split_whitespace().collect();
    parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0)
}

// ── Test bodies ───────────────────────────────────────────────────────────

#[test]
fn single_explicit_ipv4_bind_delivers_code() {
    let listener = start_ipv4(
        callback_uri("127.0.0.1", "/callback"),
        &CancellationToken::new(),
    );

    let resp = send_request(
        &format!("127.0.0.1:{}", listener.port),
        "/callback?code=mycode&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 200);
    assert_eq!(listener.finish().expect("ok code"), "mycode");
}

#[test]
fn single_explicit_ipv6_bind_delivers_code() {
    if !ipv6_available() {
        eprintln!("skipping ipv6 test: [::1]:0 unavailable in this env");
        return;
    }
    let (listener, ()) = start_listener(
        pick_free_port_ipv6,
        callback_uri("[::1]", "/callback"),
        |_| Some(()),
        &CancellationToken::new(),
        || {},
    );

    let resp = send_request(
        &format!("[::1]:{}", listener.port),
        "/callback?code=mycode6&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 200);
    assert_eq!(listener.finish().expect("ok code"), "mycode6");
}

#[test]
fn dual_bind_localhost_delivers_via_ipv4() {
    // Best-effort: if [::1] at the port is occupied, this test exercises the
    // partial-bind path with a warning. The assertion only checks that the
    // code flow completes via the IPv4 leg.
    let listener = start_ipv4(
        callback_uri("localhost", "/callback"),
        &CancellationToken::new(),
    );

    let resp = send_request(
        &format!("127.0.0.1:{}", listener.port),
        "/callback?code=dualcode&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 200);
    assert_eq!(listener.finish().expect("ok code"), "dualcode");
}

#[test]
fn partial_bind_localhost_proceeds_when_ipv6_unavailable() {
    // Pre-bind [::1] at the port to force the IPv6 leg to fail; the listener
    // must proceed with the IPv4 socket. This needs an [::1] stack: without
    // one the premise (one bind fails, the other succeeds) cannot be set up,
    // and the IPv4-only success path is covered by the single-bind test.
    if !ipv6_available() {
        eprintln!("skipping partial-bind ipv4 test: [::1] unavailable in this env");
        return;
    }
    let (listener, _hog) = start_listener(
        pick_free_port_ipv4,
        callback_uri("localhost", "/callback"),
        |port| StdTcpListener::bind(("::1", port)).ok(),
        &CancellationToken::new(),
        || {},
    );

    let resp = send_request(
        &format!("127.0.0.1:{}", listener.port),
        "/callback?code=partialcode&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 200);
    let warned = listener.warnings.text();
    assert!(
        warned.contains("failed to bind [::1]"),
        "the partial bind is reported: {warned:?}"
    );
    assert_eq!(listener.finish().expect("ok code"), "partialcode");
}

#[test]
fn both_binds_fail_returns_error() {
    // Pre-bind 127.0.0.1 at a port the OS picks to make the IPv4 leg fail.
    // Use the explicit 127.0.0.1 URI to keep this test single-bind and
    // deterministic across IPv6-disabled environments.
    let hog4 = StdTcpListener::bind("127.0.0.1:0").expect("bind v4 hog");
    let port = hog4.local_addr().unwrap().port();
    let uri = Url::parse(&format!("http://127.0.0.1:{port}/callback")).unwrap();
    let cancel = CancellationToken::new();

    let res = block_on_listener(wait_for_callback_with(&uri, "STATE", cancel, || {}));
    assert!(res.is_err(), "expected bind failure error");
    let msg = res.unwrap_err().to_string();
    assert!(
        msg.contains("could not bind callback listener"),
        "unexpected error message: {msg}"
    );
    drop(hog4);
}

#[test]
fn custom_callback_path() {
    let listener = start_ipv4(
        callback_uri("127.0.0.1", "/oauth/return"),
        &CancellationToken::new(),
    );
    let addr = format!("127.0.0.1:{}", listener.port);

    // Wrong path returns 404 and does NOT terminate the listener.
    let wrong = send_request(&addr, "/callback?code=x&state=STATE");
    assert_eq!(http_status_code(&wrong), 404);

    // Correct path delivers the code.
    let ok = send_request(&addr, "/oauth/return?code=customcode&state=STATE");
    assert_eq!(http_status_code(&ok), 200);
    assert_eq!(listener.finish().expect("ok code"), "customcode");
}

#[test]
fn root_path_uri_matches_only_root() {
    let listener = start_ipv4(callback_uri("127.0.0.1", "/"), &CancellationToken::new());
    let addr = format!("127.0.0.1:{}", listener.port);

    // /callback is NOT the configured path; it must 404.
    let wrong = send_request(&addr, "/callback?code=x&state=STATE");
    assert_eq!(http_status_code(&wrong), 404);

    // Exact "/" delivers code.
    let ok = send_request(&addr, "/?code=rootcode&state=STATE");
    assert_eq!(http_status_code(&ok), 200);
    assert_eq!(listener.finish().expect("ok code"), "rootcode");
}

#[test]
fn rejects_loose_prefix_path_match() {
    let cancel = CancellationToken::new();
    let listener = start_ipv4(callback_uri("127.0.0.1", "/callback"), &cancel);

    // /callbackOther must NOT match — regression guard against the original
    // looser `starts_with("/callback")` semantics.
    let resp = send_request(
        &format!("127.0.0.1:{}", listener.port),
        "/callbackOther?code=evil&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 404);

    // Cancel externally; the listener exits. Its result is not under test:
    // the assertion that matters is the 404 above.
    cancel.cancel();
    let _ = listener.finish();
}

#[test]
fn state_mismatch_returns_400_and_error() {
    let listener = start_ipv4(
        callback_uri("127.0.0.1", "/callback"),
        &CancellationToken::new(),
    );

    let resp = send_request(
        &format!("127.0.0.1:{}", listener.port),
        "/callback?code=abc&state=NOT-THE-STATE",
    );
    assert_eq!(http_status_code(&resp), 400);
    assert!(resp.contains("invalid state parameter"));

    let err = listener
        .finish()
        .expect_err("state mismatch should be an error");
    assert!(err.to_string().contains("invalid state parameter"));
}

#[test]
fn external_cancellation_returns_quickly_with_typed_error() {
    // Regression coverage for the SIGTERM/SIGINT wiring in the listener
    // select: external cancellation must drop through the select arms and
    // return a typed Auth error without waiting for the 5-minute timeout.
    let cancel = CancellationToken::new();
    let listener = start_ipv4(callback_uri("127.0.0.1", "/callback"), &cancel);

    let cancelled = std::time::Instant::now();
    cancel.cancel();
    let res = listener.finish();
    assert!(res.is_err(), "external cancel must return Err");
    let elapsed = cancelled.elapsed();
    assert!(
        elapsed < Duration::from_secs(2),
        "cancellation should not wait on the 5-minute callback timeout; elapsed = {elapsed:?}"
    );
}

#[test]
fn on_bound_callback_fires_once_after_listener_ready() {
    let invocations: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let counted = Arc::clone(&invocations);

    let (listener, ()) = start_listener(
        pick_free_port_ipv4,
        callback_uri("127.0.0.1", "/callback"),
        |_| Some(()),
        &CancellationToken::new(),
        move || *counted.lock().unwrap() += 1,
    );

    let resp = send_request(
        &format!("127.0.0.1:{}", listener.port),
        "/callback?code=onboundcode&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 200);
    assert_eq!(listener.finish().expect("ok code"), "onboundcode");

    let n = *invocations.lock().unwrap();
    assert_eq!(n, 1, "on_bound must fire exactly once, got {n}");
}

/// The hazard a plain `async fn` listener carries: a caller that drops the
/// future mid-wait (a `select!` branch, a timeout) must not leave the
/// loopback port bound for the rest of the process.
#[tokio::test]
async fn dropping_the_listener_future_releases_the_port() {
    // A listener that returns inside the timeout did not bind: something
    // took the port after the pick, so another is picked.
    let mut dropped_on = None;
    for _ in 0..20 {
        let port = pick_free_port_ipv4();
        let uri = Url::parse(&format!("http://127.0.0.1:{port}/callback")).unwrap();
        let listener = wait_for_callback_with(&uri, "STATE", CancellationToken::new(), || {});
        if tokio::time::timeout(Duration::from_millis(200), listener)
            .await
            .is_err()
        {
            dropped_on = Some(port);
            break;
        }
    }
    let port = dropped_on.expect("the timeout drops a bound listener mid-wait");

    let mut rebound = None;
    for _ in 0..100 {
        tokio::task::yield_now().await;
        if let Ok(l) = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await {
            rebound = Some(l);
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(
        rebound.is_some(),
        "dropping the listener future must release 127.0.0.1:{port}"
    );
}
