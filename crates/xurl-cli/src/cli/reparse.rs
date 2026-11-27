//! Second parses of argv for the parse-error path.
//!
//! When clap rejects an invocation there is no [`Cli`] to read, so the
//! renderings a clap failure hands the runner parse the same argv again
//! through a command whose root help and version flags are inert: strictly,
//! to classify the word one of those flags hid; with room for words after the
//! root positional, to classify the positional the first of them stopped clap
//! at; and leniently, to recover what clap resolved before it stopped.

use std::ffi::OsString;

use clap::error::{ContextKind, ErrorKind};
use clap::{Arg, ArgAction, ArgMatches, FromArgMatches};

use crate::cli::classify::{ROOT_COMMAND, color_intent, context_string, raw_intent, usage_command};
use crate::cli::output::OutputFormat;
use crate::cli::settings::{self, Variables};
use crate::cli::{Cli, ColorChoice, parse};

/// [`parse::root`] with the root help and version flags inert.
///
/// clap adds both flags while building the command, so neither can be edited
/// in place: each is disabled, which clap applies to every subcommand, and an
/// inert counting flag with the same spellings stands in at the root. The
/// `help` subcommand stays, because without it the word `help` binds to the
/// positional and reads as an unknown command.
fn with_inert_display_flags(command: clap::Command) -> clap::Command {
    command
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::Count),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::Count),
        )
}

/// Parses `args` as if the root help and version flags were absent, so the
/// invocation one of them interrupted can be classified like any other. The
/// error is the one that invocation raises without the flag.
pub(crate) fn parse_without_display_flags(args: &[OsString]) -> Result<Cli, clap::Error> {
    let matches = parse::try_matches(args, with_inert_display_flags)?;
    Cli::from_arg_matches(&matches)
}

/// `args` parsed with room for words after the root positional, so an
/// invocation clap stopped at the first of them can be classified like any
/// other: `xr suth status` reads as `xr suth`, while a raw-only flag anywhere
/// in argv still marks a request. `None` when it fails for any other reason.
pub(crate) fn parse_past_stray_words(args: &[OsString]) -> Option<Cli> {
    let matches = parse::try_matches(args, |command| {
        with_inert_display_flags(command).arg(
            Arg::new("stray_words")
                .num_args(1..)
                .action(ArgAction::Append),
        )
    })
    .ok()?;
    Cli::from_arg_matches(&matches).ok()
}

/// `args` read leniently as a [`Cli`]: what clap bound before any error, so
/// the word a help or version flag hid can be classified even when the
/// invocation has another usage error.
pub(crate) fn lenient_cli(args: &[OsString]) -> Option<Cli> {
    Cli::from_arg_matches(&lenient_matches(args)?).ok()
}

/// What clap resolved from `args` before an error stopped it: the flags ahead
/// of the rejected token, their environment bindings, and the defaults.
pub(crate) fn lenient_matches(args: &[OsString]) -> Option<ArgMatches> {
    parse::try_matches(args, |command| {
        with_inert_display_flags(command).ignore_errors(true)
    })
    .ok()
}

/// The color choice for a parse-error rendering: a `--color` named anywhere in
/// argv, then what clap resolved, which carries the `XURL_COLOR` binding and
/// the default.
pub(crate) fn color_choice(args: &[OsString]) -> ColorChoice {
    color_intent(args)
        .or_else(|| {
            lenient_matches(args)?
                .try_get_one::<ColorChoice>("color")
                .ok()
                .flatten()
                .copied()
        })
        .unwrap_or_default()
}

/// Whether a parse-error rendering is raw: a `--raw` named anywhere in argv,
/// then what clap resolved, which carries the `XURL_RAW` binding and the
/// default.
pub(crate) fn raw_choice(args: &[OsString]) -> bool {
    raw_intent(args)
        .or_else(|| {
            lenient_matches(args)?
                .try_get_one::<bool>("raw")
                .ok()
                .flatten()
                .copied()
        })
        .unwrap_or(false)
}

/// The output format for a parse-error rendering, from the same layers the
/// resolver reads and never refusing: the last format argv names before a
/// `--`, a typed `text` included, and only when argv names none, the variables
/// in the order `XURL_JSONL`, `XURL_JSON`, `XURL_OUTPUT`.
///
/// Read from the unparsed argv, because a clap error leaves no `Cli`, and
/// clap stops at the token it rejected, so a flag after that token reaches no
/// parse.
pub(crate) fn output_intent(args: &[OsString], variables: Variables<'_>) -> Option<OutputFormat> {
    settings::typed_format(args).unwrap_or_else(|| settings::variable_format(variables))
}

/// The correction for a value given after a space to a flag that takes it
/// only after `=`.
///
/// Such a flag stands alone without a value, so clap reads `--wait 60` as the
/// bare flag and a stray `60`, and its message names only the stray word.
/// When the word before the one clap rejected is a flag `command` declares
/// that way, this is the line that shows the spelling that works.
pub(crate) fn equals_form_hint(
    error: &clap::Error,
    args: &[OsString],
    command: &str,
) -> Option<String> {
    if error.kind() != ErrorKind::UnknownArgument {
        return None;
    }
    let stray = context_string(error, ContextKind::InvalidArg)?;
    let words: Vec<String> = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let flag = words
        .windows(2)
        .find(|pair| pair[1] == stray && takes_value_after_equals(command, &pair[0]))
        .map(|pair| pair[0].as_str())?;
    Some(format!(
        "'{flag}' takes its value after '=': {flag}={stray}"
    ))
}

/// Whether `flag`, as typed (`--wait`, `-w`), is declared `require_equals` on
/// `command` or on a command above it.
fn takes_value_after_equals(command: &str, flag: &str) -> bool {
    let root = parse::root();
    let mut scopes = vec![&root];
    for name in command.split_whitespace().skip(1) {
        match scopes.last().and_then(|scope| scope.find_subcommand(name)) {
            Some(sub) => scopes.push(sub),
            None => break,
        }
    }
    scopes
        .iter()
        .flat_map(|scope| scope.get_arguments())
        .filter(|arg| arg.is_require_equals_set())
        .any(|arg| {
            arg.get_long()
                .is_some_and(|long| flag == format!("--{long}"))
                || arg
                    .get_short()
                    .is_some_and(|short| flag == format!("-{short}"))
        })
}

/// The command whose help a parse failure points at: the one clap's usage
/// line names, or, when clap attached no usage, the subcommands it resolved
/// before it stopped.
pub(crate) fn failing_command(error: &clap::Error, args: &[OsString]) -> String {
    usage_command(error).unwrap_or_else(|| resolved_command(args))
}

/// The command `args` reach, found from the subcommands clap resolved before
/// it stopped, wherever the flags sit.
pub(crate) fn resolved_command(args: &[OsString]) -> String {
    let mut words = vec![ROOT_COMMAND.to_string()];
    if let Some(matches) = lenient_matches(args) {
        let mut current = &matches;
        while let Some((name, sub)) = current.subcommand() {
            words.push(name.to_string());
            current = sub;
        }
    }
    words.join(" ")
}
