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
//! [`xdk::error::Error::exit_code`], matching the binary's exit-code
//! contract. They never call `process::exit`.

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::error::{ContextKind, ErrorKind};
use clap::{CommandFactory, Parser};
use tracing::instrument::WithSubscriber;

use crate::cli::classify::{
    Classified, ROOT_COMMAND, classify, context_string, help_command_precedes, nearest_command,
    structured_intent, suggestion_for_rejected,
};
use crate::cli::envelope::ErrorBody;
use crate::cli::failure::Failure;
use crate::cli::hints::NextStep;
use crate::cli::output::{Diagnostics, OutputConfig, OutputFormat};
use crate::cli::reparse::{color_choice, failing_command, parse_without_help};
use crate::cli::{Cli, Commands};
use xdk::auth::Auth;
use xdk::config::Config;
use xdk::error::{EXIT_GENERAL_ERROR, EXIT_SUCCESS, EXIT_USAGE_ERROR};

/// What the structured rendering says when there is nothing to run.
///
/// Help text answers a person; an agent that asked for a machine-readable
/// format gets the usage error instead.
const NO_COMMAND_MESSAGE: &str =
    "No command given. Usage: xr [OPTIONS] [URL] [COMMAND]. Try 'xr --help'.";

/// How clap closes a failure, naming no command.
const CLAP_HELP_FOOTER: &str = "For more information, try '--help'.";

/// Runs the `xr` CLI using `std::env::args_os()` and real stdio.
///
/// The binary's `main` calls this. Library consumers wanting capture should
/// call [`run`] or [`run_with_store_path`] directly.
#[must_use]
pub async fn run_argv() -> i32 {
    let args: Vec<OsString> = std::env::args_os().collect();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut stdout_lock = stdout.lock();
    let mut stderr_lock = stderr.lock();
    run(args, &mut stdout_lock, &mut stderr_lock).await
}

/// Runs the `xr` CLI with caller-supplied args + writers.
///
/// Resolves the token-store path from `XURL_TOKEN_STORE`, falling back to
/// [`Config::default_store_path`], and delegates to [`run_with_overrides`].
pub async fn run<I, S>(args: I, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32
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
    run_with_overrides(args, stdout, stderr, &store_path, &overrides).await
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
/// rendered in that format. Otherwise it is the `Error:` line every error
/// takes, closing on the failing command's help. An unrecognized subcommand,
/// and a bare word that names no command, both render as `unknown-command` at
/// the same exit code, with or without a help flag, and a bare invocation
/// prints the root help at exit 0.
pub async fn run_with_store_path<I, S>(
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
    .await
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
pub async fn run_with_overrides<I, S>(
    args: I,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    store_path: &Path,
    overrides: &xdk::config::EnvOverrides,
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
            // The plain line is a pinned contract that scripts parse; the library
            // version rides only on the verbose and structured forms, so a bug
            // report can name both without the plain line gaining a field.
            Commands::Version => {
                let cli_version = env!("CARGO_PKG_VERSION");
                if out.format.is_structured() {
                    out.print_response(
                        stdout,
                        &serde_json::json!({
                            "name": "xr",
                            "version": cli_version,
                            "xdk_rs": xdk::CRATE_VERSION,
                        }),
                    );
                } else if out.verbose {
                    let _ = writeln!(stdout, "xr {cli_version} (xdk-rs {})", xdk::CRATE_VERSION);
                } else {
                    let _ = writeln!(stdout, "xr {cli_version}");
                }
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
            return render_unknown_command(
                &word,
                suggestion.as_deref(),
                ROOT_COMMAND,
                &out,
                stderr,
            );
        }
        Classified::Raw => {}
    }

    // ── Tier 3: Everything else (needs config + auth) ──────────────────
    let mut cfg = Config::from_overrides(overrides);
    // Honour --timeout / XURL_TIMEOUT for every HTTP path: API client,
    // OAuth2 token exchange/refresh, and the `/2/users/me` lookup.
    cfg.http_timeout_secs = cli.timeout;
    // Store loading and redirect-URI resolution warn through `tracing` too,
    // and they run before the dispatch future the renderer below is attached
    // to, so the same renderer covers them for the duration of this call.
    let auth = tracing::subscriber::with_default(Diagnostics::new(out.clone()), || {
        Auth::new_with_store_path_and_overrides(&cfg, store_path, overrides)
    });

    // Taken before `Auth` moves into dispatch: the recovery hint is chosen at
    // the error site, which is after the store is gone. The snapshot carries
    // presence flags and names, never a secret. `--app` is read here rather
    // than from `Auth`, because the override lands inside dispatch, and the
    // environment client id is read from the overrides rather than from the
    // resolved credential, which already falls back to the store.
    let snapshot = xdk::store::snapshot::StoreSnapshot::new(
        &auth.token_store,
        cli.app.as_deref().unwrap_or_default(),
        overrides
            .client_id
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
    );
    let structured = out.format.is_structured();

    // The library reports its diagnostics as `tracing` events; this renderer
    // turns them into the stderr lines the flags call for, for this dispatch
    // and this thread only.
    let diagnostics = Diagnostics::new(out.clone());
    let dispatched = crate::cli::commands::run(cli, &out, stdout, stderr, auth, overrides)
        .with_subscriber(diagnostics)
        .await;
    match dispatched {
        Ok(()) => EXIT_SUCCESS,
        Err(Failure::Emitted { exit_code }) => exit_code,
        Err(Failure::Error(e)) => {
            let code = e.exit_code();
            if carries_no_auth_method(&e) {
                let invocation: Vec<String> = args_vec
                    .iter()
                    .map(|a| a.to_string_lossy().into_owned())
                    .collect();
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

/// Whether this error is a missing-credential failure a recovery hint answers.
///
/// Both messages are constants shared with their construction sites, so the
/// seam and the library agree on the exact strings.
fn carries_no_auth_method(error: &xdk::error::Error) -> bool {
    matches!(
        error,
        xdk::error::Error::Auth(msg)
            if msg == xdk::error::NO_AUTH_METHOD || msg == xdk::error::NO_OAUTH2_TOKEN
    )
}

/// Renders a clap parse failure.
///
/// Help and version go to stdout at exit 0, except a help flag on a word that
/// names no command, which renders as that word does without the flag. An
/// unrecognized subcommand takes the unknown-command rendering, carrying
/// clap's own suggestion where clap scored one. A flag spelling where clap
/// wanted a command is an unexpected argument instead, except `-h` or
/// `--help` given to the `help` command, which prints that command's page.
/// Every other kind carries clap's words in `xr`'s dialect, as the `Error:`
/// line or the `invalid-args` envelope.
fn render_parse_error(
    error: &clap::Error,
    args: &[OsString],
    overrides: &xdk::config::EnvOverrides,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let rendered = error.to_string();
    let hidden_word = match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            parse_without_help(args).and_then(|cli| match classify(&cli) {
                Classified::UnknownCommand(word) => Some(word),
                Classified::Help | Classified::Raw => None,
            })
        }
        _ => None,
    };
    if hidden_word.is_none()
        && matches!(
            error.kind(),
            ErrorKind::DisplayHelp
                | ErrorKind::DisplayVersion
                | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        )
    {
        let _ = write!(stdout, "{rendered}");
        return EXIT_SUCCESS;
    }

    let intent = structured_intent(args, overrides.output.as_deref());
    // Quiet and verbose are unparsed here, and neither changes an error
    // envelope, so the provisional config leaves both off.
    let out = OutputConfig::new_with_no_color(
        intent.unwrap_or(OutputFormat::Text),
        false,
        false,
        color_choice(args),
        false,
        overrides.no_color,
    );

    if let Some(word) = hidden_word {
        let suggestion = nearest_command(&word);
        return render_unknown_command(&word, suggestion.as_deref(), ROOT_COMMAND, &out, stderr);
    }

    if error.kind() == ErrorKind::InvalidSubcommand
        && let Some(word) = context_string(error, ContextKind::InvalidSubcommand)
    {
        if !word.starts_with('-') {
            let suggestion = suggestion_for_rejected(error, args, &word);
            let command = failing_command(error, args);
            return render_unknown_command(&word, suggestion.as_deref(), &command, &out, stderr);
        }
        if matches!(word.as_str(), "-h" | "--help") && help_command_precedes(args, &word) {
            let _ = write!(stdout, "{}", help_command_page(args));
            return EXIT_SUCCESS;
        }
        let unexpected = Cli::command().error(
            ErrorKind::UnknownArgument,
            format!("unexpected argument '{word}' found"),
        );
        return render_invalid_args(&unexpected, args, &out, stderr);
    }

    render_invalid_args(error, args, &out, stderr)
}

/// A clap failure in `xr`'s dialect, as the `Error:` line or the
/// `invalid-args` envelope.
///
/// The renderer supplies `Error:` in place of clap's `error: ` prefix, and a
/// pointer at the command the failure belongs to replaces clap's closing
/// line, which names none.
fn render_invalid_args(
    error: &clap::Error,
    args: &[OsString],
    out: &OutputConfig,
    stderr: &mut dyn Write,
) -> i32 {
    let rendered = error.to_string();
    let body = rendered
        .strip_prefix("error: ")
        .unwrap_or(&rendered)
        .trim_end();
    let body = body
        .strip_suffix(CLAP_HELP_FOOTER)
        .unwrap_or(body)
        .trim_end();
    let command = failing_command(error, args);
    out.print_error_envelope(
        stderr,
        "invalid-args",
        EXIT_USAGE_ERROR,
        &format!("{body}\n\nTry '{command} --help'."),
    );
    EXIT_USAGE_ERROR
}

/// The `help` command's own page. clap renders it through `xr help help`, so
/// the page matches that invocation byte for byte.
fn help_command_page(args: &[OsString]) -> String {
    let bin = args
        .first()
        .cloned()
        .unwrap_or_else(|| OsString::from(ROOT_COMMAND));
    Cli::try_parse_from([bin, "help".into(), "help".into()])
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default()
}

/// The one rendering both detection paths use.
///
/// Text mode gets the sentence, pointing at the help of the nearest command
/// under `command`, the one the word was typed under, or at the help of
/// `command` itself when nothing scored close enough. Every structured mode
/// gets the envelope with the offending word in `command`, the nearest real
/// name in `suggestion`, absent when nothing scored, and a `next_step` that
/// runs the same help page.
///
/// The pointer names a help page rather than the corrected invocation: the
/// suggestion is a guess, and the corrected invocation of a write command
/// would act on it.
fn render_unknown_command(
    word: &str,
    suggestion: Option<&str>,
    command: &str,
    out: &OutputConfig,
    stderr: &mut dyn Write,
) -> i32 {
    let target = match suggestion {
        Some(nearest) => format!("{command} {nearest}"),
        None => command.to_string(),
    };
    let message = match suggestion {
        Some(nearest) => {
            format!("unknown command '{word}'. Did you mean '{nearest}'? Try '{target} --help'.")
        }
        None => format!("unknown command '{word}'. Try '{target} --help'."),
    };
    out.emit_error_envelope(
        stderr,
        ErrorBody {
            reason: "unknown-command".to_string(),
            exit_code: EXIT_USAGE_ERROR,
            message: Some(message),
            command: Some(word.to_string()),
            suggestion: suggestion.map(str::to_string),
            next_step: Some(NextStep::show_help(format!("{target} --help"))),
            ..ErrorBody::default()
        },
    );
    EXIT_USAGE_ERROR
}

// The trait objects `&mut dyn Write` in the entrypoint signature are not
// `Send` by themselves, but the function-pointer type below is, which is
// what library consumers need to dispatch the runner from a thread pool.
xdk::assert_send_sync!(fn() -> i32);
