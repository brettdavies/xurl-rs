/// Library CLI entrypoints — the canonical implementation lives in
/// [`run_with_store_path`]; [`run`] and [`run_argv`] are layered wrappers.
///
/// The three entrypoints together form the public library surface for the
/// `xr` CLI dispatcher:
///
/// - [`run_argv`]: reads `std::env::args_os()` and uses real stdio. The binary
///   calls this from `main`.
/// - [`run`]: takes args + writers; resolves the default token-store path.
/// - [`run_with_store_path`]: the canonical implementation. Takes args, writers,
///   and an explicit token-store path. Tests pass a `TempDir`-rooted path so
///   they never touch the real `~/.xurl`.
///
/// All three return a structured exit code per `error::exit_code_for_error`,
/// matching the binary's exit-code contract. They never call `process::exit`.
use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

use crate::auth::Auth;
use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::error::{EXIT_GENERAL_ERROR, EXIT_SUCCESS, exit_code_for_error};
use crate::output::OutputConfig;

/// Clap usage-error exit code (overloaded with `EXIT_AUTH_REQUIRED` per KTD9).
const EXIT_USAGE_ERROR: i32 = 2;

/// Runs the `xr` CLI using `std::env::args_os()` and real stdio.
///
/// The binary's `main` calls this. Library consumers wanting capture should
/// call [`run`] or [`run_with_store_path`] directly.
#[must_use]
pub fn run_argv() -> i32 {
    let args: Vec<OsString> = std::env::args_os().collect();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut stdout_lock = stdout.lock();
    let mut stderr_lock = stderr.lock();
    run(args, &mut stdout_lock, &mut stderr_lock)
}

/// Runs the `xr` CLI with caller-supplied args + writers.
///
/// Resolves the default token-store path via [`Config::default_store_path`]
/// and delegates to [`run_with_store_path`].
pub fn run<I, S>(args: I, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let store_path = Config::default_store_path();
    run_with_store_path(args, stdout, stderr, &store_path)
}

/// Canonical CLI entrypoint — runs the `xr` dispatcher with explicit writers
/// and an explicit token-store path.
///
/// Library tests use this entrypoint with a `TempDir`-rooted store path to
/// stay parallel-safe (no env-var mutation, no `#[serial]`).
///
/// Clap parse errors map to exit codes as follows (per KTD3):
/// - `DisplayHelp` / `DisplayVersion` → write to `stdout`, return 0.
/// - All other kinds → write to `stderr`, return 2.
///
/// Clap output is plain text regardless of `--output json` because the
/// `OutputConfig` is not constructed yet at parse-failure time (KTD3 / F6).
pub fn run_with_store_path<I, S>(
    args: I,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    store_path: &Path,
) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            let kind = e.kind();
            let rendered = e.to_string();
            return match kind {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    let _ = write!(stdout, "{rendered}");
                    EXIT_SUCCESS
                }
                _ => {
                    let _ = write!(stderr, "{rendered}");
                    EXIT_USAGE_ERROR
                }
            };
        }
    };

    let out = OutputConfig::new(cli.output.clone(), cli.quiet, cli.verbose, cli.color);

    // ── Tier 1: Meta-commands (need only parsed args) ──────────────────
    if let Some(ref cmd) = cli.command {
        match cmd {
            Commands::Completions { shell } => {
                let mut cmd = Cli::command();
                clap_complete::generate(*shell, &mut cmd, "xr", stdout);
                return EXIT_SUCCESS;
            }
            Commands::Version => {
                let _ = writeln!(stdout, "xr {}", env!("CARGO_PKG_VERSION"));
                return EXIT_SUCCESS;
            }
            Commands::Examples => {
                return match crate::cli::commands::examples::run_examples(stdout) {
                    Ok(()) => EXIT_SUCCESS,
                    Err(e) => {
                        out.print_error(stderr, &e, EXIT_GENERAL_ERROR);
                        EXIT_GENERAL_ERROR
                    }
                };
            }
            Commands::Schema { command, list, all } => {
                return match crate::cli::commands::schema::run_schema(
                    command.as_deref(),
                    *list,
                    *all,
                    &out,
                    stdout,
                ) {
                    Ok(()) => EXIT_SUCCESS,
                    Err(e) => {
                        out.print_error(stderr, &e, EXIT_GENERAL_ERROR);
                        EXIT_GENERAL_ERROR
                    }
                };
            }
            _ => {}
        }
    }

    // ── Tier 3: Everything else (needs config + auth) ──────────────────
    let mut cfg = Config::new();
    // Honour --timeout / XURL_TIMEOUT for every HTTP path: API client,
    // OAuth2 token exchange/refresh, and the `/2/users/me` lookup.
    cfg.http_timeout_secs = cli.timeout;
    let auth = Auth::new_with_store_path(&cfg, store_path);

    match crate::cli::commands::run(cli, &out, stdout, stderr, auth) {
        Ok(()) => EXIT_SUCCESS,
        Err(e) => {
            let code = exit_code_for_error(&e);
            out.print_error(stderr, &e, code);
            code
        }
    }
}

// Compile-time guarantee: the canonical entrypoint signature is callable
// from any thread. The trait objects `&mut dyn Write` are not `Send` by
// themselves, but the function-pointer type below is `Send + Sync`, which
// is what library consumers need to dispatch the runner from a thread pool.
const _: fn() = || {
    fn _assert_send_sync<T: Send + Sync>() {}
    _assert_send_sync::<fn() -> i32>();
};
