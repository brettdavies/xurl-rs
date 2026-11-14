//! Every top-level document `xr` writes carries a `status` the published
//! envelope schema declares.
//!
//! An agent that validates output against `schema/output.schema.json` rejects
//! a document whose `status` the schema omits. A document built by hand, with
//! `"status"` spelled in a `json!` map or inserted into one, or with a status
//! constant or struct field of its own, is a second copy of the status
//! vocabulary, and nothing ties it to the schema.
//!
//! The walk reads every non-test source line under `crates/xurl-cli/src` that
//! sets a `status` and checks the literal it gives against the statuses the
//! generated schema declares. Four shapes set one: a `"status"` key in a
//! `json!` map, an `insert("status", ...)` into a map, a `status: "..."`
//! struct field, and a `const STATUS_...: &str = "..."`. A `status` field set
//! from a constant resolves through that constant's own line; one set from a
//! function or a variable is not seen. The record streams of `webhooks
//! listen` and the statuses that only an array entry carries are the
//! exceptions, each named with its reason.

mod common;

use std::collections::BTreeSet;

use regex::Regex;
use serde_json::Value;
use xurl::cli::envelope::envelope_schema;

/// Files that write per-record documents on a stream. A record's `status`
/// names what happened to one event, not how a run ended, and no schema
/// describes these records.
const RECORD_STREAMS: &[(&str, &str)] = &[
    (
        "crates/xurl-cli/src/cli/commands/webhooks_forward.rs",
        "the `forwarded`, `forward-failed`, and `forward-incomplete` records, one per event",
    ),
    (
        "crates/xurl-cli/src/cli/commands/webhooks_listen.rs",
        "the `listening` record, the first line of the stream",
    ),
    (
        "crates/xurl-cli/src/cli/commands/webhooks_checkpoint.rs",
        "the `checkpoint-failed` record",
    ),
];

/// Statuses that only an entry inside a document's array carries, never the
/// document's own `status`: the file that sets it, the status, and why.
const NESTED_STATUSES: &[(&str, &str, &str)] = &[(
    "crates/xurl-cli/src/cli/skill_install/mod.rs",
    "skipped",
    "an entry of `installations` on `skill update --all`, for a host with nothing installed; \
     the status of that run's own document is `ok`, `error`, or `dry_run`",
)];

const ENVELOPE_SOURCE: &str = "crates/xurl-cli/src/cli/envelope.rs";
const COMMITTED_SCHEMA: &str = "schema/output.schema.json";

/// The shapes that set a status, each with the literal it gives in its one
/// capture group.
const LITERAL_SHAPES: &[&str] = &[
    r#""status"\s*:\s*"([^"]*)""#,
    r#"insert\(\s*"status"[^,]*,\s*(?:serde_json::)?(?:Value::String\()?\s*"([^"]*)""#,
    r#"\bstatus\s*:\s*"([^"]*)""#,
    r#"\bconst\s+STATUS\w*\s*:\s*&(?:'static\s+)?str\s*=\s*"([^"]*)""#,
];

/// The `status` values in a schema's `oneOf`.
fn statuses(schema: &Value) -> BTreeSet<String> {
    schema["oneOf"]
        .as_array()
        .expect("the schema has a oneOf")
        .iter()
        .map(|variant| {
            variant["properties"]["status"]["const"]
                .as_str()
                .unwrap_or_else(|| panic!("a variant with no status const: {variant}"))
                .to_string()
        })
        .collect()
}

/// The source of `file` up to its inline test module, with each line kept at
/// its number. Comment lines are blanked.
fn production_lines(source: &str) -> Vec<String> {
    common::production_source::up_to_inline_tests(source)
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("//") {
                String::new()
            } else {
                line.to_string()
            }
        })
        .collect()
}

/// One place that sets a `status`.
struct Site {
    file: String,
    line: usize,
    /// The literal given as its value, `None` when the value is not a literal
    /// on the same line.
    value: Option<String>,
}

fn sites() -> Vec<Site> {
    let key = Regex::new(r#""status"\s*:|insert\(\s*"status""#).unwrap();
    let shapes: Vec<Regex> = LITERAL_SHAPES
        .iter()
        .map(|shape| Regex::new(shape).unwrap())
        .collect();
    let mut found = Vec::new();
    for file in common::rust_files(&common::workspace_root().join("crates/xurl-cli/src")) {
        if file
            .file_stem()
            .is_some_and(|stem| stem.to_string_lossy().ends_with("_tests"))
        {
            continue;
        }
        let name = common::workspace_name(&file);
        if name == ENVELOPE_SOURCE {
            continue;
        }
        let source = std::fs::read_to_string(&file).expect("a source file is readable");
        for (index, line) in production_lines(&source).iter().enumerate() {
            let value = shapes
                .iter()
                .find_map(|shape| shape.captures(line))
                .map(|c| c[1].to_string());
            if value.is_none() && !key.is_match(line) {
                continue;
            }
            found.push(Site {
                file: name.clone(),
                line: index + 1,
                value,
            });
        }
    }
    found
}

fn is_excused(site: &Site) -> bool {
    RECORD_STREAMS.iter().any(|(path, _)| *path == site.file)
        || NESTED_STATUSES
            .iter()
            .any(|(path, status, _)| *path == site.file && site.value.as_deref() == Some(*status))
}

#[test]
fn no_document_is_built_with_a_status_the_schema_omits() {
    let declared = statuses(&envelope_schema());
    let mut problems = Vec::new();
    for site in sites().iter().filter(|site| !is_excused(site)) {
        match &site.value {
            Some(value) if declared.contains(value) => {}
            Some(value) => problems.push(format!(
                "{}:{}: sets the status `{value}`, which the envelope schema does not declare",
                site.file, site.line
            )),
            None => problems.push(format!(
                "{}:{}: sets a `status` whose value is not a string literal on that line, so the guard cannot check it",
                site.file, site.line
            )),
        }
    }
    assert!(
        problems.is_empty(),
        "{}\n\
         Why: an agent that validates output against {COMMITTED_SCHEMA} rejects a status the schema omits.\n\
         Fix: add a variant for the document to `Envelope` in {ENVELOPE_SOURCE}, build the document \
         from that variant, and regenerate the schema with the command in that file's header. \
         A stream that writes records instead of envelopes goes in RECORD_STREAMS in this file, \
         and a status that only an array entry carries goes in NESTED_STATUSES, each with its reason.",
        problems.join("\n")
    );
}

#[test]
fn every_exception_still_excuses_a_status() {
    let sites = sites();
    let files: BTreeSet<&str> = sites.iter().map(|site| site.file.as_str()).collect();
    let stale: Vec<&str> = RECORD_STREAMS
        .iter()
        .map(|(path, _)| *path)
        .filter(|path| !files.contains(path))
        .chain(
            NESTED_STATUSES
                .iter()
                .filter(|(path, status, _)| {
                    !sites
                        .iter()
                        .any(|site| site.file == *path && site.value.as_deref() == Some(*status))
                })
                .map(|(_, status, _)| *status),
        )
        .collect();
    assert!(
        stale.is_empty(),
        "these exceptions excuse no status any more: {stale:?}\n\
         Why: an exception that excuses nothing hides the next undeclared status in its file or under its name.\n\
         Fix: remove the entry from RECORD_STREAMS or NESTED_STATUSES in this file."
    );
}
