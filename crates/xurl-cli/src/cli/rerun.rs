//! What a step keeps of the run it answers.
//!
//! A recovery step is a command the caller runs next, so it has to run the
//! way the failed run did: answering in the output format the caller parses,
//! and, when it runs the same invocation again, under the app and the
//! rehearsal the invocation named. Two functions decide that, and nothing else
//! composes it:
//!
//! - [`explicit_invocation`] writes the invocation with the flags the
//!   environment chose (`XURL_APP`, `XURL_DRY_RUN`) spelled out, so a step
//!   built by rerunning it holds them as flags and runs the same in a shell
//!   without the variable.
//! - [`carry`] is the only way to make a [`CarriedStep`], and an envelope
//!   holds nothing else. It puts the output format after the program word of
//!   every step that runs `xr` and prints a document.
//!
//! `--app` and `--dry-run` reach only the steps built by rerunning the
//! invocation, because they depend on what the step does: a rerun keeps the
//! rehearsal, and any other step that stores a credential or signs in is not a
//! rehearsal.

use std::ffi::OsString;
use std::ops::Deref;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::cli::Cli;
use crate::cli::classify::ROOT_COMMAND;
use crate::cli::hints::{NextAction, NextStep};
use crate::cli::output::OutputFormat;
use crate::cli::settings::Decision;

/// A step that [`carry`] has put the run's output format on, the only kind an
/// envelope holds. The wire shape is the step's own.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct CarriedStep(NextStep);

impl schemars::JsonSchema for CarriedStep {
    fn inline_schema() -> bool {
        NextStep::inline_schema()
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        NextStep::schema_name()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        NextStep::schema_id()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        NextStep::json_schema(generator)
    }
}

/// The fields of a [`NextStep`] as [`CarriedStep`] writes them. `NextStep` does
/// not implement `Serialize`, so a step reaches a document only through
/// `carry`.
#[derive(Serialize)]
struct Wire<'a> {
    action: NextAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    template: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: &'a Option<String>,
}

impl Serialize for CarriedStep {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Wire {
            action: self.0.action,
            command: &self.0.command,
            template: &self.0.template,
            docs: &self.0.docs,
        }
        .serialize(serializer)
    }
}

impl CarriedStep {
    /// The step as an `xr` invocation or its absence reads.
    #[must_use]
    pub fn step(&self) -> &NextStep {
        &self.0
    }
}

impl Deref for CarriedStep {
    type Target = NextStep;

    fn deref(&self) -> &NextStep {
        &self.0
    }
}

/// Whether a step of this action keeps the run's output format.
///
/// The action decides because only one kind of step differs: help text does
/// not change with the format. Every other action either runs `xr` and prints
/// a document the caller parses, or carries no command and has nothing to
/// change. A row is written for each action, so none keeps a default.
const FORMAT_POLICY: &[(NextAction, bool)] = &[
    // Stores a credential through `xr auth apps add`.
    (NextAction::RegisterApp, true),
    // `xr auth oauth2`, which prints the authorize URL or the stored login.
    (NextAction::SignIn, true),
    // The invocation again under another app.
    (NextAction::SelectApp, true),
    // `xr auth status`, which lists the store.
    (NextAction::InspectStore, true),
    // A docs page; no command.
    (NextAction::EnrollApp, true),
    // `xr <command> --help` prints help in every format.
    (NextAction::ShowHelp, false),
    // The status read, `xr media status <id> --wait=<secs>`.
    (NextAction::ResumeWait, true),
    // The caller's own request again; no command.
    (NextAction::WaitAndRetry, true),
    // The caller's own request again; no command.
    (NextAction::Retry, true),
    // The invocation with its input changed, or the command that stores one.
    (NextAction::FixInput, true),
    // The issue tracker; no command.
    (NextAction::ReportIssue, true),
    // The invocation with `--force`.
    (NextAction::Confirm, true),
    // `xr skill install` or `xr skill update`.
    (NextAction::RunCommand, true),
];

/// Every action the policy names, for the test that holds the table to the
/// schema's list.
#[doc(hidden)]
#[must_use]
pub fn policy_actions() -> Vec<NextAction> {
    FORMAT_POLICY.iter().map(|(action, _)| *action).collect()
}

fn keeps_format(action: NextAction) -> bool {
    FORMAT_POLICY
        .iter()
        .find(|(known, _)| *known == action)
        .is_none_or(|(_, keeps)| *keeps)
}

/// The global flags a step can keep, by clap id: the output format (`output`
/// and its two aliases), the app, and the rehearsal.
#[doc(hidden)]
pub const CARRIED_GLOBALS: &[&str] = &["output", "json", "jsonl", "app", "dry_run"];

/// The global flags no step keeps, each with the reason. A flag that is
/// neither here nor in [`CARRIED_GLOBALS`] fails the walk that reads clap's
/// declarations.
#[doc(hidden)]
pub const DROPPED_GLOBALS: &[(&str, &str)] = &[
    (
        "verbose",
        "logs the failed run; a step runs the same without it",
    ),
    (
        "quiet",
        "drops advice from a human's terminal; a document is the same",
    ),
    ("color", "colors a human's terminal; a document has none"),
    (
        "no_pager",
        "pages a human's terminal; a document is not paged",
    ),
    (
        "raw",
        "compacts the rendering of a response, not what the step does",
    ),
    (
        "no_interactive",
        "a step is run by whoever follows it, who knows if a prompt can be answered",
    ),
    (
        "timeout",
        "tunes one request's patience, not what the step means",
    ),
    (
        "wait_on_rate_limit",
        "tunes one request's patience, not what the step means",
    ),
    (
        "rate_limit_max_wait",
        "tunes one request's patience, not what the step means",
    ),
    (
        "limit",
        "sizes the page of a read; a rerun keeps the one typed, and the results are what vary",
    ),
    (
        "cursor",
        "names the page of a read; a rerun keeps the one typed, and the results are what vary",
    ),
    (
        "page",
        "refused before any request; a rerun keeps the one typed",
    ),
    (
        "after",
        "names the page of a read; a rerun keeps the one typed, and the results are what vary",
    ),
];

/// How a run spelled its output format, which a step repeats: the spelling
/// that decided the format, so a caller who typed `--json` gets a step that
/// says `--json`, and one who typed `--output` gets `--output`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormatSpelling {
    /// `--output <format>`, as `--output` and `XURL_OUTPUT` decide it, or
    /// neither of the three spellings did.
    #[default]
    Output,
    /// `--json` or `--jsonl`, as that flag or its variable decides it.
    Alias,
}

impl FormatSpelling {
    /// The spelling that decided the output format: an alias when `--json`,
    /// `--jsonl`, or the variable of one did, `--output` otherwise.
    #[must_use]
    pub(crate) fn of(decision: Decision) -> Self {
        match decision.spelling() {
            Some(spelling) if matches!(spelling.id, "json" | "jsonl") => Self::Alias,
            _ => Self::Output,
        }
    }
}

/// `step` with the output format after the program word of its `command` and
/// of its `template`, unless it names one already or the action prints no
/// document.
///
/// A step with a command or template names the `xr` word, so the format has
/// a place to go; one that does not fails in a debug build, where every test
/// runs.
#[must_use]
pub fn carry(mut step: NextStep, format: &OutputFormat, spelling: FormatSpelling) -> CarriedStep {
    let words = keeps_format(step.action)
        .then(|| format_words(format, spelling))
        .flatten();
    for text in [&mut step.command, &mut step.template]
        .into_iter()
        .flatten()
    {
        *text = put_after_program(text, words.as_deref());
    }
    CarriedStep(step)
}

/// `command`, an `xr` invocation named in prose, with the output format after
/// its program word, as [`carry`] puts it on a step. For a structured
/// document's instruction that names a command outside a `next_step`.
#[must_use]
pub fn name_format(command: &str, format: &OutputFormat, spelling: FormatSpelling) -> String {
    put_after_program(command, format_words(format, spelling).as_deref())
}

/// The words that name `format` as `spelling` does, or `None` for text, which
/// is the default.
fn format_words(format: &OutputFormat, spelling: FormatSpelling) -> Option<Vec<String>> {
    match (spelling, format) {
        (_, OutputFormat::Text) => None,
        (FormatSpelling::Alias, OutputFormat::Json) => Some(vec!["--json".to_string()]),
        (FormatSpelling::Alias, OutputFormat::Jsonl) => Some(vec!["--jsonl".to_string()]),
        _ => format
            .to_possible_value()
            .map(|value| vec!["--output".to_string(), value.get_name().to_string()]),
    }
}

fn put_after_program(text: &str, words: Option<&[String]>) -> String {
    let tokens: Vec<&str> = text.split(' ').collect();
    let at = tokens.iter().position(|token| *token == ROOT_COMMAND);
    debug_assert!(
        at.is_some(),
        "a step names no `{ROOT_COMMAND}` word to put the run's flags after: {text}"
    );
    let (Some(at), Some(words)) = (at, words) else {
        return text.to_string();
    };
    if names_output(&tokens[at + 1..]) {
        return text.to_string();
    }
    let mut kept = tokens[..=at].to_vec();
    kept.extend(words.iter().map(String::as_str));
    kept.extend(&tokens[at + 1..]);
    kept.join(" ")
}

/// Whether the flags of `words`, up to a `--`, name an output format, as
/// `--output NAME`, `--output=NAME`, or one of its two aliases. What follows
/// the `--` is a child program's own arguments.
fn names_output(words: &[&str]) -> bool {
    words.iter().take_while(|word| **word != "--").any(|word| {
        matches!(*word, "--output" | "--json" | "--jsonl") || word.starts_with("--output=")
    })
}

/// Whether the flags of `words`, up to a `--`, name `flag` in either spelling.
fn names_flag<S: AsRef<str>>(words: &[S], flag: &str) -> bool {
    words
        .iter()
        .skip(1)
        .map(AsRef::as_ref)
        .take_while(|word| *word != "--")
        .any(|word| {
            word == flag
                || word
                    .strip_prefix(flag)
                    .is_some_and(|rest| rest.starts_with('='))
        })
}

/// The words `args` give, with the flags the environment chose written out
/// after the program word: `--dry-run` for a run `XURL_DRY_RUN` made a dry
/// run, and `--app` for the app `XURL_APP` named. A flag the caller typed
/// stays where it is.
#[must_use]
pub fn explicit_invocation(args: &[OsString], cli: &Cli) -> Vec<String> {
    let mut words: Vec<String> = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let mut chosen = Vec::new();
    if cli.dry_run && !names_flag(&words, "--dry-run") {
        chosen.push("--dry-run".to_string());
    }
    if let Some(app) = cli.app.as_deref().filter(|app| !app.is_empty())
        && !names_flag(&words, "--app")
    {
        chosen.extend(["--app".to_string(), app.to_string()]);
    }
    let program = words.len().min(1);
    words.splice(program..program, chosen);
    words
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    fn command(step: &CarriedStep) -> &str {
        step.command.as_deref().unwrap_or_default()
    }

    #[test]
    fn the_format_goes_after_the_program_word_of_a_command_and_of_a_template() {
        let status = carry(
            NextStep::fix_input_with("xr auth status".to_string()),
            &OutputFormat::Json,
            FormatSpelling::Output,
        );
        assert_eq!(command(&status), "xr --output json auth status");
        let registration = carry(
            NextStep::register_app(),
            &OutputFormat::Yaml,
            FormatSpelling::Output,
        );
        assert_eq!(
            registration.template.as_deref(),
            Some(
                "<secret-command> | xr --output yaml auth apps add <name> --client-id \
                 <client-id> --client-secret-file -"
            )
        );
    }

    #[test]
    fn a_format_the_step_names_is_not_named_twice() {
        for named in [
            "xr --output json auth status",
            "xr --output=yaml auth status",
            "xr --json auth status",
            "xr auth status --jsonl",
        ] {
            let step = NextStep::fix_input_with(named.to_string());
            assert_eq!(
                command(&carry(step, &OutputFormat::Json, FormatSpelling::Output)),
                named
            );
        }
    }

    #[test]
    fn a_word_after_the_double_dash_belongs_to_the_child_program() {
        for child in ["cat", "cat --json", "curl --output=f", "tool --jsonl"] {
            let step = NextStep::fix_input_with(format!("xr webhooks listen --port 0 -- {child}"));
            assert_eq!(
                command(&carry(step, &OutputFormat::Json, FormatSpelling::Output)),
                format!("xr --output json webhooks listen --port 0 -- {child}")
            );
        }
    }

    #[test]
    fn the_format_is_spelled_as_the_run_spelled_it() {
        for (format, spelling, expected) in [
            (OutputFormat::Json, FormatSpelling::Output, "--output json"),
            (OutputFormat::Json, FormatSpelling::Alias, "--json"),
            (
                OutputFormat::Jsonl,
                FormatSpelling::Output,
                "--output jsonl",
            ),
            (OutputFormat::Jsonl, FormatSpelling::Alias, "--jsonl"),
            (OutputFormat::Yaml, FormatSpelling::Output, "--output yaml"),
        ] {
            let step = NextStep::fix_input_with("xr auth status".to_string());
            assert_eq!(
                command(&carry(step, &format, spelling)),
                format!("xr {expected} auth status")
            );
            assert_eq!(
                name_format("xr auth status", &format, spelling),
                format!("xr {expected} auth status")
            );
        }
    }

    #[test]
    fn the_spelling_follows_the_spelling_that_decided_the_format() {
        use crate::cli::env::FlagEnv;
        use crate::cli::settings::{Variables, resolve_output};

        let json_variable = FlagEnv {
            json: Some("1".to_string()),
            ..FlagEnv::default()
        };
        let none = FlagEnv::default();
        for (args, output, flags, expected) in [
            (vec!["xr", "whoami"], None, &none, FormatSpelling::Output),
            (
                vec!["xr", "--output", "json", "whoami"],
                None,
                &none,
                FormatSpelling::Output,
            ),
            (
                vec!["xr", "--json", "whoami"],
                None,
                &none,
                FormatSpelling::Alias,
            ),
            (
                vec!["xr", "--jsonl", "whoami"],
                None,
                &none,
                FormatSpelling::Alias,
            ),
            (
                vec!["xr", "whoami"],
                Some("yaml"),
                &none,
                FormatSpelling::Output,
            ),
            (
                vec!["xr", "whoami"],
                None,
                &json_variable,
                FormatSpelling::Alias,
            ),
            (
                vec!["xr", "--output", "yaml", "whoami"],
                None,
                &json_variable,
                FormatSpelling::Output,
            ),
        ] {
            let args: Vec<OsString> = args.into_iter().map(OsString::from).collect();
            let matches = crate::cli::parse::try_matches(args.iter(), std::convert::identity)
                .expect("parses");
            let answer =
                resolve_output(&matches, &args, Variables::new(output, flags)).expect("resolves");
            assert_eq!(FormatSpelling::of(answer.decision), expected, "{args:?}");
        }
    }

    #[test]
    fn text_and_help_keep_the_step_as_built() {
        let status = NextStep::fix_input_with("xr auth status".to_string());
        assert_eq!(
            *carry(status.clone(), &OutputFormat::Text, FormatSpelling::Output),
            status
        );
        let help = NextStep::show_help("xr whoami --help".to_string());
        assert_eq!(
            *carry(help.clone(), &OutputFormat::Json, FormatSpelling::Output),
            help
        );
    }

    #[test]
    fn a_step_without_a_command_or_a_template_has_nowhere_to_put_a_flag() {
        let docs = NextStep::enroll_app("https://example.com".to_string());
        assert_eq!(
            *carry(docs.clone(), &OutputFormat::Json, FormatSpelling::Output),
            docs
        );
    }

    #[test]
    #[should_panic(expected = "names no `xr` word")]
    fn a_step_that_names_no_program_word_fails_loudly() {
        let _ = carry(
            NextStep::fix_input_with("auth status".to_string()),
            &OutputFormat::Text,
            FormatSpelling::Output,
        );
    }

    fn one_step_of_each_action() -> Vec<NextStep> {
        let invocation: Vec<String> = ["xr", "lists", "remove", "42"]
            .iter()
            .map(ToString::to_string)
            .collect();
        vec![
            NextStep::register_app(),
            NextStep::sign_in(Some("work"), true),
            NextStep::select_app("xr auth default <app>".to_string()),
            NextStep::inspect_store(),
            NextStep::enroll_app("https://example.com".to_string()),
            NextStep::show_help("xr whoami --help".to_string()),
            NextStep::resume_wait("4242", 2, &Default::default()),
            NextStep::wait_and_retry("https://example.com".to_string()),
            NextStep::retry(None),
            NextStep::fix_input_with("xr auth status".to_string()),
            NextStep::report_issue("https://example.com".to_string()),
            NextStep::confirm(&invocation, Some("work")),
            NextStep::run_command("xr skill install codex".to_string()),
        ]
    }

    #[test]
    fn every_action_in_the_policy_has_a_step_here_and_each_decides_as_its_row_says() {
        let steps = one_step_of_each_action();
        let mut named: Vec<NextAction> = steps.iter().map(|step| step.action).collect();
        let mut policy = policy_actions();
        named.sort_by_key(|action| format!("{action:?}"));
        policy.sort_by_key(|action| format!("{action:?}"));
        assert_eq!(named, policy, "a row and its representative step disagree");
        for step in steps {
            let carried = carry(step.clone(), &OutputFormat::Json, FormatSpelling::Output);
            let runs_xr = step.command.is_some() || step.template.is_some();
            let named_format = [&carried.command, &carried.template]
                .into_iter()
                .flatten()
                .all(|text| text.contains("--output json"));
            if !runs_xr {
                assert_eq!(*carried, step, "{:?}", step.action);
            } else if keeps_format(step.action) {
                assert!(named_format, "{:?}: {carried:?}", step.action);
            } else {
                assert_eq!(*carried, step, "{:?}", step.action);
            }
        }
    }

    fn explicit(words: &[&str], dry_run: bool, app: Option<&str>) -> Vec<String> {
        let mut cli = Cli::try_parse_from(words).expect("parses");
        cli.dry_run = cli.dry_run || dry_run;
        cli.app = cli.app.or_else(|| app.map(str::to_string));
        let args: Vec<OsString> = words.iter().map(OsString::from).collect();
        explicit_invocation(&args, &cli)
    }

    #[test]
    fn the_flags_the_environment_chose_are_written_after_the_program_word() {
        assert_eq!(
            explicit(&["xr", "whoami"], true, Some("work")),
            ["xr", "--dry-run", "--app", "work", "whoami"]
        );
        assert_eq!(explicit(&["xr", "whoami"], false, None), ["xr", "whoami"]);
        assert_eq!(
            explicit(&["xr", "whoami"], false, Some("")),
            ["xr", "whoami"]
        );
    }

    #[test]
    fn a_flag_the_caller_typed_is_not_written_again() {
        let dry_runs: [&[&str]; 3] = [
            &["xr", "--dry-run", "whoami"],
            &["xr", "--dry-run=true", "whoami"],
            &["xr", "whoami", "--dry-run"],
        ];
        for typed in dry_runs {
            assert_eq!(explicit(typed, true, None), typed, "{typed:?}");
        }
        let apps: [&[&str]; 2] = [
            &["xr", "--app", "side", "whoami"],
            &["xr", "--app=side", "whoami"],
        ];
        for typed in apps {
            assert_eq!(explicit(typed, false, None), typed, "{typed:?}");
        }
    }

    #[test]
    fn what_follows_a_double_dash_is_not_read_as_a_flag() {
        let words = explicit(
            &["xr", "webhooks", "listen", "--", "tool", "--dry-run"],
            true,
            None,
        );
        assert_eq!(
            words,
            [
                "xr",
                "--dry-run",
                "webhooks",
                "listen",
                "--",
                "tool",
                "--dry-run"
            ]
        );
    }
}
