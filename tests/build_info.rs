//! Smoke tests for the build / provenance consts exposed by `src/lib.rs`.
//! Verifies that the build script reads the vendored spec metadata
//! sidecar correctly and that the public consts resolve to the expected
//! shape.
//!
//! Spec metadata consts (`API_SPEC_VERSION`, `API_SPEC_SHA256`,
//! `API_SPEC_DATE`) are always populated because they come from a
//! checked-in sidecar at `vendor/spec-metadata.json` that ships with the
//! crate package. `CRATE_GIT_SHA` is `Option` and depends on the build
//! happening inside a git checkout; the test for it assumes a local-dev
//! or CI build context.

use xurl::{API_SPEC_DATE, API_SPEC_SHA256, API_SPEC_VERSION, CRATE_GIT_SHA, CRATE_VERSION};

#[test]
fn crate_version_matches_cargo_pkg_version() {
    assert_eq!(CRATE_VERSION, env!("CARGO_PKG_VERSION"));
}

#[test]
fn api_spec_version_has_dotted_shape() {
    assert!(!API_SPEC_VERSION.is_empty());
    // X spec versions follow `<major>.<minor>` (e.g., "2.165").
    let parts: Vec<&str> = API_SPEC_VERSION.split('.').collect();
    assert!(
        parts.len() >= 2,
        "API_SPEC_VERSION should be major.minor; got {API_SPEC_VERSION:?}"
    );
    for part in &parts {
        assert!(
            part.chars().all(|c| c.is_ascii_digit()),
            "API_SPEC_VERSION component {part:?} should be all digits"
        );
    }
}

#[test]
fn api_spec_sha256_is_64_hex_chars() {
    assert_eq!(
        API_SPEC_SHA256.len(),
        64,
        "SHA-256 should be 64 hex characters; got {API_SPEC_SHA256:?} ({} chars)",
        API_SPEC_SHA256.len()
    );
    assert!(
        API_SPEC_SHA256.chars().all(|c| c.is_ascii_hexdigit()),
        "API_SPEC_SHA256 contains a non-hex character: {API_SPEC_SHA256:?}"
    );
}

#[test]
fn api_spec_date_is_iso_short_format() {
    assert_eq!(
        API_SPEC_DATE.len(),
        10,
        "expected YYYY-MM-DD; got {API_SPEC_DATE:?}"
    );
    assert_eq!(API_SPEC_DATE.as_bytes()[4], b'-');
    assert_eq!(API_SPEC_DATE.as_bytes()[7], b'-');
    for (i, c) in API_SPEC_DATE.chars().enumerate() {
        if i == 4 || i == 7 {
            continue;
        }
        assert!(
            c.is_ascii_digit(),
            "non-digit at position {i} in date {API_SPEC_DATE:?}"
        );
    }
}

#[test]
fn crate_git_sha_is_set_in_git_checkout() {
    let sha =
        CRATE_GIT_SHA.expect("CRATE_GIT_SHA should be Some when building from a git checkout");
    assert_eq!(
        sha.len(),
        40,
        "git SHA-1 should be 40 hex characters; got {sha:?} ({} chars)",
        sha.len()
    );
    assert!(
        sha.chars().all(|c| c.is_ascii_hexdigit()),
        "CRATE_GIT_SHA contains a non-hex character: {sha:?}"
    );
}
