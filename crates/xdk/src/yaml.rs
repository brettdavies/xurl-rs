//! YAML encoding for the token store, the pending-auth file, and `xr`'s
//! `--output yaml`, and the naming of a parse failure in a file that holds
//! credentials.

use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Serializes `value` as YAML.
///
/// # Errors
///
/// Returns the serializer's error when `value` cannot be represented in YAML.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, serde_saphyr::SerializeError> {
    // A single-line string longer than the serializer's default 80 columns
    // would be written as a folded block; tokens and post text stay on one
    // line, as the store has always held them.
    serde_saphyr::to_string_with_options(
        value,
        serde_saphyr::ser_options! { folded_wrap_chars: usize::MAX },
    )
}

/// Parses `text` as YAML, with a parse error that carries no snippet of it.
///
/// The parser's error holds a crop of the input around the failure by default,
/// and its `Debug` and `Display` print the crop. A caller that keeps that error
/// as an [`Error::source`](crate::Error) for a file that holds secrets would
/// hand them to whatever prints the chain.
///
/// # Errors
///
/// Returns the parser's error, positioned but without the input.
pub(crate) fn from_str_without_snippet<T: DeserializeOwned>(
    text: &str,
) -> Result<T, serde_saphyr::Error> {
    serde_saphyr::from_str_with_options(text, serde_saphyr::options! { with_snippet: false })
}

/// [`from_str_without_snippet`] over bytes.
///
/// # Errors
///
/// As [`from_str_without_snippet`].
pub(crate) fn from_slice_without_snippet<T: DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, serde_saphyr::Error> {
    serde_saphyr::from_slice_with_options(bytes, serde_saphyr::options! { with_snippet: false })
}

/// Names the file a YAML parse failed in and where the parser stopped, without
/// a word of the file's content.
///
/// The parser's own text crops the input around the failure and can quote the
/// value that broke it, and the files this crate parses hold secrets: the
/// pending sign-in state carries the PKCE verifier, a `.twurlrc` its consumer
/// and token secrets.
pub(crate) fn failure_site(path: &Path, error: &serde_saphyr::Error) -> String {
    match error.location() {
        Some(at) => format!(
            "{} at line {}, column {}",
            path.display(),
            at.line(),
            at.column()
        ),
        None => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde::de::Error as _;

    use super::{failure_site, from_slice_without_snippet, from_str_without_snippet};

    #[test]
    fn a_located_failure_names_the_file_then_the_line_then_the_column() {
        let error = serde_saphyr::from_str::<serde_json::Value>("ok: 1\nbad: [unclosed\n")
            .expect_err("not YAML");
        let at = error.location().expect("the parser located the failure");
        assert_ne!(at.line(), at.column(), "the fixture tells the two apart");

        assert_eq!(
            failure_site(Path::new("flow.yml"), &error),
            format!("flow.yml at line {}, column {}", at.line(), at.column())
        );
    }

    #[test]
    fn a_failed_parse_keeps_its_position_and_prints_none_of_the_input() {
        const MARKER: &str = "MARKER-NOT-A-SECRET";
        let text = format!("secret: {MARKER}\nother: [unclosed\n");

        let from_str = from_str_without_snippet::<serde_json::Value>(&text).expect_err("not YAML");
        let from_slice =
            from_slice_without_snippet::<serde_json::Value>(text.as_bytes()).expect_err("not YAML");

        for error in [from_str, from_slice] {
            assert!(error.location().is_some(), "the position survives");
            for printed in [format!("{error:?}"), error.to_string()] {
                assert!(!printed.contains(MARKER), "{printed}");
            }
        }
    }

    #[test]
    fn an_unlocated_failure_names_only_the_file() {
        let error = serde_saphyr::Error::custom("MARKER-NOT-A-SECRET");
        assert!(error.location().is_none(), "the fixture has no position");

        assert_eq!(failure_site(Path::new("flow.yml"), &error), "flow.yml");
    }
}
