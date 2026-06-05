//! `OAuth2` callback server — listens for the authorization code redirect.
//!
//! Binds host, port, and path from the resolved redirect URI. For `localhost`,
//! attempts to bind both `127.0.0.1:port` and `[::1]:port`. When only one
//! succeeds the listener proceeds with a one-line warning to stderr; when both
//! fail an error is returned. Coordinates shutdown via a
//! [`tokio_util::sync::CancellationToken`]; the success path triggers
//! cancellation immediately after the code is delivered. Times out after 5
//! minutes, matching the Go implementation.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::sync::{Mutex, oneshot};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::error::{Result, XurlError};

/// Resolves when the process receives SIGINT (Ctrl+C) or, on Unix, SIGTERM.
///
/// Used by long-running paths (OAuth2 callback listener, streaming HTTP) to
/// observe shutdown signals and exit cleanly. On Windows only `ctrl_c()` is
/// available; the `cfg(unix)` arm folds SIGTERM into the same future.
pub(crate) async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(_) => {
                // SignalKind::terminate is documented to always succeed on
                // Unix; if it does fail, fall back to ctrl_c only.
                let _ = tokio::signal::ctrl_c().await;
                return;
            }
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

/// Hardcoded 5-minute timeout matching upstream Go xurl.
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(300);

/// One-shot delivery slot for the authorization code (or an error string).
///
/// Wrapped in `Arc<Mutex<Option<_>>>` so the multiple accept-loop tasks can
/// race for the slot; whichever task receives the valid callback takes the
/// sender, sends, and the sibling tasks see `None` on cancellation.
type ResultSlot = Arc<Mutex<Option<oneshot::Sender<std::result::Result<String, String>>>>>;

/// One-shot delivery slot for the listener-ready signal.
type ReadySlot = Arc<Mutex<Option<oneshot::Sender<()>>>>;

/// Bind result for a single address.
struct BoundAddress {
    listener: TcpListener,
    addr: String,
}

/// Attempts to bind every address in `addrs`, returning the successful
/// listeners and the (address, error-string) tuples for any failures.
async fn bind_all(addrs: &[String]) -> (Vec<BoundAddress>, Vec<(String, String)>) {
    let mut bound = Vec::new();
    let mut failed = Vec::new();
    for addr in addrs {
        match TcpListener::bind(&addr).await {
            Ok(listener) => bound.push(BoundAddress {
                listener,
                addr: addr.clone(),
            }),
            Err(e) => failed.push((addr.clone(), e.to_string())),
        }
    }
    (bound, failed)
}

/// Builds the address list to bind for a given URI host and port.
///
/// - `localhost` expands to both `127.0.0.1:port` and `[::1]:port`.
/// - Any other host (explicit IP or DNS name) yields a single entry,
///   IPv6 wrapped in brackets when needed.
fn address_list(host: &str, port: u16) -> Vec<String> {
    if host == "localhost" {
        return vec![format!("127.0.0.1:{port}"), format!("[::1]:{port}")];
    }
    if host.contains(':') && !host.starts_with('[') {
        return vec![format!("[{host}]:{port}")];
    }
    vec![format!("{host}:{port}")]
}

/// Resolves the listener's expected request path from the URI.
///
/// The url crate normalizes path-less URIs to `"/"`, but defensively treat
/// empty as `/callback` (the upstream-compatible default). Honour `/` exactly
/// when the URI was registered with a trailing-slash root — KTD6.
fn callback_path_from(uri: &Url) -> String {
    let p = uri.path();
    if p.is_empty() {
        "/callback".to_string()
    } else {
        p.to_string()
    }
}

/// Match `request_path` against the configured `uri_path`.
///
/// Exact-or-querystring: matches when `request_path == uri_path` or when the
/// request begins with `uri_path?`. The looser `starts_with` semantics of the
/// original implementation are tightened — `/callbackOther` no longer matches
/// `/callback`.
fn path_matches(uri_path: &str, request_path: &str) -> bool {
    if request_path == uri_path {
        return true;
    }
    let with_q = format!("{uri_path}?");
    request_path.starts_with(&with_q)
}

/// One-line warning body for partial-bind cases. The `warning:` prefix is
/// added by [`crate::output::warn_stderr`] at the emission site.
fn format_partial_bind_warning(bound_addr: &str, failed_addr: &str, failed_err: &str) -> String {
    format!(
        "callback listener bound {bound_addr} but failed to bind {failed_addr} ({failed_err}); continuing with the bound address"
    )
}

/// Drives one listener's accept loop.
///
/// Reads a single HTTP request, validates state, and writes the response.
/// Delivers the code (or an error string) through the shared `result_tx`,
/// then cancels the token to broadcast shutdown to the sibling task.
#[allow(clippy::too_many_arguments)] // local helper, internal to wait_for_callback
async fn run_accept_loop(
    listener: TcpListener,
    expected_state: String,
    uri_path: String,
    result_tx: ResultSlot,
    cancel: CancellationToken,
    ready_flag: Arc<AtomicBool>,
    ready_tx: ReadySlot,
) {
    // Signal "ready" exactly once across all sibling accept tasks. The flag is
    // raced via Ordering::AcqRel; whichever task wins fires the oneshot.
    if !ready_flag.swap(true, Ordering::AcqRel)
        && let Some(tx) = ready_tx.lock().await.take()
    {
        let _ = tx.send(());
    }

    loop {
        let accept_result = tokio::select! {
            () = cancel.cancelled() => return,
            res = listener.accept() => res,
        };
        let Ok((stream, _)) = accept_result else {
            continue;
        };

        let mut buf = [0u8; 4096];
        if stream.readable().await.is_err() {
            continue;
        }
        let Ok(n) = stream.try_read(&mut buf) else {
            continue;
        };
        if n == 0 {
            continue;
        }

        let request = String::from_utf8_lossy(&buf[..n]);
        let first_line = request.lines().next().unwrap_or("");
        let request_path = first_line.split_whitespace().nth(1).unwrap_or("");

        if !path_matches(&uri_path, request_path) {
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
            let _ = stream.writable().await;
            let _ = stream.try_write(response.as_bytes());
            continue;
        }

        let query = request_path.split('?').nth(1).unwrap_or("");
        let params: HashMap<&str, &str> =
            query.split('&').filter_map(|p| p.split_once('=')).collect();

        let code = params.get("code").copied().unwrap_or("");
        let received_state = params.get("state").copied().unwrap_or("");

        if received_state != expected_state {
            let body = "Error: invalid state parameter";
            let response = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.writable().await;
            let _ = stream.try_write(response.as_bytes());
            if let Some(tx) = result_tx.lock().await.take() {
                let _ = tx.send(Err("invalid state parameter".to_string()));
            }
            cancel.cancel();
            return;
        }

        if code.is_empty() {
            let body = "Error: empty authorization code";
            let response = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.writable().await;
            let _ = stream.try_write(response.as_bytes());
            if let Some(tx) = result_tx.lock().await.take() {
                let _ = tx.send(Err("empty authorization code".to_string()));
            }
            cancel.cancel();
            return;
        }

        let body = "Authentication successful! You can close this window.";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.writable().await;
        let _ = stream.try_write(response.as_bytes());

        if let Some(tx) = result_tx.lock().await.take() {
            let _ = tx.send(Ok(code.to_string()));
        }
        cancel.cancel();
        return;
    }
}

/// Starts a callback server, runs `on_bound` once binds succeed, and waits
/// for the authorization code.
///
/// `on_bound` is invoked after every listener is bound, after both accept
/// tasks have been spawned, and after the parent has observed the ready
/// signal that fires from inside the first task's accept loop (KTD5). For
/// the production OAuth2 flow this is where the browser is opened, so the
/// browser cannot reach the callback URL before the socket is actively
/// being drained.
///
/// `redirect_uri` provides the bind host, port, and request path. For
/// `localhost`, dual-binds `127.0.0.1` and `[::1]`; if exactly one bind
/// succeeds the function emits a one-line stderr warning and proceeds; if
/// both fail it returns an error. For explicit IPs / hostnames, single-binds.
///
/// `cancel` coordinates shutdown — the success path cancels immediately
/// after delivering the code; the 5-minute timeout broadcasts cancellation;
/// callers may also cancel externally.
///
/// # Errors
///
/// Returns an error if the URI is missing a host or port, every bind fails,
/// the runtime cannot be constructed, the listener returns an OAuth-protocol
/// error (state mismatch / empty code), or the 5-minute timeout fires.
pub fn wait_for_callback_with<F>(
    redirect_uri: &Url,
    expected_state: &str,
    cancel: CancellationToken,
    on_bound: F,
) -> Result<String>
where
    F: FnOnce() + Send + 'static,
{
    let host = redirect_uri
        .host_str()
        .ok_or_else(|| XurlError::auth("redirect URI has no host"))?
        .to_string();
    let port = redirect_uri
        .port_or_known_default()
        .ok_or_else(|| XurlError::auth("redirect URI has no port"))?;
    let expected_state = expected_state.to_string();
    let uri_path = callback_path_from(redirect_uri);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| XurlError::auth_with_cause("ServerError", &e))?;

    rt.block_on(async move {
        let addrs = address_list(&host, port);
        let (bound, failed) = bind_all(&addrs).await;

        if bound.is_empty() {
            let detail = failed
                .iter()
                .map(|(addr, err)| format!("{addr}: {err}"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(XurlError::auth(format!(
                "could not bind callback listener: {detail}"
            )));
        }

        // Partial-bind warning is localhost-only per KTD4. Explicit-IP single
        // binds never warn on absent addresses (only one was attempted).
        if host == "localhost"
            && bound.len() == 1
            && let Some((failed_addr, failed_err)) = failed.first()
        {
            let bound_addr = &bound[0].addr;
            crate::output::warn_stderr(&format_partial_bind_warning(
                bound_addr,
                failed_addr,
                failed_err,
            ));
        }

        let (result_tx, result_rx) = oneshot::channel::<std::result::Result<String, String>>();
        let result_tx = Arc::new(Mutex::new(Some(result_tx)));

        let (ready_tx, ready_rx) = oneshot::channel::<()>();
        let ready_tx = Arc::new(Mutex::new(Some(ready_tx)));
        let ready_flag = Arc::new(AtomicBool::new(false));

        let mut join_handles = Vec::new();
        for b in bound {
            let result_tx = Arc::clone(&result_tx);
            let cancel = cancel.clone();
            let ready_tx = Arc::clone(&ready_tx);
            let ready_flag = Arc::clone(&ready_flag);
            let expected_state = expected_state.clone();
            let uri_path = uri_path.clone();
            join_handles.push(tokio::spawn(async move {
                run_accept_loop(
                    b.listener,
                    expected_state,
                    uri_path,
                    result_tx,
                    cancel,
                    ready_flag,
                    ready_tx,
                )
                .await;
            }));
        }

        // Await the inside-accept ready signal, then run the on_bound hook on
        // a blocking-safe spawn (the production case is `open::that` which is
        // synchronous; tests may block briefly to record timing).
        let _ = ready_rx.await;
        tokio::task::spawn_blocking(on_bound)
            .await
            .map_err(|e| XurlError::auth_with_cause("OnBoundJoinError", &e))?;

        let result = tokio::select! {
            biased;
            res = result_rx => {
                match res {
                    Ok(Ok(code)) => Ok(code),
                    Ok(Err(e)) => Err(XurlError::auth(format!("CallbackError: {e}"))),
                    Err(_) => Err(XurlError::auth("ListenerError: oauth2 listener failed")),
                }
            }
            () = cancel.cancelled() => {
                Err(XurlError::auth("ListenerError: cancelled before code received"))
            }
            () = shutdown_signal() => {
                cancel.cancel();
                Err(XurlError::auth("Cancelled: oauth callback cancelled by signal"))
            }
            () = tokio::time::sleep(CALLBACK_TIMEOUT) => {
                cancel.cancel();
                Err(XurlError::auth("Timeout: authentication timed out"))
            }
        };

        cancel.cancel();
        for h in join_handles {
            // Best-effort: do not block the success path on the sibling's
            // graceful exit; the CancellationToken triggers it within one
            // accept poll cycle.
            h.abort();
        }
        result
    })
}

/// Convenience wrapper for the no-op `on_bound` case (e.g., direct tests of
/// the listener that do not care about the bind-before-open sequencing).
///
/// # Errors
///
/// See [`wait_for_callback_with`].
#[allow(dead_code)] // Reserved for tests that drive the listener without a side-effect on bind.
pub fn wait_for_callback(
    redirect_uri: &Url,
    expected_state: &str,
    cancel: CancellationToken,
) -> Result<String> {
    wait_for_callback_with(redirect_uri, expected_state, cancel, || {})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_list_localhost_dual_binds() {
        let addrs = address_list("localhost", 9090);
        assert_eq!(addrs, vec!["127.0.0.1:9090", "[::1]:9090"]);
    }

    #[test]
    fn address_list_explicit_ipv4_single() {
        let addrs = address_list("127.0.0.1", 9090);
        assert_eq!(addrs, vec!["127.0.0.1:9090"]);
    }

    #[test]
    fn address_list_explicit_ipv6_wraps_brackets() {
        // The url crate stores `::1` without brackets in `host_str()`; the
        // bind address must wrap it.
        let addrs = address_list("::1", 9090);
        assert_eq!(addrs, vec!["[::1]:9090"]);
    }

    #[test]
    fn callback_path_from_explicit_path() {
        let uri = Url::parse("http://localhost:8080/callback").expect("test URL must parse");
        assert_eq!(callback_path_from(&uri), "/callback");
    }

    #[test]
    fn callback_path_from_trailing_slash_root_honoured() {
        let uri = Url::parse("http://localhost:8080/").expect("test URL must parse");
        assert_eq!(callback_path_from(&uri), "/");
    }

    #[test]
    fn callback_path_from_custom_path() {
        let uri = Url::parse("http://localhost:8080/oauth/return").expect("test URL must parse");
        assert_eq!(callback_path_from(&uri), "/oauth/return");
    }

    #[test]
    fn path_matches_exact() {
        assert!(path_matches("/callback", "/callback"));
    }

    #[test]
    fn path_matches_querystring() {
        assert!(path_matches("/callback", "/callback?code=abc&state=xyz"));
    }

    #[test]
    fn path_matches_rejects_loose_prefix() {
        assert!(!path_matches("/callback", "/callbackOther"));
        assert!(!path_matches("/callback", "/callbackOther?code=abc"));
    }

    #[test]
    fn path_matches_rejects_unrelated() {
        assert!(!path_matches("/callback", "/other"));
    }

    #[test]
    fn path_matches_root_exact() {
        assert!(path_matches("/", "/"));
        assert!(path_matches("/", "/?code=abc"));
    }
}
