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

    /// API returned an error response body (raw JSON).
    #[error("{0}")]
    Api(String),

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

impl XurlError {
    /// Create an API error from a raw JSON response body.
    ///
    /// If the message is valid JSON, Display will emit the raw JSON (matching
    /// the Go behaviour where `Error()` returns `json.RawMessage` directly).
    pub fn api(body: impl Into<String>) -> Self {
        Self::Api(body.into())
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

    /// Returns true if this is an API error.
    pub fn is_api(&self) -> bool {
        matches!(self, Self::Api(_))
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
