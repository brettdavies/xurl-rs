//! The words a caller names a command with, and what `xr schema` refuses.

use clap::CommandFactory;
use xdk::error::Error;

use crate::cli::Cli;
use crate::cli::classify::{ROOT_COMMAND, nearest};

use super::{file_stem, registered_commands};

/// Names `xr schema` took before it took command words, which no row's
/// words joined with `-` reproduce: the old name, then the row it meant.
const IRREGULAR_OLD_NAMES: &[(&str, &str)] = &[
    ("redirect-uri-get", "auth apps redirect-uri get"),
    ("redirect-uri-set", "auth apps redirect-uri set"),
];

/// The long flags `xr schema` declares, which name its scopes. A bare word
/// spelled like one is refused rather than read as a command.
#[must_use]
pub fn scope_words() -> Vec<String> {
    Cli::command()
        .find_subcommand("schema")
        .map(|schema| {
            schema
                .get_arguments()
                .filter_map(|arg| arg.get_long().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Why `xr schema` refused the words it was given.
#[derive(Debug)]
pub enum Refusal {
    /// A word names no command at its depth, as `xr help` reports it.
    UnknownCommand {
        /// The word that named nothing.
        word: String,
        /// The nearest command at that depth, when one scored close enough.
        suggestion: Option<String>,
        /// The command the word was typed under, `xr` at the root.
        under: String,
    },
    /// A name `xr schema` took before it took command words.
    OldName {
        /// The name as given.
        given: String,
        /// The registry row it meant.
        command: &'static str,
    },
    /// A bare word spelled like one of `xr schema`'s scope flags.
    ScopeWord(String),
    /// Anything else, refused as a value: no typed response, no arguments.
    Invalid(Error),
}

/// The registry row the words name.
pub(super) fn resolve(words: &[String]) -> std::result::Result<&'static str, Refusal> {
    if let [word] = words
        && scope_words().iter().any(|scope| scope == word)
    {
        return Err(Refusal::ScopeWord(word.clone()));
    }
    let joined = words.join(" ");
    if let Some(command) = registered_commands().find(|command| *command == joined) {
        return Ok(command);
    }
    let root = Cli::command();
    let mut current = &root;
    let mut path: Vec<&str> = Vec::new();
    for word in words {
        let found = current
            .get_subcommands()
            .find(|sub| sub.get_name() == word || sub.get_all_aliases().any(|alias| alias == word));
        let Some(sub) = found else {
            if let [only] = words
                && let Some(command) = old_name(only)
            {
                return Err(Refusal::OldName {
                    given: only.clone(),
                    command,
                });
            }
            let candidates: Vec<String> = current
                .get_subcommands()
                .flat_map(|sub| {
                    std::iter::once(sub.get_name().to_string())
                        .chain(sub.get_all_aliases().map(String::from))
                })
                .collect();
            let under = std::iter::once(ROOT_COMMAND)
                .chain(path.iter().copied())
                .collect::<Vec<_>>()
                .join(" ");
            return Err(Refusal::UnknownCommand {
                word: word.clone(),
                suggestion: nearest(word, &candidates),
                under,
            });
        };
        path.push(sub.get_name());
        current = sub;
    }
    let named = path.join(" ");
    if let Some(command) = registered_commands().find(|command| *command == named) {
        return Ok(command);
    }
    Err(Refusal::Invalid(Error::validation_of(
        "no-typed-response",
        named,
    )))
}

/// The row an old hyphen-joined name meant, when `token` is one.
fn old_name(token: &str) -> Option<&'static str> {
    IRREGULAR_OLD_NAMES
        .iter()
        .find(|(old, _)| *old == token)
        .map(|(_, command)| *command)
        .or_else(|| {
            registered_commands()
                .find(|command| command.contains(' ') && file_stem(command) == token)
        })
}
