//! Coverage check for the auth matrix.
//!
//! Iterates `xdk::api::auth_matrix::SHORTCUT_TEMPLATES` and asserts each
//! `(method, path)` tuple resolves to a non-empty matrix entry. Catches two
//! failure modes early:
//!
//! 1. Spec drift: X removed an endpoint a shortcut targets. The codegen
//!    silently drops it; this test surfaces the gap before release.
//! 2. A shortcut whose spec template never carried a `security:` field.
//!    The runtime treats that as permissive, but the coverage check treats
//!    it as a release-time anomaly.
//!
//! Failure messages name the offending `(method, path)` so the implementer
//! sees the mismatch without grepping the spec.

use xdk::api::auth_matrix::{
    AUTH_MATRIX, SHORTCUT_TEMPLATES, WireScheme, supported_auth, supported_auth_for_request,
};

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
         Cause: the vendored spec no longer lists a `security:` block for the \
         endpoint, either because a refresh dropped the operation or because the \
         spec declares it without one.\n\
         Fix: re-run scripts/refresh-x-openapi.sh and compare the endpoint's entry \
         in crates/xdk/vendor/x-api-openapi.json against its SHORTCUT_TEMPLATES row \
         in crates/xdk/build.rs.",
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
fn supported_auth_returns_none_for_unknown_shortcut() {
    // Negative-path proof for the coverage check: an unknown (method, path)
    // pair returns None, which the coverage check above would flag if any
    // SHORTCUT_TEMPLATES entry resolved this way. Locks the assertion's
    // protective intent — if `supported_auth` ever began returning Some(_)
    // for unknown paths, the coverage check would pass vacuously and the
    // build-time guarantee would be silently broken.
    assert!(supported_auth("GET", "/2/never/heard/of").is_none());
    // A real spec path with no shortcut mapping: `endpoint-decisions.json`
    // keeps the full-archive search raw-only for good, so this stays a
    // genuine negative.
    assert!(supported_auth("GET", "/2/tweets/search/all").is_none());
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

/// A request to a path that carries ids is held to its template's schemes.
#[test]
fn a_concrete_path_resolves_to_the_template_it_fits() {
    let key = |method: &str, path: &str| {
        let listed = supported_auth(method, path).expect("the template is in the matrix");
        WireScheme::ALL_BY_PREFERENCE
            .into_iter()
            .filter(|scheme| listed.iter().any(|entry| entry.wire() == *scheme))
            .collect::<Vec<_>>()
    };
    let mut templated = 0;
    for (entry, _) in AUTH_MATRIX.entries() {
        let (method, template) = entry.split_once('\0').expect("a packed key");
        if !template.contains('{') {
            continue;
        }
        templated += 1;
        let concrete: String = template
            .split('/')
            .map(|segment| {
                if segment.starts_with('{') {
                    "123"
                } else {
                    segment
                }
            })
            .collect::<Vec<_>>()
            .join("/");
        assert_eq!(
            supported_auth_for_request(method, &concrete),
            Some(key(method, template)),
            "{method} {concrete} should take the schemes of {template}"
        );
    }
    assert!(
        templated > 50,
        "the matrix holds {templated} templated entries"
    );
}

/// A literal path that also fits a template keeps its own entry, so
/// `/2/users/me/following` is not read as `/2/users/{id}/following`.
#[test]
fn a_literal_path_keeps_its_own_entry_over_a_template() {
    let wire = |listed: &[xdk::api::auth_matrix::AuthScheme]| {
        WireScheme::ALL_BY_PREFERENCE
            .into_iter()
            .filter(|scheme| listed.iter().any(|entry| entry.wire() == *scheme))
            .collect::<Vec<_>>()
    };
    let mut differing = 0;
    for (entry, listed) in AUTH_MATRIX.entries() {
        let (method, path) = entry.split_once('\0').expect("a packed key");
        if path.contains('{') {
            continue;
        }
        assert_eq!(
            supported_auth_for_request(method, path),
            Some(wire(listed)),
            "{method} {path} should keep its own entry"
        );
        let shadowed = AUTH_MATRIX.entries().any(|(other, other_listed)| {
            let (other_method, template) = other.split_once('\0').expect("a packed key");
            let (a, b): (Vec<_>, Vec<_>) =
                (template.split('/').collect(), path.split('/').collect());
            other_method == method
                && template.contains('{')
                && a.len() == b.len()
                && a.iter()
                    .zip(&b)
                    .all(|(t, p)| (t.starts_with('{') && !p.is_empty()) || t == p)
                && wire(other_listed) != wire(listed)
        });
        differing += usize::from(shadowed);
    }
    assert!(
        differing > 0,
        "no literal path shadows a template with other schemes, so this proves nothing"
    );
}

#[test]
fn a_request_path_is_matched_without_its_case_or_trailing_slash_in_the_method_or_path() {
    let plain = supported_auth_for_request("DELETE", "/2/tweets/123");
    assert!(plain.is_some());
    assert_eq!(supported_auth_for_request("delete", "/2/tweets/123"), plain);
    assert_eq!(
        supported_auth_for_request("DELETE", "/2/tweets/123/"),
        plain
    );
    assert_eq!(
        supported_auth_for_request("GET", "/2/users/me/"),
        supported_auth_for_request("GET", "/2/users/me")
    );
}

#[test]
fn a_request_path_no_entry_fits_is_unlisted() {
    assert_eq!(supported_auth_for_request("GET", "/2/never/heard/of"), None);
    assert_eq!(
        supported_auth_for_request("GET", "/2/tweets/search/all"),
        None
    );
    assert_eq!(
        supported_auth_for_request("GET", "/2/users//following"),
        None
    );
    assert_eq!(supported_auth_for_request("TRACE", "/2/tweets/123"), None);
}
