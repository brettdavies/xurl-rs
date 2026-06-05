//! Library CLI entrypoints — the canonical implementation lives in
//! [`run_with_store_path`]; [`run`] and [`run_argv`] are layered wrappers.
//!
//! The three entrypoints together form the public library surface for the
//! `xr` CLI dispatcher:
//!
//! - [`run_argv`]: reads `std::env::args_os()` and uses real stdio. The binary
//!   calls this from `main`.
//! - [`run`]: takes args + writers; resolves the default token-store path.
//! - [`run_with_store_path`]: the canonical implementation. Takes args, writers,
//!   and an explicit token-store path. Tests pass a `TempDir`-rooted path so
//!   they never touch the real `~/.xurl`.
//!
//! All three return a structured exit code per
//! [`crate::error::XurlError::exit_code`], matching the binary's exit-code
//! contract. They never call `process::exit`.

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

use crate::auth::Auth;
use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::error::{EXIT_GENERAL_ERROR, EXIT_SUCCESS};
use crate::output::OutputConfig;

/// Clap usage-error exit code (sysexits `EX_USAGE`).
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
/// Clap parse errors map to exit codes as follows:
/// - `DisplayHelp` / `DisplayVersion` → write to `stdout`, return 0.
/// - All other kinds → write to `stderr`, return 2 (`EX_USAGE`).
///
/// When JSON intent is detected in the un-parsed argv (or via `XURL_OUTPUT`
/// env), parse-error stderr is the canonical agent-native envelope shape
/// `{"status":"error","reason":"invalid-args","exit_code":2,"message":"..."}`.
/// Otherwise clap's default text rendering is preserved.
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
    let args_vec: Vec<OsString> = args.into_iter().map(Into::into).collect();

    let cli = match Cli::try_parse_from(args_vec.iter()) {
        Ok(cli) => cli,
        Err(e) => {
            let kind = e.kind();
            let rendered = e.to_string();
            return match kind {
                ErrorKind::DisplayHelp
                | ErrorKind::DisplayVersion
                | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                    let _ = write!(stdout, "{rendered}");
                    EXIT_SUCCESS
                }
                _ => {
                    if json_intent(&args_vec) {
                        emit_invalid_args_envelope(stderr, &rendered);
                    } else {
                        let _ = write!(stderr, "{rendered}");
                    }
                    EXIT_USAGE_ERROR
                }
            };
        }
    };

    let out = OutputConfig::new_with_raw(
        cli.effective_output(),
        cli.quiet,
        cli.verbose,
        cli.color,
        cli.raw,
    )
    .with_no_interactive(cli.no_interactive);

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
            Commands::Schema {
                command,
                list,
                all,
                envelope,
            } => {
                return match crate::cli::commands::schema::run_schema(
                    command.as_deref(),
                    *list,
                    *all,
                    *envelope,
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
            Commands::Skill { .. } => {
                // Move out of `cli` so the handler owns the enum (avoids clone).
                let Some(Commands::Skill { cmd }) = cli.command else {
                    unreachable!("matched Commands::Skill above")
                };
                return crate::cli::commands::skill::run_skill(cmd, &out, stdout);
            }
            Commands::Validate { file, schema } => {
                return crate::cli::commands::validate::run_validate(
                    file.as_deref(),
                    schema.as_deref(),
                    &out,
                    stdout,
                    stderr,
                );
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
            let code = e.exit_code();
            // `EnvelopeAlreadyEmitted` is the U7 sentinel: the call site has
            // already written the canonical envelope (e.g.
            // `print_confirmation_required`) and we must NOT emit a second
            // `{"error":...,"kind":...}` line. The carried exit code surfaces
            // as the process exit unchanged.
            if !matches!(e, crate::error::XurlError::EnvelopeAlreadyEmitted { .. }) {
                out.print_error(stderr, &e, code);
            }
            code
        }
    }
}

/// Detects whether the caller asked for JSON output before clap parsing
/// completed. Inspected on the unparsed argv (because clap's error path
/// runs before `Cli` is constructed) and on `XURL_OUTPUT`.
///
/// Triggers on:
/// - `--json`
/// - `--jsonl`
/// - `--output json` / `--output jsonl`
/// - `--output=json` / `--output=jsonl`
/// - `XURL_OUTPUT=json` / `XURL_OUTPUT=jsonl` (case-insensitive)
fn json_intent(args: &[OsString]) -> bool {
    let mut iter = args.iter().peekable();
    while let Some(a) = iter.next() {
        let s = a.to_string_lossy();
        if s == "--json" || s == "--jsonl" {
            return true;
        }
        if s == "--output"
            && let Some(next) = iter.peek()
        {
            let v = next.to_string_lossy();
            if v.eq_ignore_ascii_case("json") || v.eq_ignore_ascii_case("jsonl") {
                return true;
            }
        }
        if let Some(rest) = s.strip_prefix("--output=")
            && (rest.eq_ignore_ascii_case("json") || rest.eq_ignore_ascii_case("jsonl"))
        {
            return true;
        }
    }
    std::env::var("XURL_OUTPUT")
        .map(|v| v.eq_ignore_ascii_case("json") || v.eq_ignore_ascii_case("jsonl"))
        .unwrap_or(false)
}

/// Writes the canonical `invalid-args` envelope to `stderr`.
fn emit_invalid_args_envelope(stderr: &mut dyn Write, clap_msg: &str) {
    let envelope = serde_json::json!({
        "status": "error",
        "reason": "invalid-args",
        "exit_code": EXIT_USAGE_ERROR,
        "message": clap_msg.trim_end().to_string(),
    });
    let _ = writeln!(stderr, "{envelope}");
}

// Compile-time guarantee: the canonical entrypoint signature is callable
// from any thread. The trait objects `&mut dyn Write` are not `Send` by
// themselves, but the function-pointer type below is `Send + Sync`, which
// is what library consumers need to dispatch the runner from a thread pool.
const _: fn() = || {
    fn _assert_send_sync<T: Send + Sync>() {}
    _assert_send_sync::<fn() -> i32>();
};
