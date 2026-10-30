//! Command words a family renamed, and the words that replaced them.
//!
//! A retired word refuses as any unknown command does, but its suggestion is
//! the replacement rather than clap's guess at a typo: the sibling nearest by
//! spelling can be an unrelated verb, and `rename` scores closest to `remove`.

use crate::cli::classify::ROOT_COMMAND;

/// A word a family no longer takes, and the word that took its place.
struct Retired {
    /// The family as `failing_command` names it, such as
    /// `xr broadcasts scheduled`.
    family: &'static str,
    retired: &'static str,
    replacement: &'static str,
}

const RETIRED: &[Retired] = &[Retired {
    family: "xr broadcasts scheduled",
    retired: "remove",
    replacement: "cancel",
}];

/// The word that replaced `word` under `family`, when the family retired it.
pub(crate) fn replacement(family: &str, word: &str) -> Option<&'static str> {
    RETIRED
        .iter()
        .find(|row| row.family == family && row.retired == word)
        .map(|row| row.replacement)
}

/// Each retired command by its words beneath `xr`, beside the words of the
/// command that replaced it.
pub(crate) fn retired_commands() -> impl Iterator<Item = (String, String)> {
    RETIRED.iter().map(|row| {
        let family = row.family.strip_prefix(ROOT_COMMAND).unwrap_or(row.family);
        let words = |word: &str| format!("{family} {word}").trim_start().to_string();
        (words(row.retired), words(row.replacement))
    })
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::RETIRED;
    use crate::cli::Cli;
    use crate::cli::classify::ROOT_COMMAND;

    /// Every row names a family clap parses, a replacement that family
    /// takes, and a retired word it no longer takes, so a later rename that
    /// moves either word fails here instead of pointing a caller at nothing.
    #[test]
    fn every_row_names_a_live_replacement_and_a_retired_word() {
        for row in RETIRED {
            let mut words = row.family.split(' ');
            assert_eq!(words.next(), Some(ROOT_COMMAND), "{}", row.family);
            let mut command = Cli::command();
            for word in words {
                command = command
                    .find_subcommand(word)
                    .unwrap_or_else(|| panic!("no family `{}`", row.family))
                    .clone();
            }
            assert!(
                command.find_subcommand(row.replacement).is_some(),
                "`{} {}` is not a command, so the row for `{}` points at nothing",
                row.family,
                row.replacement,
                row.retired
            );
            assert!(
                command.find_subcommand(row.retired).is_none(),
                "`{} {}` is a command again; drop its retired row",
                row.family,
                row.retired
            );
        }
    }
}
