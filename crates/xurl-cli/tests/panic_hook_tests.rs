//! A panic anywhere in `xr` answers the `internal` error in the active output
//! format and exits 1, before the release profile's abort would run.
//!
//! The hook calls `std::process::exit`, so it is proved in a child process:
//! this test binary runs itself again with `XR_PANIC_PROBE` set, and the
//! probe below installs the shipped hook and panics. The child panics in the
//! test profile, which unwinds, but the hook exits before any unwinding, as
//! it does before the abort in a release build.

mod common;

use std::process::{Command, Output};

/// Names the format the probe panics under: `json`, or `text` for a panic
/// before any format was resolved.
const PROBE: &str = "XR_PANIC_PROBE";

const ISSUE_TRACKER: &str = "https://github.com/brettdavies/xurl-rs/issues";

/// Does nothing in an ordinary run. In the child, installs the hook the
/// binary installs, writes a stdout line, and panics.
#[test]
fn panic_probe() {
    let Some(format) = std::env::var(PROBE).ok() else {
        return;
    };
    xurl::cli::panic_hook::install();
    if format == "json" {
        xurl::cli::panic_hook::publish(&xurl::cli::output::OutputConfig::new(
            xurl::cli::output::OutputFormat::Json,
            false,
            false,
            xurl::cli::ColorChoice::Never,
        ));
    }
    println!("written before the panic");
    panic!("the probe panics");
}

fn probe(format: &str) -> Output {
    Command::new(std::env::current_exe().expect("the test binary"))
        .args([
            "--exact",
            "panic_probe",
            "--nocapture",
            "--test-threads",
            "1",
        ])
        .env(PROBE, format)
        .output()
        .expect("the probe runs")
}

#[test]
fn a_panic_under_json_writes_one_internal_envelope_and_exits_1() {
    let output = probe("json");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(
        stdout.contains("written before the panic"),
        "stdout written before the panic survives: {stdout}"
    );
    let record: serde_json::Value = serde_json::from_str(stderr.trim())
        .unwrap_or_else(|e| panic!("stderr is one JSON envelope ({e}): {stderr}"));
    assert_eq!(record["status"], "error");
    assert_eq!(record["reason"], "internal");
    assert_eq!(record["exit_code"], 1);
    assert_eq!(
        record["next_step"],
        serde_json::json!({"action": "report-issue", "docs": ISSUE_TRACKER})
    );
    let message = record["message"].as_str().expect("a message");
    assert!(
        message.contains("panicked at crates/xurl-cli/tests/panic_hook_tests.rs:"),
        "Rust's wording and the location: {message}"
    );
    assert!(message.contains("the probe panics"), "{message}");
}

#[test]
fn a_panic_before_the_format_is_known_is_written_as_text() {
    let output = probe("text");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(stderr.contains("panicked at"), "{stderr}");
    assert!(
        stderr.contains(ISSUE_TRACKER),
        "names the tracker: {stderr}"
    );
    assert!(
        !stderr.trim_start().starts_with('{'),
        "text, not JSON: {stderr}"
    );
}

/// Only the binary's entry installs the hook: an in-process caller of the
/// runner keeps Rust's own panic behavior, so a panic after a run still
/// unwinds to `catch_unwind` instead of ending the process.
#[tokio::test]
async fn running_in_process_installs_no_hook() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code = xurl::cli::runner::run_with_store_path(
        ["xr", "--output", "json", "version"],
        &mut stdout,
        &mut stderr,
        &tmp.path().join(".xurl"),
    )
    .await;
    assert_eq!(code, 0);

    let caught = std::panic::catch_unwind(|| panic!("after the run"));

    assert!(caught.is_err(), "the panic unwound to catch_unwind");
}
