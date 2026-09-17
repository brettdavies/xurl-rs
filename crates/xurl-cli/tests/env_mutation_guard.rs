//! Fails when an integration test mutates the process environment.
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
//! The allowlist below names the tests that must keep mutating, because they
//! prove the process read itself works. Adding an entry is a deliberate,
//! reviewable act; forgetting an attribute is not. One guard scans every
//! workspace member so the two trees cannot drift apart on separate
//! allowlists.

mod common;

use common::{
    Allowed, enclosing_test, member_dirs, stale_allowlist_entries, workspace_root,
    workspace_sources,
};

/// Every sanctioned process-environment mutation in the integration suite.
const ALLOWLIST: &[Allowed] = &[
    Allowed {
        file: "crates/xurl-cli/tests/cli_tests.rs",
        test: "test_xurl_dry_run_env_var_engages_dry_run",
        reason: "clap binds --dry-run to XURL_DRY_RUN at parse time; EnvOverrides cannot reach that binding",
    },
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
        test: "test_from_env_missing_client_id_returns_validation_error",
        reason: "Client::from_env's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/api_tests.rs",
        test: "test_from_env_empty_client_id_returns_validation_error",
        reason: "Client::from_env's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/api_tests.rs",
        test: "test_from_env_with_client_id_set_returns_ok",
        reason: "Client::from_env's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/api_tests.rs",
        test: "test_from_env_with_client_id_but_no_secret_returns_ok",
        reason: "Client::from_env's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_config_defaults",
        reason: "Config::new's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_config_from_env_client_id",
        reason: "Config::new's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_config_from_env_client_secret",
        reason: "Config::new's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_config_from_env_all",
        reason: "Config::new's contract is reading the environment",
    },
    Allowed {
        file: "crates/xdk/tests/config_tests.rs",
        test: "test_config_from_env_api_base_url",
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
        file: "crates/xurl-cli/tests/output_writer_tests.rs",
        test: "no_color_env_reaches_the_resolved_color_decision",
        reason: "the only proof that the binary reads NO_COLOR from the process",
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

/// Every source the guard reads across the workspace members: each
/// integration test file and each source file, named relative to the
/// workspace root. Guard files are skipped because their pattern strings would
/// match themselves.
fn scanned_files() -> Vec<(String, std::path::PathBuf)> {
    let root = workspace_root();
    let name_of = |path: &std::path::Path| {
        path.strip_prefix(root)
            .expect("under the workspace")
            .to_string_lossy()
            .into_owned()
    };
    let mut files = Vec::new();
    for member in member_dirs() {
        let tests = member.join("tests");
        for entry in std::fs::read_dir(&tests).expect("tests/ must be readable") {
            let path = entry.expect("dir entry").path();
            let is_guard = path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().ends_with("_guard.rs"));
            if path.extension().is_some_and(|e| e == "rs") && !is_guard {
                files.push((name_of(&path), path));
            }
        }
    }
    for path in workspace_sources() {
        files.push((name_of(&path), path));
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
fn allowlist_entries_still_exist() {
    let stale = stale_allowlist_entries(ALLOWLIST);

    assert!(
        stale.is_empty(),
        "ALLOWLIST names tests that no longer exist:\n  {}\n\n\
         Remove the entry. A stale exemption silently widens what the guard permits.",
        stale.join("\n  ")
    );
}
