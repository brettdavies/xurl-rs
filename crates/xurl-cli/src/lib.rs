//! `xr`, the command-line client for the X (Twitter) API, built on [`xdk`].
//!
//! This crate ships the `xr` binary. The library target exists so the
//! binary's own integration tests can drive the dispatcher in-process; it is
//! not an embedder API. Rust programs use the `xdk-rs` crate.

// `xdk::Error`'s largest variant carries several `String` and `Vec<String>`
// fields, and every handler returns it through `Failure`; boxing would change
// the library's public construction surface for a size the CLI never pays for.
#![allow(clippy::result_large_err)]
#![deny(missing_docs)]

#[doc(hidden)]
pub mod cli;
