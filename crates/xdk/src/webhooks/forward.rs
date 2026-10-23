//! Forwarding a verified event to the caller's own HTTP endpoint.

use std::fmt;
use std::time::Duration;

use reqwest::header::{HeaderName, HeaderValue};

use super::budget::{Attempt, AttemptTimedOut, Outcome, RetryBudget};
use super::{Event, WEBHOOKS_TARGET};
use crate::error::{Error, Result};

/// How much of a target's answer a forward reads before letting the
/// connection go, and how long it spends reading it. The answer is
/// discarded: only its status decides the delivery.
const RESPONSE_READ_CAP: usize = 64 * 1024;
const RESPONSE_READ_TIME: Duration = Duration::from_secs(1);

/// Headers a forward sets itself, so a caller's header may not.
const RESERVED_HEADERS: [&str; 3] = ["host", "content-length", "idempotency-key"];

/// Why a forward to a URL failed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ForwardFailure {
    /// The target answered with a status outside 2xx. 408, 429, and 5xx are
    /// retried; any other, a 3xx included, is final, since a forward follows
    /// no redirect.
    Status(u16),
    /// No answer arrived: the connection failed or broke. The cause never
    /// names the URL.
    Connection(String),
    /// The attempt outlasted its timeout.
    TimedOut(Duration),
}

impl fmt::Display for ForwardFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status(status) if (300..400).contains(status) => write!(
                f,
                "the target answered {status}, a redirect, which a forward does not follow"
            ),
            Self::Status(status) => write!(f, "the target answered {status}"),
            Self::Connection(cause) => write!(f, "no answer: {cause}"),
            Self::TimedOut(after) => write!(f, "no answer within {after:?}"),
        }
    }
}

impl std::error::Error for ForwardFailure {}

impl From<AttemptTimedOut> for ForwardFailure {
    fn from(timed_out: AttemptTimedOut) -> Self {
        Self::TimedOut(timed_out.0)
    }
}

/// The caller's HTTP endpoint, validated once and sent to under a
/// [`RetryBudget`].
///
/// A forward is a `POST` of the event's bytes as X sent them, with the
/// `Content-Type` and signature header X sent, so the target verifies the
/// signature with the app's own secret. It adds `Idempotency-Key`: the
/// SHA-256 of the bytes as quoted lowercase hex, the same for a retry and for
/// a replay of the same event, then the caller's headers. A 2xx from the URL
/// itself delivers; a redirect is not followed.
///
/// `Debug` prints the URL and the names of the caller's headers, never their
/// values.
///
/// # Example
///
/// ```rust,no_run
/// use tokio_util::sync::CancellationToken;
/// use xdk::webhooks::{Receiver, ReceiverConfig, RetryBudget, SigningSecrets, UrlTarget};
///
/// #[tokio::main]
/// async fn main() -> xdk::Result<()> {
///     let secrets = SigningSecrets {
///         oauth2_client_secret: Some("the app's client secret".to_string()),
///         oauth1_consumer_secret: None,
///     };
///     let bind = "127.0.0.1:8080".parse().expect("an address");
///     let mut receiver =
///         Receiver::bind(ReceiverConfig::new(bind, secrets), CancellationToken::new()).await?;
///     let target = UrlTarget::new("http://127.0.0.1:3000/events", &["X-Team: blue"])?;
///     let budget = RetryBudget::default();
///     while let Some(event) = receiver.next_event().await {
///         let outcome = target.forward(&event, &budget).await;
///         println!("{} attempt(s): {:?}", outcome.attempts, outcome.result);
///     }
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct UrlTarget {
    url: reqwest::Url,
    headers: Vec<(HeaderName, HeaderValue)>,
    client: reqwest::Client,
}

impl fmt::Debug for UrlTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self.headers.iter().map(|(name, _)| name.as_str()).collect();
        f.debug_struct("UrlTarget")
            .field("url", &self.url.as_str())
            .field("headers", &names)
            .finish_non_exhaustive()
    }
}

impl UrlTarget {
    /// The endpoint at `url`, sent `headers`, each a `Name: Value` line.
    ///
    /// # Errors
    ///
    /// Returns a `validation` error, before anything is sent, for a URL that
    /// is not `http` or `https`, one that carries a username or password, and
    /// a header with no colon, an empty name, or a name or value HTTP does
    /// not allow, or one that sets `Host`, `Content-Length`, or
    /// `Idempotency-Key`. A header is named by its position, never by its
    /// value, and no message repeats the URL. An `http` error when the HTTP
    /// client cannot be built.
    pub fn new(url: &str, headers: &[impl AsRef<str>]) -> Result<Self> {
        let url = reqwest::Url::parse(url).map_err(|_| {
            Error::validation("the forward URL is not an absolute http or https URL")
        })?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(Error::validation(format!(
                "the forward URL's scheme is {}; a forward takes http or https",
                url.scheme()
            )));
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err(Error::validation(
                "the forward URL carries a username or password; send a credential in a header \
                 such as Authorization instead",
            ));
        }
        let headers = headers
            .iter()
            .enumerate()
            .map(|(index, line)| caller_header(index + 1, line.as_ref()))
            .collect::<Result<Vec<_>>>()?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| {
                Error::http(format!("cannot build the HTTP client: {e}")).with_source(e)
            })?;
        Ok(Self {
            url,
            headers,
            client,
        })
    }

    /// The URL events go to.
    #[must_use]
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    /// Forwards `event` under `budget`.
    pub async fn forward(
        &self,
        event: &Event,
        budget: &RetryBudget,
    ) -> Outcome<u16, ForwardFailure> {
        let outcome = budget.run(|| self.attempt(event)).await;
        match &outcome.result {
            Ok(status) => tracing::info!(
                target: WEBHOOKS_TARGET,
                kind = "forward-delivered",
                status,
                attempts = outcome.attempts,
            ),
            Err(failure) => tracing::warn!(
                target: WEBHOOKS_TARGET,
                kind = "forward-failed",
                attempts = outcome.attempts,
                "forward failed: {failure}",
            ),
        }
        outcome
    }

    /// One attempt at sending `event`, with no timeout of its own.
    pub async fn attempt(&self, event: &Event) -> Attempt<u16, ForwardFailure> {
        let mut request = self.client.post(self.url.clone()).body(event.body.clone());
        for (name, value) in &event.headers {
            request = request.header(name.as_str(), value.as_str());
        }
        request = request.header("idempotency-key", format!("\"{}\"", event.sha256()));
        for (name, value) in &self.headers {
            request = request.header(name.clone(), value.clone());
        }
        let mut response = match request.send().await {
            Ok(response) => response,
            Err(e) => return Attempt::Retryable(ForwardFailure::Connection(cause_of(e))),
        };
        let status = response.status().as_u16();
        let _ = tokio::time::timeout(RESPONSE_READ_TIME, async {
            let mut read = 0;
            while read <= RESPONSE_READ_CAP {
                match response.chunk().await {
                    Ok(Some(chunk)) => read += chunk.len(),
                    _ => break,
                }
            }
        })
        .await;
        match status {
            200..=299 => Attempt::Delivered(status),
            408 | 429 | 500..=599 => Attempt::Retryable(ForwardFailure::Status(status)),
            _ => Attempt::Final(ForwardFailure::Status(status)),
        }
    }
}

/// One caller header, from its `Name: Value` line at `position` (from 1).
fn caller_header(position: usize, line: &str) -> Result<(HeaderName, HeaderValue)> {
    let refuse = |why: &str| Error::validation(format!("header {position} {why}"));
    let Some((name, value)) = line.split_once(':') else {
        return Err(refuse("is not `Name: Value`: it has no colon"));
    };
    let name = name.trim();
    if name.is_empty() {
        return Err(refuse("has an empty name"));
    }
    let name = HeaderName::from_bytes(name.as_bytes())
        .map_err(|_| refuse("has a name HTTP does not allow"))?;
    if RESERVED_HEADERS.contains(&name.as_str()) {
        return Err(refuse(&format!("sets {name}, which a forward sets itself")));
    }
    let value = HeaderValue::from_str(value.trim())
        .map_err(|_| refuse("has a value HTTP does not allow"))?;
    Ok((name, value))
}

/// A send failure and its causes, without the URL.
fn cause_of(error: reqwest::Error) -> String {
    let error = error.without_url();
    let mut cause = error.to_string();
    let mut source = std::error::Error::source(&error);
    while let Some(inner) = source {
        cause.push_str(": ");
        cause.push_str(&inner.to_string());
        source = inner.source();
    }
    cause
}
