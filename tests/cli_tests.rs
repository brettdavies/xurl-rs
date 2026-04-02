//! CLI integration tests using assert_cmd + predicates.
//!
//! These test the compiled xr binary as a subprocess.
//! The Go tests do NOT cover CLI integration — this is new coverage.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

// ═══════════════════════════════════════════════════════════════════════════
// Basic CLI sanity tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_help_flag() {
    Command::cargo_bin("xr")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn test_version_flag() {
    Command::cargo_bin("xr")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("xr"));
}

#[test]
fn test_invalid_flag() {
    Command::cargo_bin("xr")
        .unwrap()
        .arg("--definitely-not-a-real-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Subcommand help tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_post_help() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["post", "--help"])
        .assert()
        .success();
}

#[test]
fn test_search_help() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["search", "--help"])
        .assert()
        .success();
}

#[test]
fn test_auth_help() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["auth", "--help"])
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════════════════
// Command error handling tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_post_without_text_fails() {
    // Post command requires text argument
    Command::cargo_bin("xr")
        .unwrap()
        .arg("post")
        .assert()
        .failure();
}

#[test]
fn test_search_without_query_fails() {
    // Search command requires a query
    Command::cargo_bin("xr")
        .unwrap()
        .arg("search")
        .assert()
        .failure();
}

#[test]
fn test_delete_without_id_fails() {
    Command::cargo_bin("xr")
        .unwrap()
        .arg("delete")
        .assert()
        .failure();
}

#[test]
fn test_reply_without_args_fails() {
    Command::cargo_bin("xr")
        .unwrap()
        .arg("reply")
        .assert()
        .failure();
}

// ═══════════════════════════════════════════════════════════════════════════
// Usage command tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_usage_help() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["usage", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("usage"))
        .stdout(predicate::str::contains("tweet caps"));
}

#[test]
fn test_usage_without_auth_fails() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("xr")
        .unwrap()
        .arg("usage")
        .env("HOME", tmp.path())
        .env_remove("XURL_CLIENT_ID")
        .env_remove("XURL_CLIENT_SECRET")
        .env_remove("XURL_BEARER_TOKEN")
        .assert()
        .failure();
}

// ═══════════════════════════════════════════════════════════════════════════
// Auth-required commands should fail without credentials
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_whoami_without_auth_fails() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("xr")
        .unwrap()
        .arg("whoami")
        .env("HOME", tmp.path())
        .env_remove("XURL_CLIENT_ID")
        .env_remove("XURL_CLIENT_SECRET")
        .env_remove("XURL_BEARER_TOKEN")
        .assert()
        .failure();
}

// ═══════════════════════════════════════════════════════════════════════════
// App management subcommands
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_apps_list_help() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["auth", "apps", "--help"])
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════════════════
// Exit code parity tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_exit_code_success_on_help() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();

    assert_eq!(
        output.status.code().unwrap(),
        0,
        "Expected exit code 0 for --help"
    );
}

#[test]
fn test_exit_code_failure_on_bad_flag() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .arg("--nonexistent")
        .output()
        .unwrap();

    assert_ne!(
        output.status.code().unwrap(),
        0,
        "Expected non-zero exit code for bad flag"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Verbose / trace flag tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_verbose_flag_accepted() {
    // --verbose should be accepted even if the command ultimately fails
    // due to missing auth — we just verify the flag is recognized
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--verbose", "--help"])
        .assert()
        .success();
}

#[test]
fn test_trace_flag_accepted() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--trace", "--help"])
        .assert()
        .success();
}
