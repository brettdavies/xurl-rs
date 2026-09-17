//! Integration tests for the `xr schema` subcommand.

mod common;

use predicates::prelude::*;

use std::collections::BTreeSet;

use rstest::rstest;
use xurl::cli::commands::schema::{
    FLAG_FORMS, SCHEMA_LESS_COMMANDS, registered_commands, schema_name_for_path,
};

const SCHEMA_SOURCE: &str = "crates/xurl-cli/src/cli/commands/schema.rs";

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

#[test]
fn schema_list_advertises_every_registered_command_plus_envelope() {
    let output = common::xr().args(["schema", "--list"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let listed: BTreeSet<&str> = stdout
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert!(
        listed.contains("envelope"),
        "--list should advertise the envelope schema"
    );

    let registry: BTreeSet<&str> = registered_commands().collect();
    let missing: Vec<&&str> = registry.difference(&listed).collect();
    assert!(
        missing.is_empty(),
        "`xr schema --list` omits registered commands: {missing:?}\n\
         Cause: print_schema_list in {SCHEMA_SOURCE} no longer walks every SCHEMA_ENTRIES row.\n\
         Fix: make the listing read SCHEMA_ENTRIES in full; a command reaches `--list` by \
         being in the registry, never by being listed again."
    );
    let extra: Vec<&&str> = listed
        .difference(&registry)
        .filter(|name| **name != "envelope")
        .collect();
    assert!(
        extra.is_empty(),
        "`xr schema --list` advertises commands with no registry row: {extra:?}\n\
         Cause: print_schema_list in {SCHEMA_SOURCE} prints a name that is not in SCHEMA_ENTRIES.\n\
         Fix: add the command's row to SCHEMA_ENTRIES or drop it from the listing."
    );
    assert_eq!(
        stdout.lines().count(),
        registry.len() + 1,
        "`xr schema --list` prints one row per registered command plus the envelope row"
    );
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
        "usage-credits",
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
         Cause: print_all_schemas in {SCHEMA_SOURCE} no longer walks every SCHEMA_ENTRIES row.\n\
         Fix: make `--all` read SCHEMA_ENTRIES in full; a command reaches `--all` by being in \
         the registry, never by being listed again."
    );
    let extra: Vec<&&str> = emitted.difference(&registry).collect();
    assert!(
        extra.is_empty(),
        "`xr schema --all` emits schemas for commands with no registry row: {extra:?}\n\
         Cause: print_all_schemas in {SCHEMA_SOURCE} emits a key that is not in SCHEMA_ENTRIES.\n\
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
// U5: envelope schema + drift guard
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn envelope_schema_is_draft_2020_12_with_oneof() {
    // `xr schema envelope` emits the canonical envelope JSON Schema; agents
    // pin against `$schema` + the three-variant `oneOf`.
    let output = common::xr().args(["schema", "envelope"]).output().unwrap();
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
fn envelope_schema_via_flag_equals_positional() {
    let by_pos = common::xr().args(["schema", "envelope"]).output().unwrap();
    let by_flag = common::xr()
        .args(["schema", "--envelope"])
        .output()
        .unwrap();
    assert!(by_pos.status.success());
    assert!(by_flag.status.success());
    assert_eq!(by_pos.stdout, by_flag.stdout);
}

#[test]
fn committed_envelope_schema_matches_runtime() {
    // Drift guard: schema/output.schema.json must match the runtime-emitted
    // schema byte-for-byte. Regenerate via:
    //   cargo run --bin xr -- schema envelope --output json > schema/output.schema.json
    let output = common::xr()
        .args(["schema", "envelope", "--output", "json"])
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
        "schema/output.schema.json drifted; regenerate with: cargo run --bin xr -- schema envelope --output json > schema/output.schema.json"
    );
}

#[test]
fn committed_response_schemas_match_runtime() {
    // Drift guard: every schema/responses/<cmd>.schema.json must match the
    // runtime-emitted shape for that command. Regenerate via:
    //   ./scripts/generate-response-schemas.sh
    let list = common::xr()
        .args(["schema", "--list", "--output", "text"])
        .output()
        .unwrap();
    assert!(list.status.success());
    let list_text = String::from_utf8(list.stdout).unwrap();

    let dir = concat!(env!("CARGO_WORKSPACE_DIR"), "/schema/responses");
    let mut checked = 0usize;
    for line in list_text.lines() {
        let cmd = line.split_whitespace().next().unwrap_or("");
        if cmd.is_empty() || cmd == "envelope" {
            continue;
        }
        let runtime = common::xr()
            .args(["schema", cmd, "--output", "json"])
            .output()
            .unwrap();
        assert!(runtime.status.success(), "xr schema {cmd} exited non-zero");
        let runtime_body = String::from_utf8(runtime.stdout).unwrap();
        let path = format!("{dir}/{cmd}.schema.json");
        let committed = std::fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "schema/responses/{cmd}.schema.json missing; regenerate with: ./scripts/generate-response-schemas.sh"
            )
        });
        assert_eq!(
            runtime_body.trim(),
            committed.trim(),
            "schema/responses/{cmd}.schema.json drifted; regenerate with: ./scripts/generate-response-schemas.sh"
        );
        checked += 1;
    }
    assert!(checked > 0, "no per-command schemas were checked");
}

#[test]
fn committed_response_schemas_have_no_orphans() {
    // The generator only writes files for commands in SCHEMA_ENTRIES; a
    // command removed from the list leaves its committed schema behind.
    let list = common::xr()
        .args(["schema", "--list", "--output", "text"])
        .output()
        .unwrap();
    assert!(list.status.success());
    let list_text = String::from_utf8(list.stdout).unwrap();
    let known: std::collections::BTreeSet<String> = list_text
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|cmd| !cmd.is_empty() && *cmd != "envelope")
        .map(str::to_string)
        .collect();

    let dir = concat!(env!("CARGO_WORKSPACE_DIR"), "/schema/responses");
    let orphans: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter_map(|name| {
            name.strip_suffix(".schema.json")
                .filter(|cmd| !known.contains(*cmd))
                .map(|_| name.clone())
        })
        .collect();
    assert!(
        orphans.is_empty(),
        "schema/responses/ has files for commands `xr schema --list` does not know: {orphans:?}; delete them"
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

/// Every command clap parses, under the name `xr schema` accepts for it.
fn clap_schema_names() -> BTreeSet<String> {
    common::command_paths()
        .iter()
        .map(|path| {
            let parts: Vec<&str> = path.iter().map(String::as_str).collect();
            schema_name_for_path(&parts)
        })
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
    let flag_forms: BTreeSet<&str> = FLAG_FORMS.iter().map(|(name, _)| *name).collect();

    let stale_registry: Vec<&str> = registered_commands()
        .filter(|name| !names.contains(*name) && !flag_forms.contains(name))
        .collect();
    assert!(
        stale_registry.is_empty(),
        "SCHEMA_ENTRIES names commands clap does not parse: {stale_registry:?}\n\
         Cause: the command was renamed or removed from the clap tree, or its schema name \
         differs from its clap path without a SCHEMA_NAME_OVERRIDES entry.\n\
         Fix: in {SCHEMA_SOURCE}, drop the row, rename it to the clap path joined with `-`, \
         or add the override; then run scripts/generate-response-schemas.sh."
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

    let stale_flag_forms: Vec<&&str> = FLAG_FORMS
        .iter()
        .map(|(_, base)| base)
        .filter(|base| !names.contains(**base))
        .collect();
    assert!(
        stale_flag_forms.is_empty(),
        "FLAG_FORMS points at commands clap does not parse: {stale_flag_forms:?}\n\
         Cause: the base command was renamed or removed from the clap tree.\n\
         Fix: in {SCHEMA_SOURCE}, point each entry at the command whose flag it describes."
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
#[case("skill")]
#[case("examples")]
#[case("validate")]
#[case("auth-oauth2")]
#[case("media-upload")]
fn schema_less_command_is_not_reported_unknown(#[case] command: &str) {
    common::xr()
        .args(["schema", command])
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema not available"))
        .stderr(predicate::str::contains("unknown command").not());
}
