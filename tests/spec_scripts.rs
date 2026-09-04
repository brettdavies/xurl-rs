//! Guards for the spec-drift shell tooling: the auth-method section of the
//! drift report (`scripts/diff-x-openapi-spec.sh`).
#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run a repo script with `args` and return its stdout. A non-zero exit
/// panics so a script error never reads as a passing assertion on empty
/// output.
fn run_script(name: &str, args: &[&Path]) -> String {
    let script = repo_root().join("scripts").join(name);
    assert!(script.exists(), "scripts/{name} is missing");
    let output = Command::new("bash")
        .arg(&script)
        .args(args)
        .current_dir(repo_root())
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn scripts/{name}: {e}"));
    assert!(
        output.status.success(),
        "scripts/{name} failed ({:?}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("script output is UTF-8")
}

/// Minimal spec holding one operation per `(method, path, security)`.
fn spec(ops: &[(&str, &str, Value)]) -> Value {
    let mut paths = serde_json::Map::new();
    for (method, path, security) in ops {
        let item = paths
            .entry((*path).to_string())
            .or_insert_with(|| json!({}));
        item[*method] = json!({ "security": security });
    }
    json!({
        "info": { "version": "2.168" },
        "paths": paths,
        "components": { "schemas": {} }
    })
}

/// Write both snapshots to a temp dir and run the drift report over them.
fn drift_report(local: &Value, upstream: &Value) -> String {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let local_path = dir.path().join("local.json");
    let upstream_path = dir.path().join("upstream.json");
    fs::write(&local_path, local.to_string()).expect("write local");
    fs::write(&upstream_path, upstream.to_string()).expect("write upstream");
    run_script("diff-x-openapi-spec.sh", &[&local_path, &upstream_path])
}

const NOTHING_STRUCTURAL: &str =
    "Neither paths, schemas, categorical values, nor auth methods changed";

#[test]
fn drift_report_lists_auth_method_changes_outside_the_allowlist() {
    let local = spec(&[("get", "/2/webhooks", json!([{ "BearerToken": [] }]))]);
    let upstream = spec(&[(
        "get",
        "/2/webhooks",
        json!([{ "BearerToken": [] }, { "OAuth2UserToken": [] }, { "UserToken": [] }]),
    )]);
    let report = drift_report(&local, &upstream);
    assert!(
        report.contains("**Auth-method changes:** 1 operation(s)"),
        "{report}"
    );
    assert!(
        report.contains("- `GET /2/webhooks`: +`OAuth2UserToken`, +`UserToken`"),
        "{report}"
    );
    assert!(
        report.contains("the generated auth matrix is unchanged"),
        "{report}"
    );
    assert!(!report.contains(NOTHING_STRUCTURAL), "{report}");
}

#[test]
fn drift_report_flags_auth_method_changes_on_allowlisted_operations() {
    // POST /2/tweets is in build.rs's SHORTCUT_TEMPLATES, so an auth change
    // there moves the generated matrix and the report must say so.
    let local = spec(&[(
        "post",
        "/2/tweets",
        json!([{ "OAuth2UserToken": ["tweet.read", "tweet.write", "users.read"] }, { "UserToken": [] }]),
    )]);
    let upstream = spec(&[(
        "post",
        "/2/tweets",
        json!([{ "OAuth2UserToken": ["tweet.read", "tweet.write", "users.read"] }]),
    )]);
    let report = drift_report(&local, &upstream);
    assert!(
        report.contains("- `POST /2/tweets`: -`UserToken`"),
        "{report}"
    );
    assert!(
        report.contains(
            "**1 of these are in the shortcut allowlist and change the generated auth matrix:** `POST /2/tweets`."
        ),
        "{report}"
    );
}

#[test]
fn drift_report_ignores_security_ordering() {
    // Upstream serializes scope arrays and requirement lists in
    // nondeterministic order; the same set in a different order is not an
    // auth change.
    let local = spec(&[(
        "post",
        "/2/tweets",
        json!([{ "OAuth2UserToken": ["tweet.read", "tweet.write", "users.read"] }, { "UserToken": [] }]),
    )]);
    let upstream = spec(&[(
        "post",
        "/2/tweets",
        json!([{ "UserToken": [] }, { "OAuth2UserToken": ["users.read", "tweet.write", "tweet.read"] }]),
    )]);
    let report = drift_report(&local, &upstream);
    assert!(!report.contains("Auth-method changes"), "{report}");
    assert!(report.contains(NOTHING_STRUCTURAL), "{report}");
}

#[test]
fn drift_report_shows_scope_changes_on_a_requirement() {
    let local = spec(&[(
        "get",
        "/2/users/me",
        json!([{ "OAuth2UserToken": ["users.read"] }]),
    )]);
    let upstream = spec(&[(
        "get",
        "/2/users/me",
        json!([{ "OAuth2UserToken": ["tweet.read", "users.read"] }]),
    )]);
    let report = drift_report(&local, &upstream);
    assert!(
        report.contains(
            "- `GET /2/users/me`: +`OAuth2UserToken[tweet.read users.read]`, -`OAuth2UserToken[users.read]`"
        ),
        "{report}"
    );
    assert!(
        report.contains("1 of these are in the shortcut allowlist"),
        "{report}"
    );
}
