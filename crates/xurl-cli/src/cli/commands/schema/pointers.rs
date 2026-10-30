//! The pointers from a registered command's help and dry run to its schema.

use super::{invocation, registered_commands};

/// The invocation that prints the schema of the command at `path`, the
/// command's words, when it has a registry row.
#[must_use]
pub fn pointer_for(path: &str) -> Option<String> {
    registered_commands()
        .find(|command| *command == path)
        .map(invocation)
}

/// Adds a `Response schema:` block to the help of every command with a
/// registry row, naming the invocation that prints its schema.
pub trait SchemaPointers {
    /// The command tree with each registered command's help pointing at its
    /// schema.
    #[must_use]
    fn schema_pointers(self) -> Self;
}

impl SchemaPointers for clap::Command {
    fn schema_pointers(self) -> Self {
        self.mut_subcommands(|sub| with_footers(sub, ""))
    }
}

/// `cmd`, found under the words `parent`, with a `Response schema:` block
/// ending its help when it has a registry row, and the same for every
/// command beneath it. `mut_subcommands` keeps the declaration order, which
/// completions follow.
fn with_footers(cmd: clap::Command, parent: &str) -> clap::Command {
    let path = if parent.is_empty() {
        cmd.get_name().to_string()
    } else {
        format!("{parent} {}", cmd.get_name())
    };
    let cmd = match pointer_for(&path) {
        Some(pointer) => {
            let block = format!("Response schema:\n  {pointer}");
            let help = match cmd.get_after_help().map(ToString::to_string) {
                Some(existing) if !existing.trim().is_empty() => {
                    format!("{}\n\n{block}\n", existing.trim_end())
                }
                _ => format!("{block}\n"),
            };
            cmd.after_help(help)
        }
        None => cmd,
    };
    cmd.mut_subcommands(|sub| with_footers(sub, &path))
}
