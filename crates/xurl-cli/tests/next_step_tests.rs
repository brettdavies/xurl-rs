//! The steps two errors carry: a usage error points at the help of the
//! command it belongs to, and a store that could not be loaded points at the
//! page that says how to recover it.
//!
//! Parallel-safe by construction, like `tests/cli_tests.rs`: every case runs
//! the library entrypoint against its own `TempDir`-rooted store and supplies
//! its environment as `EnvOverrides`, so nothing here reads or writes the
//! process environment.

mod common;

use std::path::Path;

use tempfile::TempDir;
use xdk::config::EnvOverrides;
use xdk::store::TokenStore;
use xurl::cli;

/// The page the `inspect-store` step names.
const STORE_DOCS: &str = "https://github.com/brettdavies/xurl-rs/blob/main/crates/xurl-cli/README.md#the-token-store-could-not-be-read";

async fn run_at(store: &Path, args: &[&str]) -> (i32, String, String) {
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code = cli::runner::run_with_overrides(
        args,
        &mut stdout,
        &mut stderr,
        store,
        &EnvOverrides::default(),
    )
    .await;
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

/// Runs against a store path nothing has written.
async fn run_isolated(args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    run_at(&tmp.path().join(".xurl"), args).await
}

fn envelope(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim())
        .unwrap_or_else(|e| panic!("stderr is not a JSON envelope ({e}): {stderr}"))
}

/// A store file that exists and is not a store.
fn unloadable_store(tmp: &TempDir) -> std::path::PathBuf {
    let store = tmp.path().join(".xurl");
    std::fs::write(&store, ":::: not yaml [\n").expect("write the broken store");
    store
}

// ── invalid-args: show-help ────────────────────────────────────────────

/// Whoever raised the usage error (clap, the classifier, or a secret flag's
/// own check), its envelope carries the help invocation its message names.
#[rstest::rstest]
#[case::root_flag(&["xr", "--output", "json", "--frobnicate"], "xr --help")]
#[case::command_argument(&["xr", "--output", "json", "whoami", "extra"], "xr whoami --help")]
#[case::family_flag(&["xr", "--output", "json", "auth", "--frob"], "xr auth --help")]
#[case::invalid_value(&["xr", "--output", "json", "skill", "install", "bogus_host"], "xr skill install --help")]
#[case::unsupported_format(&["xr", "--output", "toml", "whoami"], "xr whoami --help")]
#[case::no_command(&["xr", "--output", "json"], "xr --help")]
#[case::two_secrets_on_stdin(
    &["xr", "--output", "json", "auth", "oauth1", "--consumer-key", "ck", "--consumer-secret-file", "-",
      "--access-token-file", "-", "--token-secret-file", "-"],
    "xr auth oauth1 --help"
)]
#[tokio::test]
async fn a_usage_error_carries_the_help_to_read(#[case] args: &[&str], #[case] help: &str) {
    let (code, _stdout, stderr) = run_isolated(args).await;
    assert_eq!(code, 2, "args {args:?}; stderr: {stderr}");
    let v = envelope(&stderr);
    assert_eq!(v["reason"], "invalid-args", "args {args:?}: {v}");
    assert_eq!(
        v["next_step"],
        serde_json::json!({"action": "show-help", "command": help}),
        "args {args:?}: {v}"
    );
    let message = v["message"].as_str().expect("the message is a string");
    assert!(
        message.ends_with(&format!("Try '{help}'.")),
        "the step and the message name one command: {v}"
    );
}

// ── token-store: inspect-store ─────────────────────────────────────────

/// A store that exists and could not be loaded is the case `inspect-store`
/// names. The step carries the page and no command: the one command that
/// reports on the store is the one that just failed.
#[rstest::rstest]
#[case::status(&["xr", "--output", "json", "auth", "status"])]
#[case::apps_list(&["xr", "--output", "json", "auth", "apps", "list"])]
#[case::apps_add(&["xr", "--output", "json", "auth", "apps", "add", "work", "--client-id", "abcdefgh", "--client-secret-file"])]
#[tokio::test]
async fn a_store_that_could_not_be_loaded_carries_inspect_store(#[case] args: &[&str]) {
    let tmp = TempDir::new().expect("tempdir");
    let store = unloadable_store(&tmp);
    // The one case that ends on a flag takes its secret from a file.
    let secret = tmp.path().join("client-secret");
    std::fs::write(&secret, "shh\n").expect("write the secret");
    let mut args = args.to_vec();
    if args.last() == Some(&"--client-secret-file") {
        args.push(secret.to_str().expect("utf-8 path"));
    }
    let (code, _stdout, stderr) = run_at(&store, &args).await;
    assert_eq!(code, 77, "args {args:?}; stderr: {stderr}");
    let v = envelope(&stderr);
    assert_eq!(v["reason"], "token-store", "args {args:?}: {v}");
    assert_eq!(
        v["next_step"],
        serde_json::json!({"action": "inspect-store", "docs": STORE_DOCS}),
        "args {args:?}: {v}"
    );
    assert!(
        v["message"]
            .as_str()
            .expect("the message is a string")
            .contains(store.to_str().expect("utf-8 path")),
        "the message names the file: {v}"
    );
}

/// `token-store` is also the reason for a name the store does not hold. The
/// file loaded, so there is nothing to inspect and no step.
#[tokio::test]
async fn a_name_the_store_does_not_hold_carries_no_step() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("work", "abcdefgh", "shh").expect("add app");

    let (code, _stdout, stderr) = run_at(
        &store,
        &["xr", "--output", "json", "auth", "default", "nope"],
    )
    .await;
    assert_eq!(code, 77, "stderr: {stderr}");
    let v = envelope(&stderr);
    assert_eq!(v["reason"], "token-store", "{v}");
    assert!(v.get("next_step").is_none(), "{v}");
}

/// A command that needs a credential from the unloadable store keeps the step
/// it had, a runnable `xr auth status`, and gains the page.
#[tokio::test]
async fn a_credential_lookup_on_an_unloadable_store_keeps_its_command() {
    let tmp = TempDir::new().expect("tempdir");
    let store = unloadable_store(&tmp);
    let (code, _stdout, stderr) = run_at(&store, &["xr", "--output", "json", "whoami"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    let v = envelope(&stderr);
    assert_eq!(v["reason"], "auth-required", "{v}");
    assert_eq!(
        v["next_step"],
        serde_json::json!({"action": "inspect-store", "command": "xr auth status", "docs": STORE_DOCS}),
        "{v}"
    );
}

/// Text mode says where to read, after the error line.
#[tokio::test]
async fn text_mode_names_the_page_for_an_unloadable_store() {
    let tmp = TempDir::new().expect("tempdir");
    let store = unloadable_store(&tmp);
    let (code, _stdout, stderr) = run_at(&store, &["xr", "auth", "status"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    assert!(stderr.contains(STORE_DOCS), "stderr: {stderr}");
}

// ── The page exists ────────────────────────────────────────────────────

/// The step's page is a section of the CLI README, so the heading it anchors
/// to has to be there.
#[test]
fn the_page_inspect_store_names_is_in_the_readme() {
    let (file, anchor) = STORE_DOCS
        .strip_prefix("https://github.com/brettdavies/xurl-rs/blob/main/")
        .and_then(|rest| rest.split_once('#'))
        .expect("a README path and an anchor");
    let readme = std::fs::read_to_string(common::workspace_root().join(file)).expect("the README");
    let found = readme
        .lines()
        .filter_map(|line| line.strip_prefix("### "))
        .any(|heading| heading.to_lowercase().replace(' ', "-") == anchor);
    assert!(found, "{file} has no heading that anchors to #{anchor}");
}
