//! Integration tests for the `xr validate` subcommand: the alias set the
//! `--schema` help advertises, the set the `unknown-schema` envelope lists,
//! and the validators behind them all read one declaration.

mod common;

use std::collections::BTreeSet;

use predicates::prelude::*;

const VALIDATE_SOURCE: &str = "crates/xurl-cli/src/cli/commands/validate.rs";

/// The names the `--schema` option's help text lists, from `xr validate --help`.
fn advertised_schemas() -> BTreeSet<String> {
    let output = common::xr().args(["validate", "--help"]).output().unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    let start = help
        .find("--schema <NAME>")
        .expect("`xr validate --help` documents --schema");
    let block: String = help[start..]
        .lines()
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    regex::Regex::new(r"`([a-z-]+)`")
        .unwrap()
        .captures_iter(&block)
        .map(|c| c[1].to_string())
        .collect()
}

/// The names the `unknown-schema` envelope lists as `known_schemas`.
fn known_schemas() -> BTreeSet<String> {
    let output = common::xr()
        .args(["--output", "json", "validate", "--schema", "bogus"])
        .write_stdin("{}")
        .output()
        .unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&output.stderr)
        .unwrap_or_else(|e| panic!("stderr is not a JSON envelope: {e}"));
    assert_eq!(envelope["reason"], "unknown-schema");
    envelope["known_schemas"]
        .as_array()
        .expect("known_schemas is an array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn schema_help_and_unknown_schema_envelope_list_the_same_aliases() {
    let advertised = advertised_schemas();
    let known = known_schemas();
    let unadvertised: Vec<&String> = known.difference(&advertised).collect();
    let unwired: Vec<&String> = advertised.difference(&known).collect();
    assert!(
        unadvertised.is_empty() && unwired.is_empty(),
        "`xr validate --help` and the unknown-schema envelope disagree about the aliases.\n\
         Wired but not advertised: {unadvertised:?}\n\
         Advertised but not wired: {unwired:?}\n\
         Cause: the --schema help text and the alias catalog are written separately, so one \
         gained an alias the other did not.\n\
         Fix: both must read SCHEMA_ALIASES in {VALIDATE_SOURCE}; add the alias there and \
         re-bless help-validate.golden and reason-unknown-schema.golden."
    );
}

#[test]
fn every_known_schema_dispatches_to_a_validator() {
    for schema in known_schemas() {
        let output = common::xr()
            .args(["--output", "json", "validate", "--schema", &schema])
            .write_stdin("{}")
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("unknown-schema") && !stderr.contains("no validator"),
            "`xr validate --schema {schema}` accepted the name but did not dispatch it:\n{stderr}\n\
             Cause: the alias is in the catalog with no validator behind it.\n\
             Fix: give the alias a validator in SCHEMA_ALIASES in {VALIDATE_SOURCE}."
        );
    }
}

#[test]
fn moderators_schema_accepts_a_chat_moderators_response() {
    let fixtures: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            common::workspace_root()
                .join("crates/xdk/tests/fixtures/openapi/example_responses.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let doc = fixtures["chat_moderators"].to_string();
    common::xr()
        .args(["--output", "json", "validate", "--schema", "moderators"])
        .write_stdin(doc)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema\": \"moderators\""))
        .stdout(predicate::str::contains("\"valid\": true"));
}

#[test]
fn moderators_schema_rejects_a_post() {
    common::xr()
        .args(["--output", "json", "validate", "--schema", "moderators"])
        .write_stdin(r#"{"data":{"id":"1","text":"hi"}}"#)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("validation-failed"));
}

/// The errors recorded in the golden fixtures, on either stream and from any
/// command, by fixture name. The golden test holds each equal to the binary.
fn error_fixtures() -> Vec<(String, String)> {
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
            let document = stream.trim();
            if serde_json::from_str::<serde_json::Value>(document)
                .is_ok_and(|value| value["status"] == "error")
            {
                found.push((name.clone(), document.to_string()));
            }
        }
    }
    found.sort();
    found
}

/// Every structured error carries a `reason`, an `exit_code`, and a `message`,
/// so the validator an agent runs accepts what `xr` writes, whichever command
/// wrote it.
#[test]
fn every_error_fixture_passes_the_envelope_validator() {
    let fixtures = error_fixtures();
    assert!(fixtures.len() > 30, "the errors: {}", fixtures.len());
    let refused: Vec<String> = fixtures
        .into_iter()
        .filter_map(|(name, document)| {
            let output = common::xr()
                .args(["--output", "json", "validate", "--schema", "envelope"])
                .write_stdin(document)
                .output()
                .expect("run xr");
            let stderr = String::from_utf8_lossy(&output.stderr);
            let message = serde_json::from_str::<serde_json::Value>(stderr.trim())
                .map_or_else(|_| stderr.to_string(), |v| v["message"].to_string());
            (!output.status.success()).then(|| format!("{name}: {message}"))
        })
        .collect();
    assert!(
        refused.is_empty(),
        "`xr validate --schema envelope` refused:\n{}",
        refused.join("\n")
    );
}
