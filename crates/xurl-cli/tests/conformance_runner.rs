//! Runs the differential conformance suite against the Go xurl binary.
//!
//! `XURL_ORIGINAL_BIN` names that binary. Unset, the suite skips, which is
//! what every run without Go `xurl` installed does; set, it must name an
//! executable, and the suite fails when it does not. CI's `Go parity` job
//! installs the original at a pinned commit and sets the variable.
//!
//! The original runs against a scratch home directory, because it relocates
//! the token store it finds under the real one.
//!
//! Run with: `XURL_ORIGINAL_BIN=<path to Go xurl> cargo test -p xurl-rs --test conformance_runner -- --nocapture`

mod common;
mod conformance;

use std::path::{Path, PathBuf};

use conformance::{DifferentialRunner, TestCaseFile};

/// The variable naming the Go original.
const ORIGINAL_BIN_VAR: &str = "XURL_ORIGINAL_BIN";

/// The Go original to compare against: `None` when `configured`, the value of
/// [`ORIGINAL_BIN_VAR`], is unset.
///
/// A set value is a promise that the suite runs, so one that names no
/// executable is an error instead of a skip.
fn original_bin(configured: Option<&str>) -> Result<Option<PathBuf>, String> {
    let Some(configured) = configured else {
        return Ok(None);
    };
    let path = Path::new(configured);
    if is_executable(path) {
        Ok(Some(path.to_path_buf()))
    } else {
        Err(format!(
            "{ORIGINAL_BIN_VAR} names {configured}, which is not an executable file. \
             Point it at the Go xurl binary, or unset it to skip the suite."
        ))
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[test]
fn run_differential_conformance_suite() {
    let configured = std::env::var(ORIGINAL_BIN_VAR).ok();
    let original = match original_bin(configured.as_deref()) {
        Ok(Some(original)) => original,
        Ok(None) => {
            eprintln!("SKIP: {ORIGINAL_BIN_VAR} is unset, so there is no Go xurl to compare with");
            return;
        }
        Err(problem) => panic!("{problem}"),
    };

    let toml_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("conformance")
        .join("test_cases.toml");

    let content = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", toml_path.display()));
    let cases: TestCaseFile =
        toml::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse test_cases.toml: {e}"));

    let runner = DifferentialRunner::new(&original);
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
