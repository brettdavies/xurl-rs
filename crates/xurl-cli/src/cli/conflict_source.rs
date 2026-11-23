//! The sentence for a conflict between arguments, naming each side by where
//! its value came from.
//!
//! Clap names each side of a conflict by its argument's flag, which misleads
//! when the value came from an environment variable: the caller typed no such
//! flag. A side that came from a variable is named as that variable, and the
//! refusal says which edit clears it.

use std::ffi::OsString;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::parser::ValueSource;
use clap::{Arg, ArgMatches, Command};

use crate::cli::{parse, reparse};

/// Where the value of an argument came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Source {
    /// The command line.
    Typed,
    /// An environment variable, by name.
    Variable(String),
}

/// One argument in a conflict, as clap spells it, and where its value came
/// from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Side {
    pub(crate) name: String,
    pub(crate) source: Source,
}

impl Side {
    fn is_variable(&self) -> bool {
        matches!(self.source, Source::Variable(_))
    }

    /// The side as the sentence's subject.
    fn subject(&self) -> String {
        match &self.source {
            Source::Variable(variable) => format!("the environment variable '{variable}'"),
            Source::Typed => format!("the argument '{}'", self.name),
        }
    }

    /// The side after "cannot be used with", where one side stands alone.
    fn alone(&self) -> String {
        match &self.source {
            Source::Variable(variable) => format!("the environment variable '{variable}'"),
            Source::Typed => format!("'{}'", self.name),
        }
    }

    /// The side as a line of a list of sides.
    fn listed(&self) -> String {
        match &self.source {
            Source::Variable(variable) => format!("the environment variable '{variable}'"),
            Source::Typed => self.name.clone(),
        }
    }
}

/// A conflict's sentence and the one line that says what clears it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Refusal {
    sentence: String,
    tip: Option<String>,
    every_side_is_a_variable: bool,
    usage: Option<String>,
}

impl Refusal {
    /// The sentence that refuses `subject` beside `priors`. A conflict with no
    /// prior is an `exclusive` argument's, which conflicts with every other.
    pub(crate) fn between(subject: &Side, priors: &[Side]) -> Self {
        let mut sentence = format!("{} cannot be used with", subject.subject());
        match priors {
            [] => sentence.push_str(" one or more of the other specified arguments"),
            [prior] => {
                sentence.push(' ');
                sentence.push_str(&prior.alone());
            }
            priors => {
                sentence.push(':');
                for prior in priors {
                    sentence.push_str("\n  ");
                    sentence.push_str(&prior.listed());
                }
            }
        }
        let sides: Vec<&Side> = std::iter::once(subject).chain(priors).collect();
        Self {
            sentence,
            tip: tip_for(&sides),
            every_side_is_a_variable: sides.iter().all(|side| side.is_variable()),
            usage: None,
        }
    }

    /// Whether a side came from a variable, so this sentence differs from
    /// clap's.
    fn names_a_variable(&self) -> bool {
        self.tip.is_some()
    }

    /// The refusal as a body: the sentence, the tip, and then the usage line
    /// from `usage` unless no typed flag is at fault.
    pub(crate) fn body(&self, usage: Option<&str>) -> String {
        let mut body = self.sentence.clone();
        if let Some(tip) = &self.tip {
            body.push_str("\n\n  tip: ");
            body.push_str(tip);
        }
        if let Some(usage) = usage
            && !self.every_side_is_a_variable
        {
            body.push_str("\n\nUsage:");
            body.push_str(usage);
        }
        body
    }

    /// `clap_body`, a conflict clap worded, with this sentence in place of
    /// clap's. The usage line is the command's own when this refusal holds
    /// one and clap's otherwise, and goes unless every side is a variable.
    pub(crate) fn replacing(&self, clap_body: &str) -> String {
        let clap_usage = clap_body.split_once("\n\nUsage:").map(|(_, usage)| usage);
        self.body(self.usage.as_deref().or(clap_usage))
    }
}

/// The edit that clears a conflict: the variables to unset, or the flags to
/// leave out. `None` when no side is a variable, as clap gives none.
fn tip_for(sides: &[&Side]) -> Option<String> {
    let variables: Vec<String> = sides
        .iter()
        .filter_map(|side| match &side.source {
            Source::Variable(variable) => Some(format!("'{variable}'")),
            Source::Typed => None,
        })
        .collect();
    let flags: Vec<String> = sides
        .iter()
        .filter(|side| !side.is_variable())
        .map(|side| format!("'{}'", side.name))
        .collect();
    if variables.is_empty() {
        return None;
    }
    if let Some(page) = sides.iter().find(|side| is_offset_page(side)) {
        return Some(match &page.source {
            Source::Variable(variable) => format!("unset '{variable}'"),
            Source::Typed => format!("leave out '{}'", page.name),
        });
    }
    let tip = match (variables.as_slice(), flags.as_slice()) {
        ([only], []) => format!("unset {only}"),
        ([_, _], []) => format!("unset one of {} and {}", variables[0], variables[1]),
        (_, []) => format!("unset all but one of {}", list(&variables)),
        (variables, flags) => format!(
            "unset {}, or leave out {}",
            join_and(variables),
            join_and(flags)
        ),
    };
    Some(tip)
}

/// Whether the side is `--page`, which every command that pages refuses, so
/// the other side of its conflict cannot clear the refusal by itself.
fn is_offset_page(side: &Side) -> bool {
    side.name.starts_with("--page ")
        || matches!(&side.source, Source::Variable(variable) if variable == "XURL_PAGE")
}

/// `'A', 'B', and 'C'`.
fn list(items: &[String]) -> String {
    match items {
        [rest @ .., last] if !rest.is_empty() => format!("{}, and {last}", rest.join(", ")),
        _ => items.join(""),
    }
}

/// `'A'`, `'A' and 'B'`.
fn join_and(items: &[String]) -> String {
    items.join(" and ")
}

/// The sides of a conflict clap raised, found among the arguments of
/// `command`, each with the source `source_of` gives it. `None` for any other
/// error, for the form clap gives a repeated argument, and for a context string
/// that maps to no argument: clap's own sentence stands for those.
pub(crate) fn sides(
    error: &clap::Error,
    command: &Command,
    source_of: &dyn Fn(&Arg) -> Source,
) -> Option<(Side, Vec<Side>)> {
    if error.kind() != ErrorKind::ArgumentConflict {
        return None;
    }
    let Some(ContextValue::String(invalid)) = error.get(ContextKind::InvalidArg) else {
        return None;
    };
    let side_of = |name: &str| {
        command
            .get_arguments()
            .find(|arg| arg.to_string() == name)
            .map(|arg| Side {
                name: name.to_string(),
                source: source_of(arg),
            })
    };
    let subject = side_of(invalid)?;
    let priors = match error.get(ContextKind::PriorArg) {
        Some(ContextValue::String(prior)) if prior == invalid => return None,
        Some(ContextValue::String(prior)) => vec![side_of(prior)?],
        Some(ContextValue::Strings(priors)) => priors
            .iter()
            .map(|prior| side_of(prior))
            .collect::<Option<Vec<_>>>()?,
        None | Some(_) => Vec::new(),
    };
    Some((subject, priors))
}

/// The command `matches` names, found in the built `root`, and the matches
/// at its level.
pub(crate) fn leaf_of<'a>(
    root: &'a Command,
    matches: &'a ArgMatches,
) -> Option<(&'a Command, &'a ArgMatches)> {
    let (mut command, mut resolved) = (root, matches);
    while let Some((name, below)) = resolved.subcommand() {
        command = command.find_subcommand(name)?;
        resolved = below;
    }
    Some((command, resolved))
}

/// The source of the value `arg` holds in `matches`.
pub(crate) fn source_in(matches: &ArgMatches, arg: &Arg) -> Source {
    match (
        matches.value_source(arg.get_id().as_str()),
        arg.get_env().and_then(|name| name.to_str()),
    ) {
        (Some(ValueSource::EnvVariable), Some(variable)) => Source::Variable(variable.to_string()),
        _ => Source::Typed,
    }
}

/// The refusal a clap conflict in an invocation `args` takes, when a side of
/// it came from a variable.
///
/// The sources come from what clap resolved before it stopped, at the command
/// the invocation names.
pub(crate) fn refusal_for(error: &clap::Error, args: &[OsString]) -> Option<Refusal> {
    if error.kind() != ErrorKind::ArgumentConflict {
        return None;
    }
    let matches = reparse::lenient_matches(args)?;
    let mut root = parse::command();
    root.build();
    let (command, resolved) = leaf_of(&root, &matches)?;
    let (subject, priors) = sides(error, command, &|arg| source_in(resolved, arg))?;
    let mut refusal = Refusal::between(&subject, &priors);
    let usage = command.clone().render_usage().to_string();
    refusal.usage = Some(usage.strip_prefix("Usage:").unwrap_or(&usage).to_string());
    refusal.names_a_variable().then_some(refusal)
}

#[cfg(test)]
mod tests {
    use clap::error::{ContextKind, ContextValue, ErrorKind};
    use clap::{Arg, ArgAction, Command};

    use super::{Refusal, Side, Source, sides};

    fn typed(name: &str) -> Side {
        Side {
            name: name.to_string(),
            source: Source::Typed,
        }
    }

    fn variable(variable: &str) -> Side {
        Side {
            name: variable.to_string(),
            source: Source::Variable(variable.to_string()),
        }
    }

    fn fixture() -> Command {
        let mut command = Command::new("x")
            .arg(Arg::new("page").long("page").value_name("N"))
            .arg(Arg::new("cursor").long("cursor").value_name("TOKEN"))
            .arg(Arg::new("after").long("after").value_name("TOKEN"))
            .arg(Arg::new("all").long("all").action(ArgAction::SetTrue));
        command.build();
        command
    }

    fn conflict(invalid: &str, prior: Option<ContextValue>) -> clap::Error {
        let mut error = clap::Error::new(ErrorKind::ArgumentConflict);
        error.insert(
            ContextKind::InvalidArg,
            ContextValue::String(invalid.to_string()),
        );
        if let Some(prior) = prior {
            error.insert(ContextKind::PriorArg, prior);
        }
        error
    }

    fn source(arg: &Arg) -> Source {
        match arg.get_id().as_str() {
            "page" => Source::Variable("XURL_PAGE".to_string()),
            _ => Source::Typed,
        }
    }

    #[test]
    fn a_variable_prior_is_named_as_the_variable() {
        let error = conflict(
            "--cursor <TOKEN>",
            Some(ContextValue::String("--page <N>".to_string())),
        );
        let (subject, priors) = sides(&error, &fixture(), &source).expect("sides");
        let refusal = Refusal::between(&subject, &priors);
        assert_eq!(
            refusal.sentence,
            "the argument '--cursor <TOKEN>' cannot be used with the environment variable 'XURL_PAGE'"
        );
        assert_eq!(refusal.tip.as_deref(), Some("unset 'XURL_PAGE'"));
    }

    #[test]
    fn two_priors_are_each_named_by_their_own_source() {
        let error = conflict(
            "--cursor <TOKEN>",
            Some(ContextValue::Strings(vec![
                "--page <N>".to_string(),
                "--after <TOKEN>".to_string(),
            ])),
        );
        let (subject, priors) = sides(&error, &fixture(), &source).expect("sides");
        let refusal = Refusal::between(&subject, &priors);
        assert_eq!(
            refusal.sentence,
            "the argument '--cursor <TOKEN>' cannot be used with:\n  \
             the environment variable 'XURL_PAGE'\n  --after <TOKEN>"
        );
        assert_eq!(refusal.tip.as_deref(), Some("unset 'XURL_PAGE'"));
    }

    #[test]
    fn an_exclusive_argument_names_its_own_source_and_no_prior() {
        let error = conflict("--page <N>", None);
        let (subject, priors) = sides(&error, &fixture(), &source).expect("sides");
        assert!(priors.is_empty());
        let refusal = Refusal::between(&subject, &priors);
        assert_eq!(
            refusal.sentence,
            "the environment variable 'XURL_PAGE' cannot be used with one or more of the other specified arguments"
        );
    }

    #[test]
    fn a_context_string_that_maps_to_no_argument_leaves_clap_sentence() {
        let error = conflict(
            "--cursor <TOKEN>",
            Some(ContextValue::String("--nothing".to_string())),
        );
        assert!(sides(&error, &fixture(), &source).is_none());
    }

    #[test]
    fn the_repeat_form_leaves_clap_sentence() {
        let error = conflict(
            "--cursor <TOKEN>",
            Some(ContextValue::String("--cursor <TOKEN>".to_string())),
        );
        assert!(sides(&error, &fixture(), &source).is_none());
    }

    #[test]
    fn a_conflict_of_typed_sides_has_no_tip() {
        let refusal = Refusal::between(&typed("--a"), &[typed("--b")]);
        assert_eq!(refusal.tip, None);
        assert!(!refusal.names_a_variable());
    }

    #[test]
    fn variables_on_every_side_drop_the_usage_line() {
        let refusal = Refusal::between(&variable("XURL_CURSOR"), &[variable("XURL_PAGE")]);
        assert_eq!(
            refusal.body(Some(" xr schema [OPTIONS]")),
            "the environment variable 'XURL_CURSOR' cannot be used with the environment variable 'XURL_PAGE'\n\n  \
             tip: unset 'XURL_PAGE'"
        );
    }

    #[test]
    fn a_typed_side_keeps_the_usage_line_after_the_tip() {
        let refusal = Refusal::between(&typed("--cursor <TOKEN>"), &[variable("XURL_PAGE")]);
        assert_eq!(
            refusal.replacing("the argument '--cursor <TOKEN>' cannot be used with '--page <N>'\n\nUsage: xr --cursor <TOKEN>"),
            "the argument '--cursor <TOKEN>' cannot be used with the environment variable 'XURL_PAGE'\n\n  \
             tip: unset 'XURL_PAGE'\n\nUsage: xr --cursor <TOKEN>"
        );
    }

    #[test]
    fn three_variables_get_the_all_but_one_tip() {
        let refusal = Refusal::between(
            &variable("XURL_OUTPUT"),
            &[variable("XURL_JSON"), variable("XURL_JSONL")],
        );
        assert_eq!(
            refusal.tip.as_deref(),
            Some("unset all but one of 'XURL_OUTPUT', 'XURL_JSON', and 'XURL_JSONL'")
        );
        assert_eq!(
            refusal.sentence,
            "the environment variable 'XURL_OUTPUT' cannot be used with:\n  \
             the environment variable 'XURL_JSON'\n  the environment variable 'XURL_JSONL'"
        );
    }
}
