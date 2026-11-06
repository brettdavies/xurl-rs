//! Canonical agent-native JSON envelope shape.
//!
//! Every `--output json|jsonl` response — success, dry-run, or error — is
//! shaped as one of these three variants. Agents dispatch on the `status`
//! field; the typed `reason` discriminator carries the closed kebab-case
//! kind on errors.
//!
//! The `#[derive(JsonSchema)]` here is the source of truth for the
//! `schema/output.schema.json` committed at the repo root. The drift guard
//! at `tests/schema_tests.rs` asserts byte-equality between the file and
//! the runtime-emitted schema; regenerate via:
//!
//! ```text
//! cargo run -p xurl-rs --bin xr -- schema --envelope --output json > schema/output.schema.json
//! ```
//!
//! Field shape mirrors the corpus pattern documented in
//! `solutions/architecture-patterns/anc-cli-output-envelope-pattern-2026-04-29.md`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cli::hints::NextStep;

/// The three envelope variants — one per `status` discriminator value.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Envelope {
    /// Success envelope. `payload` holds verb-specific fields; agents
    /// merge them flat at the top level for legibility.
    Ok {
        /// What the library warned about during the run, on the document a
        /// successful run writes on stderr when a warning fired. Stdout's
        /// document never carries it.
        #[serde(skip_serializing_if = "Option::is_none")]
        warnings: Option<Vec<Warning>>,
        /// Verb-specific payload fields. Encoded as additional top-level
        /// keys via `#[serde(flatten)]` at the call site.
        #[serde(flatten)]
        payload: Value,
    },
    /// Dry-run envelope. Its body declares the keys that are the envelope's
    /// own; any other key is one of the command's inputs. Boxed like the
    /// error body, for its size.
    DryRun(Box<DryRunBody>),
    /// Error envelope. Every field the runtime emits is declared on
    /// [`ErrorBody`], so an undeclared key cannot reach a caller. Boxed
    /// because that body is much larger than the success variant.
    Error(Box<ErrorBody>),
}

/// Every key a dry run's answer carries of its own, beside the `status` tag.
///
/// The object stays open, because each command adds its inputs. A refusal
/// carries the keys of the error envelope its real run would write, so `reason`
/// and those beside it mean what they mean on [`ErrorBody`]; each is absent
/// when the real run would succeed or the refusal does not name it.
#[derive(Debug, Serialize, JsonSchema)]
#[schemars(inline)]
#[non_exhaustive]
pub struct DryRunBody {
    /// True iff every precondition checked clean.
    pub would_succeed: bool,
    /// Exit code the verb would have returned on actual execution.
    pub exit_code: i32,
    /// The `xr schema` invocation that describes the command's response,
    /// on a command with a typed response. A raw request carries none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<String>,
    /// The command the answer is about. A raw request carries none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// The credential the request would go out under, read offline. Absent
    /// on a refusal, and on a command that sends no request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<DryRunAuth>,
    /// Present and `true` on a destructive command whose real run would stop
    /// to ask. Absent under `--force`, where nothing would stop.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_required: Option<bool>,
    /// Why the real run would fail, when it would: the `reason` of its error.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub reason: Option<Reason>,
    /// The input rule the real run would refuse, under `validation`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    /// The offending input, as it was given, when the broken `rule` names one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// The message the real run's error would carry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The step the real run's error would carry; one that reruns the
    /// invocation keeps `--dry-run`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step: Option<NextStep>,
    /// Endpoint template the request would target, when no stored scheme
    /// serves it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// HTTP method the request would use, on the same refusal or a raw
    /// request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Auth scheme the caller asked for, on the same refusal; `null` when
    /// auto-detect ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested: Option<Value>,
    /// Auth schemes the endpoint accepts, on the same refusal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<Vec<String>>,
    /// Auth schemes available for the active app, on the same refusal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_in_app: Option<Vec<String>>,
    /// The active app name, on the same refusal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    /// Other apps holding credentials, when the active one holds none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_apps_with_creds: Option<Vec<String>>,
    /// The command's inputs, flattened at the top level.
    #[serde(flatten)]
    pub payload: Value,
}

/// The `auth` object of a dry-run envelope: the credential a request would go
/// out under.
#[derive(Debug, Serialize, JsonSchema)]
#[non_exhaustive]
pub struct DryRunAuth {
    /// The auth scheme: `oauth1`, `oauth2`, or `app`.
    pub scheme: String,
    /// The app whose stored credential would be sent. Absent for a bearer
    /// token the environment supplies, which belongs to no stored app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    /// The `OAuth2` login, the one credential with a user name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Whether the `OAuth2` login's token has expired. An expired token is
    /// reported, not refreshed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expired: Option<bool>,
}

impl DryRunAuth {
    /// The report of a credential, naming a login only on `OAuth2`.
    #[must_use]
    pub fn new(
        scheme: &str,
        app: Option<String>,
        oauth2_login: Option<(Option<String>, bool)>,
    ) -> Self {
        let (username, token_expired) = match oauth2_login {
            Some((username, expired)) => (username, Some(expired)),
            None => (None, None),
        };
        Self {
            scheme: scheme.to_string(),
            app,
            username,
            token_expired,
        }
    }
}

/// The closed set of `reason` values, spelled in kebab-case on the wire.
///
/// A reason the library reports keeps the spelling of `xdk::Error::kind()`;
/// [`Reason::from_kind`] is where the two meet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Reason {
    /// No usable credential, or the API answered 401.
    AuthRequired,
    /// The endpoint does not accept the auth scheme the request would use.
    AuthMethodMismatch,
    /// The app holds no client credentials to sign in with.
    ClientCredentialsMissing,
    /// The API answered 429.
    RateLimited,
    /// The API answered 404.
    NotFound,
    /// The API answered 403.
    Forbidden,
    /// The API answered 400 or 422.
    InvalidRequest,
    /// The API answered with a 5xx status.
    ServerError,
    /// The API answered with any other error status.
    ApiError,
    /// The request got no response.
    NetworkError,
    /// The arguments did not parse.
    InvalidArgs,
    /// The first word names no command.
    UnknownCommand,
    /// The HTTP method is not one `xr` sends.
    InvalidMethod,
    /// The URL has a scheme other than `http` or `https`, or did not parse.
    InvalidUrl,
    /// A path parameter holds a character that would break the URL.
    InvalidPathParam,
    /// A value was refused without an HTTP error, before or after the request.
    Validation,
    /// A body could not be read or written as JSON.
    Serialization,
    /// A file or stream could not be read or written.
    Io,
    /// Another process holds the port `webhooks listen` was asked to bind.
    AddressInUse,
    /// The token store could not be read or written.
    TokenStore,
    /// An invariant inside `xr` or the library did not hold.
    #[default]
    Internal,
    /// Media was still processing when the wait ended.
    ProcessingTimeout,
    /// A destructive command ran under `--no-interactive` without `--force`.
    ConfirmationRequired,
    /// The command has to prompt and no terminal is attached.
    NoTty,
    /// `--page` asked for offset pagination, which the X API does not offer.
    UnsupportedPagination,
    /// `validate` was given input that is not JSON.
    InvalidJson,
    /// `validate` was given a schema name it does not know.
    UnknownSchema,
    /// `validate` found the document does not match its schema.
    ValidationFailed,
    /// `skill install` was given neither a host nor `--all`.
    MissingHost,
    /// A skill destination names the home directory and none is set.
    HomeNotSet,
    /// The skill destination already holds files: anything, under `skill
    /// install`, or something other than the bundle, under `skill update`.
    DestinationNotEmpty,
    /// The skill destination exists and is not a directory.
    DestinationIsFile,
    /// `git` was not found on `PATH`.
    GitNotFound,
    /// `git clone` exited with a non-zero status.
    GitCloneFailed,
    /// `skill update --all` passed over a host with nothing installed.
    NotInstalled,
    /// `skill update` could not swap a fresh clone in for the copy it was
    /// replacing, or remove that copy.
    RemoveFailed,
}

impl Reason {
    /// The reason spelled as the library spells `kind`, the value of
    /// `xdk::Error::kind()`; `None` for a kind this set does not hold.
    #[must_use]
    pub fn from_kind(kind: &str) -> Option<Self> {
        let kind: serde::de::value::StrDeserializer<'_, serde::de::value::Error> =
            serde::de::IntoDeserializer::into_deserializer(kind);
        Self::deserialize(kind).ok()
    }
}

/// A diagnostic the library raised during a run, as a structured document
/// carries it. It did not end the run: a failure is the envelope itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Warning {
    /// What the library reported, in its own words. The wording is not
    /// contract; branch on `target`.
    pub message: String,
    /// The part of `xr` that raised it, such as `xdk::config` or `xdk::store`.
    pub target: String,
}

/// Every field an error envelope can carry, beside the `status` tag.
///
/// The emitter in [`crate::cli::output`] constructs this value and serializes it,
/// which is what lets the generated schema describe exactly what agents see.
/// Optional fields are skipped when absent, so a consumer feature-detects by
/// key presence. `reason` is one of the closed set [`Reason`] declares.
///
/// The declaration is the whole error surface, not one emitter's: the
/// verb-local fields the `validate` and `skill` commands carry are declared
/// here too, because the generated schema closes the object and a key it
/// does not name would make that schema wrong about what callers receive.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ErrorBody {
    /// Typed kebab-case kind; agents pattern-match on this.
    ///
    /// The runtime emits: `auth-required`, `auth-method-mismatch`,
    /// `client-credentials-missing`, `rate-limited`, `not-found`, `forbidden`,
    /// `invalid-request`, `server-error`, `api-error`, `network-error`,
    /// `invalid-args`, `unknown-command`, `invalid-method`,
    /// `invalid-url`, `invalid-path-param`, `validation`, `serialization`,
    /// `io`, `address-in-use`, `token-store`, `internal`, `processing-timeout`,
    /// `confirmation-required`, `no-tty`, `unsupported-pagination`, and the
    /// verb-local `invalid-json`, `unknown-schema`, `validation-failed`,
    /// `missing-host`, `home-not-set`, `destination-not-empty`,
    /// `destination-is-file`, `git-not-found`, `git-clone-failed`,
    /// `not-installed`, and `remove-failed`. The set is closed, and a newer
    /// release can add to it: treat a value you do not recognize as your
    /// default branch.
    // The schema keeps a plain string: a closed list there would make this
    // release's schema reject a reason a later release adds.
    #[schemars(with = "String")]
    pub reason: Reason,
    /// Structured exit code per the sysexits-inspired matrix in
    /// `xurl::error`.
    pub exit_code: i32,
    /// Human-readable message. Omitted entirely (not `null`) when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The input rule a `validation` refusal broke, as the validator's
    /// kebab-case code (`invalid-username`, `no-list-changes`, ...);
    /// `message` is the sentence for it. Present under `validation`, and on
    /// a dry run that refuses its input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    /// The offending input, as it was given, when the broken `rule` names
    /// one: an app name, a redirect URI, a header's position among the `-H`
    /// headers (counted from 1), the seconds a checkpoint runs ahead. Always
    /// a string; a number goes in as its decimal digits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// What the caller should do next, when a recovery step exists.
    ///
    /// `action` is closed: `register-app`, `sign-in`, `select-app`,
    /// `inspect-store`, `enroll-app`, `show-help`, `resume-wait`,
    /// `wait-and-retry`, `retry`, `fix-input`, `report-issue`, `confirm`,
    /// `run-command`. A newer release can add an action, so treat one you
    /// do not recognize as your default branch. A step carries either a
    /// `command`, runnable verbatim by a non-TTY caller, or a `template`
    /// the caller finishes or decides on first, never both: a template
    /// holds angle-bracket placeholders only the caller can fill; under
    /// `confirm`, the invocation that destroys once it runs; or, after a
    /// 401 on the status read after `xr media upload`, the resumed wait to
    /// run once the credential works. One whose recovery is not an `xr`
    /// invocation carries neither, and `docs` when a page covers it:
    /// `enroll-app`, `wait-and-retry`, `retry`, `fix-input`,
    /// `report-issue`, and `inspect-store` when a store command is what
    /// failed. After `xr media upload` finished, a `wait-and-retry` on the
    /// status read carries the command that resumes the wait, or a template
    /// when the media command was given `-H`. Three `fix-input` steps carry
    /// an invocation:
    /// `listen` again on a free port, under `address-in-use`, and two under
    /// `invalid-args`: the invocation with `-u`, `--auth`, or `-t` moved
    /// after the command word it came before, and the `xr schema`
    /// invocation that works for an old name or a scope word. Six carry a
    /// template: the `xr schema --path` invocation with the segment that
    /// named nothing left as `<property>`, under `invalid-args`, three under
    /// `auth-required`: the command that stores the secret `webhooks
    /// listen` lacks, `auth oauth2 --step 2` again with its redirect URL
    /// left as `<redirect-url>`, and the resumed wait after a 401 on the
    /// status read after `xr media upload`, to run once the credential
    /// works; and `resume-wait` and that `wait-and-retry` when the media
    /// command was given `-H`, with each header's value left as `<value>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step: Option<NextStep>,
    /// What the library warned about before the run failed, each distinct
    /// warning once. Omitted when none fired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Warning>>,
    /// The offending value, echoed verbatim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// The nearest real command, when one is within the threshold, or the
    /// command that replaced a retired word.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,

    /// Endpoint template the request targeted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// Endpoint path with path parameters substituted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered_url: Option<String>,
    /// HTTP method the request would have used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Auth scheme the caller asked for; `null` when auto-detect ran.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub requested: Option<Value>,
    /// Auth schemes the endpoint accepts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<Vec<String>>,
    /// Auth schemes available for the active app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_in_app: Option<Vec<String>>,
    /// The active app name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    /// Other apps holding credentials, when the active one holds none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_apps_with_creds: Option<Vec<String>>,

    /// Media id whose processing a wait gave up on, under
    /// `processing-timeout`, where the upload is intact; or the one X
    /// refused after processing it, under `invalid-request`. `media upload`
    /// carries it on any error that ends its wait, since the upload finished
    /// and stdout holds no result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_id: Option<String>,

    /// The handle, without its `@`, a lookup found no account for, under
    /// `not-found`: the first such handle when the lookup named several.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,

    /// Seconds until the rate limit resets, zero once it has; present on a
    /// `rate-limited` error whose own response named the reset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_secs: Option<u64>,
    /// The same reset as an RFC 3339 UTC timestamp, for a caller that
    /// schedules the retry instead of sleeping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_at: Option<String>,

    /// Post id a destructive verb targeted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_id: Option<String>,
    /// App name a destructive verb targeted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether `auth clear` targeted every credential.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    /// Whether `auth clear` targeted the `OAuth1` token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth1: Option<bool>,
    /// The `OAuth2` user `auth clear` targeted; `null` when unset.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub oauth2_username: Option<Value>,
    /// Whether `auth clear` targeted the bearer token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer: Option<bool>,

    /// Schema name a `validate` invocation named.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// Dot path an `xr schema --path` invocation named.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Property names at the last node an `xr schema --path` resolved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_properties: Option<Vec<String>>,
    /// Schema names `validate` accepts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_schemas: Option<Vec<String>>,
    /// Whether the document validated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid: Option<bool>,
    /// Skill verb the envelope reports on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Agent hosts `skill install` accepts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_hosts: Option<Vec<String>>,
    /// One report per host on a skill verb's `--all` run. The top-level
    /// `reason` and `next_step` speak for the host with the worst exit code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installations: Option<Vec<Value>>,
    /// Agent host a skill verb targeted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    /// Directory a skill verb would install into.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_dir: Option<String>,
    /// The `git clone` a skill verb would run, its hardening flags omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_preview: Option<String>,
    /// What a skill verb found at `install_dir` before it cloned: `absent`,
    /// `empty-dir`, `non-empty-dir`, or `file`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_status: Option<String>,
    /// A copy of the skill bundle at a location the host still reads but `xr`
    /// does not install to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_install_dir: Option<String>,
    /// Directory `skill update` refused to replace, because it holds
    /// something other than the bundle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refused_dir: Option<String>,
    /// Whether a dry-run skill verb would have succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub would_succeed: Option<bool>,

    /// The stack at a panic, as Rust prints it; present on the `internal`
    /// error a panic answers with when `RUST_BACKTRACE` asks for one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backtrace: Option<String>,
}

/// Reads a key that is present as `Some`, `null` included. Serde's own
/// reading of an `Option` turns a present `null` into `None`, which would
/// drop a key the envelope documents `null` as a value of.
fn present<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Value::deserialize(deserializer).map(Some)
}

impl ErrorBody {
    /// Serializes this body into the error envelope agents receive.
    ///
    /// The one conversion every emitter uses, so the `status` tag and the
    /// declared field set cannot drift between them.
    #[must_use]
    pub fn into_value(self) -> Value {
        serde_json::to_value(Envelope::Error(Box::new(self)))
            .unwrap_or_else(|_| serde_json::json!({"status": "error"}))
    }
}

/// Returns the JSON Schema (Draft 2020-12) for the [`Envelope`] type.
///
/// The runtime emitter; the committed `schema/output.schema.json` is the
/// pinned snapshot.
#[must_use]
pub fn envelope_schema() -> Value {
    let schema = schemars::schema_for!(Envelope);
    serde_json::to_value(schema).expect("Envelope schema serializes")
}
