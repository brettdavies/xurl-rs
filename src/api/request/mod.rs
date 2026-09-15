//! HTTP request building and execution for the X API.
//!
//! Mirrors the Go `ApiClient` — builds requests with auth headers,
//! handles regular/streaming/multipart responses.

use std::collections::HashMap;
use std::time::Duration;

use reqwest::blocking::Client;

use crate::auth::Auth;
use crate::config::Config;
use crate::error::{Result, XurlError};
use crate::output::OutputConfig;

mod auth_header;
mod transport;
mod url;

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
/// Threaded into [`ApiClient::send_request`], [`ApiClient::send_multipart_request`],
/// and [`ApiClient::stream_request`]; carries everything those calls need
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
    /// Emit verbose request / response diagnostics through the client's
    /// [`OutputConfig`].
    pub verbose: bool,
    /// Emit the `X-B3-Flags: 1` header to flag the request for upstream tracing.
    pub trace: bool,
    /// Cursor / `pagination_token` query parameter for list endpoints.
    ///
    /// Threaded in from the global `--cursor` / `--after` flag (or
    /// `XURL_CURSOR` / `XURL_AFTER` env vars). List shortcuts append it to
    /// the URL when non-empty; non-paginated endpoints ignore it.
    pub pagination_token: String,
}

/// Default request timeout in seconds when none is supplied.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Consumer-facing options for shortcut methods.
///
/// Exposes only the fields relevant to crate consumers, hiding internal
/// request construction details like `method`, `endpoint`, `headers`, and `data`.
#[derive(Debug, Clone)]
pub struct CallOptions {
    /// Explicit auth scheme — `"oauth1"`, `"oauth2"`, `"app"`, or empty for
    /// auto-detect.
    pub auth_type: String,
    /// OAuth2 username for the active app. Empty selects the active app's
    /// first stored OAuth2 token.
    pub username: String,
    /// Skip auth-header attachment entirely.
    pub no_auth: bool,
    /// Emit verbose request / response diagnostics.
    pub verbose: bool,
    /// Emit the `X-B3-Flags: 1` header for upstream tracing.
    pub trace: bool,
    /// Per-call HTTP timeout in seconds. Mirrors the `--timeout` flag /
    /// `XURL_TIMEOUT` env var. Used by the streaming and per-call refresh
    /// paths; non-streaming requests inherit the timeout that was passed to
    /// [`ApiClient::new`].
    pub timeout_secs: u64,
    /// Cursor / `pagination_token` query parameter for list endpoints.
    ///
    /// Threaded in from the global `--cursor` flag. List shortcuts append
    /// it to the URL when non-empty; non-paginated endpoints ignore it.
    pub pagination_token: String,
}

impl Default for CallOptions {
    fn default() -> Self {
        Self {
            auth_type: String::new(),
            username: String::new(),
            no_auth: false,
            verbose: false,
            trace: false,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            pagination_token: String::new(),
        }
    }
}

impl CallOptions {
    /// Converts to a [`RequestOptions`] with consumer fields populated
    /// and request-specific fields (method, endpoint, data, headers) at defaults.
    #[must_use]
    pub(crate) fn to_request_options(&self) -> RequestOptions {
        RequestOptions {
            auth_type: self.auth_type.clone(),
            username: self.username.clone(),
            no_auth: self.no_auth,
            verbose: self.verbose,
            trace: self.trace,
            pagination_token: self.pagination_token.clone(),
            ..Default::default()
        }
    }
}

/// Options specific to multipart requests.
///
/// Used by the media upload path to ship a single file plus form fields
/// in one `multipart/form-data` POST.
#[derive(Debug, Clone)]
pub struct MultipartOptions {
    /// Base request configuration (target, headers, auth, verbose, trace).
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

/// Handles API requests with authentication.
///
/// # Example
///
/// ```rust,no_run
/// use xurl::api::{ApiClient, RequestOptions, RequestTarget};
/// use xurl::auth::Auth;
/// use xurl::config::Config;
/// use xurl::error::XurlError;
/// use std::collections::HashMap;
///
/// let cfg = Config::new();
/// let auth = Auth::new(&cfg);
/// let mut client = ApiClient::new(&cfg, auth);
///
/// let mut opts = RequestOptions::default();
/// opts.method = "GET".to_string();
/// opts.target = RequestTarget::Template {
///     path: "/2/users/me".to_string(),
///     path_params: HashMap::new(),
///     query: Vec::new(),
/// };
///
/// match client.send_request(&opts) {
///     Ok(json) => println!("{json}"),
///     Err(XurlError::Api { status, body }) => eprintln!("API {status}: {body}"),
///     Err(e) => eprintln!("error: {e}"),
/// }
/// ```
pub struct ApiClient {
    base_url: String,
    client: Client,
    auth: Auth,
    timeout_secs: u64,
    /// Output configuration used to route verbose request/response logs
    /// through the single owner in `src/output.rs`. Library callers that
    /// haven't supplied one get the [`OutputConfig::default`] (text, no
    /// verbose) — `verbose=false` suppresses diagnostics.
    out: OutputConfig,
}

impl ApiClient {
    /// Creates a new `ApiClient` using the timeout configured on `config`.
    ///
    /// The CLI runner writes `--timeout` / `XURL_TIMEOUT` into
    /// [`Config::http_timeout_secs`]; library consumers that want a different
    /// timeout can use [`ApiClient::with_timeout`].
    pub fn new(config: &Config, auth: Auth) -> Self {
        Self::with_timeout(config, auth, config.http_timeout_secs)
    }

    /// Creates a new `ApiClient` with an explicit request timeout.
    ///
    /// The timeout bounds every non-streaming HTTP call dispatched by this
    /// client. Streaming requests intentionally retain `.timeout(None)` — the
    /// long-running shape is the point — and bound runtime via signal handlers
    /// in the streaming handler.
    pub fn with_timeout(config: &Config, auth: Auth, timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url: config.api_base_url.clone(),
            client,
            auth,
            timeout_secs,
            out: OutputConfig::default(),
        }
    }

    /// Returns the per-call timeout used by this client (seconds).
    #[must_use]
    pub fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }

    /// Installs an `OutputConfig` for verbose request/response diagnostics.
    ///
    /// The CLI runner calls this after constructing the client so the
    /// verbose logs route through the single output owner. Library callers
    /// that skip this get a default config (text, verbose off).
    pub fn set_output(&mut self, out: OutputConfig) {
        self.out = out;
    }

    /// Creates an `ApiClient` from environment variables.
    ///
    /// Reads `CLIENT_ID`, `CLIENT_SECRET`, and other env vars via [`Config::new()`],
    /// validates that `CLIENT_ID` is non-empty, and returns a ready-to-use client.
    ///
    /// For full control over configuration and auth, use [`ApiClient::new()`] instead.
    ///
    /// # Errors
    ///
    /// Returns `XurlError::Validation` if `CLIENT_ID` is not set or empty.
    #[allow(dead_code)] // Public library API — used by consumers
    pub fn from_env() -> Result<Self> {
        let cfg = Config::new();
        if cfg.client_id.is_empty() {
            return Err(XurlError::validation(
                "CLIENT_ID not set — set the environment variable or use ApiClient::new() for manual configuration",
            ));
        }
        let auth = Auth::new(&cfg);
        Ok(Self::new(&cfg, auth))
    }

    /// Builds the full URL from a target (public accessor for command layer).
    ///
    /// # Errors
    ///
    /// Returns [`XurlError::InvalidUrl`] when a `RawUrl` target's scheme
    /// is not `http` or `https`, [`XurlError::InvalidPathParam`] when a
    /// substituted value contains a URL-reserved character, or
    /// [`XurlError::Internal`] when a path template references a `{name}`
    /// segment missing from `path_params`.
    pub fn build_url_public(&self, target: &RequestTarget) -> Result<String> {
        self.build_url(target)
    }

    /// Returns the active app name carried by the underlying [`Auth`].
    ///
    /// Library-public so callers building requests outside `ApiClient` (e.g.
    /// the CLI streaming wrapper) can thread the active app into
    /// `auth_matrix::validate` for the user-facing message.
    #[must_use]
    pub fn auth_app_name(&self) -> &str {
        self.auth.app_name()
    }

    /// Builds the full URL from a target.
    fn build_url(&self, target: &RequestTarget) -> Result<String> {
        build_url_for_target(&self.base_url, target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_options_to_request_options_maps_all_fields() {
        let opts = CallOptions {
            auth_type: "oauth2".to_string(),
            username: "testuser".to_string(),
            no_auth: true,
            verbose: true,
            trace: true,
            timeout_secs: 45,
            pagination_token: "abc123".to_string(),
        };

        let req = opts.to_request_options();

        assert_eq!(req.auth_type, "oauth2");
        assert_eq!(req.username, "testuser");
        assert!(req.no_auth);
        assert!(req.verbose);
        assert!(req.trace);
        assert_eq!(req.pagination_token, "abc123");
        // Request-specific fields should be at defaults
        assert!(req.method.is_empty());
        match &req.target {
            RequestTarget::Template {
                path,
                path_params,
                query,
            } => {
                assert!(path.is_empty());
                assert!(path_params.is_empty());
                assert!(query.is_empty());
            }
            RequestTarget::RawUrl(_) => panic!("default target must be Template"),
        }
        assert!(req.data.is_empty());
        assert!(req.headers.is_empty());
    }

    #[test]
    fn call_options_default_has_safe_values() {
        let opts = CallOptions::default();
        let req = opts.to_request_options();

        assert!(!req.no_auth, "no_auth should default to false");
        assert!(!req.verbose);
        assert!(!req.trace);
        assert!(req.auth_type.is_empty());
        assert!(req.username.is_empty());
        assert!(
            opts.pagination_token.is_empty(),
            "pagination_token should default to empty so non-paginated endpoints stay clean"
        );
        assert_eq!(
            opts.timeout_secs, DEFAULT_TIMEOUT_SECS,
            "timeout_secs should default to {DEFAULT_TIMEOUT_SECS}"
        );
    }
}
