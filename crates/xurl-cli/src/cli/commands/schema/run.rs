//! What `xr schema` prints for each scope it is asked for.

use std::collections::BTreeMap;
use std::io::Write;

use schemars::generate::SchemaSettings;
use serde_json::Value;
use xdk::api::ApiResponse;
use xdk::error::{Error, Result};

use crate::cli::output::OutputConfig;

use super::resolve::{Labels, data_view, is_wrapper, node_at};
use super::words::{Refusal, resolve};
use super::{SCHEMA_ENTRIES, file_stem, generate, invocation, registered_commands};

/// The two renderings of a registry row: the default, and every subschema
/// inlined.
fn renderings(command: &str) -> Result<(Value, Value)> {
    let entry = SCHEMA_ENTRIES
        .iter()
        .find(|entry| entry.commands.contains(&command))
        .ok_or_else(|| Error::Internal(format!("no schema registered for '{command}'")))?;
    Ok((
        (entry.schema)(SchemaSettings::default()),
        (entry.schema)(inlined_settings()),
    ))
}

/// Generator settings that inline every subschema but a recursive one.
fn inlined_settings() -> SchemaSettings {
    SchemaSettings::default().with(|settings| settings.inline_subschemas = true)
}

/// The invocation that prints the shared wrapper.
const COMMON_INVOCATION: &str = "xr schema --common";

/// A registry row's default view, from its two renderings.
fn view(command: &str, full: &Value, inlined: &Value) -> Value {
    data_view(
        full,
        inlined,
        &Labels {
            command,
            common: COMMON_INVOCATION,
        },
    )
}

/// What `xr schema` was asked for.
pub struct Request<'a> {
    /// The command's words, empty when a scope flag stands alone.
    pub command: &'a [String],
    /// `--list`.
    pub list: bool,
    /// `--all`.
    pub all: bool,
    /// `--envelope`.
    pub envelope: bool,
    /// `--full`.
    pub full: bool,
    /// `--common`.
    pub common: bool,
    /// `--path`.
    pub path: Option<&'a str>,
}

/// Runs the schema subcommand.
///
/// JSON-emitting paths route through `OutputConfig::print_response` so a
/// schema body is not double-wrapped in `{"message": "..."}` under
/// `--output json`. clap holds the scopes to one at a time.
///
/// # Errors
///
/// Returns a [`Refusal`] when the words name no command, name one with no
/// typed response, use an old name or a scope word, a `--path` segment
/// names no property, or nothing was asked.
pub fn run_schema(
    request: &Request<'_>,
    out: &OutputConfig,
    stdout: &mut dyn Write,
) -> std::result::Result<(), Refusal> {
    if request.all {
        return print_all_schemas(out, stdout).map_err(Refusal::Invalid);
    }
    if request.envelope {
        out.print_response(stdout, &crate::cli::envelope::envelope_schema());
        return Ok(());
    }
    if request.list {
        print_schema_list(out, stdout);
        return Ok(());
    }
    if request.common {
        let common = match request.path {
            Some(path) => {
                let inlined = generate::<ApiResponse<Value>>(inlined_settings());
                path_node(&inlined, path, path, |prefix| {
                    format!("{COMMON_INVOCATION} --path {prefix}<property>")
                })?
            }
            None => generate::<ApiResponse<Value>>(SchemaSettings::default()),
        };
        out.print_response(stdout, &common);
        return Ok(());
    }
    if request.command.is_empty() {
        return Err(Refusal::Invalid(Error::validation("missing-command")));
    }
    let row = resolve(request.command)?;
    let (full, inlined) = renderings(row).map_err(Refusal::Invalid)?;
    let schema = if request.full {
        full
    } else if let Some(path) = request.path {
        let data = if is_wrapper(&full) {
            &inlined["properties"]["data"]
        } else {
            &inlined
        };
        let within = match path {
            "data" => "",
            _ => path.strip_prefix("data.").unwrap_or(path),
        };
        path_node(data, path, within, |prefix| {
            invocation_with(row, &format!("--path {prefix}<property>"))
        })?
    } else {
        view(row, &full, &inlined)
    };
    out.print_response(stdout, &schema);
    Ok(())
}

/// The node at `within`, a dot path under `root`; `given` is the path as the
/// caller typed it, and `template` builds the retry from the prefix that
/// resolved.
fn path_node(
    root: &Value,
    given: &str,
    within: &str,
    template: impl FnOnce(&str) -> String,
) -> std::result::Result<Value, Refusal> {
    let segments: Vec<&str> = if within.is_empty() {
        Vec::new()
    } else {
        within.split('.').collect()
    };
    node_at(root, &segments).map_err(|miss| {
        let prefix = if miss.resolved.is_empty() {
            String::new()
        } else {
            format!("{}.", miss.resolved)
        };
        Refusal::PathMiss {
            path: given.to_string(),
            segment: miss.segment,
            known: miss.known,
            template: template(&prefix),
        }
    })
}

/// [`invocation`] with `extra` added at the end.
fn invocation_with(command: &str, extra: &str) -> String {
    format!("{} {extra}", invocation(command))
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

/// Writes every registered command's default view as one JSON object keyed
/// by the command's words. The full schemas are the files under
/// `schema/responses/`.
fn print_all_schemas(out: &OutputConfig, stdout: &mut dyn Write) -> Result<()> {
    let mut all: BTreeMap<String, Value> = BTreeMap::new();
    for command in registered_commands() {
        let (full, inlined) = renderings(command)?;
        all.insert(command.to_string(), view(command, &full, &inlined));
    }
    let value = serde_json::to_value(&all)?;
    out.print_response(stdout, &value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use xdk::error::Error;

    use super::renderings;

    #[test]
    fn a_command_with_no_row_is_a_fault_in_the_registry() {
        assert!(matches!(
            renderings("no such command"),
            Err(Error::Internal(_))
        ));
    }
}
