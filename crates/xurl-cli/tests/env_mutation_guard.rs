//! Fails when an integration test mutates the process environment, or reads
//! it through a constructor that consults the process.
//!
//! Tests in one binary share a process, and `run_at` and its siblings execute
//! the CLI in-process, so a `set_var` in one test reaches every other test's
//! run. That is a data race the compiler cannot see: `std::env::set_var` is
//! `unsafe` in edition 2024 precisely because a concurrent read is undefined
//! behavior.
//!
//! Annotating readers `#[serial_test::parallel]` only ordered the scheduler
//! around the mutation. This guard removes the class instead: a test supplies
//! its environment through `EnvOverrides` or passes the flag that carries the
//! same value, so nothing needs ordering.
//!
//! The mutation is contained only when nothing else in its binary reads the
//! process: `#[serial]` orders the mutating tests against one another, not
//! against a test that builds an `Auth` or a `Config` from the process
//! meanwhile. So every other test builds through the explicit constructors
//! with `EnvOverrides`, which also keeps a variable exported in the
//! developer's shell out of the suite.
//!
//! The allowlists below name the tests that must keep mutating or reading,
//! because they prove the process read itself works. Adding an entry is a
//! deliberate, reviewable act; forgetting an attribute is not. One guard
//! scans every workspace member so the two trees cannot drift apart on
//! separate allowlists.

mod common;

use common::{
    Allowed, enclosing_test, member_dirs, rust_files, stale_allowlist_entries, workspace_name,
    workspace_sources,
};

/// Every sanctioned process-environment mutation in the integration suite.
const ALLOWLIST: &[Allowed] = &[
    Allowed {
        file: "crates/xdk/tests/auth_tests.rs",
        test: "test_redirect_uri_env_wins_via_new_with_store_path",
        reason: "proves the env-reading Auth constructor shim still reads the process",
    },
    Allowed {
        file: "crates/xdk/tests/auth_tests.rs",
        test: "get_bearer_token_header_reads_real_env",
        reason: "proves XURL_BEARER_TOKEN reaches Auth through the process-reading path",
    },
    Allowed {
        file: "crates/xdk/tests/api_tests.rs",
        test: "test_from_env_without_client_id_builds_a_store_backed_client",
        reason: "Client::from_env's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_config_defaults",
        reason: "Config::new's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_config_from_env_all",
        reason: "Config::new's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_new_matches_from_overrides_for_every_value",
        reason: "the edge proof that the process path and the injected path agree",
    },
    Allowed {
        file: "crates/xurl-cli/tests/conformance_runner.rs",
        test: "run_differential_conformance_suite",
        reason: "sole test in its binary; the variable points a comparison harness at the Go binary, not the CLI under test",
    },
    Allowed {
        file: "crates/xdk/src/config/mod.rs",
        test: "resolve_redirect_uri_env_wins",
        reason: "proves the resolve_redirect_uri wrapper reads REDIRECT_URI from the process",
    },
    Allowed {
        file: "crates/xdk/src/config/mod.rs",
        test: "resolve_redirect_uri_stored_when_no_env",
        reason: "the wrapper's unset-variable leg, which needs the variable actually unset",
    },
    Allowed {
        file: "crates/xdk/src/config/mod.rs",
        test: "resolve_redirect_uri_default_fallback",
        reason: "the wrapper's no-env no-stored leg, which needs the variable actually unset",
    },
];

/// Every sanctioned process-environment read in the integration suite.
const READ_ALLOWLIST: &[Allowed] = &[
    Allowed {
        file: "crates/xdk/tests/auth_tests.rs",
        test: "test_redirect_uri_env_wins_via_new_with_store_path",
        reason: "proves the env-reading Auth constructor shim still reads the process",
    },
    Allowed {
        file: "crates/xdk/tests/auth_tests.rs",
        test: "get_bearer_token_header_reads_real_env",
        reason: "proves XURL_BEARER_TOKEN reaches Auth through the process-reading path",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_config_defaults",
        reason: "Config::new's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_config_from_env_all",
        reason: "Config::new's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_config_default_trait",
        reason: "Config's Default delegates to Config::new, which reads the environment",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_new_matches_from_overrides_for_every_value",
        reason: "the edge proof that the process path and the injected path agree",
    },
    Allowed {
        file: "crates/xdk/tests/live_smoke.rs",
        test: "live_client",
        reason: "the release gate runs as the operator, with the operator's configuration",
    },
];

/// The calls that read the process environment, each as it opens.
/// `store_isolation_guard` already refuses `Auth::new` and
/// `Client::from_env`, which read it too.
const PROCESS_READS: &[&str] = &[
    "Auth::new_with_store_path(",
    "Config::default()",
    "Config::new()",
    "EnvOverrides::from_env()",
    "env::from_process(",
];

/// Every source the guard reads across the workspace members: each
/// integration test file and each source file, named relative to the
/// workspace root. Guard files are skipped because their pattern strings would
/// match themselves.
fn scanned_files() -> Vec<(String, std::path::PathBuf)> {
    let mut files = Vec::new();
    for member in member_dirs() {
        for path in rust_files(&member.join("tests")) {
            let is_guard = path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().ends_with("_guard.rs"));
            if !is_guard {
                files.push((workspace_name(&path), path));
            }
        }
    }
    for path in workspace_sources() {
        files.push((workspace_name(&path), path));
    }
    files.sort();
    files
}

#[test]
fn integration_tests_do_not_mutate_the_process_environment() {
    let mut violations = Vec::new();

    for (file, path) in scanned_files() {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

        for pattern in ["env::set_var(", "env::remove_var("] {
            for (offset, _) in source.match_indices(pattern) {
                let test = enclosing_test(&source, offset);
                let allowed = ALLOWLIST.iter().any(|a| a.file == file && a.test == test);
                if !allowed {
                    violations.push(format!("{file}::{test} calls {pattern}"));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "process-environment mutation found in the integration suite:\n  {}\n\n\
         Supply the value through `EnvOverrides` and the injected entrypoint, or pass the \
         flag that carries it. If the test exists to prove the process read itself, add it to \
         ALLOWLIST in this file with the reason.",
        violations.join("\n  ")
    );
}

#[test]
fn integration_tests_do_not_read_the_process_environment() {
    let mut violations = Vec::new();

    for member in member_dirs() {
        for path in rust_files(&member.join("tests")) {
            let file = workspace_name(&path);
            if file.ends_with("_guard.rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            for pattern in PROCESS_READS {
                for (offset, _) in source.match_indices(pattern) {
                    let line_start = source[..offset].rfind('\n').map_or(0, |i| i + 1);
                    let in_comment = source[line_start..offset].trim_start().starts_with("//");
                    // `OutputConfig::default()` and the like are other types.
                    let part_of_longer_name = source[..offset]
                        .chars()
                        .next_back()
                        .is_some_and(|c| c.is_alphanumeric() || c == '_');
                    if in_comment || part_of_longer_name {
                        continue;
                    }
                    let test = enclosing_test(&source, offset);
                    let allowed = READ_ALLOWLIST
                        .iter()
                        .any(|a| a.file == file && a.test == test);
                    if !allowed {
                        violations.push(format!("{file}::{test} calls {pattern}"));
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "process-environment read found in the integration suite:\n  {}\n\n\
         Build through the explicit constructor and pass `EnvOverrides`: \
         `Auth::new_with_store_path_and_overrides(cfg, path, &EnvOverrides::default())`, \
         `Config::from_overrides(&EnvOverrides::default())`, or the values the test names. \
         If the test exists to prove the process read itself, add it to READ_ALLOWLIST in \
         this file with the reason.",
        violations.join("\n  ")
    );
}

#[test]
fn allowlist_entries_still_exist() {
    let stale: Vec<String> = stale_allowlist_entries(ALLOWLIST)
        .into_iter()
        .chain(stale_allowlist_entries(READ_ALLOWLIST))
        .collect();

    assert!(
        stale.is_empty(),
        "an allowlist names tests that no longer exist:\n  {}\n\n\
         Remove the entry. A stale exemption silently widens what the guard permits.",
        stale.join("\n  ")
    );
}
