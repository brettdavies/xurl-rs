//! The one place argv reaches clap.
//!
//! Every parse of argv starts from [`command`], which is `xr`'s command with
//! the binding removed from each boolean flag whose variable reads as the
//! flag's absent value. Clap counts a value read from an `env` binding as
//! given even when the flag's own parser reads it as false, so a variable
//! that says "off" would still take part in the conflict and requirement
//! checks of the flag it is bound to. Taking the binding away makes such a
//! variable unset, which is what it says.

use std::any::TypeId;
use std::ffi::{OsStr, OsString};
use std::sync::OnceLock;

use clap::builder::ValueParser;
use clap::{Arg, ArgAction, ArgMatches, Command, CommandFactory, FromArgMatches};

use crate::cli::Cli;

/// A boolean flag whose variable reads as the flag's absent value, named by
/// the command words that reach the command declaring it and by its id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    /// The subcommand words from the root to the declaring command; empty
    /// for the root.
    pub path: Vec<String>,
    /// The argument's id.
    pub id: String,
}

/// `xr`'s command with the bindings of false booleans removed, for the
/// variables the process holds.
///
/// The analysis behind it runs once per process: the process's variables do
/// not change, and clap read them when the command was built.
#[must_use]
pub fn command() -> Command {
    static FOUND: OnceLock<Vec<Binding>> = OnceLock::new();
    let found = FOUND.get_or_init(|| false_bindings(&Cli::command(), &crate::cli::env::variable));
    unbind(Cli::command(), found)
}

/// Parses `args` with [`command`] after `customize` has edited it, which is
/// the only call that hands argv to clap.
///
/// # Errors
///
/// Returns clap's error when `args` do not parse.
pub fn try_matches<I, T>(
    args: I,
    customize: impl FnOnce(Command) -> Command,
) -> Result<ArgMatches, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    customize(command()).try_get_matches_from(args)
}

/// Parses `args` into a [`Cli`] through [`command`], formatting a failure as
/// clap's own `Parser::try_parse_from` does.
///
/// # Errors
///
/// Returns clap's error when `args` do not parse.
pub fn try_cli<I, T>(args: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    try_matches(args, std::convert::identity)
        .and_then(|matches| Cli::from_arg_matches(&matches))
        .map_err(|error| error.format(&mut Cli::command()))
}

/// `command` with the bindings of every boolean flag whose variable, as
/// `lookup` holds it, reads as the flag's absent value removed.
#[must_use]
pub fn without_false_bindings(
    command: Command,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Command {
    let found = false_bindings(&command, lookup);
    unbind(command, &found)
}

/// The boolean flags of `command` whose variable `lookup` holds and the flag's
/// own parser reads as the flag's absent value.
///
/// The absent value is the flag's default parsed by the same parser when it
/// declares one, else false for a set-true flag and true for a set-false one.
/// A value the parser rejects names no binding, so clap reports it. When no
/// variable `lookup` holds is bound anywhere in the tree, nothing is built.
#[must_use]
pub fn false_bindings(command: &Command, lookup: &dyn Fn(&str) -> Option<String>) -> Vec<Binding> {
    if !holds_a_bound_variable(command, lookup) {
        return Vec::new();
    }
    let mut built = command.clone();
    built.build();
    let mut found = Vec::new();
    collect(&built, &mut Vec::new(), lookup, &mut found);
    found
}

/// `command` with the binding of each flag `bindings` names reset.
#[must_use]
pub fn unbind(command: Command, bindings: &[Binding]) -> Command {
    unbind_at(command, bindings, &[])
}

fn holds_a_bound_variable(command: &Command, lookup: &dyn Fn(&str) -> Option<String>) -> bool {
    command
        .get_arguments()
        .filter_map(Arg::get_env)
        .filter_map(OsStr::to_str)
        .any(|name| lookup(name).is_some())
        || command
            .get_subcommands()
            .any(|sub| holds_a_bound_variable(sub, lookup))
}

fn collect(
    command: &Command,
    path: &mut Vec<String>,
    lookup: &dyn Fn(&str) -> Option<String>,
    found: &mut Vec<Binding>,
) {
    for arg in command.get_arguments() {
        let held = arg.get_env().and_then(OsStr::to_str).and_then(lookup);
        if let Some(value) = held
            && reads_absent(arg, &value)
        {
            found.push(Binding {
                path: path.clone(),
                id: arg.get_id().to_string(),
            });
        }
    }
    for sub in command.get_subcommands() {
        path.push(sub.get_name().to_string());
        collect(sub, path, lookup, found);
        path.pop();
    }
}

/// Whether `arg` is a boolean flag whose parser reads `value` as the flag's
/// absent value.
fn reads_absent(arg: &Arg, value: &str) -> bool {
    let parser = arg.get_value_parser();
    parser.type_id() == TypeId::of::<bool>()
        && absent_value(arg, parser).is_some_and(|absent| read_bool(parser, value) == Some(absent))
}

fn absent_value(arg: &Arg, parser: &ValueParser) -> Option<bool> {
    if let Some(default) = arg.get_default_values().first() {
        return read_bool(parser, default.to_str()?);
    }
    match arg.get_action() {
        ArgAction::SetTrue => Some(false),
        ArgAction::SetFalse => Some(true),
        _ => None,
    }
}

/// What `parser` reads `value` as. The parser's entry point is private to
/// clap, so a one-argument command runs it.
fn read_bool(parser: &ValueParser, value: &str) -> Option<bool> {
    Command::new("probe")
        .no_binary_name(true)
        .arg(
            Arg::new("value")
                .value_parser(parser.clone())
                .num_args(1)
                .allow_hyphen_values(true)
                .required(true),
        )
        .try_get_matches_from([value])
        .ok()?
        .get_one::<bool>("value")
        .copied()
}

fn unbind_at(mut command: Command, bindings: &[Binding], path: &[String]) -> Command {
    for binding in bindings.iter().filter(|binding| binding.path == path) {
        if command
            .get_arguments()
            .any(|arg| arg.get_id() == binding.id.as_str())
        {
            command = command.mut_arg(binding.id.as_str(), |arg| arg.env(None::<&str>));
        }
    }
    let mut below: Vec<&String> = bindings
        .iter()
        .filter(|binding| binding.path.len() > path.len() && binding.path.starts_with(path))
        .map(|binding| &binding.path[path.len()])
        .collect();
    below.sort();
    below.dedup();
    for word in below {
        let mut next = path.to_vec();
        next.push(word.clone());
        command = command.mut_subcommand(word.as_str(), |sub| unbind_at(sub, bindings, &next));
    }
    command
}

#[cfg(test)]
mod tests {
    use clap::builder::FalseyValueParser;
    use clap::{Arg, ArgAction, Command};

    use super::without_false_bindings;

    fn lookup(held: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        move |name| {
            held.iter()
                .find(|(held_name, _)| *held_name == name)
                .map(|(_, value)| (*value).to_string())
        }
    }

    fn bound(command: &Command, id: &str) -> bool {
        command
            .get_arguments()
            .find(|arg| arg.get_id() == id)
            .expect("the flag exists")
            .get_env()
            .is_some()
    }

    fn set_true(id: &'static str) -> Arg {
        Arg::new(id)
            .long(id)
            .env(format!("FIXTURE_{}", id.to_uppercase()))
            .value_parser(FalseyValueParser::new())
            .action(ArgAction::SetTrue)
    }

    fn set_with_default(id: &'static str) -> Arg {
        Arg::new(id)
            .long(id)
            .env(format!("FIXTURE_{}", id.to_uppercase()))
            .value_parser(FalseyValueParser::new())
            .num_args(0..=1)
            .default_value("false")
            .default_missing_value("true")
            .require_equals(true)
    }

    #[test]
    fn a_set_true_flag_with_a_false_variable_loses_its_binding() {
        let command = Command::new("x").arg(set_true("quiet"));
        let command = without_false_bindings(command, &lookup(&[("FIXTURE_QUIET", "0")]));
        assert!(!bound(&command, "quiet"));
    }

    #[test]
    fn a_set_true_flag_with_a_true_variable_keeps_its_binding() {
        let command = Command::new("x").arg(set_true("quiet"));
        let command = without_false_bindings(command, &lookup(&[("FIXTURE_QUIET", "1")]));
        assert!(bound(&command, "quiet"));
    }

    #[test]
    fn a_defaulted_set_flag_follows_its_default() {
        let command = Command::new("x").arg(set_with_default("quiet"));
        let false_variable =
            without_false_bindings(command.clone(), &lookup(&[("FIXTURE_QUIET", "0")]));
        assert!(!bound(&false_variable, "quiet"));
        let true_variable = without_false_bindings(command, &lookup(&[("FIXTURE_QUIET", "1")]));
        assert!(bound(&true_variable, "quiet"));
    }

    #[test]
    fn an_empty_variable_reads_false_under_a_falsey_parser() {
        let command = Command::new("x").arg(set_true("quiet"));
        let command = without_false_bindings(command, &lookup(&[("FIXTURE_QUIET", "")]));
        assert!(!bound(&command, "quiet"));
    }

    #[test]
    fn a_value_the_parser_rejects_keeps_the_binding_for_clap_to_report() {
        let command = Command::new("x").arg(
            Arg::new("quiet")
                .long("quiet")
                .env("FIXTURE_QUIET")
                .value_parser(clap::builder::BoolishValueParser::new())
                .action(ArgAction::SetTrue),
        );
        let command = without_false_bindings(command, &lookup(&[("FIXTURE_QUIET", "maybe")]));
        assert!(bound(&command, "quiet"));
    }

    #[test]
    fn a_set_false_flag_loses_its_binding_for_true_and_keeps_it_for_false() {
        let flag = || {
            Command::new("x").arg(
                Arg::new("color")
                    .long("no-color")
                    .env("FIXTURE_COLOR")
                    .value_parser(FalseyValueParser::new())
                    .action(ArgAction::SetFalse),
            )
        };
        let true_variable = without_false_bindings(flag(), &lookup(&[("FIXTURE_COLOR", "1")]));
        assert!(!bound(&true_variable, "color"));
        let false_variable = without_false_bindings(flag(), &lookup(&[("FIXTURE_COLOR", "0")]));
        assert!(bound(&false_variable, "color"));
    }

    #[test]
    fn a_non_boolean_option_equal_to_its_default_keeps_its_binding() {
        let command = Command::new("x").arg(
            Arg::new("limit")
                .long("limit")
                .env("FIXTURE_LIMIT")
                .default_value("10"),
        );
        let command = without_false_bindings(command, &lookup(&[("FIXTURE_LIMIT", "10")]));
        assert!(bound(&command, "limit"));
    }

    #[test]
    fn a_boolean_flag_on_a_subcommand_is_handled_like_a_root_one() {
        let command = Command::new("x").subcommand(Command::new("sub").arg(set_true("quiet")));
        let command = without_false_bindings(command, &lookup(&[("FIXTURE_QUIET", "0")]));
        let sub = command
            .find_subcommand("sub")
            .expect("the subcommand exists");
        assert!(!bound(sub, "quiet"));
    }

    #[test]
    fn with_no_bound_variable_set_every_binding_is_intact() {
        let command = Command::new("x")
            .arg(set_true("quiet"))
            .subcommand(Command::new("sub").arg(set_with_default("raw")));
        let command = without_false_bindings(command, &lookup(&[]));
        assert!(bound(&command, "quiet"));
        let sub = command
            .find_subcommand("sub")
            .expect("the subcommand exists");
        assert!(bound(sub, "raw"));
    }
}
