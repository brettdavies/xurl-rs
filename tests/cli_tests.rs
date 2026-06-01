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
