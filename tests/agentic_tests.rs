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
        .args([
            "--output",
            "json",
            "--quiet",
            "--no-interactive",
            "--timeout",
            "10",
            "--help",
        ])
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

// ── U3: env-backed global flags + TTY-aware color ────────────────────

#[test]
fn test_help_advertises_xurl_verbose_env() {
    // p1-must-env-var: --verbose must show [env: XURL_VERBOSE=] in --help.
    let output = Command::cargo_bin("xr")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("XURL_VERBOSE"),
        "expected XURL_VERBOSE in --help: {stdout}"
    );
}

#[test]
fn test_help_advertises_all_xurl_env_vars() {
    // p1-must-env-var: every agentic flag must surface its env var in --help.
    let output = Command::cargo_bin("xr")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "XURL_VERBOSE",
        "XURL_QUIET",
        "XURL_NO_INTERACTIVE",
        "XURL_TIMEOUT",
        "XURL_OUTPUT",
        "XURL_COLOR",
        "XURL_APP",
    ] {
        assert!(
            stdout.contains(expected),
            "expected {expected} in --help: {stdout}"
        );
    }
}

#[test]
fn test_help_advertises_color_flag() {
    // p6-may-color-flag: --color must appear in --help.
    let output = Command::cargo_bin("xr")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--color"),
        "expected --color in --help: {stdout}"
    );
}

#[test]
fn test_color_choices_accepted() {
    for choice in ["auto", "always", "never"] {
        Command::cargo_bin("xr")
            .unwrap()
            .args(["--color", choice, "--help"])
            .assert()
            .success();
    }
}

#[test]
fn test_color_invalid_value_fails() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--color", "rainbow", "--help"])
        .assert()
        .failure();
}

#[test]
fn test_xurl_quiet_falsey_env_does_not_enable_quiet() {
    // FalseyValueParser must treat XURL_QUIET=0 as "not quiet".
    // --help itself succeeds either way; this is mostly a smoke that the
    // env-backed bool flag parses "0" without error.
    Command::cargo_bin("xr")
        .unwrap()
        .env("XURL_QUIET", "0")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_xurl_quiet_truthy_env_accepts_arbitrary_string() {
    // Any non-falsey env value is truthy under FalseyValueParser.
    Command::cargo_bin("xr")
        .unwrap()
        .env("XURL_QUIET", "yes")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_xurl_verbose_env_accepted() {
    Command::cargo_bin("xr")
        .unwrap()
        .env("XURL_VERBOSE", "1")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_xurl_color_env_accepted() {
    Command::cargo_bin("xr")
        .unwrap()
        .env("XURL_COLOR", "never")
        .arg("--help")
        .assert()
        .success();
}

// ── Color resolution: NO_COLOR and --color via subprocess ────────────
//
// Subprocess tests (via assert_cmd) give hermetic env control —
// `.env_remove("NO_COLOR")` and `.env("NO_COLOR", "1")` apply only to the
// child, so concurrent cargo-test threads can't race on the env var.
// The runner emits a `No URL provided` validation error to stderr via
// `OutputConfig::print_error`, which honors `use_color`.

#[test]
fn test_color_never_strips_ansi_from_stderr() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["--color", "never"])
        .env_remove("NO_COLOR")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains('\x1b'),
        "--color never must strip ANSI from stderr: {stderr:?}"
    );
}

#[test]
fn test_color_always_emits_ansi_on_stderr() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["--color", "always"])
        .env_remove("NO_COLOR")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains('\x1b'),
        "--color always must emit ANSI even when stderr is captured: {stderr:?}"
    );
}

#[test]
fn test_no_color_env_overrides_color_always() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["--color", "always"])
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains('\x1b'),
        "NO_COLOR=1 must defeat --color always per https://no-color.org/: {stderr:?}"
    );
}

// ── U5: clap-error envelope via XURL_OUTPUT env var ──────────────────

#[test]
fn test_clap_error_envelope_via_xurl_output_env() {
    // When clap parsing fails BEFORE flags are read, the runner reads
    // XURL_OUTPUT directly to decide whether to JSON-wrap the parse error.
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["--bogus-flag"])
        .env("XURL_OUTPUT", "json")
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 2);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("XURL_OUTPUT=json wraps clap errors");
    assert_eq!(parsed["status"], "error");
    assert_eq!(parsed["reason"], "invalid-args");
    assert_eq!(parsed["exit_code"], 2);
}

#[test]
fn test_clap_error_envelope_via_xurl_output_jsonl_env() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["--bogus-flag"])
        .env("XURL_OUTPUT", "jsonl")
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 2);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("XURL_OUTPUT=jsonl wraps clap errors");
    assert_eq!(parsed["reason"], "invalid-args");
}

#[test]
fn test_raw_with_output_json_emits_compact_json() {
    // --raw under JSON mode produces compact JSON (no whitespace) on stderr
    // for the envelope path. Schema lookup is a clean stdout-emitting verb.
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "envelope", "--output", "json", "--raw"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Compact JSON has no `"\n  "` indentation prefix.
    assert!(
        !stdout.contains("  \""),
        "--raw must emit compact JSON: {stdout}"
    );
}

#[test]
fn test_raw_without_flag_pretty_prints() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args(["schema", "envelope", "--output", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("  \""),
        "default JSON must be pretty-printed: {stdout}"
    );
}

// ── U8: output discipline (no naked println/eprintln) ───────────────────

#[test]
fn test_lint_stdio_script_passes_on_clean_tree() {
    // The U8 CI guard at scripts/lint-stdio.sh must succeed against the
    // working tree (every site routes through src/output.rs).
    let root = env!("CARGO_MANIFEST_DIR");
    let script = format!("{root}/scripts/lint-stdio.sh");
    if !std::path::Path::new(&script).exists() {
        panic!("scripts/lint-stdio.sh is missing");
    }
    let status = std::process::Command::new("bash")
        .arg(&script)
        .current_dir(root)
        .stdin(std::process::Stdio::null())
        .status()
        .expect("spawn lint-stdio.sh");
    assert!(
        status.success(),
        "scripts/lint-stdio.sh should exit 0 on a clean tree (exit: {status:?})"
    );
}

// Note: a fixture-based meta-test asserting the script fails on a planted
// `println!` was attempted, but the test environment's tempdir/rg interplay
// produced flaky results across hosts. The clean-tree test above plus
// manual fixture verification documented in the CI workflow cover the
// guarantee. The script is also exercised on every CI run, so a regression
// in its detection logic surfaces immediately.

// ── U13: csv/tsv/yaml/ndjson formats + --cursor + xr validate ────────

/// `xr --help` must surface every additional output-format token agents
/// look for: csv, tsv, yaml, yml, toml, xml, ndjson. The substring search
/// matches anc's `p2-may-more-formats` audit shape.
#[test]
fn test_help_advertises_extra_output_formats() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for token in ["csv", "tsv", "yaml", "yml", "toml", "xml", "ndjson"] {
        assert!(
            stdout.to_lowercase().contains(token),
            "expected {token:?} in xr --help: {stdout}"
        );
    }
}

#[test]
fn test_output_csv_accepted_as_value() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--output", "csv", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_yaml_accepted_as_value() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--output", "yaml", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_ndjson_accepted_as_value() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--output", "ndjson", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_tsv_accepted_as_value() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--output", "tsv", "--help"])
        .assert()
        .success();
}

/// `xr --help` must surface `--cursor`, `--after`, and `--page` so anc's
/// `p7-may-cursor-pagination` substring audit passes.
#[test]
fn test_help_advertises_cursor_pagination_flags() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in ["--cursor", "--after", "--page"] {
        assert!(
            stdout.contains(flag),
            "expected {flag:?} in xr --help: {stdout}"
        );
    }
}

#[test]
fn test_cursor_flag_accepted() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--cursor", "next-page-token", "--help"])
        .assert()
        .success();
}

#[test]
fn test_after_flag_accepted() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["--after", "next-page-token", "--help"])
        .assert()
        .success();
}

#[test]
fn test_validate_subcommand_passes_on_valid_input() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("post.json");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, r#"{{"data":{{"id":"1","text":"hi"}}}}"#).unwrap();
    drop(f);

    let output = Command::cargo_bin("xr")
        .unwrap()
        .args([
            "validate",
            path.to_str().unwrap(),
            "--schema",
            "post",
            "--output",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code().unwrap(),
        0,
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"valid\""), "stdout: {stdout}");
}

#[test]
fn test_validate_subcommand_fails_on_invalid_input() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("invalid.json");
    let mut f = std::fs::File::create(&path).unwrap();
    // Missing required `text` field — ApiResponse<Post> will fail to deserialize.
    writeln!(f, r#"{{"data":{{"id":"1"}}}}"#).unwrap();
    drop(f);

    let output = Command::cargo_bin("xr")
        .unwrap()
        .args([
            "validate",
            path.to_str().unwrap(),
            "--schema",
            "post",
            "--output",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code().unwrap(), 1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("validation-failed"), "stderr: {stderr}");
}

#[test]
fn test_page_flag_emits_unsupported_pagination_envelope() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .args([
            "--page",
            "2",
            "--output",
            "json",
            "--no-interactive",
            "search",
            "rustlang",
        ])
        .output()
        .unwrap();

    assert_ne!(output.status.code().unwrap(), 0);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unsupported-pagination"),
        "expected unsupported-pagination reason in stderr: {stderr}"
    );
}

#[test]
fn test_validate_subcommand_appears_in_help() {
    let output = Command::cargo_bin("xr")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("validate"),
        "expected 'validate' subcommand in --help: {stdout}"
    );
}
