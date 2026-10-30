//! Integration tests for the `xr schema` subcommand.

mod common;

use predicates::prelude::*;

use std::collections::{BTreeMap, BTreeSet};

use rstest::rstest;
use xurl::cli::commands::schema::{
    SCHEMA_LESS_COMMANDS, file_stem, registered_commands, registered_types, scope_words,
};
use xurl::cli::commands::validate::{UNVALIDATED_TYPES, validated_types};

const SCHEMA_SOURCE: &str = "crates/xurl-cli/src/cli/commands/schema/mod.rs";
const RUN_SOURCE: &str = "crates/xurl-cli/src/cli/commands/schema/run.rs";

// ═══════════════════════════════════════════════════════════════════════════
// Single command schema
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn schema_post_outputs_valid_json_schema() {
    let output = common::xr().args(["schema", "post"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    // Must have standard JSON Schema fields
    assert!(json.get("$defs").is_some() || json.get("definitions").is_some());
    assert!(json.get("properties").is_some());
    assert_eq!(json["type"], "object");
}

#[test]
fn schema_post_contains_post_fields() {
    let output = common::xr().args(["schema", "post"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"text\""));
    assert!(stdout.contains("\"author_id\""));
    assert!(stdout.contains("\"includes\""));
    assert!(stdout.contains("\"meta\""));
}

#[test]
fn schema_whoami_contains_user_fields() {
    let output = common::xr().args(["schema", "whoami"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"username\""));
    assert!(stdout.contains("\"name\""));
}

#[test]
fn schema_like_contains_liked_field() {
    let output = common::xr().args(["schema", "like"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"liked\""));
}

#[test]
fn schema_no_extra_named_property() {
    // #[serde(flatten)] BTreeMap should produce additionalProperties, not a named "extra" field
    let output = common::xr().args(["schema", "post"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let props = json["properties"].as_object().unwrap();
    assert!(
        !props.contains_key("extra"),
        "extra should not appear as a named property"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// --list flag
// ═══════════════════════════════════════════════════════════════════════════

/// The records `xr schema --list --output json` prints.
fn list_records() -> Vec<serde_json::Value> {
    let output = common::xr()
        .args(["schema", "--list", "--output", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("`xr schema --list --output json` is one JSON value")
        .as_array()
        .expect("an array of records")
        .clone()
}

#[test]
fn schema_list_records_every_registered_command_and_nothing_else() {
    let records = list_records();
    let listed: BTreeSet<String> = records
        .iter()
        .map(|record| record["command"].as_str().expect("a command").to_string())
        .collect();
    let registry: BTreeSet<String> = registered_commands().map(str::to_string).collect();
    let missing: Vec<&String> = registry.difference(&listed).collect();
    assert!(
        missing.is_empty(),
        "`xr schema --list` omits registered commands: {missing:?}\n\
         Cause: print_schema_list in {RUN_SOURCE} no longer walks every SCHEMA_ENTRIES row.\n\
         Fix: make the listing read SCHEMA_ENTRIES in full; a command reaches `--list` by \
         being in the registry, never by being listed again."
    );
    let extra: Vec<&String> = listed.difference(&registry).collect();
    assert!(
        extra.is_empty(),
        "`xr schema --list` advertises commands with no registry row: {extra:?}\n\
         Cause: print_schema_list in {RUN_SOURCE} prints a name that is not in SCHEMA_ENTRIES.\n\
         Fix: add the command's row to SCHEMA_ENTRIES or drop it from the listing."
    );
    assert_eq!(
        records.len(),
        registry.len(),
        "one record per registered command"
    );
    for record in &records {
        let command = record["command"].as_str().unwrap_or_default();
        assert_eq!(
            record["file"],
            format!("{}.schema.json", file_stem(command)),
            "the record for {command} names its file"
        );
        assert!(
            record["type"].is_string(),
            "the record for {command} names its type"
        );
    }
}

#[test]
fn schema_list_tsv_has_a_header_and_one_row_per_command() {
    let output = common::xr()
        .args(["schema", "--list", "--output", "tsv"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut lines = stdout.lines();
    assert_eq!(lines.next(), Some("command\tfile\ttype"));
    assert_eq!(lines.count(), registered_commands().count());
}

#[test]
fn schema_list_contains_expected_commands() {
    let output = common::xr().args(["schema", "--list"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = [
        "post",
        "reply",
        "quote",
        "read",
        "search",
        "timeline",
        "mentions",
        "bookmarks",
        "likes",
        "whoami",
        "user",
        "following",
        "followers",
        "like",
        "unlike",
        "follow",
        "unfollow",
        "delete",
        "repost",
        "unrepost",
        "bookmark",
        "unbookmark",
        "mute",
        "unmute",
        "muted",
        "block",
        "unblock",
        "blocked",
        "dm",
        "dms",
        "usage",
        "usage credits",
    ];
    for cmd in expected {
        assert!(stdout.contains(cmd), "--list output missing command: {cmd}");
    }
}

#[test]
fn schema_list_shows_type_names() {
    let output = common::xr().args(["schema", "--list"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ApiResponse<Post>"));
    assert!(stdout.contains("ApiResponse<Vec<Post>>"));
    assert!(stdout.contains("ApiResponse<User>"));
    assert!(stdout.contains("ApiResponse<LikedResult>"));
}

// ═══════════════════════════════════════════════════════════════════════════
// --all flag
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn schema_all_emits_one_schema_per_registered_command() {
    let output = common::xr().args(["schema", "--all"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let obj = json.as_object().unwrap();

    let emitted: BTreeSet<&str> = obj.keys().map(String::as_str).collect();
    let registry: BTreeSet<&str> = registered_commands().collect();
    let missing: Vec<&&str> = registry.difference(&emitted).collect();
    assert!(
        missing.is_empty(),
        "`xr schema --all` omits registered commands: {missing:?}\n\
         Cause: print_all_schemas in {RUN_SOURCE} no longer walks every SCHEMA_ENTRIES row.\n\
         Fix: make `--all` read SCHEMA_ENTRIES in full; a command reaches `--all` by being in \
         the registry, never by being listed again."
    );
    let extra: Vec<&&str> = emitted.difference(&registry).collect();
    assert!(
        extra.is_empty(),
        "`xr schema --all` emits schemas for commands with no registry row: {extra:?}\n\
         Cause: print_all_schemas in {RUN_SOURCE} emits a key that is not in SCHEMA_ENTRIES.\n\
         Fix: add the command's row to SCHEMA_ENTRIES or drop it from `--all`."
    );
    // Each value should be a valid schema object — either an object schema
    // with `properties` or an array schema with `items`.
    for (cmd, schema) in obj {
        let shape_ok = schema.get("properties").is_some() || schema.get("items").is_some();
        assert!(shape_ok, "Schema for '{cmd}' missing properties or items");
    }
}

#[test]
fn schema_all_takes_precedence_over_list() {
    // When both --all and --list are provided, --all takes precedence
    let output = common::xr()
        .args(["schema", "--all", "--list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    // Should be JSON (--all), not plain text (--list)
    let _: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
// Error cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn schema_unknown_command_fails() {
    common::xr()
        .args(["schema", "bogus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown command 'bogus'"));
}

#[test]
fn schema_auth_not_available() {
    common::xr()
        .args(["schema", "auth"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema not available"));
}

#[test]
fn schema_media_not_available() {
    common::xr()
        .args(["schema", "media"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema not available"));
}

#[test]
fn schema_completions_not_available() {
    common::xr()
        .args(["schema", "completions"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema not available"));
}

#[test]
fn schema_version_not_available() {
    common::xr()
        .args(["schema", "version"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema not available"));
}

#[test]
fn schema_no_args_shows_usage() {
    common::xr()
        .arg("schema")
        .assert()
        .failure()
        .stderr(predicate::str::contains("usage: xr schema"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Command mapping correctness
// ═══════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════
// Envelope schema + drift guard
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn envelope_schema_is_draft_2020_12_with_oneof() {
    // `xr schema --envelope` emits the canonical envelope JSON Schema; agents
    // pin against `$schema` + the three-variant `oneOf`.
    let output = common::xr()
        .args(["schema", "--envelope"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        parsed["$schema"], "https://json-schema.org/draft/2020-12/schema",
        "envelope schema must declare Draft 2020-12"
    );
    let variants = parsed["oneOf"]
        .as_array()
        .expect("envelope schema has oneOf array");
    assert_eq!(
        variants.len(),
        3,
        "envelope has three variants: ok, dry_run, error"
    );
}

#[test]
fn envelope_as_a_word_is_refused_with_the_flag_as_its_step() {
    let output = common::xr()
        .args(["--output", "json", "schema", "envelope"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let envelope: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["reason"], "invalid-args");
    assert_eq!(envelope["command"], "envelope");
    assert_eq!(envelope["next_step"]["action"], "fix-input");
    assert_eq!(envelope["next_step"]["command"], "xr schema --envelope");
}

#[test]
fn committed_envelope_schema_matches_runtime() {
    // Drift guard: schema/output.schema.json must match the runtime-emitted
    // schema byte-for-byte. Regenerate via:
    //   cargo run --bin xr -- schema --envelope --output json > schema/output.schema.json
    let output = common::xr()
        .args(["schema", "--envelope", "--output", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let runtime = String::from_utf8(output.stdout).unwrap();
    let committed = std::fs::read_to_string(concat!(
        env!("CARGO_WORKSPACE_DIR"),
        "/schema/output.schema.json"
    ))
    .expect("schema/output.schema.json must be committed at the repo root");
    assert_eq!(
        runtime.trim(),
        committed.trim(),
        "schema/output.schema.json drifted; regenerate with: cargo run --bin xr -- schema --envelope --output json > schema/output.schema.json"
    );
}

#[test]
fn committed_response_schemas_match_runtime() {
    // Drift guard: every schema/responses/<file> must match the runtime-emitted
    // shape for its command. Regenerate via:
    //   ./scripts/generate-response-schemas.sh
    let dir = concat!(env!("CARGO_WORKSPACE_DIR"), "/schema/responses");
    let mut checked = 0usize;
    for record in list_records() {
        let command = record["command"].as_str().expect("a command");
        let file = record["file"].as_str().expect("a file");
        let mut args = vec!["schema", "--output", "json", "--"];
        args.extend(command.split_whitespace());
        let runtime = common::xr().args(&args).output().unwrap();
        assert!(
            runtime.status.success(),
            "xr schema {command} exited non-zero"
        );
        let runtime_body = String::from_utf8(runtime.stdout).unwrap();
        let committed = std::fs::read_to_string(format!("{dir}/{file}")).unwrap_or_else(|_| {
            panic!(
                "schema/responses/{file} missing; regenerate with: ./scripts/generate-response-schemas.sh"
            )
        });
        assert_eq!(
            runtime_body.trim(),
            committed.trim(),
            "schema/responses/{file} drifted; regenerate with: ./scripts/generate-response-schemas.sh"
        );
        checked += 1;
    }
    assert!(checked > 0, "no per-command schemas were checked");
}

#[test]
fn committed_response_schemas_have_no_orphans() {
    // The generator only writes files for commands in SCHEMA_ENTRIES; a
    // command removed from the list leaves its committed schema behind.
    let known: BTreeSet<String> = registered_commands()
        .map(|command| format!("{}.schema.json", file_stem(command)))
        .collect();
    let dir = concat!(env!("CARGO_WORKSPACE_DIR"), "/schema/responses");
    let orphans: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| !known.contains(name))
        .collect();
    assert!(
        orphans.is_empty(),
        "schema/responses/ has files for commands the registry does not know: {orphans:?}; delete them"
    );
}

#[test]
fn every_registry_row_derives_its_own_file() {
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    let mut shared = Vec::new();
    for command in registered_commands() {
        if let Some(other) = seen.insert(file_stem(command), command) {
            shared.push(format!("{other} and {command}"));
        }
    }
    assert!(
        shared.is_empty(),
        "these registry rows derive the same schema/responses/ file: {shared:?}\n\
         Cause: two command paths join with `-` to the same name, so one row's committed \
         schema would overwrite the other's.\n\
         Fix: in {SCHEMA_SOURCE}, file_stem must tell the two apart, or one command is renamed."
    );
}

#[test]
fn schema_commands_sharing_type_produce_identical_output() {
    // post, reply, quote, read should all return the same schema
    let post = common::xr().args(["schema", "post"]).output().unwrap();
    let reply = common::xr().args(["schema", "reply"]).output().unwrap();
    assert_eq!(
        post.stdout, reply.stdout,
        "post and reply should share the same schema"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Registry exhaustiveness: clap's command tree is the source of truth
// ═══════════════════════════════════════════════════════════════════════════

/// Every command clap parses, by the words `xr schema` and `xr help` take.
fn clap_schema_names() -> BTreeSet<String> {
    common::command_paths()
        .iter()
        .map(|path| path.join(" "))
        .collect()
}

#[test]
fn every_clap_command_is_in_exactly_one_schema_set() {
    let registry: BTreeSet<&str> = registered_commands().collect();
    let schema_less: BTreeSet<&str> = SCHEMA_LESS_COMMANDS.iter().copied().collect();
    let names = clap_schema_names();

    let in_neither: Vec<&str> = names
        .iter()
        .map(String::as_str)
        .filter(|name| !registry.contains(name) && !schema_less.contains(name))
        .collect();
    assert!(
        in_neither.is_empty(),
        "`xr schema` reports these commands as unknown, though clap parses them: {in_neither:?}\n\
         Cause: each is neither a SCHEMA_ENTRIES row nor named in SCHEMA_LESS_COMMANDS, so the \
         lookup falls through to the unknown-command error.\n\
         Fix: in {SCHEMA_SOURCE}, add a SCHEMA_ENTRIES row when the command emits a typed \
         response (then run scripts/generate-response-schemas.sh), or add its name to \
         SCHEMA_LESS_COMMANDS when it does not."
    );

    let in_both: Vec<&&str> = registry.intersection(&schema_less).collect();
    assert!(
        in_both.is_empty(),
        "these commands are both registered and declared schema-less: {in_both:?}\n\
         Cause: a name appears in a SCHEMA_ENTRIES row and in SCHEMA_LESS_COMMANDS, so which \
         answer `xr schema` gives depends on lookup order.\n\
         Fix: in {SCHEMA_SOURCE}, keep each name in exactly one of the two."
    );
}

#[test]
fn declared_schema_sets_name_only_commands_clap_parses() {
    let names = clap_schema_names();

    let stale_registry: Vec<&str> = registered_commands()
        .filter(|name| !names.contains(*name))
        .collect();
    assert!(
        stale_registry.is_empty(),
        "SCHEMA_ENTRIES names commands clap does not parse: {stale_registry:?}\n\
         Cause: the command was renamed or removed from the clap tree, and the row still \
         names it by its old words.\n\
         Fix: in {SCHEMA_SOURCE}, drop the row or rename it to the command's words as \
         `xr help` takes them; then run scripts/generate-response-schemas.sh."
    );

    let stale_schema_less: Vec<&&str> = SCHEMA_LESS_COMMANDS
        .iter()
        .filter(|name| !names.contains(**name))
        .collect();
    assert!(
        stale_schema_less.is_empty(),
        "SCHEMA_LESS_COMMANDS names commands clap does not parse: {stale_schema_less:?}\n\
         Cause: the command was renamed or removed from the clap tree.\n\
         Fix: in {SCHEMA_SOURCE}, drop each stale name from SCHEMA_LESS_COMMANDS."
    );
}

#[test]
fn registry_names_each_command_once() {
    let mut seen = BTreeSet::new();
    let repeated: Vec<&str> = registered_commands()
        .filter(|name| !seen.insert(*name))
        .collect();
    assert!(
        repeated.is_empty(),
        "SCHEMA_ENTRIES names these commands in more than one row: {repeated:?}\n\
         Cause: two rows list the same command, so `xr schema --list` prints it twice and \
         `xr schema <command>` answers from whichever row comes first.\n\
         Fix: in {SCHEMA_SOURCE}, keep the command in the row whose type it returns."
    );
}

#[rstest]
#[case(&["skill"])]
#[case(&["examples"])]
#[case(&["validate"])]
#[case(&["auth", "oauth2"])]
#[case(&["media", "upload"])]
#[case(&["skill", "install"])]
#[case(&["skill", "update"])]
fn schema_less_command_is_not_reported_unknown(#[case] words: &[&str]) {
    common::xr()
        .arg("schema")
        .args(words)
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema not available"))
        .stderr(predicate::str::contains("unknown command").not());
}

#[test]
fn a_family_without_a_schema_names_the_commands_beneath_it_that_have_one() {
    common::xr()
        .args(["schema", "webhooks"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "schema not available for 'webhooks'",
        ))
        .stderr(predicate::str::contains("webhooks subscriptions count"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Command words: the words `xr help` takes, and what a wrong word gets
// ═══════════════════════════════════════════════════════════════════════════

/// `xr --output json schema <args>`: the exit code and the envelope on stderr.
fn schema_refusal(args: &[&str]) -> (Option<i32>, serde_json::Value) {
    let output = common::xr()
        .args(["--output", "json", "schema"])
        .args(args)
        .output()
        .unwrap();
    let envelope = serde_json::from_slice(&output.stderr).unwrap_or(serde_json::Value::Null);
    (output.status.code(), envelope)
}

#[rstest]
#[case(&["webhooks", "subscriptions", "count"], "webhooks-subscriptions-count")]
#[case(&["auth", "apps", "redirect-uri", "get"], "auth-apps-redirect-uri-get")]
#[case(&["spaces", "by-creator"], "spaces-by-creator")]
#[case(&["search-users"], "search-users")]
fn a_command_is_named_by_its_words(#[case] words: &[&str], #[case] file: &str) {
    let output = common::xr()
        .args(["schema", "--output", "json"])
        .args(words)
        .output()
        .unwrap();
    assert!(output.status.success(), "xr schema {words:?}");
    let committed = std::fs::read_to_string(format!(
        "{}/schema/responses/{file}.schema.json",
        env!("CARGO_WORKSPACE_DIR")
    ))
    .unwrap();
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        committed.trim()
    );
}

#[test]
fn a_mistyped_word_gets_the_refusal_xr_help_gives_it() {
    let (code, envelope) = schema_refusal(&["webhooks", "subscriptoins", "count"]);
    assert_eq!(code, Some(2));
    assert_eq!(envelope["reason"], "unknown-command");
    assert_eq!(envelope["command"], "subscriptoins");
    assert_eq!(envelope["suggestion"], "subscriptions");
    assert_eq!(envelope["next_step"]["action"], "show-help");

    let (code, envelope) = schema_refusal(&["post", "extra"]);
    assert_eq!(code, Some(2));
    assert_eq!(envelope["reason"], "unknown-command");
    assert_eq!(envelope["command"], "extra");
}

#[rstest]
#[case(
    "webhooks-subscriptions-count",
    "xr schema webhooks subscriptions count"
)]
#[case("redirect-uri-get", "xr schema auth apps redirect-uri get")]
fn an_old_name_is_refused_with_the_words_that_work(#[case] old: &str, #[case] run: &str) {
    let (code, envelope) = schema_refusal(&[old]);
    assert_eq!(code, Some(2));
    assert_eq!(envelope["reason"], "invalid-args");
    assert_eq!(envelope["command"], old);
    assert_eq!(envelope["next_step"]["action"], "fix-input");
    assert_eq!(envelope["next_step"]["command"], run);

    let words: Vec<&str> = run.split_whitespace().skip(1).collect();
    let rerun = common::xr().args(&words).output().unwrap();
    assert!(rerun.status.success(), "the step `{run}` runs");
}

#[test]
fn no_top_level_command_shares_a_name_with_a_scope_flag() {
    let scopes = scope_words();
    assert!(
        !scopes.is_empty(),
        "the schema subcommand declares its scope flags"
    );
    let clashing: Vec<String> = common::top_level_families()
        .into_iter()
        .filter(|family| scopes.contains(family))
        .collect();
    assert!(
        clashing.is_empty(),
        "these top-level commands share a name with an `xr schema` flag: {clashing:?}; \
         `xr schema <name>` refuses the word as a scope, so the command's schema would be unreachable"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Every registry response type has a validator or a declared exemption
// ═══════════════════════════════════════════════════════════════════════════

const VALIDATE_SOURCE: &str = "crates/xurl-cli/src/cli/commands/validate.rs";

#[test]
fn every_registry_response_type_has_a_validator_or_a_declared_exemption() {
    let registry: BTreeSet<&str> = registered_types().collect();
    let validated: BTreeSet<String> = validated_types().collect();
    let exempt: BTreeSet<&str> = UNVALIDATED_TYPES.iter().map(|(name, _)| *name).collect();

    let uncovered: Vec<&&str> = registry
        .iter()
        .filter(|name| !validated.contains(**name) && !exempt.contains(*name))
        .collect();
    assert!(
        uncovered.is_empty(),
        "these registry response types have no `xr validate` alias and no exemption: {uncovered:?}\n\
         Cause: a schema row names a response type that SCHEMA_ALIASES does not validate and \
         UNVALIDATED_TYPES does not exempt, so agents can print the shape but never check a \
         document against it.\n\
         Fix: in {VALIDATE_SOURCE}, add a typed_alias row for the type, or add it to \
         UNVALIDATED_TYPES with the reason it is not an API response; then re-bless \
         help-validate.golden and reason-unknown-schema.golden if an alias was added.\n\
         Exempt today: {exempt:?}"
    );

    let stale: Vec<&&str> = exempt
        .iter()
        .filter(|name| !registry.contains(**name))
        .collect();
    assert!(
        stale.is_empty(),
        "UNVALIDATED_TYPES exempts response types the registry no longer names: {stale:?}\n\
         Cause: the row was removed or its type_name changed while the exemption stayed.\n\
         Fix: in {VALIDATE_SOURCE}, drop each stale entry from UNVALIDATED_TYPES."
    );

    let both: Vec<&&str> = exempt
        .iter()
        .filter(|name| validated.contains(**name))
        .collect();
    assert!(
        both.is_empty(),
        "UNVALIDATED_TYPES exempts response types that already have a validator: {both:?}\n\
         Cause: an alias was added for a type still listed as unvalidatable.\n\
         Fix: in {VALIDATE_SOURCE}, drop each entry from UNVALIDATED_TYPES."
    );
}

/// Wording that only a caller of the Rust library can act on.
const RUST_LIBRARY_WORDING: &[&str] = &["serde", "Serde", "this crate", "ApiResponse<", "Vec<"];

/// Every `description` string anywhere under `value`.
fn descriptions(value: &serde_json::Value, found: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                match (key.as_str(), child) {
                    ("description", serde_json::Value::String(text)) => found.push(text.clone()),
                    _ => descriptions(child, found),
                }
            }
        }
        serde_json::Value::Array(items) => items.iter().for_each(|item| descriptions(item, found)),
        _ => {}
    }
}

/// The committed schemas describe the wire, not the Rust library behind it:
/// `xr schema` output and the schemas an embedder hands an MCP tool
/// definition are read by agents that never see serde or this crate's types.
#[test]
fn committed_schema_descriptions_carry_no_rust_library_wording() {
    let root = concat!(env!("CARGO_WORKSPACE_DIR"), "/schema");
    let mut files = vec![format!("{root}/output.schema.json")];
    for entry in
        std::fs::read_dir(format!("{root}/responses")).expect("schema/responses is readable")
    {
        files.push(
            entry
                .expect("a directory entry")
                .path()
                .display()
                .to_string(),
        );
    }
    files.sort();

    let mut hits = Vec::new();
    for file in &files {
        let raw = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file}: {e}"));
        let schema: serde_json::Value =
            serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {file}: {e}"));
        let mut found = Vec::new();
        descriptions(&schema, &mut found);
        for text in found {
            for word in RUST_LIBRARY_WORDING
                .iter()
                .filter(|word| text.contains(**word))
            {
                let name = file.rsplit('/').next().unwrap_or(file);
                hits.push(format!("{name}: {word:?} in {text:?}"));
            }
        }
    }
    hits.sort();
    hits.dedup();
    assert!(
        hits.is_empty(),
        "schema descriptions written for the Rust library rather than the wire ({} hits):\n{}\n\
         Give the type or field a `#[schemars(description = \"...\")]` that describes what X \
         sends, keep the Rust detail in its rustdoc, then run ./scripts/generate-response-schemas.sh.",
        hits.len(),
        hits.iter().take(12).cloned().collect::<Vec<_>>().join("\n")
    );
}
