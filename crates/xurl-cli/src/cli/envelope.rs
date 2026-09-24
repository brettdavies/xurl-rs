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
//! cargo run -p xurl-rs --bin xr -- schema envelope --output json > schema/output.schema.json
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
        /// Verb-specific payload fields. Encoded as additional top-level
        /// keys via `#[serde(flatten)]` at the call site.
        #[serde(flatten)]
        payload: Value,
    },
    /// Dry-run envelope. `would_succeed` and `exit_code` are mandatory;
    /// additional context (command name, body preview, etc.) lives in
    /// `payload`.
    DryRun {
        /// True iff every precondition checked clean.
        would_succeed: bool,
        /// Exit code the verb would have returned on actual execution.
        exit_code: i32,
        /// Verb-specific context payload, flattened at the top level.
        #[serde(flatten)]
        payload: Value,
    },
    /// Error envelope. Every field the runtime emits is declared on
    /// [`ErrorBody`], so an undeclared key cannot reach a caller. Boxed
    /// because that body is much larger than the other two variants.
    Error(Box<ErrorBody>),
}

/// Every field an error envelope can carry, beside the `status` tag.
///
/// The emitter in [`crate::cli::output`] constructs this value and serializes it,
/// which is what lets the generated schema describe exactly what agents see.
/// Optional fields are skipped when absent, so a consumer feature-detects by
/// key presence. `reason` is a typed kebab-case identifier from the closed
/// set documented in this module.
///
/// The declaration is the whole error surface, not one emitter's: the
/// verb-local fields the `validate` and `skill` commands carry are declared
/// here too, because the generated schema closes the object and a key it
/// does not name would make that schema wrong about what callers receive.
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
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
    /// `io`, `token-store`, `internal`, `confirmation-required`, `no-tty`,
    /// `unsupported-pagination`, and the verb-local `invalid-json`,
    /// `unknown-schema`, `validation-failed`, `missing-host`, `home-not-set`,
    /// `destination-not-empty`, `destination-is-file`, `git-not-found`,
    /// `git-clone-failed`, `not-installed`, and `remove-failed`. The set is
    /// closed, and a newer release can add to it: treat a value you do not
    /// recognize as your default branch.
    pub reason: String,
    /// Structured exit code per the sysexits-inspired matrix in
    /// `xurl::error`.
    pub exit_code: i32,
    /// Human-readable message. Omitted entirely (not `null`) when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// What the caller should do next, when a recovery step exists.
    ///
    /// `action` is closed: `register-app`, `sign-in`, `select-app`,
    /// `inspect-store`, `enroll-app`, `show-help`. A newer release can add an
    /// action, so treat one you do not recognize as your default branch. A
    /// step carries either a `command`, runnable verbatim by a non-TTY caller,
    /// or a `template` whose angle-bracket placeholders only the caller can
    /// fill, never both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step: Option<NextStep>,
    /// The offending value, echoed verbatim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// The nearest real command, when one is within the threshold.
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2_username: Option<Value>,
    /// Whether `auth clear` targeted the bearer token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer: Option<bool>,

    /// Schema name a `validate` invocation named.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
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
    /// Agent host a skill verb targeted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    /// Directory a skill verb would install into.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_dir: Option<String>,
    /// Whether a dry-run skill verb would have succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub would_succeed: Option<bool>,
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
