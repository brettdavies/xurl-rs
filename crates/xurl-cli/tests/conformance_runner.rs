//! Runs the differential conformance suite against the Go xurl binary.
//!
//! Requires the Go binary at /home/linuxbrew/.linuxbrew/bin/xurl or
//! set XURL_ORIGINAL_BIN to point to it.
//!
//! Run with: XURL_ORIGINAL_BIN=/home/linuxbrew/.linuxbrew/bin/xurl cargo test --test conformance_runner -- --nocapture

mod common;
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

/// Unset, the variable skips the suite: most runs of this target have no Go
/// binary and say so.
#[test]
fn an_unset_original_skips_the_suite() {
    assert_eq!(original_bin(None), Ok(None));
}

/// Set, the variable is a promise that the suite runs. A path with no
/// executable behind it fails, naming the variable and the path, so a job
/// that meant to compare cannot pass by comparing nothing.
#[test]
fn an_original_that_is_not_there_fails_naming_the_variable_and_the_path() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let missing = tmp.path().join("no-such-xurl");

    let error = original_bin(Some(missing.to_str().expect("utf-8 path")))
        .expect_err("nothing is there to run");

    assert!(error.contains("XURL_ORIGINAL_BIN"), "{error}");
    assert!(
        error.contains(missing.to_str().expect("utf-8 path")),
        "{error}"
    );
}

/// A file that is there and cannot be executed is the same broken promise.
#[cfg(unix)]
#[test]
fn an_original_that_is_not_executable_fails() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let plain = tmp.path().join("xurl");
    std::fs::write(&plain, "not a program").expect("write");
    std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644)).expect("chmod");

    assert!(original_bin(Some(plain.to_str().expect("utf-8 path"))).is_err());
}

#[cfg(unix)]
#[test]
fn an_executable_original_is_the_one_the_suite_runs() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let program = tmp.path().join("xurl");
    std::fs::write(&program, "#!/bin/sh\n").expect("write");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).expect("chmod");

    assert_eq!(
        original_bin(Some(program.to_str().expect("utf-8 path"))),
        Ok(Some(program))
    );
}
