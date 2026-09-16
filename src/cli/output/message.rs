//! The text `xr` prints for a library error.
//!
//! The library's Display strings are bare fragments; the prefixes and the
//! recovery wording here are part of `xr`'s output contract and stay
//! byte-identical across releases.

use crate::api::auth_matrix::WireScheme;
use crate::error::{Error, MismatchShape, mismatch_shape};

/// The message `xr` prints for `error`, in text mode and in the envelope.
pub(crate) fn render(error: &Error) -> String {
    match error {
        Error::Http(msg) => format!("HTTP Error: {msg}"),
        Error::Io(msg) => format!("IO Error: {msg}"),
        Error::InvalidMethod(method) => format!("Invalid Method: Invalid HTTP method: {method}"),
        Error::Api { .. } | Error::Validation(_) => error.to_string(),
        Error::InvalidUrl(url) => format!("Invalid URL: {url}"),
        Error::InvalidPathParam { name, value } => {
            format!(
                "Invalid path parameter {name:?}: value {value:?} contains a reserved character"
            )
        }
        Error::Internal(msg) => format!("Internal error: {msg}"),
        Error::Json(msg) => format!("JSON Error: {msg}"),
        Error::Auth(msg) => format!("Auth Error: {msg}"),
        Error::TokenStore(msg) => format!("Token Store Error: {msg}"),
        Error::AuthMethodMismatch {
            endpoint,
            rendered_url,
            method,
            requested,
            supported,
            available_in_app,
            app,
            other_apps_with_creds,
        } => auth_method_mismatch_message(
            endpoint,
            rendered_url.as_deref(),
            method,
            requested.as_deref(),
            supported,
            available_in_app.as_deref(),
            app.as_deref(),
            other_apps_with_creds.as_deref(),
        ),
    }
}

/// Builds the message `xr` prints for `AuthMethodMismatch`.
///
/// Three shapes per the variant's docstring; each ends with an actionable
/// recovery instruction. Prefers `rendered_url` over `endpoint` for
/// user-facing strings so `{id}` placeholders don't leak into messages.
#[allow(clippy::too_many_arguments)]
fn auth_method_mismatch_message(
    endpoint: &str,
    rendered_url: Option<&str>,
    method: &str,
    requested: Option<&str>,
    supported: &[String],
    available_in_app: Option<&[String]>,
    app: Option<&str>,
    other_apps_with_creds: Option<&[String]>,
) -> String {
    let display_path = rendered_url.unwrap_or(endpoint);
    let app_name = app.unwrap_or("the active app");
    let suggest_first = |fallback: &str| {
        supported
            .first()
            .map(|s| format!(" Add credentials with: xr auth {s} --app {fallback}."))
            .unwrap_or_default()
    };
    let list = |items: &[String]| {
        if items.is_empty() {
            "none".to_string()
        } else {
            items.join(", ")
        }
    };

    match mismatch_shape(requested, available_in_app, other_apps_with_creds) {
        MismatchShape::Explicit { requested } => {
            let pretty_req = pretty_scheme(requested);
            let alt = supported
                .iter()
                .map(|s| format!("--auth {s}"))
                .collect::<Vec<_>>()
                .join(" or ");
            if alt.is_empty() {
                format!("{pretty_req} auth is not accepted at {method} {display_path}.")
            } else {
                format!("{pretty_req} auth is not accepted at {method} {display_path}. Use {alt}.")
            }
        }
        MismatchShape::WrongApp { others } => {
            let alts = others.join(", ");
            let accepts = list(supported);
            format!(
                "App '{app_name}' has no stored credentials, but other apps do ({alts}). Endpoint {method} {display_path} accepts: {accepts}. Try --app NAME with one of the apps above."
            )
        }
        MismatchShape::EmptyIntersection { available } => {
            let has = list(available);
            let accepts = list(supported);
            let suggest = suggest_first(app_name);
            format!(
                "No stored auth method on app '{app_name}' is accepted at {method} {display_path}. App has: {has}. Endpoint accepts: {accepts}.{suggest}"
            )
        }
        MismatchShape::Unknown => {
            format!("Auth method is not accepted at {method} {display_path}.")
        }
    }
}

/// Maps a wire-format auth string to its pretty-printed scheme name.
///
/// Delegates to [`WireScheme::pretty`] so the display vocabulary lives in
/// one place. Unknown strings fall back to the input verbatim so a future
/// scheme added to the matrix without an updated pretty mapping still
/// surfaces something readable.
fn pretty_scheme(name: &str) -> String {
    WireScheme::from_wire(name)
        .map(|ws| ws.pretty().to_string())
        .unwrap_or_else(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mismatch(
        requested: Option<&str>,
        available_in_app: Option<&[&str]>,
        app: Option<&str>,
        other_apps_with_creds: Option<&[&str]>,
    ) -> Error {
        let strings = |items: &[&str]| items.iter().map(ToString::to_string).collect::<Vec<_>>();
        Error::AuthMethodMismatch {
            endpoint: "/2/users/me".into(),
            rendered_url: Some("/2/users/me".into()),
            method: "GET".into(),
            requested: requested.map(str::to_string),
            supported: vec!["oauth2".into(), "oauth1".into()],
            available_in_app: available_in_app.map(strings),
            app: app.map(str::to_string),
            other_apps_with_creds: other_apps_with_creds.map(strings),
        }
    }

    /// The prefixes are the ones the golden fixtures pin; a library Display
    /// change must never reach `xr`'s output.
    #[test]
    fn every_variant_renders_with_the_prefix_xr_has_always_printed() {
        let cases = [
            (
                Error::auth("NoAuthMethod: no authentication method available"),
                "Auth Error: NoAuthMethod: no authentication method available",
            ),
            (
                Error::token_store("app \"nosuchapp\" not found"),
                "Token Store Error: app \"nosuchapp\" not found",
            ),
            (
                Error::Json("failed to parse media ID from init response".into()),
                "JSON Error: failed to parse media ID from init response",
            ),
            (
                Error::InvalidMethod("BAD METHOD".into()),
                "Invalid Method: Invalid HTTP method: BAD METHOD",
            ),
            (
                Error::InvalidPathParam {
                    name: "id".into(),
                    value: "1/2".into(),
                },
                "Invalid path parameter \"id\": value \"1/2\" contains a reserved character",
            ),
            (
                Error::Http("connection refused".into()),
                "HTTP Error: connection refused",
            ),
            (
                Error::Io("file not found".into()),
                "IO Error: file not found",
            ),
            (
                Error::InvalidUrl("ftp://example".into()),
                "Invalid URL: ftp://example",
            ),
            (
                Error::Internal("missing {id}".into()),
                "Internal error: missing {id}",
            ),
            (Error::api(400, "bad request"), "bad request"),
            (Error::validation("bad input"), "bad input"),
        ];
        for (error, expected) in cases {
            assert_eq!(render(&error), expected, "{error:?}");
        }
    }

    #[test]
    fn auth_method_mismatch_keeps_its_three_recovery_messages() {
        assert_eq!(
            render(&mismatch(Some("oauth1"), None, Some("default"), None)),
            "OAuth 1.0a auth is not accepted at GET /2/users/me. Use --auth oauth2 or --auth oauth1."
        );
        assert_eq!(
            render(&mismatch(None, Some(&["app"]), Some(""), None)),
            "No stored auth method on app '' is accepted at GET /2/users/me. App has: app. Endpoint accepts: oauth2, oauth1. Add credentials with: xr auth oauth2 --app ."
        );
        assert_eq!(
            render(&mismatch(
                None,
                Some(&[]),
                Some("default"),
                Some(&["prod", "staging"])
            )),
            "App 'default' has no stored credentials, but other apps do (prod, staging). Endpoint GET /2/users/me accepts: oauth2, oauth1. Try --app NAME with one of the apps above."
        );
        assert_eq!(
            render(&mismatch(None, None, None, None)),
            "Auth method is not accepted at GET /2/users/me."
        );
    }
}
