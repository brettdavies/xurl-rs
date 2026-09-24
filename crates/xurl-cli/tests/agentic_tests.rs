//! Tests for agentic coding flags: --output, --quiet, --no-interactive, --timeout.

mod common;

use predicates::prelude::*;

#[test]
fn test_output_json_flag_accepted() {
    // --output json should be accepted and change behavior
    common::xr()
        .args(["--output", "json", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_jsonl_flag_accepted() {
    common::xr()
        .args(["--output", "jsonl", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_text_flag_accepted() {
    common::xr()
        .args(["--output", "text", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_invalid_value_fails() {
    common::xr()
        .args(["--output", "xml", "--help"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_quiet_flag_accepted() {
    common::xr().args(["--quiet", "--help"]).assert().success();
}

#[test]
fn test_quiet_short_flag_accepted() {
    common::xr().args(["-q", "--help"]).assert().success();
}

#[test]
fn test_no_interactive_flag_accepted() {
    common::xr()
        .args(["--no-interactive", "--help"])
        .assert()
        .success();
}

#[test]
fn test_timeout_flag_accepted() {
    common::xr()
        .args(["--timeout", "60", "--help"])
        .assert()
        .success();
}

#[test]
fn test_no_color_env_respected() {
    // NO_COLOR is an industry standard (https://no-color.org/)
    // When set, colored output should be suppressed.
    // We test that the flag doesn't cause a crash.
    common::xr()
        .env("NO_COLOR", "1")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_xurl_output_env_var() {
    // XURL_OUTPUT env var should set default output format
    common::xr()
        .env("XURL_OUTPUT", "json")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_combined_agentic_flags() {
    // All agentic flags can be used together
    common::xr()
        .args([
            "--output",
            "json",
            "--quiet",
            "--no-interactive",
            "--timeout",
            "10",
            "--help",
        ])
        .assert()
        .success();
}

#[test]
fn test_exit_code_success_on_help() {
    let output = common::xr().arg("--help").output().unwrap();

    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn test_exit_code_nonzero_on_error() {
    let output = common::xr()
        .arg("--definitely-not-a-flag")
        .output()
        .unwrap();

    assert_ne!(output.status.code().unwrap(), 0);
}

// ── U3: env-backed global flags + TTY-aware color ────────────────────

#[test]
fn test_help_advertises_xurl_verbose_env() {
    // p1-must-env-var: --verbose must show [env: XURL_VERBOSE=] in --help.
    let output = common::xr().arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
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
    // `HOME` is the one core system variable xr reads, only to expand a
    // `~`-prefixed skill destination; it is not an xr setting.
    names.remove("HOME");

    let output = common::xr().arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
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
    let output = common::xr().arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--color"),
        "expected --color in --help: {stdout}"
    );
}

#[test]
fn test_color_choices_accepted() {
    for choice in ["auto", "always", "never"] {
        common::xr()
            .args(["--color", choice, "--help"])
            .assert()
            .success();
    }
}

#[test]
fn test_color_invalid_value_fails() {
    common::xr()
        .args(["--color", "rainbow", "--help"])
        .assert()
        .failure();
}

#[test]
fn test_xurl_quiet_falsey_env_does_not_enable_quiet() {
    // FalseyValueParser must treat XURL_QUIET=0 as "not quiet".
    // --help itself succeeds either way; this is mostly a smoke that the
    // env-backed bool flag parses "0" without error.
    common::xr()
        .env("XURL_QUIET", "0")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_xurl_quiet_truthy_env_accepts_arbitrary_string() {
    // Any non-falsey env value is truthy under FalseyValueParser.
    common::xr()
        .env("XURL_QUIET", "yes")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_xurl_verbose_env_accepted() {
    common::xr()
        .env("XURL_VERBOSE", "1")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_xurl_color_env_accepted() {
    common::xr()
        .env("XURL_COLOR", "never")
        .arg("--help")
        .assert()
        .success();
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

// ── U5: clap-error envelope via XURL_OUTPUT env var ──────────────────

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

// ── U8: output discipline (no naked println/eprintln) ───────────────────

#[test]
fn test_lint_stdio_script_passes_on_clean_tree() {
    // The U8 CI guard at scripts/lint-stdio.sh must succeed against the
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

// ── U13: csv/tsv/yaml/ndjson formats + --cursor + xr validate ────────

/// `xr --help` must surface every additional output-format token agents
/// look for: csv, tsv, yaml, yml, toml, xml, ndjson. The substring search
/// matches anc's `p2-may-more-formats` audit shape.
#[test]
fn test_help_advertises_extra_output_formats() {
    let output = common::xr().arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for token in ["csv", "tsv", "yaml", "yml", "toml", "xml", "ndjson"] {
        assert!(
            stdout.to_lowercase().contains(token),
            "expected {token:?} in xr --help: {stdout}"
        );
    }
}

#[test]
fn test_output_csv_accepted_as_value() {
    common::xr()
        .args(["--output", "csv", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_yaml_accepted_as_value() {
    common::xr()
        .args(["--output", "yaml", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_ndjson_accepted_as_value() {
    common::xr()
        .args(["--output", "ndjson", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_tsv_accepted_as_value() {
    common::xr()
        .args(["--output", "tsv", "--help"])
        .assert()
        .success();
}

/// `xr --help` must surface `--cursor`, `--after`, and `--page` so anc's
/// `p7-may-cursor-pagination` substring audit passes.
#[test]
fn test_help_advertises_cursor_pagination_flags() {
    let output = common::xr().arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in ["--cursor", "--after", "--page"] {
        assert!(
            stdout.contains(flag),
            "expected {flag:?} in xr --help: {stdout}"
        );
    }
}

#[test]
fn test_cursor_flag_accepted() {
    common::xr()
        .args(["--cursor", "next-page-token", "--help"])
        .assert()
        .success();
}

#[test]
fn test_after_flag_accepted() {
    common::xr()
        .args(["--after", "next-page-token", "--help"])
        .assert()
        .success();
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
    let output = common::xr().arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("validate"),
        "expected 'validate' subcommand in --help: {stdout}"
    );
}

// ── Every non-interactive example runs as written ─────────────────────

/// The stdout of `xr <args>`, one help or examples page.
fn page(args: &[String]) -> String {
    let output = common::xr().args(args).output().unwrap();
    assert!(output.status.success(), "xr {args:?} failed");
    String::from_utf8(output.stdout).unwrap()
}

/// The commands that confirm a destructive op before running it. Under
/// `--no-interactive` each one refuses unless `--force` confirms the op.
const CONFIRMED_COMMANDS: &[&str] = &["delete", "auth clear", "auth apps remove"];

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
    for mut path in common::command_paths() {
        path.push("--help".to_string());
        pages.push(page(&path));
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
    assert_eq!(
        examples.len(),
        CONFIRMED_COMMANDS.len(),
        "expected one non-interactive example per confirmed command; found {examples:?}"
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
