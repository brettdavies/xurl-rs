//! Runs `preflight.sh oauth2-interactive` against the built `xr`, so a change
//! that breaks the interactive sign-in, the harness, or the gate fails on its PR
//! rather than at the release.
//!
//! Linux only: the harness needs util-linux `script(1)`, and a fake `xdg-open`
//! stands in for the browser only where the `open` crate looks for one.
#![cfg(target_os = "linux")]

mod common;

use std::process::Command;

const GATE: &str = "scripts/release/preflight.sh";
const MODES: [&str; 5] = ["decoded", "encoded", "refused", "denied", "idle"];

fn runs(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

#[test]
fn the_gate_passes_every_mode_against_the_built_binary() {
    if !(runs("script") && runs("python3")) {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI exercises this gate nowhere else, so its runner needs script(1) and python3"
        );
        eprintln!("skipped: the harness needs script(1) and python3 on PATH");
        return;
    }
    let output = Command::new("bash")
        .arg(common::workspace_root().join(GATE))
        .arg("oauth2-interactive")
        .env("BIN_PATH", common::xr_bin())
        // A comparison binary that cannot run keeps the result off whatever
        // Homebrew `xr` this machine has, and off that binary's run time.
        .env("PREVIOUS_BIN_PATH", "/nonexistent")
        .output()
        .expect("spawn the gate");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let passed: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("✓ ")?.split(':').next())
        .collect();
    assert_eq!(passed, MODES, "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
