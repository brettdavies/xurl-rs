//! Runs the differential conformance suite against the Go xurl binary.
//!
//! Requires the Go binary at /home/linuxbrew/.linuxbrew/bin/xurl or
//! set XURL_ORIGINAL_BIN to point to it.
//!
//! Run with: XURL_ORIGINAL_BIN=/home/linuxbrew/.linuxbrew/bin/xurl cargo test --test conformance_runner -- --nocapture

mod conformance;

use conformance::{DifferentialRunner, TestCaseFile};

#[test]
fn run_differential_conformance_suite() {
    let original = std::env::var("XURL_ORIGINAL_BIN").ok();
    if original.is_none() {
        // Try known path
        let known_path = "/home/linuxbrew/.linuxbrew/bin/xurl";
        if !std::path::Path::new(known_path).exists() {
            eprintln!(
                "SKIP: Go xurl binary not found. Set XURL_ORIGINAL_BIN or install at {known_path}"
            );
            return;
        }
        unsafe {
            std::env::set_var("XURL_ORIGINAL_BIN", known_path);
        }
    }

    let toml_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("conformance")
        .join("test_cases.toml");

    let content = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", toml_path.display()));
    let cases: TestCaseFile =
        toml::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse test_cases.toml: {e}"));

    let runner = DifferentialRunner::new();
    let results = runner.run_all(&cases.test);

    DifferentialRunner::print_report(&results);

    let failures: Vec<_> = results.iter().filter(|r| !r.passed && !r.skipped).collect();
    if !failures.is_empty() {
        let names: Vec<_> = failures.iter().map(|r| r.name.as_str()).collect();
        panic!(
            "{} conformance test(s) failed: {}",
            failures.len(),
            names.join(", ")
        );
    }

    let passed = results.iter().filter(|r| r.passed && !r.skipped).count();
    let skipped = results.iter().filter(|r| r.skipped).count();
    eprintln!("\nConformance: {passed} passed, {skipped} skipped, 0 failed");
}
