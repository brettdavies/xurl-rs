/// Library-level tests for the `xurl::cli::run` / `run_with_store_path`
/// entrypoints (U4).
///
/// Parallel-safe: each test creates its own `TempDir` and passes the path to
/// `run_with_store_path`. No `HOME` / `XDG_CONFIG_HOME` mutation, no
/// `#[serial]`, no subprocess overhead. Verifies clap exit-code mapping,
/// Tier-1 dispatch (Completions / Version / Schema), stdout/stderr split,
/// and the F8 regression guard (schema body not double-wrapped in
/// `{"message": ...}` under `--output json`).
use std::io::Write;

use tempfile::TempDir;

use xurl::cli;

/// Helper: run with a fresh tempdir store path; return (exit_code, stdout, stderr).
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

#[test]
fn help_exits_zero_and_writes_banner_to_stdout() {
    let (code, stdout, stderr) = run_isolated(&["xr", "--help"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    // clap renders the `long_about` first under `--help`; the short `about`
    // string ("Auth enabled curl-like interface...") only appears under
    // `-h`. Assert against text present in both contexts.
    assert!(
        stdout.contains("A command-line tool for making authenticated requests to the X API"),
        "stdout missing banner: {stdout}"
    );
    assert!(stderr.is_empty(), "stderr should be empty, got: {stderr}");
}

#[test]
fn short_help_writes_about_string_to_stdout() {
    let (code, stdout, stderr) = run_isolated(&["xr", "-h"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("Auth enabled curl-like interface"),
        "stdout missing short about string: {stdout}"
    );
}

#[test]
fn version_subcommand_exits_zero_and_starts_with_xr() {
    let (code, stdout, stderr) = run_isolated(&["xr", "version"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.starts_with("xr "),
        "stdout should start with 'xr ': {stdout}"
    );
}

#[test]
fn bad_flag_exits_two_and_writes_to_stderr() {
    let (code, stdout, stderr) = run_isolated(&["xr", "--bogus"]);
    assert_eq!(code, 2);
    assert!(!stderr.is_empty(), "stderr should be non-empty");
    assert!(stdout.is_empty(), "stdout should be empty, got: {stdout}");
}

#[test]
fn missing_url_exits_one_and_writes_no_url_to_stderr() {
    let (code, _stdout, stderr) = run_isolated(&["xr"]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("No URL provided"),
        "stderr missing expected message: {stderr}"
    );
}

#[test]
fn completions_bash_writes_script_to_stdout() {
    let (code, stdout, stderr) = run_isolated(&["xr", "completions", "bash"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    // clap_complete's bash output defines a function named `_xr` (the binary name).
    assert!(
        stdout.contains("_xr()") || stdout.contains("_xr "),
        "expected bash completion script for `_xr`, got: {stdout}"
    );
}

#[test]
fn schema_list_mentions_known_commands() {
    let (code, stdout, stderr) = run_isolated(&["xr", "schema", "--list"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    for cmd in ["post", "whoami", "like"] {
        assert!(
            stdout.contains(cmd),
            "schema --list missing '{cmd}': {stdout}"
        );
    }
}

#[test]
fn schema_post_emits_json_schema_directly_not_wrapped_in_message() {
    // F8 regression guard: under `--output json`, the schema body must be the
    // raw JSON schema, NOT `{"message": "<stringified schema>"}`.
    let (code, stdout, stderr) = run_isolated(&["xr", "--output", "json", "schema", "post"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("schema stdout must parse as JSON");
    // A real schema has structural keys like `$schema`, `title`, `properties`,
    // or `type`. The buggy F8 path produced `{"message": "..."}` — assert
    // explicitly that the wrapper key is absent.
    assert!(
        parsed.get("message").is_none(),
        "schema body must not be wrapped in {{'message': ...}}: {stdout}"
    );
    let is_schema_shape = parsed.get("$schema").is_some()
        || parsed.get("title").is_some()
        || parsed.get("properties").is_some()
        || parsed.get("type").is_some()
        || parsed.get("$defs").is_some()
        || parsed.get("definitions").is_some();
    assert!(is_schema_shape, "expected JSON Schema shape, got: {stdout}");
}

#[test]
fn schema_unknown_command_exits_general_error() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "schema", "nope-not-a-command"]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("unknown command"),
        "stderr should explain unknown command: {stderr}"
    );
}

/// Compile-time assertion that the library entrypoint surface is `Send + Sync`
/// where it should be. `OutputConfig` is the most-important type for the
/// future async/concurrent `ApiClient` (see project_async_requirement).
#[test]
fn entrypoint_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<xurl::output::OutputConfig>();
    assert_send_sync::<xurl::config::Config>();
    // Auth is Send + Sync per src/auth/mod.rs compile-time check.
    assert_send_sync::<xurl::auth::Auth>();
    // The runner is a free function — function pointers are always Send + Sync.
    assert_send_sync::<fn() -> i32>();
}

/// Sanity check: `Vec<u8>` writers compile against the `&mut dyn Write`
/// entrypoint signature (this is what library tests rely on).
#[test]
fn runner_accepts_vec_u8_writers() {
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let _: i32 = cli::run_with_store_path(["xr", "version"], &mut stdout, &mut stderr, &store);
    // Sanity check that writers behave like Write traits.
    let _ = stdout.flush();
    let _ = stderr.flush();
}
