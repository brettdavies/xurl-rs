//! YAML encoding for the token store, the pending-auth file, and `xr`'s
//! `--output yaml`.

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
