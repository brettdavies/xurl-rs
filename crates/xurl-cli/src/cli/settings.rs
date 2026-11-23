//! The two settings `xr` resolves by layer: the output format and the page
//! token.
//!
//! A setting has several spellings, each a flag with a variable of its own. A
//! typed spelling beats every variable of its setting, and of several typed the
//! last wins. With none typed, the variables decide: two that name a value are
//! refused, one decides, and none leaves the default. A boolean variable that
//! reads false names no value, and neither does an empty one.
//!
//! The variables are not bound in clap, whose conflict and override checks
//! count a value read from the environment as typed. A variable that decides a
//! setting is checked afterwards against the arguments clap declares in
//! conflict with the spelling it stands for, so `XURL_CURSOR` beside `--page`
//! is refused as `--cursor` beside `--page` is.

use std::ffi::OsString;

use clap::builder::ValueParser;
use clap::error::{ContextKind, ContextValue};
use clap::parser::ValueSource;
use clap::{Arg, ArgMatches, Command, CommandFactory};

use crate::cli::Cli;
use crate::cli::conflict_source::{self, Refusal, Side, Source};
use crate::cli::env::FlagEnv;
use crate::cli::output::OutputFormat;
use crate::cli::parse;

/// One way to set a setting: a flag, and the variable that stands for it.
#[derive(Debug, PartialEq, Eq)]
pub struct Spelling {
    /// The flag's clap id.
    pub id: &'static str,
    /// The flag as typed, with its leading dashes.
    pub flag: &'static str,
    /// The environment variable that stands for the flag.
    pub variable: &'static str,
    /// Whether the flag takes a value, as `--output` does.
    pub takes_value: bool,
}

/// A setting, and the spellings that set it.
#[derive(Debug, PartialEq, Eq)]
pub struct Setting {
    /// The setting's name.
    pub name: &'static str,
    /// Every spelling of the setting, in the order a refusal names them.
    pub spellings: &'static [Spelling],
}

/// The format a run prints in.
pub const OUTPUT: Setting = Setting {
    name: "output format",
    spellings: &[
        Spelling {
            id: "output",
            flag: "--output",
            variable: "XURL_OUTPUT",
            takes_value: true,
        },
        Spelling {
            id: "json",
            flag: "--json",
            variable: "XURL_JSON",
            takes_value: false,
        },
        Spelling {
            id: "jsonl",
            flag: "--jsonl",
            variable: "XURL_JSONL",
            takes_value: false,
        },
    ],
};

/// The token of the page a read asks for.
pub const PAGE_TOKEN: Setting = Setting {
    name: "page token",
    spellings: &[
        Spelling {
            id: "cursor",
            flag: "--cursor",
            variable: "XURL_CURSOR",
            takes_value: true,
        },
        Spelling {
            id: "after",
            flag: "--after",
            variable: "XURL_AFTER",
            takes_value: true,
        },
    ],
};

/// Every setting resolved by layer.
pub const SETTINGS: [&Setting; 2] = [&OUTPUT, &PAGE_TOKEN];

/// The values of the variables of the two settings, as data.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Variables<'a> {
    output: Option<&'a str>,
    flags: &'a FlagEnv,
}

impl<'a> Variables<'a> {
    /// `output` is `XURL_OUTPUT`; `flags` holds the other four.
    pub(crate) fn new(output: Option<&'a str>, flags: &'a FlagEnv) -> Self {
        Self { output, flags }
    }

    /// The value the variable of `spelling` holds. An empty one is unset.
    pub(crate) fn held(&self, spelling: &Spelling) -> Option<&'a str> {
        let value = match spelling.id {
            "output" => self.output,
            _ => self.flags.get(spelling.variable),
        };
        value.filter(|value| !value.is_empty())
    }

    /// The spellings of `setting` whose variable names a value, with it. A
    /// boolean variable that reads as its flag's absent value names none.
    fn naming_a_value(&self, setting: &'static Setting) -> Vec<(&'static Spelling, &'a str)> {
        let held: Vec<_> = setting
            .spellings
            .iter()
            .filter_map(|spelling| Some((spelling, self.held(spelling)?)))
            .collect();
        if held.is_empty() {
            return held;
        }
        let command = Cli::command();
        held.into_iter()
            .filter(|(spelling, value)| {
                command
                    .get_arguments()
                    .find(|arg| arg.get_id() == spelling.id)
                    .is_none_or(|arg| !parse::reads_absent(arg, value))
            })
            .collect()
    }
}

/// A spelling named in argv, and the value typed with it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Typed {
    /// The spelling.
    pub(crate) spelling: &'static Spelling,
    /// The value attached with `=` or typed after it, for a spelling that
    /// takes one.
    pub(crate) value: Option<String>,
}

/// The spellings of `setting` that `args` name before a `--`, in the order
/// typed.
///
/// A whole token names a spelling: its flag alone, or its flag and `=`. So the
/// token `--cursor=--json` names `--cursor` and not `--json`. The read is exact
/// while every spelling is a long flag with no short form or alias and no
/// argument takes a hyphen-led value.
pub(crate) fn typed_in(args: &[OsString], setting: &'static Setting) -> Vec<Typed> {
    let mut found = Vec::new();
    let mut words = args.iter().skip(1).map(|word| word.to_string_lossy());
    while let Some(word) = words.next() {
        if word == "--" {
            break;
        }
        let Some(spelling) = setting.spellings.iter().find(|spelling| {
            word == spelling.flag
                || word
                    .strip_prefix(spelling.flag)
                    .is_some_and(|rest| rest.starts_with('='))
        }) else {
            continue;
        };
        let attached = word.split_once('=').map(|(_, value)| value.to_string());
        let value = match attached {
            None if spelling.takes_value => words.next().map(std::borrow::Cow::into_owned),
            attached => attached,
        };
        found.push(Typed { spelling, value });
    }
    found
}

/// What decided a setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Decision {
    /// A flag on the command line.
    Typed(&'static Spelling),
    /// The variable of a spelling.
    Variable(&'static Spelling),
    /// Neither: the setting's default.
    Default,
}

impl Decision {
    /// The spelling that decided, if any did.
    pub(crate) fn spelling(self) -> Option<&'static Spelling> {
        match self {
            Self::Typed(spelling) | Self::Variable(spelling) => Some(spelling),
            Self::Default => None,
        }
    }
}

/// A resolved setting: its value, and what decided it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Answer<T> {
    /// The value.
    pub(crate) value: T,
    /// What decided it.
    pub(crate) decision: Decision,
}

/// A refusal, as the body of its `invalid-args` message.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Refused(pub(crate) String);

/// The format the run prints in.
pub(crate) fn resolve_output(
    matches: &ArgMatches,
    args: &[OsString],
    variables: Variables<'_>,
) -> Result<Answer<OutputFormat>, Refused> {
    let decision = decide(&OUTPUT, matches, args, variables)?;
    let value = match decision {
        Decision::Typed(spelling) | Decision::Variable(spelling) if spelling.id == "json" => {
            OutputFormat::Json
        }
        Decision::Typed(spelling) | Decision::Variable(spelling) if spelling.id == "jsonl" => {
            OutputFormat::Jsonl
        }
        Decision::Variable(spelling) => {
            let held = variables.held(spelling).unwrap_or_default();
            read_output_variable(spelling, held)?
        }
        Decision::Typed(_) | Decision::Default => matches
            .get_one::<OutputFormat>("output")
            .cloned()
            .unwrap_or(OutputFormat::Text),
    };
    Ok(Answer { value, decision })
}

/// The token of the page to read, if one was asked for.
pub(crate) fn resolve_page_token(
    matches: &ArgMatches,
    args: &[OsString],
    variables: Variables<'_>,
) -> Result<Answer<Option<String>>, Refused> {
    let decision = decide(&PAGE_TOKEN, matches, args, variables)?;
    let value = match decision {
        Decision::Typed(spelling) => matches.get_one::<String>(spelling.id).cloned(),
        Decision::Variable(spelling) => variables.held(spelling).map(str::to_string),
        Decision::Default => None,
    }
    .filter(|token| !token.is_empty());
    Ok(Answer { value, decision })
}

/// The format the last output spelling in `args` asks for: `None` when argv
/// names no spelling, and `Some(None)` for a typed `text`, which asks for no
/// structure but is still typed. A value that names no format renders as JSON.
pub(crate) fn typed_format(args: &[OsString]) -> Option<Option<OutputFormat>> {
    let typed = typed_in(args, &OUTPUT);
    let last = typed
        .iter()
        .rev()
        .find(|typed| !typed.spelling.takes_value || typed.value.is_some())?;
    Some(match last.spelling.id {
        "json" => Some(OutputFormat::Json),
        "jsonl" => Some(OutputFormat::Jsonl),
        _ => last
            .value
            .as_deref()
            .and_then(crate::cli::classify::rendering_format),
    })
}

/// The format a usage error renders in when the caller named one in a
/// variable: `XURL_JSONL`, `XURL_JSON`, then `XURL_OUTPUT`, the first that
/// names a value. A value that names no format renders as JSON.
pub(crate) fn variable_format(variables: Variables<'_>) -> Option<OutputFormat> {
    let named = variables.naming_a_value(&OUTPUT);
    [("jsonl", OutputFormat::Jsonl), ("json", OutputFormat::Json)]
        .into_iter()
        .find(|(id, _)| named.iter().any(|(spelling, _)| spelling.id == *id))
        .map(|(_, format)| format)
        .or_else(|| {
            named
                .iter()
                .find(|(spelling, _)| spelling.id == "output")
                .and_then(|(_, value)| crate::cli::classify::rendering_format(value))
        })
}

fn decide(
    setting: &'static Setting,
    matches: &ArgMatches,
    args: &[OsString],
    variables: Variables<'_>,
) -> Result<Decision, Refused> {
    let typed: Vec<&'static Spelling> = setting
        .spellings
        .iter()
        .filter(|spelling| matches.value_source(spelling.id) == Some(ValueSource::CommandLine))
        .collect();
    match typed.as_slice() {
        [] => {}
        [only] => return Ok(Decision::Typed(only)),
        several => return Ok(Decision::Typed(typed_last(args, setting, several))),
    }
    match variables.naming_a_value(setting).as_slice() {
        [] => Ok(Decision::Default),
        [(spelling, _)] => {
            refuse_a_conflict(spelling, matches)?;
            Ok(Decision::Variable(spelling))
        }
        [(first, _), rest @ ..] => Err(two_variables(
            first,
            rest.iter().map(|(spelling, _)| *spelling),
        )),
    }
}

/// The one of `candidates` that `args` name last. The first of them when argv
/// names none, which it always does for a flag clap read from the command
/// line.
fn typed_last(
    args: &[OsString],
    setting: &'static Setting,
    candidates: &[&'static Spelling],
) -> &'static Spelling {
    typed_in(args, setting)
        .into_iter()
        .rev()
        .find(|typed| candidates.contains(&typed.spelling))
        .map_or(candidates[0], |typed| typed.spelling)
}

fn variable_side(spelling: &Spelling) -> Side {
    Side {
        name: spelling.variable.to_string(),
        source: Source::Variable(spelling.variable.to_string()),
    }
}

fn two_variables<'a>(first: &Spelling, rest: impl Iterator<Item = &'a Spelling>) -> Refused {
    let priors: Vec<Side> = rest.map(variable_side).collect();
    Refused(Refusal::between(&variable_side(first), &priors).body(None))
}

/// Refuses `spelling`'s variable when an argument that holds a value is one
/// clap declares in conflict with the flag.
fn refuse_a_conflict(spelling: &Spelling, matches: &ArgMatches) -> Result<(), Refused> {
    let mut root = parse::command();
    root.build();
    let Some((leaf, resolved)) = conflict_source::leaf_of(&root, matches) else {
        return Ok(());
    };
    let priors: Vec<Side> = conflicting(leaf, spelling.id)
        .into_iter()
        .filter_map(|arg| {
            let source = match resolved.value_source(arg.get_id().as_str())? {
                ValueSource::CommandLine => Source::Typed,
                ValueSource::EnvVariable => Source::Variable(arg.get_env()?.to_str()?.to_string()),
                _ => return None,
            };
            Some(Side {
                name: arg.to_string(),
                source,
            })
        })
        .collect();
    if priors.is_empty() {
        return Ok(());
    }
    let usage = leaf.clone().render_usage().to_string();
    let usage = usage.strip_prefix("Usage:").unwrap_or(&usage);
    Err(Refused(
        Refusal::between(&variable_side(spelling), &priors).body(Some(usage)),
    ))
}

/// The arguments of `command`, which is built, that clap declares in conflict
/// with the argument `id`, whichever of the two declares it: a conflict named
/// on either side, an argument that conflicts with every other, and the other
/// members of a group that takes one.
#[must_use]
pub fn conflicting<'a>(command: &'a Command, id: &str) -> Vec<&'a Arg> {
    let Some(arg) = command.get_arguments().find(|arg| arg.get_id() == id) else {
        return Vec::new();
    };
    command
        .get_arguments()
        .filter(|other| other.get_id() != arg.get_id())
        .filter(|other| {
            declares(command, arg, other)
                || declares(command, other, arg)
                || arg.is_exclusive_set()
                || other.is_exclusive_set()
                || share_a_single_group(command, arg, other)
        })
        .collect()
}

/// Whether `arg` names `other`, or a group holding it, in `conflicts_with`.
///
/// Clap's reader of one argument's conflicts takes a global argument down a
/// path that cannot name a group, so it reads a copy that is not global.
fn declares(command: &Command, arg: &Arg, other: &Arg) -> bool {
    command
        .get_arg_conflicts_with(&arg.clone().global(false))
        .iter()
        .any(|conflict| conflict.get_id() == other.get_id())
}

fn share_a_single_group(command: &Command, arg: &Arg, other: &Arg) -> bool {
    command.get_groups().any(|group| {
        let mut group = group.clone();
        let holds = |member: &Arg| group.get_args().any(|id| id == member.get_id());
        let both = holds(arg) && holds(other);
        both && !group.is_multiple()
    })
}

/// What `--output` reads `value` as, or the refusal that names the variable.
fn read_output_variable(spelling: &Spelling, value: &str) -> Result<OutputFormat, Refused> {
    let command = Cli::command();
    let Some(parser) = command
        .get_arguments()
        .find(|arg| arg.get_id() == spelling.id)
        .map(Arg::get_value_parser)
    else {
        return Ok(OutputFormat::Text);
    };
    read_with(parser, value).map_err(|error| {
        let mut message = format!(
            "invalid value '{value}' for the environment variable '{}'",
            spelling.variable
        );
        if let Some(ContextValue::Strings(valid)) = error.get(ContextKind::ValidValue) {
            message.push_str(&format!("\n  [possible values: {}]", valid.join(", ")));
        }
        Refused(message)
    })
}

fn read_with(parser: &ValueParser, value: &str) -> Result<OutputFormat, clap::Error> {
    let matches = parse::read_value(parser, value)?;
    Ok(matches
        .get_one::<OutputFormat>("value")
        .cloned()
        .unwrap_or(OutputFormat::Text))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use clap::ArgMatches;

    use super::{
        Answer, Decision, OUTPUT, PAGE_TOKEN, Refused, Variables, resolve_output,
        resolve_page_token, typed_in,
    };
    use crate::cli::env::FlagEnv;
    use crate::cli::output::OutputFormat;
    use crate::cli::parse;

    fn argv(words: &[&str]) -> Vec<OsString> {
        std::iter::once("xr")
            .chain(words.iter().copied())
            .map(OsString::from)
            .collect()
    }

    fn matches_of(args: &[OsString]) -> ArgMatches {
        parse::try_matches(args.iter(), std::convert::identity).expect("the argv parses")
    }

    fn flags(held: &[(&str, &str)]) -> FlagEnv {
        let value = |name: &str| {
            held.iter()
                .find(|(held_name, _)| *held_name == name)
                .map(|(_, value)| (*value).to_string())
        };
        FlagEnv {
            json: value("XURL_JSON"),
            jsonl: value("XURL_JSONL"),
            cursor: value("XURL_CURSOR"),
            after: value("XURL_AFTER"),
        }
    }

    fn output_of(
        words: &[&str],
        output: Option<&str>,
        held: &[(&str, &str)],
    ) -> Result<Answer<OutputFormat>, Refused> {
        let args = argv(words);
        let flags = flags(held);
        resolve_output(&matches_of(&args), &args, Variables::new(output, &flags))
    }

    fn token_of(words: &[&str], held: &[(&str, &str)]) -> Result<Answer<Option<String>>, Refused> {
        let args = argv(words);
        let flags = flags(held);
        resolve_page_token(&matches_of(&args), &args, Variables::new(None, &flags))
    }

    fn spelled(decision: Decision) -> (&'static str, &'static str) {
        match decision {
            Decision::Typed(spelling) => ("typed", spelling.id),
            Decision::Variable(spelling) => ("variable", spelling.variable),
            Decision::Default => ("default", ""),
        }
    }

    #[test]
    fn a_typed_spelling_answers_with_that_spelling_and_the_typed_source() {
        let answer = output_of(&["--json", "version"], Some("yaml"), &[]).expect("resolves");
        assert_eq!(answer.value, OutputFormat::Json);
        assert_eq!(spelled(answer.decision), ("typed", "json"));
    }

    #[test]
    fn a_variable_answers_with_its_spelling_and_its_name() {
        let answer = output_of(&["version"], Some("yml"), &[]).expect("resolves");
        assert_eq!(answer.value, OutputFormat::Yaml);
        assert_eq!(spelled(answer.decision), ("variable", "XURL_OUTPUT"));

        let answer = output_of(&["version"], None, &[("XURL_JSONL", "1")]).expect("resolves");
        assert_eq!(answer.value, OutputFormat::Jsonl);
        assert_eq!(spelled(answer.decision), ("variable", "XURL_JSONL"));
    }

    #[test]
    fn neither_answers_the_default() {
        let answer = output_of(&["version"], None, &[]).expect("resolves");
        assert_eq!(answer.value, OutputFormat::Text);
        assert_eq!(spelled(answer.decision), ("default", ""));

        let answer = token_of(&["version"], &[]).expect("resolves");
        assert_eq!(answer.value, None);
        assert_eq!(spelled(answer.decision), ("default", ""));
    }

    #[test]
    fn a_false_or_empty_variable_names_no_value() {
        let answer = output_of(
            &["version"],
            Some(""),
            &[("XURL_JSON", "0"), ("XURL_JSONL", "false")],
        )
        .expect("resolves");
        assert_eq!(spelled(answer.decision), ("default", ""));

        let answer = token_of(&["version"], &[("XURL_CURSOR", "")]).expect("resolves");
        assert_eq!(spelled(answer.decision), ("default", ""));
    }

    #[test]
    fn two_variables_are_refused_naming_both() {
        let Refused(body) =
            output_of(&["version"], Some("json"), &[("XURL_JSON", "1")]).expect_err("refused");
        assert_eq!(
            body,
            "the environment variable 'XURL_OUTPUT' cannot be used with the environment \
             variable 'XURL_JSON'\n\n  tip: unset one of 'XURL_OUTPUT' and 'XURL_JSON'"
        );
    }

    #[test]
    fn a_page_token_variable_beside_a_typed_page_is_refused() {
        let Refused(body) =
            token_of(&["--page", "2", "version"], &[("XURL_CURSOR", "a")]).expect_err("refused");
        assert!(
            body.starts_with(
                "the environment variable 'XURL_CURSOR' cannot be used with '--page <N>'\n\n  \
                 tip: unset 'XURL_CURSOR', or leave out '--page <N>'\n\nUsage:"
            ),
            "{body}"
        );
    }

    #[test]
    fn a_variable_that_names_no_format_is_refused_only_when_it_decides() {
        let Refused(body) = output_of(&["version"], Some("toml"), &[]).expect_err("refused");
        assert_eq!(
            body,
            "invalid value 'toml' for the environment variable 'XURL_OUTPUT'\n  \
             [possible values: text, json, jsonl, ndjson, yaml, csv, tsv]"
        );
        let answer = output_of(&["--json", "version"], Some("toml"), &[]).expect("resolves");
        assert_eq!(answer.value, OutputFormat::Json);
    }

    #[test]
    fn of_several_typed_the_last_wins_across_the_command_word() {
        let answer =
            output_of(&["--json", "version", "--output", "yaml"], None, &[]).expect("resolves");
        assert_eq!(answer.value, OutputFormat::Yaml);
        let answer =
            output_of(&["--output", "yaml", "version", "--json"], None, &[]).expect("resolves");
        assert_eq!(answer.value, OutputFormat::Json);
    }

    #[test]
    fn both_page_token_spellings_resolve_last_wins() {
        let answer = token_of(&["--cursor", "a", "--after", "b", "version"], &[]).expect("ok");
        assert_eq!(answer.value.as_deref(), Some("b"));
        let answer = token_of(&["--after", "b", "--cursor", "a", "version"], &[]).expect("ok");
        assert_eq!(answer.value.as_deref(), Some("a"));
    }

    fn spellings(words: &[&str]) -> Vec<(&'static str, Option<String>)> {
        typed_in(&argv(words), &OUTPUT)
            .into_iter()
            .map(|typed| (typed.spelling.id, typed.value))
            .collect()
    }

    #[test]
    fn the_argv_read_stops_at_a_double_dash() {
        assert_eq!(
            spellings(&["--json", "webhooks", "listen", "--", "prog", "--jsonl"]),
            [("json", None)]
        );
    }

    #[test]
    fn the_argv_read_takes_an_equals_form_and_a_following_value() {
        assert_eq!(
            spellings(&["--output=yaml", "--output", "csv", "--json"]),
            [
                ("output", Some("yaml".to_string())),
                ("output", Some("csv".to_string())),
                ("json", None),
            ]
        );
    }

    #[test]
    fn a_page_token_that_spells_a_flag_is_the_token_not_the_flag() {
        let words = ["--cursor=--json", "--after", "x"];
        assert_eq!(
            typed_in(&argv(&words), &PAGE_TOKEN)
                .into_iter()
                .map(|typed| (typed.spelling.id, typed.value))
                .collect::<Vec<_>>(),
            [
                ("cursor", Some("--json".to_string())),
                ("after", Some("x".to_string())),
            ]
        );
        assert_eq!(spellings(&words), []);
    }
}
