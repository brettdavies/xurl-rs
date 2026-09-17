//! The library's error type and the exit codes that classify it.
//!
//! Display strings are lowercase fragments with no prefix and no trailing
//! period, so they read cleanly inside an embedder's error chain; `xr`
//! applies its own prefixes when it renders one.

use serde::{Deserialize, Serialize};

/// What the caller should do next. Closed set; agents branch on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum NextAction {
    /// No app carries client credentials; register one.
    RegisterApp,
    /// The target app has credentials and no token; sign in.
    SignIn,
    /// Another app is the one to use; rerun naming it.
    SelectApp,
    /// The store could not be read or parsed; look at the file.
    InspectStore,
    /// X refused the app; enroll it in the developer portal.
    EnrollApp,
}

/// The enrollment recipe for an app X refuses.
const ENROLLMENT_DOCS: &str = "https://github.com/brettdavies/xurl-rs#x-platform-enrollment";
/// Where X documents its authentication methods.
const AUTHENTICATION_DOCS: &str = "https://docs.x.com/resources/fundamentals/authentication";
/// Where X documents its rate limits.
const RATE_LIMIT_DOCS: &str = "https://docs.x.com/resources/fundamentals/rate-limits";

/// Whether an API refusal is X declining the app itself rather than the
/// request: a 403 whose body carries either enrollment marker.
#[must_use]
pub fn refuses_enrollment(status: u16, body: &str) -> bool {
    if status != 403 {
        return false;
    }
    let haystack = body.to_ascii_lowercase();
    haystack.contains("client-not-enrolled") || haystack.contains("client-forbidden")
}

/// The library's error type.
///
/// The enum is `#[non_exhaustive]`: variants are added as the X API grows,
/// so a downstream match keeps a wildcard arm. In-crate, [`Self::kind`] and
/// [`Self::exit_code`] match every variant by name, so a new variant is
/// classified before it ships.
///
/// # Example
///
/// ```rust,no_run
/// use xdk::Error;
/// # fn run() -> Result<(), Error> {
/// # let result: Result<(), Error> = Err(Error::validation("missing field"));
/// match result {
///     Ok(()) => println!("ok"),
///     Err(Error::Api { status, body }) => eprintln!("api {status}: {body}"),
///     Err(Error::Validation(msg)) => eprintln!("validation: {msg}"),
///     Err(Error::InvalidUrl(url)) => eprintln!("bad URL: {url}"),
///     Err(other) => eprintln!("{} (kind={})", other, other.kind()),
/// }
/// # Ok(()) }
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// HTTP transport / request construction error.
    #[error("{0}")]
    Http(String),

    /// File / IO error.
    #[error("{0}")]
    Io(String),

    /// Invalid HTTP method supplied.
    #[error("invalid HTTP method: {0}")]
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
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    /// Path-parameter value contained a character that would break URL
    /// semantics (`/`, `?`, `#`, or `%`). Surfaces real IDs that contain
    /// stray separators rather than silently encoding them.
    #[error("invalid path parameter {name:?}: value {value:?} contains a reserved character")]
    InvalidPathParam {
        /// Name of the offending `{param}` segment in the path template.
        name: String,
        /// Caller-supplied value that failed validation.
        value: String,
    },

    /// Internal invariant violated — typically a programmer error such as
    /// a path template referencing a `{name}` segment that the caller never
    /// supplied in `path_params`.
    #[error("internal error: {0}")]
    Internal(String),

    /// JSON serialization / deserialization error.
    #[error("{0}")]
    Json(String),

    /// Authentication error with sub-type context.
    #[error("{0}")]
    Auth(String),

    /// Token store persistence / lookup error.
    #[error("{0}")]
    TokenStore(String),

    /// Auth method mismatch: the caller asked for, or auto-detect resolved,
    /// a scheme the endpoint's matrix entry does not accept. The payload is
    /// boxed so the error stays small on every `Result` an embedder returns;
    /// [`AuthMismatch`] describes the three shapes it takes.
    #[error("{0}")]
    AuthMethodMismatch(Box<AuthMismatch>),
}

crate::assert_send_sync!(Error);

/// What an [`Error::AuthMethodMismatch`] describes.
///
/// Three shapes share the type:
/// - **Explicit mismatch**: `requested = Some("app"|"oauth1"|"oauth2")`,
///   `available_in_app = None`. The caller asked for a scheme the endpoint
///   does not accept.
/// - **Empty intersection**: `requested = None`,
///   `available_in_app = Some([nonempty])`. Auto-detect found no stored
///   credential on the active app that the endpoint accepts.
/// - **Wrong app**: `requested = None`, `other_apps_with_creds =
///   Some([nonempty])`. The active app stores no credentials but other apps
///   in the store do. `available_in_app` is `Some([])`, or `Some(["app"])`
///   when `XURL_BEARER_TOKEN` supplies a bearer the endpoint does not accept.
///
/// `Display` is the lowercase fragment the library reports; `xr` composes
/// its own recovery wording from the same fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthMismatch {
    /// Path template (for example `/2/users/{id}/likes`), verbatim from the
    /// spec so an agent can match on it.
    pub endpoint: String,
    /// The path with `{param}` segments substituted (for example
    /// `/2/users/12345/likes`); messages prefer it over `endpoint`. `None`
    /// when no substitution context was available.
    pub rendered_url: Option<String>,
    /// HTTP method, already uppercased.
    pub method: String,
    /// What the caller asked for: `Some("app"|"oauth1"|"oauth2")` in the
    /// explicit-mismatch shape, `None` otherwise.
    pub requested: Option<String>,
    /// The schemes the endpoint accepts, as wire strings.
    pub supported: Vec<String>,
    /// The schemes the active app has stored: `None` in the
    /// explicit-mismatch shape, `Some([nonempty])` in the empty-intersection
    /// shape, `Some([])` in the wrong-app shape.
    pub available_in_app: Option<Vec<String>>,
    /// The active app's name, when one was resolved.
    pub app: Option<String>,
    /// Other apps in the store that hold credentials; populated only in the
    /// wrong-app shape.
    pub other_apps_with_creds: Option<Vec<String>>,
}

crate::assert_send_sync!(AuthMismatch);

impl AuthMismatch {
    /// Which of the three shapes these fields describe.
    #[doc(hidden)]
    #[must_use]
    pub fn shape(&self) -> MismatchShape<'_> {
        mismatch_shape(
            self.requested.as_deref(),
            self.available_in_app.as_deref(),
            self.other_apps_with_creds.as_deref(),
        )
    }
}

impl std::fmt::Display for AuthMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self.rendered_url.as_deref().unwrap_or(&self.endpoint);
        let method = &self.method;
        let app_name = self.app.as_deref().unwrap_or("the active app");
        let list = |items: &[String]| {
            if items.is_empty() {
                "none".to_string()
            } else {
                items.join(", ")
            }
        };
        match self.shape() {
            MismatchShape::Explicit { requested } if self.supported.is_empty() => {
                write!(f, "{requested} auth is not accepted at {method} {path}")
            }
            MismatchShape::Explicit { requested } => {
                let accepts = list(&self.supported);
                write!(
                    f,
                    "{requested} auth is not accepted at {method} {path} (accepts {accepts})"
                )
            }
            MismatchShape::WrongApp { others } => {
                let alts = others.join(", ");
                write!(
                    f,
                    "app '{app_name}' holds no credentials for {method} {path} (other apps with credentials: {alts})"
                )
            }
            MismatchShape::EmptyIntersection { available } => {
                let has = list(available);
                let accepts = list(&self.supported);
                write!(
                    f,
                    "no stored auth method on app '{app_name}' is accepted at {method} {path} (app has {has}; endpoint accepts {accepts})"
                )
            }
            MismatchShape::Unknown => write!(f, "auth method is not accepted at {method} {path}"),
        }
    }
}

impl From<AuthMismatch> for Error {
    fn from(mismatch: AuthMismatch) -> Self {
        Self::AuthMethodMismatch(Box::new(mismatch))
    }
}

/// Which situation an `AuthMethodMismatch`'s fields describe.
///
/// The library's Display and the `xr` renderer both classify through
/// [`mismatch_shape`], so the two cannot sort one error into different
/// shapes.
#[doc(hidden)]
#[derive(Debug)]
pub enum MismatchShape<'a> {
    /// The caller asked for a scheme the endpoint does not accept.
    Explicit { requested: &'a str },
    /// The active app stores nothing, but other apps hold credentials.
    WrongApp { others: &'a [String] },
    /// Nothing the active app stores is accepted at the endpoint.
    EmptyIntersection { available: &'a [String] },
    /// No app context was available when the error was built.
    Unknown,
}

/// Classifies the three optional fields into one [`MismatchShape`].
///
/// Only the wrong-app branch sets `other_apps_with_creds`, so
/// `available_in_app` may still carry an env-supplied bearer there.
#[doc(hidden)]
pub fn mismatch_shape<'a>(
    requested: Option<&'a str>,
    available_in_app: Option<&'a [String]>,
    other_apps_with_creds: Option<&'a [String]>,
) -> MismatchShape<'a> {
    match (requested, available_in_app, other_apps_with_creds) {
        (Some(requested), _, _) => MismatchShape::Explicit { requested },
        (None, Some(_), Some(others)) if !others.is_empty() => MismatchShape::WrongApp { others },
        (None, Some(available), _) => MismatchShape::EmptyIntersection { available },
        (None, None, _) => MismatchShape::Unknown,
    }
}

#[allow(dead_code)] // Public library API — used by consumers and integration tests
impl Error {
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

    /// The recovery step this error carries, when the library can name one.
    ///
    /// A 403 that says X refused the app is `EnrollApp`; a bare 403 on X's
    /// Pay-per-use enrollment failure is unactionable without it. Every
    /// other error is `None` here: the steps that depend on a credential
    /// store are the binary's to choose.
    ///
    /// [`NextAction`] is library API by intent, not a rendering detail: an
    /// embedder branches on it the way `xr` renders it, so it stays on the
    /// error rather than in any one consumer. The enum is `#[non_exhaustive]`,
    /// so a new action is a minor release and a `match` needs a wildcard arm.
    #[must_use]
    pub fn next_action(&self) -> Option<NextAction> {
        match self {
            Self::Api { status, body } if refuses_enrollment(*status, body) => {
                Some(NextAction::EnrollApp)
            }
            Self::Api { .. }
            | Self::Http(_)
            | Self::Io(_)
            | Self::InvalidMethod(_)
            | Self::Validation(_)
            | Self::InvalidUrl(_)
            | Self::InvalidPathParam { .. }
            | Self::Internal(_)
            | Self::Json(_)
            | Self::Auth(_)
            | Self::TokenStore(_) => None,
            // The stored-credential recovery steps are the binary's: it knows
            // which apps hold what and names the invocation.
            Self::AuthMethodMismatch(_) => None,
        }
    }

    /// The page that documents this error's recovery, when one exists.
    ///
    /// An enrollment refusal names the recipe that moves the app to the
    /// right package, a credential failure points at X's authentication
    /// overview, and a 429 at its rate-limit rules. One arm per variant, so
    /// a new variant decides its pointer before it ships.
    #[must_use]
    pub fn docs_url(&self) -> Option<&'static str> {
        match self {
            Self::Api { status, body } if refuses_enrollment(*status, body) => {
                Some(ENROLLMENT_DOCS)
            }
            Self::Api { status: 401, .. } | Self::Auth(_) | Self::AuthMethodMismatch(_) => {
                Some(AUTHENTICATION_DOCS)
            }
            Self::Api { status: 429, .. } => Some(RATE_LIMIT_DOCS),
            Self::Api { .. }
            | Self::Http(_)
            | Self::Io(_)
            | Self::InvalidMethod(_)
            | Self::Validation(_)
            | Self::InvalidUrl(_)
            | Self::InvalidPathParam { .. }
            | Self::Internal(_)
            | Self::Json(_)
            | Self::TokenStore(_) => None,
        }
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
    /// | `Api { 403, .. }`      | `forbidden`      |
    /// | `Api { 404, .. }`      | `not-found`      |
    /// | `Api { 429, .. }`      | `rate-limited`   |
    /// | `Api { 400 \| 422, .. }` | `invalid-request` |
    /// | `Api { 5xx, .. }`      | `server-error`   |
    /// | `Api { other, .. }`    | `api-error`      |
    /// | `Http`                 | `network-error`  |
    /// | `Io`                   | `io`             |
    /// | `Json`                 | `serialization`  |
    /// | `InvalidMethod`        | `invalid-method` |
    /// | `Validation`           | `validation`     |
    /// | `InvalidUrl`           | `invalid-url`    |
    /// | `InvalidPathParam`     | `invalid-path-param` |
    /// | `Internal`             | `internal`       |
    /// | `AuthMethodMismatch`   | `auth-method-mismatch` |
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Auth(_) => "auth-required",
            Self::TokenStore(_) => "token-store",
            Self::Api { status: 401, .. } => "auth-required",
            Self::Api { status: 403, .. } => "forbidden",
            Self::Api { status: 404, .. } => "not-found",
            Self::Api { status: 429, .. } => "rate-limited",
            Self::Api {
                status: 400 | 422, ..
            } => "invalid-request",
            Self::Api {
                status: 500..=599, ..
            } => "server-error",
            Self::Api { .. } => "api-error",
            Self::Http(_) => "network-error",
            Self::Io(_) => "io",
            Self::Json(_) => "serialization",
            Self::InvalidMethod(_) => "invalid-method",
            Self::AuthMethodMismatch(_) => "auth-method-mismatch",
            Self::Validation(_) => "validation",
            Self::InvalidUrl(_) => "invalid-url",
            Self::InvalidPathParam { .. } => "invalid-path-param",
            Self::Internal(_) => "internal",
        }
    }

    /// Returns the structured exit code for this error.
    ///
    /// Pattern-matches on `Api { status, .. }` directly for HTTP errors,
    /// preserves string-scanning for `Http` transport errors (no structured
    /// status available), and maps `Validation` to `EXIT_GENERAL_ERROR`.
    /// One arm per variant, with no wildcard: a variant added without an
    /// exit-code decision is a compile error, not a silent exit 1.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Auth(_) | Self::TokenStore(_) => EXIT_AUTH_REQUIRED,
            Self::Api { status: 401, .. } => EXIT_AUTH_REQUIRED,
            Self::Api { status: 429, .. } => EXIT_RATE_LIMITED,
            Self::Api { status: 404, .. } => EXIT_NOT_FOUND,
            Self::Api { .. } => EXIT_GENERAL_ERROR,
            Self::Http(msg) if msg.contains("401") || msg.contains("Unauthorized") => {
                EXIT_AUTH_REQUIRED
            }
            Self::Http(msg) if msg.contains("429") => EXIT_RATE_LIMITED,
            Self::Http(msg) if msg.contains("404") => EXIT_NOT_FOUND,
            Self::Http(_) => EXIT_GENERAL_ERROR,
            Self::Io(_) => EXIT_NETWORK_ERROR,
            Self::Json(_)
            | Self::InvalidMethod(_)
            | Self::Validation(_)
            | Self::InvalidUrl(_)
            | Self::InvalidPathParam { .. }
            | Self::Internal(_) => EXIT_GENERAL_ERROR,
            Self::AuthMethodMismatch(_) => EXIT_AUTH_MISMATCH,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Self::Http(err.to_string())
    }
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

// ── Auth failure messages ──────────────────────────────────────────
//
// One constant per message so its construction sites and the runner's
// hint seam agree on the exact string; the runner matches on them to
// decide whether a recovery hint applies.

/// The message every no-credentials failure carries.
pub const NO_AUTH_METHOD: &str = "NoAuthMethod: no authentication method available";

/// The message every missing-`OAuth2`-token failure carries.
pub const NO_OAUTH2_TOKEN: &str = "TokenNotFound: oauth2 token not found";

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
///   `EX_NOPERM`; disambiguates from clap `EX_USAGE` (2).
/// - `2` (`EXIT_AUTH_MISMATCH`): auth method mismatch — the user supplied
///   `--auth X` for an endpoint that does not accept `X`. Distinct from
///   `EXIT_AUTH_REQUIRED` (missing credential) — this signals a *wrong*
///   credential request that's fixable by changing `--auth`. Shares the
///   `EX_USAGE` numeric value with clap because both are usage faults.
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_SUCCESS: i32 = 0;
/// General / user-recoverable error. Sysexits default for anything not
/// covered by a more specific code.
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_GENERAL_ERROR: i32 = 1;
/// Auth method mismatch. `EX_USAGE` from sysexits — `2`.
///
/// Surfaces when `--auth X` is in the user's invocation and the endpoint's
/// matrix entry does not accept `X`. Also surfaces on the auto-detect
/// empty-intersection path when no stored credential on the active app
/// satisfies the endpoint. Distinct from [`EXIT_AUTH_REQUIRED`] (= `77`,
/// missing credential).
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_AUTH_MISMATCH: i32 = 2;
/// Usage error. `EX_USAGE` from sysexits — `2`.
///
/// Clap parse failures share this value, as do the errors a caller can fix
/// by changing the invocation rather than the credentials. Distinct in
/// meaning from [`EXIT_AUTH_MISMATCH`], which shares the number.
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_USAGE_ERROR: i32 = 2;
/// Authentication required. `EX_NOPERM` from sysexits — `77`.
///
/// Auth-required errors exit `77` rather than `2`, so the code unambiguously
/// distinguishes an auth failure from a clap usage error (`EX_USAGE` = `2`).
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_AUTH_REQUIRED: i32 = 77;
/// API rate limit hit (HTTP 429). Agents should back off and retry per
/// the response's rate-limit headers.
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_RATE_LIMITED: i32 = 3;
/// Resource not found (HTTP 404).
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_NOT_FOUND: i32 = 4;
/// A filesystem or connection failure the library reports as
/// [`Error::Io`]. An API response with any other status and a transport
/// failure both exit `EXIT_GENERAL_ERROR`.
#[allow(dead_code)] // Public library API — used by consumers
pub const EXIT_NETWORK_ERROR: i32 = 5;

/// Maps an [`Error`] to a structured exit code.
///
/// Free-function shim delegating to [`Error::exit_code`].
#[allow(dead_code)] // Public library API — used by consumers
#[must_use]
pub fn exit_code_for_error(e: &Error) -> i32 {
    e.exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refused_enrollment_names_the_enroll_step() {
        let refused = Error::api(403, r#"{"reason":"client-not-enrolled","detail":"x"}"#);
        assert_eq!(refused.next_action(), Some(NextAction::EnrollApp));
        let forbidden = Error::api(403, "CLIENT-FORBIDDEN");
        assert_eq!(forbidden.next_action(), Some(NextAction::EnrollApp));
    }

    #[test]
    fn error_fits_under_the_result_large_err_threshold() {
        let size = std::mem::size_of::<Error>();
        assert!(
            size <= 128,
            "Error is {size} bytes; clippy warns embedders above 128"
        );
    }

    #[test]
    fn other_errors_carry_no_step() {
        assert_eq!(Error::api(403, "plain forbidden").next_action(), None);
        assert_eq!(Error::api(401, "client-not-enrolled").next_action(), None);
        assert_eq!(Error::auth("x").next_action(), None);
    }
}
