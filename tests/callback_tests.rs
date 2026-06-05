//! OAuth2 callback listener tests.
//!
//! Drives `xurl::auth::callback::wait_for_callback_with` on a worker thread
//! and exercises bind, path-match, state-validation, partial-bind warning,
//! and cancellation behaviour from a client-side `tokio::net::TcpStream`.

use std::io::{Read, Write};
use std::net::{TcpListener as StdTcpListener, TcpStream as StdTcpStream};
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use url::Url;

use xurl::auth::callback::wait_for_callback_with;
use xurl::error::Result as XurlResult;

// ── Port allocation ───────────────────────────────────────────────────────
//
// We must not race the OS for a free port — bind both 127.0.0.1:0 AND [::1]:0
// to learn a free dual-stack port. Many tests need the same port to be free
// on both stacks; we pick a port available on 127.0.0.1 only (the IPv6 leg
// is best-effort because some CI environments disable [::1]).

fn pick_free_port_ipv4() -> u16 {
    let l = StdTcpListener::bind("127.0.0.1:0").expect("bind 127.0.0.1:0");
    let port = l.local_addr().unwrap().port();
    drop(l);
    port
}

fn pick_free_port_ipv6() -> Option<u16> {
    StdTcpListener::bind("[::1]:0")
        .ok()
        .map(|l| l.local_addr().unwrap().port())
}

// ── HTTP helpers ──────────────────────────────────────────────────────────

fn send_request(addr: &str, path: &str) -> String {
    // Retry briefly: the listener task may not have polled `accept` yet on
    // very fast hosts. Bounded retry avoids racing without making the test
    // slow.
    let mut last_err = String::new();
    for _ in 0..20 {
        match StdTcpStream::connect(addr) {
            Ok(mut stream) => {
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let req =
                    format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
                stream.write_all(req.as_bytes()).expect("write request");
                let mut buf = String::new();
                let _ = stream.read_to_string(&mut buf);
                return buf;
            }
            Err(e) => {
                last_err = e.to_string();
                thread::sleep(Duration::from_millis(25));
            }
        }
    }
    panic!("could not connect to {addr}: {last_err}");
}

fn http_status_code(response: &str) -> u16 {
    let first = response.lines().next().unwrap_or("");
    let parts: Vec<&str> = first.split_whitespace().collect();
    parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0)
}

// ── Test bodies ───────────────────────────────────────────────────────────

#[test]
fn single_explicit_ipv4_bind_delivers_code() {
    let port = pick_free_port_ipv4();
    let uri = Url::parse(&format!("http://127.0.0.1:{port}/callback")).unwrap();
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel::<XurlResult<String>>();

    let h = thread::spawn(move || {
        let res = wait_for_callback_with(&uri, "STATE", cancel, || {});
        tx.send(res).unwrap();
    });

    let resp = send_request(
        &format!("127.0.0.1:{port}"),
        "/callback?code=mycode&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 200);
    let code = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("listener returned result")
        .expect("ok code");
    assert_eq!(code, "mycode");
    h.join().unwrap();
}

#[test]
fn single_explicit_ipv6_bind_delivers_code() {
    let Some(port) = pick_free_port_ipv6() else {
        eprintln!("skipping ipv6 test: [::1]:0 unavailable in this env");
        return;
    };
    let uri = Url::parse(&format!("http://[::1]:{port}/callback")).unwrap();
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel::<XurlResult<String>>();

    let h = thread::spawn(move || {
        let res = wait_for_callback_with(&uri, "STATE", cancel, || {});
        tx.send(res).unwrap();
    });

    let resp = send_request(
        &format!("[::1]:{port}"),
        "/callback?code=mycode6&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 200);
    let code = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("listener returned result")
        .expect("ok code");
    assert_eq!(code, "mycode6");
    h.join().unwrap();
}

#[test]
fn dual_bind_localhost_delivers_via_ipv4() {
    let port_v4 = pick_free_port_ipv4();
    // Best-effort: if [::1]:port_v4 is occupied, this test exercises the
    // partial-bind path with a warning. The assertion only checks that the
    // code flow completes via the IPv4 leg.
    let uri = Url::parse(&format!("http://localhost:{port_v4}/callback")).unwrap();
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel::<XurlResult<String>>();

    let h = thread::spawn(move || {
        let res = wait_for_callback_with(&uri, "STATE", cancel, || {});
        tx.send(res).unwrap();
    });

    let resp = send_request(
        &format!("127.0.0.1:{port_v4}"),
        "/callback?code=dualcode&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 200);
    let code = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("listener returned result")
        .expect("ok code");
    assert_eq!(code, "dualcode");
    h.join().unwrap();
}

#[test]
fn partial_bind_localhost_proceeds_when_ipv6_unavailable() {
    // Pre-bind [::1]:port to force the IPv6 leg to fail; the listener must
    // proceed with the IPv4 socket. We can only run this when the OS has
    // an [::1] stack available.
    let port = pick_free_port_ipv4();
    let Ok(_hog) = StdTcpListener::bind(format!("[::1]:{port}")) else {
        // [::1] entirely unavailable — the test's premise (one bind fails,
        // the other succeeds) cannot be set up. The IPv4-only success path
        // is already covered by the single-bind test above.
        eprintln!("skipping partial-bind ipv4 test: [::1]:{port} cannot be reserved");
        return;
    };

    let uri = Url::parse(&format!("http://localhost:{port}/callback")).unwrap();
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel::<XurlResult<String>>();

    let h = thread::spawn(move || {
        let res = wait_for_callback_with(&uri, "STATE", cancel, || {});
        tx.send(res).unwrap();
    });

    let resp = send_request(
        &format!("127.0.0.1:{port}"),
        "/callback?code=partialcode&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 200);
    let code = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("listener returned result")
        .expect("ok code");
    assert_eq!(code, "partialcode");
    h.join().unwrap();
    drop(_hog);
}

#[test]
fn both_binds_fail_returns_error() {
    let port = pick_free_port_ipv4();
    // Pre-bind 127.0.0.1:port to make the IPv4 leg fail.
    let hog4 = StdTcpListener::bind(format!("127.0.0.1:{port}")).expect("bind v4 hog");
    // Also bind [::1]:port; if [::1] unavailable, this test still exercises
    // a single failing bind for an explicit-IP URI, but the localhost arm
    // requires both. Use the explicit 127.0.0.1 URI to keep this test
    // single-bind and deterministic across IPv6-disabled environments.
    let uri = Url::parse(&format!("http://127.0.0.1:{port}/callback")).unwrap();
    let cancel = CancellationToken::new();

    let res = wait_for_callback_with(&uri, "STATE", cancel, || {});
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
    let port = pick_free_port_ipv4();
    let uri = Url::parse(&format!("http://127.0.0.1:{port}/oauth/return")).unwrap();
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel::<XurlResult<String>>();

    let h = thread::spawn(move || {
        let res = wait_for_callback_with(&uri, "STATE", cancel, || {});
        tx.send(res).unwrap();
    });

    // Wrong path returns 404 and does NOT terminate the listener.
    let wrong = send_request(&format!("127.0.0.1:{port}"), "/callback?code=x&state=STATE");
    assert_eq!(http_status_code(&wrong), 404);

    // Correct path delivers the code.
    let ok = send_request(
        &format!("127.0.0.1:{port}"),
        "/oauth/return?code=customcode&state=STATE",
    );
    assert_eq!(http_status_code(&ok), 200);

    let code = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("listener result")
        .expect("ok code");
    assert_eq!(code, "customcode");
    h.join().unwrap();
}

#[test]
fn root_path_uri_matches_only_root() {
    let port = pick_free_port_ipv4();
    let uri = Url::parse(&format!("http://127.0.0.1:{port}/")).unwrap();
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel::<XurlResult<String>>();

    let h = thread::spawn(move || {
        let res = wait_for_callback_with(&uri, "STATE", cancel, || {});
        tx.send(res).unwrap();
    });

    // /callback is NOT the configured path; it must 404.
    let wrong = send_request(&format!("127.0.0.1:{port}"), "/callback?code=x&state=STATE");
    assert_eq!(http_status_code(&wrong), 404);

    // Exact "/" delivers code.
    let ok = send_request(&format!("127.0.0.1:{port}"), "/?code=rootcode&state=STATE");
    assert_eq!(http_status_code(&ok), 200);

    let code = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("listener result")
        .expect("ok code");
    assert_eq!(code, "rootcode");
    h.join().unwrap();
}

#[test]
fn rejects_loose_prefix_path_match() {
    let port = pick_free_port_ipv4();
    let uri = Url::parse(&format!("http://127.0.0.1:{port}/callback")).unwrap();
    let cancel = CancellationToken::new();
    let cancel_for_thread = cancel.clone();
    let (tx, rx) = mpsc::channel::<XurlResult<String>>();

    let h = thread::spawn(move || {
        let res = wait_for_callback_with(&uri, "STATE", cancel_for_thread, || {});
        tx.send(res).unwrap();
    });

    // /callbackOther must NOT match — regression guard against the original
    // looser `starts_with("/callback")` semantics.
    let resp = send_request(
        &format!("127.0.0.1:{port}"),
        "/callbackOther?code=evil&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 404);

    // Cancel externally; the listener should exit cleanly.
    cancel.cancel();
    let res = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("listener exits on cancel");
    // On external cancel, the result channel never sends — the listener
    // produces a ListenerError because the result_rx drops when result_tx
    // is held by the task that returned. Either an error or success is
    // acceptable; the assertion that matters is the 404 above.
    let _ = res;
    h.join().unwrap();
}

#[test]
fn state_mismatch_returns_400_and_error() {
    let port = pick_free_port_ipv4();
    let uri = Url::parse(&format!("http://127.0.0.1:{port}/callback")).unwrap();
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel::<XurlResult<String>>();

    let h = thread::spawn(move || {
        let res = wait_for_callback_with(&uri, "STATE", cancel, || {});
        tx.send(res).unwrap();
    });

    let resp = send_request(
        &format!("127.0.0.1:{port}"),
        "/callback?code=abc&state=NOT-THE-STATE",
    );
    assert_eq!(http_status_code(&resp), 400);
    assert!(resp.contains("invalid state parameter"));

    let res = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("listener result");
    let err = res.expect_err("state mismatch should be an error");
    assert!(err.to_string().contains("invalid state parameter"));
    h.join().unwrap();
}

#[test]
fn external_cancellation_returns_quickly_with_typed_error() {
    // Regression coverage for the SIGTERM/SIGINT wiring in the listener
    // select: external cancellation must drop through the select arms and
    // return a typed Auth error without waiting for the 5-minute timeout.
    let port = pick_free_port_ipv4();
    let uri = Url::parse(&format!("http://127.0.0.1:{port}/callback")).unwrap();
    let cancel = CancellationToken::new();
    let cancel_for_thread = cancel.clone();
    let (tx, rx) = mpsc::channel::<XurlResult<String>>();

    let started = std::time::Instant::now();
    let h = thread::spawn(move || {
        let res = wait_for_callback_with(&uri, "STATE", cancel_for_thread, || {});
        tx.send(res).unwrap();
    });

    // Give the listener time to bind and enter accept(), then cancel.
    thread::sleep(Duration::from_millis(50));
    cancel.cancel();

    let res = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("listener must observe cancellation promptly");
    assert!(res.is_err(), "external cancel must return Err");
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(2),
        "cancellation should not wait on the 5-minute callback timeout; elapsed = {elapsed:?}"
    );
    h.join().unwrap();
}

#[test]
fn on_bound_callback_fires_once_after_listener_ready() {
    let port = pick_free_port_ipv4();
    let uri = Url::parse(&format!("http://127.0.0.1:{port}/callback")).unwrap();
    let cancel = CancellationToken::new();
    let invocations: std::sync::Arc<Mutex<u32>> = std::sync::Arc::new(Mutex::new(0));
    let invocations_for_closure = std::sync::Arc::clone(&invocations);

    let (tx, rx) = mpsc::channel::<XurlResult<String>>();
    let h = thread::spawn(move || {
        let res = wait_for_callback_with(&uri, "STATE", cancel, move || {
            *invocations_for_closure.lock().unwrap() += 1;
        });
        tx.send(res).unwrap();
    });

    let resp = send_request(
        &format!("127.0.0.1:{port}"),
        "/callback?code=onboundcode&state=STATE",
    );
    assert_eq!(http_status_code(&resp), 200);
    let code = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("listener result")
        .expect("ok code");
    assert_eq!(code, "onboundcode");
    h.join().unwrap();

    let n = *invocations.lock().unwrap();
    assert_eq!(n, 1, "on_bound must fire exactly once, got {n}");
}
