//! The rulesets require each `Tests` job by name, and the reusable workflow
//! names those jobs `Tests (k/N)` from `ci.yml`'s `test_partitions`. A count
//! the rulesets do not match leaves required checks that no run reports, so
//! every merge waits on them, while the jobs that do run are required by none.

mod common;

use regex::Regex;
use serde_json::Value;

const WORKFLOW: &str = ".github/workflows/ci.yml";
const RULESETS: [&str; 2] = [
    ".github/rulesets/protect-dev.json",
    ".github/rulesets/protect-main.json",
];
const TESTS_CHECK: &str = "ci / Tests";

fn read(path: &str) -> String {
    std::fs::read_to_string(common::workspace_root().join(path))
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// The one `test_partitions` value `ci.yml` passes to the reusable workflow.
fn partitions(workflow: &str) -> usize {
    let line = Regex::new(r"(?m)^\s*test_partitions:\s*(\d+)\s*$").expect("valid regex");
    let counts: Vec<usize> = line
        .captures_iter(workflow)
        .map(|c| c[1].parse().expect("the pattern matches digits only"))
        .collect();
    match counts.as_slice() {
        [count] => *count,
        _ => panic!(
            "{WORKFLOW} sets `test_partitions` {} times; this guard reads exactly one",
            counts.len()
        ),
    }
}

/// Every required status check in `ruleset` whose name starts with `ci / Tests`.
fn required_tests_checks(ruleset: &Value) -> Vec<String> {
    let mut found: Vec<String> = ruleset["rules"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|rule| rule["type"] == "required_status_checks")
        .flat_map(|rule| {
            rule["parameters"]["required_status_checks"]
                .as_array()
                .into_iter()
                .flatten()
        })
        .filter_map(|check| check["context"].as_str())
        .filter(|context| context.starts_with(TESTS_CHECK))
        .map(str::to_string)
        .collect();
    found.sort();
    found
}

#[test]
fn each_ruleset_requires_every_tests_job_ci_runs() {
    let count = partitions(&read(WORKFLOW));
    let expected: Vec<String> = (1..=count)
        .map(|k| format!("{TESTS_CHECK} ({k}/{count})"))
        .collect();
    for path in RULESETS {
        let ruleset: Value =
            serde_json::from_str(&read(path)).unwrap_or_else(|e| panic!("{path} is not JSON: {e}"));
        let found = required_tests_checks(&ruleset);
        assert_eq!(
            found, expected,
            "{path} requires {found:?}, but {WORKFLOW} runs `test_partitions: {count}`, whose jobs \
             report {expected:?}. Make the file's `{TESTS_CHECK}` entries match, then apply it to \
             the live ruleset with `gh api -X PUT` before the change merges."
        );
    }
}
