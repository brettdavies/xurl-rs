//! Derives the legacy post-vocabulary table from the vendored X OpenAPI
//! spec. `build.rs` emits it for the response decoder, and
//! `tests/vocabulary_table.rs` runs this same function over spec fixtures.
//!
//! Each spec property name containing `post` maps to the spelling X used
//! before the rename: `repost` becomes `retweet`, then `post` becomes
//! `tweet`. A pair whose legacy spelling the spec also declares is excluded:
//! that spelling is a current field name wherever the spec declares it.

use std::collections::BTreeSet;

use serde_json::Value;

/// A property name the spec declares and the spelling X used before it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair {
    /// The spelling before the rename.
    pub legacy: String,
    /// The name the spec declares.
    pub current: String,
}

/// The derived pairs, each list ordered by current name.
#[derive(Debug, Default)]
pub struct Vocabulary {
    /// Pairs whose legacy spelling no spec object declares.
    pub admitted: Vec<Pair>,
    /// Pairs whose legacy spelling the spec declares as a current name.
    pub excluded: Vec<Pair>,
}

/// One `properties` object: its JSON pointer and the names it declares.
type Declaration = (String, BTreeSet<String>);

/// Derives the admitted and excluded pairs from `spec`.
///
/// # Errors
///
/// Names, by JSON pointer, every `properties` object that declares both
/// spellings of a pair. The decoder reads an object carrying both as the
/// rename caught mid-migration and drops the legacy key, which is wrong for
/// an object whose schema declares both as distinct fields.
pub fn derive(spec: &Value) -> Result<Vocabulary, String> {
    let mut objects = Vec::new();
    collect(spec, "", &mut objects);
    let declared: BTreeSet<&str> = objects
        .iter()
        .flat_map(|(_, names)| names.iter().map(String::as_str))
        .collect();

    let pairs: Vec<Pair> = declared
        .iter()
        .filter(|name| name.contains("post"))
        .map(|current| Pair {
            legacy: current
                .replace("repost", "retweet")
                .replace("post", "tweet"),
            current: (*current).to_string(),
        })
        .collect();

    let conflicts: Vec<String> = objects
        .iter()
        .flat_map(|(pointer, names)| {
            pairs
                .iter()
                .filter(|pair| names.contains(&pair.legacy) && names.contains(&pair.current))
                .map(move |pair| format!("{pointer} declares {} and {}", pair.legacy, pair.current))
        })
        .collect();
    if !conflicts.is_empty() {
        return Err(format!(
            "the spec declares both spellings of a legacy post-vocabulary pair in one object, \
             where the decoder would drop the legacy one as a rename: {}",
            conflicts.join("; ")
        ));
    }

    let (excluded, admitted) = pairs
        .into_iter()
        .partition(|pair| declared.contains(pair.legacy.as_str()));
    Ok(Vocabulary { admitted, excluded })
}

/// Appends every `properties` object under `node`, with its JSON pointer.
/// A key inside a `properties` object is a property name, never a keyword,
/// so a property named `properties` (GeoJSON's, on `PlaceGeo`) is walked as
/// the schema it holds.
fn collect(node: &Value, pointer: &str, out: &mut Vec<Declaration>) {
    match node {
        Value::Object(map) => {
            for (key, child) in map {
                let here = format!("{pointer}/{}", escape(key));
                match (key.as_str(), child) {
                    ("properties", Value::Object(properties)) => {
                        out.push((here.clone(), properties.keys().cloned().collect()));
                        for (name, schema) in properties {
                            collect(schema, &format!("{here}/{}", escape(name)), out);
                        }
                    }
                    _ => collect(child, &here, out),
                }
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect(item, &format!("{pointer}/{index}"), out);
            }
        }
        _ => {}
    }
}

/// Escapes one JSON pointer segment (RFC 6901).
fn escape(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}
