//! Build-time coverage check for the auth matrix (plan U8 / brainstorm R25).
//!
//! Iterates `xurl::api::auth_matrix::SHORTCUT_TEMPLATES` and asserts each
//! `(method, path)` tuple resolves to a non-empty matrix entry. Catches three
//! failure modes early:
//!
//! 1. A shortcut typo in the runtime `SHORTCUT_TEMPLATES` const that drifted
//!    from the build-time allowlist in `build.rs`.
//! 2. Spec drift: X removed an endpoint a shortcut targets. The codegen
//!    silently drops it; this test surfaces the gap before release.
//! 3. A shortcut whose spec template never carried a `security:` field.
//!    R19 covers this at runtime as permissive, but the coverage check
//!    treats it as a release-time anomaly.
//!
//! Failure messages name the offending `(method, path)` so the implementer
//! sees the mismatch without grepping the spec.

use xurl::api::auth_matrix::{SHORTCUT_TEMPLATES, supported_auth};

#[test]
fn every_shortcut_template_resolves_in_the_matrix() {
    let mut missing: Vec<(&'static str, &'static str)> = Vec::new();

    for (method, path) in SHORTCUT_TEMPLATES {
        match supported_auth(method, path) {
            Some(schemes) if !schemes.is_empty() => {}
            Some(_) => {
                missing.push((method, path));
            }
            None => {
                missing.push((method, path));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "Shortcut templates missing from the auth matrix:\n{}\n\
         The runtime SHORTCUT_TEMPLATES const drifted from the build-time \
         allowlist in build.rs, OR the spec was updated and a previously-\
         documented endpoint was dropped. Re-run scripts/refresh-x-openapi.sh \
         and reconcile.",
        missing
            .iter()
            .map(|(m, p)| format!("  - {m} {p}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn shortcut_templates_is_non_empty() {
    // Defensive anchor: prevents a future refactor from silently emptying
    // the allowlist and the coverage check then passing vacuously.
    assert!(
        !SHORTCUT_TEMPLATES.is_empty(),
        "SHORTCUT_TEMPLATES is empty — the coverage check would pass vacuously"
    );
}

#[test]
fn methods_are_uppercase_and_standard() {
    // Lookup uppercases internally, but a SHORTCUT_TEMPLATES entry with a
    // non-standard method would silently lookup-fail in production. Lock
    // the surface to the five methods the matrix codegen translates.
    const STANDARD: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH"];
    for (method, path) in SHORTCUT_TEMPLATES {
        assert!(
            STANDARD.contains(method),
            "SHORTCUT_TEMPLATES has a non-standard method {method:?} for {path}"
        );
    }
}

#[test]
fn paths_start_with_slash() {
    // The matrix key is `format!("{method}\0{path}")`. A bare path without
    // leading slash would silently miss every lookup at runtime because the
    // codegen does emit the leading slash from the spec.
    for (_, path) in SHORTCUT_TEMPLATES {
        assert!(
            path.starts_with('/'),
            "SHORTCUT_TEMPLATES path {path:?} must start with `/` to match the matrix key shape"
        );
    }
}
