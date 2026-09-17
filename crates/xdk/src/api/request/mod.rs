//! HTTP request building and execution for the X API.
//!
//! A [`Client`] holds the one `reqwest::Client` every request shares and the
//! credentials that sign them; shortcuts on it return a [`Call`] that is
//! configured and then sent.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};

use crate::auth::Auth;
use crate::config::Config;
use crate::error::{Error, Result};

mod auth_header;
mod builder;
mod call;
mod source;
mod transport;
mod url;

pub use builder::ClientBuilder;
pub use call::Call;
pub(crate) use source::CredentialSource;
pub use transport::{StreamLines, WIRE_TARGET};
pub(crate) use url::render_template_path;
use url::{build_url_for_target, render_template_template};

/// Typed target for an HTTP request.
///
/// Either a template path with substitutable `{name}` segments plus an
/// ordered query list (`Template`), or a fully-formed external URL that
/// bypasses the auth matrix (`RawUrl`). The split is the v2.0.0 contract
/// that lets the matrix validator reason about `(method, path)` without
/// re-parsing already-rendered URLs.
#[derive(Debug, Clone)]
pub enum RequestTarget {
    /// API path template — e.g. `"/2/users/{id}/likes"` — paired with
    /// `path_params` to substitute and a `query` vec whose insertion
    /// order is preserved on render.
    Template {
        /// Path template containing `{name}` segments to substitute. Must
        /// match the spec verbatim (the auth matrix is keyed on this string).
        path: String,
        /// Map of `{name}` segments to caller-supplied values.
        path_params: HashMap<String, String>,
        /// Ordered query parameters. Each `(key, value)` pair is
        /// percent-encoded and joined with `&`; empty `query` produces no
        /// `?` on render.
        query: Vec<(String, String)>,
    },
    /// Fully-formed URL — used by raw mode (`xr <URL>`) and by shortcuts
    /// whose path is intentionally outside the spec. The matrix validator
    /// short-circuits for `RawUrl` — the user accepted the contract by
    /// reaching for raw mode.
    RawUrl(String),
}

impl Default for RequestTarget {
    fn default() -> Self {
        Self::Template {
            path: String::new(),
            path_params: HashMap::new(),
            query: Vec::new(),
        }
    }
}

/// Common options for API requests.
///
/// Threaded into [`Client::send_request`], [`Client::send_multipart_request`],
/// and [`Client::stream_request`]; carries everything those calls need
/// beyond the client itself.
#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    /// HTTP method (`"GET"`, `"POST"`, etc). Empty defaults to `"GET"`.
    pub method: String,
    /// Typed request target — either a path template or a raw URL.
    pub target: RequestTarget,
    /// Extra HTTP headers in `"Name: Value"` form.
    pub headers: Vec<String>,
    /// Request body. JSON-shaped strings are sent as `application/json`;
    /// otherwise as `application/x-www-form-urlencoded`.
    pub data: String,
    /// Explicit auth scheme — `"oauth1"`, `"oauth2"`, `"app"`, or empty for
    /// auto-detect against the endpoint's accepted set.
    pub auth_type: String,
    /// OAuth2 username for the active app. Empty selects the active app's
    /// first stored OAuth2 token.
    pub username: String,
    /// Skip auth-header attachment entirely. Used for unauthenticated probes.
    pub no_auth: bool,
    /// Emit the `X-B3-Flags: 1` header to flag the request for upstream tracing.
    pub trace: bool,
}

/// Default request timeout in seconds when none is supplied.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// The per-call options a [`Call`] carries until it is sent.
#[derive(Debug, Clone, Default)]
pub(crate) struct CallOptions {
    /// Explicit auth scheme wire string; empty for auto-detect.
    pub(crate) auth_type: String,
    /// `OAuth2` username for a store-backed client; empty selects the
    /// active app's first stored token.
    pub(crate) username: String,
    /// Emit the `X-B3-Flags: 1` header.
    pub(crate) trace: bool,
    /// Send no `Authorization` header and skip scheme selection.
    pub(crate) no_auth: bool,
    /// Per-call bound; `None` inherits the client-level timeout.
    pub(crate) timeout: Option<Duration>,
    /// Cursor for list endpoints; single-item endpoints ignore it.
    pub(crate) pagination_token: String,
}

/// Options specific to multipart requests.
///
/// Used by the media upload path to ship a single file plus form fields
/// in one `multipart/form-data` POST.
#[derive(Debug, Clone)]
pub struct MultipartOptions {
    /// Base request configuration (target, headers, auth, trace).
    pub request: RequestOptions,
    /// Non-file form fields keyed by field name.
    pub form_fields: std::collections::HashMap<String, String>,
    /// Form field name to attach the file under (e.g. `"media"`).
    pub file_field: String,
    /// Path to the file to upload. Mutually exclusive with `file_data`.
    pub file_path: String,
    /// Filename surfaced to the server in the multipart part header. Only
    /// consulted when `file_data` carries the bytes.
    pub file_name: String,
    /// In-memory file bytes. Mutually exclusive with `file_path`.
    pub file_data: Vec<u8>,
}

/// Sends authenticated X API requests.
///
/// A `Client` is a handle over shared state: cloning it is an `Arc`
/// increment, every method takes `&self`, and one clone per task is the
/// intended shape. Build one from a credential held in code with
/// [`Client::builder`], or from a token store with [`Client::new`].
///
/// # Example
///
/// ```rust,no_run
/// use xdk::api::Client;
///
/// # async fn run() -> xdk::Result<()> {
/// let client = Client::builder().bearer("app-only-token").build()?;
/// let posts = client.search_posts("rustlang", 10).send().await?;
/// for post in &posts.data {
///     println!("{}: {}", post.id, post.text);
/// }
/// if let Some(window) = client.last_rate_limit() {
///     println!("{:?} requests left until {:?}", window.remaining, window.reset_at);
/// }
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

/// The `User-Agent` a client sends when the caller sets none: the library's
/// own name, so an embedder that never thinks about it is still identifiable.
pub const DEFAULT_USER_AGENT: &str = concat!("xdk-rs/", env!("CARGO_PKG_VERSION"));

/// The rate-limit window X reported on the most recent response, from the
/// `x-rate-limit-limit`, `x-rate-limit-remaining`, and `x-rate-limit-reset`
/// headers. A field is `None` when the response did not carry that header.
///
/// Read it through [`Client::last_rate_limit`] before retrying a failed call,
/// so a retry loop can wait for `reset_at` instead of spending requests that
/// will be refused.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimit {
    /// Requests allowed in the current window.
    pub limit: Option<u32>,
    /// Requests left in the current window.
    pub remaining: Option<u32>,
    /// When the window resets, as seconds since the Unix epoch.
    pub reset_at: Option<u64>,
}

impl RateLimit {
    /// Reads the three `x-rate-limit-*` headers; `None` when the response
    /// carried none of them, so an unrelated response never erases the last
    /// window a rate-limited endpoint reported.
    fn from_headers(headers: &reqwest::header::HeaderMap) -> Option<Self> {
        fn number<T: std::str::FromStr>(
            headers: &reqwest::header::HeaderMap,
            name: &str,
        ) -> Option<T> {
            headers.get(name)?.to_str().ok()?.trim().parse().ok()
        }
        let window = Self {
            limit: number(headers, "x-rate-limit-limit"),
            remaining: number(headers, "x-rate-limit-remaining"),
            reset_at: number(headers, "x-rate-limit-reset"),
        };
        (window.limit.is_some() || window.remaining.is_some() || window.reset_at.is_some())
            .then_some(window)
    }
}

/// What every clone of a [`Client`] shares: the one HTTP client, the base
/// URL and timeout, the credential state behind the lock a refresh holds
/// while it rotates a token, and the last rate-limit window seen.
struct Inner {
    base_url: String,
    http: reqwest::Client,
    credentials: Mutex<CredentialSource>,
    timeout: Duration,
    user_agent: String,
    rate_limit: std::sync::Mutex<Option<RateLimit>>,
}

crate::assert_send_sync!(Client);

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.inner.base_url)
            .field("timeout", &self.inner.timeout)
            .field("user_agent", &self.inner.user_agent)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Starts a client from credentials held in code.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Creates a client over a token store, using the timeout configured on
    /// `config`.
    ///
    /// The CLI runner writes `--timeout` / `XURL_TIMEOUT` into
    /// [`Config::http_timeout_secs`]; library consumers that want a different
    /// timeout can use [`Client::with_timeout`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] when the HTTP client cannot be built.
    pub fn new(config: &Config, auth: Auth) -> Result<Self> {
        Self::with_timeout(config, auth, config.http_timeout_secs)
    }

    /// Creates a client over a token store that identifies itself as
    /// `user_agent` on every request, with the timeout from `config`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] when the HTTP client cannot be built.
    pub fn new_with_user_agent(
        config: &Config,
        auth: Auth,
        user_agent: impl Into<String>,
    ) -> Result<Self> {
        Self::from_source(
            config.api_base_url.clone(),
            CredentialSource::Store(auth),
            Duration::from_secs(config.http_timeout_secs),
            user_agent.into(),
        )
    }

    /// Creates a client over a token store with an explicit request timeout.
    ///
    /// The timeout bounds every non-streaming HTTP call dispatched by this
    /// client, and the token exchange, refresh, and `/2/users/me` lookups
    /// that share its connection pool. Streaming requests carry no total
    /// timeout: the long-running shape is the point.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] when the HTTP client cannot be built. A client
    /// that cannot honor its configured timeout is a startup failure, not a
    /// silently unbounded client.
    pub fn with_timeout(config: &Config, auth: Auth, timeout_secs: u64) -> Result<Self> {
        Self::from_source(
            config.api_base_url.clone(),
            CredentialSource::Store(auth),
            Duration::from_secs(timeout_secs),
            DEFAULT_USER_AGENT.to_string(),
        )
    }

    /// The one construction site: builds the HTTP client every request,
    /// token exchange, refresh, and `/2/users/me` lookup goes through.
    pub(crate) fn from_source(
        base_url: String,
        credentials: CredentialSource,
        timeout: Duration,
        user_agent: String,
    ) -> Result<Self> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| Error::Http(format!("cannot build the HTTP client: {e}")))?;

        Ok(Self {
            inner: Arc::new(Inner {
                base_url,
                http,
                credentials: Mutex::new(credentials),
                timeout,
                user_agent,
                rate_limit: std::sync::Mutex::new(None),
            }),
        })
    }

    /// The rate-limit window X reported on the most recent response that
    /// carried one, or `None` before any such response. Usage credits are a
    /// separate, account-wide figure: read them with
    /// [`Client::get_usage_credits`](crate::api::shortcuts).
    #[must_use]
    pub fn last_rate_limit(&self) -> Option<RateLimit> {
        *self
            .inner
            .rate_limit
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Records the window a response reported; a response without the
    /// headers leaves the last window in place.
    pub(crate) fn record_rate_limit(&self, headers: &reqwest::header::HeaderMap) {
        if let Some(window) = RateLimit::from_headers(headers) {
            *self
                .inner
                .rate_limit
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(window);
        }
    }

    /// Returns the per-call timeout used by this client (seconds).
    #[must_use]
    pub fn timeout_secs(&self) -> u64 {
        self.inner.timeout.as_secs()
    }

    /// The per-request bound every non-streaming call carries.
    pub(crate) fn request_timeout(&self) -> Duration {
        self.inner.timeout
    }

    /// The one HTTP client every request, token exchange, refresh, and
    /// `/2/users/me` lookup goes through.
    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.inner.http
    }

    /// Locks and returns the credential state.
    pub(crate) async fn credentials(&self) -> MutexGuard<'_, CredentialSource> {
        self.inner.credentials.lock().await
    }

    /// Runs the `OAuth2` PKCE sign-in for `username` on this client's
    /// connection, holding the credential lock for the whole flow.
    ///
    /// See [`Auth::oauth2_flow`] for the opener and the cancellation token.
    ///
    /// # Errors
    ///
    /// Everything [`Auth::oauth2_flow`] returns, and [`Error::Validation`]
    /// when the client was built from credentials rather than a store.
    pub async fn oauth2_flow<F>(
        &self,
        username: &str,
        cancel: tokio_util::sync::CancellationToken,
        browser_opener: F,
    ) -> Result<String>
    where
        F: Fn(&str) -> std::io::Result<()> + Send + Sync + 'static,
    {
        let mut auth = self.auth().await?;
        auth.oauth2_flow(self.http(), username, cancel, browser_opener)
            .await
    }

    /// Completes the headless `OAuth2` flow from the redirect URL the user
    /// pasted, on this client's connection.
    ///
    /// # Errors
    ///
    /// Everything [`Auth::remote_oauth2_step2`] returns, and
    /// [`Error::Validation`] when the client was built from credentials
    /// rather than a store.
    pub async fn remote_oauth2_step2(
        &self,
        redirect_url: &str,
        username: &str,
        pending_path: &std::path::Path,
    ) -> Result<String> {
        let mut auth = self.auth().await?;
        auth.remote_oauth2_step2(self.http(), redirect_url, username, pending_path)
            .await
    }

    /// Locks and returns the token-store credential state.
    ///
    /// A refresh holds this lock while it rotates a token, so hold the guard
    /// only for the store read or write at hand and never across a request
    /// on the same client. The one deliberate exception is sign-in:
    /// [`Client::oauth2_flow`] and [`Client::remote_oauth2_step2`] hold it for
    /// the whole flow so no request on another clone can race a
    /// half-installed credential.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the client was built from
    /// credentials held in code: there is no store behind it.
    pub async fn auth(&self) -> Result<MappedMutexGuard<'_, Auth>> {
        let credentials = self.credentials().await;
        MutexGuard::try_map(credentials, CredentialSource::store).map_err(|_| {
            Error::validation("this client was built from credentials, not a token store")
        })
    }

    /// Creates a store-backed client from the process environment and the
    /// `~/.xurl` token store.
    ///
    /// Reads `CLIENT_ID`, `CLIENT_SECRET`, `REDIRECT_URI`, `AUTH_URL`,
    /// `TOKEN_URL`, `API_BASE_URL`, `INFO_URL`, and `XURL_BEARER_TOKEN`
    /// through [`Config::new()`], then loads the store the `xr` CLI writes.
    /// The store's default app supplies the client id and secret the
    /// environment does not, so a sign-in done with `xr auth oauth2` is
    /// reusable with nothing exported. A client with no credential anywhere
    /// still builds; its first call fails with the auth-required error and
    /// its [`NextAction`](crate::error::NextAction).
    ///
    /// [`Client::builder()`] is the path for a credential held in code;
    /// [`Client::new()`] takes an explicit [`Config`] and [`Auth`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] when the HTTP client cannot be built.
    pub fn from_env() -> Result<Self> {
        let cfg = Config::new();
        Self::new(&cfg, Auth::new(&cfg))
    }

    /// Builds the full URL from a target (public accessor for command layer).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidUrl`] when a `RawUrl` target's scheme
    /// is not `http` or `https`, [`Error::InvalidPathParam`] when a
    /// substituted value contains a URL-reserved character, or
    /// [`Error::Internal`] when a path template references a `{name}`
    /// segment missing from `path_params`.
    pub fn build_url_public(&self, target: &RequestTarget) -> Result<String> {
        self.build_url(target)
    }

    /// Builds the full URL from a target.
    fn build_url(&self, target: &RequestTarget) -> Result<String> {
        build_url_for_target(&self.inner.base_url, target)
    }
}
