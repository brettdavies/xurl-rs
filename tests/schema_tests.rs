//! Integration tests for the `xr schema` subcommand.

use assert_cmd::Command;
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════
// Single command schema
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn schema_post_outputs_valid_json_schema() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "post"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    // Must have standard JSON Schema fields
    assert!(json.get("$defs").is_some() || json.get("definitions").is_some());
    assert!(json.get("properties").is_some());
    assert_eq!(json["type"], "object");
}

#[test]
fn schema_post_contains_post_fields() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "post"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"text\""));
    assert!(stdout.contains("\"author_id\""));
    assert!(stdout.contains("\"includes\""));
    assert!(stdout.contains("\"meta\""));
}

#[test]
fn schema_whoami_contains_user_fields() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "whoami"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"username\""));
    assert!(stdout.contains("\"name\""));
}

#[test]
fn schema_like_contains_liked_field() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "like"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"liked\""));
}

#[test]
fn schema_no_extra_named_property() {
    // #[serde(flatten)] BTreeMap should produce additionalProperties, not a named "extra" field
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "post"])
        .output()
        .unwrap();
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
fn schema_list_shows_all_commands_plus_envelope() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "--list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    // 34 typed response commands + 1 envelope schema row.
    assert_eq!(lines.len(), 35, "Expected 35 rows, got {}", lines.len());
    assert!(
        stdout.contains("envelope"),
        "--list should advertise the envelope schema"
    );
}

#[test]
fn schema_list_contains_expected_commands() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "--list"])
        .output()
        .unwrap();
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
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "--list"])
        .output()
        .unwrap();
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
fn schema_all_outputs_json_with_all_commands() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "--all"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let obj = json.as_object().unwrap();
    assert_eq!(obj.len(), 34, "Expected 34 entries, got {}", obj.len());
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
    let output = Command::cargo_bin("xr")
        .unwrap()
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
    Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "bogus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown command 'bogus'"));
}

#[test]
fn schema_auth_not_available() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "auth"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema not available"));
}

#[test]
fn schema_media_not_available() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "media"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema not available"));
}

#[test]
fn schema_completions_not_available() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "completions"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema not available"));
}

#[test]
fn schema_version_not_available() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "version"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema not available"));
}

#[test]
fn schema_no_args_shows_usage() {
    Command::cargo_bin("xr")
        .unwrap()
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
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "envelope"])
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
fn envelope_schema_via_flag_equals_positional() {
    let by_pos = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "envelope"])
        .output()
        .unwrap();
    let by_flag = Command::cargo_bin("xr")
        .unwrap()
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
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "envelope", "--output", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let runtime = String::from_utf8(output.stdout).unwrap();
    let committed = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
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
    let list = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "--list", "--output", "text"])
        .output()
        .unwrap();
    assert!(list.status.success());
    let list_text = String::from_utf8(list.stdout).unwrap();

    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/schema/responses");
    let mut checked = 0usize;
    for line in list_text.lines() {
        let cmd = line.split_whitespace().next().unwrap_or("");
        if cmd.is_empty() || cmd == "envelope" {
            continue;
        }
        let runtime = Command::cargo_bin("xr")
            .unwrap()
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
fn schema_commands_sharing_type_produce_identical_output() {
    // post, reply, quote, read should all return the same schema
    let post = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "post"])
        .output()
        .unwrap();
    let reply = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "reply"])
        .output()
        .unwrap();
    assert_eq!(
        post.stdout, reply.stdout,
        "post and reply should share the same schema"
    );
}
