//! What `xr schema` prints for each thing it is asked for.

use std::collections::BTreeMap;
use std::io::Write;

use serde_json::Value;
use xdk::error::{Error, Result};

use crate::cli::output::OutputConfig;

use super::words::{Refusal, resolve};
use super::{SCHEMA_ENTRIES, file_stem, registered_commands};

/// Returns the JSON Schema for a registry row.
fn schema_for_command(command: &str) -> Result<Value> {
    SCHEMA_ENTRIES
        .iter()
        .find(|entry| entry.commands.contains(&command))
        .map(|entry| (entry.schema)())
        .ok_or_else(|| Error::Internal(format!("no schema registered for '{command}'")))
}

/// Runs the schema subcommand.
///
/// JSON-emitting paths route through `OutputConfig::print_response` so a
/// schema body is not double-wrapped in `{"message": "..."}` under
/// `--output json`. The `envelope` flag dumps the canonical output envelope
/// schema instead of a response-type schema.
///
/// # Errors
///
/// Returns a [`Refusal`] when the words name no command, name one with no
/// typed response, use an old name or a scope word, or nothing was asked.
pub fn run_schema(
    command: &[String],
    list: bool,
    all: bool,
    envelope: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
) -> std::result::Result<(), Refusal> {
    if all {
        return print_all_schemas(out, stdout).map_err(Refusal::Invalid);
    }
    if envelope {
        let schema = crate::cli::envelope::envelope_schema();
        out.print_response(stdout, &schema);
        return Ok(());
    }
    if list {
        print_schema_list(out, stdout);
        return Ok(());
    }
    if command.is_empty() {
        return Err(Refusal::Invalid(Error::validation("missing-command")));
    }
    let row = resolve(command)?;
    let schema = schema_for_command(row).map_err(Refusal::Invalid)?;
    out.print_response(stdout, &schema);
    Ok(())
}

/// Writes every registered command with its response type and its file.
///
/// Text output aligns one row per command; the structured formats get one
/// record per command, which `scripts/generate-response-schemas.sh` reads.
fn print_schema_list(out: &OutputConfig, stdout: &mut dyn Write) {
    let mut entries: Vec<(&str, &str)> = SCHEMA_ENTRIES
        .iter()
        .flat_map(|entry| entry.commands.iter().map(|cmd| (*cmd, entry.type_name)))
        .collect();
    entries.sort_by_key(|(cmd, _)| *cmd);

    if out.format.is_structured() {
        let records: Vec<Value> = entries
            .iter()
            .map(|(cmd, type_name)| {
                serde_json::json!({
                    "command": cmd,
                    "type": type_name,
                    "file": format!("{}.schema.json", file_stem(cmd)),
                })
            })
            .collect();
        out.print_response(stdout, &Value::Array(records));
        return;
    }
    let max_cmd_len = entries.iter().map(|(cmd, _)| cmd.len()).max().unwrap_or(0);
    for (cmd, type_name) in &entries {
        out.print_message(stdout, &format!("{cmd:<max_cmd_len$}  {type_name}"));
    }
}

/// Writes all schemas as a single JSON object keyed by command name to `stdout`.
fn print_all_schemas(out: &OutputConfig, stdout: &mut dyn Write) -> Result<()> {
    let mut all: BTreeMap<String, Value> = BTreeMap::new();
    for command in registered_commands() {
        all.insert(command.to_string(), schema_for_command(command)?);
    }
    let value = serde_json::to_value(&all)?;
    out.print_response(stdout, &value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use xdk::error::Error;

    use super::schema_for_command;

    #[test]
    fn a_command_with_no_row_is_a_fault_in_the_registry() {
        assert!(matches!(
            schema_for_command("no such command"),
            Err(Error::Internal(_))
        ));
    }
}
