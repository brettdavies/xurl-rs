//! Fails when a test resolves the real home directory for the token store.
//!
//! `Auth::new` and `TokenStore::new` anchor on `~/.xurl`, so a test that calls
//! them reads the developer's real login and, through any later save, races
//! every other test on that one file. The explicit-path constructors
//! (`Auth::new_with_store_path`, `TokenStore::new_with_path`,
//! `run_with_store_path`) are the seam; this guard keeps every test on it.
//!
//! A subprocess test sets `XURL_TOKEN_STORE` on the child instead of `HOME`.
//!
//! The allowlist names the tests that must touch the real path, with the
//! reason. Adding an entry is a deliberate, reviewable act.

mod common;

use std::path::{Path, PathBuf};

use common::enclosing_test;

/// A test permitted to resolve the real home directory, with the reason.
struct Allowed {
    file: &'static str,
    test: &'static str,
    reason: &'static str,
}

/// Every sanctioned real-home resolution in the suite.
const ALLOWLIST: &[Allowed] = &[Allowed {
    file: "live_smoke.rs",
    test: "live_client",
    reason: "the release gate must read the operator's real login",
}];

/// Source patterns that resolve the real home directory.
const PATTERNS: &[&str] = &[
    "Auth::new(",
    "TokenStore::new()",
    "TokenStore::with_credentials(",
    "default_store_path()",
    "default_pending_path()",
    "dirs::home_dir()",
    ".env(\"HOME\"",
];

/// Integration test files, plus library files that carry an inline test module.
fn scanned_files() -> Vec<(String, PathBuf, usize)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();

    let tests = root.join("tests");
    for entry in std::fs::read_dir(&tests).expect("tests/ must be readable") {
        let path = entry.expect("dir entry").path();
        if path.extension().is_some_and(|e| e == "rs")
            && path
                .file_name()
                .is_some_and(|n| n != "store_isolation_guard.rs")
        {
            let name = path
                .file_name()
                .expect("file name")
                .to_string_lossy()
                .into_owned();
            files.push((name, path, 0));
        }
    }

    let mut stack = vec![root.join("src")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("src/ must be readable") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let source = std::fs::read_to_string(&path).expect("source must be readable");
                if let Some(offset) = source.find("#[cfg(test)]") {
                    let name = path
                        .strip_prefix(root)
                        .expect("under root")
                        .to_string_lossy()
                        .into_owned();
                    files.push((name, path, offset));
                }
            }
        }
    }
    files
}

#[test]
fn tests_do_not_resolve_the_real_home_directory() {
    let mut violations = Vec::new();

    for (file, path, from) in scanned_files() {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let scanned = &source[from..];

        for pattern in PATTERNS {
            for (offset, _) in scanned.match_indices(pattern) {
                let test = enclosing_test(scanned, offset);
                let allowed = ALLOWLIST.iter().any(|a| a.file == file && a.test == test);
                if !allowed {
                    violations.push(format!("{file}::{test} calls {pattern}"));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "real home directory resolved in the test suite:\n  {}\n\n\
         Build the store or auth on a tempfile::TempDir path (`TokenStore::new_with_path`, \
         `Auth::new_with_store_path`, `run_with_store_path`); for a subprocess, set \
         `XURL_TOKEN_STORE` on the child instead of `HOME`. If the test exists to exercise \
         the real path, add it to ALLOWLIST in this file with the reason.",
        violations.join("\n  ")
    );
}

#[test]
fn allowlist_entries_still_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut stale = Vec::new();

    for entry in ALLOWLIST {
        let path = if entry.file.starts_with("src/") {
            root.join(entry.file)
        } else {
            root.join("tests").join(entry.file)
        };
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        if !source.contains(&format!("fn {}(", entry.test)) {
            stale.push(format!("{}::{} ({})", entry.file, entry.test, entry.reason));
        }
    }

    assert!(
        stale.is_empty(),
        "ALLOWLIST names tests that no longer exist:\n  {}",
        stale.join("\n  ")
    );
}
