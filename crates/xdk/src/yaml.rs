//! YAML encoding for the token store, the pending-auth file, and `xr`'s
//! `--output yaml`, and the naming of a parse failure in a file that holds
//! credentials.

use std::path::Path;

use serde::Serialize;

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
