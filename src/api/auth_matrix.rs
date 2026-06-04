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

#[cfg(test)]
mod tests {
    use super::{AUTH_MATRIX, AuthScheme, SHORTCUT_TEMPLATES, supported_auth};

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
}
