/// Structured exit codes for machine-readable error handling.
///
/// Following UNIX conventions and agent-native design:
/// - 0: success
/// - 1: general error
/// - 2: auth required (agent should run `xurl auth login`)
/// - 3: rate limited (agent should retry with backoff)
/// - 4: not found (resource doesn't exist)
/// - 5: network error (connectivity issue)

#[allow(dead_code)]
pub const EXIT_SUCCESS: i32 = 0;
#[allow(dead_code)]
pub const EXIT_GENERAL_ERROR: i32 = 1;
#[allow(dead_code)]
pub const EXIT_AUTH_REQUIRED: i32 = 2;
#[allow(dead_code)]
pub const EXIT_RATE_LIMITED: i32 = 3;
#[allow(dead_code)]
pub const EXIT_NOT_FOUND: i32 = 4;
#[allow(dead_code)]
pub const EXIT_NETWORK_ERROR: i32 = 5;
