//! Tests for agentic coding flags: --output, --quiet, --no-interactive, --timeout.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_output_json_flag_accepted() {
    // --output json should be accepted and change behavior
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--output", "json", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_jsonl_flag_accepted() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--output", "jsonl", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_text_flag_accepted() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--output", "text", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_invalid_value_fails() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--output", "xml", "--help"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_quiet_flag_accepted() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--quiet", "--help"])
        .assert()
        .success();
}

#[test]
fn test_quiet_short_flag_accepted() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["-q", "--help"])
        .assert()
        .success();
}

#[test]
fn test_no_interactive_flag_accepted() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--no-interactive", "--help"])
        .assert()
        .success();
}

#[test]
fn test_timeout_flag_accepted() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--timeout", "60", "--help"])
        .assert()
        .success();
}

#[test]
fn test_no_color_env_respected() {
    // NO_COLOR is an industry standard (https://no-color.org/)
    // When set, colored output should be suppressed.
    // We test that the flag doesn't cause a crash.
    Command::cargo_bin("xr")
        .unwrap()
        .env("NO_COLOR", "1")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_xurl_output_env_var() {
    // XURL_OUTPUT env var should set default output format
    Command::cargo_bin("xr")
        .unwrap()
        .env("XURL_OUTPUT", "json")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_combined_agentic_flags() {
    // All agentic flags can be used together
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--output", "json", "--quiet", "--no-interactive", "--timeout", "10", "--help"])
        .assert()
        .success();
}

#[test]
fn test_exit_code_success_on_help() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();

    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn test_exit_code_nonzero_on_error() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .arg("--definitely-not-a-flag")
        .output()
        .unwrap();

    assert_ne!(output.status.code().unwrap(), 0);
}
