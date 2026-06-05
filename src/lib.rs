//! xurl — fast Rust client for the X (Twitter) API.
//!
//! Ships two consumable surfaces:
//!
//! - The `xr` binary, a high-level CLI for the X API.
//! - The `xurl` library exposed via the modules below. Downstream Rust
//!   consumers build requests via [`api::ApiClient`], drive output through
//!   [`output::OutputConfig`], pattern-match on [`error::XurlError`], and
//!   persist auth state in [`store::TokenStore`].
//!
//! Four authentication paths are supported, selected per request from the
//! token store and environment:
//!
//! - **OAuth2 PKCE** (browser-driven or headless copy-paste) — every v2
//!   user-scoped endpoint.
//! - **OAuth1 HMAC-SHA1** — legacy v1.1 endpoints and some v2 write paths.
//! - **Bearer (app-only)** — v2 read-only endpoints and search; set via
//!   `XURL_BEARER_TOKEN`.

// `XurlError`'s largest variant (`AuthMethodMismatch`) carries multiple
// `String` and `Vec<String>` fields so agents can pattern-match on the
// envelope structure. Boxing the variant would change the public
// construction surface and break consumer code; allow the lint instead.
#![allow(clippy::result_large_err)]
#![deny(missing_docs)]

pub mod api;
pub mod auth;
pub mod cli;
pub mod config;
pub mod envelope;
pub mod error;
pub mod output;
pub mod skill_install;
pub mod store;

// ── Compile-time build and provenance metadata ──────────────────────────
//
// API spec consts are read from a checked-in sidecar at
// `vendor/spec-metadata.json` rather than derived from git context, so
// the values always describe the actual bytes that ship — uncommitted
// local refreshes, crates.io tarball installs, and downstream git-pin
// consumers all see identity that matches the bundled spec.
//
// `CRATE_GIT_SHA` is the only field that depends on git context; it
// resolves to `None` on crates.io tarball builds where `.git` is not
// present and `Some` on local development and CI builds.

/// Version of this crate, from `[package].version` in `Cargo.toml`.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Git commit SHA the crate was built from. `None` when the build had
/// no git context (e.g., `cargo install` from a crates.io tarball,
/// where `.git` is not present).
pub const CRATE_GIT_SHA: Option<&'static str> = option_env!("XURL_CRATE_GIT_SHA");

/// X API OpenAPI spec version (`info.version` field) of the vendored
/// `vendor/x-api-openapi.json` at build time, read from the
/// `vendor/spec-metadata.json` sidecar. X bumps spec content without
/// bumping this field; pair with [`API_SPEC_SHA256`] when drift detection
/// matters.
pub const API_SPEC_VERSION: &str = env!("XURL_API_SPEC_VERSION");

/// SHA-256 of `vendor/x-api-openapi.json` at the time of its most recent
/// vendoring, read from `vendor/spec-metadata.json`. Drift-sensitive
/// identifier — changes every time X changes spec content, regardless of
/// [`API_SPEC_VERSION`].
pub const API_SPEC_SHA256: &str = env!("XURL_API_SPEC_SHA256");

/// Date (UTC, `YYYY-MM-DD`) the vendored OpenAPI spec was last refreshed
/// by `scripts/refresh-x-openapi.sh`, read from `vendor/spec-metadata.json`.
pub const API_SPEC_DATE: &str = env!("XURL_API_SPEC_DATE");
