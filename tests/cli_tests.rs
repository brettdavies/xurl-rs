//! CLI integration tests using the library entrypoint `xurl::cli::run_with_store_path`.
//!
//! Parallel-safe: every test creates its own `TempDir` for token-store isolation
//! and passes the path explicitly. No `HOME` / `XDG_CONFIG_HOME` mutation, no
//! `#[serial]` annotations, no subprocess overhead. The binary's exit-code
//! contract is pinned separately by `tests/binary_contract_tests.rs`.

use std::path::Path;

use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use xurl::cli;

// ═══════════════════════════════════════════════════════════════════════════
// Test helper
// ═══════════════════════════════════════════════════════════════════════════

/// Run the library entrypoint with a fresh tempdir-rooted store path.
///
/// Returns `(exit_code, stdout, stderr)`. Each call gets its own `TempDir`,
/// dropped at the end of the call — the `.xurl` path passed to the runner
/// never collides across parallel tests.
fn run_isolated(args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code = cli::run_with_store_path(args, &mut stdout, &mut stderr, &store);
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

/// Run the entrypoint against an existing `TempDir`-rooted store path.
///
/// Lets a single test issue multiple invocations against the same `.xurl`
/// so a setup invocation (e.g., `auth apps add`) and an assertion invocation
/// (e.g., `auth apps redirect-uri get`) observe the same state.
fn run_at(store_path: &Path, args: &[&str]) -> (i32, String, String) {
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code = cli::run_with_store_path(args, &mut stdout, &mut stderr, store_path);
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// Basic CLI sanity tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_help_flag() {
    let (code, stdout, stderr) = run_isolated(&["xr", "--help"]);
    assert_eq!(code, 0, "expected 0 for --help; stderr: {stderr}");
    assert!(
        stdout.contains("Usage"),
        "stdout should contain 'Usage': {stdout}"
    );
}

#[test]
fn test_version_flag() {
    let (code, stdout, stderr) = run_isolated(&["xr", "--version"]);
    assert_eq!(code, 0, "expected 0 for --version; stderr: {stderr}");
    assert!(
        stdout.contains("xr"),
        "stdout should contain 'xr': {stdout}"
    );
}

#[test]
fn test_invalid_flag() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--definitely-not-a-real-flag"]);
    assert_ne!(code, 0, "expected non-zero exit for invalid flag");
    assert!(
        stderr.contains("error"),
        "stderr should contain 'error': {stderr}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// U5: JSON envelope on clap parse failure + --json/--jsonl/--raw aliases
// ═══════════════════════════════════════════════════════════════════════════

/// Asserts `stderr` parses as the canonical `invalid-args` envelope at
/// `exit_code` 2.
fn assert_invalid_args_envelope(stderr: &str) {
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("clap-error envelope is valid JSON");
    assert_eq!(v["status"], "error", "envelope status: {stderr}");
    assert_eq!(v["reason"], "invalid-args", "envelope reason: {stderr}");
    assert_eq!(v["exit_code"], 2, "envelope exit_code: {stderr}");
    assert!(
        v["message"].is_string(),
        "envelope message must be present: {stderr}"
    );
}

#[test]
fn test_clap_error_emits_envelope_under_output_json() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--bogus-flag", "--output", "json"]);
    assert_eq!(code, 2, "EX_USAGE expected: {stderr}");
    assert_invalid_args_envelope(&stderr);
}

#[test]
fn test_clap_error_emits_envelope_under_json_alias() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--bogus-flag", "--json"]);
    assert_eq!(code, 2, "EX_USAGE expected: {stderr}");
    assert_invalid_args_envelope(&stderr);
}

#[test]
fn test_clap_error_emits_envelope_under_jsonl_alias() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--bogus-flag", "--jsonl"]);
    assert_eq!(code, 2, "EX_USAGE expected: {stderr}");
    assert_invalid_args_envelope(&stderr);
}

#[test]
fn test_clap_error_falls_back_to_text_without_json_intent() {
    // No --output json, no --json, no XURL_OUTPUT — clap's default text
    // rendering is preserved.
    let (code, _stdout, stderr) = run_isolated(&["xr", "--bogus-flag"]);
    assert_eq!(code, 2);
    assert!(
        serde_json::from_str::<serde_json::Value>(stderr.trim()).is_err(),
        "without JSON intent, stderr should not be JSON: {stderr}"
    );
    assert!(stderr.contains("error"));
}

#[test]
fn test_help_under_output_json_still_writes_to_stdout() {
    // DisplayHelp short-circuit: --help bypasses envelope routing.
    let (code, stdout, _stderr) = run_isolated(&["xr", "--help", "--output", "json"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("Usage"), "help on stdout: {stdout}");
}

#[test]
fn test_version_under_output_json_still_writes_to_stdout() {
    let (code, stdout, _stderr) = run_isolated(&["xr", "--version", "--output", "json"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("xr"), "version on stdout: {stdout}");
}

#[test]
fn test_envelope_consistency_clap_error_has_status_key() {
    // R6 / p2-should-consistent-envelope: clap-error JSON and runtime-error
    // JSON share the `status` discriminant; agents dispatch on it uniformly.
    let (_code, _stdout, stderr) = run_isolated(&["xr", "--bogus-flag", "--json"]);
    let parsed: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
    assert!(
        parsed.get("status").is_some(),
        "clap-error envelope carries status: {stderr}"
    );
}

#[test]
fn test_raw_flag_accepted() {
    // --raw is a global boolean flag; smoke-tests parse path.
    let (code, _stdout, _stderr) = run_isolated(&["xr", "--raw", "--help"]);
    assert_eq!(code, 0);
}

#[test]
fn test_json_and_output_conflict() {
    // clap should reject `--output json --json` together (validated after
    // parsing — --help would short-circuit, so use `version` subcommand).
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "--json", "version"]);
    assert_ne!(code, 0, "--json + --output must conflict: {stderr}");
}

#[test]
fn test_json_and_jsonl_conflict() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--json", "--jsonl", "version"]);
    assert_ne!(code, 0, "--json + --jsonl must conflict: {stderr}");
}

#[test]
fn test_json_alias_envelope_equivalent_to_output_json() {
    // On a clap parse failure, `--json` and `--output json` produce
    // identical envelope JSON modulo the embedded clap-rendered message
    // (which mentions the flag name). Compare structure on shared keys.
    let (_c1, _o1, e1) = run_isolated(&["xr", "--bogus-flag", "--json"]);
    let (_c2, _o2, e2) = run_isolated(&["xr", "--bogus-flag", "--output", "json"]);
    let p1: serde_json::Value = serde_json::from_str(e1.trim()).unwrap();
    let p2: serde_json::Value = serde_json::from_str(e2.trim()).unwrap();
    assert_eq!(p1["status"], p2["status"]);
    assert_eq!(p1["reason"], p2["reason"]);
    assert_eq!(p1["exit_code"], p2["exit_code"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// Subcommand help tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_post_help() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "post", "--help"]);
    assert_eq!(code, 0, "expected 0 for post --help; stderr: {stderr}");
}

#[test]
fn test_search_help() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "search", "--help"]);
    assert_eq!(code, 0, "expected 0 for search --help; stderr: {stderr}");
}

#[test]
fn test_auth_help() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "auth", "--help"]);
    assert_eq!(code, 0, "expected 0 for auth --help; stderr: {stderr}");
}

// ═══════════════════════════════════════════════════════════════════════════
// Command error handling tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_post_without_text_fails() {
    // Post command requires text argument
    let (code, _stdout, _stderr) = run_isolated(&["xr", "post"]);
    assert_ne!(code, 0, "expected non-zero exit for `post` with no args");
}

#[test]
fn test_search_without_query_fails() {
    // Search command requires a query
    let (code, _stdout, _stderr) = run_isolated(&["xr", "search"]);
    assert_ne!(code, 0, "expected non-zero exit for `search` with no args");
}

#[test]
fn test_delete_without_id_fails() {
    let (code, _stdout, _stderr) = run_isolated(&["xr", "delete"]);
    assert_ne!(code, 0, "expected non-zero exit for `delete` with no args");
}

#[test]
fn test_reply_without_args_fails() {
    let (code, _stdout, _stderr) = run_isolated(&["xr", "reply"]);
    assert_ne!(code, 0, "expected non-zero exit for `reply` with no args");
}

// ═══════════════════════════════════════════════════════════════════════════
// Usage command tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_usage_help() {
    let (code, stdout, stderr) = run_isolated(&["xr", "usage", "--help"]);
    assert_eq!(code, 0, "expected 0 for usage --help; stderr: {stderr}");
    assert!(
        stdout.contains("usage"),
        "stdout should contain 'usage': {stdout}"
    );
    assert!(
        stdout.contains("tweet caps"),
        "stdout should contain 'tweet caps': {stdout}"
    );
}

#[test]
fn test_usage_without_auth_fails() {
    // Isolated empty token store via tempdir — no env mutation needed.
    let (code, _stdout, _stderr) = run_isolated(&["xr", "usage"]);
    assert_ne!(code, 0, "expected non-zero exit for `usage` without auth");
}

// ═══════════════════════════════════════════════════════════════════════════
// Auth-required commands should fail without credentials
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_whoami_without_auth_fails() {
    // Isolated empty token store via tempdir — no env mutation needed.
    let (code, _stdout, _stderr) = run_isolated(&["xr", "whoami"]);
    assert_ne!(code, 0, "expected non-zero exit for `whoami` without auth");
}

// ═══════════════════════════════════════════════════════════════════════════
// App management subcommands
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_apps_list_help() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "auth", "apps", "--help"]);
    assert_eq!(code, 0, "expected 0 for auth apps --help; stderr: {stderr}");
}

// ═══════════════════════════════════════════════════════════════════════════
// Exit code parity tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_exit_code_success_on_help() {
    let (code, _stdout, _stderr) = run_isolated(&["xr", "--help"]);
    assert_eq!(code, 0, "Expected exit code 0 for --help");
}

#[test]
fn test_exit_code_failure_on_bad_flag() {
    let (code, _stdout, _stderr) = run_isolated(&["xr", "--nonexistent"]);
    assert_ne!(code, 0, "Expected non-zero exit code for bad flag");
}

// ═══════════════════════════════════════════════════════════════════════════
// Verbose / trace flag tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_verbose_flag_accepted() {
    // --verbose should be accepted even if the command ultimately fails
    // due to missing auth — we just verify the flag is recognized
    let (code, _stdout, stderr) = run_isolated(&["xr", "--verbose", "--help"]);
    assert_eq!(code, 0, "expected 0 for --verbose --help; stderr: {stderr}");
}

#[test]
fn test_trace_flag_accepted() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--trace", "--help"]);
    assert_eq!(code, 0, "expected 0 for --trace --help; stderr: {stderr}");
}

// ═══════════════════════════════════════════════════════════════════════════
// U4: --redirect-uri on add/update + auth apps redirect-uri get/set
// ═══════════════════════════════════════════════════════════════════════════

fn parse_json(stdout: &str) -> serde_json::Value {
    // `print_response` emits a single pretty-printed JSON document followed
    // by a trailing newline; parsing the whole buffer is sufficient.
    serde_json::from_str(stdout.trim()).expect("stdout is valid JSON")
}

#[test]
#[serial_test::serial]
fn test_apps_add_with_redirect_uri_persists() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    let (code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
            "--redirect-uri",
            "https://example.com/cb",
        ],
    );
    assert_eq!(code, 0, "add failed; stderr: {stderr}");

    let (code2, stdout2, stderr2) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "apps",
            "redirect-uri",
            "get",
            "myapp",
        ],
    );
    assert_eq!(code2, 0, "get failed; stderr: {stderr2}");
    let v = parse_json(&stdout2);
    assert_eq!(v["app"], "myapp");
    assert_eq!(v["effective_redirect_uri"], "https://example.com/cb");
    assert_eq!(v["effective_source"], "app-config");
    assert_eq!(v["stored_redirect_uri"], "https://example.com/cb");
}

#[test]
#[serial_test::serial]
fn test_apps_add_without_redirect_uri_leaves_empty() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    let (code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
        ],
    );
    assert_eq!(code, 0, "add failed; stderr: {stderr}");

    let (code2, stdout2, stderr2) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "apps",
            "redirect-uri",
            "get",
            "myapp",
        ],
    );
    assert_eq!(code2, 0, "get failed; stderr: {stderr2}");
    let v = parse_json(&stdout2);
    assert_eq!(v["stored_redirect_uri"], serde_json::Value::Null);
    assert_eq!(v["effective_source"], "built-in-default");
}

#[test]
fn test_apps_add_with_invalid_redirect_uri_rejected() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    let (code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
            "--redirect-uri",
            "http://attacker.example.com/cb",
        ],
    );
    assert_ne!(code, 0, "expected non-zero exit for invalid redirect URI");
    assert!(
        stderr.to_lowercase().contains("redirect")
            || stderr.to_lowercase().contains("https")
            || stderr.to_lowercase().contains("loopback"),
        "stderr should mention validation failure: {stderr}"
    );
}

#[test]
fn test_apps_update_with_redirect_uri_changes_value() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    let (code, _, stderr) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
        ],
    );
    assert_eq!(code, 0, "add failed; stderr: {stderr}");

    let (code2, _, stderr2) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "update",
            "myapp",
            "--redirect-uri",
            "https://example.com/cb",
        ],
    );
    assert_eq!(code2, 0, "update failed; stderr: {stderr2}");

    let (code3, stdout3, _) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "apps",
            "redirect-uri",
            "get",
            "myapp",
        ],
    );
    assert_eq!(code3, 0);
    let v = parse_json(&stdout3);
    assert_eq!(v["stored_redirect_uri"], "https://example.com/cb");
}

#[test]
fn test_apps_update_with_empty_redirect_uri_clears() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    let (code, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
            "--redirect-uri",
            "https://example.com/cb",
        ],
    );
    assert_eq!(code, 0);

    let (code2, _, stderr2) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "update",
            "myapp",
            "--redirect-uri",
            "",
        ],
    );
    assert_eq!(
        code2, 0,
        "update --redirect-uri \"\" failed; stderr: {stderr2}"
    );

    let (code3, stdout3, _) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "apps",
            "redirect-uri",
            "get",
            "myapp",
        ],
    );
    assert_eq!(code3, 0);
    let v = parse_json(&stdout3);
    assert_eq!(v["stored_redirect_uri"], serde_json::Value::Null);
}

#[test]
fn test_apps_update_no_fields_errors() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    let (code, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
        ],
    );
    assert_eq!(code, 0);

    let (code2, _stdout, stderr2) = run_at(&store, &["xr", "auth", "apps", "update", "myapp"]);
    assert_ne!(code2, 0, "expected non-zero exit for empty update");
    assert!(
        stderr2.to_lowercase().contains("nothing to update"),
        "stderr should mention 'nothing to update': {stderr2}"
    );
}

#[test]
fn test_redirect_uri_set_persists() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    let (code, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
        ],
    );
    assert_eq!(code, 0);

    let (code2, _stdout, stderr2) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "redirect-uri",
            "set",
            "myapp",
            "https://example.com/cb",
        ],
    );
    assert_eq!(code2, 0, "set failed; stderr: {stderr2}");

    let (code3, stdout3, _) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "apps",
            "redirect-uri",
            "get",
            "myapp",
        ],
    );
    assert_eq!(code3, 0);
    let v = parse_json(&stdout3);
    assert_eq!(v["stored_redirect_uri"], "https://example.com/cb");
}

#[test]
fn test_redirect_uri_set_invalid_rejected() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    let (code, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
        ],
    );
    assert_eq!(code, 0);

    let (code2, _stdout, stderr2) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "redirect-uri",
            "set",
            "myapp",
            "http://attacker.example.com/cb",
        ],
    );
    assert_ne!(code2, 0, "expected non-zero exit for invalid set");
    assert!(
        stderr2.to_lowercase().contains("redirect")
            || stderr2.to_lowercase().contains("https")
            || stderr2.to_lowercase().contains("loopback"),
        "stderr should mention validation failure: {stderr2}"
    );
}

#[test]
fn test_redirect_uri_get_text_output() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    let (code, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
            "--redirect-uri",
            "https://example.com/cb",
        ],
    );
    assert_eq!(code, 0);

    let (code2, stdout2, stderr2) = run_at(
        &store,
        &["xr", "auth", "apps", "redirect-uri", "get", "myapp"],
    );
    assert_eq!(code2, 0, "get failed; stderr: {stderr2}");
    assert!(stdout2.contains("app:"), "missing app: line: {stdout2}");
    assert!(
        stdout2.contains("effective_redirect_uri:"),
        "missing effective_redirect_uri: line: {stdout2}"
    );
    assert!(
        stdout2.contains("effective_source:"),
        "missing effective_source: line: {stdout2}"
    );
    assert!(
        stdout2.contains("stored_redirect_uri:"),
        "missing stored_redirect_uri: line: {stdout2}"
    );
    assert!(
        stdout2.contains("https://example.com/cb"),
        "missing stored URI value: {stdout2}"
    );
}

#[test]
fn test_redirect_uri_get_uses_default_app_when_name_omitted() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    // Add "myapp" and explicitly set it as the default so the no-NAME `get`
    // resolves through it. (TokenStore seeds a "default" placeholder on a
    // fresh tempdir, so add_app alone does not flip the default.)
    let (code, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
            "--redirect-uri",
            "https://example.com/cb",
        ],
    );
    assert_eq!(code, 0);

    let (code_d, _, stderr_d) = run_at(&store, &["xr", "auth", "default", "myapp"]);
    assert_eq!(code_d, 0, "set-default failed; stderr: {stderr_d}");

    let (code2, stdout2, stderr2) = run_at(&store, &["xr", "auth", "apps", "redirect-uri", "get"]);
    assert_eq!(code2, 0, "get failed; stderr: {stderr2}");
    assert!(
        stdout2.contains("app: myapp"),
        "expected default app name in output: {stdout2}"
    );
    assert!(
        stdout2.contains("https://example.com/cb"),
        "expected stored URI in output: {stdout2}"
    );
}

#[test]
#[serial_test::serial]
fn test_redirect_uri_get_uses_placeholder_default_when_store_empty() {
    // Fresh tempdir → TokenStore seeds the placeholder "default" app.
    // The omit-NAME `get` resolves through it and surfaces the built-in
    // default URI (no stored value on the placeholder).
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage that
    // would flip the asserted text label to the env-var variant.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    let (code, stdout, stderr) = run_at(&store, &["xr", "auth", "apps", "redirect-uri", "get"]);
    assert_eq!(code, 0, "get failed; stderr: {stderr}");
    assert!(stdout.contains("app:"), "missing app: line: {stdout}");
    assert!(
        stdout.contains("effective_source: built-in default"),
        "expected built-in default source: {stdout}"
    );
    assert!(
        stdout.contains("stored_redirect_uri: (none)"),
        "expected stored marker: {stdout}"
    );
}

#[test]
#[serial_test::serial]
fn test_redirect_uri_get_json_output_app_config_source() {
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage from
    // the env-override test in the same binary.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    let (code, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
            "--redirect-uri",
            "https://example.com/cb",
        ],
    );
    assert_eq!(code, 0);

    let (code2, stdout2, _) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "apps",
            "redirect-uri",
            "get",
            "myapp",
        ],
    );
    assert_eq!(code2, 0);
    let v = parse_json(&stdout2);
    assert_eq!(v["app"], "myapp");
    assert_eq!(v["effective_redirect_uri"], "https://example.com/cb");
    assert_eq!(v["effective_source"], "app-config");
}

// ═══════════════════════════════════════════════════════════════════════════
// U5: Status + Apps-List rendering — text mode, JSON mode, secret exclusion
// ═══════════════════════════════════════════════════════════════════════════

/// Populates a tempdir-rooted store with one app that carries every credential
/// kind (`client_secret`, `oauth2_tokens`, `oauth1_token`, `bearer_token`) so
/// the secret-exclusion assertions exercise every leak path.
fn populate_credentialed_store(store_path: &Path) {
    use xurl::store::TokenStore;
    let mut ts = TokenStore::new_with_path(store_path.to_str().unwrap());
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE-AAA")
        .expect("add_app");
    ts.save_oauth2_token_for_app(
        "myapp",
        "alice",
        "ACCESS-TOKEN-BBB",
        "REFRESH-TOKEN-CCC",
        1_900_000_000,
    )
    .expect("save_oauth2");
    ts.save_oauth1_tokens_for_app(
        "myapp",
        "OA1-ACCESS-TOKEN",
        "TOKEN-SECRET-EEE",
        "OA1-CONSUMER-KEY",
        "CONSUMER-SECRET-DDD",
    )
    .expect("save_oauth1");
    ts.save_bearer_token_for_app("myapp", "BEARER-VALUE-FFF")
        .expect("save_bearer");
    // KTD9 + R20: the unnamed slot carries OAuth2 credentials that must also
    // be excluded from rendered JSON. The banned-string list below grows to
    // match.
    ts.save_oauth2_token_unnamed_for_app(
        "myapp",
        "UNNAMED-AT-AAA",
        "UNNAMED-RT-BBB",
        1_900_000_000,
    )
    .expect("save_oauth2_unnamed");
    ts.set_default_app("myapp").expect("set_default_app");
    // `TokenStore::new_with_path` seeds an empty `"default"` placeholder app
    // on first load; drop it so the JSON array carries exactly one entry.
    let _ = ts.remove_app("default");
}

/// Asserts the JSON stdout from a status/list/get invocation does not contain
/// any of the credential field names or fixture credential values.
fn assert_no_credentials(stdout: &str, context: &str) {
    let banned: &[&str] = &[
        // Credential values from `populate_credentialed_store`.
        "SECRET-VALUE-AAA",
        "ACCESS-TOKEN-BBB",
        "REFRESH-TOKEN-CCC",
        "CONSUMER-SECRET-DDD",
        "TOKEN-SECRET-EEE",
        "BEARER-VALUE-FFF",
        // Unnamed (`/me`-failed salvage) slot credentials from
        // `populate_credentialed_store` per KTD1; the JSON entry surfaces
        // only `oauth2_unnamed: true`, never the raw token strings.
        "UNNAMED-AT-AAA",
        "UNNAMED-RT-BBB",
        // Credential field names that would only appear if `App` were
        // serialized directly or via `From<&App>`.
        "client_secret",
        "access_token",
        "refresh_token",
        "consumer_secret",
        "token_secret",
    ];
    for needle in banned {
        assert!(
            !stdout.contains(needle),
            "[{context}] credential leak: stdout contains {needle:?}\n--- stdout ---\n{stdout}"
        );
    }
}

#[test]
fn test_auth_status_json_excludes_all_credentials() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    populate_credentialed_store(&store);

    let (code, stdout, stderr) = run_at(&store, &["xr", "--output", "json", "auth", "status"]);
    assert_eq!(code, 0, "status failed; stderr: {stderr}");
    assert_no_credentials(&stdout, "auth status --output json");

    // Sanity: the JSON still carries the expected non-secret fields.
    let v = parse_json(&stdout);
    let arr = v.as_array().expect("status emits a JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "myapp");
    assert_eq!(arr[0]["client_id_hint"], "CLIENT-I");
    assert_eq!(arr[0]["default"], true);
    assert_eq!(arr[0]["oauth1"], true);
    assert_eq!(arr[0]["bearer"], true);
    assert_eq!(arr[0]["oauth2_users"], serde_json::json!(["alice"]));
}

#[test]
fn test_auth_apps_list_json_excludes_all_credentials() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    populate_credentialed_store(&store);

    let (code, stdout, stderr) =
        run_at(&store, &["xr", "--output", "json", "auth", "apps", "list"]);
    assert_eq!(code, 0, "apps list failed; stderr: {stderr}");
    assert_no_credentials(&stdout, "auth apps list --output json");

    let v = parse_json(&stdout);
    let arr = v.as_array().expect("apps list emits a JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "myapp");
}

#[test]
fn test_redirect_uri_get_json_excludes_all_credentials() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    populate_credentialed_store(&store);

    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "apps",
            "redirect-uri",
            "get",
            "myapp",
        ],
    );
    assert_eq!(code, 0, "redirect-uri get failed; stderr: {stderr}");
    assert_no_credentials(&stdout, "auth apps redirect-uri get --output json");
}

#[test]
#[serial_test::serial]
fn test_auth_status_text_includes_redirect_uri_line() {
    // R24: status text output gains a `redirect_uri:` line per app.
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage from
    // another env-mutating test in the same binary.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    let (code, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
            "--redirect-uri",
            "http://localhost:7777/cb",
        ],
    );
    assert_eq!(code, 0);
    let (code_d, _, _) = run_at(&store, &["xr", "auth", "default", "myapp"]);
    assert_eq!(code_d, 0);

    let (code2, stdout2, stderr2) = run_at(&store, &["xr", "auth", "status"]);
    assert_eq!(code2, 0, "status failed; stderr: {stderr2}");
    assert!(
        stdout2.contains("redirect_uri: http://localhost:7777/cb [app config]"),
        "missing redirect_uri line with stored URI: {stdout2}"
    );
    assert!(
        !stdout2.contains("stored_redirect_uri:"),
        "stored_redirect_uri line should not appear without env override: {stdout2}"
    );
}

#[test]
#[serial_test::serial]
fn test_auth_status_text_default_built_in_when_no_stored_uri() {
    // R24: status text falls through to built-in default when no env, no stored.
    // `#[serial]` + explicit env removal guards against `REDIRECT_URI` leaking
    // from another env-mutating test in the same binary.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    let (code, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
        ],
    );
    assert_eq!(code, 0);

    let (code2, stdout2, _) = run_at(&store, &["xr", "auth", "status"]);
    assert_eq!(code2, 0);
    assert!(
        stdout2.contains("redirect_uri: http://localhost:8080/callback [built-in default]"),
        "expected built-in default redirect URI line: {stdout2}"
    );
}

#[test]
#[serial_test::serial]
fn test_auth_status_json_emits_app_config_source_and_no_stored_field() {
    // R21: source serializes as kebab-case; `redirect_uri_stored` is absent
    // when the env does not override the stored value.
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage that
    // would flip the asserted source to `env-var`.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    let (code, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abcdefgh1234",
            "--client-secret",
            "xyz",
            "--redirect-uri",
            "http://localhost:7777/cb",
        ],
    );
    assert_eq!(code, 0);
    let (code_d, _, _) = run_at(&store, &["xr", "auth", "default", "myapp"]);
    assert_eq!(code_d, 0);

    let (code2, stdout2, _) = run_at(&store, &["xr", "--output", "json", "auth", "status"]);
    assert_eq!(code2, 0);
    let v = parse_json(&stdout2);
    let arr = v.as_array().expect("status emits a JSON array");
    let entry = arr
        .iter()
        .find(|e| e["name"] == "myapp")
        .expect("myapp entry");
    assert_eq!(entry["redirect_uri"], "http://localhost:7777/cb");
    assert_eq!(entry["redirect_uri_source"], "app-config");
    assert_eq!(entry["client_id_hint"], "abcdefgh");
    assert_eq!(entry["default"], true);
    assert!(
        entry.get("redirect_uri_stored").is_none(),
        "redirect_uri_stored should be omitted when not env-overridden: {stdout2}"
    );
}

#[test]
#[serial_test::serial]
fn test_auth_status_json_env_override_surfaces_stored_field() {
    // R21 + R19: when REDIRECT_URI overrides the stored value, the JSON entry
    // includes `redirect_uri_stored` and `redirect_uri_source == "env-var"`.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    let (code, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "abc",
            "--client-secret",
            "xyz",
            "--redirect-uri",
            "http://localhost:7777/cb",
        ],
    );
    assert_eq!(code, 0);
    let (code_d, _, _) = run_at(&store, &["xr", "auth", "default", "myapp"]);
    assert_eq!(code_d, 0);

    unsafe {
        std::env::set_var("REDIRECT_URI", "https://override.example.com/cb");
    }
    let (code2, stdout2, _) = run_at(&store, &["xr", "--output", "json", "auth", "status"]);
    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    assert_eq!(code2, 0);
    let v = parse_json(&stdout2);
    let arr = v.as_array().expect("status emits a JSON array");
    let entry = arr
        .iter()
        .find(|e| e["name"] == "myapp")
        .expect("myapp entry");
    assert_eq!(entry["redirect_uri"], "https://override.example.com/cb");
    assert_eq!(entry["redirect_uri_source"], "env-var");
    assert_eq!(entry["redirect_uri_stored"], "http://localhost:7777/cb");
}

#[test]
#[serial_test::serial]
fn test_auth_status_json_default_flag_per_app() {
    // R21: with two apps, only the default app's entry has `default: true`.
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage from
    // a parallel env-mutating test (env source would not affect this
    // assertion, but the discipline keeps the snapshot stable).
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    let (c1, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "alpha",
            "--client-id",
            "aaa11111",
            "--client-secret",
            "sa",
        ],
    );
    assert_eq!(c1, 0);
    let (c2, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "beta",
            "--client-id",
            "bbb22222",
            "--client-secret",
            "sb",
        ],
    );
    assert_eq!(c2, 0);
    let (cd, _, _) = run_at(&store, &["xr", "auth", "default", "beta"]);
    assert_eq!(cd, 0);

    let (code, stdout, _) = run_at(&store, &["xr", "--output", "json", "auth", "status"]);
    assert_eq!(code, 0);
    let v = parse_json(&stdout);
    let arr = v.as_array().expect("status emits a JSON array");
    let mut alpha_default = None;
    let mut beta_default = None;
    for entry in arr {
        match entry["name"].as_str() {
            Some("alpha") => alpha_default = entry["default"].as_bool(),
            Some("beta") => beta_default = entry["default"].as_bool(),
            _ => {}
        }
    }
    assert_eq!(alpha_default, Some(false), "alpha should not be default");
    assert_eq!(beta_default, Some(true), "beta should be default");
}

#[test]
#[serial_test::serial]
fn test_auth_apps_list_json_shape_per_app() {
    // R21 (list): per-app object carries `name`, `client_id_hint`,
    // `redirect_uri`, `redirect_uri_source`, `oauth2_users`, `oauth1`,
    // `bearer`, `default`.
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage that
    // would flip the asserted `redirect_uri_source` to `env-var`.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    let (c1, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "alpha",
            "--client-id",
            "aaa11111",
            "--client-secret",
            "sa",
        ],
    );
    assert_eq!(c1, 0);
    let (cd, _, _) = run_at(&store, &["xr", "auth", "default", "alpha"]);
    assert_eq!(cd, 0);

    let (code, stdout, _) = run_at(&store, &["xr", "--output", "json", "auth", "apps", "list"]);
    assert_eq!(code, 0);
    let v = parse_json(&stdout);
    let arr = v.as_array().expect("apps list emits a JSON array");
    let entry = &arr[0];
    for field in [
        "name",
        "client_id_hint",
        "redirect_uri",
        "redirect_uri_source",
        "oauth2_users",
        "oauth1",
        "bearer",
        "default",
    ] {
        assert!(
            entry.get(field).is_some(),
            "missing field {field}: {stdout}"
        );
    }
    assert_eq!(entry["redirect_uri_source"], "built-in-default");
    assert_eq!(entry["oauth1"], false);
    assert_eq!(entry["bearer"], false);
}

#[test]
#[serial_test::serial]
fn test_auth_status_text_snapshot_two_apps_default_case() {
    // Text-output regression: locks in the new `redirect_uri:` line per app
    // for the no-env, no-stored-URI case across two user-added apps. A fresh
    // `TokenStore` also seeds a `"default"` placeholder app, so the iteration
    // emits a line for each of the three apps; the snapshot anchors them all.
    // `#[serial]` + env removal guards against `REDIRECT_URI` leaking across
    // tests in the same binary.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    unsafe {
        std::env::remove_var("REDIRECT_URI");
    }

    let (c1, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "alpha",
            "--client-id",
            "aaa11111",
            "--client-secret",
            "sa",
        ],
    );
    assert_eq!(c1, 0);
    let (c2, _, _) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "beta",
            "--client-id",
            "bbb22222",
            "--client-secret",
            "sb",
        ],
    );
    assert_eq!(c2, 0);
    let (cd, _, _) = run_at(&store, &["xr", "auth", "default", "alpha"]);
    assert_eq!(cd, 0);

    let (code, stdout, _) = run_at(&store, &["xr", "auth", "status"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("alpha"), "missing alpha row: {stdout}");
    assert!(stdout.contains("beta"), "missing beta row: {stdout}");
    let redirect_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("redirect_uri:"))
        .collect();
    assert!(
        redirect_lines.len() >= 2,
        "expected at least one redirect_uri line per user app: {stdout}"
    );
    for line in &redirect_lines {
        assert!(
            line.contains("http://localhost:8080/callback"),
            "expected built-in default in line: {line}"
        );
        assert!(
            line.contains("[built-in default]"),
            "expected built-in default label: {line}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// U3: resolve_my_user_id --username fallback
//
// Drives the `like` shortcut through wiremock to verify the resolver picks
// `/2/users/by/username/<u>` when `-u` is non-empty and `/2/users/me` when
// empty. A single shortcut is representative because all 18 engagement
// handlers route through the same resolver.
// ═══════════════════════════════════════════════════════════════════════════

/// Wiremock harness mirroring `tests/auth_remote_tests.rs::TestServer`.
///
/// Owns the runtime so the `MockServer` (started inside it) outlives every
/// async mount call. The leaked `&'static MockServer` keeps the server alive
/// for the duration of the test without a manual `Arc` dance.
struct CliMockServer {
    _rt: tokio::runtime::Runtime,
    server: &'static MockServer,
    uri: String,
}

impl CliMockServer {
    fn new() -> Self {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let server = rt.block_on(async {
            let s = MockServer::start().await;
            Box::leak(Box::new(s))
        });
        let uri = server.uri();
        Self {
            _rt: rt,
            server,
            uri,
        }
    }

    fn mount(&self, mock: Mock) {
        self._rt.block_on(async {
            mock.mount(self.server).await;
        });
    }

    fn uri(&self) -> &str {
        &self.uri
    }
}

/// Seeds a tempdir-rooted store with a single app carrying a bearer token.
///
/// `--auth app` resolves through `get_bearer_token_header` which reads the
/// bearer slot on the active app, so this is the minimal credential shape
/// the `like` POST needs to leave the resolver and reach the mocked endpoint.
fn populate_bearer_store(store_path: &Path) {
    use xurl::store::TokenStore;
    let mut ts = TokenStore::new_with_path(store_path.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_bearer_token_for_app("myapp", "BEARER-TOKEN-VALUE")
        .expect("save_bearer");
    ts.set_default_app("myapp").expect("set_default_app");
    let _ = ts.remove_app("default");
}

#[test]
#[serial_test::serial]
fn test_like_with_username_flag_calls_lookup_by_username() {
    // `-u alice` routes through `/2/users/by/username/alice`; the resolved id
    // (67890) drives the like POST. `expect(1)` on each mock fails the test
    // (on server drop) if either endpoint is hit zero times or > 1.
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/by/username/alice"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "67890", "username": "alice", "name": "Alice"}
            })))
            .expect(1),
    );
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/users/67890/likes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"liked": true}
            })))
            .expect(1),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
    }
    let (code, stdout, stderr) = run_at(
        &store,
        &["xr", "like", "12345", "-u", "alice", "--auth", "app"],
    );
    unsafe {
        std::env::remove_var("API_BASE_URL");
    }

    assert_eq!(code, 0, "like failed; stderr: {stderr}; stdout: {stdout}");
}

#[test]
#[serial_test::serial]
fn test_like_without_username_flag_calls_me() {
    // No `-u` → empty `opts.username` → resolver hits `/2/users/me`.
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "111", "username": "self", "name": "Self"}
            })))
            .expect(1),
    );
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/users/111/likes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"liked": true}
            })))
            .expect(1),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
    }
    let (code, stdout, stderr) = run_at(&store, &["xr", "like", "12345", "--auth", "app"]);
    unsafe {
        std::env::remove_var("API_BASE_URL");
    }

    assert_eq!(code, 0, "like failed; stderr: {stderr}; stdout: {stdout}");
}

#[test]
#[serial_test::serial]
fn test_like_with_username_flag_lookup_404() {
    // `-u alice` + 404 from lookup → resolver bubbles the transport error up
    // and the like POST is never issued. Mock the lookup only; if anything
    // else hits the server, wiremock returns 404 by default and the test
    // still surfaces a non-zero exit.
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/by/username/alice"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "title": "Not Found",
                "detail": "Could not find user with username: [alice]",
                "status": 404,
                "type": "https://api.x.com/2/problems/resource-not-found"
            })))
            .expect(1),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
    }
    let (code, _stdout, stderr) = run_at(
        &store,
        &["xr", "like", "12345", "-u", "alice", "--auth", "app"],
    );
    unsafe {
        std::env::remove_var("API_BASE_URL");
    }

    assert_ne!(
        code, 0,
        "expected non-zero exit when lookup 404s; stderr: {stderr}"
    );
}

#[test]
#[serial_test::serial]
fn test_like_with_empty_username_falls_back_to_me() {
    // `-u ""` collapses through `CommonFlags::to_call_options()` to an empty
    // `opts.username`, which falls into the `/me` branch. Documents the
    // current contract: a future change that treats `Some("")` differently
    // from `None` is intentional, not accidental.
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "222", "username": "self", "name": "Self"}
            })))
            .expect(1),
    );
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/users/222/likes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"liked": true}
            })))
            .expect(1),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
    }
    let (code, stdout, stderr) =
        run_at(&store, &["xr", "like", "12345", "-u", "", "--auth", "app"]);
    unsafe {
        std::env::remove_var("API_BASE_URL");
    }

    assert_eq!(code, 0, "like failed; stderr: {stderr}; stdout: {stdout}");
}

#[test]
#[serial_test::serial]
fn test_like_with_at_prefix_username_strips_at() {
    // `lookup_user`'s internal `resolve_username` strips a leading `@` before
    // building the path. The mock matches the bare handle; if the strip were
    // skipped, wiremock would 404 the `@alice` request and exit would be
    // non-zero.
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/by/username/alice"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "333", "username": "alice", "name": "Alice"}
            })))
            .expect(1),
    );
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/users/333/likes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"liked": true}
            })))
            .expect(1),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
    }
    let (code, stdout, stderr) = run_at(
        &store,
        &["xr", "like", "12345", "-u", "@alice", "--auth", "app"],
    );
    unsafe {
        std::env::remove_var("API_BASE_URL");
    }

    assert_eq!(code, 0, "like failed; stderr: {stderr}; stdout: {stdout}");
}

// ═══════════════════════════════════════════════════════════════════════════
// `auth oauth2 [USERNAME]` positional (U4)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_oauth2_positional_username_threads_through() {
    // Parse-level assertion: `xr auth oauth2 alice --no-browser --step 1`
    // must succeed at the clap layer with the positional bound to `alice`.
    // Driving the full OAuth2 flow end-to-end lives in `auth_remote_tests.rs`
    // (`exchange_code_for_token_nonempty_username_skips_me_and_saves_named`).
    use clap::Parser;

    let parsed = xurl::cli::Cli::try_parse_from([
        "xr",
        "auth",
        "oauth2",
        "alice",
        "--no-browser",
        "--step",
        "1",
    ])
    .expect("positional + --no-browser --step 1 must parse");

    let Some(xurl::cli::Commands::Auth { command }) = parsed.command else {
        panic!("expected Auth subcommand");
    };
    match command {
        xurl::cli::AuthCommands::Oauth2 {
            no_browser,
            step,
            auth_url,
            username,
        } => {
            assert!(no_browser, "--no-browser should be set");
            assert_eq!(step, Some(1));
            assert!(auth_url.is_none());
            assert_eq!(
                username.as_deref(),
                Some("alice"),
                "positional username must bind to `alice`",
            );
        }
        other => panic!("expected AuthCommands::Oauth2, got {other:?}"),
    }
}

#[test]
fn test_oauth2_positional_invalid_extra_args() {
    // Two positionals on `auth oauth2` must fail with a clap usage error
    // (exit code 2 via the runner).
    let (code, _stdout, stderr) = run_isolated(&["xr", "auth", "oauth2", "alice", "bob"]);
    assert_eq!(
        code, 2,
        "expected clap usage exit code 2 for extra positional; stderr: {stderr}"
    );
    assert!(
        stderr.contains("error"),
        "stderr should contain clap error text: {stderr}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// U5: credential-less-default warning + status/list unnamed-slot rendering
// ═══════════════════════════════════════════════════════════════════════════

/// Seeds a store where the default app `default` has no `client_id` but
/// another registered app `myapp` does — the configuration that triggers the
/// credential-less-default warning per R13.
fn seed_credential_less_default_with_alternative(store_path: &Path) {
    use xurl::store::TokenStore;
    let mut ts = TokenStore::new_with_path(store_path.to_str().expect("utf-8 path"));
    // `TokenStore::new_with_path` seeds an empty `default` placeholder app
    // with empty credentials; that is exactly what we want here.
    ts.add_app("myapp", "MYAPP-CLIENT-ID", "MYAPP-SECRET")
        .expect("add_app");
    // The default app remains `default` (the placeholder).
}

/// Seeds a store where only the default app `default` exists with no
/// credentials; no credentialed alternative — the warning must NOT fire.
fn seed_credential_less_default_only(store_path: &Path) {
    use xurl::store::TokenStore;
    let _ts = TokenStore::new_with_path(store_path.to_str().expect("utf-8 path"));
    // `new_with_path` already seeds an empty `default` app; nothing else to do.
}

/// Seeds a store where the default app `default` HAS credentials — the
/// warning must NOT fire.
fn seed_default_with_credentials(store_path: &Path) {
    use xurl::store::TokenStore;
    let mut ts = TokenStore::new_with_path(store_path.to_str().expect("utf-8 path"));
    ts.update_app("default", "DEFAULT-CLIENT-ID", "DEFAULT-SECRET")
        .expect("update_app");
}

#[test]
fn test_credential_less_default_warning_fires() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_credential_less_default_with_alternative(&store);

    // `--no-browser --step 1` emits the auth URL and returns; it does NOT
    // touch the network or write a token. The credential-less-default check
    // runs BEFORE this dispatch.
    let (_code, _stdout, stderr) = run_at(
        &store,
        &["xr", "auth", "oauth2", "--no-browser", "--step", "1"],
    );
    assert!(
        stderr.contains("warning: --app not specified"),
        "stderr should contain credential-less-default warning; got: {stderr}"
    );
    assert!(
        stderr.contains("--app myapp"),
        "stderr should reference the credentialed alternative `myapp`; got: {stderr}"
    );
}

#[test]
fn test_credential_less_default_warning_suppressed_by_explicit_app() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_credential_less_default_with_alternative(&store);

    let (_code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--app",
            "myapp",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "1",
        ],
    );
    assert!(
        !stderr.contains("warning: --app not specified"),
        "explicit `--app` must suppress the warning; got stderr: {stderr}"
    );
}

#[test]
fn test_credential_less_default_warning_suppressed_no_credentialed_alternative() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_credential_less_default_only(&store);

    let (_code, _stdout, stderr) = run_at(
        &store,
        &["xr", "auth", "oauth2", "--no-browser", "--step", "1"],
    );
    assert!(
        !stderr.contains("warning: --app not specified"),
        "warning must NOT fire when no credentialed alternative exists; got stderr: {stderr}"
    );
}

#[test]
fn test_credential_less_default_warning_suppressed_when_default_has_credentials() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_default_with_credentials(&store);

    let (_code, _stdout, stderr) = run_at(
        &store,
        &["xr", "auth", "oauth2", "--no-browser", "--step", "1"],
    );
    assert!(
        !stderr.contains("warning: --app not specified"),
        "warning must NOT fire when the default app already has credentials; got stderr: {stderr}"
    );
}

/// Seeds a store with one app `myapp` carrying an unnamed OAuth2 token and
/// no named OAuth2 entries.
fn seed_app_with_unnamed_oauth2(store_path: &Path) {
    use xurl::store::TokenStore;
    let mut ts = TokenStore::new_with_path(store_path.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "MYAPP-CLIENT-ID", "MYAPP-SECRET")
        .expect("add_app");
    ts.save_oauth2_token_unnamed_for_app(
        "myapp",
        "UNNAMED-AT-AAA",
        "UNNAMED-RT-BBB",
        1_900_000_000,
    )
    .expect("save_oauth2_unnamed");
    ts.set_default_app("myapp").expect("set_default_app");
    let _ = ts.remove_app("default");
}

#[test]
fn test_status_text_shows_unnamed_oauth2() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_app_with_unnamed_oauth2(&store);

    let (code, stdout, stderr) = run_at(&store, &["xr", "auth", "status"]);
    assert_eq!(code, 0, "auth status failed; stderr: {stderr}");
    assert!(
        stdout.contains("oauth2: (unknown user)"),
        "status text should render `oauth2: (unknown user)` for the unnamed slot; got:\n{stdout}"
    );
}

#[test]
fn test_status_json_emits_oauth2_unnamed_true() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_app_with_unnamed_oauth2(&store);

    let (code, stdout, stderr) = run_at(&store, &["xr", "--output", "json", "auth", "status"]);
    assert_eq!(
        code, 0,
        "auth status --output json failed; stderr: {stderr}"
    );
    let v = parse_json(&stdout);
    let arr = v.as_array().expect("status emits a JSON array");
    let entry = arr
        .iter()
        .find(|e| e["name"] == "myapp")
        .expect("myapp entry present");
    assert_eq!(
        entry["oauth2_unnamed"],
        serde_json::Value::Bool(true),
        "oauth2_unnamed must be true; got entry: {entry}"
    );
}

#[test]
fn test_status_json_omits_oauth2_unnamed_when_false() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    // Reuse the bearer-only fixture: no named OAuth2, no unnamed slot.
    populate_bearer_store(&store);

    let (code, stdout, stderr) = run_at(&store, &["xr", "--output", "json", "auth", "status"]);
    assert_eq!(
        code, 0,
        "auth status --output json failed; stderr: {stderr}"
    );
    let v = parse_json(&stdout);
    let arr = v.as_array().expect("status emits a JSON array");
    let entry = arr
        .iter()
        .find(|e| e["name"] == "myapp")
        .expect("myapp entry present");
    assert!(
        entry.get("oauth2_unnamed").is_none(),
        "oauth2_unnamed must be omitted when false; got entry: {entry}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// U10: `xr skill install` — agent bundle distribution
// ═══════════════════════════════════════════════════════════════════════════
//
// Hermetic by construction: every invocation that touches the host map runs
// with `HOME` redirected to a fresh `TempDir`, so the tests never read or
// write the developer's real `~/.claude/skills/...`.

/// Helper: run `xr` with a caller-supplied `HOME` env var. Mirrors
/// `run_isolated` but also overrides `HOME` for the duration of the call. The
/// child process is `cli::run_with_store_path`, which reads `HOME` only via
/// `skill_install::expand_tilde`. Tests serialize via `serial_test` because
/// process-wide env mutation races otherwise.
fn run_with_home(args: &[&str], home: Option<&str>) -> (i32, String, String) {
    use std::sync::Mutex;
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let prior = std::env::var_os("HOME");
    // SAFETY: ENV_LOCK serialises all env mutations across these tests.
    unsafe {
        match home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }
    let result = run_isolated(args);
    // SAFETY: see above.
    unsafe {
        match prior {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }
    result
}

#[test]
fn skill_install_help_advertises_host_all_dry_run() {
    let (code, stdout, stderr) = run_isolated(&["xr", "skill", "install", "--help"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("--all"),
        "--all missing from help: {stdout}"
    );
    assert!(
        stdout.contains("--dry-run"),
        "--dry-run missing from help: {stdout}"
    );
    assert!(
        stdout.contains("HOST") || stdout.contains("[HOST]") || stdout.contains("host"),
        "host arg missing from help: {stdout}"
    );
    assert!(
        stdout.contains("claude_code"),
        "claude_code possible value missing from help: {stdout}"
    );
}

#[test]
fn skill_install_dry_run_emits_envelope_without_spawning_git() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    let (code, stdout, stderr) = run_with_home(
        &[
            "xr",
            "skill",
            "install",
            "claude_code",
            "--dry-run",
            "--output",
            "json",
        ],
        Some(&home),
    );
    assert_eq!(code, 0, "expected 0 for dry-run; stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON envelope");
    assert_eq!(v["status"], "dry_run", "envelope: {v}");
    assert_eq!(v["host"], "claude_code");
    assert_eq!(v["would_succeed"], true);
    assert_eq!(v["exit_code"], 0);
    assert_eq!(v["action"], "skill-install");
    let preview = v["command_preview"]
        .as_str()
        .expect("command_preview string");
    assert!(
        preview.starts_with("git clone --depth 1 "),
        "command_preview shape unexpected: {preview}"
    );
    assert!(
        preview.contains("github.com/brettdavies/xurl-rs-skill.git"),
        "command_preview missing repo URL: {preview}"
    );
    let install_dir = v["install_dir"].as_str().expect("install_dir string");
    assert!(
        install_dir.contains(".claude/skills/xurl-rs"),
        "install_dir not under .claude/skills/xurl-rs: {install_dir}"
    );
    // The dry-run path must not have created the destination — it only
    // resolves and reports.
    assert!(
        !std::path::Path::new(install_dir).exists(),
        "dry-run created the destination directory: {install_dir}"
    );
}

#[test]
fn skill_install_existing_non_empty_destination_errors() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    // Pre-populate the destination so the conflict check fires.
    let dest = tmp.path().join(".claude").join("skills").join("xurl-rs");
    std::fs::create_dir_all(&dest).expect("mkdir -p dest");
    std::fs::write(dest.join("placeholder"), b"x").expect("write placeholder");

    let (code, stdout, stderr) = run_with_home(
        &["xr", "skill", "install", "claude_code", "--output", "json"],
        Some(&home),
    );
    assert_eq!(code, 1, "expected 1 for dest-not-empty; stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON envelope");
    assert_eq!(v["status"], "error");
    assert_eq!(v["reason"], "destination-not-empty");
    assert_eq!(v["exit_code"], 1);
    assert_eq!(v["destination_status"], "non-empty-dir");
}

#[test]
fn skill_install_all_dry_run_lists_every_host() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    let (code, stdout, stderr) = run_with_home(
        &[
            "xr",
            "skill",
            "install",
            "--all",
            "--dry-run",
            "--output",
            "json",
        ],
        Some(&home),
    );
    assert_eq!(code, 0, "expected 0 for --all dry-run; stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON envelope");
    assert_eq!(v["status"], "dry_run");
    assert_eq!(v["action"], "skill-install");
    let arr = v["installations"]
        .as_array()
        .expect("installations is an array");
    let host_names: Vec<&str> = arr
        .iter()
        .map(|e| e["host"].as_str().expect("host string"))
        .collect();
    for expected in xurl::skill_install::KNOWN_HOSTS {
        assert!(
            host_names.contains(expected),
            "host {expected} missing from --all envelope; got {host_names:?}"
        );
    }
}

#[test]
fn skill_install_home_unset_emits_home_not_set_reason() {
    let (code, stdout, stderr) = run_with_home(
        &["xr", "skill", "install", "claude_code", "--output", "json"],
        None,
    );
    assert_eq!(code, 1, "expected 1 for HOME unset; stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON envelope");
    assert_eq!(v["status"], "error");
    assert_eq!(v["reason"], "home-not-set");
    assert_eq!(v["exit_code"], 1);
}

#[test]
fn skill_install_no_args_lists_supported_hosts() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    let (code, stdout, stderr) = run_with_home(&["xr", "skill", "install"], Some(&home));
    assert_eq!(
        code, 2,
        "expected 2 (usage error) for missing host; stderr: {stderr}"
    );
    for expected in xurl::skill_install::KNOWN_HOSTS {
        assert!(
            stdout.contains(expected),
            "host {expected} missing from text listing: {stdout}"
        );
    }
}

#[test]
fn skill_install_no_args_json_lists_supported_hosts_in_envelope() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    let (code, stdout, stderr) =
        run_with_home(&["xr", "skill", "install", "--output", "json"], Some(&home));
    assert_eq!(code, 2, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON envelope");
    assert_eq!(v["status"], "error");
    assert_eq!(v["reason"], "missing-host");
    assert_eq!(v["exit_code"], 2);
    let hosts = v["known_hosts"]
        .as_array()
        .expect("known_hosts is an array");
    let names: Vec<&str> = hosts.iter().map(|h| h.as_str().expect("str")).collect();
    for expected in xurl::skill_install::KNOWN_HOSTS {
        assert!(names.contains(expected), "missing {expected}: {names:?}");
    }
}

#[test]
fn skill_install_dry_run_text_output_is_single_line_command() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    let (code, stdout, stderr) = run_with_home(
        &["xr", "skill", "install", "claude_code", "--dry-run"],
        Some(&home),
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let line = stdout.trim();
    assert!(
        line.starts_with("git clone --depth 1 "),
        "text dry-run output shape unexpected: {line}"
    );
    assert!(
        line.contains("xurl-rs-skill.git"),
        "text dry-run missing repo URL: {line}"
    );
}

#[test]
fn skill_install_dest_is_regular_file_errors() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    // Plant a regular file at the destination path.
    let dest_parent = tmp.path().join(".claude").join("skills");
    std::fs::create_dir_all(&dest_parent).expect("mkdir -p parent");
    std::fs::write(dest_parent.join("xurl-rs"), b"i'm a file").expect("write file");

    let (code, stdout, stderr) = run_with_home(
        &["xr", "skill", "install", "claude_code", "--output", "json"],
        Some(&home),
    );
    assert_eq!(code, 1, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON envelope");
    assert_eq!(v["status"], "error");
    assert_eq!(v["reason"], "destination-is-file");
    assert_eq!(v["destination_status"], "file");
}

// ═══════════════════════════════════════════════════════════════════════════
// U6: P3 progressive help — `after_help` on every subcommand + xr examples
// ═══════════════════════════════════════════════════════════════════════════

/// Returns the index of the first line within `lines[start..]` that starts
/// (after trimming) with `"Examples:"`, or `None` if not present.
fn examples_line_index(lines: &[&str]) -> Option<usize> {
    lines
        .iter()
        .position(|l| l.trim_start().starts_with("Examples:"))
}

#[test]
fn test_post_help_includes_paired_text_and_json_examples() {
    let (code, stdout, stderr) = run_isolated(&["xr", "post", "--help"]);
    assert_eq!(code, 0, "expected 0 for post --help; stderr: {stderr}");
    assert!(
        stdout.contains("Examples:"),
        "post --help must include an 'Examples:' block; got:\n{stdout}"
    );
    assert!(
        stdout.contains("--output json"),
        "post --help must include at least one --output json example; got:\n{stdout}"
    );

    // Paired text + JSON within 5 lines: walk the Examples block, find a text
    // invocation, and confirm a --output json example appears within 5
    // following lines.
    let lines: Vec<&str> = stdout.lines().collect();
    let start = examples_line_index(&lines).expect("Examples block must exist");
    let mut paired = false;
    for i in start..lines.len() {
        let line = lines[i];
        if line.contains("xr post ") && !line.contains("--output") {
            let window_end = (i + 6).min(lines.len());
            if lines[i + 1..window_end]
                .iter()
                .any(|l| l.contains("--output json"))
            {
                paired = true;
                break;
            }
        }
    }
    assert!(
        paired,
        "post --help must pair a text invocation with --output json within 5 lines; got:\n{stdout}"
    );
}

#[test]
fn test_auth_oauth2_help_shows_no_browser_example() {
    let (code, stdout, stderr) = run_isolated(&["xr", "auth", "oauth2", "--help"]);
    assert_eq!(
        code, 0,
        "expected 0 for auth oauth2 --help; stderr: {stderr}"
    );
    assert!(
        stdout.contains("Examples:"),
        "auth oauth2 --help must include an Examples block; got:\n{stdout}"
    );
    assert!(
        stdout.contains("--no-browser"),
        "auth oauth2 --help must advertise the --no-browser headless flow; got:\n{stdout}"
    );
    assert!(
        stdout.contains("--step 1"),
        "auth oauth2 --help must show the headless step 1 invocation; got:\n{stdout}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// U7: --force, --dry-run, --limit mutation-safety envelopes
//
// The `wiremock::Mock::expect(0)` calls below double as no-HTTP guards: any
// stray API call would fail the test when the mock-server drop checks the
// expectation. The dry-run-only tests need no mounted endpoints because the
// envelope path short-circuits before HTTP.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[serial_test::serial]
fn test_delete_no_interactive_without_force_emits_confirmation_required_envelope() {
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    // Mount the delete endpoint with expect(0) — verifies no HTTP fires.
    ts.mount(
        Mock::given(method("DELETE"))
            .and(path("/2/tweets/12345"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
        std::env::set_var("XURL_NO_INTERACTIVE", "1");
    }
    let (code, _stdout, stderr) = run_at(
        &store,
        &["xr", "--output", "json", "delete", "12345", "--auth", "app"],
    );
    unsafe {
        std::env::remove_var("API_BASE_URL");
        std::env::remove_var("XURL_NO_INTERACTIVE");
    }

    assert_eq!(
        code, 1,
        "delete without --force under --no-interactive must exit 1; stderr: {stderr}"
    );
    // The envelope goes to stderr per `print_confirmation_required`.
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is valid JSON envelope");
    assert_eq!(envelope["status"], "error");
    assert_eq!(envelope["reason"], "confirmation-required");
    assert_eq!(envelope["exit_code"], 1);
}

#[test]
#[serial_test::serial]
fn test_delete_force_no_interactive_calls_api_and_succeeds() {
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    ts.mount(
        Mock::given(method("DELETE"))
            .and(path("/2/tweets/12345"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"deleted": true}
            })))
            .expect(1),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
        std::env::set_var("XURL_NO_INTERACTIVE", "1");
    }
    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr", "--output", "json", "delete", "12345", "--force", "--auth", "app",
        ],
    );
    unsafe {
        std::env::remove_var("API_BASE_URL");
        std::env::remove_var("XURL_NO_INTERACTIVE");
    }

    assert_eq!(
        code, 0,
        "delete --force --no-interactive must exit 0; stderr: {stderr}; stdout: {stdout}"
    );
    let v = parse_json(&stdout);
    assert_eq!(v["data"]["deleted"], serde_json::Value::Bool(true));
}

#[test]
#[serial_test::serial]
fn test_post_dry_run_emits_envelope_and_skips_api() {
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    // No HTTP must fire.
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
        std::env::set_var("XURL_DRY_RUN", "1");
    }
    let (code, stdout, stderr) = run_at(
        &store,
        &["xr", "--output", "json", "post", "Hello", "--auth", "app"],
    );
    unsafe {
        std::env::remove_var("API_BASE_URL");
        std::env::remove_var("XURL_DRY_RUN");
    }

    assert_eq!(
        code, 0,
        "post --dry-run must exit 0; stderr: {stderr}; stdout: {stdout}"
    );
    let v = parse_json(&stdout);
    assert_eq!(v["status"], "dry_run");
    assert_eq!(v["would_succeed"], serde_json::Value::Bool(true));
    assert_eq!(v["exit_code"], 0);
    assert_eq!(v["command"], "post");
    assert_eq!(v["body"], "Hello");
}

#[test]
#[serial_test::serial]
fn test_post_empty_body_dry_run_reports_empty_body_reason() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    unsafe {
        std::env::set_var("XURL_DRY_RUN", "1");
    }
    let (code, stdout, stderr) = run_at(
        &store,
        &["xr", "--output", "json", "post", "", "--auth", "app"],
    );
    unsafe {
        std::env::remove_var("XURL_DRY_RUN");
    }

    assert_eq!(
        code, 0,
        "post '' --dry-run must exit 0 (envelope, not error); stderr: {stderr}; stdout: {stdout}"
    );
    let v = parse_json(&stdout);
    assert_eq!(v["status"], "dry_run");
    assert_eq!(v["would_succeed"], serde_json::Value::Bool(false));
    assert_eq!(v["reason"], "empty-body");
    assert_eq!(v["exit_code"], 1);
}

#[test]
#[serial_test::serial]
fn test_post_body_too_long_dry_run_reports_body_too_long_reason() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    // 281 chars: one past the 280-char limit.
    let too_long: String = std::iter::repeat_n('x', 281).collect();
    unsafe {
        std::env::set_var("XURL_DRY_RUN", "1");
    }
    let (code, stdout, _stderr) = run_at(
        &store,
        &["xr", "--output", "json", "post", &too_long, "--auth", "app"],
    );
    unsafe {
        std::env::remove_var("XURL_DRY_RUN");
    }

    assert_eq!(code, 0);
    let v = parse_json(&stdout);
    assert_eq!(v["status"], "dry_run");
    assert_eq!(v["would_succeed"], serde_json::Value::Bool(false));
    assert_eq!(v["reason"], "body-too-long");
}

#[test]
#[serial_test::serial]
fn test_search_global_limit_50_respected() {
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    // `max_results=50` MUST appear on the query string when --limit 50 is set.
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .and(wiremock::matchers::query_param("max_results", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": []
            })))
            .expect(1),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
    }
    let (code, _stdout, stderr) = run_at(
        &store,
        &["xr", "--limit", "50", "search", "x", "--auth", "app"],
    );
    unsafe {
        std::env::remove_var("API_BASE_URL");
    }

    assert_eq!(code, 0, "search --limit 50 failed; stderr: {stderr}");
}

#[test]
#[serial_test::serial]
fn test_search_global_limit_500_clamped_to_100() {
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    // 500 must be clamped to 100 before the API call.
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .and(wiremock::matchers::query_param("max_results", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": []
            })))
            .expect(1),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
    }
    let (code, _stdout, stderr) = run_at(
        &store,
        &["xr", "--limit", "500", "search", "x", "--auth", "app"],
    );
    unsafe {
        std::env::remove_var("API_BASE_URL");
    }

    assert_eq!(code, 0, "search --limit 500 must clamp; stderr: {stderr}");
}

#[test]
#[serial_test::serial]
fn test_search_per_cmd_max_results_overrides_global_limit() {
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    // `-n 20` MUST take precedence over `--limit 80`.
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .and(wiremock::matchers::query_param("max_results", "20"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": []
            })))
            .expect(1),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
    }
    let (code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr", "--limit", "80", "search", "x", "-n", "20", "--auth", "app",
        ],
    );
    unsafe {
        std::env::remove_var("API_BASE_URL");
    }

    assert_eq!(
        code, 0,
        "per-cmd -n must override --limit; stderr: {stderr}"
    );
}

#[test]
fn test_examples_subcommand_runs() {
    let (code, stdout, stderr) = run_isolated(&["xr", "examples"]);
    assert_eq!(code, 0, "expected 0 for xr examples; stderr: {stderr}");
    assert!(!stdout.is_empty(), "examples output must be non-empty");
    for section in [
        "AUTHENTICATE:",
        "POST AND READ:",
        "MANAGE SOCIAL GRAPH:",
        "MEDIA UPLOAD:",
        "INSPECT SCHEMAS:",
    ] {
        assert!(
            stdout.contains(section),
            "examples output missing section {section}; got:\n{stdout}"
        );
    }
}

#[test]
fn test_search_help_demonstrates_env_var_precedence() {
    let (code, stdout, stderr) = run_isolated(&["xr", "search", "--help"]);
    assert_eq!(code, 0, "expected 0 for search --help; stderr: {stderr}");
    assert!(
        stdout.contains("XURL_OUTPUT=json xr search"),
        "search --help must demo XURL_OUTPUT precedence; got:\n{stdout}"
    );
}

#[test]
#[serial_test::serial]
fn test_auth_clear_force_no_interactive_dry_run_envelope() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    unsafe {
        std::env::set_var("XURL_NO_INTERACTIVE", "1");
        std::env::set_var("XURL_DRY_RUN", "1");
    }
    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr", "--output", "json", "auth", "clear", "--all", "--force",
        ],
    );
    unsafe {
        std::env::remove_var("XURL_NO_INTERACTIVE");
        std::env::remove_var("XURL_DRY_RUN");
    }

    assert_eq!(
        code, 0,
        "auth clear --force --no-interactive --dry-run must exit 0; stderr: {stderr}"
    );
    let v = parse_json(&stdout);
    assert_eq!(v["status"], "dry_run");
    assert_eq!(v["command"], "auth-clear");
    assert_eq!(v["all"], serde_json::Value::Bool(true));
}

#[test]
#[serial_test::serial]
fn test_xurl_dry_run_env_var_engages_dry_run() {
    let ts = CliMockServer::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    // No HTTP must fire.
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0),
    );

    unsafe {
        std::env::set_var("API_BASE_URL", ts.uri());
        std::env::set_var("XURL_DRY_RUN", "1");
    }
    let (code, stdout, stderr) = run_at(
        &store,
        &["xr", "--output", "json", "post", "Hi", "--auth", "app"],
    );
    unsafe {
        std::env::remove_var("API_BASE_URL");
        std::env::remove_var("XURL_DRY_RUN");
    }

    assert_eq!(
        code, 0,
        "XURL_DRY_RUN=1 must engage dry-run; stderr: {stderr}; stdout: {stdout}"
    );
    let v = parse_json(&stdout);
    assert_eq!(v["status"], "dry_run");
}

#[test]
fn test_dry_run_help_advertised_on_post() {
    // Sanity: the help text MUST mention --dry-run so anc's p5-must-dry-run
    // gate sees the advertisement.
    let (code, stdout, _stderr) = run_isolated(&["xr", "post", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("--dry-run"),
        "post --help must advertise --dry-run; got: {stdout}"
    );
}

#[test]
fn test_root_help_lists_env_vars_and_exit_codes() {
    let (code, stdout, stderr) = run_isolated(&["xr", "--help"]);
    assert_eq!(code, 0, "expected 0 for --help; stderr: {stderr}");
    assert!(
        stdout.contains("ENVIRONMENT VARIABLES:"),
        "root --help must include ENVIRONMENT VARIABLES section; got:\n{stdout}"
    );
    assert!(
        stdout.contains("EXIT CODES:"),
        "root --help must include EXIT CODES section; got:\n{stdout}"
    );
    // Every env var the binary reads at the root level must appear.
    for env_var in [
        "XURL_OUTPUT",
        "XURL_QUIET",
        "XURL_NO_INTERACTIVE",
        "XURL_TIMEOUT",
        "XURL_COLOR",
        "XURL_VERBOSE",
        "XURL_APP",
        "REDIRECT_URI",
    ] {
        assert!(
            stdout.contains(env_var),
            "root --help must document {env_var}; got:\n{stdout}"
        );
    }
    assert!(
        stdout.contains("TTY behavior:") || stdout.contains("not a TTY"),
        "root --help must call out TTY-aware behavior; got:\n{stdout}"
    );
}

/// Walk every subcommand (recursively, including nested ones like
/// `auth oauth2`, `auth apps add`, `auth apps redirect-uri get`) and confirm
/// each `--help` carries an Examples block. Parametric coverage: a future
/// new subcommand without examples fails this test instead of silently
/// regressing the P3 audit.
#[test]
fn test_every_subcommand_help_has_examples_block() {
    use clap::CommandFactory;

    fn collect_paths(cmd: &clap::Command, prefix: &[String], out: &mut Vec<Vec<String>>) {
        for sub in cmd.get_subcommands() {
            let name = sub.get_name().to_string();
            // Skip clap's auto-generated `help` subcommand and aliases.
            if name == "help" {
                continue;
            }
            let mut path = prefix.to_vec();
            path.push(name);
            out.push(path.clone());
            collect_paths(sub, &path, out);
        }
    }

    let cli = xurl::cli::Cli::command();
    let mut paths: Vec<Vec<String>> = Vec::new();
    collect_paths(&cli, &[], &mut paths);
    assert!(!paths.is_empty(), "Cli must expose at least one subcommand");

    let mut missing: Vec<String> = Vec::new();
    for path in &paths {
        let mut args: Vec<String> = vec!["xr".to_string()];
        args.extend(path.iter().cloned());
        args.push("--help".to_string());
        let argv: Vec<&str> = args.iter().map(String::as_str).collect();
        let (code, stdout, stderr) = run_isolated(&argv);
        assert_eq!(
            code,
            0,
            "expected 0 for `{}` --help; stderr: {stderr}",
            path.join(" ")
        );
        if !stdout.contains("Examples:") {
            missing.push(path.join(" "));
        }
    }
    assert!(
        missing.is_empty(),
        "the following subcommands' --help lacks an Examples: block:\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn test_force_help_advertised_on_delete() {
    let (code, stdout, _stderr) = run_isolated(&["xr", "delete", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("--force"),
        "delete --help must advertise --force; got: {stdout}"
    );
}

#[test]
fn test_limit_help_advertised_globally() {
    let (code, stdout, _stderr) = run_isolated(&["xr", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("--limit"),
        "xr --help must advertise --limit globally; got: {stdout}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// U9: TTY-gated dialoguer + `--no-browser` env + headless auto-engage
// ═══════════════════════════════════════════════════════════════════════════

/// `xr auth default --no-interactive --output json` (no app_name supplied)
/// must emit the canonical `no-tty` envelope on stderr and skip any dialoguer
/// call.  Two apps are seeded so the picker would otherwise prompt.
#[test]
fn test_auth_default_no_interactive_emits_no_tty_envelope() {
    use xurl::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8"));
    ts.add_app("alpha", "ALPHA-CID", "ALPHA-SECRET")
        .expect("add alpha");
    ts.add_app("beta", "BETA-CID", "BETA-SECRET")
        .expect("add beta");
    drop(ts);

    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "default",
            "--no-interactive",
            "--output",
            "json",
        ],
    );
    assert_ne!(code, 0, "expected non-zero exit; stdout: {stdout}");
    let trimmed = stderr.trim();
    assert!(
        !trimmed.is_empty(),
        "stderr envelope must be present; got empty stderr"
    );
    let v: serde_json::Value =
        serde_json::from_str(trimmed).unwrap_or_else(|_| panic!("envelope must parse: {trimmed}"));
    assert_eq!(v["status"], "error", "envelope status: {trimmed}");
    assert_eq!(v["reason"], "no-tty", "envelope reason: {trimmed}");
    assert!(
        v["message"]
            .as_str()
            .map(|s| s.contains("default app") || s.contains("interactively"))
            .unwrap_or(false),
        "envelope message: {trimmed}"
    );
}

/// `xr auth default` without `--no-interactive` while the test harness's
/// stdin/stderr are not real TTYs (cargo test) must still skip dialoguer and
/// emit the `no-tty` envelope. The TTY check is independent of the
/// `--no-interactive` flag.
#[test]
fn test_auth_default_non_tty_emits_no_tty_envelope() {
    use xurl::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8"));
    ts.add_app("alpha", "ALPHA-CID", "ALPHA-SECRET")
        .expect("add alpha");
    ts.add_app("beta", "BETA-CID", "BETA-SECRET")
        .expect("add beta");
    drop(ts);

    let (code, _stdout, stderr) = run_at(&store, &["xr", "auth", "default", "--output", "json"]);
    assert_ne!(code, 0, "expected non-zero exit");
    let trimmed = stderr.trim();
    let v: serde_json::Value =
        serde_json::from_str(trimmed).unwrap_or_else(|_| panic!("envelope must parse: {trimmed}"));
    assert_eq!(v["status"], "error", "envelope status: {trimmed}");
    assert_eq!(v["reason"], "no-tty", "envelope reason: {trimmed}");
}

/// `xr auth oauth2 --help` must advertise the new `XURL_NO_BROWSER` env var.
#[test]
fn test_auth_oauth2_help_advertises_no_browser_env_var() {
    let (code, stdout, _stderr) = run_isolated(&["xr", "auth", "oauth2", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("XURL_NO_BROWSER"),
        "auth oauth2 --help must advertise XURL_NO_BROWSER env var; got:\n{stdout}"
    );
}

/// `xr auth oauth2 --no-browser --output json` (no `--step`) emits the
/// canonical `{"status":"awaiting_callback","url":"..."}` envelope on stdout
/// and exits 0; the user is expected to invoke step 2 separately. Validates
/// the U9 "explicit --no-browser without --step" auto-promotion to step 1.
///
/// Uses a subprocess with `HOME` redirected to a tempdir because the OAuth2
/// step-1 pending-state file lives at `$HOME/.xurl.pending` — the library
/// entrypoint does not isolate that path, so an in-process call would
/// pollute the user's real home directory.
#[test]
fn test_auth_oauth2_no_browser_emits_awaiting_callback_envelope() {
    let tmp = TempDir::new().expect("tempdir");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_xr"))
        .env("HOME", tmp.path())
        .env_remove("XURL_NO_BROWSER")
        .env_remove("XURL_OUTPUT")
        // Other tests in this binary set `XURL_DRY_RUN=1` to exercise the
        // dry-run envelope; that env var leaks into the subprocess unless we
        // clear it here, which would short-circuit U9's awaiting_callback
        // path through the U7 dry-run branch.
        .env_remove("XURL_DRY_RUN")
        .args(["auth", "oauth2", "--no-browser", "--output", "json"])
        .output()
        .expect("spawn xr");
    assert!(
        output.status.success(),
        "expected 0 for --no-browser; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    let v: serde_json::Value =
        serde_json::from_str(trimmed).unwrap_or_else(|_| panic!("envelope must parse: {trimmed}"));
    assert_eq!(v["status"], "awaiting_callback", "status: {trimmed}");
    assert!(v["url"].is_string(), "url present: {trimmed}");
    let url = v["url"].as_str().expect("url string");
    assert!(
        url.contains("oauth2/authorize"),
        "url must point at OAuth2 authorize endpoint: {url}"
    );
}

/// `XURL_NO_BROWSER=1 xr auth oauth2 --output json` is equivalent to passing
/// `--no-browser` explicitly — env-var routing for headless runners.
#[test]
fn test_auth_oauth2_xurl_no_browser_env_engages_headless_flow() {
    let tmp = TempDir::new().expect("tempdir");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_xr"))
        .env("HOME", tmp.path())
        .env("XURL_NO_BROWSER", "1")
        .env_remove("XURL_OUTPUT")
        .env_remove("XURL_DRY_RUN")
        .args(["auth", "oauth2", "--output", "json"])
        .output()
        .expect("spawn xr");
    assert!(
        output.status.success(),
        "XURL_NO_BROWSER auto-engage must succeed; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    let v: serde_json::Value =
        serde_json::from_str(trimmed).unwrap_or_else(|_| panic!("envelope must parse: {trimmed}"));
    assert_eq!(v["status"], "awaiting_callback");
    assert!(v["url"].is_string());
}

/// When stdout is not a TTY (subprocess piped output) and neither
/// `--no-browser` nor `XURL_NO_BROWSER` is set, `xr auth oauth2 --output
/// json` must auto-engage the headless path rather than attempting to spawn
/// a browser. Confirms scenario 5 of the U9 plan.
#[test]
fn test_auth_oauth2_auto_engages_headless_when_stdout_not_tty() {
    let tmp = TempDir::new().expect("tempdir");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_xr"))
        .env("HOME", tmp.path())
        .env_remove("XURL_NO_BROWSER")
        .env_remove("XURL_OUTPUT")
        .env_remove("XURL_DRY_RUN")
        .args(["auth", "oauth2", "--output", "json"])
        .output()
        .expect("spawn xr");
    assert!(
        output.status.success(),
        "auto-engage must succeed; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    let v: serde_json::Value =
        serde_json::from_str(trimmed).unwrap_or_else(|_| panic!("envelope must parse: {trimmed}"));
    assert_eq!(
        v["status"], "awaiting_callback",
        "auto-engaged envelope must match explicit --no-browser shape: {trimmed}"
    );
    assert!(v["url"].is_string(), "url present: {trimmed}");
}
