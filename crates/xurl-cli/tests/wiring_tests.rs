//! Integration tests for --output, --quiet, --no-interactive, and exit code wiring.
//!
//! These tests verify that the flags actually change behavior (not just parse).

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Registers one app at `store` so the per-app renderers have an entry to
/// emit; an empty store is empty and renders an empty array.
fn register_app_at(store: &std::path::Path) {
    let mut ts = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "MYAPP-CLIENT-ID", "MYAPP-SECRET")
        .expect("add_app");
}

// ═══════════════════════════════════════════════════════════════════════════
// --output json wiring
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_version_honors_the_json_flag_with_one_object() {
    // version is a Tier 1 meta-command, so the question here is only whether
    // the flag reaches it: an object where the plain form is one line. The
    // fields it carries are pinned in binary_contract_tests.rs.
    let structured = common::xr()
        .args(["version", "--output", "json"])
        .output()
        .unwrap();
    let plain = common::xr().arg("version").output().unwrap();

    assert!(structured.status.success());
    let value: serde_json::Value = serde_json::from_slice(&structured.stdout)
        .expect("version --output json prints one JSON object");
    assert!(value.is_object(), "the flag reached the command: {value}");
    assert_ne!(
        structured.stdout, plain.stdout,
        "--output json changed what version printed"
    );
}

#[test]
fn test_json_output_no_ansi_codes() {
    // JSON output should never contain ANSI escape sequences
    let output = common::xr()
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
    // When a command fails with --output json, stderr should be the canonical
    // agent-native envelope: {"status":"error","reason":<kebab>,
    // "exit_code":<int>,"message":<str>}.
    let tmp = TempDir::new().unwrap();

    let output = common::xr_with_store(&tmp.path().join(".xurl"))
        .args(["whoami", "--output", "json"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    let parsed: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
    assert_eq!(parsed["status"], "error");
    assert!(
        parsed["reason"].is_string(),
        "reason field should be a string"
    );
    assert!(
        parsed["exit_code"].is_number(),
        "exit_code field should be a number"
    );
    assert!(
        parsed["message"].is_string(),
        "message field should be a string"
    );
}

#[test]
fn test_text_output_has_color_by_default() {
    // Default text output for version should contain the plain version string
    let output = common::xr().args(["version"]).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("xr"));
}

// ═══════════════════════════════════════════════════════════════════════════
// NO_COLOR environment variable
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_no_color_env_strips_ansi() {
    // NO_COLOR=1 should strip ANSI codes even in text mode
    let output = common::xr()
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
    // XURL_OUTPUT=json should make auth status output JSON.
    // The status renderer emits a JSON array of per-app entries
    // (`print_response`), one per registered app.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    register_app_at(&store);

    let output = common::xr_with_store(&store)
        .args(["auth", "status"])
        .env("XURL_OUTPUT", "json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let arr = parsed["apps"]
        .as_array()
        .expect("auth status emits apps as a JSON array");
    assert!(!arr.is_empty(), "expected at least one entry: {stdout}");
    assert!(arr[0]["name"].is_string());
}

#[test]
fn test_explicit_output_overrides_env() {
    // --output text should override XURL_OUTPUT=json
    let output = common::xr()
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
    let output = common::xr().args(["version", "--quiet"]).output().unwrap();

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
    let output = common::xr().args(["version", "--quiet"]).output().unwrap();

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

    let output = common::xr_with_store(&tmp.path().join(".xurl"))
        .args(["auth", "default", "--no-interactive"])
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
    let output = common::xr().args(["version"]).output().unwrap();

    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn test_exit_code_auth_required_is_77() {
    // Auth-required failures exit with sysexits EX_NOPERM (77), disambiguating
    // from clap usage errors (EX_USAGE = 2). Behavior change in v1.3.0.
    let tmp = TempDir::new().unwrap();

    let output = common::xr_with_store(&tmp.path().join(".xurl"))
        .args(["whoami"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code().unwrap(),
        77,
        "Auth error should exit with code 77 (EX_NOPERM)"
    );
}

#[test]
fn test_exit_code_json_error_includes_code() {
    // With --output json, the envelope's exit_code field carries the semantic
    // exit code (77 for auth-required).
    let tmp = TempDir::new().unwrap();

    let output = common::xr_with_store(&tmp.path().join(".xurl"))
        .args(["whoami", "--output", "json"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
    assert_eq!(parsed["status"], "error");
    assert_eq!(parsed["exit_code"].as_i64().unwrap(), 77);
    assert_eq!(parsed["reason"], "auth-required");
}

// ═══════════════════════════════════════════════════════════════════════════
// Combined flags
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_json_quiet_combined() {
    // --output json --quiet should still produce the JSON-array shape from
    // `auth status` because `print_response` is independent of `--quiet`.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    register_app_at(&store);

    let output = common::xr_with_store(&store)
        .args(["auth", "status", "--output", "json", "--quiet"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let arr = parsed["apps"]
        .as_array()
        .expect("auth status emits apps as a JSON array");
    assert!(!arr.is_empty(), "expected at least one entry: {stdout}");
    assert!(arr[0]["name"].is_string());
}

#[test]
fn test_all_agentic_flags_wired_correctly() {
    // All flags together should work and produce a parseable JSON document.
    let tmp = TempDir::new().unwrap();

    let output = common::xr_with_store(&tmp.path().join(".xurl"))
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
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(serde_json::from_str::<serde_json::Value>(stdout.trim()).is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════
// Auth subcommand output format wiring
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_auth_status_json_output() {
    // `xurl auth status --output json` emits a JSON array of per-app
    // entries, one per registered app.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    register_app_at(&store);

    let output = common::xr_with_store(&store)
        .args(["auth", "status", "--output", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let arr = parsed["apps"]
        .as_array()
        .expect("status emits apps as a JSON array");
    assert!(
        arr.iter().any(|e| e["name"] == "myapp"),
        "expected the registered app: {stdout}"
    );
}

#[test]
fn test_auth_apps_list_json_output() {
    // `xurl auth apps list --output json` emits a JSON array of per-app
    // entries, one per registered app.
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    register_app_at(&store);

    let output = common::xr_with_store(&store)
        .args(["auth", "apps", "list", "--output", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let arr = parsed["apps"]
        .as_array()
        .expect("apps list emits apps as a JSON array");
    assert!(
        arr.iter().any(|e| e["name"] == "myapp"),
        "expected the registered app: {stdout}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// XURL_TOKEN_STORE environment variable
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_xurl_token_store_env_selects_store_file() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join("store.yaml");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_xr"))
        .args([
            "auth",
            "apps",
            "add",
            "probe",
            "--client-id",
            "id",
            "--client-secret",
            "secret",
        ])
        .env("XURL_TOKEN_STORE", &store)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let saved = std::fs::read_to_string(&store).unwrap();
    assert!(
        saved.contains("probe"),
        "the store named by XURL_TOKEN_STORE must hold the app: {saved}"
    );
}

#[test]
fn test_xr_seam_default_store_rejects_writes() {
    let output = common::xr()
        .args([
            "auth",
            "apps",
            "add",
            "probe",
            "--client-id",
            "id",
            "--client-secret",
            "secret",
        ])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "a spawn that never asked for a store must not be able to write one"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No such file"), "stderr: {stderr}");
}

// ═══════════════════════════════════════════════════════════════════════════
// Auth status and apps list carry their array under `apps` (R17b)
// ═══════════════════════════════════════════════════════════════════════════

/// Both list-shaped auth verbs answer with the same success envelope every
/// other auth verb uses, so one `.status` branch covers the whole surface.
#[test]
fn test_auth_status_and_apps_list_wrap_entries_under_apps() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    register_app_at(&store);

    for args in [
        &["auth", "status", "--output", "json"][..],
        &["auth", "apps", "list", "--output", "json"][..],
    ] {
        let output = common::xr_with_store(&store).args(args).output().unwrap();
        assert!(output.status.success(), "{args:?} failed");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();

        assert_eq!(parsed["status"], "ok", "{args:?} lacks status: {stdout}");
        let apps = parsed["apps"]
            .as_array()
            .unwrap_or_else(|| panic!("{args:?} has no apps array: {stdout}"));
        assert!(
            apps.iter().any(|e| e["name"] == "myapp"),
            "{args:?} lost the registered app: {stdout}"
        );
    }
}

/// An empty store answers with the same envelope and an empty array, so a
/// caller iterates `apps` without a zero-app special case.
#[test]
fn test_auth_status_empty_store_emits_empty_apps_array() {
    let tmp = TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");

    let output = common::xr_with_store(&store)
        .args(["auth", "status", "--output", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(parsed["status"], "ok", "empty store lacks status: {stdout}");
    assert_eq!(
        parsed["apps"].as_array().map(Vec::len),
        Some(0),
        "empty store should emit apps: []: {stdout}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Resolve-then-call verbs
// ═══════════════════════════════════════════════════════════════════════════

/// The response fixture the library's spec tests use for `key`.
fn fixture(key: &str) -> serde_json::Value {
    let all: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            common::workspace_root()
                .join("crates/xdk/tests/fixtures/openapi/example_responses.json"),
        )
        .expect("fixture file"),
    )
    .expect("fixture JSON");
    all[key].clone()
}

#[tokio::test]
async fn follow_resolves_the_caller_and_the_handle_before_calling_the_shortcut() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "111", "name": "Me", "username": "me"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/users/by/username/helper"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "222", "name": "Helper", "username": "helper"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/users/111/following"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("action_following")))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = common::oauth1_store(tmp.path());

    let (code, stdout, stderr) = common::run_in_process(
        &store,
        &server.uri(),
        &["follow", "@helper", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["following"], true);
    let sequence: Vec<String> = server
        .received_requests()
        .await
        .expect("recorded")
        .iter()
        .map(|req| format!("{} {}", req.method, req.url.path()))
        .collect();
    assert_eq!(
        sequence,
        [
            "GET /2/users/me",
            "GET /2/users/by/username/helper",
            "POST /2/users/111/following",
        ],
        "the caller's id and the handle resolve before the one shortcut call"
    );
}

#[tokio::test]
async fn dm_prints_the_send_confirmation_the_spec_documents() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by/username/helper"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "222", "name": "Helper", "username": "helper"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/dm_conversations/with/222/messages"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"dm_conversation_id": "222-111", "dm_event_id": "1580705921830768647"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = common::oauth1_store(tmp.path());

    let (code, stdout, stderr) = common::run_in_process(
        &store,
        &server.uri(),
        &["dm", "@helper", "hello there", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["dm_event_id"], "1580705921830768647");
    assert_eq!(body["data"]["dm_conversation_id"], "222-111");
}
