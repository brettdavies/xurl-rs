//! Helpers shared by the spec-facing integration tests: loading the
//! vendored OpenAPI document and following its `$ref`s.
#![allow(dead_code)]

use serde_json::Value;

/// The vendored X API OpenAPI document, relative to the crate root.
pub const SPEC_PATH: &str = "vendor/x-api-openapi.json";

/// Parses the vendored spec.
pub fn load_spec() -> Value {
    let raw = std::fs::read_to_string(SPEC_PATH)
        .unwrap_or_else(|e| panic!("read {SPEC_PATH}: {e} (run scripts/refresh-x-openapi.sh)"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {SPEC_PATH}: {e}"))
}

/// Follows a `$ref` on either side. schemars roots refs at `#/$defs/`;
/// the spec roots them at `#/components/schemas/`.
pub fn resolve<'a>(node: &'a Value, root: &'a Value) -> &'a Value {
    let Some(reference) = node.get("$ref").and_then(Value::as_str) else {
        return node;
    };
    let target = reference
        .strip_prefix("#/$defs/")
        .map(|name| root.pointer(&format!("/$defs/{name}")))
        .or_else(|| {
            reference
                .strip_prefix("#/components/schemas/")
                .map(|name| root.pointer(&format!("/components/schemas/{name}")))
        })
        .flatten();
    target.unwrap_or_else(|| panic!("unresolvable $ref {reference}"))
}
