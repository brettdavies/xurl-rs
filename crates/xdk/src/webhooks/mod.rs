//! A receiver for X API webhook deliveries.
//!
//! X sends two kinds of request to a registered webhook URL. A `GET` with a
//! `crc_token` query parameter is its Challenge-Response Check: the receiver
//! proves it holds the app's secret by answering with an HMAC of the token.
//! A `POST` is an event, signed over its raw body with the same secret.
//!
//! [`Receiver`] binds an address, answers the check, and yields each event
//! whose signature verifies. It prints nothing and opens no tunnel: X
//! requires a public HTTPS URL with no port, so the caller exposes the bound
//! address through a proxy or tunnel of its own and registers that URL with
//! [`Client::create_webhook`](crate::api::Client::create_webhook).
//!
//! The signing rules are X's, from its webhook documentation: the secret is
//! the app's `OAuth2` client secret when it has one and its `OAuth1` consumer
//! secret otherwise, an event carries `X-Twitter-Webhooks-Signature-OAuth2`
//! or the legacy `X-Twitter-Webhooks-Signature`, and both are
//! `sha256=<base64 HMAC-SHA256>`.

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Incoming;
use hyper::header::{ALLOW, CONTENT_TYPE};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::{TokioIo, TokioTimer};
use tokio::net::TcpListener;
use tokio::sync::{Semaphore, mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::error::{Error, Result};

mod budget;
mod checkpoint;
mod forward;
mod signing;

pub use budget::{Attempt, AttemptTimedOut, Outcome, RetryBudget};
#[doc(hidden)]
pub use checkpoint::CHECKPOINT_SHAPE;
pub use checkpoint::{CheckpointWriter, ReplayWindow, read_checkpoint, replay_window};
pub use forward::{ForwardFailure, UrlTarget, validate_forward_url};
use signing::verified;
pub use signing::{
    LEGACY_SIGNATURE_HEADER, OAUTH2_SIGNATURE_HEADER, SignedSample, SigningSecrets, sign,
};

/// The `tracing` target of every event this module emits.
pub const WEBHOOKS_TARGET: &str = "xdk::webhooks";

/// The path a receiver answers on unless [`ReceiverConfig::path`] names another.
pub const DEFAULT_PATH: &str = "/webhook";

/// The largest event body a receiver reads unless
/// [`ReceiverConfig::max_body_bytes`] names another size.
pub const DEFAULT_MAX_BODY_BYTES: usize = 1024 * 1024;

/// The most connections a receiver serves at once unless
/// [`ReceiverConfig::max_connections`] names another number.
pub const DEFAULT_MAX_CONNECTIONS: usize = 64;

/// How long X waits for a webhook to answer, per its documentation. A request
/// whose headers or body take longer than this to arrive is dropped.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Events held for a caller that is slower than X delivers, each with X's
/// request waiting on its answer.
const EVENT_BUFFER: usize = 64;

/// How long the accept loop waits after the listener reports an error, such
/// as the process running out of file descriptors, before it asks again.
const ACCEPT_RETRY_DELAY: Duration = Duration::from_millis(50);

/// Where a [`Receiver`] listens and what it accepts.
#[derive(Debug, Clone)]
pub struct ReceiverConfig {
    /// The address to bind. Port 0 takes any free port;
    /// [`Receiver::local_addr`] reports the one bound.
    pub bind: SocketAddr,
    /// The request path the receiver answers on.
    pub path: String,
    /// The app's signing secrets.
    pub secrets: SigningSecrets,
    /// Yield a `POST` that carries no signature header. A `POST` whose
    /// signature is present and wrong is refused either way.
    pub allow_unsigned: bool,
    /// The largest event body to read; a larger one is answered 413.
    pub max_body_bytes: usize,
    /// The most connections to serve at once. A connection past this number
    /// waits to be accepted until one in service closes, which bounds what a
    /// flood of connections to a public URL can hold open.
    pub max_connections: usize,
}

impl ReceiverConfig {
    /// A receiver on `bind` at [`DEFAULT_PATH`] that yields signed events
    /// only, reads bodies up to [`DEFAULT_MAX_BODY_BYTES`], and serves up to
    /// [`DEFAULT_MAX_CONNECTIONS`] connections at once.
    #[must_use]
    pub fn new(bind: SocketAddr, secrets: SigningSecrets) -> Self {
        Self {
            bind,
            path: DEFAULT_PATH.to_string(),
            secrets,
            allow_unsigned: false,
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
            max_connections: DEFAULT_MAX_CONNECTIONS,
        }
    }
}

/// How an event's origin was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Signature {
    /// The `OAuth2` signature header verified against the client secret.
    OAuth2,
    /// The legacy signature header verified against the consumer secret.
    OAuth1,
    /// The request carried no signature header and
    /// [`ReceiverConfig::allow_unsigned`] let it through.
    Unsigned,
}

/// One delivery X made to the webhook.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Event {
    /// The request body exactly as it arrived, which is what the signature
    /// covers.
    pub body: Vec<u8>,
    /// How the event's origin was established.
    pub signature: Signature,
    /// The headers a forward repeats, as they arrived: `Content-Type`, and
    /// the signature header that established [`Event::signature`]. Names are
    /// lowercase; a header whose value is not visible ASCII is left out.
    pub headers: Vec<(String, String)>,
    /// When the request arrived.
    pub received_at: SystemTime,
}

impl Event {
    /// The SHA-256 of [`Event::body`] as 64 lowercase hex characters: a key
    /// a target deduplicates on, the same for every delivery of the same
    /// bytes, a replay included.
    #[must_use]
    pub fn sha256(&self) -> String {
        use sha2::Digest as _;
        use std::fmt::Write as _;
        let mut hex = String::with_capacity(64);
        for byte in sha2::Sha256::digest(&self.body) {
            let _ = write!(hex, "{byte:02x}");
        }
        hex
    }
}

/// A bound webhook receiver.
///
/// Dropping it stops the listener, ends every connection it is serving but
/// those of the deliveries whose event the caller took, and releases the
/// port. Those go on to write their 200 for as long as the runtime runs, so a
/// caller that ends the runtime or the process next stops it with
/// [`Receiver::close`] instead. The token given to [`Receiver::bind`] stays as
/// the caller left it.
///
/// # Example
///
/// ```rust,no_run
/// use tokio_util::sync::CancellationToken;
/// use xdk::webhooks::{Receiver, ReceiverConfig, SigningSecrets};
///
/// #[tokio::main]
/// async fn main() -> xdk::Result<()> {
///     let secrets = SigningSecrets {
///         oauth2_client_secret: Some("the app's client secret".to_string()),
///         oauth1_consumer_secret: None,
///     };
///     let bind = "127.0.0.1:8080".parse().expect("an address");
///     let config = ReceiverConfig::new(bind, secrets);
///     // Binding starts answering X's CRC check.
///     let mut receiver = Receiver::bind(config, CancellationToken::new()).await?;
///     println!("answering on {}", receiver.local_addr());
///     while let Some(event) = receiver.next_event().await {
///         println!("{} bytes at {:?}", event.body.len(), event.received_at);
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Receiver {
    local_addr: SocketAddr,
    events: mpsc::Receiver<Delivery>,
    // A child of the caller's token: the caller's stop reaches every task
    // through it, and `Drop` cancels it without touching the caller's token.
    stop: CancellationToken,
    task: tokio::task::JoinHandle<()>,
    connections: TaskTracker,
}

/// What a receiver stamps each [`Event::received_at`] with.
// `xr`'s in-process test runner passes one through `RunHooks`
// (`crates/xurl-cli/src/cli/runner.rs`) to `webhooks listen`
// (`crates/xurl-cli/src/cli/commands/webhooks_listen.rs`), so a test can put
// two arrivals in different minutes without waiting for the clock.
// `Receiver::bind` reads `SystemTime::now`.
#[doc(hidden)]
pub type ArrivalClock = Arc<dyn Fn() -> SystemTime + Send + Sync>;

impl Receiver {
    /// Binds `config.bind` and starts answering.
    ///
    /// `cancel` is the caller's stop: once it is cancelled the listener
    /// closes and [`Receiver::next_event`] returns `None`. The receiver never
    /// cancels it itself, even when it is dropped, so the caller can pass a
    /// token it shares with other work, a signal handler's among them. The
    /// library registers no signal handler.
    ///
    /// # Errors
    ///
    /// Returns a `validation` error when `config.secrets` holds no secret,
    /// since the CRC check cannot be answered without one,
    /// [`Error::AddressInUse`] when another socket holds the address, and an
    /// `io` error when it cannot be bound for any other reason.
    pub async fn bind(config: ReceiverConfig, cancel: CancellationToken) -> Result<Self> {
        Self::bind_with_clock(config, cancel, Arc::new(SystemTime::now)).await
    }

    /// [`Receiver::bind`], stamping each arrival with `clock`. `cancel` is the
    /// caller's stop here too: the receiver never cancels it itself.
    // `webhooks listen` calls this when the in-process runner gives it an
    // `ArrivalClock`; see that type.
    #[doc(hidden)]
    pub async fn bind_with_clock(
        config: ReceiverConfig,
        cancel: CancellationToken,
        clock: ArrivalClock,
    ) -> Result<Self> {
        if config.secrets.is_empty() {
            return Err(Error::validation("no-signing-secret"));
        }
        let listener = TcpListener::bind(config.bind).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                Error::AddressInUse {
                    address: config.bind,
                    source: e,
                }
            } else {
                Error::io(format!("could not bind {}: {e}", config.bind))
            }
        })?;
        let local_addr = listener
            .local_addr()
            .map_err(|e| Error::io(format!("could not read the bound address: {e}")))?;
        let (sender, events) = mpsc::channel(EVENT_BUFFER);
        let shared = Arc::new(Shared {
            config,
            sender,
            clock,
        });
        let stop = cancel.child_token();
        let connections = TaskTracker::new();
        let task = tokio::spawn(accept_loop(
            listener,
            shared,
            stop.clone(),
            connections.clone(),
        ));
        Ok(Self {
            local_addr,
            events,
            stop,
            task,
            connections,
        })
    }

    /// The address the receiver is bound to.
    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// The next event whose signature verified, or `None` once the receiver
    /// has been cancelled.
    ///
    /// X's delivery of an event is answered 200 when this returns it. An
    /// event still waiting when the receiver stops is answered 503, or its
    /// connection closes unanswered, so X is never told an event arrived
    /// that the caller did not take. X waits ten seconds for its answer,
    /// and every event behind one the caller is still working on waits
    /// unanswered, so a caller that does slow work on an event, such as a
    /// forward, hands the event to a task or a channel before it calls this
    /// again.
    pub async fn next_event(&mut self) -> Option<Event> {
        let delivery = tokio::select! {
            biased;
            delivery = self.events.recv() => delivery,
            () = self.stop.cancelled() => None,
        }?;
        delivery.taken.store(true, Ordering::Release);
        let _ = delivery.answer.send(());
        Some(delivery.event)
    }

    /// Stops the receiver as dropping it does, and returns once each delivery
    /// whose event the caller took has written its 200, or has had ten
    /// seconds to, X's own wait for an answer.
    pub async fn close(mut self) {
        self.stop.cancel();
        let _ = (&mut self.task).await;
        self.connections.close();
        self.connections.wait().await;
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.stop.cancel();
        self.task.abort();
    }
}

struct Shared {
    config: ReceiverConfig,
    sender: mpsc::Sender<Delivery>,
    clock: ArrivalClock,
}

/// A verified event on its way to the caller. `answer` wakes the request
/// waiting to answer X; `taken` tells the request's connection, which a stop
/// can reach before that answer is written, that X is owed it.
struct Delivery {
    event: Event,
    answer: oneshot::Sender<()>,
    taken: Arc<AtomicBool>,
}

async fn accept_loop(
    listener: TcpListener,
    shared: Arc<Shared>,
    cancel: CancellationToken,
    connections: TaskTracker,
) {
    // At least one slot, so a config that asks for none still serves.
    let slots = Arc::new(Semaphore::new(shared.config.max_connections.max(1)));
    loop {
        // The slot is taken before the connection is accepted, so a
        // connection past the limit waits in the listener's backlog.
        let slot = tokio::select! {
            () = cancel.cancelled() => return,
            slot = Arc::clone(&slots).acquire_owned() => slot,
        };
        let Ok(slot) = slot else {
            return;
        };
        let accepted = tokio::select! {
            () = cancel.cancelled() => return,
            accepted = listener.accept() => accepted,
        };
        let Ok((stream, _)) = accepted else {
            tracing::warn!(target: WEBHOOKS_TARGET, kind = "accept-failed");
            tokio::time::sleep(ACCEPT_RETRY_DELAY).await;
            continue;
        };
        let shared = Arc::clone(&shared);
        let cancel = cancel.clone();
        connections.spawn(async move {
            let _slot = slot;
            // One request per connection (keep-alive is off), so one flag.
            let taken = Arc::new(AtomicBool::new(false));
            let service = {
                let taken = Arc::clone(&taken);
                service_fn(move |request| {
                    let shared = Arc::clone(&shared);
                    let taken = Arc::clone(&taken);
                    async move { Ok::<_, Infallible>(respond(&shared, request, taken).await) }
                })
            };
            let connection = hyper::server::conn::http1::Builder::new()
                .timer(TokioTimer::new())
                .header_read_timeout(REQUEST_TIMEOUT)
                .keep_alive(false)
                .serve_connection(TokioIo::new(stream), service);
            tokio::pin!(connection);
            tokio::select! {
                () = cancel.cancelled() => {}
                _ = &mut connection => return,
            }
            if taken.load(Ordering::Acquire) {
                let _ = tokio::time::timeout(REQUEST_TIMEOUT, connection).await;
            }
        });
    }
}

async fn respond(
    shared: &Shared,
    request: Request<Incoming>,
    taken: Arc<AtomicBool>,
) -> Response<Full<Bytes>> {
    if request.uri().path() != shared.config.path {
        return plain(StatusCode::NOT_FOUND, "Not Found");
    }
    if request.method() == Method::GET {
        return answer_crc(shared, &request);
    }
    if request.method() == Method::POST {
        return receive_event(shared, request, taken).await;
    }
    let mut response = plain(StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed");
    response
        .headers_mut()
        .insert(ALLOW, hyper::header::HeaderValue::from_static("GET, POST"));
    response
}

fn answer_crc(shared: &Shared, request: &Request<Incoming>) -> Response<Full<Bytes>> {
    let token = request.uri().query().and_then(|query| {
        url::form_urlencoded::parse(query.as_bytes())
            .find(|(name, _)| name == "crc_token")
            .map(|(_, value)| value.into_owned())
    });
    let Some(token) = token.filter(|t| !t.is_empty()) else {
        tracing::warn!(target: WEBHOOKS_TARGET, kind = "crc-token-missing");
        return plain(StatusCode::BAD_REQUEST, "Missing crc_token");
    };
    // `Receiver::bind` refuses a config with no secret, so the first arm of
    // this match is the only one a bound receiver reaches.
    let response_token = match shared.config.secrets.crc_secret() {
        Some(secret) => sign(secret, token.as_bytes()),
        None => Err(Error::Internal("no signing secret".to_string())),
    };
    let Ok(response_token) = response_token else {
        tracing::error!(target: WEBHOOKS_TARGET, kind = "crc-not-signed");
        return plain(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error");
    };
    let body = serde_json::json!({ "response_token": response_token });
    tracing::info!(target: WEBHOOKS_TARGET, kind = "crc-answered");
    let mut response = Response::new(Full::new(Bytes::from(body.to_string())));
    response.headers_mut().insert(
        CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );
    response
}

async fn receive_event(
    shared: &Shared,
    request: Request<Incoming>,
    taken: Arc<AtomicBool>,
) -> Response<Full<Bytes>> {
    let received_at = (shared.clock)();
    let header = |name: &str| {
        request
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
    };
    let oauth2_signature = header(OAUTH2_SIGNATURE_HEADER);
    let legacy_signature = header(LEGACY_SIGNATURE_HEADER);
    let content_type = header(CONTENT_TYPE.as_str());

    let limited = Limited::new(request.into_body(), shared.config.max_body_bytes);
    let body = match tokio::time::timeout(REQUEST_TIMEOUT, limited.collect()).await {
        Ok(Ok(collected)) => collected.to_bytes(),
        Ok(Err(_)) => {
            tracing::warn!(target: WEBHOOKS_TARGET, kind = "event-too-large");
            return plain(StatusCode::PAYLOAD_TOO_LARGE, "Payload Too Large");
        }
        Err(_) => return plain(StatusCode::REQUEST_TIMEOUT, "Request Timeout"),
    };

    let secrets = &shared.config.secrets;
    // The OAuth2 header decides whenever it is present, as X's documentation
    // says; the legacy header is read only in its absence.
    let signature = if let Some(given) = &oauth2_signature {
        verified(secrets.oauth2_client_secret.as_deref(), &body, given).then_some(Signature::OAuth2)
    } else if let Some(given) = &legacy_signature {
        verified(secrets.oauth1_consumer_secret.as_deref(), &body, given)
            .then_some(Signature::OAuth1)
    } else if shared.config.allow_unsigned {
        Some(Signature::Unsigned)
    } else {
        tracing::warn!(target: WEBHOOKS_TARGET, kind = "signature-missing");
        return plain(StatusCode::UNAUTHORIZED, "Missing signature");
    };
    let Some(signature) = signature else {
        tracing::warn!(target: WEBHOOKS_TARGET, kind = "signature-invalid");
        return plain(StatusCode::UNAUTHORIZED, "Invalid signature");
    };

    let signature_header = match signature {
        Signature::OAuth2 => oauth2_signature.map(|value| (OAUTH2_SIGNATURE_HEADER, value)),
        Signature::OAuth1 => legacy_signature.map(|value| (LEGACY_SIGNATURE_HEADER, value)),
        Signature::Unsigned => None,
    };
    let headers = content_type
        .map(|value| (CONTENT_TYPE.as_str(), value))
        .into_iter()
        .chain(signature_header)
        .map(|(name, value)| (name.to_string(), value))
        .collect();
    let event = Event {
        body: body.to_vec(),
        signature,
        headers,
        received_at,
    };
    let (answer, answered) = oneshot::channel();
    let delivery = Delivery {
        event,
        answer,
        taken,
    };
    if shared.sender.send(delivery).await.is_err() || answered.await.is_err() {
        return plain(StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable");
    }
    Response::new(Full::new(Bytes::new()))
}

fn plain(status: StatusCode, body: &'static str) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::from_static(body.as_bytes())));
    *response.status_mut() = status;
    response
}
