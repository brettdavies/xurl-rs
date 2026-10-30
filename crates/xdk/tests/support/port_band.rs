//! The ports a test names before the `OAuth2` callback listener binds them,
//! and the one failure that lets such a test pick again. The tests of both
//! crates include this file with `#[path]`, so the rule has one copy.

/// Ports below every ephemeral range: Linux hands out 32768-60999 for a
/// port-0 bind, and macOS and Windows 49152-65535, so no listener that binds
/// port 0 lands here.
pub const BIND_TEST_PORTS: std::ops::RangeInclusive<u16> = 20000..=32767;

/// A port from [`BIND_TEST_PORTS`].
pub fn band_port() -> u16 {
    use std::hash::{BuildHasher, RandomState};
    let span = u64::from(BIND_TEST_PORTS.end() - BIND_TEST_PORTS.start()) + 1;
    let offset = RandomState::new().hash_one(0u8) % span;
    BIND_TEST_PORTS.start() + u16::try_from(offset).expect("within the band")
}

/// Whether `error` is the callback listener failing to bind `port` itself,
/// the one failure a port named before the bind can meet that is not the
/// listener's fault. The listener reports it as a message, not a variant, so
/// this reads the wording `xdk::auth::callback` gives it.
pub fn failed_to_bind(error: &xdk::Error, port: u16) -> bool {
    let text = error.to_string();
    text.contains("could not bind callback listener") && text.contains(&format!(":{port}:"))
}
