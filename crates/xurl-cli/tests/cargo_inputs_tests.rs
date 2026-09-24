//! `scripts/cargo-inputs.sh` decides which changed paths can reach cargo. The
//! pre-push hook and CI skip their Rust battery when it prints nothing, so a
//! document a test reads must never be classified as inert.
//!
//! The cases live in `fixtures/cargo_inputs/paths.tsv` rather than here: the
//! script collects the names of documents Rust reads from string literals
//! under `crates/`, so a path spelled in this file would count as read.
#![cfg(unix)]

mod common;

use std::io::Write;
use std::process::{Command, Stdio};

const SCRIPT: &str = "scripts/cargo-inputs.sh";
const CASES: &str = "crates/xurl-cli/tests/fixtures/cargo_inputs/paths.tsv";

/// Every `(path, is_input)` case in the fixture, in file order.
fn cases() -> Vec<(String, bool)> {
    let text = std::fs::read_to_string(common::workspace_root().join(CASES))
        .unwrap_or_else(|e| panic!("read {CASES}: {e}"));
    text.lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (path, verdict) = line
                .split_once('\t')
                .unwrap_or_else(|| panic!("{CASES}: no tab in {line:?}"));
            let is_input = match verdict {
                "input" => true,
                "inert" => false,
                other => panic!("{CASES}: unknown verdict {other:?} for {path}"),
            };
            (path.to_string(), is_input)
        })
        .collect()
}

/// The subset of `paths` the script prints.
fn cargo_inputs(paths: &[String]) -> Vec<String> {
    let root = common::workspace_root();
    assert!(root.join(SCRIPT).exists(), "{SCRIPT} is missing");
    let mut child = Command::new("bash")
        .arg(root.join(SCRIPT))
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn {SCRIPT}: {e}"));
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(format!("{}\n", paths.join("\n")).as_bytes())
        .expect("write paths");
    let output = child.wait_with_output().expect("wait for the script");
    assert!(
        output.status.success(),
        "{SCRIPT} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("stdout is UTF-8")
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn each_path_gets_its_verdict() {
    let cases = cases();
    assert!(
        cases.iter().any(|(_, input)| *input) && cases.iter().any(|(_, input)| !*input),
        "{CASES} must hold both verdicts"
    );
    for (path, expected) in &cases {
        let printed = !cargo_inputs(std::slice::from_ref(path)).is_empty();
        assert_eq!(
            printed,
            *expected,
            "{path}: expected {}",
            if *expected { "a cargo input" } else { "inert" }
        );
    }
}

#[test]
fn a_mixed_change_keeps_its_cargo_inputs_in_input_order() {
    let paths: Vec<String> = cases().into_iter().map(|(path, _)| path).collect();
    let expected: Vec<String> = cases()
        .into_iter()
        .filter(|(_, input)| *input)
        .map(|(path, _)| path)
        .collect();
    assert_eq!(cargo_inputs(&paths), expected);
}

#[test]
fn a_documentation_only_change_prints_nothing() {
    let inert: Vec<String> = cases()
        .into_iter()
        .filter(|(_, input)| !*input)
        .map(|(path, _)| path)
        .collect();
    assert!(cargo_inputs(&inert).is_empty());
}
