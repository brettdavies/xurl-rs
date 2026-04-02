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
fn schema_post_contains_tweet_fields() {
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
fn schema_list_shows_all_29_commands() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "--list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 29, "Expected 29 commands, got {}", lines.len());
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
        "block",
        "unblock",
        "mute",
        "unmute",
        "dm",
        "dms",
        "usage",
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
    assert!(stdout.contains("ApiResponse<Tweet>"));
    assert!(stdout.contains("ApiResponse<Vec<Tweet>>"));
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
    assert_eq!(obj.len(), 29, "Expected 29 entries, got {}", obj.len());
    // Each value should be a valid schema object
    for (cmd, schema) in obj {
        assert!(
            schema.get("properties").is_some(),
            "Schema for '{cmd}' missing properties"
        );
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
