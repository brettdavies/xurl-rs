//! CLI integration tests using the library entrypoint
//! `xurl::cli::runner::run_with_overrides`.
//!
//! Parallel-safe by construction: every test creates its own `TempDir` for
//! token-store isolation and supplies its environment as `EnvOverrides`, so
//! nothing here reads or writes the process environment. The single exception
//! proves a clap `env =` binding that injection cannot reach, and
//! `tests/env_mutation_guard.rs` is the allowlist keeping that set from
//! growing by accident. The binary's exit-code contract is pinned separately
//! by `tests/binary_contract_tests.rs`.

mod common;

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
async fn run_isolated(args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code = cli::runner::run_with_overrides(
        args,
        &mut stdout,
        &mut stderr,
        &store,
        &xdk::config::EnvOverrides::default(),
    )
    .await;
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

/// Run the entrypoint against an existing store path with explicit
/// environment values, reading nothing from the process.
async fn run_at_with(
    store_path: &Path,
    overrides: &xdk::config::EnvOverrides,
    args: &[&str],
) -> (i32, String, String) {
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code =
        cli::runner::run_with_overrides(args, &mut stdout, &mut stderr, store_path, overrides)
            .await;
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

/// Overrides pointing the client at a stubbed server.
fn api_env(base_url: &str) -> xdk::config::EnvOverrides {
    xdk::config::EnvOverrides {
        api_base_url: Some(base_url.to_string()),
        ..xdk::config::EnvOverrides::default()
    }
}

/// Run the entrypoint against an existing `TempDir`-rooted store path.
///
/// Lets a single test issue multiple invocations against the same `.xurl`
/// so a setup invocation (e.g., `auth apps add`) and an assertion invocation
/// (e.g., `auth apps redirect-uri get`) observe the same state.
async fn run_at(store_path: &Path, args: &[&str]) -> (i32, String, String) {
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code = cli::runner::run_with_overrides(
        args,
        &mut stdout,
        &mut stderr,
        store_path,
        &xdk::config::EnvOverrides::default(),
    )
    .await;
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// Basic CLI sanity tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_help_flag() {
    let (code, stdout, stderr) = run_isolated(&["xr", "--help"]).await;
    assert_eq!(code, 0, "expected 0 for --help; stderr: {stderr}");
    assert!(
        stdout.contains("Usage"),
        "stdout should contain 'Usage': {stdout}"
    );
}

/// `--verbose` and `XURL_VERBOSE` say they add the legacy-vocabulary note,
/// the one line the flag prints that is not request or response traffic.
#[tokio::test]
async fn test_verbose_help_names_the_legacy_vocabulary_note() {
    let (code, stdout, stderr) = run_isolated(&["xr", "--help"]).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    let verbose = stdout
        .split("--verbose")
        .nth(1)
        .and_then(|rest| rest.split("\n\n").next())
        .unwrap_or_default();
    assert!(
        verbose.contains("legacy post vocabulary"),
        "the --verbose help names the note: {verbose:?}"
    );
    let env = stdout
        .lines()
        .find(|line| line.trim_start().starts_with("XURL_VERBOSE"))
        .unwrap_or_default();
    assert!(
        env.contains("legacy"),
        "the XURL_VERBOSE line names the note: {env:?}"
    );
}

#[tokio::test]
async fn test_version_flag() {
    let (code, stdout, stderr) = run_isolated(&["xr", "--version"]).await;
    assert_eq!(code, 0, "expected 0 for --version; stderr: {stderr}");
    assert!(
        stdout.contains("xr"),
        "stdout should contain 'xr': {stdout}"
    );
}

#[tokio::test]
async fn test_invalid_flag() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--definitely-not-a-real-flag"]).await;
    assert_ne!(code, 0, "expected non-zero exit for invalid flag");
    assert!(
        stderr.contains("Error: unexpected argument '--definitely-not-a-real-flag' found"),
        "stderr should carry the Error: line: {stderr}"
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

#[tokio::test]
async fn test_clap_error_emits_envelope_under_output_json() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--bogus-flag", "--output", "json"]).await;
    assert_eq!(code, 2, "EX_USAGE expected: {stderr}");
    assert_invalid_args_envelope(&stderr);
}

#[tokio::test]
async fn test_clap_error_emits_envelope_under_json_alias() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--bogus-flag", "--json"]).await;
    assert_eq!(code, 2, "EX_USAGE expected: {stderr}");
    assert_invalid_args_envelope(&stderr);
}

#[tokio::test]
async fn test_clap_error_emits_envelope_under_jsonl_alias() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--bogus-flag", "--jsonl"]).await;
    assert_eq!(code, 2, "EX_USAGE expected: {stderr}");
    assert_invalid_args_envelope(&stderr);
}

#[tokio::test]
async fn test_clap_error_falls_back_to_text_without_json_intent() {
    // No --output json, no --json, no XURL_OUTPUT: the text rendering.
    let (code, _stdout, stderr) = run_isolated(&["xr", "--bogus-flag"]).await;
    assert_eq!(code, 2);
    assert!(
        serde_json::from_str::<serde_json::Value>(stderr.trim()).is_err(),
        "without JSON intent, stderr should not be JSON: {stderr}"
    );
    assert!(
        stderr.contains("Error: unexpected argument '--bogus-flag' found"),
        "stderr: {stderr}"
    );
}

#[tokio::test]
async fn test_help_under_output_json_still_writes_to_stdout() {
    // DisplayHelp short-circuit: --help bypasses envelope routing.
    let (code, stdout, _stderr) = run_isolated(&["xr", "--help", "--output", "json"]).await;
    assert_eq!(code, 0);
    assert!(stdout.contains("Usage"), "help on stdout: {stdout}");
}

#[tokio::test]
async fn test_version_under_output_json_still_writes_to_stdout() {
    let (code, stdout, _stderr) = run_isolated(&["xr", "--version", "--output", "json"]).await;
    assert_eq!(code, 0);
    assert!(stdout.contains("xr"), "version on stdout: {stdout}");
}

#[tokio::test]
async fn test_envelope_consistency_clap_error_has_status_key() {
    // R6 / p2-should-consistent-envelope: clap-error JSON and runtime-error
    // JSON share the `status` discriminant; agents dispatch on it uniformly.
    let (_code, _stdout, stderr) = run_isolated(&["xr", "--bogus-flag", "--json"]).await;
    let parsed: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
    assert!(
        parsed.get("status").is_some(),
        "clap-error envelope carries status: {stderr}"
    );
}

#[tokio::test]
async fn test_raw_flag_accepted() {
    // --raw is a global boolean flag; smoke-tests parse path.
    let (code, _stdout, _stderr) = run_isolated(&["xr", "--raw", "--help"]).await;
    assert_eq!(code, 0);
}

#[tokio::test]
async fn test_json_and_output_conflict() {
    // clap should reject `--output json --json` together (validated after
    // parsing — --help would short-circuit, so use `version` subcommand).
    let (code, _stdout, stderr) =
        run_isolated(&["xr", "--output", "json", "--json", "version"]).await;
    assert_ne!(code, 0, "--json + --output must conflict: {stderr}");
}

#[tokio::test]
async fn test_json_and_jsonl_conflict() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--json", "--jsonl", "version"]).await;
    assert_ne!(code, 0, "--json + --jsonl must conflict: {stderr}");
}

#[tokio::test]
async fn test_json_alias_envelope_equivalent_to_output_json() {
    // On a clap parse failure, `--json` and `--output json` produce
    // identical envelope JSON modulo the embedded clap-rendered message
    // (which mentions the flag name). Compare structure on shared keys.
    let (_c1, _o1, e1) = run_isolated(&["xr", "--bogus-flag", "--json"]).await;
    let (_c2, _o2, e2) = run_isolated(&["xr", "--bogus-flag", "--output", "json"]).await;
    let p1: serde_json::Value = serde_json::from_str(e1.trim()).unwrap();
    let p2: serde_json::Value = serde_json::from_str(e2.trim()).unwrap();
    assert_eq!(p1["status"], p2["status"]);
    assert_eq!(p1["reason"], p2["reason"]);
    assert_eq!(p1["exit_code"], p2["exit_code"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// Subcommand help tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_post_help() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "post", "--help"]).await;
    assert_eq!(code, 0, "expected 0 for post --help; stderr: {stderr}");
}

#[tokio::test]
async fn test_search_help() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "search", "--help"]).await;
    assert_eq!(code, 0, "expected 0 for search --help; stderr: {stderr}");
}

#[tokio::test]
async fn test_auth_help() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "auth", "--help"]).await;
    assert_eq!(code, 0, "expected 0 for auth --help; stderr: {stderr}");
}

// ═══════════════════════════════════════════════════════════════════════════
// Command error handling tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_post_without_text_fails() {
    // Post command requires text argument
    let (code, _stdout, _stderr) = run_isolated(&["xr", "post"]).await;
    assert_ne!(code, 0, "expected non-zero exit for `post` with no args");
}

#[tokio::test]
async fn test_search_without_query_fails() {
    // Search command requires a query
    let (code, _stdout, _stderr) = run_isolated(&["xr", "search"]).await;
    assert_ne!(code, 0, "expected non-zero exit for `search` with no args");
}

#[tokio::test]
async fn test_delete_without_id_fails() {
    let (code, _stdout, _stderr) = run_isolated(&["xr", "delete"]).await;
    assert_ne!(code, 0, "expected non-zero exit for `delete` with no args");
}

#[tokio::test]
async fn test_reply_without_args_fails() {
    let (code, _stdout, _stderr) = run_isolated(&["xr", "reply"]).await;
    assert_ne!(code, 0, "expected non-zero exit for `reply` with no args");
}

// ═══════════════════════════════════════════════════════════════════════════
// Usage command tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_usage_help() {
    let (code, stdout, stderr) = run_isolated(&["xr", "usage", "--help"]).await;
    assert_eq!(code, 0, "expected 0 for usage --help; stderr: {stderr}");
    assert!(
        stdout.contains("usage"),
        "stdout should contain 'usage': {stdout}"
    );
    assert!(
        stdout.contains("post caps"),
        "stdout should contain 'post caps': {stdout}"
    );
}

#[tokio::test]
async fn test_usage_without_auth_fails() {
    // Isolated empty token store via tempdir — no env mutation needed.
    let (code, _stdout, _stderr) = run_isolated(&["xr", "usage"]).await;
    assert_ne!(code, 0, "expected non-zero exit for `usage` without auth");
}

// ═══════════════════════════════════════════════════════════════════════════
// Auth-required commands should fail without credentials
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_whoami_without_auth_fails() {
    // Isolated empty token store via tempdir — no env mutation needed.
    let (code, _stdout, _stderr) = run_isolated(&["xr", "whoami"]).await;
    assert_ne!(code, 0, "expected non-zero exit for `whoami` without auth");
}

// ═══════════════════════════════════════════════════════════════════════════
// App management subcommands
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_apps_list_help() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "auth", "apps", "--help"]).await;
    assert_eq!(code, 0, "expected 0 for auth apps --help; stderr: {stderr}");
}

// ═══════════════════════════════════════════════════════════════════════════
// Exit code parity tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_exit_code_success_on_help() {
    let (code, _stdout, _stderr) = run_isolated(&["xr", "--help"]).await;
    assert_eq!(code, 0, "Expected exit code 0 for --help");
}

#[tokio::test]
async fn test_exit_code_failure_on_bad_flag() {
    let (code, _stdout, _stderr) = run_isolated(&["xr", "--nonexistent"]).await;
    assert_ne!(code, 0, "Expected non-zero exit code for bad flag");
}

// ═══════════════════════════════════════════════════════════════════════════
// Verbose / trace flag tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_verbose_flag_accepted() {
    // --verbose should be accepted even if the command ultimately fails
    // due to missing auth — we just verify the flag is recognized
    let (code, _stdout, stderr) = run_isolated(&["xr", "--verbose", "--help"]).await;
    assert_eq!(code, 0, "expected 0 for --verbose --help; stderr: {stderr}");
}

#[tokio::test]
async fn test_trace_flag_accepted() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--trace", "--help"]).await;
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

#[tokio::test]
async fn test_apps_add_with_redirect_uri_persists() {
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
            "https://example.com/cb",
        ],
    )
    .await;
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
    )
    .await;
    assert_eq!(code2, 0, "get failed; stderr: {stderr2}");
    let v = parse_json(&stdout2);
    assert_eq!(v["app"], "myapp");
    assert_eq!(v["effective_redirect_uri"], "https://example.com/cb");
    assert_eq!(v["effective_source"], "app-config");
    assert_eq!(v["stored_redirect_uri"], "https://example.com/cb");
}

#[tokio::test]
async fn test_apps_add_without_redirect_uri_leaves_empty() {
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
        ],
    )
    .await;
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
    )
    .await;
    assert_eq!(code2, 0, "get failed; stderr: {stderr2}");
    let v = parse_json(&stdout2);
    assert_eq!(v["stored_redirect_uri"], serde_json::Value::Null);
    assert_eq!(v["effective_source"], "built-in-default");
}

#[tokio::test]
async fn test_apps_add_with_invalid_redirect_uri_rejected() {
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
    )
    .await;
    assert_ne!(code, 0, "expected non-zero exit for invalid redirect URI");
    assert!(
        stderr.to_lowercase().contains("redirect")
            || stderr.to_lowercase().contains("https")
            || stderr.to_lowercase().contains("loopback"),
        "stderr should mention validation failure: {stderr}"
    );
}

#[tokio::test]
async fn test_apps_update_with_redirect_uri_changes_value() {
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
    )
    .await;
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
    )
    .await;
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
    )
    .await;
    assert_eq!(code3, 0);
    let v = parse_json(&stdout3);
    assert_eq!(v["stored_redirect_uri"], "https://example.com/cb");
}

#[tokio::test]
async fn test_apps_update_with_empty_redirect_uri_clears() {
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
    )
    .await;
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
    )
    .await;
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
    )
    .await;
    assert_eq!(code3, 0);
    let v = parse_json(&stdout3);
    assert_eq!(v["stored_redirect_uri"], serde_json::Value::Null);
}

#[tokio::test]
async fn test_apps_update_no_fields_errors() {
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
    )
    .await;
    assert_eq!(code, 0);

    let (code2, _stdout, stderr2) =
        run_at(&store, &["xr", "auth", "apps", "update", "myapp"]).await;
    assert_ne!(code2, 0, "expected non-zero exit for empty update");
    assert!(
        stderr2.to_lowercase().contains("nothing to update"),
        "stderr should mention 'nothing to update': {stderr2}"
    );
}

#[tokio::test]
async fn test_redirect_uri_set_persists() {
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
    )
    .await;
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
    )
    .await;
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
    )
    .await;
    assert_eq!(code3, 0);
    let v = parse_json(&stdout3);
    assert_eq!(v["stored_redirect_uri"], "https://example.com/cb");
}

#[tokio::test]
async fn test_redirect_uri_set_invalid_rejected() {
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
    )
    .await;
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
    )
    .await;
    assert_ne!(code2, 0, "expected non-zero exit for invalid set");
    assert!(
        stderr2.to_lowercase().contains("redirect")
            || stderr2.to_lowercase().contains("https")
            || stderr2.to_lowercase().contains("loopback"),
        "stderr should mention validation failure: {stderr2}"
    );
}

#[tokio::test]
async fn test_redirect_uri_get_text_output() {
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
    )
    .await;
    assert_eq!(code, 0);

    let (code2, stdout2, stderr2) = run_at(
        &store,
        &["xr", "auth", "apps", "redirect-uri", "get", "myapp"],
    )
    .await;
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

#[tokio::test]
async fn test_redirect_uri_get_uses_default_app_when_name_omitted() {
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage that
    // would override the stored value and fail the URI assertion below.
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
    )
    .await;
    assert_eq!(code, 0);

    let (code_d, _, stderr_d) = run_at(&store, &["xr", "auth", "default", "myapp"]).await;
    assert_eq!(code_d, 0, "set-default failed; stderr: {stderr_d}");

    let (code2, stdout2, stderr2) =
        run_at(&store, &["xr", "auth", "apps", "redirect-uri", "get"]).await;
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

#[tokio::test]
async fn test_redirect_uri_get_reports_no_default_app_when_store_empty() {
    // An empty store has no apps at all, so the omit-NAME `get` has nothing
    // to resolve and says so instead of reporting a placeholder's built-in URI.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    let (code, _stdout, stderr) =
        run_at(&store, &["xr", "auth", "apps", "redirect-uri", "get"]).await;
    assert_ne!(code, 0, "an empty store has no default app to report");
    assert!(
        stderr.contains("no default app set"),
        "expected the no-default-app error; got: {stderr}"
    );
}

#[tokio::test]
async fn test_redirect_uri_get_json_output_app_config_source() {
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage from
    // the env-override test in the same binary.
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
    )
    .await;
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
    )
    .await;
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
    use xdk::store::TokenStore;
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

#[tokio::test]
async fn test_auth_status_json_excludes_all_credentials() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    populate_credentialed_store(&store);

    let (code, stdout, stderr) =
        run_at(&store, &["xr", "--output", "json", "auth", "status"]).await;
    assert_eq!(code, 0, "status failed; stderr: {stderr}");
    assert_no_credentials(&stdout, "auth status --output json");

    // Sanity: the JSON still carries the expected non-secret fields.
    let v = parse_json(&stdout);
    let arr = v["apps"]
        .as_array()
        .expect("status emits apps as a JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "myapp");
    assert_eq!(arr[0]["client_id_hint"], "CLIENT-I");
    assert_eq!(arr[0]["default"], true);
    assert_eq!(arr[0]["oauth1"], true);
    assert_eq!(arr[0]["bearer"], true);
    assert_eq!(arr[0]["oauth2_users"], serde_json::json!(["alice"]));
}

#[tokio::test]
async fn test_auth_apps_list_json_excludes_all_credentials() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    populate_credentialed_store(&store);

    let (code, stdout, stderr) =
        run_at(&store, &["xr", "--output", "json", "auth", "apps", "list"]).await;
    assert_eq!(code, 0, "apps list failed; stderr: {stderr}");
    assert_no_credentials(&stdout, "auth apps list --output json");

    let v = parse_json(&stdout);
    let arr = v["apps"]
        .as_array()
        .expect("apps list emits apps as a JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "myapp");
}

#[tokio::test]
async fn test_redirect_uri_get_json_excludes_all_credentials() {
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
    )
    .await;
    assert_eq!(code, 0, "redirect-uri get failed; stderr: {stderr}");
    assert_no_credentials(&stdout, "auth apps redirect-uri get --output json");
}

/// The text renderers for `auth status` and `auth apps list` build their own
/// lines rather than serializing `App`, so the banned-string sweep has to run
/// against them separately from the JSON path.
#[rstest::rstest]
#[case::status(&["xr", "auth", "status"])]
#[case::apps_list(&["xr", "auth", "apps", "list"])]
#[tokio::test]
async fn test_auth_text_renderers_exclude_all_credentials(#[case] args: &[&str]) {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    populate_credentialed_store(&store);

    // A sweep over output from a store with nothing in it would pass
    // vacuously; the on-disk file must carry the values being hunted for.
    let raw = std::fs::read_to_string(&store).expect("store file");
    for needle in ["SECRET-VALUE-AAA", "TOKEN-SECRET-EEE", "BEARER-VALUE-FFF"] {
        assert!(
            raw.contains(needle),
            "fixture must hold {needle:?} for the sweep to have something to catch"
        );
    }

    let (code, stdout, stderr) = run_at(&store, args).await;
    assert_eq!(code, 0, "args {args:?} failed; stderr: {stderr}");
    assert_no_credentials(&stdout, &format!("{args:?} text mode"));
    assert!(
        stdout.contains("myapp"),
        "args {args:?} must have rendered the app; got: {stdout}"
    );
    assert!(
        stdout.contains("client_id: CLIENT-I..."),
        "args {args:?} must render the client-id hint, not the value; got: {stdout}"
    );
}

#[tokio::test]
async fn test_auth_status_text_includes_redirect_uri_line() {
    // R24: status text output gains a `redirect_uri:` line per app.
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage from
    // another env-mutating test in the same binary.
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
    )
    .await;
    assert_eq!(code, 0);
    let (code_d, _, _) = run_at(&store, &["xr", "auth", "default", "myapp"]).await;
    assert_eq!(code_d, 0);

    let (code2, stdout2, stderr2) = run_at(&store, &["xr", "auth", "status"]).await;
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

#[tokio::test]
async fn test_auth_status_text_default_built_in_when_no_stored_uri() {
    // R24: status text falls through to built-in default when no env, no stored.
    // `#[serial]` + explicit env removal guards against `REDIRECT_URI` leaking
    // from another env-mutating test in the same binary.
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
    )
    .await;
    assert_eq!(code, 0);

    let (code2, stdout2, _) = run_at(&store, &["xr", "auth", "status"]).await;
    assert_eq!(code2, 0);
    assert!(
        stdout2.contains("redirect_uri: http://localhost:8080/callback [built-in default]"),
        "expected built-in default redirect URI line: {stdout2}"
    );
}

#[tokio::test]
async fn test_auth_status_json_emits_app_config_source_and_no_stored_field() {
    // R21: source serializes as kebab-case; `redirect_uri_stored` is absent
    // when the env does not override the stored value.
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage that
    // would flip the asserted source to `env-var`.
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
            "abcdefgh1234",
            "--client-secret",
            "xyz",
            "--redirect-uri",
            "http://localhost:7777/cb",
        ],
    )
    .await;
    assert_eq!(code, 0);
    let (code_d, _, _) = run_at(&store, &["xr", "auth", "default", "myapp"]).await;
    assert_eq!(code_d, 0);

    let (code2, stdout2, _) = run_at(&store, &["xr", "--output", "json", "auth", "status"]).await;
    assert_eq!(code2, 0);
    let v = parse_json(&stdout2);
    let arr = v["apps"]
        .as_array()
        .expect("status emits apps as a JSON array");
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

#[tokio::test]
async fn test_auth_status_json_env_override_surfaces_stored_field() {
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
    )
    .await;
    assert_eq!(code, 0);
    let (code_d, _, _) = run_at(&store, &["xr", "auth", "default", "myapp"]).await;
    assert_eq!(code_d, 0);

    let overrides = xdk::config::EnvOverrides {
        redirect_uri: Some("https://override.example.com/cb".into()),
        ..xdk::config::EnvOverrides::default()
    };
    let (code2, stdout2, _) = run_at_with(
        &store,
        &overrides,
        &["xr", "--output", "json", "auth", "status"],
    )
    .await;
    assert_eq!(code2, 0);
    let v = parse_json(&stdout2);
    let arr = v["apps"]
        .as_array()
        .expect("status emits apps as a JSON array");
    let entry = arr
        .iter()
        .find(|e| e["name"] == "myapp")
        .expect("myapp entry");
    assert_eq!(entry["redirect_uri"], "https://override.example.com/cb");
    assert_eq!(entry["redirect_uri_source"], "env-var");
    assert_eq!(entry["redirect_uri_stored"], "http://localhost:7777/cb");
}

#[tokio::test]
async fn test_auth_status_json_default_flag_per_app() {
    // R21: with two apps, only the default app's entry has `default: true`.
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage from
    // a parallel env-mutating test (env source would not affect this
    // assertion, but the discipline keeps the snapshot stable).
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

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
    )
    .await;
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
    )
    .await;
    assert_eq!(c2, 0);
    let (cd, _, _) = run_at(&store, &["xr", "auth", "default", "beta"]).await;
    assert_eq!(cd, 0);

    let (code, stdout, _) = run_at(&store, &["xr", "--output", "json", "auth", "status"]).await;
    assert_eq!(code, 0);
    let v = parse_json(&stdout);
    let arr = v["apps"]
        .as_array()
        .expect("status emits apps as a JSON array");
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

#[tokio::test]
async fn test_auth_apps_list_json_shape_per_app() {
    // R21 (list): per-app object carries `name`, `client_id_hint`,
    // `redirect_uri`, `redirect_uri_source`, `oauth2_users`, `oauth1`,
    // `bearer`, `default`.
    // `#[serial]` + env removal guards against `REDIRECT_URI` leakage that
    // would flip the asserted `redirect_uri_source` to `env-var`.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

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
    )
    .await;
    assert_eq!(c1, 0);
    let (cd, _, _) = run_at(&store, &["xr", "auth", "default", "alpha"]).await;
    assert_eq!(cd, 0);

    let (code, stdout, _) =
        run_at(&store, &["xr", "--output", "json", "auth", "apps", "list"]).await;
    assert_eq!(code, 0);
    let v = parse_json(&stdout);
    let arr = v["apps"]
        .as_array()
        .expect("apps list emits apps as a JSON array");
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

#[tokio::test]
async fn test_auth_status_text_snapshot_two_apps_default_case() {
    // Text-output regression: locks in the new `redirect_uri:` line per app
    // for the no-env, no-stored-URI case across two user-added apps. A fresh
    // `TokenStore` also seeds a `"default"` placeholder app, so the iteration
    // emits a line for each of the three apps; the snapshot anchors them all.
    // `#[serial]` + env removal guards against `REDIRECT_URI` leaking across
    // tests in the same binary.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

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
    )
    .await;
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
    )
    .await;
    assert_eq!(c2, 0);
    let (cd, _, _) = run_at(&store, &["xr", "auth", "default", "alpha"]).await;
    assert_eq!(cd, 0);

    let (code, stdout, _) = run_at(&store, &["xr", "auth", "status"]).await;
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
    server: MockServer,
    uri: String,
}

impl CliMockServer {
    async fn new() -> Self {
        let server = MockServer::start().await;
        let uri = server.uri();
        Self { server, uri }
    }

    async fn mount(&self, mock: Mock) {
        mock.mount(&self.server).await;
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
    let mut ts = populate_app_store(store_path);
    ts.save_bearer_token_for_app("myapp", "BEARER-TOKEN-VALUE")
        .expect("save_bearer");
}

/// Seeds a tempdir-rooted store with `myapp` as the only app and the
/// default, carrying client credentials and no tokens.
fn populate_app_store(store_path: &Path) -> xdk::store::TokenStore {
    use xdk::store::TokenStore;
    let mut ts = TokenStore::new_with_path(store_path.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.set_default_app("myapp").expect("set_default_app");
    let _ = ts.remove_app("default");
    ts
}

/// Seeds a tempdir-rooted store with a single app carrying an OAuth1 token.
///
/// Counterpart to [`populate_bearer_store`]: user-context endpoints like
/// `POST /2/users/{id}/likes` and `DELETE /2/tweets/{id}` accept OAuth1 (and
/// OAuth2) but NOT Bearer per the spec-derived matrix, so tests that drive
/// those endpoints under `--auth oauth1` need this fixture.
fn populate_oauth1_store(store_path: &Path) {
    use xdk::store::TokenStore;
    let mut ts = TokenStore::new_with_path(store_path.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_oauth1_tokens_for_app(
        "myapp",
        "OA1-ACCESS-TOKEN",
        "TOKEN-SECRET",
        "OA1-CONSUMER-KEY",
        "CONSUMER-SECRET",
    )
    .expect("save_oauth1");
    ts.set_default_app("myapp").expect("set_default_app");
    let _ = ts.remove_app("default");
}

/// `xr auth oauth1` hands its four credential flags down two layers as
/// adjacent positional `String` arguments, and `save_oauth1_tokens_for_app`
/// takes them in a different order than the CLI declares them. Each fixture
/// value names the slot it belongs in, so a transposition the compiler cannot
/// see surfaces as a value sitting in the wrong field.
#[tokio::test]
async fn test_auth_oauth1_cli_writes_each_flag_to_its_own_store_slot() {
    use xdk::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    drop(populate_app_store(&store));

    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "oauth1",
            "--consumer-key",
            "BELONGS-IN-CONSUMER-KEY",
            "--consumer-secret",
            "BELONGS-IN-CONSUMER-SECRET",
            "--access-token",
            "BELONGS-IN-ACCESS-TOKEN",
            "--token-secret",
            "BELONGS-IN-TOKEN-SECRET",
        ],
    )
    .await;
    assert_eq!(code, 0, "auth oauth1 failed; stderr: {stderr}");
    assert!(
        stdout.contains("OAuth1 credentials saved successfully!"),
        "expected the success message; got: {stdout}"
    );

    let ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    let app = ts.get_app("myapp").expect("myapp survives the save");
    let token = app
        .oauth1_token
        .as_ref()
        .expect("the OAuth1 slot on myapp is populated");
    let oauth1 = token
        .oauth1
        .as_ref()
        .expect("the OAuth1 payload is populated");
    assert_eq!(oauth1.consumer_key, "BELONGS-IN-CONSUMER-KEY");
    assert_eq!(oauth1.consumer_secret, "BELONGS-IN-CONSUMER-SECRET");
    assert_eq!(oauth1.access_token, "BELONGS-IN-ACCESS-TOKEN");
    assert_eq!(oauth1.token_secret, "BELONGS-IN-TOKEN-SECRET");
}

#[tokio::test]
async fn test_like_with_username_flag_calls_lookup_by_username() {
    // `-u alice` routes through `/2/users/by/username/alice`; the resolved id
    // (67890) drives the like POST. `expect(1)` on each mock fails the test
    // (on server drop) if either endpoint is hit zero times or > 1.
    //
    // Uses OAuth1 because `POST /2/users/{id}/likes` accepts OAuth1 + OAuth2
    // per the spec matrix but rejects Bearer (v2.0.0 enforcement).
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_oauth1_store(&store);

    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/by/username/alice"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "67890", "username": "alice", "name": "Alice"}
            })))
            .expect(1),
    )
    .await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/users/67890/likes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"liked": true}
            })))
            .expect(1),
    )
    .await;

    let (code, stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &["xr", "like", "12345", "-u", "alice", "--auth", "oauth1"],
    )
    .await;

    assert_eq!(code, 0, "like failed; stderr: {stderr}; stdout: {stdout}");
}

#[tokio::test]
async fn test_like_without_username_flag_calls_me() {
    // No `-u` → empty `opts.username` → resolver hits `/2/users/me`.
    // OAuth1 because both `/2/users/me` and the like POST accept it but
    // reject Bearer under v2.0.0 enforcement.
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_oauth1_store(&store);

    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "111", "username": "self", "name": "Self"}
            })))
            .expect(1),
    )
    .await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/users/111/likes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"liked": true}
            })))
            .expect(1),
    )
    .await;

    let (code, stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &["xr", "like", "12345", "--auth", "oauth1"],
    )
    .await;

    assert_eq!(code, 0, "like failed; stderr: {stderr}; stdout: {stdout}");
}

#[tokio::test]
async fn test_like_with_username_flag_lookup_404() {
    // `-u alice` + 404 from lookup → resolver bubbles the transport error up
    // and the like POST is never issued. Mock the lookup only; if anything
    // else hits the server, wiremock returns 404 by default and the test
    // still surfaces a non-zero exit.
    //
    // Uses OAuth1 because the like-flow endpoints reject Bearer under v2.0.0
    // enforcement, and reaching the 404 path requires validation to yield.
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_oauth1_store(&store);

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
    )
    .await;

    let (code, _stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &["xr", "like", "12345", "-u", "alice", "--auth", "oauth1"],
    )
    .await;

    assert_ne!(
        code, 0,
        "expected non-zero exit when lookup 404s; stderr: {stderr}"
    );
}

#[tokio::test]
async fn test_like_with_empty_username_falls_back_to_me() {
    // `-u ""` collapses through `CommonFlags::to_call_options()` to an empty
    // `opts.username`, which falls into the `/me` branch. Documents the
    // current contract: a future change that treats `Some("")` differently
    // from `None` is intentional, not accidental.
    //
    // OAuth1 because `/2/users/me` + `/2/users/{id}/likes` accept it but
    // reject Bearer under v2.0.0 enforcement.
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_oauth1_store(&store);

    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "222", "username": "self", "name": "Self"}
            })))
            .expect(1),
    )
    .await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/users/222/likes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"liked": true}
            })))
            .expect(1),
    )
    .await;

    let (code, stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &["xr", "like", "12345", "-u", "", "--auth", "oauth1"],
    )
    .await;

    assert_eq!(code, 0, "like failed; stderr: {stderr}; stdout: {stdout}");
}

#[tokio::test]
async fn test_like_with_at_prefix_username_strips_at() {
    // `lookup_user`'s internal `resolve_username` strips a leading `@` before
    // building the path. The mock matches the bare handle; if the strip were
    // skipped, wiremock would 404 the `@alice` request and exit would be
    // non-zero.
    //
    // OAuth1 because `/2/users/by/username/{username}` + the like POST accept
    // it but reject Bearer under v2.0.0 enforcement.
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_oauth1_store(&store);

    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/users/by/username/alice"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "333", "username": "alice", "name": "Alice"}
            })))
            .expect(1),
    )
    .await;
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/users/333/likes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"liked": true}
            })))
            .expect(1),
    )
    .await;

    let (code, stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &["xr", "like", "12345", "-u", "@alice", "--auth", "oauth1"],
    )
    .await;

    assert_eq!(code, 0, "like failed; stderr: {stderr}; stdout: {stdout}");
}

#[tokio::test]
async fn test_oauth2_positional_invalid_extra_args() {
    // Two positionals on `auth oauth2` must fail with a clap usage error
    // (exit code 2 via the runner).
    let (code, _stdout, stderr) = run_isolated(&["xr", "auth", "oauth2", "alice", "bob"]).await;
    assert_eq!(
        code, 2,
        "expected clap usage exit code 2 for extra positional; stderr: {stderr}"
    );
    assert!(
        stderr.contains("Error: unexpected argument 'bob' found"),
        "stderr should name the extra positional: {stderr}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Status and list rendering: the unnamed OAuth2 slot
// ═══════════════════════════════════════════════════════════════════════════

/// Seeds a store with one app `myapp` carrying an unnamed OAuth2 token and
/// no named OAuth2 entries.
fn seed_app_with_unnamed_oauth2(store_path: &Path) {
    use xdk::store::TokenStore;
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

#[tokio::test]
async fn test_status_text_shows_unnamed_oauth2() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_app_with_unnamed_oauth2(&store);

    let (code, stdout, stderr) = run_at(&store, &["xr", "auth", "status"]).await;
    assert_eq!(code, 0, "auth status failed; stderr: {stderr}");
    assert!(
        stdout.contains("oauth2: (unknown user)"),
        "status text should render `oauth2: (unknown user)` for the unnamed slot; got:\n{stdout}"
    );
}

#[tokio::test]
async fn test_status_json_emits_oauth2_unnamed_true() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    seed_app_with_unnamed_oauth2(&store);

    let (code, stdout, stderr) =
        run_at(&store, &["xr", "--output", "json", "auth", "status"]).await;
    assert_eq!(
        code, 0,
        "auth status --output json failed; stderr: {stderr}"
    );
    let v = parse_json(&stdout);
    let arr = v["apps"]
        .as_array()
        .expect("status emits apps as a JSON array");
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

#[tokio::test]
async fn test_status_json_omits_oauth2_unnamed_when_false() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    // Reuse the bearer-only fixture: no named OAuth2, no unnamed slot.
    populate_bearer_store(&store);

    let (code, stdout, stderr) =
        run_at(&store, &["xr", "--output", "json", "auth", "status"]).await;
    assert_eq!(
        code, 0,
        "auth status --output json failed; stderr: {stderr}"
    );
    let v = parse_json(&stdout);
    let arr = v["apps"]
        .as_array()
        .expect("status emits apps as a JSON array");
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

/// Helper: run `xr` with a caller-supplied home directory.
///
/// The value is injected through `EnvOverrides`, so the process `HOME` is
/// neither read nor written and these tests run alongside everything else.
async fn run_with_home(args: &[&str], home: Option<&str>) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    run_at_with(
        &store,
        &xdk::config::EnvOverrides {
            home: home.map(str::to_string),
            ..xdk::config::EnvOverrides::default()
        },
        args,
    )
    .await
}

#[tokio::test]
async fn skill_install_help_advertises_host_all_dry_run() {
    let (code, stdout, stderr) = run_isolated(&["xr", "skill", "install", "--help"]).await;
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

#[tokio::test]
async fn skill_install_dry_run_emits_envelope_without_spawning_git() {
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
    )
    .await;
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

#[tokio::test]
async fn skill_install_existing_non_empty_destination_errors() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    // Pre-populate the destination so the conflict check fires.
    let dest = tmp.path().join(".claude").join("skills").join("xurl-rs");
    std::fs::create_dir_all(&dest).expect("mkdir -p dest");
    std::fs::write(dest.join("placeholder"), b"x").expect("write placeholder");

    let (code, stdout, stderr) = run_with_home(
        &["xr", "skill", "install", "claude_code", "--output", "json"],
        Some(&home),
    )
    .await;
    assert_eq!(code, 1, "expected 1 for dest-not-empty; stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON envelope");
    assert_eq!(v["status"], "error");
    assert_eq!(v["reason"], "destination-not-empty");
    assert_eq!(v["exit_code"], 1);
    assert_eq!(v["destination_status"], "non-empty-dir");
}

#[tokio::test]
async fn skill_install_all_dry_run_lists_every_host() {
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
    )
    .await;
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
    for expected in xurl::cli::skill_install::KNOWN_HOSTS {
        assert!(
            host_names.contains(expected),
            "host {expected} missing from --all envelope; got {host_names:?}"
        );
    }
}

#[tokio::test]
async fn skill_install_home_unset_emits_home_not_set_reason() {
    let (code, stdout, stderr) = run_with_home(
        &["xr", "skill", "install", "claude_code", "--output", "json"],
        None,
    )
    .await;
    assert_eq!(code, 1, "expected 1 for HOME unset; stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON envelope");
    assert_eq!(v["status"], "error");
    assert_eq!(v["reason"], "home-not-set");
    assert_eq!(v["exit_code"], 1);
}

#[tokio::test]
async fn skill_install_no_args_lists_supported_hosts() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    let (code, stdout, stderr) = run_with_home(&["xr", "skill", "install"], Some(&home)).await;
    assert_eq!(
        code, 2,
        "expected 2 (usage error) for missing host; stderr: {stderr}"
    );
    for expected in xurl::cli::skill_install::KNOWN_HOSTS {
        assert!(
            stdout.contains(expected),
            "host {expected} missing from text listing: {stdout}"
        );
    }
}

#[tokio::test]
async fn skill_install_no_args_json_lists_supported_hosts_in_envelope() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    let (code, stdout, stderr) =
        run_with_home(&["xr", "skill", "install", "--output", "json"], Some(&home)).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON envelope");
    assert_eq!(v["status"], "error");
    assert_eq!(v["reason"], "missing-host");
    assert_eq!(v["exit_code"], 2);
    let hosts = v["known_hosts"]
        .as_array()
        .expect("known_hosts is an array");
    let names: Vec<&str> = hosts.iter().map(|h| h.as_str().expect("str")).collect();
    for expected in xurl::cli::skill_install::KNOWN_HOSTS {
        assert!(names.contains(expected), "missing {expected}: {names:?}");
    }
}

#[tokio::test]
async fn skill_install_dry_run_text_output_is_single_line_command() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    let (code, stdout, stderr) = run_with_home(
        &["xr", "skill", "install", "claude_code", "--dry-run"],
        Some(&home),
    )
    .await;
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

#[tokio::test]
async fn skill_install_dest_is_regular_file_errors() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_string_lossy().into_owned();
    // Plant a regular file at the destination path.
    let dest_parent = tmp.path().join(".claude").join("skills");
    std::fs::create_dir_all(&dest_parent).expect("mkdir -p parent");
    std::fs::write(dest_parent.join("xurl-rs"), b"i'm a file").expect("write file");

    let (code, stdout, stderr) = run_with_home(
        &["xr", "skill", "install", "claude_code", "--output", "json"],
        Some(&home),
    )
    .await;
    assert_eq!(code, 1, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON envelope");
    assert_eq!(v["status"], "error");
    assert_eq!(v["reason"], "destination-is-file");
    assert_eq!(v["destination_status"], "file");
}

/// The skill manifest the build generates the host table from.
fn skill_manifest() -> serde_json::Value {
    let path = common::workspace_root().join("crates/xurl-cli/src/cli/skill_install/skill.json");
    let text = std::fs::read_to_string(&path).expect("skill.json must be readable");
    serde_json::from_str(&text).expect("skill.json must parse")
}

/// The `~`-prefixed destination template in `host`'s install command.
fn install_template(manifest: &serde_json::Value, host: &str) -> String {
    let cmd = manifest["install"][host]
        .as_str()
        .expect("install command string");
    cmd.split_whitespace()
        .last()
        .expect("install command ends in its destination")
        .to_string()
}

/// `host -> install_dir` from `skill install --all --dry-run`, with `vars`
/// added to the hermetic environment.
fn dry_run_install_dirs(vars: &[(&str, &Path)]) -> std::collections::BTreeMap<String, String> {
    let mut cmd = common::xr();
    for (key, value) in vars {
        cmd.env(key, value);
    }
    let out = cmd
        .args(["skill", "install", "--all", "--dry-run", "--output", "json"])
        .output()
        .expect("run xr");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "expected a JSON envelope ({e}); stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    v["installations"]
        .as_array()
        .expect("installations array")
        .iter()
        .map(|e| {
            (
                e["host"].as_str().expect("host").to_string(),
                e["install_dir"].as_str().expect("install_dir").to_string(),
            )
        })
        .collect()
}

#[test]
fn skill_home_env_stands_in_for_home_in_every_destination() {
    let manifest = skill_manifest();
    let skill_home = TempDir::new().expect("tempdir");
    let dirs = dry_run_install_dirs(&[("XURL_SKILL_HOME", skill_home.path())]);
    for host in manifest["install"].as_object().expect("install map").keys() {
        let template = install_template(&manifest, host);
        let rest = template
            .strip_prefix("~/")
            .expect("template starts with ~/");
        let expected = skill_home.path().join(rest);
        assert_eq!(
            dirs.get(host).map(String::as_str),
            Some(expected.to_string_lossy().as_ref()),
            "host {host}"
        );
    }
}

#[test]
fn each_host_config_dir_env_wins_over_skill_home_for_its_own_host() {
    let manifest = skill_manifest();
    let entries = manifest["config_dir_env"]
        .as_object()
        .expect("skill.json carries a config_dir_env entry for every host");
    let skill_home = TempDir::new().expect("tempdir");
    let config_root = TempDir::new().expect("tempdir");
    let host_dirs: Vec<(String, std::path::PathBuf)> = entries
        .iter()
        .filter_map(|(host, entry)| {
            entry["var"]
                .as_str()
                .map(|var| (var.to_string(), config_root.path().join(host)))
        })
        .collect();
    assert!(
        !host_dirs.is_empty(),
        "at least one host documents a config-dir variable"
    );
    let mut vars: Vec<(&str, &Path)> = vec![("XURL_SKILL_HOME", skill_home.path())];
    vars.extend(
        host_dirs
            .iter()
            .map(|(var, dir)| (var.as_str(), dir.as_path())),
    );
    let dirs = dry_run_install_dirs(&vars);

    for (host, entry) in entries {
        let template = install_template(&manifest, host);
        let expected = match entry["var"].as_str() {
            Some(_) => {
                let replaces = entry["replaces"]
                    .as_str()
                    .expect("replaces for a host with a var");
                let rest = template
                    .strip_prefix(replaces)
                    .and_then(|r| r.strip_prefix('/'))
                    .expect("replaces is a directory prefix of the template");
                config_root.path().join(host).join(rest)
            }
            None => skill_home
                .path()
                .join(template.strip_prefix("~/").expect("~/ template")),
        };
        assert_eq!(
            dirs.get(host).map(String::as_str),
            Some(expected.to_string_lossy().as_ref()),
            "host {host}"
        );
    }
}

/// `host`'s `install_dir` from `skill install <host> --dry-run`, with `vars`
/// added to the hermetic environment.
fn dry_run_install_dir(host: &str, vars: &[(&str, &Path)]) -> String {
    let mut cmd = common::xr();
    for (key, value) in vars {
        cmd.env(key, value);
    }
    let out = cmd
        .args(["skill", "install", host, "--dry-run", "--output", "json"])
        .output()
        .expect("run xr");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("JSON envelope");
    v["install_dir"].as_str().expect("install_dir").to_string()
}

#[test]
fn a_base_dir_env_applies_only_while_skill_home_is_unset() {
    let manifest = skill_manifest();
    let (host, entry) = manifest["base_dir_env"]
        .as_object()
        .expect("base_dir_env map")
        .iter()
        .next()
        .expect("a host that follows a base-directory variable");
    let var = entry["var"].as_str().expect("var");
    let replaces = entry["replaces"].as_str().expect("replaces");
    let template = install_template(&manifest, host);
    let base = TempDir::new().expect("tempdir");
    let skill_home = TempDir::new().expect("tempdir");

    // A dry run spawns nothing and writes nothing, so handing the child this
    // standard variable reaches no other tool.
    let under_base = dry_run_install_dir(host, &[(var, base.path())]);
    let rest = template
        .strip_prefix(replaces)
        .and_then(|r| r.strip_prefix('/'))
        .expect("replaces is a directory prefix of the template");
    assert_eq!(under_base, base.path().join(rest).to_string_lossy());

    let under_skill_home = dry_run_install_dir(
        host,
        &[(var, base.path()), ("XURL_SKILL_HOME", skill_home.path())],
    );
    let home_rest = template.strip_prefix("~/").expect("~/ template");
    assert_eq!(
        under_skill_home,
        skill_home.path().join(home_rest).to_string_lossy(),
        "XURL_SKILL_HOME replaces the whole home, {var} included"
    );
}

#[test]
fn an_empty_host_config_dir_env_falls_through_to_skill_home() {
    let manifest = skill_manifest();
    let (host, var) = manifest["config_dir_env"]
        .as_object()
        .expect("config_dir_env map")
        .iter()
        .find_map(|(host, entry)| {
            entry["var"]
                .as_str()
                .map(|var| (host.clone(), var.to_string()))
        })
        .expect("a host with a config-dir variable");
    let skill_home = TempDir::new().expect("tempdir");
    let mut cmd = common::xr();
    cmd.env(&var, "").env("XURL_SKILL_HOME", skill_home.path());
    let out = cmd
        .args(["skill", "install", &host, "--dry-run", "--output", "json"])
        .output()
        .expect("run xr");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("JSON envelope");
    let template = install_template(&manifest, &host);
    let expected = skill_home
        .path()
        .join(template.strip_prefix("~/").expect("~/ template"));
    assert_eq!(
        v["install_dir"].as_str(),
        Some(expected.to_string_lossy().as_ref()),
        "an empty {var} is unset, so {host} resolves under XURL_SKILL_HOME"
    );
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

#[tokio::test]
async fn test_post_help_includes_paired_text_and_json_examples() {
    let (code, stdout, stderr) = run_isolated(&["xr", "post", "--help"]).await;
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

#[tokio::test]
async fn test_auth_oauth2_help_shows_no_browser_example() {
    let (code, stdout, stderr) = run_isolated(&["xr", "auth", "oauth2", "--help"]).await;
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

#[tokio::test]
async fn test_delete_no_interactive_without_force_emits_confirmation_required_envelope() {
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    // Mount the delete endpoint with expect(0) — verifies no HTTP fires.
    ts.mount(
        Mock::given(method("DELETE"))
            .and(path("/2/tweets/12345"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0),
    )
    .await;

    let (code, _stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &[
            "xr",
            "--no-interactive",
            "--output",
            "json",
            "delete",
            "12345",
            "--auth",
            "app",
        ],
    )
    .await;

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

#[tokio::test]
async fn test_delete_force_no_interactive_calls_api_and_succeeds() {
    // `DELETE /2/tweets/{id}` accepts OAuth1 + OAuth2 but rejects Bearer
    // per the spec matrix (v2.0.0 enforcement). Seed an OAuth1 store and
    // pass `--auth oauth1` to clear validation.
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_oauth1_store(&store);

    ts.mount(
        Mock::given(method("DELETE"))
            .and(path("/2/tweets/12345"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"deleted": true}
            })))
            .expect(1),
    )
    .await;

    let (code, stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &[
            "xr", "--output", "json", "delete", "12345", "--force", "--auth", "oauth1",
        ],
    )
    .await;

    assert_eq!(
        code, 0,
        "delete --force --no-interactive must exit 0; stderr: {stderr}; stdout: {stdout}"
    );
    let v = parse_json(&stdout);
    assert_eq!(v["data"]["deleted"], serde_json::Value::Bool(true));
}

#[tokio::test]
async fn test_post_dry_run_emits_envelope_and_skips_api() {
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    // No HTTP must fire.
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0),
    )
    .await;

    let (code, stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &[
            "xr",
            "--dry-run",
            "--output",
            "json",
            "post",
            "Hello",
            "--auth",
            "app",
        ],
    )
    .await;

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

#[tokio::test]
async fn test_post_empty_body_dry_run_reports_empty_body_reason() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--dry-run",
            "--output",
            "json",
            "post",
            "",
            "--auth",
            "app",
        ],
    )
    .await;

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

#[tokio::test]
async fn test_post_body_too_long_dry_run_reports_body_too_long_reason() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    // 281 chars: one past the 280-char limit.
    let too_long: String = std::iter::repeat_n('x', 281).collect();
    let (code, stdout, _stderr) = run_at(
        &store,
        &[
            "xr",
            "--dry-run",
            "--output",
            "json",
            "post",
            &too_long,
            "--auth",
            "app",
        ],
    )
    .await;

    assert_eq!(code, 0);
    let v = parse_json(&stdout);
    assert_eq!(v["status"], "dry_run");
    assert_eq!(v["would_succeed"], serde_json::Value::Bool(false));
    assert_eq!(v["reason"], "body-too-long");
}

/// X answers `POST /2/tweets` in its legacy vocabulary; `--output json`
/// reports the key under the name the spec uses, carrying X's value as sent.
#[tokio::test]
async fn test_post_json_reports_edit_history_post_ids_for_the_legacy_key() {
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_oauth1_store(&store);
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {"id": "2101712260468977783", "text": "hi", "edit_history_tweet_ids": [""]}
            })))
            .expect(1),
    )
    .await;

    let (code, stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &["xr", "--output", "json", "post", "hi", "--auth", "oauth1"],
    )
    .await;

    assert_eq!(code, 0, "post failed; stderr: {stderr}; stdout: {stdout}");
    let data = &parse_json(&stdout)["data"];
    assert_eq!(
        data["edit_history_post_ids"],
        serde_json::json!([""]),
        "stdout: {stdout}"
    );
    assert!(
        data.get("edit_history_tweet_ids").is_none(),
        "stdout: {stdout}"
    );
}

/// A raw request prints the body exactly as X sent it, legacy keys included.
#[tokio::test]
async fn test_raw_request_keeps_the_legacy_spelling_x_sent() {
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);
    let body = serde_json::json!({
        "data": {
            "id": "123",
            "text": "hi",
            "edit_history_tweet_ids": ["123"],
            "public_metrics": {"retweet_count": 1}
        }
    });
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body.clone()))
            .expect(1),
    )
    .await;

    let (code, stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &["xr", "--output", "json", "/2/tweets/123", "--auth", "app"],
    )
    .await;

    assert_eq!(
        code, 0,
        "raw read failed; stderr: {stderr}; stdout: {stdout}"
    );
    assert_eq!(parse_json(&stdout), body, "stdout: {stdout}");
}

#[tokio::test]
async fn test_search_global_limit_50_respected() {
    let ts = CliMockServer::new().await;
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
    )
    .await;

    let (code, _stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &["xr", "--limit", "50", "search", "x", "--auth", "app"],
    )
    .await;

    assert_eq!(code, 0, "search --limit 50 failed; stderr: {stderr}");
}

#[tokio::test]
async fn test_search_global_limit_500_clamped_to_100() {
    let ts = CliMockServer::new().await;
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
    )
    .await;

    let (code, _stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &["xr", "--limit", "500", "search", "x", "--auth", "app"],
    )
    .await;

    assert_eq!(code, 0, "search --limit 500 must clamp; stderr: {stderr}");
}

#[tokio::test]
async fn test_search_per_cmd_max_results_overrides_global_limit() {
    let ts = CliMockServer::new().await;
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
    )
    .await;

    let (code, _stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &[
            "xr", "--limit", "80", "search", "x", "-n", "20", "--auth", "app",
        ],
    )
    .await;

    assert_eq!(
        code, 0,
        "per-cmd -n must override --limit; stderr: {stderr}"
    );
}

#[tokio::test]
async fn test_examples_subcommand_runs() {
    let (code, stdout, stderr) = run_isolated(&["xr", "examples"]).await;
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

#[tokio::test]
async fn test_search_help_demonstrates_env_var_precedence() {
    let (code, stdout, stderr) = run_isolated(&["xr", "search", "--help"]).await;
    assert_eq!(code, 0, "expected 0 for search --help; stderr: {stderr}");
    assert!(
        stdout.contains("XURL_OUTPUT=json xr search"),
        "search --help must demo XURL_OUTPUT precedence; got:\n{stdout}"
    );
}

#[tokio::test]
async fn test_auth_clear_force_no_interactive_dry_run_envelope() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--no-interactive",
            "--dry-run",
            "--output",
            "json",
            "auth",
            "clear",
            "--all",
            "--force",
        ],
    )
    .await;

    assert_eq!(
        code, 0,
        "auth clear --force --no-interactive --dry-run must exit 0; stderr: {stderr}"
    );
    let v = parse_json(&stdout);
    assert_eq!(v["status"], "dry_run");
    assert_eq!(v["command"], "auth-clear");
    assert_eq!(v["all"], serde_json::Value::Bool(true));
}

/// Each `auth clear` selector must reach its own envelope field. The dispatch
/// forwards five adjacent flags positionally, four of which are `bool`.
#[rstest::rstest]
#[case::oauth1(&["--oauth1"], serde_json::json!({"all": false, "oauth1": true, "oauth2_username": null, "bearer": false}))]
#[case::bearer(&["--bearer"], serde_json::json!({"all": false, "oauth1": false, "oauth2_username": null, "bearer": true}))]
#[case::oauth2_username(&["--oauth2-username", "alice"], serde_json::json!({"all": false, "oauth1": false, "oauth2_username": "alice", "bearer": false}))]
#[tokio::test]
async fn test_auth_clear_dry_run_names_each_selector(
    #[case] flags: &[&str],
    #[case] expected: serde_json::Value,
) {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_credentialed_store(&store);

    let mut argv = vec![
        "xr",
        "--no-interactive",
        "--dry-run",
        "--output",
        "json",
        "auth",
        "clear",
        "--force",
    ];
    argv.extend_from_slice(flags);
    let (code, stdout, stderr) = run_at(&store, &argv).await;

    assert_eq!(code, 0, "flags {flags:?} failed; stderr: {stderr}");
    let v = parse_json(&stdout);
    assert_eq!(v["status"], "dry_run", "flags {flags:?}; got: {stdout}");
    assert_eq!(v["command"], "auth-clear", "flags {flags:?}; got: {stdout}");
    for key in ["all", "oauth1", "oauth2_username", "bearer"] {
        assert_eq!(
            v[key], expected[key],
            "flags {flags:?} put the wrong value in {key:?}; got: {stdout}"
        );
    }
}

/// A selector clears its own credential and nothing else. The three slots hold
/// distinct values so a selector wired to the wrong `clear_*` call shows up as
/// a surviving credential that should be gone (or the reverse).
#[rstest::rstest]
#[case::oauth1(&["--oauth1"], false, true, true)]
#[case::bearer(&["--bearer"], true, true, false)]
#[case::oauth2_username(&["--oauth2-username", "alice"], true, false, true)]
#[tokio::test]
async fn test_auth_clear_selector_removes_only_its_own_credential(
    #[case] flags: &[&str],
    #[case] oauth1_kept: bool,
    #[case] oauth2_alice_kept: bool,
    #[case] bearer_kept: bool,
) {
    use xdk::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_credentialed_store(&store);

    let mut argv = vec!["xr", "--no-interactive", "auth", "clear", "--force"];
    argv.extend_from_slice(flags);
    let (code, stdout, stderr) = run_at(&store, &argv).await;
    assert_eq!(code, 0, "flags {flags:?} failed; stderr: {stderr}");
    assert!(
        stdout.contains("cleared"),
        "flags {flags:?} must report what was cleared; got: {stdout}"
    );

    let ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    let app = ts
        .get_app("myapp")
        .expect("myapp survives a selective clear");
    assert_eq!(
        app.oauth1_token.is_some(),
        oauth1_kept,
        "flags {flags:?}: oauth1 slot"
    );
    assert_eq!(
        app.oauth2_tokens.contains_key("alice"),
        oauth2_alice_kept,
        "flags {flags:?}: oauth2 slot for alice"
    );
    assert_eq!(
        app.bearer_token.is_some(),
        bearer_kept,
        "flags {flags:?}: bearer slot"
    );
}

#[tokio::test]
#[serial_test::serial]
async fn test_xurl_dry_run_env_var_engages_dry_run() {
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    // No HTTP must fire.
    ts.mount(
        Mock::given(method("POST"))
            .and(path("/2/tweets"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0),
    )
    .await;

    // ALLOWLISTED ENV MUTATION (see tests/env_mutation_guard.rs).
    //
    // `--dry-run` binds to `XURL_DRY_RUN` through clap's `env =` attribute,
    // which reads the process at parse time and is not fed by `EnvOverrides`.
    // Injection cannot reach that binding, so proving it works means exporting
    // the variable. Do not convert this to the flag: the flag path is covered
    // by `test_post_dry_run_emits_envelope_and_skips_api`, and converting this
    // one would leave the env binding untested.
    unsafe {
        std::env::set_var("XURL_DRY_RUN", "1");
    }
    let (code, stdout, stderr) = run_at_with(
        &store,
        &api_env(ts.uri()),
        &["xr", "--output", "json", "post", "Hi", "--auth", "app"],
    )
    .await;
    unsafe {
        std::env::remove_var("XURL_DRY_RUN");
    }

    assert_eq!(
        code, 0,
        "XURL_DRY_RUN=1 must engage dry-run; stderr: {stderr}; stdout: {stdout}"
    );
    let v = parse_json(&stdout);
    assert_eq!(
        v["status"], "dry_run",
        "the env binding, not the flag, must have engaged dry-run"
    );
}

#[tokio::test]
async fn test_dry_run_help_advertised_on_post() {
    // Sanity: the help text MUST mention --dry-run so anc's p5-must-dry-run
    // gate sees the advertisement.
    let (code, stdout, _stderr) = run_isolated(&["xr", "post", "--help"]).await;
    assert_eq!(code, 0);
    assert!(
        stdout.contains("--dry-run"),
        "post --help must advertise --dry-run; got: {stdout}"
    );
}

#[tokio::test]
async fn test_root_help_lists_env_vars_and_exit_codes() {
    let (code, stdout, stderr) = run_isolated(&["xr", "--help"]).await;
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
#[tokio::test]
async fn test_every_subcommand_help_has_examples_block() {
    /// The first token of each entry in a help page's `Commands:` block, so
    /// the walk needs no access to the parser definition.
    fn commands_in(help: &str) -> Vec<String> {
        let mut lines = help.lines().skip_while(|l| *l != "Commands:");
        lines.next();
        lines
            .take_while(|l| !l.trim().is_empty())
            .filter_map(|l| l.strip_prefix("  ").filter(|rest| !rest.starts_with(' ')))
            .filter_map(|entry| entry.split_whitespace().next())
            .filter(|name| *name != "help")
            .map(str::to_string)
            .collect()
    }

    let mut paths: Vec<Vec<String>> = Vec::new();
    let mut missing: Vec<String> = Vec::new();
    let mut pending: Vec<Vec<String>> = vec![Vec::new()];
    while let Some(path) = pending.pop() {
        let mut args: Vec<String> = vec!["xr".to_string()];
        args.extend(path.iter().cloned());
        args.push("--help".to_string());
        let argv: Vec<&str> = args.iter().map(String::as_str).collect();
        let (code, stdout, stderr) = run_isolated(&argv).await;
        assert_eq!(
            code,
            0,
            "expected 0 for `{}` --help; stderr: {stderr}",
            path.join(" ")
        );
        for sub in commands_in(&stdout) {
            let mut next = path.clone();
            next.push(sub);
            pending.push(next);
        }
        if path.is_empty() {
            continue;
        }
        if !stdout.contains("Examples:") {
            missing.push(path.join(" "));
        }
        paths.push(path);
    }
    assert!(!paths.is_empty(), "Cli must expose at least one subcommand");
    assert!(
        missing.is_empty(),
        "the following subcommands' --help lacks an Examples: block:\n  {}",
        missing.join("\n  ")
    );
}

#[tokio::test]
async fn test_block_and_unblock_dry_run_envelopes_name_their_own_command() {
    // The two handlers share a body shape with mute/unmute, so a copied
    // `command` value would still emit a well-formed envelope and pass every
    // other gate. Pin the name each one reports.
    for (command, target) in [("block", "@spammer"), ("unblock", "@spammer")] {
        let (code, stdout, stderr) =
            run_isolated(&["xr", command, target, "--dry-run", "--output", "json"]).await;
        assert_eq!(
            code, 0,
            "expected 0 for `{command}` dry-run; stderr: {stderr}"
        );
        let v: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("valid JSON envelope");
        assert_eq!(v["status"], "dry_run", "envelope: {v}");
        assert_eq!(v["command"], command, "envelope: {v}");
        assert_eq!(v["target_username"], target, "envelope: {v}");
        assert_eq!(v["would_succeed"], true, "envelope: {v}");
    }
}

#[tokio::test]
async fn test_force_help_advertised_on_delete() {
    let (code, stdout, _stderr) = run_isolated(&["xr", "delete", "--help"]).await;
    assert_eq!(code, 0);
    assert!(
        stdout.contains("--force"),
        "delete --help must advertise --force; got: {stdout}"
    );
}

#[tokio::test]
async fn test_limit_help_advertised_globally() {
    let (code, stdout, _stderr) = run_isolated(&["xr", "--help"]).await;
    assert_eq!(code, 0);
    assert!(
        stdout.contains("--limit"),
        "xr --help must advertise --limit globally; got: {stdout}"
    );
}

/// The authenticated user the paging requests below are made for.
const PAGING_USER_ID: &str = "2244994945";

/// Every command that pages: its arguments, and the path its request lands on
/// once the authenticated user is resolved.
const PAGING_COMMANDS: &[(&[&str], &str)] = &[
    (&["search", "x"], "/2/tweets/search/recent"),
    (
        &["timeline"],
        "/2/users/2244994945/timelines/reverse_chronological",
    ),
    (&["mentions"], "/2/users/2244994945/mentions"),
    (&["bookmarks"], "/2/users/2244994945/bookmarks"),
    (&["likes"], "/2/users/2244994945/liked_tweets"),
    (&["following"], "/2/users/2244994945/following"),
    (&["followers"], "/2/users/2244994945/followers"),
    (&["muted"], "/2/users/2244994945/muting"),
    (&["blocked"], "/2/users/2244994945/blocking"),
    (&["dms"], "/2/dm_events"),
];

/// `--limit` and `--cursor` reach the wire as `max_results` and
/// `pagination_token` on every command that pages.
#[tokio::test]
async fn every_paging_command_sends_the_limit_and_cursor() {
    for (args, request_path) in PAGING_COMMANDS {
        let ts = CliMockServer::new().await;
        let tmp = TempDir::new().expect("tempdir");
        let store = tmp.path().join(".xurl");
        populate_oauth1_store(&store);
        ts.mount(
            Mock::given(method("GET"))
                .and(path("/2/users/me"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": {"id": PAGING_USER_ID, "name": "Paging User", "username": "paging"}
                }))),
        )
        .await;
        ts.mount(
            Mock::given(method("GET"))
                .and(path(*request_path))
                .and(wiremock::matchers::query_param("max_results", "25"))
                .and(wiremock::matchers::query_param(
                    "pagination_token",
                    "CURSOR-TOKEN",
                ))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": [],
                    "meta": {"result_count": 0}
                })))
                .expect(1),
        )
        .await;

        let mut argv = vec![
            "xr",
            "--output",
            "json",
            "--limit",
            "25",
            "--cursor",
            "CURSOR-TOKEN",
        ];
        argv.extend_from_slice(args);
        argv.extend_from_slice(&["--auth", "oauth1"]);
        let (code, stdout, stderr) = run_at_with(&store, &api_env(ts.uri()), &argv).await;
        assert_eq!(
            code, 0,
            "xr {args:?} must send max_results and pagination_token to {request_path}; \
             stderr: {stderr}; stdout: {stdout}"
        );
    }
}

/// The `--limit` and `--cursor` help name exactly the commands that page, so
/// the help cannot promise paging a command does not do.
#[test]
fn the_limit_and_cursor_help_name_exactly_the_commands_that_page() {
    use clap::CommandFactory;
    let paging: std::collections::BTreeSet<&str> =
        PAGING_COMMANDS.iter().map(|(args, _)| args[0]).collect();
    let command = cli::Cli::command();
    let subcommands: std::collections::BTreeSet<&str> = command
        .get_subcommands()
        .map(clap::Command::get_name)
        .collect();
    for flag in ["limit", "cursor"] {
        let help = command
            .get_arguments()
            .find(|arg| arg.get_id() == flag)
            .and_then(|arg| arg.get_long_help())
            .unwrap_or_else(|| panic!("--{flag} has long help"))
            .to_string();
        let named: std::collections::BTreeSet<&str> = help
            .split('`')
            .skip(1)
            .step_by(2)
            .filter(|word| subcommands.contains(word))
            .collect();
        assert_eq!(named, paging, "--{flag} help names {named:?}; help: {help}");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// U9: TTY-gated dialoguer + `--no-browser` env + headless auto-engage
// ═══════════════════════════════════════════════════════════════════════════

/// `xr auth default --no-interactive --output json` (no app_name supplied)
/// must emit the canonical `no-tty` envelope on stderr and skip any dialoguer
/// call.  Two apps are seeded so the picker would otherwise prompt.
#[tokio::test]
async fn test_auth_default_no_interactive_emits_no_tty_envelope() {
    use xdk::store::TokenStore;
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
    )
    .await;
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
#[tokio::test]
async fn test_auth_default_non_tty_emits_no_tty_envelope() {
    use xdk::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8"));
    ts.add_app("alpha", "ALPHA-CID", "ALPHA-SECRET")
        .expect("add alpha");
    ts.add_app("beta", "BETA-CID", "BETA-SECRET")
        .expect("add beta");
    drop(ts);

    let (code, _stdout, stderr) =
        run_at(&store, &["xr", "auth", "default", "--output", "json"]).await;
    assert_ne!(code, 0, "expected non-zero exit");
    let trimmed = stderr.trim();
    let v: serde_json::Value =
        serde_json::from_str(trimmed).unwrap_or_else(|_| panic!("envelope must parse: {trimmed}"));
    assert_eq!(v["status"], "error", "envelope status: {trimmed}");
    assert_eq!(v["reason"], "no-tty", "envelope reason: {trimmed}");
}

/// The named-app branch of `auth default` skips the picker entirely and writes
/// both defaults. App name and username are adjacent optional positionals, so
/// the store read-back is what pins which argument reached which setter.
#[tokio::test]
async fn test_auth_default_named_app_sets_default_app_and_user() {
    use xdk::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_credentialed_store(&store);
    // `populate_credentialed_store` already makes myapp the default; adding a
    // second app leaves a store where the wrong branch has somewhere to land.
    let mut seed = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    seed.add_app("other", "OTHER-CID", "OTHER-SECRET")
        .expect("add other");
    drop(seed);

    let (code, stdout, stderr) = run_at(&store, &["xr", "auth", "default", "other"]).await;
    assert_eq!(code, 0, "auth default other failed; stderr: {stderr}");
    assert!(
        stdout.contains("Default app set to \"other\""),
        "expected the named app in the message; got: {stdout}"
    );

    let (code2, stdout2, stderr2) =
        run_at(&store, &["xr", "auth", "default", "myapp", "alice"]).await;
    assert_eq!(
        code2, 0,
        "auth default myapp alice failed; stderr: {stderr2}"
    );
    assert!(
        stdout2.contains("Default app set to \"myapp\""),
        "expected the app line; got: {stdout2}"
    );
    assert!(
        stdout2.contains("Default user set to \"alice\""),
        "expected the user line; got: {stdout2}"
    );

    let ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    assert_eq!(ts.get_default_app(), "myapp");
    assert_eq!(ts.get_default_user("myapp"), "alice");
    assert_eq!(
        ts.get_default_user("other"),
        "",
        "the username positional must not have reached the other app"
    );
}

/// `xr auth oauth2 --help` must advertise the new `XURL_NO_BROWSER` env var.
#[tokio::test]
async fn test_auth_oauth2_help_advertises_no_browser_env_var() {
    let (code, stdout, _stderr) = run_isolated(&["xr", "auth", "oauth2", "--help"]).await;
    assert_eq!(code, 0);
    assert!(
        stdout.contains("XURL_NO_BROWSER"),
        "auth oauth2 --help must advertise XURL_NO_BROWSER env var; got:\n{stdout}"
    );
}

/// Registers one credentialed app in `store` through the CLI, so a
/// sign-in has a client id to build its URL from.
async fn register_app_at(store: &Path) {
    let (code, _stdout, stderr) = run_at(
        store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "MYAPP-CLIENT-ID",
            "--client-secret",
            "MYAPP-SECRET",
        ],
    )
    .await;
    assert_eq!(code, 0, "apps add failed; stderr: {stderr}");
}

/// `xr auth oauth2 --no-browser --output json` (no `--step`) emits the
/// canonical `{"status":"awaiting_callback","url":"..."}` envelope on stdout
/// and exits 0; the user is expected to invoke step 2 separately. Validates
/// the U9 "explicit --no-browser without --step" auto-promotion to step 1.
///
/// Uses a subprocess with `XURL_TOKEN_STORE` pointed at a tempdir store so
/// the OAuth2 step-1 pending state lands beside that store.
#[tokio::test]
async fn test_auth_oauth2_no_browser_emits_awaiting_callback_envelope() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    register_app_at(&store).await;
    let output = common::xr_with_store(&store)
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
#[tokio::test]
async fn test_auth_oauth2_xurl_no_browser_env_engages_headless_flow() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    register_app_at(&store).await;
    let output = common::xr_with_store(&store)
        .env("XURL_NO_BROWSER", "1")
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
#[tokio::test]
async fn test_auth_oauth2_auto_engages_headless_when_stdout_not_tty() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    register_app_at(&store).await;
    let output = common::xr_with_store(&store)
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

// ── Injected environment overrides (U2) ─────────────────────────────────────

#[tokio::test]
async fn test_injected_redirect_uri_takes_env_precedence_without_touching_process() {
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
            "--redirect-uri",
            "https://stored.example.com/cb",
        ],
    )
    .await;
    assert_eq!(code, 0, "setup failed; stderr: {stderr}");

    let overrides = xdk::config::EnvOverrides {
        redirect_uri: Some("https://injected.example.com/cb".into()),
        ..xdk::config::EnvOverrides::default()
    };
    let (code2, stdout2, stderr2) = run_at_with(
        &store,
        &overrides,
        &["xr", "auth", "apps", "redirect-uri", "get", "myapp"],
    )
    .await;

    assert_eq!(code2, 0, "get failed; stderr: {stderr2}");
    assert!(
        stdout2.contains("https://injected.example.com/cb"),
        "injected value must win over the stored one: {stdout2}"
    );
    assert!(
        stdout2.contains("REDIRECT_URI environment variable"),
        "injected value carries env provenance: {stdout2}"
    );
    assert!(
        stdout2.contains("https://stored.example.com/cb"),
        "stored value is still reported alongside: {stdout2}"
    );
}

#[tokio::test]
async fn test_absent_redirect_uri_override_lets_stored_value_win() {
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
            "https://stored.example.com/cb",
        ],
    )
    .await;
    assert_eq!(code, 0);

    let (code2, stdout2, _) = run_at_with(
        &store,
        &xdk::config::EnvOverrides::default(),
        &["xr", "auth", "apps", "redirect-uri", "get", "myapp"],
    )
    .await;

    assert_eq!(code2, 0);
    assert!(
        stdout2.contains("https://stored.example.com/cb"),
        "with no override the stored value wins: {stdout2}"
    );
    assert!(
        stdout2.contains("app config"),
        "and reports app-config provenance: {stdout2}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Env bearer token counts as the `app` scheme in shortcut auth resolution
// ═══════════════════════════════════════════════════════════════════════════

/// Overrides pointing the client at a stubbed server with an env bearer.
fn api_env_with_bearer(base_url: &str, token: &str) -> xdk::config::EnvOverrides {
    xdk::config::EnvOverrides {
        bearer_token: Some(token.to_string()),
        ..api_env(base_url)
    }
}

/// `XURL_BEARER_TOKEN=… xr search "x"` on an empty store must resolve the
/// bearer from the environment and reach the endpoint, not exit 77.
#[tokio::test]
async fn test_env_bearer_resolves_shortcut_on_empty_store() {
    use wiremock::matchers::header;
    let ts = CliMockServer::new().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .and(header("Authorization", "Bearer env-bearer-value"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"id": "1", "text": "hello"}],
                "meta": {"result_count": 1}
            })))
            .expect(1),
    )
    .await;

    let (code, stdout, stderr) = run_at_with(
        &store,
        &api_env_with_bearer(ts.uri(), "env-bearer-value"),
        &["xr", "search", "hello"],
    )
    .await;

    assert_eq!(
        code, 0,
        "search with the env bearer on an empty store must succeed; stderr: {stderr}; stdout: {stdout}"
    );
    assert!(
        !store.exists(),
        "resolving the env bearer must not write the store"
    );
}

/// `auth status --output json` marks the active app's bearer as present from
/// the environment when `XURL_BEARER_TOKEN` is set and the app stores none.
#[tokio::test]
async fn test_status_json_reports_env_bearer_on_active_app() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_app_store(&store);

    let env = xdk::config::EnvOverrides {
        bearer_token: Some("env-bearer-value".to_string()),
        ..xdk::config::EnvOverrides::default()
    };
    let (code, stdout, stderr) =
        run_at_with(&store, &env, &["xr", "--output", "json", "auth", "status"]).await;
    assert_eq!(code, 0, "auth status failed; stderr: {stderr}");

    let v = parse_json(&stdout);
    let arr = v["apps"]
        .as_array()
        .expect("status emits apps as a JSON array");
    let entry = arr
        .iter()
        .find(|e| e["name"] == "myapp")
        .expect("myapp entry present");
    assert_eq!(
        entry["bearer"],
        serde_json::Value::Bool(true),
        "bearer must be reported present; got entry: {entry}"
    );
    assert_eq!(
        entry["bearer_source"], "env",
        "bearer_source must name the environment; got entry: {entry}"
    );
    assert!(
        !stdout.contains("env-bearer-value"),
        "the token value must never appear in status output"
    );
}

/// The text rendering of `auth status` names the env variable beside the
/// bearer mark for the active app.
#[tokio::test]
async fn test_status_text_reports_env_bearer_on_active_app() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_app_store(&store);

    let env = xdk::config::EnvOverrides {
        bearer_token: Some("env-bearer-value".to_string()),
        ..xdk::config::EnvOverrides::default()
    };
    let (code, stdout, stderr) = run_at_with(&store, &env, &["xr", "auth", "status"]).await;
    assert_eq!(code, 0, "auth status failed; stderr: {stderr}");
    assert!(
        stdout.contains("bearer: \u{2713} [XURL_BEARER_TOKEN environment variable]"),
        "status text must mark the bearer as present from the environment; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("env-bearer-value"),
        "the token value must never appear in status output"
    );
}

/// A bearer stored on the app reports `bearer_source: store`, so the two
/// origins are distinguishable in the envelope.
#[tokio::test]
async fn test_status_json_reports_store_bearer_source() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    let (code, stdout, stderr) =
        run_at(&store, &["xr", "--output", "json", "auth", "status"]).await;
    assert_eq!(code, 0, "auth status failed; stderr: {stderr}");
    let v = parse_json(&stdout);
    let entry = v["apps"]
        .as_array()
        .expect("status emits apps as a JSON array")
        .iter()
        .find(|e| e["name"] == "myapp")
        .cloned()
        .expect("myapp entry present");
    assert_eq!(entry["bearer"], serde_json::Value::Bool(true));
    assert_eq!(
        entry["bearer_source"], "store",
        "a stored bearer reports its origin; got entry: {entry}"
    );
}

/// An app with no bearer anywhere omits `bearer_source` entirely.
#[tokio::test]
async fn test_status_json_omits_bearer_source_when_absent() {
    use xdk::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");

    let (code, stdout, stderr) =
        run_at(&store, &["xr", "--output", "json", "auth", "status"]).await;
    assert_eq!(code, 0, "auth status failed; stderr: {stderr}");
    let v = parse_json(&stdout);
    let entry = v["apps"]
        .as_array()
        .expect("status emits apps as a JSON array")
        .iter()
        .find(|e| e["name"] == "myapp")
        .cloned()
        .expect("myapp entry present");
    assert_eq!(entry["bearer"], serde_json::Value::Bool(false));
    assert!(
        entry.get("bearer_source").is_none(),
        "bearer_source must be omitted when no bearer is present; got entry: {entry}"
    );
}

/// On a fresh store the text status still reports the env bearer, so a
/// bearer-only setup with nothing registered is visible.
#[tokio::test]
async fn test_status_text_reports_env_bearer_on_fresh_store() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let env = xdk::config::EnvOverrides {
        bearer_token: Some("env-bearer-value".to_string()),
        ..xdk::config::EnvOverrides::default()
    };
    let (code, stdout, stderr) = run_at_with(&store, &env, &["xr", "auth", "status"]).await;
    assert_eq!(code, 0, "auth status failed; stderr: {stderr}");
    assert!(
        stdout.contains("bearer: \u{2713} [XURL_BEARER_TOKEN environment variable]"),
        "the env bearer must be reported on a fresh store; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("env-bearer-value"),
        "the token value must never appear in status output"
    );
}

/// With the env bearer set, an empty active app, and credentials stored on
/// another app, a user-context endpoint still names the other app: the env
/// bearer counts as available but does not hide the wrong-app recovery hint.
#[tokio::test]
async fn test_env_bearer_keeps_wrong_app_hint_on_user_context_endpoint() {
    use xdk::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8"));
    ts.add_app("work", "WORK-CLIENT-ID", "WORK-SECRET")
        .expect("add work");
    ts.save_oauth2_token_for_app("work", "alice", "ACCESS", "REFRESH", 1_900_000_000)
        .expect("save_oauth2");
    // A credential-less app that is the active one: registration promotes
    // past such a default, so this state has to be built on purpose.
    ts.add_app("blank", "", "").expect("add blank");
    ts.set_default_app("blank").expect("set_default_app");

    let env = xdk::config::EnvOverrides {
        bearer_token: Some("env-bearer-value".to_string()),
        ..xdk::config::EnvOverrides::default()
    };
    let (code, _stdout, stderr) =
        run_at_with(&store, &env, &["xr", "--output", "json", "like", "12345"]).await;
    assert_eq!(code, 2, "wrong-app mismatch exits 2; stderr: {stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope");
    assert_eq!(v["reason"], "auth-method-mismatch", "envelope: {v}");
    assert_eq!(
        v["other_apps_with_creds"],
        serde_json::json!(["work"]),
        "the wrong-app hint must survive the env bearer; envelope: {v}"
    );
    assert_eq!(
        v["available_in_app"],
        serde_json::json!(["app"]),
        "available_in_app stays truthful about the env bearer; envelope: {v}"
    );
    assert!(
        v["message"].as_str().unwrap_or("").contains("--app"),
        "text names the --app recovery; envelope: {v}"
    );
}

/// `auth apps list --output json` carries `bearer_source` through the same
/// builder as `auth status`.
#[tokio::test]
async fn test_apps_list_json_reports_env_bearer_source() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_app_store(&store);

    let env = xdk::config::EnvOverrides {
        bearer_token: Some("env-bearer-value".to_string()),
        ..xdk::config::EnvOverrides::default()
    };
    let (code, stdout, stderr) = run_at_with(
        &store,
        &env,
        &["xr", "--output", "json", "auth", "apps", "list"],
    )
    .await;
    assert_eq!(code, 0, "apps list failed; stderr: {stderr}");
    let v = parse_json(&stdout);
    let entry = v["apps"]
        .as_array()
        .expect("apps list emits apps as a JSON array")
        .iter()
        .find(|e| e["name"] == "myapp")
        .cloned()
        .expect("myapp entry present");
    assert_eq!(entry["bearer"], serde_json::Value::Bool(true));
    assert_eq!(entry["bearer_source"], "env", "got entry: {entry}");
    assert!(!stdout.contains("env-bearer-value"));
}

/// When both a stored bearer and the env bearer exist, the env value wins
/// the precedence chain and status reports it as the source.
#[tokio::test]
async fn test_status_json_env_bearer_wins_over_stored_bearer() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_bearer_store(&store);

    let env = xdk::config::EnvOverrides {
        bearer_token: Some("env-bearer-value".to_string()),
        ..xdk::config::EnvOverrides::default()
    };
    let (code, stdout, stderr) =
        run_at_with(&store, &env, &["xr", "--output", "json", "auth", "status"]).await;
    assert_eq!(code, 0, "auth status failed; stderr: {stderr}");
    let v = parse_json(&stdout);
    let entry = v["apps"]
        .as_array()
        .expect("status emits apps as a JSON array")
        .iter()
        .find(|e| e["name"] == "myapp")
        .cloned()
        .expect("myapp entry present");
    assert_eq!(entry["bearer"], serde_json::Value::Bool(true));
    assert_eq!(
        entry["bearer_source"], "env",
        "env wins over the stored bearer; got entry: {entry}"
    );
}

/// `--app NAME` moves the env bearer mark to NAME's entry and off the default.
#[tokio::test]
async fn test_status_json_env_bearer_follows_app_flag() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_app_store(&store)
        .add_app("otherapp", "OTHER-CLIENT-ID", "OTHER-SECRET")
        .expect("add otherapp");

    let env = xdk::config::EnvOverrides {
        bearer_token: Some("env-bearer-value".to_string()),
        ..xdk::config::EnvOverrides::default()
    };
    let (code, stdout, stderr) = run_at_with(
        &store,
        &env,
        &[
            "xr", "--app", "otherapp", "--output", "json", "auth", "status",
        ],
    )
    .await;
    assert_eq!(code, 0, "auth status failed; stderr: {stderr}");
    let v = parse_json(&stdout);
    let arr = v["apps"]
        .as_array()
        .expect("status emits apps as a JSON array");
    let other = arr
        .iter()
        .find(|e| e["name"] == "otherapp")
        .expect("otherapp entry present");
    let myapp = arr
        .iter()
        .find(|e| e["name"] == "myapp")
        .expect("myapp entry present");
    assert_eq!(other["bearer_source"], "env", "got: {other}");
    assert_eq!(
        myapp["bearer"],
        serde_json::Value::Bool(false),
        "got: {myapp}"
    );
    assert!(myapp.get("bearer_source").is_none(), "got: {myapp}");
}

// ═══════════════════════════════════════════════════════════════════════════
// U10: an empty store is empty; a store that failed to load is never
// overwritten; registration promotes past a credential-less default
// ═══════════════════════════════════════════════════════════════════════════

/// The README Quick Start on a fresh install: register an app, then start
/// the headless sign-in. The authorization URL must carry the registered
/// app's client id, which requires that app to be the default.
#[tokio::test]
async fn test_fresh_store_quick_start_uses_registered_client_id() {
    let tmp = TempDir::new().expect("tempdir");
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
            "MYAPP-CLIENT-ID",
            "--client-secret",
            "MYAPP-SECRET",
        ],
    )
    .await;
    assert_eq!(code, 0, "apps add failed; stderr: {stderr}");

    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "1",
        ],
    )
    .await;
    assert_eq!(code, 0, "step 1 failed; stderr: {stderr}");
    let v = parse_json(&stdout);
    let url = v["auth_url"].as_str().expect("auth_url present");
    assert!(
        url.contains("client_id=MYAPP-CLIENT-ID"),
        "the sign-in URL must carry the registered app's client id; got: {url}"
    );
}

/// A store file neither parser accepts is reported, never overwritten.
#[tokio::test]
async fn test_unparseable_store_is_never_overwritten() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let garbage = b"\x00\x01 this is not yaml or json \x02\x03";
    std::fs::write(&store, garbage).expect("seed the damaged store");

    let (code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "MYAPP-CLIENT-ID",
            "--client-secret",
            "MYAPP-SECRET",
        ],
    )
    .await;
    assert_ne!(
        code, 0,
        "apps add must refuse a damaged store; stderr: {stderr}"
    );
    assert!(
        stderr.contains(store.to_str().expect("utf-8 path")),
        "the error must name the store path; stderr: {stderr}"
    );
    let after = std::fs::read(&store).expect("store still readable");
    assert_eq!(
        after, garbage,
        "a store that failed to load must be byte-identical afterward"
    );
}

/// `auth status` on an empty store prints the registration sentence and
/// exits 0; structured mode prints an empty array.
#[tokio::test]
async fn test_status_on_empty_store_names_the_registration_command() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let (code, stdout, stderr) = run_at(&store, &["xr", "auth", "status"]).await;
    assert_eq!(code, 0, "an empty store is not an error; stderr: {stderr}");
    assert!(
        stdout.contains(
            "No apps registered. Run: xr auth apps add NAME --client-id ID --client-secret SECRET"
        ),
        "got:\n{stdout}"
    );

    let (code, stdout, _) = run_at(&store, &["xr", "--output", "json", "auth", "status"]).await;
    assert_eq!(code, 0);
    assert_eq!(
        parse_json(&stdout),
        serde_json::json!({"status": "ok", "apps": []}),
        "got: {stdout}"
    );
}

/// With `XURL_BEARER_TOKEN` set, the empty-store text adds the bearer line.
#[tokio::test]
async fn test_status_on_empty_store_reports_the_env_bearer() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let env = xdk::config::EnvOverrides {
        bearer_token: Some("env-bearer-value".to_string()),
        ..xdk::config::EnvOverrides::default()
    };

    let (code, stdout, stderr) = run_at_with(&store, &env, &["xr", "auth", "status"]).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("No apps registered"), "got:\n{stdout}");
    assert!(
        stdout.contains("bearer: \u{2713} [XURL_BEARER_TOKEN environment variable]"),
        "got:\n{stdout}"
    );
    assert!(!stdout.contains("env-bearer-value"));
}

/// The first registered app is the default and the message names the
/// plain sign-in; the second is not and its message names `--app`.
#[tokio::test]
async fn test_apps_add_message_names_the_next_command() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "A",
            "--client-secret",
            "B",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("(default)"),
        "first app is the default: {stdout}"
    );
    assert!(stdout.contains("Next: xr auth oauth2"), "got: {stdout}");
    assert!(
        !stdout.contains("--app"),
        "no --app needed for the default: {stdout}"
    );

    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "apps",
            "add",
            "second",
            "--client-id",
            "C",
            "--client-secret",
            "D",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    let v = parse_json(&stdout);
    assert_eq!(v["default"], serde_json::Value::Bool(false), "got: {v}");
    assert_eq!(v["next_step"]["action"], "sign-in", "got: {v}");
    assert_eq!(
        v["next_step"]["command"], "xr auth oauth2 --no-browser --step 1 --app second",
        "a structured caller gets the headless form naming the app; got: {v}"
    );
}

/// `apps add` rejects a name outside the allowed set.
#[tokio::test]
async fn test_apps_add_rejects_a_spaced_name() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let (code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "my app",
            "--client-id",
            "A",
            "--client-secret",
            "B",
        ],
    )
    .await;
    assert_ne!(code, 0, "a spaced name must be rejected");
    assert!(stderr.contains("my app"), "names the value; got: {stderr}");
}

/// Sign-in on an empty store refuses before building a URL or writing the
/// pending file, and hands an agent a registration template.
#[tokio::test]
async fn test_oauth2_on_empty_store_refuses_with_register_app() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "1",
        ],
    )
    .await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stdout.is_empty(), "no URL is printed; stdout: {stdout}");
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope");
    assert_eq!(v["reason"], "client-credentials-missing", "got: {v}");
    assert_eq!(v["next_step"]["action"], "register-app", "got: {v}");
    assert!(v["next_step"]["template"].is_string(), "got: {v}");
    assert!(v["next_step"]["command"].is_null(), "never both; got: {v}");

    let pending = tmp.path().join(".xurl.pending");
    assert!(!pending.exists(), "no pending file is written");
}

/// With a credentialed app present but a credential-less one selected, the
/// refusal names the app to use.
#[tokio::test]
async fn test_oauth2_with_blank_target_names_the_credentialed_app() {
    use xdk::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8"));
    ts.add_app("myapp", "MYAPP-CLIENT-ID", "MYAPP-SECRET")
        .expect("add myapp");
    ts.add_app("blank", "", "").expect("add blank");

    let (code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--app",
            "blank",
            "--output",
            "json",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "1",
        ],
    )
    .await;
    assert_eq!(code, 2, "stderr: {stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope");
    assert_eq!(v["reason"], "client-credentials-missing", "got: {v}");
    assert_eq!(v["app"], "blank", "got: {v}");
    assert_eq!(v["next_step"]["action"], "select-app", "got: {v}");
    assert_eq!(
        v["next_step"]["command"], "xr auth oauth2 --no-browser --step 1 --app myapp",
        "got: {v}"
    );
}

/// `--app NAME` picks that app's client id for the sign-in URL.
#[tokio::test]
async fn test_oauth2_step1_uses_the_named_app_client_id() {
    use xdk::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8"));
    ts.add_app("first", "FIRST-CLIENT-ID", "FIRST-SECRET")
        .expect("add first");
    ts.add_app("myapp", "MYAPP-CLIENT-ID", "MYAPP-SECRET")
        .expect("add myapp");

    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--app",
            "myapp",
            "--output",
            "json",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "1",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    let url = parse_json(&stdout)["auth_url"]
        .as_str()
        .expect("auth_url present")
        .to_string();
    assert!(url.contains("client_id=MYAPP-CLIENT-ID"), "got: {url}");
}

/// Env credentials drive a sign-in on an empty store and the token lands on
/// a lazily created `default`.
#[tokio::test]
async fn test_env_client_id_signs_in_on_an_empty_store() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let env = xdk::config::EnvOverrides {
        client_id: Some("ENV-CLIENT-ID".to_string()),
        client_secret: Some("ENV-CLIENT-SECRET".to_string()),
        ..xdk::config::EnvOverrides::default()
    };

    let (code, stdout, stderr) = run_at_with(
        &store,
        &env,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "1",
        ],
    )
    .await;
    assert_eq!(
        code, 0,
        "env credentials must start the flow; stderr: {stderr}"
    );
    let url = parse_json(&stdout)["auth_url"]
        .as_str()
        .expect("auth_url present")
        .to_string();
    assert!(url.contains("client_id=ENV-CLIENT-ID"), "got: {url}");
}

/// Registering beside exported env credentials writes no env secret.
#[tokio::test]
async fn test_apps_add_with_env_credentials_writes_no_env_secret() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let env = xdk::config::EnvOverrides {
        client_id: Some("ENV-CLIENT-ID".to_string()),
        client_secret: Some("ENV-CLIENT-SECRET".to_string()),
        ..xdk::config::EnvOverrides::default()
    };

    let (code, _stdout, stderr) = run_at_with(
        &store,
        &env,
        &[
            "xr",
            "auth",
            "apps",
            "add",
            "myapp",
            "--client-id",
            "MYAPP-ID",
            "--client-secret",
            "MYAPP-SECRET",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    let written = std::fs::read_to_string(&store).expect("store written");
    assert!(
        written.contains("myapp"),
        "the app is registered: {written}"
    );
    assert!(
        !written.contains("ENV-CLIENT-SECRET"),
        "no env secret is persisted: {written}"
    );
}

/// The text rendering of the sign-in refusal names the app and the fix, and
/// a structured caller is not required to read it.
#[tokio::test]
async fn test_oauth2_refusal_text_names_the_registration_command() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let (code, stdout, stderr) = run_at(
        &store,
        &["xr", "auth", "oauth2", "--no-browser", "--step", "1"],
    )
    .await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stdout.is_empty(), "no URL is printed; stdout: {stdout}");
    assert!(
        stderr.contains("no app carries client credentials"),
        "got: {stderr}"
    );
}

/// A bad app name is a usage mistake, not an authentication failure.
#[tokio::test]
async fn test_apps_add_spaced_name_is_a_validation_error() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let (code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "apps",
            "add",
            "my app",
            "--client-id",
            "A",
            "--client-secret",
            "B",
        ],
    )
    .await;
    assert_eq!(code, 1, "a rejected name is not exit 77; stderr: {stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope");
    assert_eq!(v["reason"], "validation", "got: {v}");
}

/// A damaged store is reported as damaged, not as an empty one, by the
/// read verbs as well as the write ones.
#[rstest::rstest]
#[case::status(&["xr", "auth", "status"])]
#[case::apps_list(&["xr", "auth", "apps", "list"])]
#[case::status_json(&["xr", "--output", "json", "auth", "status"])]
#[tokio::test]
async fn test_read_verbs_report_a_damaged_store(#[case] args: &[&str]) {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    std::fs::write(&store, b"\x00\x01 neither yaml nor json \x02").expect("seed");

    let (code, stdout, stderr) = run_at(&store, args).await;
    assert_ne!(
        code, 0,
        "a damaged store is not an empty one; args: {args:?}"
    );
    assert!(
        !stdout.contains("No apps registered"),
        "must not claim the store is empty; stdout: {stdout}"
    );
    assert!(
        stderr.contains(store.to_str().expect("utf-8 path")),
        "the error names the path; stderr: {stderr}"
    );
}

/// `auth apps list` mirrors `auth status` on an empty store.
#[tokio::test]
async fn test_apps_list_on_empty_store_matches_status() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let (code, stdout, stderr) = run_at(&store, &["xr", "auth", "apps", "list"]).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("No apps registered. Run: xr auth apps add"),
        "got:\n{stdout}"
    );

    let (code, stdout, _) =
        run_at(&store, &["xr", "--output", "json", "auth", "apps", "list"]).await;
    assert_eq!(code, 0);
    assert_eq!(
        parse_json(&stdout),
        serde_json::json!({"status": "ok", "apps": []}),
        "got: {stdout}"
    );

    let env = xdk::config::EnvOverrides {
        bearer_token: Some("env-bearer-value".to_string()),
        ..xdk::config::EnvOverrides::default()
    };
    let (code, stdout, _) = run_at_with(&store, &env, &["xr", "auth", "apps", "list"]).await;
    assert_eq!(code, 0);
    assert!(
        stdout.contains("bearer: \u{2713} [XURL_BEARER_TOKEN environment variable]"),
        "got:\n{stdout}"
    );
}

/// Naming a credential-less app explicitly, with no alternative available,
/// reports that app rather than the generic sentence.
#[tokio::test]
async fn test_oauth2_explicit_blank_app_names_that_app() {
    use xdk::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8"));
    ts.add_app("blank", "", "").expect("add blank");

    let (code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--app",
            "blank",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "1",
        ],
    )
    .await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stderr.contains("\"blank\""), "names the app; got: {stderr}");
}

/// Every structured error the binary emits must validate against the
/// declared envelope body, including the verb-local shapes that carry their
/// own fields. The schema closes the object, so an emitter that grew a key
/// the type does not name would publish a schema that rejects its own output.
#[rstest::rstest]
#[case::unknown_schema(&["validate", "--schema", "nope", "-"], "{}", "unknown-schema")]
#[case::invalid_json(&["validate", "--schema", "post", "-"], "not json at all", "invalid-json")]
#[case::validation_failed(&["validate", "--schema", "post", "-"], "{\"unexpected\": 1}", "validation-failed")]
fn test_verb_local_error_envelopes_match_the_declared_body(
    #[case] args: &[&str],
    #[case] stdin: &str,
    #[case] expected_reason: &str,
) {
    use std::io::Write as _;
    let mut child = common::xr_std()
        .args(args)
        .arg("--output")
        .arg("json")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn xr");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");
    let emitted = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let mut value: serde_json::Value = serde_json::from_str(emitted.trim())
        .unwrap_or_else(|e| panic!("envelope must parse ({e}): {emitted}"));
    let obj = value.as_object_mut().expect("an object");
    assert_eq!(obj["reason"], expected_reason, "got: {emitted}");
    obj.remove("status");
    serde_json::from_value::<xurl::cli::envelope::ErrorBody>(value)
        .unwrap_or_else(|e| panic!("undeclared key in a verb-local envelope ({e}): {emitted}"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Boolean flags with an optional value require `=` for that value; the
// parse-level cases live in `src/cli/parse_tests.rs`
// ═══════════════════════════════════════════════════════════════════════════

/// `xr --quiet whoami` on an empty store reaches `whoami` and its auth
/// error, not the raw-mode "No URL provided" path.
#[tokio::test]
#[serial_test::parallel]
async fn test_quiet_whoami_reaches_whoami() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--quiet", "whoami"]).await;
    assert_eq!(
        code, 77,
        "whoami on an empty store exits 77; stderr: {stderr}"
    );
    assert!(
        !stderr.contains("No URL provided"),
        "the command word must not be consumed as the flag value; stderr: {stderr}"
    );
}

/// Spawns the built binary against `store` and the mocked API, with one env
/// binding set, and returns (exit code, stdout, stderr).
fn spawn_with_env(
    store: &Path,
    base_url: &str,
    env: &[(&str, &str)],
    args: &[&str],
) -> (i32, String, String) {
    let mut cmd = common::xr_with_store(store);
    cmd.env("API_BASE_URL", base_url)
        .env("XURL_BEARER_TOKEN", "env-bearer-value");
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.args(args).output().expect("spawn xr");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Mounts the bearer-capable search endpoint returning a compact JSON body.
async fn mock_search(ts: &CliMockServer) {
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(r#"{"data":[{"id":"1","text":"hi"}]}"#),
            ),
    )
    .await;
}

const SEARCH_PATH: &str = "/2/tweets/search/recent?query=hi";

/// An env-provided true is read through the value parser and `--flag=false`
/// still overrides it, for each of the six env-backed boolean flags.
#[tokio::test]
async fn test_env_true_then_equals_false_verbose() {
    let ts = CliMockServer::new().await;
    mock_search(&ts).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let env = [("XURL_VERBOSE", "true")];

    let (code, _, stderr) = spawn_with_env(&store, ts.uri(), &env, &["--auth", "app", SEARCH_PATH]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stderr.contains("> GET"),
        "env true must enable verbose; stderr: {stderr}"
    );

    let (code, _, stderr) = spawn_with_env(
        &store,
        ts.uri(),
        &env,
        &["--verbose=false", "--auth", "app", SEARCH_PATH],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        !stderr.contains("> GET"),
        "--verbose=false must clear it; stderr: {stderr}"
    );
}

#[tokio::test]
async fn test_env_true_then_equals_false_quiet() {
    let ts = CliMockServer::new().await;
    mock_search(&ts).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let env = [("XURL_QUIET", "true"), ("XURL_VERBOSE", "true")];

    let (code, _, stderr) = spawn_with_env(&store, ts.uri(), &env, &["--auth", "app", SEARCH_PATH]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        !stderr.contains("> GET"),
        "env quiet must silence verbose; stderr: {stderr}"
    );

    let (code, _, stderr) = spawn_with_env(
        &store,
        ts.uri(),
        &env,
        &["--quiet=false", "--auth", "app", SEARCH_PATH],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stderr.contains("> GET"),
        "--quiet=false must clear it; stderr: {stderr}"
    );
}

#[tokio::test]
async fn test_env_true_then_equals_false_raw() {
    let ts = CliMockServer::new().await;
    mock_search(&ts).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let env = [("XURL_RAW", "true")];

    let (code, stdout, stderr) = spawn_with_env(
        &store,
        ts.uri(),
        &env,
        &["--output", "json", "--auth", "app", SEARCH_PATH],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        !stdout.contains("\n  "),
        "env raw must print the body as sent; stdout: {stdout}"
    );

    let (code, stdout, stderr) = spawn_with_env(
        &store,
        ts.uri(),
        &env,
        &[
            "--raw=false",
            "--output",
            "json",
            "--auth",
            "app",
            SEARCH_PATH,
        ],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("\n  "),
        "--raw=false must pretty-print; stdout: {stdout}"
    );
}

#[test]
fn test_env_true_then_equals_false_dry_run() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let env = [("XURL_DRY_RUN", "true")];
    let add = [
        "--output",
        "json",
        "auth",
        "apps",
        "add",
        "myapp",
        "--client-id",
        "a",
        "--client-secret",
        "b",
    ];

    let (code, stdout, stderr) = spawn_with_env(&store, "http://127.0.0.1:9", &env, &add);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("\"dry_run\""),
        "env dry-run must emit the dry-run envelope; stdout: {stdout}"
    );
    assert!(!store.exists(), "dry run must not write the store");

    let mut args = vec!["--dry-run=false"];
    args.extend_from_slice(&add);
    let (code, stdout, stderr) = spawn_with_env(&store, "http://127.0.0.1:9", &env, &args);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        !stdout.contains("\"dry_run\""),
        "--dry-run=false must register; stdout: {stdout}"
    );
    assert!(store.exists(), "the real run writes the store");
}

#[test]
fn test_env_true_then_equals_false_no_browser() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let env = [("XURL_NO_BROWSER", "true")];
    let base = ["--dry-run", "--output", "json", "auth", "oauth2"];

    let (code, stdout, stderr) = spawn_with_env(&store, "http://127.0.0.1:9", &env, &base);
    assert_eq!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("dry-run envelope");
    assert_eq!(
        v["no_browser"], true,
        "env true must set no_browser; got {v}"
    );

    let mut args = base.to_vec();
    args.push("--no-browser=false");
    let (code, stdout, stderr) = spawn_with_env(&store, "http://127.0.0.1:9", &env, &args);
    assert_eq!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("dry-run envelope");
    assert_eq!(
        v["no_browser"], false,
        "--no-browser=false must clear it; got {v}"
    );
}

/// `no_interactive` has no observable effect without a terminal on stdin,
/// so this pair asserts the parse: the env value is accepted and the
/// `=false` form still reaches the command.
#[test]
fn test_env_true_then_equals_false_no_interactive() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let env = [("XURL_NO_INTERACTIVE", "true")];

    let (code, _, stderr) = spawn_with_env(
        &store,
        "http://127.0.0.1:9",
        &env,
        &["auth", "apps", "list"],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let (code, _, stderr) = spawn_with_env(
        &store,
        "http://127.0.0.1:9",
        &env,
        &["--no-interactive=false", "auth", "apps", "list"],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
}

// ═══════════════════════════════════════════════════════════════════════════
// U12a: every message-shaped auth verb emits a status-ok envelope
// ═══════════════════════════════════════════════════════════════════════════

/// Seeds a store with one registered app and returns the store path guard.
fn seeded_store(tmp: &TempDir) -> std::path::PathBuf {
    let store = tmp.path().join(".xurl");
    populate_app_store(&store);
    store
}

/// Every auth verb whose structured success is a message object today must
/// carry `status: ok` so an agent branches on one field across the surface.
#[rstest::rstest]
#[case::apps_add(&["auth", "apps", "add", "fresh", "--client-id", "A", "--client-secret", "B"], &["message", "default", "next_step"])]
#[case::apps_update(&["auth", "apps", "update", "myapp", "--client-id", "NEW"], &["message"])]
#[case::apps_redirect_uri_set(&["auth", "apps", "redirect-uri", "set", "myapp", "https://example.test/cb"], &["app", "redirect_uri"])]
#[case::apps_redirect_uri_get(&["auth", "apps", "redirect-uri", "get", "myapp"], &["app", "effective_redirect_uri"])]
#[case::default(&["auth", "default", "myapp"], &["message"])]
#[case::app_bearer(&["auth", "app", "--bearer-token", "TOK"], &["message"])]
#[case::apps_remove(&["auth", "apps", "remove", "myapp", "--force"], &["message"])]
#[tokio::test]
async fn test_auth_verbs_emit_status_ok(#[case] args: &[&str], #[case] expected_keys: &[&str]) {
    let tmp = TempDir::new().expect("tempdir");
    let store = seeded_store(&tmp);

    let mut argv = vec!["xr", "--output", "json"];
    argv.extend_from_slice(args);
    let (code, stdout, stderr) = run_at(&store, &argv).await;
    assert_eq!(code, 0, "args {args:?} failed; stderr: {stderr}");

    let v = parse_json(&stdout);
    assert_eq!(v["status"], "ok", "args {args:?}; got: {v}");
    for key in expected_keys {
        assert!(
            v.get(*key).is_some(),
            "args {args:?} must keep the {key} key; got: {v}"
        );
    }
}

/// The headless step-1 object gains `status` and keeps its documented keys.
#[tokio::test]
async fn test_oauth2_step1_envelope_carries_status_ok() {
    let tmp = TempDir::new().expect("tempdir");
    let store = seeded_store(&tmp);

    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "1",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    let v = parse_json(&stdout);
    assert_eq!(v["status"], "ok", "got: {v}");
    assert!(v["auth_url"].is_string(), "got: {v}");
    assert!(v["instructions"].is_string(), "got: {v}");
}

/// The two list-shaped verbs carry their entries under `apps`, inside the
/// same success envelope every other auth verb uses.
#[rstest::rstest]
#[case::status(&["auth", "status"])]
#[case::apps_list(&["auth", "apps", "list"])]
#[tokio::test]
async fn test_list_shaped_verbs_wrap_entries_under_apps(#[case] args: &[&str]) {
    let tmp = TempDir::new().expect("tempdir");
    let store = seeded_store(&tmp);

    let mut argv = vec!["xr", "--output", "json"];
    argv.extend_from_slice(args);
    let (code, stdout, stderr) = run_at(&store, &argv).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    let v = parse_json(&stdout);
    assert_eq!(
        v["status"], "ok",
        "args {args:?} lack the envelope; got: {stdout}"
    );
    assert!(
        v["apps"].is_array(),
        "args {args:?} must carry entries under `apps`; got: {stdout}"
    );
}

/// The status-ok contract holds across every structured format, not just
/// JSON: an agent picking yaml or jsonl reads the same field.
#[rstest::rstest]
#[case::jsonl("jsonl")]
#[case::yaml("yaml")]
#[tokio::test]
async fn test_status_ok_holds_across_structured_formats(#[case] format: &str) {
    let tmp = TempDir::new().expect("tempdir");
    let store = seeded_store(&tmp);

    let (code, stdout, stderr) = run_at(
        &store,
        &["xr", "--output", format, "auth", "default", "myapp"],
    )
    .await;
    assert_eq!(code, 0, "format {format} failed; stderr: {stderr}");
    assert!(
        stdout.contains("ok"),
        "the {format} rendering must carry the status; got: {stdout}"
    );
    assert!(
        stdout.contains("Default app set"),
        "and keep the message; got: {stdout}"
    );
}

/// Text output is untouched by the envelope change.
#[rstest::rstest]
#[case::apps_update(&["auth", "apps", "update", "myapp", "--client-id", "NEW"], "updated")]
#[case::default(&["auth", "default", "myapp"], "Default app set to")]
#[case::app_bearer(&["auth", "app", "--bearer-token", "TOK"], "App authentication successful")]
#[tokio::test]
async fn test_text_output_is_unchanged(#[case] args: &[&str], #[case] expected: &str) {
    let tmp = TempDir::new().expect("tempdir");
    let store = seeded_store(&tmp);

    let mut argv = vec!["xr"];
    argv.extend_from_slice(args);
    let (code, stdout, stderr) = run_at(&store, &argv).await;
    assert_eq!(code, 0, "args {args:?} failed; stderr: {stderr}");
    assert!(stdout.contains(expected), "args {args:?}; got: {stdout}");
    assert!(
        !stdout.contains("status"),
        "text mode carries no envelope key; got: {stdout}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// U5: the no-credentials error carries a next step, in both modes
// ═══════════════════════════════════════════════════════════════════════════

/// The JSON baseline. Every key the no-credentials envelope carries today
/// must survive byte-identical, and `next_step` must be the only addition.
#[tokio::test]
async fn test_no_credentials_envelope_keeps_every_existing_key() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let (code, _stdout, stderr) = run_at(&store, &["xr", "--output", "json", "whoami"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope");
    let obj = v.as_object().expect("an object");

    assert_eq!(obj["status"], "error", "got: {v}");
    assert_eq!(obj["reason"], "auth-required", "got: {v}");
    assert_eq!(obj["exit_code"], 77, "got: {v}");
    assert_eq!(
        obj["message"], "Auth Error: NoAuthMethod: no authentication method available",
        "the human message is unchanged; got: {v}"
    );

    let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec!["exit_code", "message", "next_step", "reason", "status"],
        "next_step is the only addition to the envelope; got: {v}"
    );
}

/// Text mode on an empty store points at registration.
#[tokio::test]
async fn test_hint_empty_store_names_registration() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let (code, _stdout, stderr) = run_at(&store, &["xr", "whoami"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    assert!(
        stderr.contains("Auth Error: NoAuthMethod"),
        "the error line is unchanged; got: {stderr}"
    );
    assert!(
        stderr.contains("xr auth apps add"),
        "and is followed by the registration line; got: {stderr}"
    );
}

/// One credentialed app with no tokens points at sign-in, not registration.
#[tokio::test]
async fn test_hint_credentialed_app_names_sign_in() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    populate_app_store(&store);

    let (code, _stdout, stderr) = run_at(&store, &["xr", "whoami"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    assert!(stderr.contains("xr auth oauth2"), "got: {stderr}");
    assert!(
        !stderr.contains("apps add"),
        "registration is the wrong advice here; got: {stderr}"
    );
}

/// A credentialed app that is not the target is named with `--app`.
#[tokio::test]
async fn test_hint_names_the_credentialed_alternative() {
    use xdk::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8"));
    ts.add_app("work", "WORK-CLIENT-ID", "WORK-SECRET")
        .expect("add work");
    ts.add_app("blank", "", "").expect("add blank");

    let (code, _stdout, stderr) = run_at(&store, &["xr", "--app", "blank", "whoami"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    assert!(
        stderr.contains("--app work"),
        "the hint names the app that can sign in; got: {stderr}"
    );
}

/// An app already holding tokens turns the hint into a rerun of this very
/// invocation with `--app` inserted.
#[tokio::test]
async fn test_hint_reruns_the_invocation_against_the_signed_in_app() {
    use xdk::store::TokenStore;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8"));
    ts.add_app("work", "WORK-CLIENT-ID", "WORK-SECRET")
        .expect("add work");
    ts.save_oauth2_token_for_app("work", "alice", "AT", "RT", 1_900_000_000)
        .expect("save_oauth2");
    ts.add_app("blank", "", "").expect("add blank");
    ts.set_default_app("blank").expect("set default");

    // A raw URL has no endpoint matrix entry, so this reaches the generic
    // no-credentials error rather than the wrong-app envelope.
    let (code, _stdout, stderr) =
        run_at(&store, &["xr", "--output", "json", "/2/some/unmapped/path"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope");
    assert_eq!(v["next_step"]["action"], "select-app", "got: {v}");
    let command = v["next_step"]["command"].as_str().expect("a command");
    assert!(command.contains("--app work"), "got: {command}");
    assert!(
        command.contains("/2/some/unmapped/path"),
        "the rerun keeps the original target; got: {command}"
    );
}

/// Exported client credentials mean sign-in, not registration, even with an
/// empty store: the snapshot carries what the environment supplied.
#[tokio::test]
async fn test_hint_env_client_id_names_sign_in() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let env = xdk::config::EnvOverrides {
        client_id: Some("ENV-CLIENT-ID".to_string()),
        client_secret: Some("ENV-CLIENT-SECRET".to_string()),
        ..xdk::config::EnvOverrides::default()
    };

    let (code, _stdout, stderr) = run_at_with(&store, &env, &["xr", "whoami"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    assert!(stderr.contains("xr auth oauth2"), "got: {stderr}");
    assert!(!stderr.contains("apps add"), "got: {stderr}");
}

/// A store the loader could not read sends the reader to the file, naming it.
#[tokio::test]
async fn test_hint_unreadable_store_names_the_path() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("store-as-directory");
    std::fs::create_dir(&store).expect("seed a directory");

    let (code, _stdout, stderr) = run_at(&store, &["xr", "whoami"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    assert!(
        stderr.contains(store.to_str().expect("utf-8 path")),
        "the hint names the store path; got: {stderr}"
    );
    assert!(
        !stderr.contains("apps add"),
        "a damaged store is not a missing registration; got: {stderr}"
    );
}

/// `--quiet` suppresses the advice and never the error.
#[tokio::test]
async fn test_hint_is_suppressed_under_quiet() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let (code, _stdout, stderr) = run_at(&store, &["xr", "whoami", "--quiet"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    assert!(stderr.contains("Auth Error: NoAuthMethod"), "got: {stderr}");
    assert!(
        !stderr.contains("apps add"),
        "quiet drops the hint lines; got: {stderr}"
    );
}

/// `NO_COLOR` keeps every line free of escape sequences.
#[test]
fn test_hint_carries_no_escape_sequences_under_no_color() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let out = common::xr_with_store(&store)
        .env("NO_COLOR", "1")
        .arg("whoami")
        .output()
        .expect("spawn xr");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("apps add"),
        "the hint prints; got: {stderr}"
    );
    assert!(
        !stderr.contains('\u{1b}'),
        "no line carries an escape sequence; got: {stderr:?}"
    );
}

/// The hint reaches every structured format, and none of them leak prose.
#[rstest::rstest]
#[case::json("json")]
#[case::jsonl("jsonl")]
#[case::yaml("yaml")]
#[tokio::test]
async fn test_hint_reaches_every_structured_format(#[case] format: &str) {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let (code, _stdout, stderr) = run_at(&store, &["xr", "--output", format, "whoami"]).await;
    assert_eq!(code, 77, "format {format}; stderr: {stderr}");
    assert!(
        stderr.contains("next_step") && stderr.contains("register-app"),
        "the {format} envelope carries the step; got: {stderr}"
    );
    assert!(
        !stderr.contains("Register an app first"),
        "prose stays out of the structured rendering; got: {stderr}"
    );
}

/// An unrelated error carries no hint at all.
#[tokio::test]
async fn test_other_errors_carry_no_hint() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let (code, _stdout, stderr) = run_at(&store, &["xr", "--output", "json", "example.com"]).await;
    assert_ne!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope");
    assert!(
        v.get("next_step").is_none(),
        "only the no-credentials error gets a hint; got: {v}"
    );
}

/// A grandfathered spaced app name is quoted wherever a hint prints it.
#[tokio::test]
async fn test_hint_quotes_a_spaced_app_name() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    std::fs::write(
        &store,
        "apps:\n  my app:\n    client_id: SPACED-ID\n    client_secret: SPACED-SECRET\n  blank:\n    client_id: ''\n    client_secret: ''\ndefault_app: blank\n",
    )
    .expect("seed a store holding a spaced name");

    let (code, _stdout, stderr) = run_at(&store, &["xr", "--output", "json", "whoami"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope");
    let command = v["next_step"]["command"].as_str().expect("a command");
    assert!(
        command.contains("'my app'"),
        "a name a shell would split is quoted; got: {command}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// U11: the post-sign-in 403 names the enrollment fix
// ═══════════════════════════════════════════════════════════════════════════

/// Runs a shortcut against a mock returning `body` with the given status.
async fn run_against_canned_response(status: u16, body: &str) -> (i32, String, String) {
    let ts = CliMockServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .respond_with(ResponseTemplate::new(status).set_body_string(body)),
    )
    .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    run_at_with(
        &store,
        &api_env_with_bearer(ts.uri(), "env-bearer-value"),
        &["xr", "--output", "json", "search", "hello"],
    )
    .await
}

/// The JSON baseline: a canned 403 keeps every key it carries today and
/// gains `next_step` as the only addition.
#[tokio::test]
async fn test_enrollment_403_envelope_keeps_every_existing_key() {
    let body = r#"{"title":"Unsupported Authentication","detail":"Your client is not enrolled: client-not-enrolled","status":403}"#;
    let (code, _stdout, stderr) = run_against_canned_response(403, body).await;
    assert_ne!(code, 0, "stderr: {stderr}");

    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope");
    let obj = v.as_object().expect("an object");
    assert_eq!(obj["status"], "error", "got: {v}");
    assert!(
        obj["message"]
            .as_str()
            .unwrap()
            .contains("client-not-enrolled")
    );

    let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec!["exit_code", "message", "next_step", "reason", "status"],
        "next_step is the only addition; got: {v}"
    );
    assert_eq!(obj["next_step"]["action"], "enroll-app", "got: {v}");
    assert!(obj["next_step"]["docs"].is_string(), "got: {v}");
    assert!(
        obj["next_step"]["command"].is_null(),
        "no command; got: {v}"
    );
    assert!(
        obj["next_step"]["template"].is_null(),
        "no template; got: {v}"
    );
}

/// Text mode quotes the body's detail line and points at the recipe.
#[tokio::test]
async fn test_enrollment_403_text_quotes_the_detail() {
    let ts = CliMockServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .respond_with(ResponseTemplate::new(403).set_body_string(
                r#"{"detail":"Your client is not enrolled: client-not-enrolled","status":403}"#,
            )),
    )
    .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let (code, _stdout, stderr) = run_at_with(
        &store,
        &api_env_with_bearer(ts.uri(), "env-bearer-value"),
        &["xr", "search", "hello"],
    )
    .await;
    assert_ne!(code, 0, "stderr: {stderr}");
    assert!(
        stderr.contains("Your client is not enrolled"),
        "the detail line is quoted; got: {stderr}"
    );
    assert!(
        stderr.contains("Troubleshooting"),
        "and the recipe is named; got: {stderr}"
    );
}

/// The other marker fires too, and a body with no detail simply omits the
/// quote.
#[tokio::test]
async fn test_enrollment_403_without_detail_omits_the_quote() {
    let ts = CliMockServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .respond_with(
                ResponseTemplate::new(403).set_body_string(r#"{"title":"client-forbidden"}"#),
            ),
    )
    .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let (code, _stdout, stderr) = run_at_with(
        &store,
        &api_env_with_bearer(ts.uri(), "env-bearer-value"),
        &["xr", "search", "hello"],
    )
    .await;
    assert_ne!(code, 0, "stderr: {stderr}");
    assert!(stderr.contains("Troubleshooting"), "got: {stderr}");
}

/// A 403 that is not an enrollment refusal carries no hint.
#[tokio::test]
async fn test_unrelated_403_carries_no_hint() {
    let body = r#"{"title":"Forbidden","detail":"You cannot like your own post","status":403}"#;
    let (code, _stdout, stderr) = run_against_canned_response(403, body).await;
    assert_ne!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope");
    assert!(
        v.get("next_step").is_none(),
        "only an enrollment refusal gets the hint; got: {v}"
    );
}

/// `--quiet` keeps the error line and drops the advice.
#[tokio::test]
async fn test_enrollment_403_hint_is_suppressed_under_quiet() {
    let ts = CliMockServer::new().await;
    ts.mount(
        Mock::given(method("GET"))
            .and(path("/2/tweets/search/recent"))
            .respond_with(ResponseTemplate::new(403).set_body_string(
                r#"{"detail":"Your client is not enrolled: client-not-enrolled","status":403}"#,
            )),
    )
    .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let (code, _stdout, stderr) = run_at_with(
        &store,
        &api_env_with_bearer(ts.uri(), "env-bearer-value"),
        &["xr", "search", "hello", "--quiet"],
    )
    .await;
    assert_ne!(code, 0, "stderr: {stderr}");
    assert!(
        !stderr.contains("Troubleshooting"),
        "quiet drops the advice; got: {stderr}"
    );
}
