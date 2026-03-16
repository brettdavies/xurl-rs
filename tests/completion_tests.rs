//! Tests for shell completion generation.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_completion_bash_generates_output() {
    Command::cargo_bin("xurl")
        .unwrap()
        .args(["--generate-completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xurl"))
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_completion_zsh_generates_output() {
    Command::cargo_bin("xurl")
        .unwrap()
        .args(["--generate-completion", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xurl"))
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_completion_fish_generates_output() {
    Command::cargo_bin("xurl")
        .unwrap()
        .args(["--generate-completion", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xurl"))
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_completion_powershell_generates_output() {
    Command::cargo_bin("xurl")
        .unwrap()
        .args(["--generate-completion", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xurl"))
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_completion_elvish_generates_output() {
    Command::cargo_bin("xurl")
        .unwrap()
        .args(["--generate-completion", "elvish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xurl"))
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_completion_invalid_shell_fails() {
    Command::cargo_bin("xurl")
        .unwrap()
        .args(["--generate-completion", "notashell"])
        .assert()
        .failure();
}
