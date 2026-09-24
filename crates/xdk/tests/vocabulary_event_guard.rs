//! The legacy-vocabulary event carries schema identifiers only: the two
//! spellings from the spec-derived table, the JSON type and length of the
//! value X sent, and whether the rename collided. A value, an id, or a
//! request path at the emit site would copy post text, handles, or DM
//! contents into every subscriber's output, so the emit site and the
//! types of what it records are pinned here rather than left to review.

use std::path::Path;

/// The module that emits the event, relative to the crate root.
const EMITTER: &str = "src/api/response/vocabulary.rs";

/// The one emit site. Every comparison below ignores whitespace, so the
/// pin survives any layout rustfmt gives the macro.
const EMIT_SITE: &str = "tracing::debug!(target: VOCABULARY_TARGET, legacy, normalized, \
                         value_type, value_len, collision);";

/// The signatures that fix what each recorded name can hold: `&'static str`
/// for the table's spellings and the JSON type name, a length, and a flag.
const BINDINGS: &[&str] = &[
    "fn report(legacy: &'static str, normalized: &'static str, sent: &Value, collision: bool)",
    "let (value_type, value_len) = shape(sent);",
    "fn shape(value: &Value) -> (&'static str, Option<usize>)",
];

#[test]
fn the_vocabulary_event_records_only_the_five_schema_fields() {
    let source = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(EMITTER))
        .unwrap_or_else(|e| panic!("{EMITTER} must be readable: {e}"));
    let production = squeeze(source.split("#[cfg(test)]").next().unwrap_or_default());

    let sites: Vec<&str> = production
        .match_indices("tracing::")
        .map(|(at, _)| {
            let rest = &production[at..];
            rest.find(';').map_or(rest, |end| &rest[..=end])
        })
        .collect();
    let missing: Vec<&&str> = BINDINGS
        .iter()
        .filter(|binding| !production.contains(&squeeze(binding)))
        .collect();

    assert!(
        sites == [squeeze(EMIT_SITE)] && missing.is_empty(),
        "the vocabulary event records the two spellings, the value's JSON type and length, \
         and the collision flag, from one emit site in {EMITTER}: `{EMIT_SITE}`, bound \
         through {BINDINGS:?}. Context beyond the key pair belongs in the response struct's \
         name, which carries no instance data.\n\
         Emit sites found: {sites:?}\nBindings missing: {missing:?}"
    );
}

/// `text` with every whitespace character removed.
fn squeeze(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
