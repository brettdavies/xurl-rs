/// Endpoint -> accepted auth methods, generated from the vendored X OpenAPI
/// spec at build time.
///
/// Source of truth: `vendor/x-api-openapi.json`. Codegen lives in
/// `build.rs::emit_auth_matrix`; the same `SHORTCUT_TEMPLATES` allowlist is
/// duplicated here for runtime callers (notably U8's coverage check). Both
/// copies must list the same `(method, path)` pairs — the build panics if
/// a spec lookup misses, which surfaces drift between the two.
///
/// Lookup is a direct hash on the packed key `"METHOD\0/path/template"`;
/// no path-template precedence resolution. Unknown `(method, path)` pairs
/// return `None`, which the validator treats as permissive per R19.
use std::fmt::Write as _;

use crate::api::request::RequestTarget;
use crate::error::{Result, XurlError};

/// Auth schemes an X API endpoint accepts, as declared by its OpenAPI
/// `security:` list. Scope lists are captured verbatim for v2 scope
/// checking — v1 ignores them (KTD-4, KTD-P3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthScheme {
    /// App-only Bearer token (`type: http, scheme: bearer`).
    Bearer,
    /// OAuth 1.0a user context (`type: http, scheme: OAuth`).
    OAuth1User,
    /// OAuth 2.0 PKCE user context. Empty slice means the endpoint accepts
    /// OAuth2 with no scope requirement.
    OAuth2User(&'static [&'static str]),
}

// phf_codegen emits `&STATIC_NAME` references inside the generated map,
// which clippy flags as `needless_borrow`. Suppress in this module — the
// generated source is not edited by hand.
#[allow(clippy::needless_borrow)]
mod generated {
    use super::AuthScheme;
    include!(concat!(env!("OUT_DIR"), "/auth_matrix.rs"));
}

pub use generated::AUTH_MATRIX;

/// `(METHOD, spec path)` pairs the shortcut + media layer targets, mirroring
/// the build-time allowlist verbatim. Consumed by U8's coverage check, which
/// asserts every shortcut site lands in the matrix.
pub const SHORTCUT_TEMPLATES: &[(&str, &str)] = &[
    ("POST", "/2/tweets"),
    ("GET", "/2/tweets/{id}"),
    ("DELETE", "/2/tweets/{id}"),
    ("GET", "/2/tweets/search/recent"),
    ("GET", "/2/users/me"),
    ("GET", "/2/users/by/username/{username}"),
    ("GET", "/2/users/{id}/timelines/reverse_chronological"),
    ("GET", "/2/users/{id}/mentions"),
    ("GET", "/2/users/{id}/followers"),
    ("GET", "/2/users/{id}/liked_tweets"),
    ("GET", "/2/users/{id}/blocking"),
    ("POST", "/2/users/{id}/likes"),
    ("DELETE", "/2/users/{id}/likes/{tweet_id}"),
    ("POST", "/2/users/{id}/retweets"),
    ("DELETE", "/2/users/{id}/retweets/{source_tweet_id}"),
    ("GET", "/2/users/{id}/bookmarks"),
    ("POST", "/2/users/{id}/bookmarks"),
    ("DELETE", "/2/users/{id}/bookmarks/{tweet_id}"),
    ("GET", "/2/users/{id}/following"),
    ("POST", "/2/users/{id}/following"),
    (
        "DELETE",
        "/2/users/{source_user_id}/following/{target_user_id}",
    ),
    ("GET", "/2/users/{id}/muting"),
    ("POST", "/2/users/{id}/muting"),
    (
        "DELETE",
        "/2/users/{source_user_id}/muting/{target_user_id}",
    ),
    ("POST", "/2/dm_conversations/with/{participant_id}/messages"),
    ("GET", "/2/dm_events"),
    ("GET", "/2/usage/tweets"),
    ("POST", "/2/media/upload"),
    ("GET", "/2/media/upload"),
    ("POST", "/2/media/upload/initialize"),
    ("POST", "/2/media/upload/{id}/append"),
    ("POST", "/2/media/upload/{id}/finalize"),
];

/// Maximum HTTP method length we accept. Standard methods top out at
/// 7 characters (`OPTIONS`); the matrix only emits five of them. Anything
/// longer can't match a real entry, so we short-circuit lookup.
const MAX_METHOD_LEN: usize = 8;

/// Look up the auth schemes an endpoint accepts. Returns `None` when the
/// `(method, path)` pair isn't in the matrix — the validator treats that
/// as permissive (R19).
///
/// `method` is uppercased into a stack buffer before the hash lookup;
/// non-ASCII or oversize methods short-circuit to `None` because they
/// can't match any emitted entry.
#[must_use]
pub fn supported_auth(method: &str, path: &str) -> Option<&'static [AuthScheme]> {
    let bytes = method.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_METHOD_LEN {
        return None;
    }
    let mut buf = [0u8; MAX_METHOD_LEN];
    for (i, &b) in bytes.iter().enumerate() {
        if !b.is_ascii() {
            return None;
        }
        buf[i] = b.to_ascii_uppercase();
    }
    let upper = std::str::from_utf8(&buf[..bytes.len()]).expect("ASCII upper is valid UTF-8");

    let mut key = String::with_capacity(upper.len() + 1 + path.len());
    write!(&mut key, "{upper}\0{path}").expect("write to String never fails");
    AUTH_MATRIX.get(key.as_str()).copied()
}

/// Maps an [`AuthScheme`] to its wire-format `--auth` flag value.
///
/// - [`AuthScheme::Bearer`] → `"app"` (matches the CLI's `--auth app`).
/// - [`AuthScheme::OAuth1User`] → `"oauth1"`.
/// - [`AuthScheme::OAuth2User`] → `"oauth2"` (scope list ignored — v1
///   doesn't surface scopes; KTD-4 / KTD-P3).
#[must_use]
pub fn auth_scheme_wire_str(scheme: AuthScheme) -> &'static str {
    match scheme {
        AuthScheme::Bearer => "app",
        AuthScheme::OAuth1User => "oauth1",
        AuthScheme::OAuth2User(_) => "oauth2",
    }
}

/// Collapses a slice of [`AuthScheme`] entries into the deduplicated wire
/// list used in `AuthMethodMismatch.supported`.
///
/// Insertion order is preserved (matrix entries already follow the spec's
/// `security:` list order). Duplicates can appear when the spec lists
/// multiple OAuth2 scope sets for the same endpoint — collapse them so the
/// envelope reads `["oauth2", "oauth1"]`, not `["oauth2", "oauth2", "oauth1"]`.
pub(crate) fn schemes_to_wire_list(schemes: &[AuthScheme]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(schemes.len());
    for s in schemes {
        let w = auth_scheme_wire_str(*s);
        if !out.iter().any(|existing| existing == w) {
            out.push(w.to_string());
        }
    }
    out
}

/// Validates that `requested_auth` is accepted at `(method, path)` per the
/// spec-derived auth matrix.
///
/// Rules (per brainstorm R10, R18, R19 and plan U6):
/// 1. [`RequestTarget::RawUrl`] → `Ok(())` unconditionally (R18 — raw mode
///    bypasses the matrix).
/// 2. [`RequestTarget::Template`] with matrix-miss → `Ok(())` (R19 —
///    unknown endpoints are permissive).
/// 3. [`RequestTarget::Template`] with matrix-hit AND empty `requested_auth`
///    → `Ok(())` (auto-detect path; U7 owns the dispatch).
/// 4. [`RequestTarget::Template`] with matrix-hit AND `requested_auth` in
///    the supported set → `Ok(())`.
/// 5. [`RequestTarget::Template`] with matrix-hit AND `requested_auth` NOT
///    in the supported set → `Err(XurlError::AuthMethodMismatch{...})`.
///
/// The error variant is U6's "explicit-mismatch" shape: `requested =
/// Some(requested_auth)`, `available_in_app = None`. U7's empty-intersection
/// shape is constructed elsewhere.
///
/// `method` is uppercased before lookup; the matrix key is case-insensitive
/// on method per [`supported_auth`].
///
/// # Errors
///
/// Returns [`XurlError::AuthMethodMismatch`] when rule 5 fires.
pub fn validate(target: &RequestTarget, method: &str, requested_auth: &str) -> Result<()> {
    // Rule 1: raw mode bypasses the matrix.
    let RequestTarget::Template { path, .. } = target else {
        return Ok(());
    };

    // Rule 2: unknown endpoints are permissive.
    let Some(schemes) = supported_auth(method, path) else {
        return Ok(());
    };

    // Rule 3: empty requested_auth is the auto-detect path — U7 owns it.
    if requested_auth.is_empty() {
        return Ok(());
    }

    // Rule 4/5: explicit auth must be in the supported set.
    let requested_norm = requested_auth.to_ascii_lowercase();
    let supported = schemes_to_wire_list(schemes);
    if supported.iter().any(|s| s == &requested_norm) {
        return Ok(());
    }

    Err(XurlError::AuthMethodMismatch {
        endpoint: path.clone(),
        method: method.to_ascii_uppercase(),
        requested: Some(requested_norm),
        supported,
        available_in_app: None,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{AUTH_MATRIX, AuthScheme, SHORTCUT_TEMPLATES, supported_auth, validate};
    use crate::api::request::RequestTarget;
    use crate::error::XurlError;

    /// Shortcut + media layer currently targets 32 (method, path) pairs.
    /// Updating this requires updating both the build-time allowlist in
    /// `build.rs` and the runtime mirror at the top of this file.
    const EXPECTED_SHORTCUT_COUNT: usize = 32;

    #[test]
    fn shortcut_templates_anchor_count() {
        assert_eq!(
            SHORTCUT_TEMPLATES.len(),
            EXPECTED_SHORTCUT_COUNT,
            "SHORTCUT_TEMPLATES drifted from the locked-in shortcut count"
        );
    }

    #[test]
    fn matrix_has_at_least_one_entry_per_allowlist_pair() {
        // Every allowlist pair either appears in the matrix or its spec
        // entry had no `security:` field (treated as permissive at
        // runtime). At minimum the matrix must be non-empty.
        assert!(
            !AUTH_MATRIX.is_empty(),
            "AUTH_MATRIX is empty — build-time codegen produced nothing"
        );
        assert!(
            AUTH_MATRIX.len() >= EXPECTED_SHORTCUT_COUNT - 2,
            "AUTH_MATRIX has {} entries, expected at least {}",
            AUTH_MATRIX.len(),
            EXPECTED_SHORTCUT_COUNT - 2,
        );
    }

    #[test]
    fn supported_auth_media_upload_post() {
        let schemes = supported_auth("POST", "/2/media/upload")
            .expect("/2/media/upload POST must be in the matrix");
        assert_eq!(
            schemes,
            &[
                AuthScheme::OAuth2User(&["media.write"]),
                AuthScheme::OAuth1User,
            ],
            "media upload init must accept OAuth2 (media.write) + OAuth1"
        );
    }

    #[test]
    fn supported_auth_delete_tweet() {
        let schemes = supported_auth("DELETE", "/2/tweets/{id}")
            .expect("DELETE /2/tweets/{{id}} must be in the matrix");
        assert!(
            !schemes.is_empty(),
            "DELETE /2/tweets/{{id}} must declare at least one scheme"
        );
        assert!(
            schemes.contains(&AuthScheme::OAuth1User),
            "DELETE /2/tweets/{{id}} must accept OAuth1 user context"
        );
    }

    #[test]
    fn literal_vs_param_no_precedence_resolution() {
        // `/2/users/me` is in the allowlist; `/2/users/{id}` is NOT (no
        // shortcut targets the bare user-lookup endpoint). Lookup must hit
        // the literal entry only — no fallback to the sibling param path.
        let me = supported_auth("GET", "/2/users/me");
        assert!(me.is_some(), "/2/users/me must be in the matrix");

        let bare = supported_auth("GET", "/2/users/{id}");
        assert!(
            bare.is_none(),
            "lookup must be a direct hash hit, no path-template precedence resolution"
        );
    }

    #[test]
    fn unknown_path_returns_none() {
        assert_eq!(supported_auth("POST", "/2/never/heard/of"), None);
    }

    #[test]
    fn unknown_method_returns_none() {
        // PATCH is not declared on `/2/tweets`. Method + path is the
        // composite key — wrong method must miss even when path is known.
        assert_eq!(supported_auth("PATCH", "/2/tweets"), None);
    }

    #[test]
    fn method_case_insensitive() {
        let upper = supported_auth("POST", "/2/tweets");
        let lower = supported_auth("post", "/2/tweets");
        assert!(upper.is_some());
        assert_eq!(upper, lower, "method comparison must be case-insensitive");
    }

    // ── validate() tests ────────────────────────────────────────────────

    fn tmpl(path: &str) -> RequestTarget {
        RequestTarget::Template {
            path: path.to_string(),
            path_params: HashMap::new(),
            query: Vec::new(),
        }
    }

    #[test]
    fn validate_raw_url_always_ok() {
        // Rule 1 (R18): raw mode bypasses the matrix regardless of method,
        // path, or requested auth — including auth strings that would fail
        // for any template path.
        let target = RequestTarget::RawUrl("https://api.x.com/2/media/upload".to_string());
        assert!(validate(&target, "POST", "app").is_ok());
        assert!(validate(&target, "GET", "oauth1").is_ok());
        assert!(validate(&target, "DELETE", "").is_ok());
    }

    #[test]
    fn validate_template_empty_requested_is_ok() {
        // Rule 3: empty `requested_auth` is the auto-detect path (U7 owns
        // dispatch). Validator must not reject — even for endpoints with a
        // restrictive supported set.
        let target = tmpl("/2/media/upload");
        assert!(validate(&target, "POST", "").is_ok());
    }

    #[test]
    fn validate_template_matrix_miss_is_ok() {
        // Rule 2 (R19): unknown endpoints are permissive even when the user
        // pinned an `--auth` value. The validator yields to upstream
        // semantics rather than fabricating supported lists.
        let target = tmpl("/2/never/heard/of");
        assert!(validate(&target, "POST", "app").is_ok());
        assert!(validate(&target, "GET", "oauth2").is_ok());
    }

    #[test]
    fn validate_template_requested_in_supported_is_ok() {
        // Rule 4: an explicit `--auth` value the endpoint accepts passes
        // through. `/2/media/upload` POST accepts OAuth2 (media.write) and
        // OAuth1 per the spec — both must be accepted.
        let target = tmpl("/2/media/upload");
        assert!(validate(&target, "POST", "oauth1").is_ok());
        assert!(validate(&target, "POST", "oauth2").is_ok());
    }

    #[test]
    fn validate_template_requested_case_insensitive() {
        // The matrix lookup is case-insensitive on method; the auth string
        // is normalised the same way so `OAuth1` matches `oauth1` even
        // though clap conventionally lowercases the value.
        let target = tmpl("/2/media/upload");
        assert!(validate(&target, "post", "OAuth1").is_ok());
    }

    #[test]
    fn validate_template_requested_not_in_supported_errors() {
        // Rule 5: this is the explicit-mismatch path. `/2/media/upload`
        // POST accepts OAuth1 + OAuth2 but not Bearer. Validator must emit
        // the U6 envelope shape: `requested = Some("app")`,
        // `available_in_app = None`.
        let target = tmpl("/2/media/upload");
        let err = validate(&target, "POST", "app").unwrap_err();
        match err {
            XurlError::AuthMethodMismatch {
                endpoint,
                method,
                requested,
                supported,
                available_in_app,
            } => {
                assert_eq!(endpoint, "/2/media/upload");
                assert_eq!(method, "POST");
                assert_eq!(requested.as_deref(), Some("app"));
                // OAuth2 (media.write) + OAuth1 collapse to ["oauth2", "oauth1"].
                assert_eq!(supported, vec!["oauth2".to_string(), "oauth1".to_string()]);
                assert!(
                    available_in_app.is_none(),
                    "U6 explicit-mismatch shape must leave `available_in_app` at None"
                );
            }
            other => panic!("expected AuthMethodMismatch, got {other:?}"),
        }
    }

    #[test]
    fn validate_envelope_message_lists_alternatives() {
        // The Display message (and envelope `message` field) must surface
        // the actionable `--auth ...` alternatives so a user can fix the
        // invocation without consulting the matrix by hand.
        let target = tmpl("/2/media/upload");
        let err = validate(&target, "POST", "app").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Bearer (app)"),
            "message must pretty-print requested scheme: {msg}"
        );
        assert!(
            msg.contains("--auth oauth2") && msg.contains("--auth oauth1"),
            "message must list both supported alternatives: {msg}"
        );
        assert!(
            msg.contains("POST /2/media/upload"),
            "message must include method + endpoint: {msg}"
        );
    }
}
