//! What an invocation is asking for, decided without touching the world.
//!
//! Every function here is pure: it reads a parsed [`Cli`], an argv slice, or a
//! clap error, and returns a decision. The runner owns the writers, the exit
//! codes, and the rendering; this module owns the question of what the caller
//! meant. That split is what lets classification run ahead of the config and
//! the token store, so a mistyped command never waits on a file read.

use std::ffi::OsString;

use clap::CommandFactory;
use clap::error::{ContextKind, ContextValue};

use crate::cli::Cli;
use crate::output::OutputFormat;

/// Jaro score a candidate must beat to be offered as the nearest command.
const SUGGESTION_THRESHOLD: f64 = 0.7;

/// What an invocation is asking for, once clap has parsed it.
pub(crate) enum Classified {
    /// Nothing to run: the root help answers it.
    Help,
    /// A word that names neither a command nor a raw target.
    UnknownCommand(String),
    /// A raw request, or a subcommand that dispatches normally.
    Raw,
}

/// Classifies a parsed invocation, reading no config and no store.
///
/// Running after the parse rather than over argv is what keeps this free of
/// the pre-parse pitfalls: clap has already consumed `help`, `--`, and every
/// value-taking flag, and global flags and aliases resolved with it.
pub(crate) fn classify(cli: &Cli) -> Classified {
    if cli.command.is_some() {
        return Classified::Raw;
    }
    // Every flag that only raw mode reads. Their presence says the caller
    // means a request, so the positional stays a URL.
    let raw_only_flag = cli.method.is_some()
        || !cli.headers.is_empty()
        || cli.data.is_some()
        || cli.auth_type.is_some()
        || cli.username.is_some()
        || cli.file.is_some()
        || cli.trace
        || cli.stream;

    let Some(word) = cli.url.as_deref() else {
        return if raw_only_flag {
            Classified::Raw
        } else {
            Classified::Help
        };
    };
    if word.starts_with("http://") || word.starts_with("https://") || word.starts_with('/') {
        return Classified::Raw;
    }
    if raw_only_flag {
        return Classified::Raw;
    }
    if is_command_word(word) {
        return Classified::UnknownCommand(word.to_string());
    }
    Classified::Raw
}

/// Whether the whole token reads as a command name: a letter, then letters,
/// digits, or hyphens.
fn is_command_word(word: &str) -> bool {
    let mut chars = word.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// The nearest root command to `word`, when one scores above the threshold.
pub(crate) fn nearest_command(word: &str) -> Option<String> {
    nearest(word, &root_candidates())
}

/// The nearest candidate to `word`, when one scores above the threshold.
///
/// Jaro rather than Jaro-Winkler, which clap avoids for subcommands because
/// the prefix weight over-favors a shared opening (clap-rs/clap#4660).
fn nearest(word: &str, candidates: &[String]) -> Option<String> {
    let word = word.to_lowercase();
    candidates
        .iter()
        .map(|candidate| (strsim::jaro(&word, candidate), candidate))
        .filter(|(score, _)| *score > SUGGESTION_THRESHOLD)
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, candidate)| candidate.clone())
}

/// Every name a root subcommand answers to.
fn root_candidates() -> Vec<String> {
    let cmd = Cli::command();
    let mut names: Vec<String> = cmd
        .get_subcommands()
        .flat_map(|sub| {
            std::iter::once(sub.get_name().to_string())
                .chain(sub.get_all_aliases().map(String::from))
        })
        .collect();
    // clap generates the `help` subcommand while building the command, so it
    // is absent from the declarations read above.
    names.push("help".to_string());
    names
}

/// The suggestion for a word clap itself rejected.
///
/// clap's own score comes first. Where clap left the word unscored and an
/// argv token `help` precedes it, the word is scored against the root
/// commands, which is the `xr help WORD` path.
pub(crate) fn suggestion_for_rejected(
    error: &clap::Error,
    args: &[OsString],
    word: &str,
) -> Option<String> {
    clap_suggestion(error).or_else(|| {
        args.iter()
            .take_while(|a| a.to_string_lossy() != word)
            .any(|a| a.to_string_lossy() == "help")
            .then(|| nearest_command(word))
            .flatten()
    })
}

/// clap's own suggestion for an unrecognized subcommand, where it scored one.
fn clap_suggestion(error: &clap::Error) -> Option<String> {
    match error.get(ContextKind::SuggestedSubcommand)? {
        ContextValue::String(value) => Some(value.clone()),
        ContextValue::Strings(values) => values.first().cloned(),
        _ => None,
    }
}

/// One string of clap error context.
pub(crate) fn context_string(error: &clap::Error, kind: ContextKind) -> Option<String> {
    match error.get(kind)? {
        ContextValue::String(value) => Some(value.clone()),
        _ => None,
    }
}

/// The output format the caller named before clap parsed anything.
///
/// Read from the unparsed argv, because the clap error path runs before `Cli`
/// exists, and from `XURL_OUTPUT` as supplied by the caller. `text` and any
/// spelling outside the set return `None`, which keeps clap's own rendering.
/// The last format named wins, matching how clap resolves a repeated flag.
pub(crate) fn structured_intent(args: &[OsString], output: Option<&str>) -> Option<OutputFormat> {
    let mut found = None;
    let mut iter = args.iter().peekable();
    while let Some(a) = iter.next() {
        let s = a.to_string_lossy();
        if s == "--json" {
            found = Some(OutputFormat::Json);
        } else if s == "--jsonl" {
            found = Some(OutputFormat::Jsonl);
        } else if s == "--output"
            && let Some(next) = iter.peek()
        {
            found = structured_format(&next.to_string_lossy()).or(found);
        } else if let Some(rest) = s.strip_prefix("--output=") {
            found = structured_format(rest).or(found);
        }
    }
    found.or_else(|| output.and_then(structured_format))
}

/// The structured format a spelling names, if any.
///
/// `yml` is here and absent from the value enum: a caller that spells YAML
/// that way gets the usage error rendered as YAML rather than as text.
fn structured_format(value: &str) -> Option<OutputFormat> {
    match value.to_ascii_lowercase().as_str() {
        "json" => Some(OutputFormat::Json),
        "jsonl" => Some(OutputFormat::Jsonl),
        "ndjson" => Some(OutputFormat::Ndjson),
        "yaml" | "yml" => Some(OutputFormat::Yaml),
        "csv" => Some(OutputFormat::Csv),
        "tsv" => Some(OutputFormat::Tsv),
        _ => None,
    }
}
