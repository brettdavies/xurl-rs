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
        preview.contains("github.com/brettdavies/xurl-rs.git"),
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
        line.contains("xurl-rs.git"),
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
