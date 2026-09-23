//! Second parses of argv for the parse-error path.
//!
//! When clap rejects an invocation there is no [`Cli`] to read, so the
//! renderings a clap failure hands the runner parse the same argv again
//! through a command whose root help and version flags are inert: strictly,
//! to classify the word one of those flags hid, and leniently, to recover what
//! clap resolved before it stopped.

use std::ffi::OsString;

use clap::{Arg, ArgAction, ArgMatches, CommandFactory, FromArgMatches};

use crate::cli::classify::{ROOT_COMMAND, color_intent, usage_command};
use crate::cli::{Cli, ColorChoice};

/// `Cli::command()` with the root help and version flags inert.
///
/// clap adds both flags while building the command, so neither can be edited
/// in place: each is disabled, which clap applies to every subcommand, and an
/// inert counting flag with the same spellings stands in at the root. The
/// `help` subcommand stays, because without it the word `help` binds to the
/// positional and reads as an unknown command.
fn command_with_inert_display_flags() -> clap::Command {
    Cli::command()
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
/// invocation one of them interrupted can be classified like any other. Any
/// parse error returns `None`, which leaves clap's display in place.
pub(crate) fn parse_without_display_flags(args: &[OsString]) -> Option<Cli> {
    let matches = command_with_inert_display_flags()
        .try_get_matches_from(args)
        .ok()?;
    Cli::from_arg_matches(&matches).ok()
}

/// What clap resolved from `args` before an error stopped it: the flags ahead
/// of the rejected token, their environment bindings, and the defaults.
fn lenient_matches(args: &[OsString]) -> Option<ArgMatches> {
    command_with_inert_display_flags()
        .ignore_errors(true)
        .try_get_matches_from(args)
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

/// The command whose help a parse failure points at: the one clap's usage
/// line names, or, when clap attached no usage, the subcommands it resolved
/// before it stopped.
pub(crate) fn failing_command(error: &clap::Error, args: &[OsString]) -> String {
    usage_command(error).unwrap_or_else(|| {
        let mut words = vec![ROOT_COMMAND.to_string()];
        if let Some(matches) = lenient_matches(args) {
            let mut current = &matches;
            while let Some((name, sub)) = current.subcommand() {
                words.push(name.to_string());
                current = sub;
            }
        }
        words.join(" ")
    })
}
