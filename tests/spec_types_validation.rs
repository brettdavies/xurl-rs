//! Validates the hand-written typed responses against the vendored X API
//! OpenAPI spec.
//!
//! A spec refresh lands only vendored artifacts; the auth-method matrix
//! regenerates at build time, but nothing regenerates the hand-written
//! structs in `src/api/response/types.rs`. This gate fails when the spec
//! renames or removes a field (or a whole component schema) those structs
//! still declare, so a refresh PR stays red until the types get their
//! review pass.
//!
//! Direction is struct -> spec only: every named struct field must exist in
//! the mapped component schema. New spec-only fields never fail here (the
//! structs capture them via their `extra` flatten buckets, and the drift
//! report on the refresh PR surfaces them for promotion).
//!
//! Mapped payloads are the typed responses with a component-schema
//! counterpart. Envelope types (`ApiResponse`, `Includes`, `ResponseMeta`,
//! `ApiError`), action confirmations (`LikedResult`, ...), and
//! `MediaUploadResponse` validate against inline response schemas rather
//! than components and stay out of this gate.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use xurl::api::response::types::{DmEvent, Post, UsageData, User};

/// Documented divergences from the vendored spec, keyed by
/// (`<SchemaName> :: <dotted struct path>`, field). Every entry needs a
/// reason: an upstream spec bug or lag, never local convenience.
const ALLOWED_DIVERGENCES: &[(&str, &str)] = &[];

const SPEC_PATH: &str = "vendor/x-api-openapi.json";

struct Failure {
    location: String,
    field: String,
    detail: String,
}

fn load_spec() -> Value {
    let raw = std::fs::read_to_string(SPEC_PATH)
        .unwrap_or_else(|e| panic!("read {SPEC_PATH}: {e} (run scripts/refresh-x-openapi.sh)"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {SPEC_PATH}: {e}"))
}

/// Follows a `$ref` on either side. schemars roots refs at `#/$defs/`;
/// the spec roots them at `#/components/schemas/`.
fn resolve<'a>(node: &'a Value, root: &'a Value) -> &'a Value {
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

/// Collects the property map of a spec schema, merging `allOf` members so
/// composed schemas expose their full field surface.
fn spec_properties<'a>(node: &'a Value, root: &'a Value, out: &mut BTreeMap<String, &'a Value>) {
    let node = resolve(node, root);
    if let Some(props) = node.get("properties").and_then(Value::as_object) {
        for (k, v) in props {
            out.entry(k.clone()).or_insert(v);
        }
    }
    for key in ["allOf", "anyOf", "oneOf"] {
        if let Some(members) = node.get(key).and_then(Value::as_array) {
            for member in members {
                spec_properties(member, root, out);
            }
        }
    }
}

/// Unwraps schemars composition (`anyOf` for `Option<T>` and friends) into
/// the object-bearing member schemas.
fn struct_object_nodes<'a>(node: &'a Value, root: &'a Value, out: &mut Vec<&'a Value>) {
    let node = resolve(node, root);
    if node.get("properties").is_some() {
        out.push(node);
    }
    for key in ["anyOf", "oneOf", "allOf"] {
        if let Some(members) = node.get(key).and_then(Value::as_array) {
            for member in members {
                struct_object_nodes(member, root, out);
            }
        }
    }
}

/// Steps both sides through array `items` so element schemas compare
/// directly. A side without `items` passes through unchanged.
fn unwrap_items<'a>(node: &'a Value, root: &'a Value) -> &'a Value {
    let resolved = resolve(node, root);
    match resolved.get("items") {
        Some(items) => resolve(items, root),
        None => resolved,
    }
}

#[allow(clippy::too_many_arguments)]
fn compare(
    struct_node: &Value,
    spec_node: &Value,
    struct_root: &Value,
    spec_root: &Value,
    location: &str,
    visited: &mut BTreeSet<String>,
    failures: &mut Vec<Failure>,
) {
    // Cycle guard on the pair of resolved refs.
    let key = format!(
        "{}|{}",
        struct_node
            .get("$ref")
            .and_then(Value::as_str)
            .unwrap_or(location),
        spec_node.get("$ref").and_then(Value::as_str).unwrap_or("-"),
    );
    if !visited.insert(key) {
        return;
    }

    let mut struct_objects = Vec::new();
    struct_object_nodes(struct_node, struct_root, &mut struct_objects);
    if struct_objects.is_empty() {
        return;
    }

    let mut spec_props = BTreeMap::new();
    spec_properties(spec_node, spec_root, &mut spec_props);
    if spec_props.is_empty() {
        // Presence-only gate: an opaque spec node (free-form object,
        // primitive) cannot invalidate named struct fields.
        return;
    }

    for object in struct_objects {
        let Some(props) = object.get("properties").and_then(Value::as_object) else {
            continue;
        };
        for (field, sub) in props {
            if ALLOWED_DIVERGENCES.contains(&(location, field.as_str())) {
                continue;
            }
            match spec_props.get(field.as_str()) {
                None => failures.push(Failure {
                    location: location.to_string(),
                    field: field.clone(),
                    detail: "field absent from the spec schema (renamed or removed upstream)"
                        .to_string(),
                }),
                Some(spec_sub) => {
                    let next_location = format!("{location}.{field}");
                    compare(
                        unwrap_items(sub, struct_root),
                        unwrap_items(spec_sub, spec_root),
                        struct_root,
                        spec_root,
                        &next_location,
                        visited,
                        failures,
                    );
                }
            }
        }
    }
}

#[test]
fn typed_responses_match_vendored_spec() {
    let spec = load_spec();

    // The shortcut->schema mapping: typed payload -> spec component schema.
    let mappings: Vec<(&str, Value)> = vec![
        (
            "Post",
            serde_json::to_value(schemars::schema_for!(Post)).unwrap(),
        ),
        (
            "User",
            serde_json::to_value(schemars::schema_for!(User)).unwrap(),
        ),
        (
            "DmEvent",
            serde_json::to_value(schemars::schema_for!(DmEvent)).unwrap(),
        ),
        (
            "Usage",
            serde_json::to_value(schemars::schema_for!(UsageData)).unwrap(),
        ),
    ];

    let mut failures = Vec::new();
    for (schema_name, struct_schema) in &mappings {
        let Some(spec_schema) = spec.pointer(&format!("/components/schemas/{schema_name}")) else {
            failures.push(Failure {
                location: (*schema_name).to_string(),
                field: "<schema>".to_string(),
                detail: "component schema missing from the vendored spec (renamed or removed \
                         upstream); update the mapping and the typed structs"
                    .to_string(),
            });
            continue;
        };
        let mut visited = BTreeSet::new();
        compare(
            struct_schema,
            spec_schema,
            struct_schema,
            &spec,
            schema_name,
            &mut visited,
            &mut failures,
        );
    }

    if !failures.is_empty() {
        let mut report = String::from(
            "typed responses diverge from the vendored spec; fix src/api/response/types.rs or \
             extend ALLOWED_DIVERGENCES with a documented reason:\n",
        );
        for f in &failures {
            report.push_str(&format!(
                "  {} :: {} -> {}\n",
                f.location, f.field, f.detail
            ));
        }
        panic!("{report}");
    }
}
