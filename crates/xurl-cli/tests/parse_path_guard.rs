//! Fails when production code hands argv to clap anywhere but `parse.rs`.
//!
//! `parse.rs` removes the binding of each boolean flag whose variable reads as
//! the flag's absent value before it parses. A parse that skips it lets a
//! variable that says "off" take part in the conflict and requirement checks
//! of its flag again.
//!
//! Two groups of calls hand argv to clap: the ones that take it as an
//! argument, and the ones that read the process's own. Code that only reads
//! `Cli::command()` for its help, completions, or word lists parses no argv
//! and is not matched. Unit tests are not production code: an inline
//! `#[cfg(test)] mod name { ... }` and a file a `#[cfg(test)] mod name;`
//! declares are left out.

mod common;

use std::path::{Path, PathBuf};

use common::production_source::up_to_inline_tests;
use common::{declared_test_module, workspace_name, workspace_sources};
use regex::Regex;

const PARSE_FILE: &str = "crates/xurl-cli/src/cli/parse.rs";

/// Calls that take argv as an argument.
const GIVEN_ARGV: [&str; 6] = [
    "try_get_matches_from",
    "get_matches_from",
    "try_parse_from",
    "parse_from",
    "try_update_from",
    "update_from",
];

/// Calls that read the process's own argv.
const PROCESS_ARGV: [&str; 5] = [
    r"\.get_matches\(\)",
    r"\.try_get_matches\(\)",
    r"\.get_matches_mut\(\)",
    r"::parse\(\)",
    r"::try_parse\(\)",
];

fn given_argv() -> Regex {
    Regex::new(&format!(
        r"\b(?:{})\s*(?:::<[^>]*>)?\s*\(",
        GIVEN_ARGV.join("|")
    ))
    .expect("the pattern compiles")
}

fn process_argv() -> Regex {
    Regex::new(&PROCESS_ARGV.join("|")).expect("the pattern compiles")
}

/// The files `#[cfg(test)] mod name;` declares in `source`, which hold unit
/// tests whole.
fn declared_test_files(path: &Path, source: &str) -> Vec<PathBuf> {
    Regex::new(r"#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;")
        .expect("the pattern compiles")
        .captures_iter(source)
        .map(|module| declared_test_module(path, &module[1]))
        .collect()
}

/// Production sources as `(path, text up to its inline tests)`, with every
/// file a test-module declaration names left out.
fn production() -> Vec<(PathBuf, String)> {
    let sources: Vec<(PathBuf, String)> = workspace_sources()
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            (path, text)
        })
        .collect();
    let test_files: Vec<PathBuf> = sources
        .iter()
        .flat_map(|(path, text)| declared_test_files(path, text))
        .collect();
    sources
        .into_iter()
        .filter(|(path, _)| !test_files.contains(path))
        .map(|(path, text)| {
            let shipped = up_to_inline_tests(&text).to_string();
            (path, shipped)
        })
        .collect()
}

/// The lines of `text` that call into clap with argv, as `line: text`,
/// leaving out lines that are comments.
fn calls(text: &str, patterns: &[&Regex]) -> Vec<String> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim_start().starts_with("//"))
        .filter(|(_, line)| patterns.iter().any(|pattern| pattern.is_match(line)))
        .map(|(index, line)| format!("{}: {}", index + 1, line.trim()))
        .collect()
}

#[test]
fn only_parse_rs_hands_argv_to_clap() {
    let (given, process) = (given_argv(), process_argv());
    let files = production();
    assert!(!files.is_empty(), "the guard scanned no production file");

    let mut found = Vec::new();
    let mut parse_calls = 0;
    for (path, text) in &files {
        let name = workspace_name(path);
        let hits = calls(text, &[&given, &process]);
        if name == PARSE_FILE {
            parse_calls = calls(text, &[&given]).len();
        } else {
            found.extend(hits.into_iter().map(|hit| format!("{name}:{hit}")));
        }
    }

    assert!(
        parse_calls > 0,
        "{PARSE_FILE} holds no call that hands argv to clap, so the guard reads the wrong file"
    );
    assert!(
        found.is_empty(),
        "production code hands argv to clap outside {PARSE_FILE}, so the false-variable step \
         is skipped:\n  {}\n\nParse through `crate::cli::parse::try_matches` or `try_cli`.",
        found.join("\n  ")
    );
}
