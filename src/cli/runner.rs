//! Library CLI entrypoints — the canonical implementation lives in
//! [`run_with_overrides`]; the others are layered wrappers, each resolving one
//! input the layer below cannot invent.
//!
//! The four entrypoints together form the public library surface for the
//! `xr` CLI dispatcher:
//!
//! - [`run_argv`]: reads `std::env::args_os()` and uses real stdio. The binary
//!   calls this from `main`.
//! - [`run`]: takes args + writers; resolves the token-store path from
//!   `XURL_TOKEN_STORE`, falling back to the default.
//! - [`run_with_store_path`]: takes an explicit token-store path; resolves the
//!   environment from the process.
//! - [`run_with_overrides`]: the canonical implementation. Takes args, writers,
//!   a token-store path, and the environment as data, reading nothing from the
//!   process. Tests use it with a `TempDir`-rooted path so they never touch the
//!   real `~/.xurl` and never depend on ambient variables.
//!
//! All four return a structured exit code per
//! [`crate::error::Error::exit_code`], matching the binary's exit-code
//! contract. They never call `process::exit`.

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::error::{ContextKind, ErrorKind};
use clap::{CommandFactory, Parser};

use crate::auth::Auth;
use crate::cli::classify::{
    Classified, classify, context_string, nearest_command, structured_intent,
    suggestion_for_rejected,
};
use crate::cli::envelope::ErrorBody;
use crate::cli::failure::Failure;
use crate::cli::output::{Diagnostics, OutputConfig, OutputFormat};
use crate::cli::{Cli, ColorChoice, Commands};
use crate::config::Config;
use crate::error::{EXIT_GENERAL_ERROR, EXIT_SUCCESS, EXIT_USAGE_ERROR};

/// What the structured rendering says when there is nothing to run.
///
/// Help text answers a person; an agent that asked for a machine-readable
/// format gets the usage error instead.
const NO_COMMAND_MESSAGE: &str =
    "No command given. Usage: xr [OPTIONS] [URL] [COMMAND]. Try 'xr --help' for more information.";

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
/// Resolves the token-store path from `XURL_TOKEN_STORE`, falling back to
/// [`Config::default_store_path`], and delegates to [`run_with_overrides`].
pub fn run<I, S>(args: I, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let overrides = crate::cli::env::from_process();
    let store_path = overrides
        .token_store
        .as_deref()
        .filter(|p| !p.is_empty())
        .map_or_else(Config::default_store_path, PathBuf::from);
    run_with_overrides(args, stdout, stderr, &store_path, &overrides)
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
/// When the un-parsed argv (or `XURL_OUTPUT`) names a structured format,
/// parse-error stderr is the canonical envelope
/// `{"status":"error","reason":"invalid-args","exit_code":2,"message":"..."}`
/// rendered in that format. Otherwise clap's default text rendering is
/// preserved. An unrecognized subcommand, and a bare word that names no
/// command, both render as `unknown-command` at the same exit code, and a
/// bare invocation prints the root help at exit 0.
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
    run_with_overrides(
        args,
        stdout,
        stderr,
        store_path,
        &crate::cli::env::from_process(),
    )
}

/// The worker entrypoint — everything [`run_with_store_path`] does, with the
/// environment supplied as data instead of read from the process.
///
/// This is the only entrypoint that reads no environment variables. The layers
/// above it exist to resolve the two inputs it cannot invent: the token-store
/// path and `overrides`. A library consumer that embeds `xr`, or a test that
/// must stay isolated from whatever else the process is doing, calls this.
///
/// Parse-error behavior matches [`run_with_store_path`], with the output
/// intent taken from `overrides` rather than `XURL_OUTPUT`.
pub fn run_with_overrides<I, S>(
    args: I,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    store_path: &Path,
    overrides: &crate::config::EnvOverrides,
) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let args_vec: Vec<OsString> = args.into_iter().map(Into::into).collect();

    let cli = match Cli::try_parse_from(args_vec.iter()) {
        Ok(cli) => cli,
        Err(e) => {
            return render_parse_error(&e, &args_vec, overrides, stdout, stderr);
        }
    };

    let out = OutputConfig::new_with_no_color(
        cli.effective_output(),
        cli.quiet,
        cli.verbose,
        cli.color,
        cli.raw,
        overrides.no_color,
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
                return crate::cli::commands::skill::run_skill(
                    cmd,
                    &out,
                    stdout,
                    overrides.home.as_deref(),
                );
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

    // ── Tier 2: Classification, ahead of every load ────────────────────
    match classify(&cli) {
        Classified::Help => {
            return if out.format.is_structured() {
                out.print_error_envelope(
                    stderr,
                    "invalid-args",
                    EXIT_USAGE_ERROR,
                    NO_COMMAND_MESSAGE,
                );
                EXIT_USAGE_ERROR
            } else {
                let _ = write!(stdout, "{}", Cli::command().render_help());
                EXIT_SUCCESS
            };
        }
        Classified::UnknownCommand(word) => {
            let suggestion = nearest_command(&word);
            return render_unknown_command(&word, suggestion.as_deref(), &out, stderr);
        }
        Classified::Raw => {}
    }

    // ── Tier 3: Everything else (needs config + auth) ──────────────────
    let mut cfg = Config::from_overrides(overrides);
    // Honour --timeout / XURL_TIMEOUT for every HTTP path: API client,
    // OAuth2 token exchange/refresh, and the `/2/users/me` lookup.
    cfg.http_timeout_secs = cli.timeout;
    let auth = Auth::new_with_store_path_and_overrides(&cfg, store_path, overrides);

    // Taken before `Auth` moves into dispatch: the recovery hint is chosen at
    // the error site, which is after the store is gone. The snapshot carries
    // presence flags and names, never a secret. `--app` is read here rather
    // than from `Auth`, because the override lands inside dispatch, and the
    // environment client id is read from the overrides rather than from the
    // resolved credential, which already falls back to the store.
    let snapshot = crate::store::snapshot::StoreSnapshot::new(
        &auth.token_store,
        cli.app.as_deref().unwrap_or_default(),
        overrides
            .client_id
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
    );
    let invocation: Vec<String> = args_vec
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let structured = out.format.is_structured();

    // The library reports its diagnostics as `tracing` events; this renderer
    // turns them into the stderr lines the flags call for, for this dispatch
    // and this thread only.
    let diagnostics = Diagnostics::new(out.clone());
    let dispatched = tracing::subscriber::with_default(diagnostics, || {
        crate::cli::commands::run(cli, &out, stdout, stderr, auth, overrides)
    });
    match dispatched {
        Ok(()) => EXIT_SUCCESS,
        Err(Failure::Emitted { exit_code }) => exit_code,
        Err(Failure::Error(e)) => {
            let code = e.exit_code();
            if carries_no_auth_method(&e) {
                let hint = crate::cli::hints::choose_hint(&snapshot, &invocation, structured);
                out.print_error_with_hint(stderr, &e, code, &hint);
            } else if let Some(hint) = crate::cli::hints::enrollment_hint(&e) {
                out.print_error_with_hint(stderr, &e, code, &hint);
            } else {
                out.print_error(stderr, &e, code);
            }
            code
        }
    }
}

/// Whether this error is the no-credentials failure a recovery hint answers.
///
/// Matched on the carried message rather than a new variant, because the
/// public error enum is exhaustively matched downstream and cannot grow one
/// in a 3.x release.
fn carries_no_auth_method(error: &crate::error::Error) -> bool {
    matches!(
        error,
        crate::error::Error::Auth(msg)
            if msg == crate::error::NO_AUTH_METHOD || msg == "TokenNotFound: oauth2 token not found"
    )
}

/// Renders a clap parse failure.
///
/// Help and version go to stdout at exit 0. An unrecognized subcommand takes
/// the unknown-command rendering, carrying clap's own suggestion where clap
/// scored one. Every other kind keeps clap's text, or the `invalid-args`
/// envelope under structured intent.
fn render_parse_error(
    error: &clap::Error,
    args: &[OsString],
    overrides: &crate::config::EnvOverrides,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let rendered = error.to_string();
    if matches!(
        error.kind(),
        ErrorKind::DisplayHelp
            | ErrorKind::DisplayVersion
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    ) {
        let _ = write!(stdout, "{rendered}");
        return EXIT_SUCCESS;
    }

    let intent = structured_intent(args, overrides.output.as_deref());
    // Quiet and verbose are unparsed here, and neither changes an error
    // envelope, so the provisional config leaves both off.
    let out = OutputConfig::new_with_no_color(
        intent.clone().unwrap_or(OutputFormat::Text),
        false,
        false,
        ColorChoice::Auto,
        false,
        overrides.no_color,
    );

    if error.kind() == ErrorKind::InvalidSubcommand
        && let Some(word) = context_string(error, ContextKind::InvalidSubcommand)
    {
        let suggestion = suggestion_for_rejected(error, args, &word);
        return render_unknown_command(&word, suggestion.as_deref(), &out, stderr);
    }

    if intent.is_some() {
        out.print_error_envelope(
            stderr,
            "invalid-args",
            EXIT_USAGE_ERROR,
            rendered.trim_end(),
        );
    } else {
        let _ = write!(stderr, "{rendered}");
    }
    EXIT_USAGE_ERROR
}

/// The one rendering both detection paths use.
///
/// Text mode gets the sentence; every structured mode gets the envelope with
/// the offending word in `command` and the nearest real name in `suggestion`,
/// which is absent when nothing scored close enough.
fn render_unknown_command(
    word: &str,
    suggestion: Option<&str>,
    out: &OutputConfig,
    stderr: &mut dyn Write,
) -> i32 {
    let message = match suggestion {
        Some(nearest) => {
            format!("unknown command '{word}'. Did you mean '{nearest}'? Try 'xr --help'.")
        }
        None => format!("unknown command '{word}'. Try 'xr --help'."),
    };
    out.emit_error_envelope(
        stderr,
        ErrorBody {
            reason: "unknown-command".to_string(),
            exit_code: EXIT_USAGE_ERROR,
            message: Some(message),
            command: Some(word.to_string()),
            suggestion: suggestion.map(str::to_string),
            ..ErrorBody::default()
        },
    );
    EXIT_USAGE_ERROR
}

// Compile-time guarantee: the canonical entrypoint signature is callable
// from any thread. The trait objects `&mut dyn Write` are not `Send` by
// themselves, but the function-pointer type below is `Send + Sync`, which
// is what library consumers need to dispatch the runner from a thread pool.
const _: fn() = || {
    fn _assert_send_sync<T: Send + Sync>() {}
    _assert_send_sync::<fn() -> i32>();
};
