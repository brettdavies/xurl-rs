//! A typed flag and the variables of its setting, and what a refusal names.
//!
//! Every case spawns the built `xr` with the variables set on the child, since
//! clap reads a variable bound to an argument when the command is built.

mod common;

use std::process::Output;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

/// The lines between the first line and the usage line, or the help pointer
/// when the refusal carries no usage line.
fn between_first_line_and_usage(output: &Output) -> Vec<String> {
    stderr_of(output)
        .lines()
        .skip(1)
        .take_while(|line| !line.starts_with("Usage:") && !line.starts_with("Try '"))
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

// ── What a run prints, and which page token it sends ──────────────────────

/// The format of what a run printed on stdout, from the first bytes of
/// `schema --list`, which every format renders its own way.
#[derive(Debug, PartialEq, Eq)]
enum Shown {
    Text,
    Json,
    Jsonl,
    Yaml,
}

fn shown(output: &Output) -> Shown {
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.starts_with("[\n") {
        Shown::Json
    } else if stdout.starts_with("[{") {
        Shown::Jsonl
    } else if stdout.starts_with("- ") {
        Shown::Yaml
    } else {
        Shown::Text
    }
}

/// Runs `xr <args> schema --list` and asserts it exits 0 printing `expected`.
fn assert_shows(env: &[(&str, &str)], args: &[&str], expected: Shown) {
    let mut argv = args.to_vec();
    argv.extend(["schema", "--list"]);
    let output = xr(env, &argv);
    assert_eq!(
        output.status.code(),
        Some(0),
        "env {env:?} args {args:?}: {}",
        stderr_of(&output)
    );
    assert_eq!(
        shown(&output),
        expected,
        "env {env:?} args {args:?}: {}",
        String::from_utf8_lossy(&output.stdout)
            .chars()
            .take(80)
            .collect::<String>()
    );
}

/// The `pagination_token` the one request of `xr <args>` carries, with `env`
/// set on the child, against a local server.
async fn token_sent(env: &[(&str, &str)], args: &[&str]) -> Option<String> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"data":[{"id":"1","text":"hi"}],"meta":{"result_count":1}}"#),
        )
        .mount(&server)
        .await;
    let uri = server.uri();
    let mut all = vec![
        ("API_BASE_URL", uri.as_str()),
        ("XURL_BEARER_TOKEN", "env-bearer-value"),
    ];
    all.extend_from_slice(env);
    let output = xr(&all, args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "env {env:?} args {args:?}: {}",
        stderr_of(&output)
    );
    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(requests.len(), 1, "env {env:?} args {args:?}");
    requests[0]
        .url
        .query_pairs()
        .find(|(name, _)| name == "pagination_token")
        .map(|(_, value)| value.into_owned())
}

const SEARCH: [&str; 4] = ["search", "--auth", "app", "hi"];

fn search_with(front: &[&str], back: &[&str]) -> Vec<String> {
    front
        .iter()
        .chain(&SEARCH)
        .chain(back)
        .map(|word| (*word).to_string())
        .collect()
}

async fn token_for(env: &[(&str, &str)], front: &[&str], back: &[&str]) -> Option<String> {
    let args = search_with(front, back);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    token_sent(env, &args).await
}

// ── The output format: a typed flag beats every variable of its setting ───

#[test]
fn ae1_a_typed_json_beats_xurl_output() {
    assert_shows(&[("XURL_OUTPUT", "yaml")], &["--json"], Shown::Json);
}

#[test]
fn ae2_a_typed_output_beats_xurl_jsonl() {
    assert_shows(&[("XURL_JSONL", "1")], &["--output", "yaml"], Shown::Yaml);
}

#[test]
fn ae3_a_typed_output_beats_a_false_xurl_json() {
    assert_shows(&[("XURL_JSON", "0")], &["--output", "json"], Shown::Json);
}

#[test]
fn ae4_a_false_xurl_json_is_unset_beside_xurl_output() {
    assert_shows(
        &[("XURL_OUTPUT", "yaml"), ("XURL_JSON", "0")],
        &[],
        Shown::Yaml,
    );
}

#[test]
fn ae16_two_false_alias_variables_leave_text() {
    assert_shows(&[("XURL_JSON", "0"), ("XURL_JSONL", "0")], &[], Shown::Text);
}

#[test]
fn a_typed_text_beats_a_true_variable() {
    assert_shows(&[("XURL_JSON", "1")], &["--output", "text"], Shown::Text);
}

#[test]
fn a_typed_spelling_silences_two_variables_that_would_refuse() {
    assert_shows(
        &[("XURL_OUTPUT", "json"), ("XURL_JSON", "1")],
        &["--output", "yaml"],
        Shown::Yaml,
    );
}

#[test]
fn a_typed_spelling_silences_a_variable_that_names_no_format() {
    assert_shows(&[("XURL_OUTPUT", "toml")], &["--json"], Shown::Json);
}

#[test]
fn an_empty_xurl_output_is_unset() {
    assert_shows(&[("XURL_OUTPUT", "")], &[], Shown::Text);
}

#[test]
fn an_unknown_boolean_variable_reads_true_as_the_flag_does() {
    assert_shows(&[("XURL_JSON", "maybe")], &[], Shown::Json);
}

#[test]
fn each_output_variable_takes_effect_alone() {
    assert_shows(&[("XURL_JSON", "1")], &[], Shown::Json);
    assert_shows(&[("XURL_JSONL", "1")], &[], Shown::Jsonl);
    assert_shows(&[("XURL_OUTPUT", "yml")], &[], Shown::Yaml);
}

// ── The output format: of several typed, the last wins ────────────────────

#[test]
fn ae7_an_equals_form_output_after_json_wins() {
    assert_shows(&[], &["--json", "--output=yaml"], Shown::Yaml);
}

#[test]
fn ae8_a_json_after_output_wins() {
    assert_shows(&[], &["--output", "yaml", "--json"], Shown::Json);
}

#[test]
fn ae12_a_flag_after_the_command_word_beats_one_before_it() {
    let output = xr(&[], &["--json", "schema", "--list", "--output", "yaml"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr_of(&output));
    assert_eq!(shown(&output), Shown::Yaml);
}

#[test]
fn a_json_after_the_command_word_beats_an_output_before_it() {
    let output = xr(&[], &["--output", "yaml", "schema", "--list", "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr_of(&output));
    assert_eq!(shown(&output), Shown::Json);
}

#[test]
fn a_repeated_output_resolves_last_wins() {
    assert_shows(&[], &["--output", "json", "--output", "yaml"], Shown::Yaml);
}

#[test]
fn a_repeated_json_is_accepted() {
    assert_shows(&[], &["--json", "--json"], Shown::Json);
}

#[test]
fn json_then_jsonl_resolves_to_jsonl() {
    let output = xr(&[], &["--json", "--jsonl", "version"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr_of(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("{\"name\":\"xr\""), "{stdout}");
}

// ── The output format: refusals ───────────────────────────────────────────

fn message_of(output: &Output) -> String {
    envelope(output)["message"]
        .as_str()
        .expect("a message")
        .to_string()
}

#[test]
fn ae11_two_variables_of_one_setting_are_refused_in_a_json_envelope() {
    let output = xr(
        &[("XURL_OUTPUT", "json"), ("XURL_JSON", "1")],
        &["schema", "--list"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    let document = envelope(&output);
    assert_eq!(document["reason"], "invalid-args", "{document}");
    assert_eq!(
        message_of(&output),
        "the environment variable 'XURL_OUTPUT' cannot be used with the environment variable 'XURL_JSON'\n\n  \
         tip: unset one of 'XURL_OUTPUT' and 'XURL_JSON'\n\nTry 'xr schema --help'."
    );
}

#[test]
fn two_variables_are_refused_even_when_one_names_text() {
    let output = xr(
        &[("XURL_OUTPUT", "text"), ("XURL_JSON", "1")],
        &["schema", "--list"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert!(
        message_of(&output).starts_with(
            "the environment variable 'XURL_OUTPUT' cannot be used with the environment variable 'XURL_JSON'"
        ),
        "{}",
        message_of(&output)
    );
}

#[test]
fn three_variables_get_the_all_but_one_tip() {
    let output = xr(
        &[
            ("XURL_OUTPUT", "json"),
            ("XURL_JSON", "1"),
            ("XURL_JSONL", "1"),
        ],
        &["schema", "--list"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(
        message_of(&output),
        "the environment variable 'XURL_OUTPUT' cannot be used with:\n  \
         the environment variable 'XURL_JSON'\n  the environment variable 'XURL_JSONL'\n\n  \
         tip: unset all but one of 'XURL_OUTPUT', 'XURL_JSON', and 'XURL_JSONL'\n\nTry 'xr schema --help'."
    );
}

#[test]
fn a_variable_naming_no_format_is_refused_naming_the_variable() {
    let output = xr(&[("XURL_OUTPUT", "toml")], &["schema", "--list"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    let document = envelope(&output);
    assert_eq!(document["reason"], "invalid-args", "{document}");
    assert_eq!(
        message_of(&output),
        "invalid value 'toml' for the environment variable 'XURL_OUTPUT'\n  \
         [possible values: text, json, jsonl, ndjson, yaml, csv, tsv]\n\nTry 'xr schema --help'."
    );
}

// ── The page token ────────────────────────────────────────────────────────

#[test]
fn ae5_a_cursor_variable_beside_a_typed_page_names_the_variable() {
    let output = xr(
        &[("XURL_CURSOR", "abc")],
        &["--page", "2", "schema", "--list"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(
        first_line(&output),
        "the environment variable 'XURL_CURSOR' cannot be used with '--page <N>'"
    );
    assert_eq!(
        between_first_line_and_usage(&output),
        ["  tip: unset 'XURL_CURSOR', or leave out '--page <N>'"]
    );
    assert!(
        !stderr_of(&output).contains("--cursor"),
        "the flag the caller did not type is not named: {}",
        stderr_of(&output)
    );
}

#[test]
fn ae18_an_after_variable_beside_a_typed_page_names_the_variable() {
    let output = xr(&[("XURL_AFTER", "x")], &["--page", "2", "schema", "--list"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(
        first_line(&output),
        "the environment variable 'XURL_AFTER' cannot be used with '--page <N>'"
    );
    assert_eq!(
        between_first_line_and_usage(&output),
        ["  tip: unset 'XURL_AFTER', or leave out '--page <N>'"]
    );
    assert!(
        !stderr_of(&output).contains("--after"),
        "the flag the caller did not type is not named: {}",
        stderr_of(&output)
    );
}

#[test]
fn ae15_two_page_token_variables_are_refused_both_named() {
    let output = xr(
        &[("XURL_CURSOR", "a"), ("XURL_AFTER", "b")],
        &["schema", "--list"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(
        first_line(&output),
        "the environment variable 'XURL_CURSOR' cannot be used with the environment variable 'XURL_AFTER'"
    );
    assert_eq!(
        between_first_line_and_usage(&output),
        ["  tip: unset one of 'XURL_CURSOR' and 'XURL_AFTER'"]
    );
}

#[test]
fn a_cursor_variable_beside_the_page_variable_is_refused_before_pagination() {
    let output = xr(
        &[("XURL_CURSOR", "a"), ("XURL_PAGE", "2")],
        &["schema", "--list"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(
        first_line(&output),
        "the environment variable 'XURL_CURSOR' cannot be used with the environment variable 'XURL_PAGE'"
    );
    assert_eq!(
        between_first_line_and_usage(&output),
        ["  tip: unset one of 'XURL_CURSOR' and 'XURL_PAGE'"]
    );
    assert!(!stderr_of(&output).contains("unsupported-pagination"));
}

#[tokio::test]
async fn ae9_the_page_token_typed_last_is_sent() {
    assert_eq!(
        token_for(&[], &["--cursor", "a", "--after", "b"], &[]).await,
        Some("b".to_string())
    );
    assert_eq!(
        token_for(&[], &["--after", "b", "--cursor", "a"], &[]).await,
        Some("a".to_string())
    );
}

#[tokio::test]
async fn a_page_token_after_the_command_word_beats_one_before_it() {
    assert_eq!(
        token_for(&[], &["--cursor", "a"], &["--after", "b"]).await,
        Some("b".to_string())
    );
    assert_eq!(
        token_for(&[], &["--after", "b"], &["--cursor", "a"]).await,
        Some("a".to_string())
    );
}

#[tokio::test]
async fn a_repeated_cursor_sends_the_last() {
    assert_eq!(
        token_for(&[], &["--cursor", "a", "--cursor", "b"], &[]).await,
        Some("b".to_string())
    );
}

#[tokio::test]
async fn a_typed_spelling_silences_the_other_spellings_variable() {
    assert_eq!(
        token_for(&[("XURL_AFTER", "b")], &["--cursor", "a"], &[]).await,
        Some("a".to_string())
    );
    assert_eq!(
        token_for(&[("XURL_CURSOR", "a")], &["--after", "b"], &[]).await,
        Some("b".to_string())
    );
}

#[tokio::test]
async fn each_page_token_variable_takes_effect_alone() {
    assert_eq!(
        token_for(&[("XURL_CURSOR", "a")], &[], &[]).await,
        Some("a".to_string())
    );
    assert_eq!(
        token_for(&[("XURL_AFTER", "b")], &[], &[]).await,
        Some("b".to_string())
    );
}

#[tokio::test]
async fn an_empty_page_token_variable_is_unset() {
    assert_eq!(token_for(&[("XURL_CURSOR", "")], &[], &[]).await, None);
    assert_eq!(
        token_for(&[("XURL_CURSOR", ""), ("XURL_AFTER", "b")], &[], &[]).await,
        Some("b".to_string())
    );
}

// ── A failed parse renders in the format the same layers pick ─────────────

fn envelope_reason(output: &Output) -> String {
    envelope(output)["reason"]
        .as_str()
        .expect("a reason")
        .to_string()
}

#[test]
fn a_json_variable_renders_a_failed_parse_as_json() {
    let output = xr(&[("XURL_JSON", "true")], &["auth", "zzz"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(envelope_reason(&output), "unknown-command");
}

#[test]
fn a_jsonl_variable_renders_a_failed_parse_as_one_line() {
    let output = xr(&[("XURL_JSONL", "true")], &["auth", "zzz"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(stderr_of(&output).trim_end().lines().count(), 1);
    assert_eq!(envelope_reason(&output), "unknown-command");
}

#[test]
fn an_alias_variable_beats_xurl_output_for_a_failed_parse_and_is_not_refused() {
    let output = xr(
        &[("XURL_OUTPUT", "yaml"), ("XURL_JSONL", "1")],
        &["auth", "zzz"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert_eq!(stderr_of(&output).trim_end().lines().count(), 1);
    assert_eq!(envelope_reason(&output), "unknown-command");
}

#[test]
fn ae17_a_typed_text_renders_a_failed_parse_as_text_beside_a_format_variable() {
    let output = xr(&[("XURL_OUTPUT", "json")], &["--output", "text", "--bogus"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert!(
        stderr_of(&output).starts_with("Error: "),
        "{}",
        stderr_of(&output)
    );
}

#[test]
fn a_typed_text_after_json_renders_a_failed_parse_as_text() {
    let output = xr(&[], &["--json", "--output", "text", "--bogus"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert!(
        stderr_of(&output).starts_with("Error: "),
        "{}",
        stderr_of(&output)
    );
}

#[test]
fn a_format_after_a_double_dash_belongs_to_the_program_and_renders_text() {
    let output = xr(
        &[],
        &["webhooks", "listen", "--bogus", "--", "prog", "--json"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    assert!(
        stderr_of(&output).starts_with("Error: "),
        "{}",
        stderr_of(&output)
    );
}

#[test]
fn ae15_renders_in_the_format_xurl_output_names() {
    let output = xr(
        &[
            ("XURL_OUTPUT", "yaml"),
            ("XURL_CURSOR", "a"),
            ("XURL_AFTER", "b"),
        ],
        &["schema", "--list"],
    );

    assert_eq!(output.status.code(), Some(2), "{}", stderr_of(&output));
    let text = stderr_of(&output);
    assert!(
        text.contains("reason: invalid-args") && !text.starts_with('{'),
        "YAML: {text}"
    );
    assert!(
        text.contains("the environment variable 'XURL_CURSOR' cannot be used with the environment variable 'XURL_AFTER'"),
        "{text}"
    );
}
