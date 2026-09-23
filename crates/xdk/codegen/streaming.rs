//! Derives the streaming endpoint set from the vendored X OpenAPI spec.
//! `build.rs` emits it for `src/api/endpoints.rs`, and
//! `tests/streaming_table.rs` runs this same function over spec fixtures.
//!
//! A path streams when any of its operations carries
//! `x-twitter-streaming: true`, the marker X puts on every long-lived
//! connection it serves.

use std::collections::BTreeSet;

use serde_json::Value;

/// Every spec path with a streaming operation, in path order.
///
/// # Errors
///
/// Returns an error when no operation carries the marker. A spec that marks
/// nothing turns off streaming auto-detection for every endpoint, which
/// reads as a spec refresh that dropped the extension rather than as X
/// retiring every stream.
pub fn derive(spec: &Value) -> Result<Vec<String>, String> {
    let paths: BTreeSet<&str> = spec
        .get("paths")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter(|(_, item)| {
            item.as_object().is_some_and(|ops| {
                ops.values()
                    .any(|op| op.get("x-twitter-streaming") == Some(&Value::Bool(true)))
            })
        })
        .map(|(path, _)| path.as_str())
        .collect();
    if paths.is_empty() {
        return Err(
            "no operation carries `x-twitter-streaming: true`, so no endpoint would \
                    stream without --stream; check that the spec refresh kept the extension"
                .to_string(),
        );
    }
    Ok(paths.into_iter().map(str::to_string).collect())
}
