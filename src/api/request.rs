/// HTTP request building and execution for the X API.
///
/// Mirrors the Go `ApiClient` — builds requests with auth headers,
/// handles regular/streaming/multipart responses.
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::time::Duration;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use reqwest::blocking::{Client, multipart};

use crate::auth::Auth;
use crate::config::Config;
use crate::error::{Result, XurlError};
use crate::output::OutputConfig;

/// Percent-encoding set for path-parameter values and query parameters.
///
/// Matches RFC 3986 §2.3 "unreserved" characters (alphanumeric, `-_.~`)
/// — everything else is encoded. Mirrors `percent-encoding`'s
/// `NON_ALPHANUMERIC` set widened to keep the URL-safe punctuation.
const URL_VALUE_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'&')
    .add(b'+')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

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

    /// Sends a regular API request and returns the JSON response.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP method is invalid, the request fails,
    /// or the API returns an error status (>= 400).
    pub fn send_request(&mut self, options: &RequestOptions) -> Result<serde_json::Value> {
        let method = options.method.to_uppercase();
        let method = if method.is_empty() { "GET" } else { &method };
        // Auth-matrix validation lives inside `get_auth_header` (called
        // below) so each request performs one matrix lookup, not two. The
        // explicit-auth branch there rejects with
        // `XurlError::AuthMethodMismatch` before any header is produced;
        // `no_auth: true` short-circuits past the call site entirely.
        let url = self.build_url(&options.target)?;

        // Build the request
        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| XurlError::InvalidMethod(method.to_string()))?;

        let mut builder = self.client.request(req_method.clone(), &url);

        // Add body for POST/PUT/PATCH
        if !options.data.is_empty() && (method == "POST" || method == "PUT" || method == "PATCH") {
            // Detect content type
            if serde_json::from_str::<serde_json::Value>(&options.data).is_ok() {
                builder = builder
                    .header("Content-Type", "application/json")
                    .body(options.data.clone());
            } else {
                builder = builder
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(options.data.clone());
            }
        }

        // Add custom headers
        for header in &options.headers {
            if let Some((key, value)) = header.split_once(':') {
                builder = builder.header(key.trim(), value.trim());
            }
        }

        // Add auth header (skip if no_auth is set, or if the caller already
        // supplied an `Authorization` header in `options.headers`). When auth
        // resolution fails (e.g., TokenNotFound for the resolved app),
        // propagate the error so the user sees the real problem instead of
        // letting the request go out unauthenticated and surfacing as a
        // confusing 401 from upstream. The older "silently skip on Err" form
        // let auth bugs masquerade as upstream auth rejections.
        let user_supplied_auth = headers_have_authorization(&options.headers);
        if !options.no_auth && !user_supplied_auth {
            let auth_header = self.get_auth_header(options)?;
            builder = builder.header("Authorization", auth_header);
        }
        if user_supplied_auth && options.verbose {
            let mut err = std::io::stderr().lock();
            self.out.verbose(
                &mut err,
                "info: user-supplied Authorization detected; skipping xurl auth append for this request",
            );
        }

        // Add common headers
        builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));

        if options.trace {
            builder = builder.header("X-B3-Flags", "1");
        }

        if options.verbose {
            let mut err = std::io::stderr().lock();
            if self.out.use_color {
                self.out
                    .verbose(&mut err, &format!("\x1b[1;34m> {method}\x1b[0m {url}"));
            } else {
                self.out.verbose(&mut err, &format!("> {method} {url}"));
            }
        }

        let resp = builder.send()?;

        if options.verbose {
            let mut err = std::io::stderr().lock();
            log_response_headers(&self.out, &mut err, resp.status(), resp.headers());
        }

        let status = resp.status();
        let body = resp.text().unwrap_or_default();

        let json: serde_json::Value = if body.is_empty() {
            serde_json::json!({})
        } else if let Ok(v) = serde_json::from_str(&body) {
            v
        } else {
            if status.as_u16() >= 400 {
                return Err(XurlError::api(
                    status.as_u16(),
                    format!("HTTP error: {status}"),
                ));
            }
            serde_json::json!({})
        };

        if status.as_u16() >= 400 {
            return Err(XurlError::api(status.as_u16(), json.to_string()));
        }

        Ok(json)
    }

    /// Sends a multipart request (used for media upload chunks).
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP method is invalid, file I/O fails,
    /// the request fails, or the API returns an error status (>= 400).
    pub fn send_multipart_request(
        &mut self,
        options: &MultipartOptions,
    ) -> Result<serde_json::Value> {
        let method = options.request.method.to_uppercase();
        let method = if method.is_empty() { "POST" } else { &method };
        // Auth-matrix validation lives inside `get_auth_header` (called
        // below). `no_auth: true` short-circuits past the call site.
        let url = self.build_url(&options.request.target)?;

        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| XurlError::InvalidMethod(method.to_string()))?;

        let mut form = multipart::Form::new();

        // Add file from path or data
        if !options.file_field.is_empty() && !options.file_path.is_empty() {
            let part = multipart::Part::file(&options.file_path)
                .map_err(|e| XurlError::Io(format!("error opening file: {e}")))?;
            form = form.part(options.file_field.clone(), part);
        } else if !options.file_field.is_empty() && !options.file_data.is_empty() {
            let part = multipart::Part::bytes(options.file_data.clone())
                .file_name(options.file_name.clone());
            form = form.part(options.file_field.clone(), part);
        }

        // Add form fields
        for (key, value) in &options.form_fields {
            form = form.text(key.clone(), value.clone());
        }

        let mut builder = self.client.request(req_method, &url).multipart(form);

        // Add custom headers
        for header in &options.request.headers {
            if let Some((key, value)) = header.split_once(':') {
                builder = builder.header(key.trim(), value.trim());
            }
        }

        // Add auth header (skip if no_auth is set, or if the caller already
        // supplied an `Authorization` header). Propagate auth errors rather
        // than silently sending the request unauthenticated; see the matching
        // propagation site in `send_request` for the rationale.
        let user_supplied_auth = headers_have_authorization(&options.request.headers);
        if !options.request.no_auth && !user_supplied_auth {
            let auth_header = self.get_auth_header(&options.request)?;
            builder = builder.header("Authorization", auth_header);
        }
        if user_supplied_auth && options.request.verbose {
            let mut err = std::io::stderr().lock();
            self.out.verbose(
                &mut err,
                "info: user-supplied Authorization detected; skipping xurl auth append for this request",
            );
        }

        builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));

        if options.request.trace {
            builder = builder.header("X-B3-Flags", "1");
        }

        if options.request.verbose {
            let mut err = std::io::stderr().lock();
            if self.out.use_color {
                self.out
                    .verbose(&mut err, &format!("\x1b[1;34m> {method}\x1b[0m {url}"));
            } else {
                self.out.verbose(&mut err, &format!("> {method} {url}"));
            }
        }

        let resp = builder.send()?;
        let status = resp.status();
        let body = resp.text().unwrap_or_default();

        let json: serde_json::Value = if body.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&body).unwrap_or(serde_json::json!({}))
        };

        if status.as_u16() >= 400 {
            return Err(XurlError::api(status.as_u16(), json.to_string()));
        }

        Ok(json)
    }

    /// Sends a streaming request — reads lines until EOF.
    ///
    /// All output flows through this client's configured `OutputConfig`
    /// (set via [`ApiClient::set_output`]); the CLI binary calls the
    /// `stream_request_with_output` helper in `cli::commands` which threads
    /// the runner's `OutputConfig` and writers in directly. Library callers
    /// pass their own `stdout`/`stderr` here so a streaming session can be
    /// captured in tests or redirected to a custom sink.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP method is invalid, the request fails,
    /// the API returns an error status (>= 400), or a read error occurs.
    #[allow(dead_code)] // Public library API — used by consumers and integration tests
    pub fn stream_request(
        &mut self,
        options: &RequestOptions,
        stdout: &mut dyn std::io::Write,
        stderr: &mut dyn std::io::Write,
    ) -> Result<()> {
        let method = options.method.to_uppercase();
        let method = if method.is_empty() { "GET" } else { &method };
        // Auth-matrix validation lives inside `get_auth_header` (called
        // below). Streaming honours the same fail-fast rule: an explicit
        // `--auth X` against an endpoint that doesn't accept `X` rejects
        // via `get_auth_header` before `builder.send()` opens any socket.
        let url = self.build_url(&options.target)?;

        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| XurlError::InvalidMethod(method.to_string()))?;

        let mut builder = Client::builder()
            .timeout(None)
            .build()
            .unwrap_or_else(|_| Client::new())
            .request(req_method, &url);

        if !options.data.is_empty() {
            if serde_json::from_str::<serde_json::Value>(&options.data).is_ok() {
                builder = builder
                    .header("Content-Type", "application/json")
                    .body(options.data.clone());
            } else {
                builder = builder
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(options.data.clone());
            }
        }

        for header in &options.headers {
            if let Some((key, value)) = header.split_once(':') {
                builder = builder.header(key.trim(), value.trim());
            }
        }

        let user_supplied_auth = headers_have_authorization(&options.headers);
        if !options.no_auth && !user_supplied_auth {
            let auth_header = self.get_auth_header(options)?;
            builder = builder.header("Authorization", auth_header);
        }
        if user_supplied_auth && options.verbose {
            self.out.verbose(
                stderr,
                "info: user-supplied Authorization detected; skipping xurl auth append for this request",
            );
        }

        builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));

        if options.trace {
            builder = builder.header("X-B3-Flags", "1");
        }

        if options.verbose {
            if self.out.use_color {
                self.out
                    .verbose(stderr, &format!("\x1b[1;34m> {method}\x1b[0m {url}"));
            } else {
                self.out.verbose(stderr, &format!("> {method} {url}"));
            }
        }

        self.out
            .status(stderr, &format!("Connecting to streaming endpoint: {url}"));

        let resp = builder.send()?;

        if options.verbose {
            log_response_headers(&self.out, stderr, resp.status(), resp.headers());
        }

        let resp_status = resp.status();
        if resp_status.as_u16() >= 400 {
            let body = resp.text().unwrap_or_default();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                return Err(XurlError::api(resp_status.as_u16(), json.to_string()));
            }
            return Err(XurlError::api(resp_status.as_u16(), body));
        }

        self.out
            .status(stderr, "--- Streaming response started ---");
        self.out.status(stderr, "--- Press Ctrl+C to stop ---");

        let reader = BufReader::with_capacity(1024 * 1024, resp);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if line.is_empty() {
                        continue;
                    }
                    self.out.print_stream_line(stdout, &line);
                }
                Err(e) => {
                    return Err(XurlError::Io(e.to_string()));
                }
            }
        }

        self.out.status(stderr, "--- End of stream ---");
        Ok(())
    }

    /// Gets the authorization header for a request (public accessor for command layer).
    ///
    /// # Errors
    ///
    /// Returns an error if no valid auth method is found, or — when the
    /// auto-detect path resolves an empty intersection between stored
    /// credentials and the endpoint's accepted schemes — an
    /// [`XurlError::AuthMethodMismatch`] in the empty-intersection shape.
    pub fn get_auth_header_public(&mut self, options: &RequestOptions) -> Result<String> {
        self.get_auth_header(options)
    }

    /// Gets the authorization header for a request.
    ///
    /// When `options.auth_type` is non-empty, dispatches directly to that
    /// scheme. When empty, runs the endpoint-aware auto-detect: intersects
    /// the active app's stored credentials with the endpoint's accepted
    /// auth schemes per the matrix, picks the first match in OAuth2 →
    /// OAuth1 → Bearer preference order, and falls back to that fixed
    /// order for [`RequestTarget::RawUrl`] and matrix-miss
    /// [`RequestTarget::Template`] targets (the permissive surface for
    /// unknown endpoints).
    fn get_auth_header(&mut self, options: &RequestOptions) -> Result<String> {
        let auth_type = &options.auth_type;
        let method_raw = options.method.to_uppercase();
        let method = if method_raw.is_empty() {
            "GET"
        } else {
            method_raw.as_str()
        };

        // One matrix lookup for the entire decision. Both the explicit-auth
        // validation below and the auto-detect intersection further down
        // consume this, eliminating the prior duplicate `supported_auth`
        // call that fired from `auth_matrix::validate` plus a second pass
        // here.
        let endpoint_schemes = match &options.target {
            RequestTarget::Template { path, .. } => {
                crate::api::auth_matrix::supported_auth(method, path).map(|s| (path.clone(), s))
            }
            RequestTarget::RawUrl(_) => None,
        };

        if !auth_type.is_empty() {
            // Validate the explicit-auth request against the matrix entry
            // when present. Matrix-miss is permissive per the unknown-
            // endpoint rule. Empty wire list (currently unreachable) would
            // collapse to permissive too — the matrix only emits entries
            // for endpoints that declare a `security:` list.
            if let Some((path, schemes)) = &endpoint_schemes {
                let supported_static = crate::api::auth_matrix::schemes_to_wire_list(schemes);
                let requested_norm = auth_type.to_ascii_lowercase();
                if !supported_static.contains(&requested_norm.as_str()) {
                    let supported: Vec<String> =
                        supported_static.iter().map(|s| (*s).to_string()).collect();
                    let rendered_url = render_template_template(&options.target).ok();
                    let raw_app = self.auth.app_name();
                    let app_name = if raw_app.is_empty() {
                        self.auth.token_store.default_app.clone()
                    } else {
                        raw_app.to_string()
                    };
                    return Err(XurlError::AuthMethodMismatch {
                        endpoint: path.clone(),
                        rendered_url,
                        method: method.to_string(),
                        requested: Some(requested_norm),
                        supported,
                        available_in_app: None,
                        app: Some(app_name),
                        other_apps_with_creds: None,
                    });
                }
            }
            let url = self.build_url(&options.target)?;
            return match auth_type.to_lowercase().as_str() {
                "oauth1" => self.auth.get_oauth1_header(method, &url, None),
                "oauth2" => self.auth.get_oauth2_header(&options.username),
                "app" => self.auth.get_bearer_token_header(),
                _ => Err(XurlError::auth(format!("invalid auth type: {auth_type}"))),
            };
        }

        // Auto-detect: scope every check to the active app (set by
        // `--app NAME` via `Auth::with_app_name`) so a `--app NAME`
        // invocation without `--auth` picks NAME's tokens, not the
        // default app's. Per-app probes route through `_for_app(app_name)`
        // accessors on the token store so the active-app contract holds
        // even when the active app differs from the default.
        let raw_app = self.auth.app_name();
        // Empty active app name is the "use default app" convention. Resolve
        // it to the store's actual default_app name so the envelope's `app`
        // field carries something the user can act on (e.g. "default" rather
        // than "").
        let app_name = if raw_app.is_empty() {
            self.auth.token_store.default_app.clone()
        } else {
            raw_app.to_string()
        };
        let available_in_app = self.available_auth_in_app(&app_name);

        // Auto-detect filter: walk the preference order and keep every
        // scheme the active app has, optionally intersected with the
        // endpoint's accepted set. The closure expression collapses the
        // two earlier branches that duplicated the availability filter.
        let endpoint_supported_static: Option<Vec<&'static str>> = endpoint_schemes
            .as_ref()
            .map(|(_, schemes)| crate::api::auth_matrix::schemes_to_wire_list(schemes));
        let candidate_order: Vec<crate::api::auth_matrix::WireScheme> =
            crate::api::auth_matrix::WireScheme::ALL_BY_PREFERENCE
                .into_iter()
                .filter(|m| {
                    let wire = m.as_wire();
                    let in_app = available_in_app.contains(&wire);
                    let in_endpoint = endpoint_supported_static
                        .as_ref()
                        .is_none_or(|sup| sup.contains(&wire));
                    in_app && in_endpoint
                })
                .collect();

        if candidate_order.is_empty() {
            // Empty intersection (or empty active app entirely). The
            // matrix-hit branches construct the typed envelope; the
            // matrix-miss branch falls back to the generic auth error
            // because no endpoint context is in scope.
            if let Some((path, _)) = &endpoint_schemes {
                let rendered_url = render_template_template(&options.target).ok();
                let endpoint_supported = endpoint_supported_static
                    .as_ref()
                    .map(|sup| sup.iter().map(|s| (*s).to_string()).collect::<Vec<_>>())
                    .unwrap_or_default();
                if available_in_app.is_empty() {
                    // Active app holds nothing. Check whether OTHER apps
                    // in the store hold credentials. If so, surface a
                    // wrong-app envelope (exit 2) instead of generic
                    // auth-required (exit 77) — the user logged in, just
                    // not against the app they invoked.
                    let other_apps = self.other_apps_with_credentials(&app_name);
                    if other_apps.is_empty() {
                        return Err(XurlError::auth(
                            "NoAuthMethod: no authentication method available",
                        ));
                    }
                    return Err(XurlError::AuthMethodMismatch {
                        endpoint: path.clone(),
                        rendered_url,
                        method: method.to_string(),
                        requested: None,
                        supported: endpoint_supported,
                        available_in_app: Some(Vec::new()),
                        app: Some(app_name.clone()),
                        other_apps_with_creds: Some(other_apps),
                    });
                }
                return Err(XurlError::AuthMethodMismatch {
                    endpoint: path.clone(),
                    rendered_url,
                    method: method.to_string(),
                    requested: None,
                    supported: endpoint_supported,
                    available_in_app: Some(
                        available_in_app.iter().map(|s| (*s).to_string()).collect(),
                    ),
                    app: Some(app_name.clone()),
                    other_apps_with_creds: None,
                });
            }
            return Err(XurlError::auth(
                "NoAuthMethod: no authentication method available",
            ));
        }

        // Pick the first candidate in OAuth2 → OAuth1 → Bearer preference
        // order. Dispatching on the typed [`WireScheme`] makes adding a
        // new variant a compile error — the previous `&str`-keyed match
        // could panic at runtime if the candidate list ever grew without
        // a matching arm.
        use crate::api::auth_matrix::WireScheme;
        match candidate_order[0] {
            WireScheme::OAuth2 => self.auth.get_oauth2_header(&options.username),
            WireScheme::OAuth1 => {
                let url = self.build_url(&options.target)?;
                self.auth.get_oauth1_header(method, &url, None)
            }
            WireScheme::App => self.auth.get_bearer_token_header(),
        }
    }

    /// Probes the active app for which auth schemes have stored credentials.
    ///
    /// Returned vector lists the wire strings (`"oauth2"`, `"oauth1"`,
    /// `"app"`) in OAuth2 → OAuth1 → Bearer order. Used by the auto-detect
    /// intersection in [`Self::get_auth_header`] and by the empty-intersection
    /// error envelope to populate `available_in_app`.
    fn available_auth_in_app(&self, app_name: &str) -> Vec<&'static str> {
        let mut out: Vec<&'static str> = Vec::with_capacity(3);
        if self
            .auth
            .token_store
            .get_first_oauth2_token_for_app(app_name)
            .is_some()
        {
            out.push("oauth2");
        }
        if self
            .auth
            .token_store
            .get_oauth1_tokens_for_app(app_name)
            .is_some()
        {
            out.push("oauth1");
        }
        if self
            .auth
            .token_store
            .get_bearer_token_for_app(app_name)
            .is_some()
        {
            out.push("app");
        }
        out
    }

    /// Names of apps OTHER than `active` that hold at least one stored
    /// credential.
    ///
    /// Used by `get_auth_header` to surface a "wrong-app" envelope when the
    /// active app is empty but the user has credentials elsewhere. Returns
    /// the apps in stable BTreeMap iteration order so the resulting message
    /// is deterministic.
    fn other_apps_with_credentials(&self, active: &str) -> Vec<String> {
        self.auth
            .token_store
            .apps_with_credentials()
            .into_iter()
            .filter(|name| name != active)
            .collect()
    }
}

/// Renders the path portion of a [`RequestTarget::Template`] without the
/// `base_url` prefix or query string.
///
/// Used by error-envelope construction so user-facing messages show the
/// substituted path (`/2/users/12345/likes`) instead of the spec template
/// (`/2/users/{id}/likes`). Returns `Err` when the target is `RawUrl` (no
/// substitution applies) or when substitution itself fails — both surface
/// as `None` at the call site so the envelope falls back to `endpoint`.
fn render_template_template(target: &RequestTarget) -> Result<String> {
    match target {
        RequestTarget::Template {
            path, path_params, ..
        } => render_template_path(path, path_params),
        RequestTarget::RawUrl(_) => Err(XurlError::Internal(
            "RawUrl target has no template to render".to_string(),
        )),
    }
}

/// Renders a [`RequestTarget`] against `base_url` into a full URL string.
///
/// Free function so unit tests can exercise the rendering without
/// instantiating a full [`ApiClient`].
fn build_url_for_target(base_url: &str, target: &RequestTarget) -> Result<String> {
    match target {
        RequestTarget::Template {
            path,
            path_params,
            query,
        } => {
            let rendered_path = render_template_path(path, path_params)?;

            let mut url = base_url.to_string();
            if !url.ends_with('/') {
                url.push('/');
            }
            if let Some(stripped) = rendered_path.strip_prefix('/') {
                url.push_str(stripped);
            } else {
                url.push_str(&rendered_path);
            }

            if !query.is_empty() {
                url.push('?');
                for (i, (key, value)) in query.iter().enumerate() {
                    if i > 0 {
                        url.push('&');
                    }
                    write_encoded(&mut url, key);
                    url.push('=');
                    write_encoded(&mut url, value);
                }
            }
            Ok(url)
        }
        RequestTarget::RawUrl(raw) => {
            validate_raw_url_scheme(raw)?;
            Ok(raw.clone())
        }
    }
}

/// Substitutes `{name}` segments in a path template using `path_params`.
///
/// Each substituted value is rejected up-front if it contains `/`, `?`,
/// `#`, or `%` (would break URL semantics on encode/decode), then
/// percent-encoded against [`URL_VALUE_ENCODE_SET`]. A `{name}` whose
/// `name` is missing from `path_params` is a programmer error and
/// surfaces as [`XurlError::Internal`].
pub(crate) fn render_template_path(
    template: &str,
    path_params: &HashMap<String, String>,
) -> Result<String> {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            // Find matching closing brace
            if let Some(end) = template[i + 1..].find('}') {
                let name = &template[i + 1..i + 1 + end];
                let value = path_params.get(name).ok_or_else(|| {
                    XurlError::Internal(format!(
                        "path template {template:?} references {{{name}}} but path_params has no such key"
                    ))
                })?;
                if value.contains('/')
                    || value.contains('?')
                    || value.contains('#')
                    || value.contains('%')
                {
                    return Err(XurlError::InvalidPathParam {
                        name: name.to_string(),
                        value: value.clone(),
                    });
                }
                write_encoded(&mut out, value);
                i += 1 + end + 1;
                continue;
            }
        }
        // Push raw byte (template literal char). Safe because template
        // segments outside braces are ASCII per spec.
        out.push(char::from(bytes[i]));
        i += 1;
    }
    Ok(out)
}

/// Validates that a raw URL uses the `http` or `https` scheme.
///
/// Refuses `file://`, `ftp://`, `data:` etc. before any handle to the
/// filesystem or external service is created. Case-insensitive match
/// on the scheme prefix.
fn validate_raw_url_scheme(url: &str) -> Result<()> {
    let lower = url.trim_start().to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        return Ok(());
    }
    Err(XurlError::InvalidUrl(format!(
        "URL must start with http:// or https://: {url}"
    )))
}

/// Returns `true` when any `"Name: Value"` entry has `"Authorization"` as its
/// key, compared case-insensitively (ASCII).
///
/// Used by the three send paths to suppress xurl's own `Authorization`
/// append when the caller already supplied one — `reqwest::RequestBuilder::header`
/// is `HeaderMap::append`, so unconditional appending would send two
/// `Authorization` headers and most servers reject or pick arbitrarily.
fn headers_have_authorization(headers: &[String]) -> bool {
    headers
        .iter()
        .filter_map(|h| h.split_once(':'))
        .any(|(key, _)| key.trim().eq_ignore_ascii_case("authorization"))
}

/// Appends a percent-encoded value to `out` using [`URL_VALUE_ENCODE_SET`].
fn write_encoded(out: &mut String, value: &str) {
    for chunk in utf8_percent_encode(value, URL_VALUE_ENCODE_SET) {
        out.push_str(chunk);
    }
}

/// Emits the verbose response-header dump (`< STATUS`, `< key: value`, blank
/// line) through the supplied `OutputConfig`. Lives at module scope so
/// `send_request`, `send_multipart_request`, and `stream_request` share one
/// implementation.
fn log_response_headers(
    out: &OutputConfig,
    err: &mut dyn std::io::Write,
    status: reqwest::StatusCode,
    headers: &reqwest::header::HeaderMap,
) {
    if out.use_color {
        out.verbose(err, &format!("\x1b[1;31m< {status}\x1b[0m"));
        for (key, value) in headers {
            out.verbose(
                err,
                &format!("\x1b[1;32m< {key}\x1b[0m: {}", value.to_str().unwrap_or("")),
            );
        }
    } else {
        out.verbose(err, &format!("< {status}"));
        for (key, value) in headers {
            out.verbose(err, &format!("< {key}: {}", value.to_str().unwrap_or("")));
        }
    }
    out.verbose(err, "");
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

    // ── build_url tests ────────────────────────────────────────────────

    const TEST_BASE_URL: &str = "https://api.x.com";

    fn tmpl(path: &str) -> RequestTarget {
        RequestTarget::Template {
            path: path.to_string(),
            path_params: HashMap::new(),
            query: Vec::new(),
        }
    }

    #[test]
    fn build_url_template_empty_params_and_query() {
        let url = build_url_for_target(TEST_BASE_URL, &tmpl("/2/users/me")).unwrap();
        assert_eq!(url, "https://api.x.com/2/users/me");
    }

    #[test]
    fn build_url_template_substitutes_path_param() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "12345".to_string());
        let target = RequestTarget::Template {
            path: "/2/users/{id}/likes".to_string(),
            path_params: params,
            query: Vec::new(),
        };
        let url = build_url_for_target(TEST_BASE_URL, &target).unwrap();
        assert_eq!(url, "https://api.x.com/2/users/12345/likes");
    }

    #[test]
    fn build_url_template_query_preserves_insertion_order() {
        let target = RequestTarget::Template {
            path: "/2/tweets/search/recent".to_string(),
            path_params: HashMap::new(),
            query: vec![
                ("query".to_string(), "rustlang".to_string()),
                ("max_results".to_string(), "10".to_string()),
            ],
        };
        let url = build_url_for_target(TEST_BASE_URL, &target).unwrap();
        assert_eq!(
            url,
            "https://api.x.com/2/tweets/search/recent?query=rustlang&max_results=10"
        );
    }

    #[test]
    fn build_url_template_percent_encodes_value_with_spaces() {
        let target = RequestTarget::Template {
            path: "/2/tweets/search/recent".to_string(),
            path_params: HashMap::new(),
            query: vec![("query".to_string(), "hello world".to_string())],
        };
        let url = build_url_for_target(TEST_BASE_URL, &target).unwrap();
        assert_eq!(
            url,
            "https://api.x.com/2/tweets/search/recent?query=hello%20world"
        );
    }

    #[test]
    fn build_url_template_rejects_path_param_with_slash() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "abc/etc/passwd".to_string());
        let target = RequestTarget::Template {
            path: "/2/users/{id}/likes".to_string(),
            path_params: params,
            query: Vec::new(),
        };
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        match err {
            XurlError::InvalidPathParam { name, value } => {
                assert_eq!(name, "id");
                assert_eq!(value, "abc/etc/passwd");
            }
            other => panic!("expected InvalidPathParam, got {other:?}"),
        }
    }

    #[test]
    fn build_url_template_rejects_path_param_with_hash() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "abc#fragment".to_string());
        let target = RequestTarget::Template {
            path: "/2/users/{id}/likes".to_string(),
            path_params: params,
            query: Vec::new(),
        };
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        match err {
            XurlError::InvalidPathParam { name, value } => {
                assert_eq!(name, "id");
                assert_eq!(value, "abc#fragment");
            }
            other => panic!("expected InvalidPathParam, got {other:?}"),
        }
    }

    #[test]
    fn build_url_template_rejects_path_param_with_percent() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "already%20encoded".to_string());
        let target = RequestTarget::Template {
            path: "/2/users/{id}/likes".to_string(),
            path_params: params,
            query: Vec::new(),
        };
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        match err {
            XurlError::InvalidPathParam { name, value } => {
                assert_eq!(name, "id");
                assert_eq!(value, "already%20encoded");
            }
            other => panic!("expected InvalidPathParam, got {other:?}"),
        }
    }

    #[test]
    fn build_url_template_rejects_path_param_with_question_mark() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "abc?injected".to_string());
        let target = RequestTarget::Template {
            path: "/2/users/{id}".to_string(),
            path_params: params,
            query: Vec::new(),
        };
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        assert!(matches!(err, XurlError::InvalidPathParam { .. }));
    }

    #[test]
    fn build_url_template_missing_path_param_is_internal_error() {
        let target = RequestTarget::Template {
            path: "/2/users/{id}/likes".to_string(),
            path_params: HashMap::new(),
            query: Vec::new(),
        };
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        assert!(matches!(err, XurlError::Internal(_)), "got {err:?}");
    }

    #[test]
    fn build_url_raw_url_https_returns_clone() {
        let target = RequestTarget::RawUrl("https://api.x.com/2/raw".to_string());
        let url = build_url_for_target(TEST_BASE_URL, &target).unwrap();
        assert_eq!(url, "https://api.x.com/2/raw");
    }

    #[test]
    fn build_url_raw_url_http_returns_clone() {
        let target = RequestTarget::RawUrl("http://localhost:8080/dev".to_string());
        let url = build_url_for_target(TEST_BASE_URL, &target).unwrap();
        assert_eq!(url, "http://localhost:8080/dev");
    }

    #[test]
    fn build_url_raw_url_file_scheme_rejected() {
        let target = RequestTarget::RawUrl("file:///etc/passwd".to_string());
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        assert!(matches!(err, XurlError::InvalidUrl(_)), "got {err:?}");
    }

    #[test]
    fn build_url_raw_url_ftp_scheme_rejected() {
        let target = RequestTarget::RawUrl("ftp://attacker.com/payload".to_string());
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        assert!(matches!(err, XurlError::InvalidUrl(_)), "got {err:?}");
    }

    // ── headers_have_authorization tests ───────────────────────────────

    #[test]
    fn headers_have_authorization_detects_canonical_case() {
        let headers = vec!["Authorization: Bearer foo".to_string()];
        assert!(headers_have_authorization(&headers));
    }

    #[test]
    fn headers_have_authorization_is_case_insensitive() {
        for raw in [
            "authorization: Bearer foo",
            "AUTHORIZATION: Bearer foo",
            "aUtHoRiZaTiOn: Bearer foo",
        ] {
            assert!(
                headers_have_authorization(&[raw.to_string()]),
                "did not detect: {raw}"
            );
        }
    }

    #[test]
    fn headers_have_authorization_ignores_surrounding_whitespace_on_key() {
        let headers = vec!["  Authorization  : Bearer foo".to_string()];
        assert!(headers_have_authorization(&headers));
    }

    #[test]
    fn headers_have_authorization_false_for_empty_list() {
        let headers: Vec<String> = Vec::new();
        assert!(!headers_have_authorization(&headers));
    }

    #[test]
    fn headers_have_authorization_false_for_other_headers() {
        let headers = vec![
            "Cookie: session=abc".to_string(),
            "User-Agent: custom/1.0".to_string(),
            "X-Authorization-Hint: ignored".to_string(),
        ];
        assert!(!headers_have_authorization(&headers));
    }

    #[test]
    fn headers_have_authorization_false_for_unparseable_entry() {
        let headers = vec!["malformed-no-colon".to_string()];
        assert!(!headers_have_authorization(&headers));
    }

    #[test]
    fn headers_have_authorization_detects_when_mixed_with_others() {
        let headers = vec![
            "Content-Type: application/json".to_string(),
            "Authorization: Bearer foo".to_string(),
            "X-B3-Flags: 1".to_string(),
        ];
        assert!(headers_have_authorization(&headers));
    }
}
