/// Typed error system matching xurl's error categories.
///
/// The Go source uses string-typed errors with a `Type` field. We replicate
/// that with thiserror variants so Rust callers get pattern matching while
/// the Display output stays identical to xurl.
use thiserror::Error;

/// Top-level error type for xurl-rs.
#[derive(Debug, Error)]
pub enum XurlError {
    /// HTTP transport / request construction error.
    #[error("HTTP Error: {0}")]
    Http(String),

    /// File / IO error.
    #[error("IO Error: {0}")]
    Io(String),

    /// Invalid HTTP method supplied.
    #[error("Invalid Method: Invalid HTTP method: {0}")]
    InvalidMethod(String),

    /// API returned an HTTP error response (status >= 400).
    #[error("{body}")]
    Api {
        /// HTTP status code from the API response.
        status: u16,
        /// Raw response body (typically JSON).
        body: String,
    },

    /// Non-HTTP validation or logic error (e.g., missing fields, errors-only 200 responses).
    #[error("{0}")]
    Validation(String),

    /// Raw URL supplied with an unsupported scheme. Only `http://` and
    /// `https://` are accepted; file/ftp/etc are rejected before any
    /// network or filesystem activity.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Path-parameter value contained a character that would break URL
    /// semantics (`/`, `?`, `#`, or `%`). Surfaces real IDs that contain
    /// stray separators rather than silently encoding them.
    #[error("Invalid path parameter {name:?}: value {value:?} contains a reserved character")]
    InvalidPathParam {
        /// Name of the offending `{param}` segment in the path template.
        name: String,
        /// Caller-supplied value that failed validation.
        value: String,
    },

    /// Internal invariant violated — typically a programmer error such as
    /// a path template referencing a `{name}` segment that the caller never
    /// supplied in `path_params`.
    #[error("Internal error: {0}")]
    Internal(String),

    /// JSON serialization / deserialization error.
    #[error("JSON Error: {0}")]
    Json(String),

    /// Authentication error with sub-type context.
    #[error("Auth Error: {0}")]
    Auth(String),

    /// Token store persistence / lookup error.
    #[error("Token Store Error: {0}")]
    TokenStore(String),

    /// Sentinel: a structured envelope was already emitted by the call site.
    ///
    /// The runner short-circuits its trailing `print_error` for this variant
    /// and propagates the carried exit code unchanged. Used by U7's
    /// `print_confirmation_required` so the canonical envelope
    /// `{"status":"error","reason":"confirmation-required",…}` is the only
    /// thing the agent sees on stderr (no duplicated `{"error":...,"kind":...}`
    /// from the generic `print_error` path).
    #[error("envelope-already-emitted")]
    EnvelopeAlreadyEmitted {
        /// Exit code the runner should surface for this error.
        exit_code: i32,
    },

    /// Auth method mismatch: the user supplied (or the auto-detect resolved)
    /// an auth method the endpoint's matrix entry doesn't accept.
    ///
    /// Two shapes share the variant:
    /// - **Explicit-mismatch** (U6): `requested = Some("app"|"oauth1"|"oauth2")`,
    ///   `available_in_app = None`. The user passed `--auth X` and `X` isn't
    ///   in the endpoint's supported set.
    /// - **Empty-intersection** (U7): `requested = None`,
    ///   `available_in_app = Some([...])`. Auto-detect resolved a non-empty
    ///   `available_in_app` against `supported` to an empty intersection — no
    ///   stored credential on the active app satisfies the endpoint.
    ///
    /// The `Display` impl renders the same body that fills the envelope's
    /// `message` field (R12).
    #[error("{}", auth_method_mismatch_message(.endpoint, .method, .requested.as_deref(), .supported, .available_in_app.as_deref()))]
    AuthMethodMismatch {
        /// Path template (e.g. `/2/media/upload`) — keyed verbatim against
        /// the spec, never the rendered URL.
        endpoint: String,
        /// HTTP method, already uppercased.
        method: String,
        /// What the user asked for. `Some("app"|"oauth1"|"oauth2")` in the
        /// explicit-mismatch shape; `None` in the empty-intersection shape.
        requested: Option<String>,
        /// Auth methods the endpoint accepts, as user-facing strings.
        supported: Vec<String>,
        /// Auth methods the active app actually has stored. `None` in the
        /// explicit-mismatch shape; `Some([...])` in the empty-intersection
        /// shape so the user can see what would and wouldn't have worked.
        available_in_app: Option<Vec<String>>,
    },
}

/// Builds the user-facing message that fills both the `Display` output and
/// the JSON envelope's `message` field for `AuthMethodMismatch`.
///
/// Two shapes per the variant's docstring; both end with an actionable
/// `Use --auth X` / `xr auth Y --app NAME` instruction.
fn auth_method_mismatch_message(
    endpoint: &str,
    method: &str,
    requested: Option<&str>,
    supported: &[String],
    available_in_app: Option<&[String]>,
) -> String {
    match (requested, available_in_app) {
        (Some(req), _) => {
            let pretty_req = pretty_scheme(req);
            let alt = supported
                .iter()
                .map(|s| format!("--auth {s}"))
                .collect::<Vec<_>>()
                .join(" or ");
            if alt.is_empty() {
                format!("{pretty_req} auth is not accepted at {method} {endpoint}.")
            } else {
                format!("{pretty_req} auth is not accepted at {method} {endpoint}. Use {alt}.")
            }
        }
        (None, Some(avail)) => {
            let has = if avail.is_empty() {
                "none".to_string()
            } else {
                avail.join(", ")
            };
            let accepts = if supported.is_empty() {
                "none".to_string()
            } else {
                supported.join(", ")
            };
            let suggest = supported
                .first()
                .map(|s| format!(" Add credentials with: xr auth {s} --app <APP>."))
                .unwrap_or_default();
            format!(
                "No stored auth method on the active app is accepted at {method} {endpoint}. App has: {has}. Endpoint accepts: {accepts}.{suggest}"
            )
        }
        (None, None) => {
            format!("Auth method is not accepted at {method} {endpoint}.")
        }
    }
}

/// Maps a wire-format auth string to its pretty-printed scheme name.
///
/// `"app" -> "Bearer (app)"`, `"oauth1" -> "OAuth 1.0a"`,
/// `"oauth2" -> "OAuth 2.0"`. Falls back to the input for unknown strings.
fn pretty_scheme(name: &str) -> String {
    match name {
        "app" => "Bearer (app)".to_string(),
        "oauth1" => "OAuth 1.0a".to_string(),
        "oauth2" => "OAuth 2.0".to_string(),
        other => other.to_string(),
    }
}

#[allow(dead_code)] // Public library API — used by consumers and integration tests
impl XurlError {
    /// Create an API error with an HTTP status code and response body.
    pub fn api(status: u16, body: impl Into<String>) -> Self {
        Self::Api {
            status,
            body: body.into(),
        }
    }

    /// Create a validation error for non-HTTP error conditions.
    pub fn validation(body: impl Into<String>) -> Self {
        Self::Validation(body.into())
    }

    /// Create an auth error with a descriptive message.
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    /// Create an auth error with a message and underlying cause.
    pub fn auth_with_cause(message: &str, cause: &dyn std::fmt::Display) -> Self {
        Self::Auth(format!("{message} (cause: {cause})"))
    }

    /// Create a token store error.
    pub fn token_store(message: impl Into<String>) -> Self {
        Self::TokenStore(message.into())
    }

    /// Returns true if this is an API error (HTTP status >= 400).
    #[must_use]
    pub fn is_api(&self) -> bool {
        matches!(self, Self::Api { .. })
    }

    /// Returns true if this is a validation error.
    #[must_use]
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation(_))
    }

    /// Returns a typed kebab-case identifier for this error.
    ///
    /// The closed set is the envelope `reason` vocabulary that agents
    /// pattern-match on. Never returns English; never embeds state.
    ///
    /// | Variant                | `kind()`         |
    /// | ---------------------- | ---------------- |
    /// | `Auth`                 | `auth-required`  |
    /// | `TokenStore`           | `token-store`    |
    /// | `Api { 401, .. }`      | `auth-required`  |
    /// | `Api { 429, .. }`      | `rate-limited`   |
    /// | `Api { 404, .. }`      | `not-found`      |
    /// | `Api { other, .. }`    | `network-error`  |
    /// | `Http`                 | `network-error`  |
    /// | `Io`                   | `io`             |
    /// | `Json`                 | `serialization`  |
    /// | `InvalidMethod`        | `invalid-method` |
    /// | `Validation`           | `validation`     |
    /// | `InvalidUrl`           | `invalid-url`    |
    /// | `InvalidPathParam`     | `invalid-path-param` |
    /// | `Internal`             | `internal`       |
    /// | `EnvelopeAlreadyEmitted` | `confirmation-required` |
    /// | `AuthMethodMismatch`   | `auth-method-mismatch` |
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Auth(_) => "auth-required",
            Self::TokenStore(_) => "token-store",
            Self::Api { status: 401, .. } => "auth-required",
            Self::Api { status: 429, .. } => "rate-limited",
            Self::Api { status: 404, .. } => "not-found",
            Self::Api { .. } => "network-error",
            Self::Http(_) => "network-error",
            Self::Io(_) => "io",
            Self::Json(_) => "serialization",
            Self::InvalidMethod(_) => "invalid-method",
            Self::AuthMethodMismatch { .. } => "auth-method-mismatch",
            Self::Validation(_) => "validation",
            Self::InvalidUrl(_) => "invalid-url",
            Self::InvalidPathParam { .. } => "invalid-path-param",
            Self::Internal(_) => "internal",
            Self::EnvelopeAlreadyEmitted { .. } => "confirmation-required",
        }
    }

    /// Returns the structured exit code for this error.
    ///
    /// Pattern-matches on `Api { status, .. }` directly for HTTP errors,
    /// preserves string-scanning for `Http` transport errors (no structured
    /// status available), and maps `Validation` to `EXIT_GENERAL_ERROR`.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Auth(_) | Self::TokenStore(_) => EXIT_AUTH_REQUIRED,
            Self::Api { status: 401, .. } => EXIT_AUTH_REQUIRED,
            Self::Api { status: 429, .. } => EXIT_RATE_LIMITED,
            Self::Api { status: 404, .. } => EXIT_NOT_FOUND,
            Self::Http(msg) if msg.contains("401") || msg.contains("Unauthorized") => {
                EXIT_AUTH_REQUIRED
            }
            Self::Http(msg) if msg.contains("429") => EXIT_RATE_LIMITED,
            Self::Http(msg) if msg.contains("404") => EXIT_NOT_FOUND,
            Self::Io(_) => EXIT_NETWORK_ERROR,
            Self::Validation(_)
            | Self::InvalidUrl(_)
            | Self::InvalidPathParam { .. }
            | Self::Internal(_) => EXIT_GENERAL_ERROR,
            Self::AuthMethodMismatch { .. } => EXIT_AUTH_MISMATCH,
            Self::EnvelopeAlreadyEmitted { exit_code } => *exit_code,
            _ => EXIT_GENERAL_ERROR,
        }
    }
}

impl From<reqwest::Error> for XurlError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err.to_string())
    }
}

impl From<std::io::Error> for XurlError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for XurlError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<serde_yaml::Error> for XurlError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<url::ParseError> for XurlError {
    fn from(err: url::ParseError) -> Self {
        Self::Http(err.to_string())
    }
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, XurlError>;

// ── Exit codes ─────────────────────────────────────────────────────

/// Structured exit codes for machine-readable error handling.
///
/// Follows the sysexits-style matrix from the agent-native CLI envelope
/// pattern (corpus doc #1):
/// - `0` (`EXIT_SUCCESS`): success.
/// - `1` (`EXIT_GENERAL_ERROR`): general / user-recoverable error.
/// - `2` (`EXIT_USAGE_ERROR`): clap usage error (invalid args). Not surfaced
///   here; emitted directly by the runner on parse failure.
/// - `3` (`EXIT_RATE_LIMITED`): API rate limit hit — agent should back off.
/// - `4` (`EXIT_NOT_FOUND`): resource not found.
/// - `5` (`EXIT_NETWORK_ERROR`): network / connectivity issue.
/// - `77` (`EXIT_AUTH_REQUIRED`): authentication required. Matches sysexits
///   `EX_NOPERM`; disambiguates from clap `EX_USAGE` (2). Behavior change in
///   v1.3.0 — auth-required errors were previously exit `2`.
/// - `2` (`EXIT_AUTH_MISMATCH`): auth method mismatch — the user supplied
///   `--auth X` for an endpoint that does not accept `X`. Distinct from
///   `EXIT_AUTH_REQUIRED` (missing credential) — this signals a *wrong*
///   credential request that's fixable by changing `--auth`. Shares the
///   `EX_USAGE` numeric value with clap because both are usage faults.
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_SUCCESS: i32 = 0;
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_GENERAL_ERROR: i32 = 1;
/// Auth method mismatch. `EX_USAGE` from sysexits — `2`.
///
/// Surfaces when `--auth X` is in the user's invocation and the endpoint's
/// matrix entry does not accept `X`. Also surfaces in U7's empty-intersection
/// path when auto-detect finds no compatible stored credential. Distinct
/// from [`EXIT_AUTH_REQUIRED`] (= `77`, missing credential).
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_AUTH_MISMATCH: i32 = 2;
/// Authentication required. `EX_NOPERM` from sysexits — `77`.
///
/// **Behavior change in v1.3.0:** auth-required errors moved from exit `2`
/// to `77` so the code unambiguously distinguishes auth failures from clap
/// usage errors (which keep `EX_USAGE` = `2`).
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_AUTH_REQUIRED: i32 = 77;
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_RATE_LIMITED: i32 = 3;
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_NOT_FOUND: i32 = 4;
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_NETWORK_ERROR: i32 = 5;

/// Maps an [`XurlError`] to a structured exit code.
///
/// Free-function shim delegating to [`XurlError::exit_code`].
#[allow(dead_code)] // Public library API — used by consumers
#[must_use]
pub fn exit_code_for_error(e: &XurlError) -> i32 {
    e.exit_code()
}
