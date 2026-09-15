//! The unknown-command contract: a mistyped command names the nearest real
//! one and exits as a usage error, in the same wording and the same envelope
//! whether clap or the classifier caught it, before any config or store is
//! read; a bare `xr` prints help.
//!
//! Parallel-safe by construction, like `tests/cli_tests.rs`: every case runs
//! the library entrypoint against its own `TempDir`-rooted store and supplies
//! its environment as `EnvOverrides`, so nothing here reads or writes the
//! process environment.

mod common;

use tempfile::TempDir;
use xurl::cli;
use xurl::config::EnvOverrides;

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Runs the entrypoint with a fresh tempdir-rooted store and no environment.
fn run_isolated(args: &[&str]) -> (i32, String, String) {
    run_with_overrides(args, &EnvOverrides::default())
}

/// Runs the entrypoint with `XURL_OUTPUT` supplied as data.
fn run_with_output_env(args: &[&str], output: &str) -> (i32, String, String) {
    run_with_overrides(
        args,
        &EnvOverrides {
            output: Some(output.to_string()),
            ..EnvOverrides::default()
        },
    )
}

fn run_with_overrides(args: &[&str], overrides: &EnvOverrides) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code = cli::runner::run_with_overrides(args, &mut stdout, &mut stderr, &store, overrides);
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

/// The text line without the color wrap.
///
/// `OutputConfig` resolves `--color auto` against the process stderr, which
/// is a terminal when the suite runs under `--nocapture`, so a text-mode
/// assertion compares the stripped line.
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        for c in chars.by_ref() {
            if c == 'm' {
                break;
            }
        }
    }
    out
}

/// The envelope `stderr` carries, whichever structured format rendered it.
fn envelope(stderr: &str, format: &str) -> serde_json::Value {
    if format == "yaml" {
        return serde_yaml::from_str(stderr).expect("stderr is a YAML envelope");
    }
    serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope")
}

/// Asserts the unknown-command rendering for `format`.
fn assert_unknown_command(stderr: &str, format: &str, word: &str, suggestion: Option<&str>) {
    if format == "text" {
        assert_eq!(plain(stderr).trim_end(), text_line(word, suggestion));
        return;
    }
    assert!(
        !plain(stderr).starts_with("Error:"),
        "{format} must render the envelope alone; got: {stderr}"
    );
    let v = envelope(stderr, format);
    assert_eq!(v["status"], "error", "{format}: {stderr}");
    assert_eq!(v["reason"], "unknown-command", "{format}: {stderr}");
    assert_eq!(v["exit_code"], 2, "{format}: {stderr}");
    assert_eq!(v["command"], word, "{format}: {stderr}");
    match suggestion {
        Some(expected) => assert_eq!(v["suggestion"], expected, "{format}: {stderr}"),
        None => assert!(
            v.get("suggestion").is_none(),
            "{format} carries no suggestion; got: {stderr}"
        ),
    }
}

fn text_line(word: &str, suggestion: Option<&str>) -> String {
    match suggestion {
        Some(s) => format!("Error: unknown command '{word}'. Did you mean '{s}'? Try 'xr --help'."),
        None => format!("Error: unknown command '{word}'. Try 'xr --help'."),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// The classifier path: a bare word that is not a command
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mistyped_command_names_the_nearest_one() {
    let (code, stdout, stderr) = run_isolated(&["xr", "whoam"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stdout.is_empty(), "stdout stays empty: {stdout}");
    assert_eq!(
        plain(&stderr).trim_end(),
        "Error: unknown command 'whoam'. Did you mean 'whoami'? Try 'xr --help'."
    );
}

#[test]
fn mistyped_command_envelope_carries_command_and_suggestion() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "whoam"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "whoam", Some("whoami"));
}

#[test]
fn the_output_flag_is_read_after_the_word_too() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "whoam", "--output", "json"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "whoam", Some("whoami"));
}

#[test]
fn a_mistyped_command_is_matched_case_insensitively_and_echoed_verbatim() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "Whoam"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "Whoam", Some("whoami"));
}

#[test]
fn a_word_near_nothing_gets_no_suggestion() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "zzzzzz"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "zzzzzz", None);

    let (code, _stdout, stderr) = run_isolated(&["xr", "zzzzzz"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_eq!(
        plain(&stderr).trim_end(),
        "Error: unknown command 'zzzzzz'. Try 'xr --help'."
    );
}

#[test]
fn a_hostname_without_a_dot_is_an_unknown_command() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "localhost"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "localhost", None);
}

#[test]
fn a_swallowed_flag_value_is_reported_as_the_word_it_is() {
    // `--quiet false` no longer consumes `false`, so it falls through to the
    // positional and is classified there.
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "--quiet", "false"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "false", None);
}

#[test]
fn classification_precedes_the_store_load() {
    let tmp = TempDir::new().expect("tempdir");
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    // The store path is a directory: any read of it fails loudly.
    let code = cli::runner::run_with_overrides(
        ["xr", "whoam"],
        &mut stdout,
        &mut stderr,
        tmp.path(),
        &EnvOverrides::default(),
    );
    let rendered = String::from_utf8_lossy(&stderr);
    assert_eq!(code, 2, "stderr: {rendered}");
    assert_eq!(
        plain(&rendered).trim_end(),
        "Error: unknown command 'whoam'. Did you mean 'whoami'? Try 'xr --help'.",
        "nothing from the store loader precedes the line"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// The clap path: a word clap itself rejected
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn help_for_a_mistyped_command_scores_it_against_the_root() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "help", "whoam"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_eq!(
        plain(&stderr).trim_end(),
        "Error: unknown command 'whoam'. Did you mean 'whoami'? Try 'xr --help'."
    );
}

#[test]
fn the_output_flag_is_read_before_the_word_on_the_clap_path() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "help", "whoam"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "whoam", Some("whoami"));
}

#[test]
fn a_mistyped_verb_takes_claps_own_suggestion() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "auth", "statsu"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "statsu", Some("status"));
}

#[test]
fn a_verb_near_nothing_gets_no_suggestion() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "auth", "whoam"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "whoam", None);
}

// ═══════════════════════════════════════════════════════════════════════════
// What stays a raw request
// ═══════════════════════════════════════════════════════════════════════════

#[rstest::rstest]
#[case::dotted_host(&["xr", "--output", "json", "example.com"])]
#[case::port_and_path(&["xr", "--output", "json", "localhost:8080/x"])]
#[case::leading_digit(&["xr", "--output", "json", "123"])]
#[case::method_flag(&["xr", "--output", "json", "-X", "POST", "whoam"])]
#[case::data_flag(&["xr", "--output", "json", "-d", "{}", "whoam"])]
fn a_raw_target_keeps_the_url_error(#[case] args: &[&str]) {
    let (code, _stdout, stderr) = run_isolated(args);
    assert_eq!(code, 1, "args {args:?}; stderr: {stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr is a JSON envelope");
    assert_eq!(v["reason"], "validation", "args {args:?}; got: {v}");
}

// ═══════════════════════════════════════════════════════════════════════════
// A bare invocation
// ═══════════════════════════════════════════════════════════════════════════

#[rstest::rstest]
#[case::bare(&["xr"])]
#[case::quiet(&["xr", "-q"])]
fn a_bare_invocation_prints_help(#[case] args: &[&str]) {
    let (code, stdout, stderr) = run_isolated(args);
    assert_eq!(code, 0, "args {args:?}; stderr: {stderr}");
    assert!(
        stdout.contains("Usage: xr") && stdout.contains("Commands:"),
        "args {args:?}; stdout: {stdout}"
    );
    assert!(stderr.is_empty(), "args {args:?}; stderr: {stderr}");
}

#[test]
fn a_bare_invocation_under_structured_intent_is_a_usage_error() {
    let (code, stdout, stderr) = run_isolated(&["xr", "--output", "json"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stdout.is_empty(), "stdout stays empty: {stdout}");
    let v = envelope(&stderr, "json");
    assert_eq!(v["reason"], "invalid-args", "got: {v}");
    assert_eq!(v["exit_code"], 2, "got: {v}");
}

#[test]
fn a_raw_only_flag_alone_still_asks_for_a_url() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "-X", "POST"]);
    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(
        plain(&stderr).contains("No URL provided"),
        "stderr: {stderr}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Every format, from either source
// ═══════════════════════════════════════════════════════════════════════════

#[rstest::rstest]
#[case::text("text")]
#[case::json("json")]
#[case::jsonl("jsonl")]
#[case::ndjson("ndjson")]
#[case::yaml("yaml")]
#[case::csv("csv")]
#[case::tsv("tsv")]
fn unknown_command_renders_in_every_format(#[case] format: &str) {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", format, "whoam"]);
    assert_eq!(code, 2, "flag source, {format}; stderr: {stderr}");
    assert_unknown_command(&stderr, format, "whoam", Some("whoami"));
}

/// `XURL_OUTPUT` reaches the classifier through clap's `env` binding, which
/// reads the process rather than the injected overrides, so this half of the
/// contract is proven against a spawned binary.
#[rstest::rstest]
#[case::text("text")]
#[case::json("json")]
#[case::jsonl("jsonl")]
#[case::ndjson("ndjson")]
#[case::yaml("yaml")]
#[case::csv("csv")]
#[case::tsv("tsv")]
fn unknown_command_reads_the_output_env_var(#[case] format: &str) {
    let output = common::xr()
        .env("XURL_OUTPUT", format)
        .arg("whoam")
        .output()
        .expect("spawn xr");
    assert_eq!(output.status.code(), Some(2), "{format}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_unknown_command(&stderr, format, "whoam", Some("whoami"));
}

#[rstest::rstest]
#[case::text("text")]
#[case::json("json")]
#[case::jsonl("jsonl")]
#[case::ndjson("ndjson")]
#[case::yaml("yaml")]
#[case::csv("csv")]
#[case::tsv("tsv")]
fn invalid_args_renders_in_every_format(#[case] format: &str) {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", format, "--frobnicate"]);
    assert_eq!(code, 2, "flag source, {format}; stderr: {stderr}");
    assert_invalid_args(&stderr, format);

    let (code, _stdout, stderr) = run_with_output_env(&["xr", "--frobnicate"], format);
    assert_eq!(code, 2, "env source, {format}; stderr: {stderr}");
    assert_invalid_args(&stderr, format);
}

/// `yml` names the YAML rendering on the parse-error path, where the value
/// never reaches clap's own parser.
#[test]
fn the_yml_spelling_still_picks_the_yaml_rendering() {
    let (code, _stdout, stderr) = run_with_output_env(&["xr", "--frobnicate"], "yml");
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_invalid_args(&stderr, "yaml");
}

#[test]
fn the_json_alias_picks_the_json_rendering() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--json", "--frobnicate"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_invalid_args(&stderr, "json");
}

fn assert_invalid_args(stderr: &str, format: &str) {
    if format == "text" {
        assert!(
            stderr.contains("unexpected argument"),
            "text mode keeps clap's own rendering; got: {stderr}"
        );
        return;
    }
    let v = envelope(stderr, format);
    assert_eq!(v["status"], "error", "{format}: {stderr}");
    assert_eq!(v["reason"], "invalid-args", "{format}: {stderr}");
    assert_eq!(v["exit_code"], 2, "{format}: {stderr}");
    assert!(
        v["message"].is_string(),
        "{format} carries the message: {stderr}"
    );
}
