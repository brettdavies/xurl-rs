//! The library's error type and the exit codes that classify it.
//!
//! Display strings are lowercase fragments with no prefix and no trailing
//! period, so they read cleanly inside an embedder's error chain; `xr`
//! applies its own prefixes when it renders one.

use serde::{Deserialize, Serialize};

/// What the caller should do next. Closed set; agents branch on it.
///
/// A newer release can add a member, so a caller treats one it does not
/// recognize as its default branch.
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
    /// The word named no command; read the help of the nearest one.
    // `xr` reaches this alone: its unknown-command envelope carries it
    // (`crates/xurl-cli/src/cli/runner.rs`, `render_unknown_command`), and no
    // library error returns it.
    #[doc(hidden)]
    ShowHelp,
    /// Media was still processing when the wait's deadline passed; wait on
    /// the same media id again.
    ResumeWait,
    /// X rate limited the request and said when the window resets; send it
    /// again once that time has passed.
    WaitAndRetry,
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

/// A lower-level failure an [`Error`] wraps, as
/// [`std::error::Error::source`] returns it.
pub type Source = Box<dyn std::error::Error + Send + Sync + 'static>;

/// The library's error type.
///
/// The enum is `#[non_exhaustive]`: variants are added as the X API grows,
/// so a downstream match keeps a wildcard arm. In-crate, [`Self::kind`] and
/// [`Self::exit_code`] match every variant by name, so a new variant is
/// classified before it ships.
///
/// A variant that wraps a lower-level failure keeps it: `source()` returns
/// the `reqwest`, `std::io`, `serde_json`, or `serde_yaml` error underneath,
/// so a caller can walk the chain or downcast to it. `Display` stays the
/// message alone.
///
/// # Example
///
/// ```rust,no_run
/// use xdk::Error;
/// # fn run() -> Result<(), Error> {
/// # let result: Result<(), Error> = Err(Error::validation("missing field"));
/// match result {
///     Ok(()) => println!("ok"),
///     Err(Error::Api { status, body, .. }) => eprintln!("api {status}: {body}"),
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
    #[error("{message}")]
    Http {
        /// What failed.
        message: String,
        /// The failure underneath, typically a `reqwest::Error`.
        #[source]
        source: Option<Source>,
    },

    /// File / IO error.
    #[error("{message}")]
    Io {
        /// What failed.
        message: String,
        /// The failure underneath, typically a `std::io::Error`.
        #[source]
        source: Option<Source>,
    },

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
        /// When the rate-limit window resets, as seconds since the Unix
        /// epoch: the `x-rate-limit-reset` header of this response. `None`
        /// when the response named no reset, which is every response but a
        /// 429 and some 429s too. It is never the window an earlier response
        /// reported, so a caller that waits on it waits on what X said about
        /// this request.
        reset_at: Option<u64>,
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
    #[error("{message}")]
    Json {
        /// What failed.
        message: String,
        /// The failure underneath, typically a `serde_json::Error`.
        #[source]
        source: Option<Source>,
    },

    /// Authentication error with sub-type context.
    #[error("{message}")]
    Auth {
        /// What failed.
        message: String,
        /// The failure underneath, when a lower-level error caused it.
        #[source]
        source: Option<Source>,
    },

    /// Token store persistence / lookup error.
    #[error("{message}")]
    TokenStore {
        /// What failed.
        message: String,
        /// The failure underneath, when a lower-level error caused it.
        #[source]
        source: Option<Source>,
    },

    /// A wait on media processing reached its deadline with the job still
    /// running. The upload itself is intact: X keeps a media id valid for 24
    /// hours, so another wait on [`Self::ProcessingTimeout::media_id`] picks
    /// the job up where this one left it.
    #[error("media {media_id} was still processing when the {}-second wait ended", waited.as_secs())]
    ProcessingTimeout {
        /// The media id whose processing had not finished.
        media_id: String,
        /// The deadline the wait ran to.
        waited: std::time::Duration,
    },

    /// Auth method mismatch: the caller asked for, or auto-detect resolved,
    /// a scheme the endpoint's matrix entry does not accept. The payload is
    /// boxed so the error stays small on every `Result` an embedder returns;
    /// [`AuthMismatch`] describes the three shapes it takes.
    #[error("{0}")]
    AuthMethodMismatch(Box<AuthMismatch>),
}

crate::assert_send_sync!(Error);

// A boxed `dyn Error` carries no unwind-safety auto traits, so wrapping a
// cause would silently take them from `Error` and from every type holding
// one. The cause is only ever read, through `source()`, so a panic cannot
// leave it half-written; `anyhow::Error` makes the same two impls.
impl std::panic::UnwindSafe for Error {}
impl std::panic::RefUnwindSafe for Error {}

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
            reset_at: None,
        }
    }

    /// Create a validation error for non-HTTP error conditions.
    pub fn validation(body: impl Into<String>) -> Self {
        Self::Validation(body.into())
    }

    /// Create a transport error from a message alone.
    pub fn http(message: impl Into<String>) -> Self {
        Self::Http {
            message: message.into(),
            source: None,
        }
    }

    /// Create a file / IO error from a message alone.
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
            source: None,
        }
    }

    /// Create a serialization error from a message alone.
    pub fn json(message: impl Into<String>) -> Self {
        Self::Json {
            message: message.into(),
            source: None,
        }
    }

    /// Create an auth error with a descriptive message.
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth {
            message: message.into(),
            source: None,
        }
    }

    /// Create an auth error with a message and underlying cause.
    pub fn auth_with_cause(message: &str, cause: &dyn std::fmt::Display) -> Self {
        Self::auth(format!("{message} (cause: {cause})"))
    }

    /// Create a token store error.
    pub fn token_store(message: impl Into<String>) -> Self {
        Self::TokenStore {
            message: message.into(),
            source: None,
        }
    }

    /// Attaches the lower-level failure this error was built from, so
    /// [`std::error::Error::source`] returns it. A variant that carries no
    /// source is returned unchanged.
    #[must_use]
    pub fn with_source(mut self, cause: impl Into<Source>) -> Self {
        if let Self::Http { source, .. }
        | Self::Io { source, .. }
        | Self::Json { source, .. }
        | Self::Auth { source, .. }
        | Self::TokenStore { source, .. } = &mut self
        {
            *source = Some(cause.into());
        }
        self
    }

    /// The recovery step this error carries, when the library can name one.
    ///
    /// A 403 that says X refused the app is `EnrollApp`; a bare 403 on X's
    /// Pay-per-use enrollment failure is unactionable without it. A
    /// processing timeout is `ResumeWait`, and a 429 that named its reset is
    /// `WaitAndRetry`; a 429 that named none has no time to wait for. Every
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
            Self::Api { status, body, .. } if refuses_enrollment(*status, body) => {
                Some(NextAction::EnrollApp)
            }
            Self::Api {
                status: 429,
                reset_at: Some(_),
                ..
            } => Some(NextAction::WaitAndRetry),
            Self::ProcessingTimeout { .. } => Some(NextAction::ResumeWait),
            Self::Api { .. }
            | Self::Http { .. }
            | Self::Io { .. }
            | Self::InvalidMethod(_)
            | Self::Validation(_)
            | Self::InvalidUrl(_)
            | Self::InvalidPathParam { .. }
            | Self::Internal(_)
            | Self::Json { .. }
            | Self::Auth { .. }
            | Self::TokenStore { .. } => None,
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
            Self::Api { status, body, .. } if refuses_enrollment(*status, body) => {
                Some(ENROLLMENT_DOCS)
            }
            Self::Api { status: 401, .. } | Self::Auth { .. } | Self::AuthMethodMismatch(_) => {
                Some(AUTHENTICATION_DOCS)
            }
            Self::Api { status: 429, .. } => Some(RATE_LIMIT_DOCS),
            Self::Api { .. }
            | Self::Http { .. }
            | Self::Io { .. }
            | Self::InvalidMethod(_)
            | Self::Validation(_)
            | Self::InvalidUrl(_)
            | Self::InvalidPathParam { .. }
            | Self::Internal(_)
            | Self::Json { .. }
            | Self::TokenStore { .. }
            | Self::ProcessingTimeout { .. } => None,
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
    /// | `ProcessingTimeout`    | `processing-timeout` |
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Auth { .. } => "auth-required",
            Self::TokenStore { .. } => "token-store",
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
            Self::Http { .. } => "network-error",
            Self::Io { .. } => "io",
            Self::Json { .. } => "serialization",
            Self::InvalidMethod(_) => "invalid-method",
            Self::AuthMethodMismatch(_) => "auth-method-mismatch",
            Self::Validation(_) => "validation",
            Self::InvalidUrl(_) => "invalid-url",
            Self::InvalidPathParam { .. } => "invalid-path-param",
            Self::Internal(_) => "internal",
            Self::ProcessingTimeout { .. } => "processing-timeout",
        }
    }

    /// Returns the structured exit code for this error.
    ///
    /// Pattern-matches on `Api { status, .. }` for HTTP errors; a transport
    /// failure (`Http`) never carries a status, and its message quotes the
    /// URL, so it is never read for one. One arm per variant, with no
    /// wildcard: a variant added without an exit-code decision is a compile
    /// error, not a silent exit 1.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Auth { .. } | Self::TokenStore { .. } => EXIT_AUTH_REQUIRED,
            Self::Api { status: 401, .. } => EXIT_AUTH_REQUIRED,
            Self::Api { status: 429, .. } => EXIT_RATE_LIMITED,
            Self::Api { status: 404, .. } => EXIT_NOT_FOUND,
            Self::Api { .. } => EXIT_GENERAL_ERROR,
            Self::Http { .. } | Self::Io { .. } => EXIT_NETWORK_ERROR,
            Self::Json { .. }
            | Self::InvalidMethod(_)
            | Self::Validation(_)
            | Self::InvalidUrl(_)
            | Self::InvalidPathParam { .. }
            | Self::Internal(_)
            | Self::ProcessingTimeout { .. } => EXIT_GENERAL_ERROR,
            Self::AuthMethodMismatch(_) => EXIT_AUTH_MISMATCH,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::http(err.to_string()).with_source(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::io(err.to_string()).with_source(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::json(err.to_string()).with_source(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self::json(err.to_string()).with_source(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Self::http(err.to_string()).with_source(err)
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
/// A transport or filesystem failure: [`Error::Http`] and [`Error::Io`]. An
/// API response whose status has no code of its own exits
/// [`EXIT_GENERAL_ERROR`].
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
