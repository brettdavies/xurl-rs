//! The process environment, read once per run by the binary.
//!
//! The library's [`EnvOverrides::from_env`] reads the variables an X API
//! client needs. The four the CLI alone consumes are read here, so no
//! library code path touches `XURL_OUTPUT`, `HOME`, `XURL_TOKEN_STORE`, or
//! `NO_COLOR`.

use crate::config::EnvOverrides;

/// Every variable `xr` reads, as data.
#[must_use]
pub fn from_process() -> EnvOverrides {
    let mut overrides = EnvOverrides::from_env();
    overrides.output = std::env::var("XURL_OUTPUT").ok();
    overrides.home = std::env::var("HOME").ok();
    overrides.token_store = std::env::var("XURL_TOKEN_STORE").ok();
    overrides.no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    overrides
}
