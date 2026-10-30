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

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{CommandFactory, Parser};
use tracing::instrument::WithSubscriber;

use crate::cli::classify::{
    Classified, ROOT_COMMAND, classify, context_string, help_command_precedes, invoked_help,
    nearest_command, rejected_word, suggestion_for_rejected,
};
use crate::cli::envelope::{ErrorBody, Reason};
use crate::cli::failure::Failure;
use crate::cli::hints::NextStep;
use crate::cli::output::{Diagnostics, OutputConfig, OutputFormat};
use crate::cli::reparse::{
    color_choice, equals_form_hint, failing_command, lenient_cli, output_intent,
    parse_past_stray_words, parse_without_display_flags, raw_choice,
};
use crate::cli::retired_verbs;
use crate::cli::skill_install::SkillEnv;
use crate::cli::{Cli, Commands};
use xdk::auth::Auth;
use xdk::config::Config;
use xdk::error::{EXIT_GENERAL_ERROR, EXIT_SUCCESS, EXIT_USAGE_ERROR};

/// What the structured rendering says when there is nothing to run.
///
/// Help text answers a person; an agent that asked for a machine-readable
/// format gets the usage error instead.
const NO_COMMAND_MESSAGE: &str = "No command given. Usage: xr [OPTIONS] [URL] [COMMAND].";

/// How clap closes a failure, naming no command.
const CLAP_HELP_FOOTER: &str = "For more information, try '--help'.";

/// Runs the `xr` CLI using `std::env::args_os()` and real stdio.
///
/// The binary's `main` calls this, and it alone installs the panic hook, so
/// a panic anywhere answers the `internal` error and exits 1. Library
/// consumers wanting capture should call [`run`] or [`run_with_store_path`]
/// directly; they keep Rust's own panic behavior.
///
/// Stdout and stderr are locked per write rather than for the whole run, so
/// the hook can write from whichever thread panicked.
#[must_use]
pub async fn run_argv() -> i32 {
    crate::cli::panic_hook::install();
    let args: Vec<OsString> = std::env::args_os().collect();
    run(args, &mut std::io::stdout(), &mut std::io::stderr()).await
}

/// Runs the `xr` CLI with caller-supplied args + writers.
///
/// Resolves the token-store path from `XURL_TOKEN_STORE`, falling back to
/// [`Config::default_store_path`] (`~/.xurl/auth.yml`), and delegates to
/// [`run_with_overrides`].
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
    let skill_env = crate::cli::env::skill_from_process(&overrides);
    run_with_env(args, stdout, stderr, &store_path, &overrides, &skill_env).await
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
/// rendered in that format, and in JSON when it names a format `xr` does not
/// print. Otherwise it is the `Error:` line every error takes, closing on the
/// failing command's help. An unrecognized subcommand,
/// and a bare word that names no command, both render as `unknown-command` at
/// the same exit code, with or without a help or version flag or a word after
/// it, and a bare invocation prints the root help at exit 0.
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
    let overrides = crate::cli::env::from_process();
    let skill_env = crate::cli::env::skill_from_process(&overrides);
    run_with_env(args, stdout, stderr, store_path, &overrides, &skill_env).await
}

/// The worker entrypoint — everything [`run_with_store_path`] does, with the
/// environment supplied as data instead of read from the process.
///
/// This and [`run_with_env`] are the entrypoints that read no environment
/// variables. The layers above them exist to resolve the inputs they cannot
/// invent: the token-store path and `overrides`. A library consumer that embeds
/// `xr`, or a test that must stay isolated from whatever else the process is
/// doing, calls this.
///
/// Skill destinations resolve against `overrides.home` alone: neither
/// `XURL_SKILL_HOME` nor a host's config- or base-directory variable applies.
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
    let skill_env = SkillEnv {
        home: overrides.home.clone(),
        ..SkillEnv::default()
    };
    run_with_env(args, stdout, stderr, store_path, overrides, &skill_env).await
}

/// [`run_with_overrides`] with the skill-destination environment supplied as
/// data too: `XURL_SKILL_HOME` and each host's config- and base-directory
/// variable, which `overrides` does not carry.
pub async fn run_with_env<I, S>(
    args: I,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    store_path: &Path,
    overrides: &xdk::config::EnvOverrides,
    skill_env: &SkillEnv,
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
    crate::cli::panic_hook::publish(&out);

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
                        let help = invoked_help(&args_vec);
                        match crate::cli::hints::general_hint(&e, &help) {
                            Some(hint) => {
                                out.print_error_with_hint(stderr, &e, EXIT_GENERAL_ERROR, &hint);
                            }
                            None => out.print_error(stderr, &e, EXIT_GENERAL_ERROR),
                        }
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
                    command, *list, *all, *envelope, &out, stdout,
                ) {
                    Ok(()) => EXIT_SUCCESS,
                    Err(refusal) => render_schema_refusal(refusal, &args_vec, &out, stderr),
                };
            }
            Commands::Skill { .. } => {
                // Move out of `cli` so the handler owns the enum (avoids clone).
                let Some(Commands::Skill { cmd }) = cli.command else {
                    unreachable!("matched Commands::Skill above")
                };
                return crate::cli::commands::skill::run_skill(cmd, &out, stdout, skill_env);
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
                out.print_invalid_args(stderr, NO_COMMAND_MESSAGE, " ", ROOT_COMMAND);
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
                Suggestion::nearest(suggestion.as_deref()),
                ROOT_COMMAND,
                &out,
                stderr,
            );
        }
        Classified::Raw => {}
    }
    if let Some(misplaced) = crate::cli::misplaced_flags::misplaced(&cli, &args_vec) {
        return render_misplaced_flags(&misplaced, &args_vec, &out, stderr);
    }

    // ── Tier 3: Everything else (needs config + auth) ──────────────────
    let mut cfg = Config::from_overrides(overrides);
    // Honour --timeout / XURL_TIMEOUT for every HTTP path: API client,
    // OAuth2 token exchange/refresh, and the `/2/users/me` lookup.
    cfg.http_timeout_secs = cli.timeout;
    // Store loading and redirect-URI resolution warn through `tracing` too,
    // and they run before the dispatch future the renderer below is attached
    // to, so the same renderer covers them for the duration of this call. A
    // single-file store is moved into its directory here, on the first
    // command that opens it, and the renderer prints the move.
    let auth = tracing::subscriber::with_default(Diagnostics::new(out.clone()), || {
        let store_path = xdk::store::adopt_directory_layout(store_path);
        Auth::new_with_store_path_and_overrides(&cfg, &store_path, overrides)
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
    let invocation = || -> Vec<String> {
        args_vec
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    };
    match dispatched {
        Ok(()) => EXIT_SUCCESS,
        Err(Failure::Emitted { exit_code }) => exit_code,
        Err(Failure::Refused { error, reason }) => {
            let hint = crate::cli::hints::show_help_hint(&invoked_help(&args_vec));
            out.print_refusal(stderr, &error, reason, &hint);
            error.exit_code()
        }
        Err(Failure::Unconfirmed(ctx)) => {
            let next_step = NextStep::confirm(&invocation());
            out.print_confirmation_required(stderr, &ctx, EXIT_GENERAL_ERROR, next_step);
            EXIT_GENERAL_ERROR
        }
        Err(Failure::DryRunRefusal { ctx, error }) => {
            let hint = recovery_hint(&error, &snapshot, &args_vec, structured);
            out.print_dry_run_refusal(stdout, &ctx, &error, hint.as_ref());
            EXIT_SUCCESS
        }
        Err(Failure::DryRunInvalid { ctx, error }) => {
            let hint = crate::cli::hints::show_help_hint(&invoked_help(&args_vec));
            out.print_dry_run_refusal(stdout, &ctx, &error, Some(&hint));
            EXIT_SUCCESS
        }
        Err(Failure::Error(e)) => {
            let code = e.exit_code();
            match recovery_hint(&e, &snapshot, &args_vec, structured) {
                Some(hint) => out.print_error_with_hint(stderr, &e, code, &hint),
                None => out.print_error(stderr, &e, code),
            }
            code
        }
    }
}

/// The recovery hint for a library error, chosen from the store as this
/// run loaded it and the invocation as given. A dry run's refusal takes the
/// same hint its real run would.
fn recovery_hint(
    error: &xdk::error::Error,
    snapshot: &xdk::store::snapshot::StoreSnapshot,
    args: &[OsString],
    structured: bool,
) -> Option<crate::cli::hints::Hint> {
    let invocation: Vec<String> = args
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    if carries_no_auth_method(error) {
        return Some(crate::cli::hints::choose_hint(
            snapshot,
            &invocation,
            structured,
        ));
    }
    crate::cli::hints::unloadable_store_hint(snapshot, error)
        .or_else(|| crate::cli::hints::enrollment_hint(error))
        .or_else(|| crate::cli::hints::resume_wait_hint(error))
        .or_else(|| crate::cli::hints::address_in_use_hint(error, &invocation))
        .or_else(|| crate::cli::hints::no_signing_secret_hint(snapshot, error, &invocation))
        .or_else(|| crate::cli::hints::wait_and_retry_hint(error))
        .or_else(|| crate::cli::hints::mismatch_hint(snapshot, error, &invocation, structured))
        .or_else(|| crate::cli::hints::general_hint(error, &invoked_help(args)))
}

/// Whether this error is a missing-credential failure a recovery hint answers.
///
/// Both messages are constants shared with their construction sites, so the
/// seam and the library agree on the exact strings.
fn carries_no_auth_method(error: &xdk::error::Error) -> bool {
    matches!(
        error,
        xdk::error::Error::Auth { message, .. }
            if message == xdk::error::NO_AUTH_METHOD || message == xdk::error::NO_OAUTH2_TOKEN
    )
}

/// Whether a clap error is a help or version display rather than a failure.
fn is_display(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::DisplayHelp
            | ErrorKind::DisplayVersion
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    )
}

/// Renders a clap parse failure.
///
/// Help and version go to stdout at exit 0, except a help or version flag on a
/// word that names no command, which renders as the invocation does without
/// the flag: as that word does, or, when the invocation is a usage error of its
/// own (a repeated or unknown flag), as that error. A word clap rejected after
/// a root positional that names no command renders as that positional does
/// alone, so `xr suth status` reports `suth`; after a raw target, or beside a
/// raw-only flag, it is an unexpected argument. Any other unrecognized subcommand
/// takes the unknown-command rendering, carrying clap's own suggestion where
/// clap scored one. A flag spelling where clap wanted a command is an
/// unexpected argument instead, except `-h` or `--help` given to the `help`
/// command, which prints that command's page. Every other kind carries clap's
/// words in `xr`'s dialect, as the `Error:` line or the `invalid-args`
/// envelope.
fn render_parse_error(
    error: &clap::Error,
    args: &[OsString],
    overrides: &xdk::config::EnvOverrides,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let rendered = error.to_string();
    let unknown_word = |cli: &Cli| match classify(cli) {
        Classified::UnknownCommand(word) => Some(word),
        Classified::Help | Classified::Raw => None,
    };
    let display = is_display(error.kind());
    let hidden_word = if display {
        match parse_without_display_flags(args) {
            Ok(cli) => unknown_word(&cli),
            Err(without_flag) => {
                if !is_display(without_flag.kind())
                    && lenient_cli(args)
                        .and_then(|cli| unknown_word(&cli))
                        .is_some()
                {
                    return render_parse_error(&without_flag, args, overrides, stdout, stderr);
                }
                None
            }
        }
    } else {
        None
    };
    if hidden_word.is_none() && display {
        let _ = write!(stdout, "{rendered}");
        return EXIT_SUCCESS;
    }

    let intent = output_intent(args, overrides.output.as_deref());
    // Quiet and verbose are unparsed here, and neither changes an error
    // envelope, so the provisional config leaves both off.
    let out = OutputConfig::new_with_no_color(
        intent.unwrap_or(OutputFormat::Text),
        false,
        false,
        color_choice(args),
        raw_choice(args),
        overrides.no_color,
    );

    let rejected = rejected_word(error);
    let past_stray_words = rejected.as_ref().and_then(|_| parse_past_stray_words(args));
    let root_word = hidden_word.or_else(|| unknown_word(past_stray_words.as_ref()?));
    if let Some(word) = root_word {
        let suggestion = nearest_command(&word);
        return render_unknown_command(
            &word,
            Suggestion::nearest(suggestion.as_deref()),
            ROOT_COMMAND,
            &out,
            stderr,
        );
    }
    // clap scores a stray word against the commands even after a raw target,
    // where the caller meant a request and named no command.
    if past_stray_words.is_some()
        && error.kind() == ErrorKind::InvalidSubcommand
        && let Some(word) = &rejected
    {
        return render_invalid_args(&unexpected_argument(word, args), args, &out, stderr);
    }

    if error.kind() == ErrorKind::InvalidSubcommand
        && let Some(word) = context_string(error, ContextKind::InvalidSubcommand)
    {
        if !word.starts_with('-') {
            let command = failing_command(error, args);
            let nearest = suggestion_for_rejected(error, args, &word);
            let suggestion = Suggestion::for_word(&command, &word, nearest.as_deref());
            return render_unknown_command(&word, suggestion, &command, &out, stderr);
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
    // clap puts its own tips between the error line and the usage line.
    let body = match equals_form_hint(error, args, &command) {
        Some(hint) => match body.split_once("\n\nUsage:") {
            Some((failure, usage)) => format!("{failure}\n\n  tip: {hint}\n\nUsage:{usage}"),
            None => format!("{body}\n\n  tip: {hint}"),
        },
        None => body.to_string(),
    };
    out.print_invalid_args(stderr, &body, "\n\n", &command);
    EXIT_USAGE_ERROR
}

/// clap's rejection of `word` as a stray argument at the root, built as clap
/// builds it for a word it scores against no command, usage line included.
fn unexpected_argument(word: &str, args: &[OsString]) -> clap::Error {
    let mut cmd = Cli::command();
    if let Some(bin) = args.first().and_then(|arg| Path::new(arg).file_name()) {
        cmd = cmd.bin_name(bin.to_string_lossy().into_owned());
    }
    let usage = cmd.render_usage();
    let mut error = clap::Error::new(ErrorKind::UnknownArgument).with_cmd(&cmd);
    error.insert(
        ContextKind::InvalidArg,
        ContextValue::String(word.to_string()),
    );
    error.insert(ContextKind::Usage, ContextValue::StyledStr(usage));
    error
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

/// Renders raw-only flags given ahead of a command word: `invalid-args`,
/// with the invocation that moves them after the word as its step when one
/// exists, and the command's help otherwise. The message and the step name
/// the same invocation.
fn render_misplaced_flags(
    misplaced: &crate::cli::misplaced_flags::Misplaced,
    args: &[OsString],
    out: &OutputConfig,
    stderr: &mut dyn Write,
) -> i32 {
    let quoted: Vec<String> = misplaced
        .flags
        .iter()
        .map(|flag| format!("'{flag}'"))
        .collect();
    let (named, verb, pronoun) = match quoted.as_slice() {
        [one] => (one.clone(), "comes", "it"),
        [first, second] => (format!("{first} and {second}"), "come", "them"),
        [rest @ .., last] => (format!("{}, and {last}", rest.join(", ")), "come", "them"),
        [] => (String::new(), "comes", "it"),
    };
    let problem = format!(
        "{named} {verb} before the command word, where only a raw request reads {pronoun}."
    );
    let (message, next_step) = match &misplaced.moved {
        Some(moved) => (
            format!("{problem} Run: {moved}"),
            NextStep::fix_input_with(moved.clone()),
        ),
        None => {
            let help = invoked_help(args);
            (
                format!("{problem} Try '{help}'."),
                NextStep::show_help(help),
            )
        }
    };
    out.emit_error_envelope(
        stderr,
        ErrorBody {
            reason: Reason::InvalidArgs,
            exit_code: EXIT_USAGE_ERROR,
            message: Some(message),
            next_step: Some(next_step),
            ..ErrorBody::default()
        },
    );
    EXIT_USAGE_ERROR
}

/// Renders what `xr schema` refused. A word that names no command takes
/// the refusal `xr help` gives it; an old name or a scope word is
/// `invalid-args` with the invocation that works as its step, when one
/// exists; a fault in the registry names the issue tracker; anything else is
/// the value refusal every command gives.
fn render_schema_refusal(
    refusal: crate::cli::commands::schema::Refusal,
    args: &[OsString],
    out: &OutputConfig,
    stderr: &mut dyn Write,
) -> i32 {
    use crate::cli::commands::schema::{Refusal, invocation};
    let (given, problem, run) = match refusal {
        Refusal::UnknownCommand {
            word,
            suggestion,
            under,
        } => {
            return render_unknown_command(
                &word,
                Suggestion::for_word(&under, &word, suggestion.as_deref()),
                &under,
                out,
                stderr,
            );
        }
        Refusal::Invalid(error) => {
            let hint = match error {
                xdk::error::Error::Internal(_) => crate::cli::hints::general_hint(&error, ""),
                _ => None,
            }
            .unwrap_or_else(|| crate::cli::hints::show_help_hint(&invoked_help(args)));
            out.print_refusal(stderr, &error, None, &hint);
            return EXIT_GENERAL_ERROR;
        }
        Refusal::OldName { given, command } => {
            let problem = format!(
                "'{given}' is a name `xr schema` no longer takes; it takes the command's words."
            );
            (given, problem, Some(invocation(command)))
        }
        Refusal::ScopeWord(word) => {
            let flag = format!("--{word}");
            let alone = Cli::try_parse_from(["xr", "schema", flag.as_str()]).is_ok();
            let problem = format!(
                "'{word}' names a scope of `xr schema`, not a command; it is the flag {flag}."
            );
            (word, problem, alone.then(|| format!("xr schema {flag}")))
        }
    };
    let (message, next_step) = match run {
        Some(run) => (
            format!("{problem} Run: {run}"),
            NextStep::fix_input_with(run),
        ),
        None => (
            format!("{problem} Try 'xr schema --help'."),
            NextStep::show_help("xr schema --help".to_string()),
        ),
    };
    out.emit_error_envelope(
        stderr,
        ErrorBody {
            reason: Reason::InvalidArgs,
            exit_code: EXIT_USAGE_ERROR,
            message: Some(message),
            command: Some(given),
            next_step: Some(next_step),
            ..ErrorBody::default()
        },
    );
    EXIT_USAGE_ERROR
}

/// The command an unknown word's pointer names, and why it names it.
#[derive(Clone, Copy)]
enum Suggestion<'a> {
    /// Nothing scored close enough to name.
    Nothing,
    /// The nearest real command by spelling: a guess at a typo.
    Nearest(&'a str),
    /// The command that replaced a word its family retired.
    Replacement(&'a str),
}

impl<'a> Suggestion<'a> {
    fn nearest(name: Option<&'a str>) -> Self {
        name.map_or(Self::Nothing, Self::Nearest)
    }

    /// The word that replaced `word` when `command`'s family retired it, and
    /// the nearest name by spelling otherwise.
    fn for_word(command: &str, word: &str, nearest: Option<&'a str>) -> Self {
        match retired_verbs::replacement(command, word) {
            Some(replacement) => Self::Replacement(replacement),
            None => Self::nearest(nearest),
        }
    }

    fn name(self) -> Option<&'a str> {
        match self {
            Self::Nothing => None,
            Self::Nearest(name) | Self::Replacement(name) => Some(name),
        }
    }
}

/// The one rendering both detection paths use.
///
/// Text mode gets the sentence, pointing at the help of the suggested command
/// under `command`, the one the word was typed under, or at the help of
/// `command` itself when nothing scored close enough. Every structured mode
/// gets the envelope with the offending word in `command`, the suggested name
/// in `suggestion`, absent when there is none, and a `next_step` that runs the
/// same help page. A replacement reads as one, not as a guess at a typo.
///
/// The pointer names a help page rather than the corrected invocation: the
/// suggestion can be a guess, and the corrected invocation of a write command
/// would act on it.
fn render_unknown_command(
    word: &str,
    suggestion: Suggestion<'_>,
    command: &str,
    out: &OutputConfig,
    stderr: &mut dyn Write,
) -> i32 {
    let target = match suggestion.name() {
        Some(name) => format!("{command} {name}"),
        None => command.to_string(),
    };
    let message = match suggestion {
        Suggestion::Nothing => format!("unknown command '{word}'. Try '{target} --help'."),
        Suggestion::Nearest(name) => {
            format!("unknown command '{word}'. Did you mean '{name}'? Try '{target} --help'.")
        }
        Suggestion::Replacement(name) => {
            format!("unknown command '{word}': '{name}' replaced it. Try '{target} --help'.")
        }
    };
    out.emit_error_envelope(
        stderr,
        ErrorBody {
            reason: Reason::UnknownCommand,
            exit_code: EXIT_USAGE_ERROR,
            message: Some(message),
            command: Some(word.to_string()),
            suggestion: suggestion.name().map(str::to_string),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ColorChoice;

    /// A command the registry resolves but holds no schema for is a fault in
    /// xr, so its refusal points at the issue tracker rather than at a help
    /// page the caller can do nothing with.
    #[test]
    fn a_broken_schema_registry_reports_an_internal_fault() {
        let out = OutputConfig::new(OutputFormat::Json, false, false, ColorChoice::Never);
        let mut stderr = Vec::new();
        let code = render_schema_refusal(
            crate::cli::commands::schema::Refusal::Invalid(xdk::error::Error::Internal(
                "no schema registered for 'post'".into(),
            )),
            &["xr", "schema", "post"].map(OsString::from),
            &out,
            &mut stderr,
        );
        let envelope: serde_json::Value =
            serde_json::from_slice(&stderr).expect("a JSON error envelope");
        assert_eq!(code, EXIT_GENERAL_ERROR);
        assert_eq!(envelope["reason"], "internal");
        assert_eq!(envelope["next_step"]["action"], "report-issue");
    }
}
