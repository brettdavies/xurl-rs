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

pub mod api;
pub mod auth;
pub mod cli;
pub mod config;
pub mod envelope;
pub mod error;
pub mod output;
pub mod skill_install;
pub mod store;
