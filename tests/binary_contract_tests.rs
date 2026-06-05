//! Binary-contract subprocess tests (U6 / R22).
//!
//! Pins the `xr` binary's exit-code contract and stdout/stderr stream split
//! so `src/main.rs` cannot silently drift from the library mapping in
//! `xurl::cli::run_argv`. Library behavior is covered by `cli_tests.rs` and
//! `cli_run_tests.rs`; this file exists only to catch drift at the process
//! boundary (clap-error routing, exit-code propagation, SIGPIPE restoration).

use assert_cmd::Command;

#[test]
fn help_flag_exits_zero_with_stdout() {
    let assert = Command::cargo_bin("xr")
        .expect("xr binary built")
        .arg("--help")
        .assert()
        .success()
        .code(0);
    let out = assert.get_output();
    assert!(!out.stdout.is_empty(), "stdout should be non-empty");
    assert!(
        out.stderr.is_empty(),
        "stderr should be empty for --help, got: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn version_flag_exits_zero_with_stdout() {
    // clap's `DisplayVersion` path: `--help`/`--version` write to stdout, exit 0.
    let assert = Command::cargo_bin("xr")
        .expect("xr binary built")
        .arg("--version")
        .assert()
        .success()
        .code(0);
    let out = assert.get_output();
    assert!(!out.stdout.is_empty(), "stdout should be non-empty");
}

#[test]
fn version_subcommand_exits_zero_with_stdout() {
    // Tier-1 `Version` path: routes through OutputConfig::print_message, not clap.
    let assert = Command::cargo_bin("xr")
        .expect("xr binary built")
        .arg("version")
        .assert()
        .success()
        .code(0);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.starts_with("xr "),
        "stdout should start with 'xr ': {stdout}"
    );
}

#[test]
fn bad_flag_exits_two_with_stderr() {
    let assert = Command::cargo_bin("xr")
        .expect("xr binary built")
        .arg("--bogus")
        .assert()
        .failure()
        .code(2);
    let out = assert.get_output();
    assert!(
        !out.stderr.is_empty(),
        "stderr should be non-empty for clap usage error"
    );
}

#[test]
fn missing_url_exits_one() {
    // Raw mode (no subcommand, no URL) — EXIT_GENERAL_ERROR.
    let assert = Command::cargo_bin("xr")
        .expect("xr binary built")
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("No URL provided"),
        "stderr should mention missing URL: {stderr}"
    );
}

/// SIGPIPE smoke test. `main()` restores `SIG_DFL` for SIGPIPE on Unix so a
/// short pipeline like `xr --help | head -1` exits cleanly (default SIGPIPE
/// terminates the process with signal 13) instead of panicking with
/// `BrokenPipe`. If SIGPIPE restoration drifts, we expect either a Rust panic
/// (no exit code, signal-killed via SIGABRT) or non-trivial stderr output —
/// both surface here.
#[cfg(unix)]
#[test]
fn sigpipe_smoke() {
    use std::io::Read;
    use std::process::{Command as StdCommand, Stdio};

    let bin = assert_cmd::cargo::cargo_bin("xr");
    let mut child = StdCommand::new(&bin)
        .arg("--help")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn xr --help");

    // Read only a few bytes from stdout, then drop the handle to close the
    // read end of the pipe. The child's next write triggers SIGPIPE; with
    // SIG_DFL restored, the child exits via signal 13 cleanly. Without the
    // restoration, Rust converts SIGPIPE-induced EPIPE into a `BrokenPipe`
    // panic on stdout writes.
    let mut buf = [0u8; 16];
    let mut stdout = child.stdout.take().expect("stdout pipe");
    let _ = stdout.read(&mut buf);
    drop(stdout);

    let output = child.wait_with_output().expect("wait_with_output");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Acceptable outcomes:
    //   1. Process finished writing before our reader closed → exit code 0.
    //   2. Process was killed by SIGPIPE (signal 13) → no exit code, no panic
    //      message on stderr.
    // Unacceptable: any `panicked at` / `BrokenPipe` message on stderr means
    // SIGPIPE restoration drifted.
    assert!(
        !stderr.contains("panicked at"),
        "child panicked (SIGPIPE restoration likely drifted): {stderr}"
    );
    assert!(
        !stderr.contains("BrokenPipe"),
        "child surfaced BrokenPipe (SIGPIPE restoration likely drifted): {stderr}"
    );
}
