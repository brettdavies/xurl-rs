//! Tests for shell completion generation.

mod common;

use predicates::prelude::*;

#[test]
fn test_completion_bash_generates_output() {
    common::xr()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xr"))
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_completion_zsh_generates_output() {
    common::xr()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xr"))
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_completion_fish_generates_output() {
    common::xr()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xr"))
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_completion_powershell_generates_output() {
    common::xr()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xr"))
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_completion_elvish_generates_output() {
    common::xr()
        .args(["completions", "elvish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xr"))
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_completion_invalid_shell_fails() {
    common::xr()
        .args(["completions", "notashell"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn test_completion_no_argument_exits_two() {
    common::xr().arg("completions").assert().failure().code(2);
}

#[test]
fn test_completions_bash_contains_subcommand_names() {
    common::xr()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("completions"))
        .stdout(predicate::str::contains("post"))
        .stdout(predicate::str::contains("usage"))
        .stdout(predicate::str::contains("auth"));
}

#[test]
fn test_completions_bash_output_is_substantial() {
    let output = common::xr()
        .args(["completions", "bash"])
        .output()
        .expect("failed to run xr completions bash");
    assert!(output.status.success());
    assert!(
        output.stdout.len() > 1024,
        "completion script should be >1KB for 29+ subcommands, got {} bytes",
        output.stdout.len()
    );
}
