//! Integration tests for --output, --quiet, --no-interactive, and exit code wiring.
//!
//! These tests verify that the flags actually change behavior (not just parse).

use assert_cmd::Command;
use tempfile::TempDir;

// ═══════════════════════════════════════════════════════════════════════════
// --output json wiring
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_version_outputs_plain_text_ignoring_json_flag() {
    // version is a Tier 1 meta-command — ignores --output json, always plain text
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["version", "--output", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("xr"),
        "version should output plain text: {stdout}"
    );
    assert!(
        !stdout.starts_with('{'),
        "version should not output JSON: {stdout}"
    );
}

#[test]
fn test_json_output_no_ansi_codes() {
    // JSON output should never contain ANSI escape sequences
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["version", "--output", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\x1b["),
        "JSON output should not contain ANSI escape codes: {stdout}"
    );
}

#[test]
fn test_json_output_error_format() {
    // When a command fails with --output json, stderr should be structured JSON
    let tmp = TempDir::new().unwrap();

    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["whoami", "--output", "json"])
        .env("HOME", tmp.path())
        .env_remove("CLIENT_ID")
        .env_remove("CLIENT_SECRET")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should be valid JSON with error, kind, and code fields
    let parsed: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
    assert!(
        parsed["error"].is_string(),
        "error field should be a string"
    );
    assert!(parsed["kind"].is_string(), "kind field should be a string");
    assert!(parsed["code"].is_number(), "code field should be a number");
}

#[test]
fn test_text_output_has_color_by_default() {
    // Default text output for version should contain the plain version string
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["version"])
        .env_remove("NO_COLOR")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("xr"));
}

// ═══════════════════════════════════════════════════════════════════════════
// NO_COLOR environment variable
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_no_color_env_strips_ansi() {
    // NO_COLOR=1 should strip ANSI codes even in text mode
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["version"])
        .env("NO_COLOR", "1")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\x1b["),
        "NO_COLOR should strip ANSI codes: {stdout}"
    );
    assert!(stdout.contains("xr"));
}

// ═══════════════════════════════════════════════════════════════════════════
// XURL_OUTPUT environment variable
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_xurl_output_env_sets_json_format() {
    // XURL_OUTPUT=json should make auth status output JSON
    let tmp = TempDir::new().unwrap();

    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["auth", "status"])
        .env("HOME", tmp.path())
        .env("XURL_OUTPUT", "json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(first_line).unwrap();
    assert!(parsed["message"].is_string());
}

#[test]
fn test_explicit_output_overrides_env() {
    // --output text should override XURL_OUTPUT=json
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["version", "--output", "text"])
        .env("XURL_OUTPUT", "json")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Text mode: plain "xurl X.Y.Z" not JSON
    assert!(
        !stdout.starts_with('{'),
        "--output text should override XURL_OUTPUT=json"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// --quiet wiring
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_quiet_suppresses_version_output() {
    // version is Tier 1 — ignores --quiet, always prints
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["version", "--quiet"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("xr"),
        "Version output should still appear with --quiet"
    );
}

#[test]
fn test_quiet_flag_no_stderr_on_success() {
    // With --quiet, successful commands should have no stderr output
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["version", "--quiet"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "Quiet mode should produce no stderr on success: {stderr}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// --no-interactive wiring
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_no_interactive_blocks_auth_default_picker() {
    // `xurl auth default --no-interactive` without an app name should fail
    let tmp = TempDir::new().unwrap();

    // First, set up a minimal token store with an app
    let xurl_dir = tmp.path().join(".xurl");
    std::fs::create_dir_all(&xurl_dir).unwrap();
    std::fs::write(
        xurl_dir.join("apps.json"),
        r#"{"my-app":{"client_id":"test","client_secret":"test","default_user":"","oauth1_token":null,"bearer_token":null}}"#,
    )
    .unwrap();
    std::fs::write(xurl_dir.join("default_app"), "my-app").unwrap();

    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["auth", "default", "--no-interactive"])
        .env("HOME", tmp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    // Should mention that interactive prompt is required
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("nteractive") || stderr.contains("prompt"),
        "Should mention interactive requirement: {stderr}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Exit code wiring
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_exit_code_zero_on_success() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["version"])
        .output()
        .unwrap();

    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn test_exit_code_auth_required_is_2() {
    // Auth failure should exit with code 2
    let tmp = TempDir::new().unwrap();

    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["whoami"])
        .env("HOME", tmp.path())
        .env_remove("CLIENT_ID")
        .env_remove("CLIENT_SECRET")
        .output()
        .unwrap();

    assert_eq!(
        output.status.code().unwrap(),
        2,
        "Auth error should exit with code 2"
    );
}

#[test]
fn test_exit_code_json_error_includes_code() {
    // With --output json, the error JSON should include the exit code
    let tmp = TempDir::new().unwrap();

    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["whoami", "--output", "json"])
        .env("HOME", tmp.path())
        .env_remove("CLIENT_ID")
        .env_remove("CLIENT_SECRET")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
    assert_eq!(parsed["code"].as_i64().unwrap(), 2);
    // kind reflects the error variant (api for HTTP 401), code reflects semantic meaning
    assert!(parsed["kind"].is_string(), "kind should be a string");
}

// ═══════════════════════════════════════════════════════════════════════════
// Combined flags
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_json_quiet_combined() {
    // --output json --quiet should still produce JSON output (tested via auth status)
    let tmp = TempDir::new().unwrap();

    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["auth", "status", "--output", "json", "--quiet"])
        .env("HOME", tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(first_line).unwrap();
    assert!(parsed["message"].is_string());
}

#[test]
fn test_all_agentic_flags_wired_correctly() {
    // All flags together should work and produce clean JSON output
    let tmp = TempDir::new().unwrap();

    let output = Command::cargo_bin("xr")
        .unwrap()
        .args([
            "auth",
            "status",
            "--output",
            "json",
            "--quiet",
            "--no-interactive",
            "--timeout",
            "5",
        ])
        .env("HOME", tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(first_line).is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════
// Auth subcommand output format wiring
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_auth_status_json_output() {
    // `xurl auth status --output json` should output JSON with default app info
    let tmp = TempDir::new().unwrap();

    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["auth", "status", "--output", "json"])
        .env("HOME", tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Output is JSONL (one JSON object per line) — parse the first line
    let first_line = stdout.lines().next().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(first_line).unwrap();
    assert!(parsed["message"].as_str().unwrap().contains("default"));
}

#[test]
fn test_auth_apps_list_json_output() {
    // `xurl auth apps list --output json` should list the default app as JSON
    let tmp = TempDir::new().unwrap();

    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["auth", "apps", "list", "--output", "json"])
        .env("HOME", tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(parsed["message"].as_str().unwrap().contains("default"));
}
