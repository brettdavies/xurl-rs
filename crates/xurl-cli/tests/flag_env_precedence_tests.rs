//! A typed flag and the variables of its setting, and what a refusal names.
//!
//! Every case spawns the built `xr` with the variables set on the child, since
//! clap reads a variable bound to an argument when the command is built.

mod common;

use std::process::Output;

/// Runs `xr` with `env` set on the child and returns its output.
fn xr(env: &[(&str, &str)], args: &[&str]) -> Output {
    let mut command = common::xr();
    for (name, value) in env {
        command.env(name, value);
    }
    command.args(args).output().expect("xr runs")
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The first line of the text refusal, without its `Error: ` prefix.
fn first_line(output: &Output) -> String {
    let stderr = stderr_of(output);
    let line = stderr.lines().next().expect("a refusal has a first line");
    line.strip_prefix("Error: ")
        .unwrap_or_else(|| panic!("not a text refusal: {stderr}"))
        .to_string()
}

/// The lines between the first line and the usage line.
fn between_first_line_and_usage(output: &Output) -> Vec<String> {
    stderr_of(output)
        .lines()
        .skip(1)
        .take_while(|line| !line.starts_with("Usage:"))
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn envelope(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stderr).unwrap_or_else(|e| panic!("{e}: {}", stderr_of(output)))
}

// ── A conflict clap raises names a side that came from a variable ─────────

#[test]
fn a_typed_after_beside_xurl_page_names_the_variable() {
    let output = xr(&[("XURL_PAGE", "2")], &["--after", "5", "schema", "--list"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(
        first_line(&output),
        "the argument '--after <TOKEN>' cannot be used with the environment variable 'XURL_PAGE'"
    );
    assert_eq!(
        between_first_line_and_usage(&output),
        ["  tip: unset 'XURL_PAGE', or leave out '--after <TOKEN>'"]
    );
    assert!(
        !stderr_of(&output).contains("--page"),
        "the flag the caller did not type is not named: {}",
        stderr_of(&output)
    );
}

#[test]
fn a_typed_cursor_beside_xurl_page_names_the_variable() {
    let output = xr(
        &[("XURL_PAGE", "2")],
        &["--cursor", "abc", "schema", "--list"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(
        first_line(&output),
        "the argument '--cursor <TOKEN>' cannot be used with the environment variable 'XURL_PAGE'"
    );
    assert_eq!(
        between_first_line_and_usage(&output),
        ["  tip: unset 'XURL_PAGE', or leave out '--cursor <TOKEN>'"]
    );
}

#[test]
fn a_flag_after_the_command_word_is_refused_the_same_way() {
    let output = xr(
        &[("XURL_PAGE", "2")],
        &["schema", "--list", "--cursor", "abc"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(
        first_line(&output),
        "the argument '--cursor <TOKEN>' cannot be used with the environment variable 'XURL_PAGE'"
    );
    assert!(
        stderr_of(&output)
            .trim_end()
            .ends_with("Try 'xr schema --help'."),
        "the pointer names the command: {}",
        stderr_of(&output)
    );
}

#[test]
fn two_typed_flags_keep_clap_sentence_and_get_no_tip() {
    let output = xr(&[], &["--cursor", "a", "--page", "2", "schema", "--list"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    let first = first_line(&output);
    assert!(
        first.starts_with("the argument '--") && first.contains("' cannot be used with '--"),
        "{first}"
    );
    assert!(!first.contains("environment variable"), "{first}");
    assert!(between_first_line_and_usage(&output).is_empty());
}

#[test]
fn a_repeated_flag_outside_the_two_settings_keeps_clap_sentence() {
    let output = xr(&[], &["--app", "a", "--app", "b", "whoami"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(
        first_line(&output),
        "the argument '--app <APP>' cannot be used multiple times"
    );
}

#[test]
fn the_refusal_envelope_carries_the_keys_it_carries_today() {
    let output = xr(
        &[("XURL_PAGE", "2")],
        &["--output", "json", "--cursor", "abc", "schema", "--list"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    let document = envelope(&output);
    assert_eq!(document["reason"], "invalid-args", "{document}");
    assert_eq!(document["exit_code"], 2, "{document}");
    assert_eq!(document["next_step"]["action"], "show-help", "{document}");
    let mut keys: Vec<&str> = document
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["exit_code", "message", "next_step", "reason", "status"]
    );
    let message = document["message"].as_str().expect("a message");
    assert!(
        message.starts_with(
            "the argument '--cursor <TOKEN>' cannot be used with the environment variable 'XURL_PAGE'\n\n  tip: unset 'XURL_PAGE'"
        ),
        "{message}"
    );
}
