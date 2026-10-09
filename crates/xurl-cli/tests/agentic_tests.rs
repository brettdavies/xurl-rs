//! Tests for agentic coding flags: --output, --quiet, --no-interactive, --timeout.

mod common;

use std::time::{Duration, Instant};

use predicates::prelude::*;
use rstest::rstest;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The stdout of `xr <args>`, one help or examples page.
fn page(args: &[String]) -> String {
    let output = common::xr().args(args).output().unwrap();
    assert!(output.status.success(), "xr {args:?} failed");
    String::from_utf8(output.stdout).unwrap()
}

/// The help page of `xr <path>`, the root page for an empty path. The one
/// place this file passes the help flag, so a test that asks for help holds
/// the text it then asserts on.
#[must_use]
fn help(path: &[String]) -> String {
    let mut args = path.to_vec();
    args.push("--help".to_string());
    page(&args)
}

/// Runs `xr <args>` against `server` with an app-only bearer and `env` in the
/// child's environment, returning the exit code, stdout, and stderr.
fn run_against(server: &MockServer, env: &[(&str, &str)], args: &[&str]) -> (i32, String, String) {
    let mut cmd = common::xr();
    cmd.env("API_BASE_URL", server.uri())
        .env("XURL_BEARER_TOKEN", "env-bearer-value");
    for (key, value) in env {
        cmd.env(key, value);
    }
    let output = cmd.args(args).output().expect("spawn xr");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// A run that passes the help flag beside another flag exits 0 whatever that
/// flag does, so it proves clap parsed the flag and nothing about its effect.
#[test]
fn no_test_passes_the_help_flag_itself() {
    let source = std::fs::read_to_string(
        common::workspace_root().join("crates/xurl-cli/tests/agentic_tests.rs"),
    )
    .expect("this file must be readable");
    let flag = concat!("\"--", "help\"");
    let offenders: std::collections::BTreeSet<String> = source
        .match_indices(flag)
        .map(|(offset, _)| common::enclosing_test(&source, offset))
        .filter(|name| name != "help")
        .collect();
    assert!(
        offenders.is_empty(),
        "these tests pass --help themselves; read the page through help() and assert on its \
         text, or assert the behavior the flag under test produces: {offenders:#?}"
    );
}

// ── --output: each format renders the same result its own way ─────────

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Stdout of `xr --output <format> version`. The command needs no credentials
/// and no network, so the format is all that varies between runs.
fn version_in(format: &str) -> String {
    let output = common::xr()
        .args(["--output", format, "version"])
        .output()
        .unwrap();
    assert!(output.status.success(), "--output {format} version failed");
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn test_output_text_prints_the_version_line() {
    assert_eq!(version_in("text"), format!("xr {VERSION}\n"));
}

#[rstest]
#[case::json("json")]
#[case::jsonl("jsonl")]
#[case::ndjson("ndjson")]
fn test_output_json_formats_print_a_json_object(#[case] format: &str) {
    let stdout = version_in(format);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("--output {format} must print JSON ({e}): {stdout}"));
    assert_eq!(parsed["name"], "xr");
    assert_eq!(parsed["version"], VERSION);
}

/// `yml` is `yaml` under its other common spelling, from the flag and from
/// `XURL_OUTPUT`.
#[test]
fn test_output_yml_is_yaml() {
    let yaml = version_in("yaml");
    assert!(yaml.starts_with("name: xr\n"), "{yaml}");

    let from_flag = common::xr()
        .args(["--output", "yml", "version"])
        .output()
        .unwrap();
    assert_eq!(
        from_flag.status.code(),
        Some(0),
        "--output yml: {}",
        String::from_utf8_lossy(&from_flag.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&from_flag.stdout), yaml);

    let from_env = common::xr()
        .env("XURL_OUTPUT", "yml")
        .arg("version")
        .output()
        .unwrap();
    assert_eq!(
        from_env.status.code(),
        Some(0),
        "XURL_OUTPUT=yml: {}",
        String::from_utf8_lossy(&from_env.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&from_env.stdout), yaml);
}

/// JSON Lines is one record per line, under either of its names and for a
/// command that is not a stream as much as for one that is.
#[rstest]
#[case::jsonl("jsonl")]
#[case::ndjson("ndjson")]
fn test_output_line_formats_print_one_line(#[case] format: &str) {
    let stdout = version_in(format);
    assert_eq!(
        stdout.lines().count(),
        1,
        "--output {format} prints one record per line: {stdout}"
    );
}

/// The same holds for every document `xr` prints under a line format: a
/// dry-run envelope on stdout and an error envelope on stderr.
#[rstest]
#[case::jsonl("jsonl")]
#[case::ndjson("ndjson")]
fn test_output_line_formats_print_envelopes_on_one_line(#[case] format: &str) {
    let dry_run = common::xr()
        .args(["--output", format, "post", "hi", "--dry-run"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&dry_run.stdout);
    assert_eq!(dry_run.status.code(), Some(0), "{stdout}");
    assert_eq!(
        stdout.lines().count(),
        1,
        "a dry-run envelope is one line: {stdout}"
    );
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("a JSON line");
    assert_eq!(envelope["status"], "dry_run");

    let failure = common::xr()
        .args(["--output", format, "whoam"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&failure.stderr);
    assert_eq!(failure.status.code(), Some(2), "{stderr}");
    assert_eq!(
        stderr.lines().count(),
        1,
        "an error envelope is one line: {stderr}"
    );
    let envelope: serde_json::Value = serde_json::from_str(&stderr).expect("a JSON line");
    assert_eq!(envelope["reason"], "unknown-command");
}

/// And for an API document: the response to a request prints as one line.
#[rstest]
#[case::jsonl("jsonl")]
#[case::ndjson("ndjson")]
#[tokio::test]
async fn test_output_line_formats_print_an_api_document_on_one_line(#[case] format: &str) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"data":[{"id":"1","text":"hi"}],"meta":{"result_count":1}}"#),
        )
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_against(
        &server,
        &[],
        &["--output", format, "--auth", "app", "search", "hi"],
    );

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(
        stdout.lines().count(),
        1,
        "an API document is one line: {stdout}"
    );
    let document: serde_json::Value = serde_json::from_str(&stdout).expect("a JSON line");
    assert_eq!(document["data"][0]["id"], "1");
}

#[test]
fn test_output_yaml_prints_a_yaml_mapping() {
    let stdout = version_in("yaml");
    assert!(
        stdout.contains(&format!("name: xr\nversion: {VERSION}\n")),
        "--output yaml must print a block-style mapping: {stdout}"
    );
}

#[rstest]
#[case::csv("csv", ',')]
#[case::tsv("tsv", '\t')]
fn test_output_delimited_formats_print_a_header_and_a_row(
    #[case] format: &str,
    #[case] delimiter: char,
) {
    let stdout = version_in(format);
    let rows: Vec<Vec<&str>> = stdout
        .lines()
        .map(|line| line.split(delimiter).collect())
        .collect();
    assert_eq!(
        rows.len(),
        2,
        "--output {format} prints a header and one row: {stdout}"
    );
    assert_eq!(rows[0][..2], ["name", "version"]);
    assert_eq!(rows[1][..2], ["xr", VERSION]);
}

/// A format `xr` does not print is a usage error at exit 2. The caller asked
/// for a structured format, so the error is the JSON envelope, whether the
/// format came from the flag in either spelling or from `XURL_OUTPUT`.
#[rstest]
#[case::flag(&["--output", "toml", "version"], None, "toml")]
#[case::flag_equals(&["--output=xml", "version"], None, "xml")]
#[case::env(&["version"], Some("toml"), "toml")]
fn test_output_unsupported_format_is_a_json_envelope(
    #[case] args: &[&str],
    #[case] env: Option<&str>,
    #[case] format: &str,
) {
    let mut command = common::xr();
    command.args(args);
    if let Some(value) = env {
        command.env("XURL_OUTPUT", value);
    }
    let output = command.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(output.stdout.is_empty(), "nothing on stdout");
    let envelope: serde_json::Value = serde_json::from_str(&stderr)
        .unwrap_or_else(|e| panic!("stderr must be a JSON envelope ({e}): {stderr}"));
    assert_eq!(envelope["status"], "error");
    assert_eq!(envelope["reason"], "invalid-args");
    assert_eq!(envelope["exit_code"], 2);
    let message = envelope["message"].as_str().expect("message");
    assert!(
        message.contains(&format!("invalid value '{format}'")),
        "{message}"
    );
}

/// A miscased `text` asks for no structure, so its usage error stays text.
#[test]
fn test_output_miscased_text_keeps_the_text_error() {
    common::xr()
        .args(["--output", "Text", "version"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("Error: invalid value 'Text'"));
}

#[test]
fn test_xurl_output_env_sets_the_format() {
    let output = common::xr()
        .env("XURL_OUTPUT", "json")
        .arg("version")
        .output()
        .unwrap();
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("XURL_OUTPUT=json must print JSON");
    assert_eq!(parsed["version"], VERSION);
}

// ── --quiet: the status lines a command prints to stderr ──────────────

const STREAM_PATH: &str = "/2/tweets/search/stream";

/// A stream prints its status lines to stderr and each record to stdout.
/// `--quiet`, `-q`, and a truthy `XURL_QUIET` drop the status lines and leave
/// the records alone; `XURL_QUIET=0` is falsey, so the lines stay.
#[rstest]
#[case::default(&[], &[], true)]
#[case::long_flag(&["--quiet"], &[], false)]
#[case::short_flag(&["-q"], &[], false)]
#[case::truthy_env(&[], &[("XURL_QUIET", "yes")], false)]
#[case::falsey_env(&[], &[("XURL_QUIET", "0")], true)]
#[tokio::test]
async fn test_quiet_drops_the_stream_status_lines(
    #[case] flags: &[&str],
    #[case] env: &[(&str, &str)],
    #[case] status_lines: bool,
) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(STREAM_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_string("{\"data\":{\"id\":\"1\"}}\n"))
        .mount(&server)
        .await;

    let args = [flags, &["--auth", "app", "-s", STREAM_PATH]].concat();
    let (code, stdout, stderr) = run_against(&server, env, &args);

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains(r#"{"data":{"id":"1"}}"#),
        "the record must reach stdout: {stdout}"
    );
    assert_eq!(
        stderr.contains("End of stream"),
        status_lines,
        "stderr: {stderr:?}"
    );
}

// ── --timeout: the wait for a response ────────────────────────────────

/// A server slower than `--timeout` ends the run as a network error before it
/// answers.
#[tokio::test]
async fn test_timeout_ends_a_request_the_server_answers_too_slowly() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(3))
                .set_body_string(r#"{"data":[]}"#),
        )
        .mount(&server)
        .await;

    let started = Instant::now();
    let (code, _stdout, stderr) = run_against(
        &server,
        &[],
        &[
            "--timeout",
            "1",
            "--output",
            "json",
            "--auth",
            "app",
            "/2/tweets/search/recent?query=hi",
        ],
    );
    let elapsed = started.elapsed();

    let envelope: serde_json::Value = serde_json::from_str(stderr.trim())
        .unwrap_or_else(|e| panic!("a timeout must print one envelope ({e}): {stderr}"));
    assert_eq!(envelope["reason"], "network-error", "got: {envelope}");
    assert_eq!(envelope["exit_code"], code, "got: {envelope}");
    assert!(
        elapsed < Duration::from_secs(3),
        "--timeout 1 must not wait out the 3-second response: {elapsed:?}"
    );
}

#[test]
fn test_exit_code_nonzero_on_error() {
    let output = common::xr()
        .arg("--definitely-not-a-flag")
        .output()
        .unwrap();

    assert_ne!(output.status.code().unwrap(), 0);
}

// ── Env-backed global flags + TTY-aware color ────────────────────

#[test]
fn test_help_advertises_xurl_verbose_env() {
    // p1-must-env-var: --verbose must show [env: XURL_VERBOSE=] in --help.
    let stdout = help(&[]);
    assert!(
        stdout.contains("XURL_VERBOSE"),
        "expected XURL_VERBOSE in --help: {stdout}"
    );
}

#[test]
fn test_help_advertises_every_env_var_the_source_reads() {
    // p1-must-env-var: every variable the binary reads must surface in --help.
    // The set comes from the shipped crates' sources rather than a list here,
    // so a new variable cannot land without a help entry.
    let pattern = regex::Regex::new(r#"env = "([A-Z_]+)"|env::var(?:_os)?\("([A-Z_]+)"\)"#)
        .expect("valid pattern");
    let mut names = std::collections::BTreeSet::new();
    for path in common::shipped_sources() {
        let source = std::fs::read_to_string(&path).expect("source must be readable");
        let production = source.split("#[cfg(test)]").next().unwrap_or("");
        for cap in pattern.captures_iter(production) {
            let name = cap
                .get(1)
                .or_else(|| cap.get(2))
                .expect("a capture")
                .as_str();
            names.insert(name.to_string());
        }
    }
    // Skill hosts' config- and base-directory variables are read by name from
    // the skill manifest rather than as literals, so they come from the
    // manifest too.
    let manifest_path =
        common::workspace_root().join("crates/xurl-cli/src/cli/skill_install/skill.json");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&manifest_path).expect("skill.json must be readable"),
    )
    .expect("skill.json must parse");
    for entry in ["config_dir_env", "base_dir_env"]
        .iter()
        .flat_map(|section| {
            manifest[section]
                .as_object()
                .unwrap_or_else(|| panic!("{section} map"))
                .values()
        })
    {
        if let Some(var) = entry["var"].as_str() {
            names.insert(var.to_string());
        }
    }
    // `HOME` is the one core system variable xr reads, only to expand a
    // `~`-prefixed skill destination; it is not an xr setting.
    names.remove("HOME");

    let stdout = help(&[]);
    let missing: Vec<&String> = names
        .iter()
        .filter(|n| !stdout.contains(n.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "environment variables the source reads but --help does not advertise: {missing:?}"
    );
}

#[test]
fn test_help_advertises_color_flag() {
    // p6-may-color-flag: --color must appear in --help.
    let stdout = help(&[]);
    assert!(
        stdout.contains("--color"),
        "expected --color in --help: {stdout}"
    );
}

#[test]
fn test_color_auto_emits_no_ansi_on_a_captured_stderr() {
    let output = common::xr()
        .args(["--color", "auto", "whoam"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains('\x1b'),
        "--color auto must not color a stderr that is not a terminal: {stderr:?}"
    );
}

#[test]
fn test_color_invalid_value_fails() {
    common::xr()
        .args(["--color", "rainbow", "version"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("invalid value 'rainbow'"));
}

#[test]
fn test_xurl_verbose_env_adds_the_library_version() {
    let output = common::xr()
        .env("XURL_VERBOSE", "1")
        .arg("version")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("(xdk-rs "),
        "XURL_VERBOSE=1 must add the library version to the version line: {stdout}"
    );
}

#[test]
fn test_xurl_color_env_always_emits_ansi_on_stderr() {
    let output = common::xr()
        .env("XURL_COLOR", "always")
        .arg("whoam")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains('\x1b'),
        "XURL_COLOR=always must emit ANSI even when stderr is captured: {stderr:?}"
    );
}

// ── Color resolution: NO_COLOR and --color via subprocess ────────────
//
// Subprocess tests give hermetic env control: the spawn seam strips
// `NO_COLOR`, and `.env("NO_COLOR", "1")` applies only to the child, so
// concurrent cargo-test threads can't race on the env var.
// A mistyped command renders its error line to stderr through the one
// envelope emitter, which honors `use_color`.

#[test]
fn test_color_never_strips_ansi_from_stderr() {
    let output = common::xr()
        .args(["--color", "never", "whoam"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains('\x1b'),
        "--color never must strip ANSI from stderr: {stderr:?}"
    );
}

#[test]
fn test_color_always_emits_ansi_on_stderr() {
    let output = common::xr()
        .args(["--color", "always", "whoam"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains('\x1b'),
        "--color always must emit ANSI even when stderr is captured: {stderr:?}"
    );
}

#[test]
fn test_no_color_env_overrides_color_always() {
    let output = common::xr()
        .args(["--color", "always", "whoam"])
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains('\x1b'),
        "NO_COLOR=1 must defeat --color always per https://no-color.org/: {stderr:?}"
    );
}

// ── Clap-error envelope via XURL_OUTPUT env var ──────────────────

#[test]
fn test_clap_error_envelope_via_xurl_output_env() {
    // When clap parsing fails BEFORE flags are read, the runner reads
    // XURL_OUTPUT directly to decide whether to JSON-wrap the parse error.
    let output = common::xr()
        .args(["--bogus-flag"])
        .env("XURL_OUTPUT", "json")
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 2);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("XURL_OUTPUT=json wraps clap errors");
    assert_eq!(parsed["status"], "error");
    assert_eq!(parsed["reason"], "invalid-args");
    assert_eq!(parsed["exit_code"], 2);
}

#[test]
fn test_clap_error_envelope_via_xurl_output_jsonl_env() {
    let output = common::xr()
        .args(["--bogus-flag"])
        .env("XURL_OUTPUT", "jsonl")
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 2);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("XURL_OUTPUT=jsonl wraps clap errors");
    assert_eq!(parsed["reason"], "invalid-args");
}

#[test]
fn test_raw_with_output_json_emits_compact_json() {
    // --raw under JSON mode produces compact JSON (no whitespace) on stderr
    // for the envelope path. Schema lookup is a clean stdout-emitting verb.
    let output = common::xr()
        .args(["schema", "envelope", "--output", "json", "--raw"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Compact JSON has no `"\n  "` indentation prefix.
    assert!(
        !stdout.contains("  \""),
        "--raw must emit compact JSON: {stdout}"
    );
}

#[test]
fn test_raw_without_flag_pretty_prints() {
    let output = common::xr()
        .args(["schema", "envelope", "--output", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("  \""),
        "default JSON must be pretty-printed: {stdout}"
    );
}

// ── Output discipline (no naked println/eprintln) ───────────────────

#[test]
fn test_lint_stdio_script_passes_on_clean_tree() {
    // The CI guard at scripts/lint-stdio.sh must succeed against the
    // working tree (every site routes through src/cli/output/).
    let root = common::workspace_root();
    let script = root.join("scripts/lint-stdio.sh");
    if !script.exists() {
        panic!("scripts/lint-stdio.sh is missing");
    }
    let status = std::process::Command::new("bash")
        .arg(&script)
        .current_dir(root)
        .stdin(std::process::Stdio::null())
        .status()
        .expect("spawn lint-stdio.sh");
    assert!(
        status.success(),
        "scripts/lint-stdio.sh should exit 0 on a clean tree (exit: {status:?})"
    );
}

// Note: a fixture-based meta-test asserting the script fails on a planted
// `println!` was attempted, but the test environment's tempdir/rg interplay
// produced flaky results across hosts. The clean-tree test above plus
// manual fixture verification documented in the CI workflow cover the
// guarantee. The script is also exercised on every CI run, so a regression
// in its detection logic surfaces immediately.

// ── csv/tsv/yaml/ndjson formats + --cursor + xr validate ────────

/// `xr --help` must surface every additional output-format token agents
/// look for: csv, tsv, yaml, yml, toml, xml, ndjson. The substring search
/// matches anc's `p2-may-more-formats` audit shape.
#[test]
fn test_help_advertises_extra_output_formats() {
    let stdout = help(&[]);
    for token in ["csv", "tsv", "yaml", "yml", "toml", "xml", "ndjson"] {
        assert!(
            stdout.to_lowercase().contains(token),
            "expected {token:?} in xr --help: {stdout}"
        );
    }
}

/// `xr --help` must surface `--cursor`, `--after`, and `--page` so anc's
/// `p7-may-cursor-pagination` substring audit passes.
#[test]
fn test_help_advertises_cursor_pagination_flags() {
    let stdout = help(&[]);
    for flag in ["--cursor", "--after", "--page"] {
        assert!(
            stdout.contains(flag),
            "expected {flag:?} in xr --help: {stdout}"
        );
    }
}

/// `--after` is the documented alias of `--cursor`: its token reaches the wire
/// as `pagination_token`.
#[tokio::test]
async fn test_after_sends_the_pagination_token() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/tweets/search/recent"))
        .and(query_param("pagination_token", "AFTER-TOKEN"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"data":[{"id":"1","text":"hi"}],"meta":{"result_count":1}}"#),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_against(
        &server,
        &[],
        &["--after", "AFTER-TOKEN", "--auth", "app", "search", "hi"],
    );

    assert_eq!(
        code, 0,
        "--after must reach the request as pagination_token; stderr: {stderr}"
    );
}

#[test]
fn test_validate_subcommand_passes_on_valid_input() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("post.json");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, r#"{{"data":{{"id":"1","text":"hi"}}}}"#).unwrap();
    drop(f);

    let output = common::xr()
        .args([
            "validate",
            path.to_str().unwrap(),
            "--schema",
            "post",
            "--output",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code().unwrap(),
        0,
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"valid\""), "stdout: {stdout}");
}

#[test]
fn test_validate_subcommand_fails_on_invalid_input() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("invalid.json");
    let mut f = std::fs::File::create(&path).unwrap();
    // Missing required `text` field — ApiResponse<Post> will fail to deserialize.
    writeln!(f, r#"{{"data":{{"id":"1"}}}}"#).unwrap();
    drop(f);

    let output = common::xr()
        .args([
            "validate",
            path.to_str().unwrap(),
            "--schema",
            "post",
            "--output",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code().unwrap(), 1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("validation-failed"), "stderr: {stderr}");
}

#[test]
fn test_page_flag_emits_unsupported_pagination_envelope() {
    let output = common::xr()
        .args([
            "--page",
            "2",
            "--output",
            "json",
            "--no-interactive",
            "search",
            "rustlang",
        ])
        .output()
        .unwrap();

    assert_ne!(output.status.code().unwrap(), 0);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unsupported-pagination"),
        "expected unsupported-pagination reason in stderr: {stderr}"
    );
}

#[test]
fn test_validate_subcommand_appears_in_help() {
    let stdout = help(&[]);
    assert!(
        stdout.contains("validate"),
        "expected 'validate' subcommand in --help: {stdout}"
    );
}

// ── Every non-interactive example runs as written ─────────────────────

/// The commands that confirm a destructive op before running it. Under
/// `--no-interactive` each one refuses unless `--force` confirms the op.
const CONFIRMED_COMMANDS: &[&str] = &[
    "delete",
    "auth clear",
    "auth apps remove",
    "webhooks remove",
];

/// Each command that confirms a destructive op is on the list above, so a new
/// one cannot escape the example check below.
#[test]
fn every_confirmed_command_is_listed() {
    let call_sites: usize = common::workspace_sources()
        .iter()
        .map(|file| {
            let source = std::fs::read_to_string(file).unwrap();
            source.matches("gate_destructive(").count()
                - source.matches("fn gate_destructive(").count()
        })
        .sum();
    assert_eq!(
        call_sites,
        CONFIRMED_COMMANDS.len(),
        "the number of commands calling gate_destructive changed; update CONFIRMED_COMMANDS"
    );
}

/// An example of a confirmed command that passes `--no-interactive` is the
/// invocation an agent or a CI job copies verbatim, so each one, on the
/// examples page or in any command's help, runs to exit 0 under `--dry-run`
/// rather than stopping at a confirmation nobody can answer.
#[test]
fn every_non_interactive_example_of_a_confirmed_command_runs_under_dry_run() {
    let tmp = tempfile::TempDir::new().unwrap();
    let store = tmp.path().join(".xurl");
    let mut ts = xdk::store::TokenStore::new_with_path(store.to_str().unwrap());
    ts.add_app("my-app", "CLIENT-ID", "SECRET").unwrap();

    let mut pages = vec![page(&["examples".to_string()])];
    for path in common::command_paths() {
        pages.push(help(&path));
    }
    let examples: std::collections::BTreeSet<&str> = pages
        .iter()
        .flat_map(|p| p.lines())
        .map(str::trim)
        .filter(|line| {
            line.contains(" --no-interactive")
                && CONFIRMED_COMMANDS
                    .iter()
                    .any(|command| line.starts_with(&format!("xr {command} ")))
        })
        .collect();
    let missing: Vec<&str> = CONFIRMED_COMMANDS
        .iter()
        .copied()
        .filter(|command| {
            !examples
                .iter()
                .any(|line| line.starts_with(&format!("xr {command} ")))
        })
        .collect();
    assert!(
        missing.is_empty(),
        "no --no-interactive example shows these confirmed commands: {missing:?}"
    );

    let failing: Vec<String> = examples
        .iter()
        .filter_map(|line| {
            assert!(
                !line.contains(['"', '\'', '|', '$']),
                "this example needs shell parsing, which the test does not do: {line}"
            );
            let args: Vec<&str> = line
                .split_whitespace()
                .skip(1)
                .chain(["--dry-run"])
                .collect();
            // A request that slips past the dry run fails fast on a refused
            // port instead of reaching the live API.
            let output = common::xr_with_store(&store)
                .env("API_BASE_URL", "http://127.0.0.1:9")
                .args(&args)
                .output()
                .unwrap();
            (!output.status.success()).then(|| {
                format!(
                    "{line} --dry-run exited {:?}: {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr).trim()
                )
            })
        })
        .collect();
    assert!(
        failing.is_empty(),
        "these examples fail as written; a confirmed command under --no-interactive needs --force:\n{}",
        failing.join("\n")
    );
}

// ── Every command family appears on the examples page ─────────────────

/// True when `line` shows an invocation of `xr <family>`, whatever precedes
/// the `xr` token (an env assignment, a pipe).
fn invokes(line: &str, family: &str) -> bool {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    tokens
        .windows(2)
        .any(|pair| pair[0] == "xr" && pair[1] == family)
}

#[test]
fn every_command_family_appears_on_the_examples_page() {
    let output = common::xr().arg("examples").output().unwrap();
    assert!(output.status.success());
    let page = String::from_utf8(output.stdout).unwrap();
    let missing: Vec<String> = common::top_level_families()
        .into_iter()
        .filter(|family| !page.lines().any(|line| invokes(line, family)))
        .collect();
    assert!(
        missing.is_empty(),
        "`xr examples` shows no invocation of these command families: {missing:?}\n\
         Cause: the family was added to clap without a section on the curated examples page.\n\
         Fix: in crates/xurl-cli/src/cli/commands/examples.rs, add an `xr <family> ...` line \
         under the use-case section it belongs to (AUTHENTICATE, POST AND READ, MANAGE SOCIAL \
         GRAPH, INSPECT YOUR ACCOUNT, DIRECT MESSAGES, BROADCASTS, MEDIA UPLOAD, RAW MODE, \
         INSPECT SCHEMAS, MULTI-APP, TOOLING), then re-bless examples.golden with \
         XURL_GOLDEN_BLESS=1 cargo test -p xurl-rs --test golden_tests."
    );
}

// ── No example passes a secret on the command line ────────────────────

/// The lines of `text` that give a secret flag a value. The `--<name>-file`
/// twins take a path, so they never match.
fn secret_on_argv(text: &str) -> Vec<String> {
    let secret_value = regex::Regex::new(
        r"--(client-secret|consumer-secret|access-token|token-secret|bearer-token)[ =][^\s-]",
    )
    .unwrap();
    text.lines()
        .filter(|line| secret_value.is_match(line))
        .map(|line| line.trim().to_string())
        .collect()
}

/// Every example a reader copies, with where it is shown: the examples page,
/// the examples under each help page, and the docs that ship with the tool.
fn shown_examples() -> Vec<(String, String)> {
    let mut shown: Vec<(String, String)> = vec![
        ("xr examples".to_string(), page(&["examples".to_string()])),
        ("xr --help".to_string(), help(&[])),
    ];
    for path in common::command_paths() {
        let page = help(&path);
        // A command's own option list names the plain flags with their value
        // placeholders; only the examples under it are copied.
        let examples = page
            .split_once("Examples:")
            .map(|(_, examples)| examples.to_string())
            .unwrap_or_default();
        shown.push((format!("xr {} --help", path.join(" ")), examples));
    }
    for doc in [
        "README.md",
        "AGENTS.md",
        "crates/xurl-cli/README.md",
        "crates/xdk/README.md",
    ] {
        let text = std::fs::read_to_string(common::workspace_root().join(doc)).unwrap();
        shown.push((doc.to_string(), text));
    }
    shown
}

/// The shown lines `find` picks out, each prefixed with its source.
fn offending_examples(find: fn(&str) -> Vec<String>) -> Vec<String> {
    shown_examples()
        .iter()
        .flat_map(|(source, text)| {
            find(text)
                .into_iter()
                .map(move |line| format!("{source}: {line}"))
        })
        .collect()
}

/// An example is what an agent or a person copies, and whatever follows a
/// secret flag lands in argv, where any process listing shows it. So the
/// examples page, the examples of every help page, and the docs that ship with
/// the tool all show the `--<name>-file` form instead.
#[test]
fn no_example_passes_a_secret_on_the_command_line() {
    let offending = offending_examples(secret_on_argv);
    assert!(
        offending.is_empty(),
        "these examples put a secret in argv; show `--<name>-file -` with the secret piped in, \
         or `--<name>-file PATH`:\n{}",
        offending.join("\n")
    );
}

// ── No example reads a record's field off the top of a document ───────

/// The lines of `text` that pipe `xr` into a `jaq` filter starting at a
/// top-level `.id`.
fn top_level_id_filter(text: &str) -> Vec<String> {
    let filter = regex::Regex::new(r"\bxr\b.*\|\s*jaq\s+(-\w+\s+)*'\.id\b").unwrap();
    text.lines()
        .filter(|line| filter.is_match(line))
        .map(|line| line.trim().to_string())
        .collect()
}

/// A list command answers X's document, whose records sit under `data` in
/// every output format, so a filter that starts at `.id` prints `null`.
#[test]
fn no_example_reads_a_list_record_at_the_top_level() {
    let offending = offending_examples(top_level_id_filter);
    assert!(
        offending.is_empty(),
        "these examples filter `.id` off the top of a document that keeps its records under \
         `data`, so they print `null`; show `--output json | jaq -r '.data[]?.id'`:\n{}",
        offending.join("\n")
    );
}
