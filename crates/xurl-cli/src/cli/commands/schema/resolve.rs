//! The views `xr schema` derives from a response type's schema: the
//! command's `data` resolved in place, and the node at a dot path.
//!
//! Both read the schema schemars renders with `inline_subschemas`, which
//! inlines every subschema and keeps a `$ref` only for a recursive type. The
//! type's name, which an inlined body no longer carries, comes from the
//! default rendering.

use serde_json::{Map, Value};

/// What the default view adds to `data`'s resolved schema.
pub struct Labels<'a> {
    /// The command's words.
    pub command: &'a str,
    /// The invocation that prints the shared wrapper, labelled on a command
    /// that answers with it.
    pub common: &'a str,
}

/// A dot path that names no property.
#[derive(Debug, PartialEq, Eq)]
pub struct PathMiss {
    /// The segments that resolved, joined with `.`.
    pub resolved: String,
    /// The segment that named nothing.
    pub segment: String,
    /// The property names at the last node that resolved, sorted.
    pub known: Vec<String>,
}

/// The default view: the element `data` carries, every subschema inlined,
/// under the labels an agent reads before writing a filter.
///
/// `full` is the default rendering and `inlined` the inlined one of the same
/// type. For a type the API wrapper carries, `data` is its `data` property;
/// for a shape the CLI builds itself, the whole type.
#[must_use]
pub fn data_view(full: &Value, inlined: &Value, labels: &Labels<'_>) -> Value {
    let wrapped = is_wrapper(full);
    let (full_data, inlined_data) = if wrapped {
        (&full["properties"]["data"], &inlined["properties"]["data"])
    } else {
        (full, inlined)
    };
    let nullable = is_nullable(full_data);
    let (full_element, array) = element(full_data);
    let (inlined_element, _) = element(inlined_data);

    let data_type = full_element
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|reference| reference.rsplit('/').next())
        .or_else(|| {
            full.get("title")
                .and_then(Value::as_str)
                .filter(|_| !wrapped)
        });

    let mut body: Map<String, Value> = inlined_element.as_object().cloned().unwrap_or_default();
    body.remove("$schema");
    body.remove("title");
    if wrapped || array {
        body.remove("description");
        if let Some(description) =
            data_type.and_then(|name| full.pointer(&format!("/$defs/{name}/description")))
        {
            body.insert("description".into(), description.clone());
        }
    }
    if let Some(defs) = inlined.get("$defs") {
        body.insert("$defs".into(), defs.clone());
    }

    body.insert("command".into(), labels.command.into());
    body.insert(
        "data_cardinality".into(),
        if array { "array" } else { "object" }.into(),
    );
    if let Some(name) = data_type {
        body.insert("data_type".into(), name.into());
    }
    if wrapped {
        body.insert("common_schema".into(), labels.common.into());
    }
    if nullable {
        body.insert("data_nullable".into(), true.into());
    }
    Value::Object(body)
}

/// Whether `schema` is the API wrapper: `data` beside `includes`.
#[must_use]
pub fn is_wrapper(schema: &Value) -> bool {
    schema.get("properties").is_some_and(|properties| {
        properties.get("data").is_some() && properties.get("includes").is_some()
    })
}

/// The node at `segments` under `root`. Each step, the last included, goes
/// through a nullable value and an array to the object it carries.
///
/// # Errors
///
/// Returns the [`PathMiss`] for the first segment that names no property.
pub fn node_at(root: &Value, segments: &[&str]) -> Result<Value, PathMiss> {
    let mut current = root;
    let mut resolved: Vec<&str> = Vec::new();
    for segment in segments {
        current = through(current);
        match current
            .get("properties")
            .and_then(|properties| properties.get(*segment))
        {
            Some(next) => {
                resolved.push(segment);
                current = next;
            }
            None => {
                let mut known: Vec<String> = current
                    .get("properties")
                    .and_then(Value::as_object)
                    .map(|properties| properties.keys().cloned().collect())
                    .unwrap_or_default();
                known.sort();
                return Err(PathMiss {
                    resolved: resolved.join("."),
                    segment: (*segment).to_string(),
                    known,
                });
            }
        }
    }
    Ok(through(current).clone())
}

/// The element `node` carries: a nullable value unwrapped, then one array.
/// The flag says whether an array was unwrapped.
fn element(node: &Value) -> (&Value, bool) {
    let mut current = node;
    while let Some(inner) = non_null_branch(current) {
        current = inner;
    }
    if has_type(current, "array")
        && let Some(items) = current.get("items")
    {
        return (items, true);
    }
    (current, false)
}

/// `node` with every nullable value and array stepped through.
fn through(node: &Value) -> &Value {
    let mut current = node;
    loop {
        if let Some(inner) = non_null_branch(current) {
            current = inner;
        } else if has_type(current, "array")
            && let Some(items) = current.get("items")
        {
            current = items;
        } else {
            return current;
        }
    }
}

/// Whether `node` may be null: `null` among its types, or an `anyOf` with a
/// `null` branch.
fn is_nullable(node: &Value) -> bool {
    has_type(node, "null") || non_null_branch(node).is_some()
}

/// The one branch of an `anyOf` that is not `null`, when the rest are.
fn non_null_branch(node: &Value) -> Option<&Value> {
    let branches = node.get("anyOf")?.as_array()?;
    let mut others = branches
        .iter()
        .filter(|branch| !has_only_type(branch, "null"));
    let only = others.next()?;
    (others.next().is_none() && branches.len() > 1).then_some(only)
}

/// Whether `node`'s `type` is `name` or a list holding it.
fn has_type(node: &Value, name: &str) -> bool {
    match node.get("type") {
        Some(Value::String(single)) => single == name,
        Some(Value::Array(types)) => types.iter().any(|kind| kind == name),
        _ => false,
    }
}

/// Whether `node`'s `type` is exactly `name`.
fn has_only_type(node: &Value, name: &str) -> bool {
    node.get("type").and_then(Value::as_str) == Some(name)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Labels, PathMiss, data_view, node_at};

    const LABELS: Labels<'static> = Labels {
        command: "thing get",
        common: "xr schema --common",
    };

    fn wrapper(
        data: serde_json::Value,
        inlined_data: serde_json::Value,
    ) -> (serde_json::Value, serde_json::Value) {
        let defs = json!({"Thing": {"description": "A thing.", "type": "object", "properties": {"id": {"type": "string"}}}});
        let full = json!({
            "title": "ApiResponse",
            "properties": {"data": data, "includes": {"$ref": "#/$defs/Includes"}},
            "$defs": defs,
        });
        let inlined = json!({
            "title": "ApiResponse",
            "properties": {"data": inlined_data, "includes": {"type": "object"}},
        });
        (full, inlined)
    }

    #[test]
    fn an_object_data_is_inlined_under_its_labels_and_its_own_description() {
        let (full, inlined) = wrapper(
            json!({"$ref": "#/$defs/Thing", "description": "Primary payload."}),
            json!({"description": "Primary payload.", "type": "object", "properties": {"id": {"type": "string"}}}),
        );
        let view = data_view(&full, &inlined, &LABELS);
        assert_eq!(view["command"], "thing get");
        assert_eq!(view["data_cardinality"], "object");
        assert_eq!(view["data_type"], "Thing");
        assert_eq!(view["common_schema"], "xr schema --common");
        assert_eq!(view["description"], "A thing.");
        assert_eq!(view["properties"]["id"]["type"], "string");
        assert!(view.get("data_nullable").is_none());
        assert!(view.get("$defs").is_none());
    }

    #[test]
    fn a_nullable_array_unwraps_to_its_element_and_says_so() {
        let (full, inlined) = wrapper(
            json!({"type": ["array", "null"], "items": {"$ref": "#/$defs/Thing"}}),
            json!({"type": ["array", "null"], "items": {"type": "object", "properties": {"id": {"type": "string"}}}}),
        );
        let view = data_view(&full, &inlined, &LABELS);
        assert_eq!(view["data_cardinality"], "array");
        assert_eq!(view["data_nullable"], true);
        assert_eq!(view["data_type"], "Thing");
        assert_eq!(view["type"], "object");
    }

    #[test]
    fn a_nullable_any_of_unwraps_to_its_one_other_branch() {
        let (full, inlined) = wrapper(
            json!({"anyOf": [{"$ref": "#/$defs/Thing"}, {"type": "null"}]}),
            json!({"anyOf": [{"type": "object", "properties": {"id": {"type": "string"}}}, {"type": "null"}]}),
        );
        let view = data_view(&full, &inlined, &LABELS);
        assert_eq!(view["data_nullable"], true);
        assert_eq!(view["data_type"], "Thing");
        assert_eq!(view["properties"]["id"]["type"], "string");
    }

    #[test]
    fn a_cli_built_object_takes_its_root_title_and_no_common_label() {
        let full = json!({"$schema": "x", "title": "Report", "type": "object", "properties": {"ok": {"type": "boolean"}}});
        let view = data_view(&full, &full, &LABELS);
        assert_eq!(view["data_type"], "Report");
        assert_eq!(view["data_cardinality"], "object");
        assert!(view.get("common_schema").is_none());
        assert!(view.get("$schema").is_none() && view.get("title").is_none());
    }

    #[test]
    fn definitions_left_by_a_recursive_type_stay_with_the_view() {
        let (full, mut inlined) = wrapper(
            json!({"$ref": "#/$defs/Thing"}),
            json!({"$ref": "#/$defs/Thing"}),
        );
        inlined["$defs"] = json!({"Thing": {"type": "object"}});
        let view = data_view(&full, &inlined, &LABELS);
        assert_eq!(view["$defs"]["Thing"]["type"], "object");
    }

    #[test]
    fn a_dot_path_steps_through_arrays_and_nullable_values() {
        let root = json!({"type": "array", "items": {"type": "object", "properties": {
            "metrics": {"anyOf": [{"type": "object", "properties": {"likes": {"type": "integer"}}}, {"type": "null"}]},
        }}});
        assert_eq!(
            node_at(&root, &["metrics"]).unwrap()["properties"]["likes"]["type"],
            "integer"
        );
        assert_eq!(
            node_at(&root, &["metrics", "likes"]).unwrap()["type"],
            "integer"
        );
        let listed = json!({"type": "object", "properties": {
            "users": {"type": ["array", "null"], "items": {"type": "object", "properties": {"id": {}}}},
        }});
        assert_eq!(node_at(&listed, &["users"]).unwrap()["type"], "object");
    }

    #[test]
    fn a_missing_segment_names_what_resolved_and_what_is_there() {
        let root = json!({"type": "object", "properties": {"b": {"type": "string"}, "a": {"type": "object", "properties": {"x": {}}}}});
        assert_eq!(
            node_at(&root, &["a", "nope"]),
            Err(PathMiss {
                resolved: "a".into(),
                segment: "nope".into(),
                known: vec!["x".into()],
            })
        );
        assert_eq!(
            node_at(&root, &["b", "length"]),
            Err(PathMiss {
                resolved: "b".into(),
                segment: "length".into(),
                known: vec![],
            })
        );
    }
}
