//! Fails when a test resolves the real home directory, or touches the variables
//! a process finds it and its per-user state through.
//!
//! `Auth::new` and `TokenStore::new` anchor on `~/.xurl`, so a test that calls
//! them reads the developer's real login and, through any later save, races
//! every other test on that one file. The explicit-path constructors
//! (`Auth::new_with_store_path`, `TokenStore::new_with_path`,
//! `run_with_store_path`) are the seam; this guard keeps every test on it.
//!
//! A subprocess test spawns the binary through `common::xr()`,
//! `common::xr_with_store`, or `common::xr_std_at`, which set
//! `XURL_TOKEN_STORE` on the child, and never resets `HOME`.
//!
//! A test also leaves `HOME`, `USERPROFILE` (the Windows home), `TMPDIR`,
//! `CARGO_HOME`, `RUSTUP_HOME`, and every `XDG_` directory alone: setting one on
//! a child points it at a fabricated home, removing one sends it to a fallback,
//! reading one hands the test the developer's real path, and `env_clear` or a
//! bulk `envs` does any of these at once. `dirs::` resolves the same per-user
//! directories directly.
//!
//! Every `.rs`
//! under each workspace member's `tests/` is scanned, subdirectories
//! included; `tests/common/` is the seam itself and is the one directory left
//! out. One guard covers both crates so the two trees cannot drift apart on
//! separate allowlists.
//!
//! The allowlist names the tests that must touch the real path, with the
//! reason. Adding an entry is a deliberate, reviewable act.

mod common;

use std::path::PathBuf;

use common::{
    Allowed, enclosing_test, member_dirs, rust_files, stale_allowlist_entries, workspace_root,
};

/// Every sanctioned real-home resolution in the suite.
const ALLOWLIST: &[Allowed] = &[
    Allowed {
        file: "crates/xdk/tests/live_smoke.rs",
        test: "live_client",
        reason: "the release gate must read the operator's real login",
    },
    Allowed {
        file: "crates/xurl-cli/tests/wiring_tests.rs",
        test: "test_xurl_token_store_env_selects_store_file",
        reason: "proves the variable the spawn seam relies on, so it spawns raw on purpose",
    },
    Allowed {
        file: "crates/xurl-cli/tests/cli_tests.rs",
        test: "a_base_dir_env_applies_only_while_skill_home_is_unset",
        reason: "XDG_CONFIG_HOME is the input under test; the child runs --dry-run, so it spawns nothing and writes nothing",
    },
    Allowed {
        file: "crates/xurl-cli/tests/golden_tests.rs",
        test: "capture",
        reason: "removes HOME for the home-not-set case, the one envelope that needs no home at all; the skill cases use XURL_SKILL_HOME and the store stays on XURL_TOKEN_STORE",
    },
];

/// Source patterns that resolve the real home directory or reset a child's
/// whole environment.
const PATTERNS: &[&str] = &[
    "Auth::new(",
    "TokenStore::new()",
    "TokenStore::with_credentials(",
    "default_store_path()",
    "default_pending_path()",
    "dirs::",
    "env::home_dir(",
    ".env_clear()",
    ".envs(",
    "CARGO_BIN_EXE_xr",
    "cargo_bin(\"xr\")",
    "cargo_bin!(\"xr\")",
];

/// The variables a process finds its home and per-user state through, as they
/// open a string literal. `XDG_` is a prefix, so it stays open.
const HOME_VARS: &[&str] = &[
    "\"HOME\"",
    "\"USERPROFILE\"",
    "\"TMPDIR\"",
    "\"CARGO_HOME\"",
    "\"RUSTUP_HOME\"",
    "\"XDG_",
];

/// The calls that set, remove, or read one variable. Matching the call rather
/// than the bare name leaves a test free to mention one, as in a list of names.
const ENV_CALLS: &[&str] = &[".env(", ".env_remove(", "var(", "var_os("];

/// Every pattern the guard reports: the fixed list, then each call on each
/// variable.
fn patterns() -> Vec<String> {
    let mut all: Vec<String> = PATTERNS.iter().map(|p| (*p).to_string()).collect();
    for call in ENV_CALLS {
        for var in HOME_VARS {
            all.push(format!("{call}{var}"));
        }
    }
    all
}

/// Integration test files in full, plus the test-carrying region of each
/// source file, across every workspace member. Each entry is the
/// workspace-relative name and the text to scan, read once.
fn scanned_sources() -> Vec<(String, String)> {
    let root = workspace_root();
    let read = |path: &std::path::Path| {
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
    };
    let name_of = |path: &std::path::Path| {
        path.strip_prefix(root)
            .expect("under the workspace")
            .to_string_lossy()
            .into_owned()
    };
    let mut files = Vec::new();
    let mut sources = Vec::new();

    for member in member_dirs() {
        for path in rust_files(&member.join("tests")) {
            let under_common = path
                .strip_prefix(&member)
                .expect("under the member")
                .starts_with("tests/common");
            let is_self = path
                .file_name()
                .is_some_and(|n| n == "store_isolation_guard.rs");
            if under_common || is_self {
                continue;
            }
            files.push((name_of(&path), read(&path)));
        }
        for path in rust_files(&member.join("src")) {
            let source = read(&path);
            sources.push((path, source));
        }
    }

    // A source file is scanned from its inline `#[cfg(test)]` marker, or in
    // full when its parent declares it as a test-only module.
    let test_module =
        regex::Regex::new(r"#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))? )?mod (\w+);").unwrap();
    let test_only: std::collections::BTreeSet<PathBuf> = sources
        .iter()
        .flat_map(|(path, source)| {
            let dir = path.parent().expect("a file has a parent").to_path_buf();
            let stem = path.file_stem().expect("a file has a stem").to_os_string();
            test_module
                .captures_iter(source)
                .map(move |m| {
                    let child = format!("{}.rs", &m[1]);
                    if stem == "mod" || stem == "lib" || stem == "main" {
                        dir.join(child)
                    } else {
                        dir.join(&stem).join(child)
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();
    for (path, source) in sources {
        let offset = if test_only.contains(&path) {
            Some(0)
        } else {
            source.find("#[cfg(test)]")
        };
        if let Some(offset) = offset {
            files.push((name_of(&path), source[offset..].to_string()));
        }
    }
    files
}

#[test]
fn tests_do_not_resolve_the_real_home_directory() {
    let patterns = patterns();
    let mut violations = Vec::new();

    for (file, scanned) in scanned_sources() {
        for pattern in &patterns {
            for (offset, _) in scanned.match_indices(pattern.as_str()) {
                let test = enclosing_test(&scanned, offset);
                let allowed = ALLOWLIST.iter().any(|a| a.file == file && a.test == test);
                if !allowed {
                    violations.push(format!("{file}::{test} calls {pattern}"));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "real home directory resolved, or a home variable touched, in the test suite:\n  {}\n\n\
         Build the store or auth on a tempfile::TempDir path (`TokenStore::new_with_path`, \
         `Auth::new_with_store_path`, `run_with_store_path`); spawn the binary through \
         `common::xr()` or `common::xr_with_store`, never with `HOME` or another home variable set, removed, \
         or read. If the test exists to exercise \
         the real path, add it to ALLOWLIST in this file with the reason.",
        violations.join("\n  ")
    );
}

#[test]
fn allowlist_entries_still_exist() {
    let stale = stale_allowlist_entries(ALLOWLIST);

    assert!(
        stale.is_empty(),
        "ALLOWLIST names tests that no longer exist:\n  {}",
        stale.join("\n  ")
    );
}
