/// Re-exports from the library's error module for binary-internal use.
///
/// The canonical definitions live in `xurl::error` so library consumers
/// (e.g., bird) can access them directly. This module exists only to
/// keep binary-internal imports short.
#[allow(unused_imports)]
pub use crate::error::{
    EXIT_AUTH_REQUIRED, EXIT_GENERAL_ERROR, EXIT_NETWORK_ERROR, EXIT_NOT_FOUND, EXIT_RATE_LIMITED,
    EXIT_SUCCESS,
};
