//! `xr`, the command-line client for the X (Twitter) API, built on [`xdk`].
//!
//! This crate ships the `xr` binary. The library target exists so the
//! binary's own integration tests can drive the dispatcher in-process; it is
//! not an embedder API. Rust programs use the `xdk-rs` crate.

#![deny(missing_docs)]

#[doc(hidden)]
pub mod cli;
