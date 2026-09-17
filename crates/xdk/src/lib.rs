//! xdk — async Rust client for the X (Twitter) API.
//!
//! Downstream Rust consumers build requests via [`api::Client`],
//! pattern-match on [`Error`], and persist auth state in
//! [`store::TokenStore`]. The `xr` command-line client is a separate crate,
//! `xurl-rs`, built on this one.
//!
//! Every published module is one an embedder has a reason to call:
//!
//! - [`api`]: the client and its builder, one [`api::Call`] per shortcut,
//!   the typed responses, the raw-request path for endpoints without a
//!   shortcut, and the last rate-limit window a response reported.
//! - [`auth`]: credentials held in code, the refresh hook that receives a
//!   rotated token pair, the store-backed [`auth::Auth`], and the OAuth2
//!   sign-in flows.
//! - [`config`]: base URL, timeouts, and the environment overrides a client
//!   built from the environment reads.
//! - [`error`]: [`Error`], [`Result`], the machine-readable
//!   [`error::NextAction`], and the exit codes a CLI built on this crate maps
//!   them to.
//! - [`store`]: [`store::TokenStore`], the on-disk credential store the CLI
//!   shares, and the reference implementation of the refresh hook.
//!
//! Items marked `#[doc(hidden)]` are seams the `xr` binary reaches across the
//! crate boundary; they stay callable but are not part of this surface.
//!
//! # Cargo features
//!
//! - `rustls` (default): TLS through [rustls](https://docs.rs/rustls) with
//!   the platform's certificate verifier; no system TLS library is linked.
//! - `native-tls`: TLS through the operating system's library (OpenSSL on
//!   Linux, Secure Transport on macOS, SChannel on Windows) via
//!   [native-tls](https://docs.rs/native-tls). To use it alone, turn the
//!   default off: `xdk-rs = { version = "0.1", default-features = false,
//!   features = ["native-tls"] }`. With both backends enabled, reqwest
//!   picks `native-tls`.
//! - `testing`: reserved for an in-process mock server and fixtures; it
//!   enables nothing yet.
//!
//! A build with neither TLS feature fails at compile time with a message
//! naming both, rather than at the first `https` request.
//!
//! Four authentication paths are supported, selected per request from the
//! token store and environment:
//!
//! - **OAuth2 PKCE** (browser-driven or headless copy-paste) — every v2
//!   user-scoped endpoint.
//! - **OAuth1 HMAC-SHA1** — legacy v1.1 endpoints and some v2 write paths.
//! - **Bearer (app-only)** — v2 read-only endpoints and search; set via
//!   `XURL_BEARER_TOKEN`.

// `Error`'s largest variant (`AuthMethodMismatch`) carries multiple
// `String` and `Vec<String>` fields so agents can pattern-match on the
// envelope structure. Boxing the variant would change the public
// construction surface and break consumer code; allow the lint instead.
#![allow(clippy::result_large_err)]
#![deny(missing_docs)]

// reqwest compiles without a TLS backend and only fails at the first https
// request, with an error that never mentions TLS; failing the build names
// the fix instead.
#[cfg(not(any(feature = "rustls", feature = "native-tls")))]
compile_error!(
    "xdk-rs needs a TLS backend: enable the `rustls` feature (on by default) or `native-tls`, \
     for example `xdk-rs = { version = \"0.1\", default-features = false, features = [\"native-tls\"] }`"
);

pub mod api;
pub mod auth;
pub mod config;
pub mod error;
pub mod store;

pub use error::{Error, Result};

/// Fails the build when `$t` stops being shareable across tasks and threads.
///
/// Invoked beside each type it guards, so the failure surfaces there rather
/// than at a distant call site. Exported so the `xr` crate guards its own
/// types with the same assertion; it is not embedder API.
#[doc(hidden)]
#[macro_export]
macro_rules! assert_send_sync {
    ($t:ty) => {
        const _: fn() = || {
            fn assert<T: Send + Sync>() {}
            assert::<$t>();
        };
    };
}

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
pub const CRATE_GIT_SHA: Option<&'static str> = option_env!("XDK_CRATE_GIT_SHA");

/// X API OpenAPI spec version (`info.version` field) of the vendored
/// `vendor/x-api-openapi.json` at build time, read from the
/// `vendor/spec-metadata.json` sidecar. X bumps spec content without
/// bumping this field; pair with [`API_SPEC_SHA256`] when drift detection
/// matters.
pub const API_SPEC_VERSION: &str = env!("XDK_API_SPEC_VERSION");

/// SHA-256 of `vendor/x-api-openapi.json` at the time of its most recent
/// vendoring, read from `vendor/spec-metadata.json`. Drift-sensitive
/// identifier — changes every time X changes spec content, regardless of
/// [`API_SPEC_VERSION`].
pub const API_SPEC_SHA256: &str = env!("XDK_API_SPEC_SHA256");

/// Date (UTC, `YYYY-MM-DD`) the vendored OpenAPI spec was last refreshed
/// by `scripts/refresh-x-openapi.sh`, read from `vendor/spec-metadata.json`.
pub const API_SPEC_DATE: &str = env!("XDK_API_SPEC_DATE");
