//! Binary-contract subprocess tests.
//!
//! Pins the `xr` binary's exit-code contract and stdout/stderr stream split
//! so `src/main.rs` cannot silently drift from the library mapping in
//! `xurl::cli::run_argv`. Library behavior is covered by `cli_tests.rs` and
//! `cli_run_tests.rs`; this file exists only to catch drift at the process
//! boundary (clap-error routing, exit-code propagation, SIGPIPE restoration).

mod common;

#[test]
fn help_flag_exits_zero_with_stdout() {
    let assert = common::xr().arg("--help").assert().success().code(0);
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
    let assert = common::xr().arg("--version").assert().success().code(0);
    let out = assert.get_output();
    assert!(!out.stdout.is_empty(), "stdout should be non-empty");
}

#[test]
fn version_subcommand_exits_zero_with_stdout() {
    // Tier-1 `Version` path: routes through OutputConfig::print_message, not clap.
    let assert = common::xr().arg("version").assert().success().code(0);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.starts_with("xr "),
        "stdout should start with 'xr ': {stdout}"
    );
}

#[test]
fn version_plain_line_carries_no_library_field() {
    let assert = common::xr().arg("version").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert_eq!(
        stdout.trim(),
        format!("xr {}", env!("CARGO_PKG_VERSION")),
        "the plain line is `xr <version>` and nothing else"
    );
}

#[test]
fn version_verbose_names_the_library_version() {
    let assert = common::xr()
        .args(["version", "--verbose"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert_eq!(
        stdout.trim(),
        format!(
            "xr {} (xdk-rs {})",
            env!("CARGO_PKG_VERSION"),
            xdk::CRATE_VERSION
        )
    );
}

#[test]
fn version_json_carries_both_versions() {
    let assert = common::xr()
        .args(["version", "--output", "json"])
        .assert()
        .success();
    let value: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout)
        .expect("`xr version --output json` prints one JSON object");
    assert_eq!(value["name"], "xr");
    assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(value["xdk_rs"], xdk::CRATE_VERSION);
}

#[test]
fn bad_flag_exits_two_with_stderr() {
    let assert = common::xr().arg("--bogus").assert().failure().code(2);
    let out = assert.get_output();
    assert!(
        !out.stderr.is_empty(),
        "stderr should be non-empty for clap usage error"
    );
}

#[test]
fn bare_invocation_prints_help() {
    // Nothing to run, and nothing raw mode could send — the root help.
    let assert = common::xr().assert().success().code(0);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("Usage: xr") && stdout.contains("Commands:"),
        "stdout should carry the root help: {stdout}"
    );
}

#[test]
fn missing_url_exits_two() {
    // Raw mode with a raw-only flag and no URL is a usage error.
    let assert = common::xr().args(["-X", "POST"]).assert().failure().code(2);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("No URL provided"),
        "stderr should mention missing URL: {stderr}"
    );
}

#[test]
fn unknown_command_exits_two() {
    let assert = common::xr().arg("whoam").assert().failure().code(2);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("unknown command 'whoam'") && stderr.contains("'whoami'"),
        "stderr should name the nearest command: {stderr}"
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
    use std::process::Stdio;

    let mut child = common::xr_std()
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
    // The exit status holds whatever a panic prints: the panic hook answers
    // with the `internal` record and exit 1, which these text checks alone
    // would pass over if its wording changed.
    use std::os::unix::process::ExitStatusExt;
    #[cfg(unix)]
    let sigpipe = libc::SIGPIPE;
    assert!(
        output.status.code() == Some(0) || output.status.signal() == Some(sigpipe),
        "child neither exited 0 nor died by SIGPIPE (restoration likely drifted): {:?}; \
         stderr: {stderr}",
        output.status
    );
}

/// The signed-sample builder `listen` prints from lives outside the library's
/// `testing` feature, which pulls in `wiremock`; the release `xr` reaches it
/// without that feature, which only the dev-dependency turns on.
#[test]
fn the_release_build_enables_no_testing_feature_of_the_library() {
    let output = std::process::Command::new(env!("CARGO"))
        .args([
            "tree",
            "--offline",
            "--locked",
            "-p",
            "xurl-rs",
            "-e",
            "features,no-dev",
            "-i",
            "xdk-rs",
        ])
        .current_dir(common::workspace_root())
        .output()
        .expect("cargo tree runs");
    let tree = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(tree.contains("xdk-rs feature \"default\""), "{tree}");
    assert!(!tree.contains("feature \"testing\""), "{tree}");
}
