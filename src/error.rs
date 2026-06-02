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

    /// JSON serialization / deserialization error.
    #[error("JSON Error: {0}")]
    Json(String),

    /// Authentication error with sub-type context.
    #[error("Auth Error: {0}")]
    Auth(String),

    /// Token store persistence / lookup error.
    #[error("Token Store Error: {0}")]
    TokenStore(String),
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
            Self::Validation(_) => "validation",
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
            Self::Validation(_) => EXIT_GENERAL_ERROR,
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
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_SUCCESS: i32 = 0;
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_GENERAL_ERROR: i32 = 1;
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
