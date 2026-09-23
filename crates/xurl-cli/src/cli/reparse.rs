//! Second parses of argv for the parse-error path.
//!
//! When clap rejects an invocation there is no [`Cli`] to read, so the
//! renderings a clap failure hands the runner parse the same argv again
//! through a command whose root help flag is inert: strictly, to classify the
//! word a help flag hid, and leniently, to recover what clap resolved before
//! it stopped.

use std::ffi::OsString;

use clap::{Arg, ArgAction, ArgMatches, CommandFactory, FromArgMatches};

use crate::cli::classify::color_intent;
use crate::cli::{Cli, ColorChoice};

/// `Cli::command()` with the root help flag inert.
///
/// clap adds the help flag while building the command, so it cannot be edited
/// in place: it is disabled, which clap applies to every subcommand, and an
/// inert counting flag with the same spellings stands in at the root. The
/// `help` subcommand stays, because without it the word `help` binds to the
/// positional and reads as an unknown command.
fn command_with_inert_help() -> clap::Command {
    Cli::command().disable_help_flag(true).arg(
        Arg::new("help")
            .short('h')
            .long("help")
            .action(ArgAction::Count),
    )
}

/// Parses `args` as if the root help flag were absent, so the invocation it
/// interrupted can be classified like any other. Any parse error returns
/// `None`, which leaves clap's help display in place.
pub(crate) fn parse_without_help(args: &[OsString]) -> Option<Cli> {
    let matches = command_with_inert_help().try_get_matches_from(args).ok()?;
    Cli::from_arg_matches(&matches).ok()
}

/// What clap resolved from `args` before an error stopped it: the flags ahead
/// of the rejected token, their environment bindings, and the defaults.
fn lenient_matches(args: &[OsString]) -> Option<ArgMatches> {
    command_with_inert_help()
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
