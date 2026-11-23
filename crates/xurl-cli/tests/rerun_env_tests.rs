//! A step names, as flags, what the environment chose for the run it answers.
//!
//! `clap` reads `XURL_*` from the real process environment, so each case
//! spawns `xr` with the variable set and none of its flag. A step a caller
//! follows in a shell without the variable must run the same.

mod common;

use serde_json::Value;
use tempfile::TempDir;

use common::xr_with_store;

/// Two apps with client credentials and no login, `work` the default.
const STORE: &str = "apps:\n  work:\n    client_id: 'client-work'\n    client_secret: 'secret'\n    \
                     oauth2_tokens: {}\n  side:\n    client_id: 'client-side'\n    \
                     client_secret: 'secret'\n    oauth2_tokens: {}\ndefault_app: work\n";

/// The failure `xr` writes on stderr, run under `env` with `args`.
fn refused(env: &[(&str, &str)], args: &[&str]) -> Value {
    refused_over(STORE, env, args)
}

/// [`refused`] over a store file holding `store`.
fn refused_over(contents: &str, env: &[(&str, &str)], args: &[&str]) -> Value {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    std::fs::write(&store, contents).expect("write the store");
    let mut cmd = xr_with_store(&store);
    cmd.env("API_BASE_URL", common::CLOSED);
    for (key, value) in env {
        cmd.env(key, value);
    }
    let output = cmd.args(args).output().expect("xr runs");
    serde_json::from_slice(&output.stderr)
        .unwrap_or_else(|e| panic!("stderr is not a document ({e}): {output:?}"))
}

fn step_text(document: &Value) -> String {
    let step = &document["next_step"];
    step["command"]
        .as_str()
        .or(step["template"].as_str())
        .unwrap_or_default()
        .to_string()
}

#[test]
fn a_step_names_the_output_format_the_environment_selected() {
    let mut faults = Vec::new();
    for (what, args) in [
        ("a sign-in", vec!["whoami"]),
        (
            "a step that lists the apps",
            vec!["--app", "ghost", "whoami"],
        ),
    ] {
        let document = refused(&[("XURL_OUTPUT", "json")], &args);
        let text = step_text(&document);
        if !text.contains("--output json") {
            faults.push(format!("{what}: `{text}` drops the format: {document}"));
        }
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}

#[test]
fn a_sign_in_step_names_the_app_the_run_used_when_the_environment_names_another() {
    let document = refused(
        &[("XURL_APP", "side"), ("XURL_OUTPUT", "json")],
        &["--app", "work", "whoami"],
    );
    let text = step_text(&document);
    assert!(
        text.contains("--app work"),
        "`{text}` leaves the app to the shell, where XURL_APP says side: {document}"
    );
}

#[test]
fn a_registration_step_names_the_output_format_the_environment_selected() {
    let document = refused_over(
        "apps: {}\ndefault_app: ''\n",
        &[("XURL_OUTPUT", "json")],
        &["whoami"],
    );
    assert_eq!(
        document["next_step"]["action"], "register-app",
        "{document}"
    );
    let text = step_text(&document);
    assert!(text.contains("xr --output json auth apps add"), "{text}");
}

#[test]
fn a_step_that_inspects_an_unreadable_store_names_the_output_format() {
    let document = refused_over(
        "apps: [not, a, map\n",
        &[("XURL_OUTPUT", "json")],
        &["whoami"],
    );
    assert_eq!(
        document["next_step"]["action"], "inspect-store",
        "{document}"
    );
    let text = step_text(&document);
    assert!(text.contains("xr --output json auth status"), "{text}");
}

#[test]
fn a_step_that_runs_the_invocation_again_names_the_app_the_environment_selected() {
    let document = refused(
        &[("XURL_APP", "side"), ("XURL_OUTPUT", "json")],
        &[
            "auth",
            "oauth2",
            "--no-browser",
            "--step",
            "2",
            "--auth-url",
            "https://localhost/callback?code=c&state=s",
        ],
    );
    let text = step_text(&document);
    assert!(
        text.contains("--app side") && text.contains("--output json"),
        "`{text}` runs where the shell's variable may say another app: {document}"
    );
}
