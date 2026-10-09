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
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Incoming;
use hyper::header::{ALLOW, CONTENT_TYPE};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::{TokioIo, TokioTimer};
use tokio::net::TcpListener;
use tokio::sync::{Semaphore, mpsc};
use tokio_util::sync::CancellationToken;

use crate::error::{Error, Result};

mod signing;

use signing::{LEGACY_SIGNATURE_HEADER, OAUTH2_SIGNATURE_HEADER, verified};
pub use signing::{SigningSecrets, sign};

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

/// Events held for a caller that is slower than X delivers.
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
}

/// A bound webhook receiver.
///
/// Dropping it stops the listener and releases the port.
#[derive(Debug)]
pub struct Receiver {
    local_addr: SocketAddr,
    events: mpsc::Receiver<Event>,
    cancel: CancellationToken,
    task: tokio::task::JoinHandle<()>,
}

impl Receiver {
    /// Binds `config.bind` and starts answering.
    ///
    /// `cancel` is the external stop: once it is cancelled the listener
    /// closes and [`Receiver::next_event`] returns `None`. The library
    /// registers no signal handler.
    ///
    /// # Errors
    ///
    /// Returns a `validation` error when `config.secrets` holds no secret,
    /// since the CRC check cannot be answered without one, and an `io` error
    /// when the address cannot be bound.
    pub async fn bind(config: ReceiverConfig, cancel: CancellationToken) -> Result<Self> {
        if config.secrets.is_empty() {
            return Err(Error::validation(
                "a webhook receiver needs the app's OAuth2 client secret or OAuth1 consumer secret",
            ));
        }
        let listener = TcpListener::bind(config.bind)
            .await
            .map_err(|e| Error::io(format!("could not bind {}: {e}", config.bind)))?;
        let local_addr = listener
            .local_addr()
            .map_err(|e| Error::io(format!("could not read the bound address: {e}")))?;
        let (sender, events) = mpsc::channel(EVENT_BUFFER);
        let shared = Arc::new(Shared { config, sender });
        let task = tokio::spawn(accept_loop(listener, shared, cancel.clone()));
        Ok(Self {
            local_addr,
            events,
            cancel,
            task,
        })
    }

    /// The address the receiver is bound to.
    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// The next event whose signature verified, or `None` once the receiver
    /// has been cancelled.
    pub async fn next_event(&mut self) -> Option<Event> {
        tokio::select! {
            biased;
            event = self.events.recv() => event,
            () = self.cancel.cancelled() => None,
        }
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.cancel.cancel();
        self.task.abort();
    }
}

struct Shared {
    config: ReceiverConfig,
    sender: mpsc::Sender<Event>,
}

async fn accept_loop(listener: TcpListener, shared: Arc<Shared>, cancel: CancellationToken) {
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
        tokio::spawn(async move {
            let _slot = slot;
            let service = service_fn(move |request| {
                let shared = Arc::clone(&shared);
                async move { Ok::<_, Infallible>(respond(&shared, request).await) }
            });
            let connection = hyper::server::conn::http1::Builder::new()
                .timer(TokioTimer::new())
                .header_read_timeout(REQUEST_TIMEOUT)
                .keep_alive(false)
                .serve_connection(TokioIo::new(stream), service);
            tokio::select! {
                () = cancel.cancelled() => {}
                _ = connection => {}
            }
        });
    }
}

async fn respond(shared: &Shared, request: Request<Incoming>) -> Response<Full<Bytes>> {
    if request.uri().path() != shared.config.path {
        return plain(StatusCode::NOT_FOUND, "Not Found");
    }
    if request.method() == Method::GET {
        return answer_crc(shared, &request);
    }
    if request.method() == Method::POST {
        return receive_event(shared, request).await;
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
        None => Err(Error::validation("no signing secret")),
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

async fn receive_event(shared: &Shared, request: Request<Incoming>) -> Response<Full<Bytes>> {
    let header = |name: &str| {
        request
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
    };
    let oauth2_signature = header(OAUTH2_SIGNATURE_HEADER);
    let legacy_signature = header(LEGACY_SIGNATURE_HEADER);

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
    let signature = if let Some(given) = oauth2_signature {
        verified(secrets.oauth2_client_secret.as_deref(), &body, &given)
            .then_some(Signature::OAuth2)
    } else if let Some(given) = legacy_signature {
        verified(secrets.oauth1_consumer_secret.as_deref(), &body, &given)
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

    let event = Event {
        body: body.to_vec(),
        signature,
    };
    if shared.sender.send(event).await.is_err() {
        return plain(StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable");
    }
    Response::new(Full::new(Bytes::new()))
}

fn plain(status: StatusCode, body: &'static str) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::from_static(body.as_bytes())));
    *response.status_mut() = status;
    response
}
