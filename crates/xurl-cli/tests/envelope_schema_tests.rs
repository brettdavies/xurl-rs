//! The envelope schema against the documents `xr` writes.
//!
//! The error variant closes its object, so a key it does not name makes the
//! schema wrong about what callers receive. The dry-run variant stays open for
//! a command's inputs, so a key it does not name passes a validator unnoticed;
//! there, each case names the command's inputs, and every other key has to be
//! declared.

mod common;

use std::path::Path;

use serde_json::Value;
use tempfile::TempDir;
use xurl::cli::envelope::{Envelope, Warning, envelope_schema};
use xurl::cli::hints::NextStep;
use xurl::cli::skill_install::InstallEnvelope;

/// The `oneOf` entry whose `status` is `status`.
fn variant<'a>(schema: &'a Value, status: &str) -> &'a Value {
    schema["oneOf"]
        .as_array()
        .expect("oneOf")
        .iter()
        .find(|variant| variant["properties"]["status"]["const"] == status)
        .unwrap_or_else(|| panic!("no {status} variant: {schema}"))
}

fn declared(variant: &Value) -> Vec<&str> {
    variant["properties"]
        .as_object()
        .expect("properties")
        .keys()
        .map(String::as_str)
        .collect()
}

/// Why `value` does not match `schema`, when it does not. Reads the parts of
/// JSON Schema the generated envelope uses.
fn violation(value: &Value, schema: &Value, root: &Value) -> Option<String> {
    if let Some(reference) = schema["$ref"].as_str() {
        let name = reference.strip_prefix("#/$defs/").expect("a local $ref");
        return violation(value, &root["$defs"][name], root);
    }
    for key in ["anyOf", "oneOf"] {
        if let Some(options) = schema[key].as_array() {
            let matched = options
                .iter()
                .any(|option| violation(value, option, root).is_none());
            return (!matched).then(|| format!("{value} matches no option of {schema}"));
        }
    }
    if let Some(expected) = schema.get("const")
        && expected != value
    {
        return Some(format!("{value} is not {expected}"));
    }
    if let Some(allowed) = schema["enum"].as_array()
        && !allowed.contains(value)
    {
        return Some(format!("{value} is not in {allowed:?}"));
    }
    let names = match &schema["type"] {
        Value::String(name) => vec![name.as_str()],
        Value::Array(names) => names.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    };
    if !names.is_empty() && !names.iter().any(|name| is_type(value, name)) {
        return Some(format!("{value} is not of type {names:?}"));
    }
    if let Value::Array(items) = value
        && let Some(item_schema) = schema.get("items")
    {
        return items
            .iter()
            .find_map(|item| violation(item, item_schema, root));
    }
    let Value::Object(map) = value else {
        return None;
    };
    for required in schema["required"].as_array().into_iter().flatten() {
        let name = required.as_str().expect("a required key");
        if !map.contains_key(name) {
            return Some(format!("missing the required key `{name}`: {value}"));
        }
    }
    for (key, item) in map {
        match schema["properties"].get(key) {
            Some(property) => {
                if let Some(problem) = violation(item, property, root) {
                    return Some(format!("`{key}`: {problem}"));
                }
            }
            None if schema["additionalProperties"] == false => {
                return Some(format!("the schema closes its object and names no `{key}`"));
            }
            None => {}
        }
    }
    None
}

fn is_type(value: &Value, name: &str) -> bool {
    match name {
        "null" => value.is_null(),
        "boolean" => value.is_boolean(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "string" => value.is_string(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        other => panic!("a type this reader does not know: {other}"),
    }
}

/// `xr <args>` under `--output json`, against the store `store` makes in a
/// scratch directory and an API at a closed port. Dry runs send nothing, and
/// the port is why a run that tried would fail rather than reach X.
fn dry_run_at(store: fn(&Path) -> std::path::PathBuf, args: &[&str]) -> Value {
    let tmp = TempDir::new().expect("tempdir");
    let output = common::xr_with_store(&store(tmp.path()))
        .env("API_BASE_URL", "http://127.0.0.1:9")
        .args(["--output", "json", "--dry-run"])
        .args(args)
        .output()
        .expect("run xr");
    assert_eq!(output.status.code(), Some(0), "{args:?}: {output:?}");
    serde_json::from_slice(&output.stdout).expect("a JSON envelope")
}

fn empty_store(dir: &Path) -> std::path::PathBuf {
    dir.join(".xurl")
}

/// Each case is a dry run the documents describe, with the inputs its context
/// carries. Everything else in the envelope is the envelope's own, and the
/// dry-run variant has to say so.
#[rstest::rstest]
#[case::credential_and_confirmation(common::oauth2_store, &["delete", "1234567890"], &["post_id"])]
#[case::no_credential(empty_store, &["whoami"], &[])]
#[case::invalid_input(empty_store, &["post", ""], &["body", "media_ids"])]
#[case::invalid_input_with_a_value(
    empty_store,
    &["webhooks", "listen", "--forward", "https://example.com/hook", "-H", "X-Token abc"],
    &["bind", "forward", "path", "secret"],
)]
#[case::scheme_not_accepted(common::oauth2_store, &["usage"], &[])]
#[case::sign_in_refused(
    empty_store,
    &["auth", "oauth2", "--no-browser", "--step", "1"],
    &["no_browser", "step", "username"],
)]
fn a_real_dry_run_declares_every_key_but_the_commands_inputs(
    #[case] store: fn(&Path) -> std::path::PathBuf,
    #[case] args: &[&str],
    #[case] inputs: &[&str],
) {
    let schema = envelope_schema();
    let dry_run = variant(&schema, "dry_run");
    let envelope = dry_run_at(store, args);

    assert_eq!(violation(&envelope, dry_run, &schema), None, "{envelope}");

    let names = declared(dry_run);
    let undeclared: Vec<&String> = envelope
        .as_object()
        .expect("an object")
        .keys()
        .filter(|key| !inputs.contains(&key.as_str()) && !names.contains(&key.as_str()))
        .collect();
    assert!(
        undeclared.is_empty(),
        "the dry-run variant of schema/output.schema.json does not declare {undeclared:?}: {envelope}"
    );
}

/// The JSON documents a golden fixture holds, by status. A fixture records one
/// real run, and the golden test holds it equal to the binary.
fn fixture_documents(status: &str) -> Vec<(String, Value)> {
    let dir = common::workspace_root().join("crates/xurl-cli/tests/golden");
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("the golden fixtures") {
        let path = entry.expect("entry").path();
        let name = path.file_name().expect("a name").display().to_string();
        let text = std::fs::read_to_string(&path).expect("a fixture");
        let (_, streams) = text.split_once("--- stdout ---").expect("a stdout section");
        let (stdout, stderr) = streams
            .split_once("--- stderr ---")
            .expect("a stderr section");
        for stream in [stdout, stderr] {
            if let Ok(document) = serde_json::from_str::<Value>(stream.trim())
                && document["status"] == status
            {
                found.push((name.clone(), document));
            }
        }
    }
    assert!(
        found.len() > 30,
        "too few `{status}` fixtures: {}",
        found.len()
    );
    found
}

#[test]
fn every_dry_run_fixture_matches_what_the_schema_declares() {
    let schema = envelope_schema();
    let dry_run = variant(&schema, "dry_run");
    for (name, document) in fixture_documents("dry_run") {
        assert_eq!(violation(&document, dry_run, &schema), None, "{name}");
    }
}

#[test]
fn every_error_fixture_stays_inside_the_closed_error_variant() {
    let schema = envelope_schema();
    let error = variant(&schema, "error");
    for (name, document) in fixture_documents("error") {
        assert_eq!(violation(&document, error, &schema), None, "{name}");
    }
}

/// Every key the skill verbs can put on an error, the ones no fixture shows
/// among them. A field added to the struct has to be filled in here.
#[test]
fn every_key_a_skill_verb_can_carry_is_declared_on_the_error_variant() {
    let schema = envelope_schema();
    let error = variant(&schema, "error");
    let envelope = InstallEnvelope {
        action: "skill-update",
        host: "codex",
        install_dir: "/new/xurl-rs".to_string(),
        command_preview: "git clone --depth 1 URL /new/xurl-rs".to_string(),
        destination_status: "non-empty-dir",
        status: "error",
        would_succeed: Some(false),
        exit_code: Some(1),
        reason: Some("destination-not-empty"),
        legacy_install_dir: Some("/old/xurl-rs".to_string()),
        refused_dir: Some("/old/xurl-rs".to_string()),
        message: Some("refused".to_string()),
        next_step: Some(NextStep::fix_input(None)),
    };
    let document = serde_json::to_value(&envelope).expect("an envelope");

    assert_eq!(violation(&document, error, &schema), None, "{document}");
}

/// Each case is a document an emitter writes, spelled key by key beside the
/// envelope that builds it. The document has to equal what the envelope
/// serializes to and has to match the variant the schema declares for it.
#[test]
fn cancelled_and_awaiting_callback_documents_match_their_variants() {
    let schema = envelope_schema();
    let cases = [
        (
            "cancelled",
            Envelope::Cancelled {
                exit_code: Some(1),
                message: Some("cancelled at the prompt".to_string()),
                reason: None,
                context: serde_json::json!({"command": "auth-clear", "app": "work", "all": true})
                    .as_object()
                    .cloned()
                    .expect("an object"),
                warnings: Some(vec![Warning {
                    message: "a value was rejected".to_string(),
                    target: "xdk::config".to_string(),
                }]),
            },
            serde_json::json!({
                "status": "cancelled",
                "exit_code": 1,
                "message": "cancelled at the prompt",
                "command": "auth-clear",
                "app": "work",
                "all": true,
                "warnings": [{"message": "a value was rejected", "target": "xdk::config"}],
            }),
        ),
        (
            "cancelled",
            Envelope::Cancelled {
                exit_code: None,
                message: None,
                reason: Some("sigterm".to_string()),
                context: serde_json::Map::new(),
                warnings: None,
            },
            serde_json::json!({"status": "cancelled", "reason": "sigterm"}),
        ),
        (
            "awaiting_callback",
            Envelope::AwaitingCallback {
                url: "https://x.com/i/oauth2/authorize?state=s".to_string(),
                instructions: "Open the URL in a browser, authorize, then run 'xr auth oauth2 --step 2 <redirect-url>'".to_string(),
            },
            serde_json::json!({
                "status": "awaiting_callback",
                "url": "https://x.com/i/oauth2/authorize?state=s",
                "instructions": "Open the URL in a browser, authorize, then run 'xr auth oauth2 --step 2 <redirect-url>'",
            }),
        ),
    ];
    for (status, envelope, written) in cases {
        let document = envelope.into_value();
        assert_eq!(document, written);
        assert_eq!(
            violation(&document, variant(&schema, status), &schema),
            None,
            "{document}"
        );
    }
}
