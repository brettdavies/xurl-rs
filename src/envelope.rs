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
use serde::Serialize;
use serde_json::Value;

/// The three envelope variants — one per `status` discriminator value.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
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
    /// Error envelope. `reason` is mandatory — a typed kebab-case identifier
    /// from the closed set: `auth-required`, `rate-limited`, `not-found`,
    /// `network-error`, `invalid-args`, `invalid-method`, `validation`,
    /// `serialization`, `io`, `token-store`.
    Error {
        /// Typed kebab-case kind. Closed set; agents pattern-match on this.
        reason: String,
        /// Structured exit code per the sysexits-inspired matrix in
        /// `xurl::error`.
        exit_code: i32,
        /// Optional human-readable message. Omitted entirely (not `null`)
        /// when absent so consumers feature-detect by key presence.
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
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
