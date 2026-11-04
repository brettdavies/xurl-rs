//! Runs the harness behind `preflight.sh oauth2-interactive` against the built
//! `xr`, so a change that breaks the interactive sign-in or the harness fails on
//! its PR rather than at the release.
//!
//! Linux only: the harness needs util-linux `script(1)`, and a fake `xdg-open`
//! stands in for the browser only where the `open` crate looks for one.
#![cfg(target_os = "linux")]

mod common;

use std::process::Command;

const HARNESS: &str = "scripts/release/oauth2-interactive.py";
const MODES: [&str; 5] = ["decoded", "encoded", "refused", "denied", "idle"];

fn runs(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

#[test]
fn every_mode_passes_against_the_built_binary() {
    if !(runs("script") && runs("python3")) {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI exercises this gate nowhere else, so its runner needs script(1) and python3"
        );
        eprintln!("skipped: the harness needs script(1) and python3 on PATH");
        return;
    }
    let output = Command::new("python3")
        .args(["-B", "-I"])
        .arg(common::workspace_root().join(HARNESS))
        .arg(common::xr_bin())
        .output()
        .expect("spawn the harness");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let passed: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("PASS\t")?.split('\t').next())
        .collect();
    assert_eq!(passed, MODES, "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
