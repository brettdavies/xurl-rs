//! CLI integration tests using the library entrypoint `xurl::cli::run_with_store_path`.
//!
//! Parallel-safe: every test creates its own `TempDir` for token-store isolation
//! and passes the path explicitly. No `HOME` / `XDG_CONFIG_HOME` mutation, no
//! `#[serial]` annotations, no subprocess overhead. The binary's exit-code
//! contract is pinned separately by `tests/binary_contract_tests.rs`.

use std::path::Path;

use tempfile::TempDir;

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
fn test_apps_add_with_redirect_uri_persists() {
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
fn test_apps_add_without_redirect_uri_leaves_empty() {
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
fn test_redirect_uri_get_uses_placeholder_default_when_store_empty() {
    // Fresh tempdir → TokenStore seeds the placeholder "default" app.
    // The omit-NAME `get` resolves through it and surfaces the built-in
    // default URI (no stored value on the placeholder).
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

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
fn test_redirect_uri_get_json_output_app_config_source() {
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
