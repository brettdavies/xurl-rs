//! The unknown-command contract: a mistyped command names the nearest real
//! one and exits as a usage error, in the same wording and the same envelope
//! whether clap or the classifier caught it and whether or not a help or
//! version flag follows it, before any config or store is read; a bare `xr`
//! prints help.
//!
//! Parallel-safe by construction, like `tests/cli_tests.rs`: every case runs
//! the library entrypoint against its own `TempDir`-rooted store and supplies
//! its environment as `EnvOverrides`, so nothing here reads or writes the
//! process environment.

mod common;

use clap::Parser;
use tempfile::TempDir;
use xdk::config::EnvOverrides;
use xurl::cli;
use xurl::cli::Cli;

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Runs the entrypoint with a fresh tempdir-rooted store and no environment.
async fn run_isolated(args: &[&str]) -> (i32, String, String) {
    run_with_overrides(args, &EnvOverrides::default()).await
}

/// Runs the entrypoint with `XURL_OUTPUT` supplied as data.
async fn run_with_output_env(args: &[&str], output: &str) -> (i32, String, String) {
    run_with_overrides(
        args,
        &EnvOverrides {
            output: Some(output.to_string()),
            ..EnvOverrides::default()
        },
    )
    .await
}

async fn run_with_overrides(args: &[&str], overrides: &EnvOverrides) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code =
        cli::runner::run_with_overrides(args, &mut stdout, &mut stderr, &store, overrides).await;
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

/// Asserts the unknown-command rendering of a root-level word for `format`.
fn assert_unknown_command(stderr: &str, format: &str, word: &str, suggestion: Option<&str>) {
    assert_unknown_command_under(stderr, format, word, suggestion, "xr");
}

/// Asserts the unknown-command rendering of a word typed under `parent`.
///
/// Structured formats carry a `next_step` an agent can run as given: the help
/// of the nearest command, or of `parent` when nothing scored.
fn assert_unknown_command_under(
    stderr: &str,
    format: &str,
    word: &str,
    suggestion: Option<&str>,
    parent: &str,
) {
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
    let step = &v["next_step"];
    assert_eq!(step["action"], "show-help", "{format}: {stderr}");
    let expected = match suggestion {
        Some(nearest) => format!("{parent} {nearest} --help"),
        None => format!("{parent} --help"),
    };
    assert_eq!(step["command"], expected.as_str(), "{format}: {stderr}");
    assert!(
        step.get("template").is_none(),
        "{format} runs as given; got: {stderr}"
    );
}

fn text_line(word: &str, suggestion: Option<&str>) -> String {
    match suggestion {
        Some(s) => {
            format!("Error: unknown command '{word}'. Did you mean '{s}'? Try 'xr {s} --help'.")
        }
        None => format!("Error: unknown command '{word}'. Try 'xr --help'."),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// The classifier path: a bare word that is not a command
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn mistyped_command_names_the_nearest_one() {
    let (code, stdout, stderr) = run_isolated(&["xr", "whoam"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stdout.is_empty(), "stdout stays empty: {stdout}");
    assert_eq!(
        plain(&stderr).trim_end(),
        "Error: unknown command 'whoam'. Did you mean 'whoami'? Try 'xr whoami --help'."
    );
}

#[tokio::test]
async fn mistyped_command_envelope_carries_command_and_suggestion() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "whoam"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "whoam", Some("whoami"));
}

#[tokio::test]
async fn the_output_flag_is_read_after_the_word_too() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "whoam", "--output", "json"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "whoam", Some("whoami"));
}

#[tokio::test]
async fn a_mistyped_command_is_matched_case_insensitively_and_echoed_verbatim() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "Whoam"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "Whoam", Some("whoami"));
}

#[tokio::test]
async fn a_word_near_nothing_gets_no_suggestion() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "zzzzzz"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "zzzzzz", None);

    let (code, _stdout, stderr) = run_isolated(&["xr", "zzzzzz"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_eq!(
        plain(&stderr).trim_end(),
        "Error: unknown command 'zzzzzz'. Try 'xr --help'."
    );
}

#[tokio::test]
async fn a_hostname_without_a_dot_is_an_unknown_command() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "localhost"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "localhost", None);
}

#[tokio::test]
async fn a_swallowed_flag_value_is_reported_as_the_word_it_is() {
    // `--quiet false` no longer consumes `false`, so it falls through to the
    // positional and is classified there.
    let (code, _stdout, stderr) =
        run_isolated(&["xr", "--output", "json", "--quiet", "false"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "false", None);
}

#[tokio::test]
async fn classification_precedes_the_store_load() {
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
    )
    .await;
    let rendered = String::from_utf8_lossy(&stderr);
    assert_eq!(code, 2, "stderr: {rendered}");
    assert_eq!(
        plain(&rendered).trim_end(),
        "Error: unknown command 'whoam'. Did you mean 'whoami'? Try 'xr whoami --help'.",
        "nothing from the store loader precedes the line"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// The clap path: a word clap itself rejected
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn help_for_a_mistyped_command_scores_it_against_the_root() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "help", "whoam"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_eq!(
        plain(&stderr).trim_end(),
        "Error: unknown command 'whoam'. Did you mean 'whoami'? Try 'xr whoami --help'."
    );
}

#[tokio::test]
async fn the_output_flag_is_read_before_the_word_on_the_clap_path() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "help", "whoam"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command(&stderr, "json", "whoam", Some("whoami"));
}

#[tokio::test]
async fn a_mistyped_verb_takes_claps_own_suggestion() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "auth", "statsu"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command_under(&stderr, "json", "statsu", Some("status"), "xr auth");
}

#[tokio::test]
async fn a_verb_near_nothing_gets_no_suggestion() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", "auth", "whoam"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command_under(&stderr, "json", "whoam", None, "xr auth");
}

#[tokio::test]
async fn a_mistyped_nested_verb_steps_to_its_own_family() {
    let (code, _stdout, stderr) =
        run_isolated(&["xr", "--output", "json", "auth", "apps", "ad"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command_under(&stderr, "json", "ad", Some("add"), "xr auth apps");
}

#[tokio::test]
async fn the_next_step_names_xr_whatever_the_binary_is_called() {
    let (code, _stdout, stderr) =
        run_isolated(&["xurl-rs", "--output", "json", "auth", "statsu"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_unknown_command_under(&stderr, "json", "statsu", Some("status"), "xr auth");
}

/// The `next_step` command runs as given, from any detection path, and prints
/// a help page.
#[rstest::rstest]
#[case::root_word(&["xr", "--output", "json", "whoam"])]
#[case::root_word_near_nothing(&["xr", "--output", "json", "zzzzzz"])]
#[case::help_flag(&["xr", "--output", "json", "webhooks", "--help"])]
#[case::help_command(&["xr", "--output", "json", "help", "whoam"])]
#[case::version_flag(&["xr", "--output", "json", "webhooks", "--version"])]
#[case::verb(&["xr", "--output", "json", "auth", "statsu"])]
#[case::nested_verb(&["xr", "--output", "json", "auth", "apps", "ad"])]
#[tokio::test]
async fn the_next_step_runs_as_given(#[case] args: &[&str]) {
    let (_, _, stderr) = run_isolated(args).await;
    let v = envelope(&stderr, "json");
    let step = v["next_step"]["command"]
        .as_str()
        .unwrap_or_else(|| panic!("args {args:?} carry a runnable step; got: {v}"));
    let argv: Vec<&str> = step.split_whitespace().collect();
    let (code, stdout, stderr) = run_isolated(&argv).await;
    assert_eq!(code, 0, "step {step:?}; stderr: {stderr}");
    let target = step
        .strip_suffix(" --help")
        .unwrap_or_else(|| panic!("step {step:?} asks for help"));
    assert!(
        stdout.contains(&format!("Usage: {target}")),
        "step {step:?} prints the page of {target}; stdout: {stdout}"
    );
}

/// Asked for its own help, the `help` command prints its page, as
/// `xr help help` does.
#[rstest::rstest]
#[case::long(&["xr", "help", "--help"])]
#[case::short(&["xr", "help", "-h"])]
#[case::after_a_command(&["xr", "help", "whoami", "--help"])]
#[tokio::test]
async fn the_help_command_asked_for_help_prints_its_own_page(#[case] args: &[&str]) {
    let (_, page, _) = run_isolated(&["xr", "help", "help"]).await;
    assert!(page.contains("Usage: xr help"), "the page itself: {page}");
    let (code, stdout, stderr) = run_isolated(args).await;
    assert_eq!(code, 0, "args {args:?}; stderr: {stderr}");
    assert_eq!(stdout, page, "args {args:?}");
    assert!(stderr.is_empty(), "args {args:?}; stderr: {stderr}");
}

/// A flag spelling where clap wanted a command is an unexpected argument:
/// never an unknown command, and never a suggestion to run `help`.
#[rstest::rstest]
#[case::long_after_the_separator(&["--", "webhooks", "--help"], "--help")]
#[case::short_after_the_separator(&["--", "webhooks", "-h"], "-h")]
#[case::any_flag_after_the_separator(&["--", "webhooks", "--frob"], "--frob")]
#[case::any_flag_under_help(&["help", "--frob"], "--frob")]
#[tokio::test]
async fn a_flag_where_a_command_goes_is_an_unexpected_argument(
    #[case] rest: &[&str],
    #[case] flag: &str,
) {
    let mut args = vec!["xr"];
    args.extend(rest);
    let (code, stdout, stderr) = run_isolated(&args).await;
    assert_eq!(code, 2, "args {args:?}; stderr: {stderr}");
    assert!(stdout.is_empty(), "args {args:?}; stdout: {stdout}");
    let text = plain(&stderr);
    assert!(
        text.contains(&format!("unexpected argument '{flag}' found")),
        "args {args:?}; stderr: {stderr}"
    );
    assert!(
        !text.contains("command") && !text.contains("'help'"),
        "args {args:?} names no command; stderr: {stderr}"
    );

    let mut json = vec!["xr", "--output", "json"];
    json.extend(rest);
    let (code, _stdout, stderr) = run_isolated(&json).await;
    assert_eq!(code, 2, "args {json:?}; stderr: {stderr}");
    let v = envelope(&stderr, "json");
    assert_eq!(v["reason"], "invalid-args", "args {json:?}; got: {v}");
    assert!(v.get("command").is_none(), "args {json:?}; got: {v}");
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
#[tokio::test]
async fn a_raw_target_keeps_the_url_error(#[case] args: &[&str]) {
    let (code, _stdout, stderr) = run_isolated(args).await;
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
#[tokio::test]
async fn a_bare_invocation_prints_help(#[case] args: &[&str]) {
    let (code, stdout, stderr) = run_isolated(args).await;
    assert_eq!(code, 0, "args {args:?}; stderr: {stderr}");
    assert!(
        stdout.contains("Usage: xr") && stdout.contains("Commands:"),
        "args {args:?}; stdout: {stdout}"
    );
    assert!(stderr.is_empty(), "args {args:?}; stderr: {stderr}");
}

#[tokio::test]
async fn a_bare_invocation_under_structured_intent_is_a_usage_error() {
    let (code, stdout, stderr) = run_isolated(&["xr", "--output", "json"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stdout.is_empty(), "stdout stays empty: {stdout}");
    let v = envelope(&stderr, "json");
    assert_eq!(v["reason"], "invalid-args", "got: {v}");
    assert_eq!(v["exit_code"], 2, "got: {v}");
}

#[tokio::test]
async fn a_raw_only_flag_alone_still_asks_for_a_url() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "-X", "POST"]).await;
    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(
        plain(&stderr).contains("No URL provided"),
        "stderr: {stderr}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A help or version flag
// ═══════════════════════════════════════════════════════════════════════════

#[rstest::rstest]
#[case::near_bookmarks("webhooks", Some("bookmarks"))]
#[case::near_whoami("whoam", Some("whoami"))]
#[case::near_nothing("zzzzzz", None)]
#[tokio::test]
async fn a_help_or_version_flag_does_not_hide_a_mistyped_command(
    #[case] word: &str,
    #[case] suggestion: Option<&str>,
    #[values("--help", "-h", "--version", "-V")] flag: &str,
) {
    let (code, stdout, stderr) = run_isolated(&["xr", word, flag]).await;
    assert_eq!(code, 2, "{word} {flag}; stdout: {stdout}; stderr: {stderr}");
    assert!(stdout.is_empty(), "{word} {flag} prints no help: {stdout}");
    assert_eq!(
        plain(&stderr).trim_end(),
        text_line(word, suggestion),
        "{word} {flag}"
    );
    assert_eq!(
        (code, stdout, stderr),
        run_isolated(&["xr", word]).await,
        "{word} {flag} renders exactly as {word} alone"
    );

    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", "json", word, flag]).await;
    assert_eq!(code, 2, "json, {word} {flag}; stderr: {stderr}");
    assert_unknown_command(&stderr, "json", word, suggestion);
}

/// The flag's place in argv changes nothing: ahead of the word, it still
/// renders as the word does alone.
#[rstest::rstest]
#[case::help("-h")]
#[case::version("--version")]
#[case::version_short("-V")]
#[tokio::test]
async fn a_display_flag_ahead_of_the_word_does_not_hide_it(#[case] flag: &str) {
    assert_eq!(
        run_isolated(&["xr", flag, "webhooks"]).await,
        run_isolated(&["xr", "webhooks"]).await,
        "{flag} webhooks"
    );
}

/// Every other help or version display is clap's own rendering at exit 0:
/// no command at all, a real command, a nested family, a URL, and a raw-only
/// flag.
#[rstest::rstest]
#[case::root_long(&["xr", "--help"])]
#[case::root_short(&["xr", "-h"])]
#[case::root_under_structured_intent(&["xr", "--output", "json", "--help"])]
#[case::help_subcommand(&["xr", "help"])]
#[case::help_subcommand_with_a_command(&["xr", "help", "whoami"])]
#[case::command_long(&["xr", "whoami", "--help"])]
#[case::command_short(&["xr", "whoami", "-h"])]
#[case::nested_family(&["xr", "auth", "apps", "--help"])]
#[case::missing_subcommand(&["xr", "auth"])]
#[case::url(&["xr", "/2/users/me", "--help"])]
#[case::raw_only_flag(&["xr", "-X", "POST", "webhooks", "--help"])]
#[case::version(&["xr", "--version"])]
#[case::version_short(&["xr", "-V"])]
#[case::version_under_structured_intent(&["xr", "--output", "json", "--version"])]
#[case::version_after_a_url(&["xr", "/2/users/me", "--version"])]
#[tokio::test]
async fn every_other_help_or_version_display_is_claps_own(#[case] args: &[&str]) {
    let display = Cli::try_parse_from(args)
        .expect_err("clap displays help or the version")
        .to_string();
    let (code, stdout, stderr) = run_isolated(args).await;
    assert_eq!(code, 0, "args {args:?}; stderr: {stderr}");
    assert_eq!(stdout, display, "args {args:?}");
    assert!(stderr.is_empty(), "args {args:?}; stderr: {stderr}");
}

// ═══════════════════════════════════════════════════════════════════════════
// Color on the parse-error path
// ═══════════════════════════════════════════════════════════════════════════

const RED: &str = "\u{1b}[31mError: ";

/// `--color` reaches a rendering clap's failure hands the runner, wherever
/// the flag sits in argv, as it does for an error raised after the parse.
#[rstest::rstest]
#[case::help_flag_path(&["xr", "--color", "always", "webhooks", "--help"])]
#[case::help_flag_path_flag_last(&["xr", "webhooks", "-h", "--color=always"])]
#[case::clap_rejection(&["xr", "--color", "always", "auth", "statsu"])]
#[case::clap_rejection_flag_after_the_word(&["xr", "auth", "statsu", "--color", "always"])]
#[case::clap_text(&["xr", "--color", "always", "--bogus-flag"])]
#[case::version_flag_path(&["xr", "--color", "always", "webhooks", "--version"])]
#[tokio::test]
async fn an_explicit_color_flag_reaches_the_parse_error_rendering(#[case] args: &[&str]) {
    let (code, _stdout, stderr) = run_isolated(args).await;
    assert_eq!(code, 2, "args {args:?}; stderr: {stderr}");
    assert!(stderr.starts_with(RED), "args {args:?}; stderr: {stderr:?}");

    let never: Vec<&str> = args
        .iter()
        .map(|a| match *a {
            "always" => "never",
            "--color=always" => "--color=never",
            other => other,
        })
        .collect();
    let (code, _stdout, stderr) = run_isolated(&never).await;
    assert_eq!(code, 2, "args {never:?}; stderr: {stderr}");
    assert!(
        !stderr.contains('\u{1b}'),
        "args {never:?}; stderr: {stderr:?}"
    );
}

#[tokio::test]
async fn no_color_outranks_an_explicit_color_flag_on_the_parse_error_path() {
    let (code, _stdout, stderr) = run_with_overrides(
        &["xr", "--color", "always", "auth", "statsu"],
        &EnvOverrides {
            no_color: true,
            ..EnvOverrides::default()
        },
    )
    .await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(!stderr.contains('\u{1b}'), "stderr: {stderr:?}");
}

/// `XURL_COLOR` reaches the parse-error path through clap's `env` binding,
/// which reads the process rather than the injected overrides.
#[rstest::rstest]
#[case::help_flag_path(&["webhooks", "--help"])]
#[case::clap_rejection(&["auth", "statsu"])]
fn the_color_env_var_reaches_the_parse_error_rendering(#[case] args: &[&str]) {
    let output = common::xr()
        .env("XURL_COLOR", "always")
        .args(args)
        .output()
        .expect("spawn xr");
    assert_eq!(output.status.code(), Some(2), "args {args:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with(RED), "args {args:?}; stderr: {stderr:?}");
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
#[tokio::test]
async fn unknown_command_renders_in_every_format(
    #[case] format: &str,
    #[values(None, Some("--help"), Some("-h"))] help: Option<&str>,
) {
    let mut args = vec!["xr", "--output", format, "whoam"];
    args.extend(help);
    let (code, stdout, stderr) = run_isolated(&args).await;
    assert_eq!(code, 2, "flag source, {format}, {help:?}; stderr: {stderr}");
    assert!(stdout.is_empty(), "{format}, {help:?}; stdout: {stdout}");
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
fn unknown_command_reads_the_output_env_var(
    #[case] format: &str,
    #[values(None, Some("--help"), Some("-h"))] help: Option<&str>,
) {
    let output = common::xr()
        .env("XURL_OUTPUT", format)
        .arg("whoam")
        .args(help)
        .output()
        .expect("spawn xr");
    assert_eq!(output.status.code(), Some(2), "{format}, {help:?}");
    assert!(output.stdout.is_empty(), "{format}, {help:?}");
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
#[tokio::test]
async fn invalid_args_renders_in_every_format(#[case] format: &str) {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--output", format, "--frobnicate"]).await;
    assert_eq!(code, 2, "flag source, {format}; stderr: {stderr}");
    assert_invalid_args(&stderr, format);

    let (code, _stdout, stderr) = run_with_output_env(&["xr", "--frobnicate"], format).await;
    assert_eq!(code, 2, "env source, {format}; stderr: {stderr}");
    assert_invalid_args(&stderr, format);
}

/// `yml` names the YAML rendering on the parse-error path, where the value
/// never reaches clap's own parser.
#[tokio::test]
async fn the_yml_spelling_still_picks_the_yaml_rendering() {
    let (code, _stdout, stderr) = run_with_output_env(&["xr", "--frobnicate"], "yml").await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_invalid_args(&stderr, "yaml");
}

#[tokio::test]
async fn the_json_alias_picks_the_json_rendering() {
    let (code, _stdout, stderr) = run_isolated(&["xr", "--json", "--frobnicate"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    assert_invalid_args(&stderr, "json");
}

/// Asserts the `invalid-args` rendering of a root-level `--frobnicate`: clap's
/// words in `xr`'s dialect, with no `error:` prefix inside the message.
fn assert_invalid_args(stderr: &str, format: &str) {
    let opening = "unexpected argument '--frobnicate' found";
    let closing = "Try 'xr --help'.";
    if format == "text" {
        let text = plain(stderr);
        assert!(
            text.starts_with(&format!("Error: {opening}")),
            "text: {stderr}"
        );
        assert!(text.trim_end().ends_with(closing), "text: {stderr}");
        return;
    }
    let v = envelope(stderr, format);
    assert_eq!(v["status"], "error", "{format}: {stderr}");
    assert_eq!(v["reason"], "invalid-args", "{format}: {stderr}");
    assert_eq!(v["exit_code"], 2, "{format}: {stderr}");
    let message = v["message"].as_str().expect("the message is a string");
    assert!(message.starts_with(opening), "{format}: {stderr}");
    assert!(message.ends_with(closing), "{format}: {stderr}");
}

// ═══════════════════════════════════════════════════════════════════════════
// One dialect, whoever raised the error
// ═══════════════════════════════════════════════════════════════════════════

/// Every text error opens with `Error:` and closes by pointing at the help of
/// the command it belongs to, whether clap or `xr` raised it.
#[rstest::rstest]
#[case::root_flag(&["xr", "--bogus-flag"], "unexpected argument '--bogus-flag' found", "Try 'xr --help'.")]
#[case::command_argument(&["xr", "whoami", "extra"], "unexpected argument 'extra' found", "Try 'xr whoami --help'.")]
#[case::family_flag(&["xr", "auth", "--frob"], "unexpected argument '--frob' found", "Try 'xr auth --help'.")]
#[case::invalid_value(&["xr", "skill", "install", "bogus_host"], "invalid value 'bogus_host' for '[HOST]'", "Try 'xr skill install --help'.")]
#[case::unknown_verb(&["xr", "auth", "statsu"], "unknown command 'statsu'. Did you mean 'status'?", "Try 'xr auth status --help'.")]
#[case::unknown_verb_near_nothing(&["xr", "auth", "whoam"], "unknown command 'whoam'.", "Try 'xr auth --help'.")]
#[case::unknown_nested_verb(&["xr", "auth", "apps", "ad"], "unknown command 'ad'. Did you mean 'add'?", "Try 'xr auth apps add --help'.")]
#[case::unknown_root_word(&["xr", "whoam"], "unknown command 'whoam'. Did you mean 'whoami'?", "Try 'xr whoami --help'.")]
#[case::no_url(&["xr", "-X", "POST"], "No URL provided.", "Try 'xr --help'.")]
#[tokio::test]
async fn every_text_error_speaks_one_dialect(
    #[case] args: &[&str],
    #[case] opening: &str,
    #[case] closing: &str,
) {
    let (code, _stdout, stderr) = run_isolated(args).await;
    assert_ne!(code, 0, "args {args:?}; stderr: {stderr}");
    let text = plain(&stderr);
    let text = text.trim_end();
    assert!(
        text.starts_with(&format!("Error: {opening}")),
        "args {args:?}; stderr: {stderr}"
    );
    assert!(text.ends_with(closing), "args {args:?}; stderr: {stderr}");
    assert!(
        !text.contains("For more information") && !text.contains("error: "),
        "args {args:?} carries no second dialect; stderr: {stderr}"
    );
}

/// The pointer names `xr` however the binary was invoked. clap takes the
/// command in its usage line from `argv[0]`, which is `xurl-rs` under the
/// Homebrew alias and can carry `.exe` on Windows.
#[rstest::rstest]
#[case::verb(&["auth", "statsu"], "Try 'xr auth status --help'.")]
#[case::argument(&["whoami", "extra"], "Try 'xr whoami --help'.")]
#[case::invalid_value(&["skill", "install", "bogus_host"], "Try 'xr skill install --help'.")]
#[tokio::test]
async fn the_pointer_names_xr_whatever_the_binary_is_called(
    #[case] rest: &[&str],
    #[case] closing: &str,
    #[values("xurl-rs", "/usr/local/bin/xurl-rs", "xr.exe")] bin: &str,
) {
    let mut args = vec![bin];
    args.extend(rest);
    let (code, _stdout, stderr) = run_isolated(&args).await;
    assert_eq!(code, 2, "args {args:?}; stderr: {stderr}");
    assert!(
        plain(&stderr).trim_end().ends_with(closing),
        "args {args:?}; stderr: {stderr}"
    );
}

#[tokio::test]
async fn a_structured_parse_error_names_the_command_it_belongs_to() {
    let (code, _stdout, stderr) =
        run_isolated(&["xr", "--output", "json", "whoami", "extra"]).await;
    assert_eq!(code, 2, "stderr: {stderr}");
    let v = envelope(&stderr, "json");
    assert_eq!(v["reason"], "invalid-args", "got: {v}");
    let message = v["message"].as_str().expect("the message is a string");
    assert!(
        message.starts_with("unexpected argument 'extra' found"),
        "got: {v}"
    );
    assert!(message.ends_with("Try 'xr whoami --help'."), "got: {v}");
}
