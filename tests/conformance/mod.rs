//! Differential conformance test runner.
//!
//! Runs the same commands against both `xurl` (Go original) and `xurl-rs`
//! (Rust port), captures stdout/stderr/exit codes, normalizes non-deterministic
//! fields, and reports differences.
//!
//! Test cases are defined declaratively in `test_cases.toml`.

use std::collections::HashMap;
use std::env;
use std::process::{Command, Output};

use serde::Deserialize;

mod normalize;

use normalize::normalize_output;

// ── Test case definition ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TestCaseFile {
    pub test: Vec<TestCase>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub compare: Vec<CompareChannel>,
    #[serde(default)]
    pub normalize: Vec<String>,
    #[serde(default)]
    pub expect_failure: bool,
    #[serde(default)]
    pub stdout_contains: Option<String>,
    #[serde(default)]
    pub json_ignore_fields: Vec<String>,
    #[serde(default)]
    pub json_ignore_order: bool,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub skip_reason: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompareChannel {
    ExitCode,
    Stdout,
    Stderr,
    StdoutJson,
}

// ── Test result ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub exit_code_match: bool,
    pub stdout_match: bool,
    pub stderr_match: bool,
    pub original_exit: Option<i32>,
    pub port_exit: Option<i32>,
    pub stdout_diff: String,
    pub skipped: bool,
    pub skip_reason: String,
}

// ── Differential runner ────────────────────────────────────────────────────

pub struct DifferentialRunner {
    original_bin: String,
    port_bin: String,
}

impl DifferentialRunner {
    pub fn new() -> Self {
        let original_bin = env::var("XURL_ORIGINAL_BIN")
            .unwrap_or_else(|_| "xurl".to_string());
        let port_bin = env::var("XURL_PORT_BIN")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_xurl-rs").to_string());

        Self {
            original_bin,
            port_bin,
        }
    }

    pub fn new_with_bins(original: &str, port: &str) -> Self {
        Self {
            original_bin: original.to_string(),
            port_bin: port.to_string(),
        }
    }

    /// Run a single test case against both binaries and compare.
    pub fn run_test(&self, case: &TestCase) -> TestResult {
        // Skip if marked
        if let Some(ref reason) = case.skip_reason {
            return TestResult {
                name: case.name.clone(),
                passed: true,
                exit_code_match: true,
                stdout_match: true,
                stderr_match: true,
                original_exit: None,
                port_exit: None,
                stdout_diff: String::new(),
                skipped: true,
                skip_reason: reason.clone(),
            };
        }

        let original_output = self.run_command(&self.original_bin, case);
        let port_output = self.run_command(&self.port_bin, case);

        let orig_exit = original_output.status.code();
        let port_exit = port_output.status.code();

        let mut exit_code_match = true;
        let mut stdout_match = true;
        let mut stderr_match = true;
        let mut stdout_diff = String::new();

        for channel in &case.compare {
            match channel {
                CompareChannel::ExitCode => {
                    exit_code_match = orig_exit == port_exit;
                }
                CompareChannel::Stdout => {
                    let orig = self.normalize_output(&original_output.stdout, &case.normalize);
                    let port = self.normalize_output(&port_output.stdout, &case.normalize);
                    stdout_match = orig == port;
                    if !stdout_match {
                        stdout_diff = format!(
                            "--- original\n+++ port\n{}\n{}",
                            String::from_utf8_lossy(&original_output.stdout),
                            String::from_utf8_lossy(&port_output.stdout)
                        );
                    }
                }
                CompareChannel::Stderr => {
                    let orig = self.normalize_output(&original_output.stderr, &case.normalize);
                    let port = self.normalize_output(&port_output.stderr, &case.normalize);
                    stderr_match = orig == port;
                }
                CompareChannel::StdoutJson => {
                    stdout_match = self.compare_json(
                        &original_output.stdout,
                        &port_output.stdout,
                        &case.json_ignore_fields,
                        case.json_ignore_order,
                    );
                    if !stdout_match {
                        stdout_diff = format!(
                            "JSON mismatch:\n--- original\n{}\n+++ port\n{}",
                            String::from_utf8_lossy(&original_output.stdout),
                            String::from_utf8_lossy(&port_output.stdout)
                        );
                    }
                }
            }
        }

        // Check stdout_contains if specified
        if let Some(ref expected) = case.stdout_contains {
            let port_stdout = String::from_utf8_lossy(&port_output.stdout);
            if !port_stdout.contains(expected.as_str()) {
                stdout_match = false;
                stdout_diff = format!(
                    "Expected stdout to contain: {expected:?}\nGot: {port_stdout}"
                );
            }
        }

        let passed = exit_code_match && stdout_match && stderr_match;

        TestResult {
            name: case.name.clone(),
            passed,
            exit_code_match,
            stdout_match,
            stderr_match,
            original_exit: orig_exit,
            port_exit: port_exit,
            stdout_diff,
            skipped: false,
            skip_reason: String::new(),
        }
    }

    fn run_command(&self, bin: &str, case: &TestCase) -> Output {
        let mut cmd = Command::new(bin);
        cmd.args(&case.args);

        for (key, value) in &case.env {
            cmd.env(key, value);
        }

        let timeout = case.timeout_secs.unwrap_or(30);
        // Note: actual timeout enforcement would use timeout(1) or similar
        cmd.output()
            .unwrap_or_else(|e| panic!("Failed to run {bin}: {e}"))
    }

    fn normalize_output(&self, output: &[u8], normalizations: &[String]) -> Vec<u8> {
        let s = String::from_utf8_lossy(output).to_string();
        let normalized = normalize_output(&s, normalizations);
        normalized.into_bytes()
    }

    fn compare_json(
        &self,
        original: &[u8],
        port: &[u8],
        ignore_fields: &[String],
        _ignore_order: bool,
    ) -> bool {
        let orig: Result<serde_json::Value, _> = serde_json::from_slice(original);
        let port: Result<serde_json::Value, _> = serde_json::from_slice(port);

        match (orig, port) {
            (Ok(mut a), Ok(mut b)) => {
                strip_fields(&mut a, ignore_fields);
                strip_fields(&mut b, ignore_fields);
                a == b
            }
            _ => original == port,
        }
    }

    /// Run all test cases from a TOML file.
    pub fn run_all(&self, cases: &[TestCase]) -> Vec<TestResult> {
        cases.iter().map(|c| self.run_test(c)).collect()
    }

    /// Print a summary report.
    pub fn print_report(results: &[TestResult]) {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed && !r.skipped).count();
        let failed = results.iter().filter(|r| !r.passed && !r.skipped).count();
        let skipped = results.iter().filter(|r| r.skipped).count();

        println!("\n=== Differential Conformance Results ===");
        println!("Total: {total}  Pass: {passed}  Fail: {failed}  Skip: {skipped}");

        for result in results {
            if result.skipped {
                println!("  SKIP: {} ({})", result.name, result.skip_reason);
            } else if result.passed {
                println!("  PASS: {}", result.name);
            } else {
                println!("  FAIL: {}", result.name);
                if !result.exit_code_match {
                    println!(
                        "    exit code: original={:?} port={:?}",
                        result.original_exit, result.port_exit
                    );
                }
                if !result.stdout_match {
                    println!("    stdout differs");
                }
                if !result.stderr_match {
                    println!("    stderr differs");
                }
                if !result.stdout_diff.is_empty() {
                    println!("    {}", result.stdout_diff);
                }
            }
        }
    }
}

/// Recursively strip fields from a JSON value.
fn strip_fields(value: &mut serde_json::Value, fields: &[String]) {
    match value {
        serde_json::Value::Object(map) => {
            for field in fields {
                map.remove(field);
            }
            for v in map.values_mut() {
                strip_fields(v, fields);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                strip_fields(v, fields);
            }
        }
        _ => {}
    }
}

// ── Integration test that loads test_cases.toml ────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_load_test_cases() {
        let toml_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("conformance")
            .join("test_cases.toml");

        if !toml_path.exists() {
            eprintln!("Skipping: test_cases.toml not found at {}", toml_path.display());
            return;
        }

        let content = std::fs::read_to_string(&toml_path).unwrap();
        let cases: TestCaseFile = toml::from_str(&content).unwrap();

        assert!(
            !cases.test.is_empty(),
            "Expected at least one test case in test_cases.toml"
        );

        for case in &cases.test {
            assert!(!case.name.is_empty(), "Test case name should not be empty");
            assert!(
                !case.args.is_empty() || case.skip_reason.is_some(),
                "Test case '{}' should have args or be skipped",
                case.name
            );
        }
    }

    #[test]
    fn test_strip_fields() {
        let mut val = serde_json::json!({
            "data": {
                "id": "123",
                "timestamp": "2024-01-01T00:00:00Z",
                "nested": {
                    "timestamp": "2024-01-01",
                    "value": 42
                }
            }
        });

        strip_fields(&mut val, &["timestamp".to_string()]);

        assert!(val["data"]["timestamp"].is_null());
        assert!(val["data"]["nested"]["timestamp"].is_null());
        assert_eq!(val["data"]["nested"]["value"], 42);
    }

    #[test]
    fn test_differential_runner_creation() {
        let runner = DifferentialRunner::new_with_bins("/usr/bin/echo", "/usr/bin/echo");
        let case = TestCase {
            name: "echo-test".to_string(),
            args: vec!["hello".to_string()],
            env: HashMap::new(),
            compare: vec![CompareChannel::ExitCode, CompareChannel::Stdout],
            normalize: vec![],
            expect_failure: false,
            stdout_contains: None,
            json_ignore_fields: vec![],
            json_ignore_order: false,
            timeout_secs: Some(5),
            skip_reason: None,
            tags: vec![],
        };

        let result = runner.run_test(&case);
        assert!(result.passed, "echo 'hello' should match against itself");
        assert!(result.exit_code_match);
        assert!(result.stdout_match);
    }
}
