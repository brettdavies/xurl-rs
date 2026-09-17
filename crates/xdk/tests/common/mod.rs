//! Helpers shared by the spec-facing integration tests: loading the
//! vendored OpenAPI document and following its `$ref`s.
#![allow(dead_code)]

use serde_json::Value;
use xdk::api::auth_matrix::Endpoint;

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

/// The responses `endpoint` declares in the spec, keyed by status (`"200"`,
/// `"201"`, `"default"`).
pub fn declared_responses<'a>(
    spec: &'a Value,
    endpoint: &Endpoint,
) -> &'a serde_json::Map<String, Value> {
    spec.pointer(&format!(
        "/paths/{}/{}/responses",
        endpoint.path.replace('/', "~1"),
        endpoint.method.to_lowercase()
    ))
    .and_then(Value::as_object)
    .unwrap_or_else(|| {
        panic!(
            "{} {} is not in the vendored spec",
            endpoint.method, endpoint.path
        )
    })
}

/// What the spec says a reply with a given status carries.
pub enum Reply<'a> {
    /// A JSON body of this schema.
    Json(&'a Value),
    /// A declared reply with no `application/json` body.
    NoJson,
    /// The spec declares no reply with this status.
    Undeclared,
}

/// The reply the spec declares for `endpoint` answering with `status`.
pub fn response_schema<'a>(spec: &'a Value, endpoint: &Endpoint, status: u16) -> Reply<'a> {
    match declared_responses(spec, endpoint).get(&status.to_string()) {
        None => Reply::Undeclared,
        Some(response) => match response.pointer("/content/application~1json/schema") {
            Some(schema) => Reply::Json(schema),
            None => Reply::NoJson,
        },
    }
}

/// Which of an endpoint's declared replies a fixture represents.
#[derive(Clone, Copy, Debug)]
pub enum ReplyKind {
    /// The first 2xx reply, as `application/json`.
    Success,
    /// The `default` failure reply as `application/json` (the `Error` shape).
    Error,
    /// The `default` failure reply as `application/problem+json` (a `Problem`).
    Problem,
}

/// The schema a fixture of `kind` for `endpoint` must satisfy.
pub fn fixture_schema<'a>(spec: &'a Value, endpoint: &Endpoint, kind: ReplyKind) -> &'a Value {
    let (status, content_type) = match kind {
        ReplyKind::Success => return success_schema(spec, endpoint),
        ReplyKind::Error => ("default", "application/json"),
        ReplyKind::Problem => ("default", "application/problem+json"),
    };
    declared_responses(spec, endpoint)
        .get(status)
        .and_then(|response| {
            response.pointer(&format!(
                "/content/{}/schema",
                content_type.replace('/', "~1")
            ))
        })
        .unwrap_or_else(|| {
            panic!(
                "{} {} declares no {status} reply as {content_type}",
                endpoint.method, endpoint.path
            )
        })
}

/// The JSON schema of `endpoint`'s first 2xx response.
pub fn success_schema<'a>(spec: &'a Value, endpoint: &Endpoint) -> &'a Value {
    let (status, _) = declared_responses(spec, endpoint)
        .iter()
        .find(|(status, _)| status.starts_with('2'))
        .unwrap_or_else(|| {
            panic!(
                "{} {} declares no 2xx response",
                endpoint.method, endpoint.path
            )
        });
    match response_schema(spec, endpoint, status.parse().expect("numeric status")) {
        Reply::Json(schema) => schema,
        _ => panic!(
            "{} {} {status} declares no application/json schema",
            endpoint.method, endpoint.path
        ),
    }
}

pub fn kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

/// Checks `value` against the OpenAPI `schema`, appending one line per
/// mismatch to `errors`, each starting with the dotted path of the field.
pub fn check(value: &Value, schema: &Value, root: &Value, path: &str, errors: &mut Vec<String>) {
    let schema = resolve(schema, root);
    if let Some(all) = schema.get("allOf").and_then(Value::as_array) {
        for member in all {
            check(value, member, root, path, errors);
        }
    }
    for key in ["anyOf", "oneOf"] {
        if let Some(options) = schema.get(key).and_then(Value::as_array) {
            let matches_one = options.iter().any(|option| {
                let mut sub = Vec::new();
                check(value, option, root, path, &mut sub);
                sub.is_empty()
            });
            if !matches_one {
                errors.push(format!(
                    "{path}: matches none of the schema's {key} alternatives"
                ));
            }
        }
    }
    if value.is_null() {
        if schema.get("nullable") != Some(&Value::Bool(true)) && schema.get("type").is_some() {
            errors.push(format!(
                "{path}: null where the spec wants {}",
                schema["type"]
            ));
        }
        return;
    }
    if let Some(ty) = schema.get("type").and_then(Value::as_str) {
        let ok = match ty {
            "string" => value.is_string(),
            "number" => value.is_number(),
            "integer" => value.is_i64() || value.is_u64(),
            "boolean" => value.is_boolean(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            _ => true,
        };
        if !ok {
            errors.push(format!("{path}: is {} but the spec says {ty}", kind(value)));
            return;
        }
    }
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array)
        && !allowed.contains(value)
    {
        errors.push(format!(
            "{path}: {value} is not one of the spec's enum values {allowed:?}"
        ));
    }
    if let Some(object) = value.as_object() {
        let properties = schema.get("properties").and_then(Value::as_object);
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for name in required.iter().filter_map(Value::as_str) {
                if !object.contains_key(name) {
                    errors.push(format!("{path}: missing the required field `{name}`"));
                }
            }
        }
        for (name, field) in object {
            let field_path = format!("{path}.{name}");
            match properties.and_then(|props| props.get(name)) {
                Some(field_schema) => check(field, field_schema, root, &field_path, errors),
                None => match schema.get("additionalProperties") {
                    Some(Value::Bool(false)) => errors.push(format!(
                        "{field_path}: not in the spec's schema, which allows no other properties"
                    )),
                    Some(extra) if extra.is_object() => {
                        check(field, extra, root, &field_path, errors);
                    }
                    _ => {}
                },
            }
        }
    }
    if let (Some(items), Some(item_schema)) = (value.as_array(), schema.get("items")) {
        for (i, item) in items.iter().enumerate() {
            check(item, item_schema, root, &format!("{path}[{i}]"), errors);
        }
    }
}
