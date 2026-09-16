//! URL construction for a request target: path-template substitution and
//! percent-encoding of path-parameter and query values.

use std::collections::HashMap;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

use crate::error::{Error, Result};

use super::RequestTarget;

/// Percent-encoding set for path-parameter values and query parameters.
///
/// Matches RFC 3986 §2.3 "unreserved" characters (alphanumeric, `-_.~`)
/// — everything else is encoded. Mirrors `percent-encoding`'s
/// `NON_ALPHANUMERIC` set widened to keep the URL-safe punctuation.
const URL_VALUE_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'&')
    .add(b'+')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

/// Renders the path portion of a [`RequestTarget::Template`] without the
/// `base_url` prefix or query string.
///
/// Used by error-envelope construction so user-facing messages show the
/// substituted path (`/2/users/12345/likes`) instead of the spec template
/// (`/2/users/{id}/likes`). Returns `Err` when the target is `RawUrl` (no
/// substitution applies) or when substitution itself fails — both surface
/// as `None` at the call site so the envelope falls back to `endpoint`.
pub(super) fn render_template_template(target: &RequestTarget) -> Result<String> {
    match target {
        RequestTarget::Template {
            path, path_params, ..
        } => render_template_path(path, path_params),
        RequestTarget::RawUrl(_) => Err(Error::Internal(
            "RawUrl target has no template to render".to_string(),
        )),
    }
}

/// Renders a [`RequestTarget`] against `base_url` into a full URL string.
///
/// Free function so unit tests can exercise the rendering without
/// instantiating a full [`ApiClient`].
pub(super) fn build_url_for_target(base_url: &str, target: &RequestTarget) -> Result<String> {
    match target {
        RequestTarget::Template {
            path,
            path_params,
            query,
        } => {
            let rendered_path = render_template_path(path, path_params)?;

            let mut url = base_url.to_string();
            if !url.ends_with('/') {
                url.push('/');
            }
            if let Some(stripped) = rendered_path.strip_prefix('/') {
                url.push_str(stripped);
            } else {
                url.push_str(&rendered_path);
            }

            if !query.is_empty() {
                url.push('?');
                for (i, (key, value)) in query.iter().enumerate() {
                    if i > 0 {
                        url.push('&');
                    }
                    write_encoded(&mut url, key);
                    url.push('=');
                    write_encoded(&mut url, value);
                }
            }
            Ok(url)
        }
        RequestTarget::RawUrl(raw) => {
            validate_raw_url_scheme(raw)?;
            Ok(raw.clone())
        }
    }
}

/// Substitutes `{name}` segments in a path template using `path_params`.
///
/// Each substituted value is rejected up-front if it contains `/`, `?`,
/// `#`, or `%` (would break URL semantics on encode/decode), then
/// percent-encoded against [`URL_VALUE_ENCODE_SET`]. A `{name}` whose
/// `name` is missing from `path_params` is a programmer error and
/// surfaces as [`Error::Internal`].
pub(crate) fn render_template_path(
    template: &str,
    path_params: &HashMap<String, String>,
) -> Result<String> {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            // Find matching closing brace
            if let Some(end) = template[i + 1..].find('}') {
                let name = &template[i + 1..i + 1 + end];
                let value = path_params.get(name).ok_or_else(|| {
                    Error::Internal(format!(
                        "path template {template:?} references {{{name}}} but path_params has no such key"
                    ))
                })?;
                if value.contains('/')
                    || value.contains('?')
                    || value.contains('#')
                    || value.contains('%')
                {
                    return Err(Error::InvalidPathParam {
                        name: name.to_string(),
                        value: value.clone(),
                    });
                }
                write_encoded(&mut out, value);
                i += 1 + end + 1;
                continue;
            }
        }
        // Push raw byte (template literal char). Safe because template
        // segments outside braces are ASCII per spec.
        out.push(char::from(bytes[i]));
        i += 1;
    }
    Ok(out)
}

/// Validates that a raw URL uses the `http` or `https` scheme.
///
/// Refuses `file://`, `ftp://`, `data:` etc. before any handle to the
/// filesystem or external service is created. Case-insensitive match
/// on the scheme prefix.
fn validate_raw_url_scheme(url: &str) -> Result<()> {
    let lower = url.trim_start().to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        return Ok(());
    }
    Err(Error::InvalidUrl(format!(
        "URL must start with http:// or https://: {url}"
    )))
}

/// Appends a percent-encoded value to `out` using [`URL_VALUE_ENCODE_SET`].
fn write_encoded(out: &mut String, value: &str) {
    for chunk in utf8_percent_encode(value, URL_VALUE_ENCODE_SET) {
        out.push_str(chunk);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── build_url tests ────────────────────────────────────────────────

    const TEST_BASE_URL: &str = "https://api.x.com";

    fn tmpl(path: &str) -> RequestTarget {
        RequestTarget::Template {
            path: path.to_string(),
            path_params: HashMap::new(),
            query: Vec::new(),
        }
    }

    #[test]
    fn build_url_template_empty_params_and_query() {
        let url = build_url_for_target(TEST_BASE_URL, &tmpl("/2/users/me")).unwrap();
        assert_eq!(url, "https://api.x.com/2/users/me");
    }

    #[test]
    fn build_url_template_substitutes_path_param() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "12345".to_string());
        let target = RequestTarget::Template {
            path: "/2/users/{id}/likes".to_string(),
            path_params: params,
            query: Vec::new(),
        };
        let url = build_url_for_target(TEST_BASE_URL, &target).unwrap();
        assert_eq!(url, "https://api.x.com/2/users/12345/likes");
    }

    #[test]
    fn build_url_template_query_preserves_insertion_order() {
        let target = RequestTarget::Template {
            path: "/2/tweets/search/recent".to_string(),
            path_params: HashMap::new(),
            query: vec![
                ("query".to_string(), "rustlang".to_string()),
                ("max_results".to_string(), "10".to_string()),
            ],
        };
        let url = build_url_for_target(TEST_BASE_URL, &target).unwrap();
        assert_eq!(
            url,
            "https://api.x.com/2/tweets/search/recent?query=rustlang&max_results=10"
        );
    }

    #[test]
    fn build_url_template_percent_encodes_value_with_spaces() {
        let target = RequestTarget::Template {
            path: "/2/tweets/search/recent".to_string(),
            path_params: HashMap::new(),
            query: vec![("query".to_string(), "hello world".to_string())],
        };
        let url = build_url_for_target(TEST_BASE_URL, &target).unwrap();
        assert_eq!(
            url,
            "https://api.x.com/2/tweets/search/recent?query=hello%20world"
        );
    }

    #[test]
    fn build_url_template_rejects_path_param_with_slash() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "abc/etc/passwd".to_string());
        let target = RequestTarget::Template {
            path: "/2/users/{id}/likes".to_string(),
            path_params: params,
            query: Vec::new(),
        };
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        match err {
            Error::InvalidPathParam { name, value } => {
                assert_eq!(name, "id");
                assert_eq!(value, "abc/etc/passwd");
            }
            other => panic!("expected InvalidPathParam, got {other:?}"),
        }
    }

    #[test]
    fn build_url_template_rejects_path_param_with_hash() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "abc#fragment".to_string());
        let target = RequestTarget::Template {
            path: "/2/users/{id}/likes".to_string(),
            path_params: params,
            query: Vec::new(),
        };
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        match err {
            Error::InvalidPathParam { name, value } => {
                assert_eq!(name, "id");
                assert_eq!(value, "abc#fragment");
            }
            other => panic!("expected InvalidPathParam, got {other:?}"),
        }
    }

    #[test]
    fn build_url_template_rejects_path_param_with_percent() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "already%20encoded".to_string());
        let target = RequestTarget::Template {
            path: "/2/users/{id}/likes".to_string(),
            path_params: params,
            query: Vec::new(),
        };
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        match err {
            Error::InvalidPathParam { name, value } => {
                assert_eq!(name, "id");
                assert_eq!(value, "already%20encoded");
            }
            other => panic!("expected InvalidPathParam, got {other:?}"),
        }
    }

    #[test]
    fn build_url_template_rejects_path_param_with_question_mark() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "abc?injected".to_string());
        let target = RequestTarget::Template {
            path: "/2/users/{id}".to_string(),
            path_params: params,
            query: Vec::new(),
        };
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        assert!(matches!(err, Error::InvalidPathParam { .. }));
    }

    #[test]
    fn build_url_template_missing_path_param_is_internal_error() {
        let target = RequestTarget::Template {
            path: "/2/users/{id}/likes".to_string(),
            path_params: HashMap::new(),
            query: Vec::new(),
        };
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        assert!(matches!(err, Error::Internal(_)), "got {err:?}");
    }

    #[test]
    fn build_url_raw_url_https_returns_clone() {
        let target = RequestTarget::RawUrl("https://api.x.com/2/raw".to_string());
        let url = build_url_for_target(TEST_BASE_URL, &target).unwrap();
        assert_eq!(url, "https://api.x.com/2/raw");
    }

    #[test]
    fn build_url_raw_url_http_returns_clone() {
        let target = RequestTarget::RawUrl("http://localhost:8080/dev".to_string());
        let url = build_url_for_target(TEST_BASE_URL, &target).unwrap();
        assert_eq!(url, "http://localhost:8080/dev");
    }

    #[test]
    fn build_url_raw_url_file_scheme_rejected() {
        let target = RequestTarget::RawUrl("file:///etc/passwd".to_string());
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        assert!(matches!(err, Error::InvalidUrl(_)), "got {err:?}");
    }

    #[test]
    fn build_url_raw_url_ftp_scheme_rejected() {
        let target = RequestTarget::RawUrl("ftp://attacker.com/payload".to_string());
        let err = build_url_for_target(TEST_BASE_URL, &target).unwrap_err();
        assert!(matches!(err, Error::InvalidUrl(_)), "got {err:?}");
    }
}
