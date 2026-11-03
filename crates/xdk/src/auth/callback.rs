//! `OAuth2` callback server — listens for the authorization code redirect.
//!
//! Binds host, port, and path from the resolved redirect URI. For `localhost`,
//! attempts to bind both `127.0.0.1:port` and `[::1]:port`. When only one
//! succeeds the listener proceeds with a warning; when both fail an error is
//! returned. Each connection is answered on its own task. Stops when a
//! caller-supplied [`tokio_util::sync::CancellationToken`] is cancelled, a
//! token the listener never cancels itself. Times out after 5 minutes,
//! matching the Go implementation. The library registers no signal handler:
//! the caller cancels the token when its own shutdown arrives.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::error::{
    CALLBACK_EMPTY_CODE, CALLBACK_INVALID_STATE, CALLBACK_TIMED_OUT, Error, Result,
};

/// Hardcoded 5-minute timeout matching upstream Go xurl.
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(300);

/// The most of one request the listener reads: a callback is a GET whose
/// request line and headers fit inside it.
const REQUEST_HEAD_LIMIT: usize = 4096;

/// One-shot delivery slot for the authorization code (or the refusal).
///
/// Wrapped in `Arc<Mutex<Option<_>>>` so the connection tasks of every
/// accept loop can race for the slot; whichever answers a callback first
/// takes the sender, sends, and the others find `None`.
type ResultSlot = Arc<Mutex<Option<oneshot::Sender<std::result::Result<String, Refusal>>>>>;

/// Why the listener refused a callback that reached its path.
#[derive(Debug, Clone, Copy)]
enum Refusal {
    InvalidState,
    EmptyCode,
}

impl Refusal {
    /// The page the browser shows.
    fn page(self) -> &'static str {
        match self {
            Self::InvalidState => "Error: invalid state parameter",
            Self::EmptyCode => "Error: empty authorization code",
        }
    }

    /// The message of the error the caller gets.
    fn message(self) -> &'static str {
        match self {
            Self::InvalidState => CALLBACK_INVALID_STATE,
            Self::EmptyCode => CALLBACK_EMPTY_CODE,
        }
    }
}

/// What every connection to one listener checks its request against and
/// reports through.
struct Callback {
    expected_state: String,
    uri_path: String,
    result_tx: ResultSlot,
    /// The listener's own stop, a child of the caller's token: a delivered
    /// code or a refusal ends the listener without cancelling the caller's.
    done: CancellationToken,
}

/// The `code` and `state` an authorization redirect carries.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct RedirectParams {
    pub(crate) code: Option<String>,
    pub(crate) state: Option<String>,
}

/// Reads `code` and `state` from a redirect's query string, decoded as
/// `application/x-www-form-urlencoded`, the encoding the authorize URL sends
/// `state` in. The listener and the headless step 2 both read the redirect
/// through it, so the two agree on a state whatever encoding the redirect
/// gives it. A repeated parameter takes its last value.
pub(crate) fn redirect_params(query: &str) -> RedirectParams {
    let mut params = RedirectParams::default();
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        match key.as_ref() {
            "code" => params.code = Some(value.into_owned()),
            "state" => params.state = Some(value.into_owned()),
            _ => {}
        }
    }
    params
}

/// One-shot delivery slot for the listener-ready signal.
type ReadySlot = Arc<Mutex<Option<oneshot::Sender<()>>>>;

/// The accept-loop tasks behind one listener.
///
/// Dropping it cancels the token and aborts every task, so a future the
/// caller drops mid-wait (a `select!` branch, a timeout) releases the
/// loopback port instead of leaving it bound for the process lifetime.
struct ListenerTasks {
    cancel: CancellationToken,
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl Drop for ListenerTasks {
    fn drop(&mut self) {
        self.cancel.cancel();
        for handle in &self.handles {
            handle.abort();
        }
    }
}

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
/// when the URI was registered with a trailing-slash root.
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
/// request begins with `uri_path?`, so `/callbackOther` does not match
/// `/callback`.
fn path_matches(uri_path: &str, request_path: &str) -> bool {
    if request_path == uri_path {
        return true;
    }
    let with_q = format!("{uri_path}?");
    request_path.starts_with(&with_q)
}

/// One-line warning body for partial-bind cases; the subscriber that renders
/// the warning adds its `warning:` prefix.
fn format_partial_bind_warning(bound_addr: &str, failed_addr: &str, failed_err: &str) -> String {
    format!(
        "callback listener bound {bound_addr} but failed to bind {failed_addr} ({failed_err}); continuing with the bound address"
    )
}

/// Reads `stream` into `buf` up to the blank line that ends the request head,
/// the peer's end of stream, or a full buffer, and returns the length read.
///
/// Reading the whole head leaves nothing unread when the connection closes:
/// closing with unread bytes sends a reset in place of a clean close (RFC
/// 1122 section 4.2.2.13), and the client can lose the response to it.
async fn read_request_head(stream: &mut TcpStream, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut filled = 0;
    while filled < buf.len() && !buf[..filled].windows(4).any(|w| w == b"\r\n\r\n") {
        let n = stream.read(&mut buf[filled..]).await?;
        if n == 0 {
            break;
        }
        filled += n;
    }
    Ok(filled)
}

/// Writes an HTTP/1.1 response with `status` and a plain-text `body`.
async fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

/// Drives one listener's accept loop.
///
/// Answers each connection on its own task, so a connection that sends
/// nothing (a browser's preconnect, a port scan) cannot hold up the one that
/// carries the redirect. The tasks belong to the loop: when it ends, the set
/// drops and aborts them.
async fn run_accept_loop(
    listener: TcpListener,
    callback: Arc<Callback>,
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

    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            () = callback.done.cancelled() => return,
            Some(_) = connections.join_next(), if !connections.is_empty() => {}
            accepted = listener.accept() => {
                if let Ok((stream, _)) = accepted {
                    connections.spawn(answer(stream, Arc::clone(&callback)));
                }
            }
        }
    }
}

/// Reads one request from `stream` and answers it. A callback on the
/// listener's path delivers its code, or its refusal, through `callback` and
/// ends the listener; any other path gets a 404 and leaves it serving.
async fn answer(mut stream: TcpStream, callback: Arc<Callback>) {
    let mut buf = [0u8; REQUEST_HEAD_LIMIT];
    let n = match read_request_head(&mut stream, &mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let request = String::from_utf8_lossy(&buf[..n]);
    let first_line = request.lines().next().unwrap_or("");
    let request_path = first_line.split_whitespace().nth(1).unwrap_or("");

    if !path_matches(&callback.uri_path, request_path) {
        respond(&mut stream, "404 Not Found", "Not Found").await;
        return;
    }

    let query = request_path.split_once('?').map_or("", |(_, query)| query);
    let RedirectParams { code, state } = redirect_params(query);
    let outcome = if state.as_deref().unwrap_or("") != callback.expected_state {
        Err(Refusal::InvalidState)
    } else {
        code.filter(|code| !code.is_empty())
            .ok_or(Refusal::EmptyCode)
    };

    match &outcome {
        Ok(_) => {
            respond(
                &mut stream,
                "200 OK",
                "Authentication successful! You can close this window.",
            )
            .await;
        }
        Err(refusal) => respond(&mut stream, "400 Bad Request", refusal.page()).await,
    }
    if let Some(tx) = callback.result_tx.lock().await.take() {
        let _ = tx.send(outcome);
    }
    callback.done.cancel();
}

/// Starts a callback server, runs `on_bound` once binds succeed, and waits
/// for the authorization code.
///
/// `on_bound` is invoked after every listener is bound, after both accept
/// tasks have been spawned, and after the parent has observed the ready
/// signal that fires from inside the first task's accept loop. For
/// the production OAuth2 flow this is where the browser is opened, so the
/// browser cannot reach the callback URL before the socket is actively
/// being drained.
///
/// `redirect_uri` provides the bind host, port, and request path. For
/// `localhost`, dual-binds `127.0.0.1` and `[::1]`; if exactly one bind
/// succeeds the function emits a warning and proceeds; if both fail it
/// returns an error. For explicit IPs / hostnames, single-binds.
///
/// `expected_state` is compared with the callback's `state` decoded as
/// `application/x-www-form-urlencoded`, so a redirect that percent-encodes it
/// still matches.
///
/// `cancel` is the caller's stop: cancelling it ends the wait. The listener
/// never cancels it itself, whether it delivers a code, refuses a callback,
/// or times out, so the caller can pass a token it shares with other work,
/// a signal handler's among them. Dropping the returned future releases the
/// listener's port.
///
/// # Errors
///
/// Returns an error if the URI is missing a host or port, every bind fails,
/// the listener returns an OAuth-protocol error (state mismatch / empty
/// code), the token is cancelled, or the 5-minute timeout fires.
pub async fn wait_for_callback_with<F>(
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
        .ok_or_else(|| Error::auth("redirect URI has no host"))?
        .to_string();
    let port = redirect_uri
        .port_or_known_default()
        .ok_or_else(|| Error::auth("redirect URI has no port"))?;
    let addrs = address_list(&host, port);
    let (bound, failed) = bind_all(&addrs).await;

    if bound.is_empty() {
        let detail = failed
            .iter()
            .map(|(addr, err)| format!("{addr}: {err}"))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(Error::auth(format!(
            "could not bind callback listener: {detail}"
        )));
    }

    // Partial-bind warning is localhost-only. Explicit-IP single binds never
    // warn on absent addresses (only one was attempted).
    if host == "localhost"
        && bound.len() == 1
        && let Some((failed_addr, failed_err)) = failed.first()
    {
        let bound_addr = &bound[0].addr;
        tracing::warn!(
            target: "xdk::auth",
            "{}",
            format_partial_bind_warning(bound_addr, failed_addr, failed_err)
        );
    }

    serve(bound, redirect_uri, expected_state, cancel, on_bound).await
}

/// [`wait_for_callback_with`] on a listener the caller has already bound,
/// at the port `redirect_uri` names, instead of binding that port itself.
///
/// For a caller that holds its own socket: one bound before privileges
/// drop, one handed over by a service manager, or one a test binds so no
/// other process can take the port between the choice and the bind.
/// `redirect_uri` still gives the request path; the listener is the only
/// address served, and `on_bound` runs once it is accepting.
///
/// # Errors
///
/// Returns an error if `listener` cannot be made non-blocking or is bound to
/// a port other than the one `redirect_uri` names, and otherwise as
/// [`wait_for_callback_with`] does once its listeners are bound.
pub async fn wait_for_callback_on<F>(
    listener: std::net::TcpListener,
    redirect_uri: &Url,
    expected_state: &str,
    cancel: CancellationToken,
    on_bound: F,
) -> Result<String>
where
    F: FnOnce() + Send + 'static,
{
    let addr = listener
        .local_addr()
        .map_err(|e| Error::auth_with_cause("ListenerError", &e).with_source(e))?;
    let port = redirect_uri
        .port_or_known_default()
        .ok_or_else(|| Error::auth("redirect URI has no port"))?;
    if addr.port() != port {
        return Err(Error::auth(format!(
            "the callback listener is bound to port {}, but the redirect URI names port {port}",
            addr.port()
        )));
    }
    listener
        .set_nonblocking(true)
        .map_err(|e| Error::auth_with_cause("ListenerError", &e).with_source(e))?;
    let listener = TcpListener::from_std(listener)
        .map_err(|e| Error::auth_with_cause("ListenerError", &e).with_source(e))?;
    let bound = vec![BoundAddress {
        listener,
        addr: addr.to_string(),
    }];
    serve(bound, redirect_uri, expected_state, cancel, on_bound).await
}

/// Runs the accept loops on `bound`, runs `on_bound` once the first is
/// accepting, and waits for the authorization code.
async fn serve<F>(
    bound: Vec<BoundAddress>,
    redirect_uri: &Url,
    expected_state: &str,
    cancel: CancellationToken,
    on_bound: F,
) -> Result<String>
where
    F: FnOnce() + Send + 'static,
{
    let (result_tx, result_rx) = oneshot::channel::<std::result::Result<String, Refusal>>();
    let callback = Arc::new(Callback {
        expected_state: expected_state.to_string(),
        uri_path: callback_path_from(redirect_uri),
        result_tx: Arc::new(Mutex::new(Some(result_tx))),
        done: cancel.child_token(),
    });

    let (ready_tx, ready_rx) = oneshot::channel::<()>();
    let ready_tx = Arc::new(Mutex::new(Some(ready_tx)));
    let ready_flag = Arc::new(AtomicBool::new(false));

    let mut tasks = ListenerTasks {
        cancel: callback.done.clone(),
        handles: Vec::with_capacity(bound.len()),
    };
    for b in bound {
        let callback = Arc::clone(&callback);
        let ready_tx = Arc::clone(&ready_tx);
        let ready_flag = Arc::clone(&ready_flag);
        // The accept loops are polled outside the caller's future, so they
        // carry its dispatcher or their warnings never reach the binary.
        tasks.handles.push(tokio::spawn(
            tracing::instrument::WithSubscriber::with_current_subscriber(async move {
                run_accept_loop(b.listener, callback, ready_flag, ready_tx).await;
            }),
        ));
    }

    // Await the inside-accept ready signal, then run the on_bound hook on
    // a blocking-safe spawn (the production case is `open::that` which is
    // synchronous; tests may block briefly to record timing).
    let _ = ready_rx.await;
    tokio::task::spawn_blocking(on_bound)
        .await
        .map_err(|e| Error::auth_with_cause("OnBoundJoinError", &e).with_source(e))?;

    let result = tokio::select! {
        biased;
        res = result_rx => {
            match res {
                Ok(Ok(code)) => Ok(code),
                Ok(Err(refusal)) => Err(Error::auth(refusal.message())),
                Err(_) => Err(Error::auth("ListenerError: oauth2 listener failed")),
            }
        }
        () = cancel.cancelled() => {
            Err(Error::auth("ListenerError: cancelled before code received"))
        }
        () = tokio::time::sleep(CALLBACK_TIMEOUT) => {
            Err(Error::auth(CALLBACK_TIMED_OUT))
        }
    };

    drop(tasks);
    result
}

/// Convenience wrapper for the no-op `on_bound` case (e.g., direct tests of
/// the listener that do not care about the bind-before-open sequencing).
///
/// # Errors
///
/// See [`wait_for_callback_with`].
#[allow(dead_code)] // Reserved for tests that drive the listener without a side-effect on bind.
pub async fn wait_for_callback(
    redirect_uri: &Url,
    expected_state: &str,
    cancel: CancellationToken,
) -> Result<String> {
    wait_for_callback_with(redirect_uri, expected_state, cancel, || {}).await
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
