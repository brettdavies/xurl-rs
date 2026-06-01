/// Schema subcommand — outputs JSON Schema for command response types.
use std::collections::BTreeMap;
use std::io::Write;

use schemars::schema_for;
use serde_json::Value;

use crate::api::{
    ApiResponse, BlockingResult, BookmarkedResult, DeletedResult, DmEvent, FollowingResult,
    LikedResult, MutingResult, RetweetedResult, Tweet, UsageData, User,
};
use crate::error::{Result, XurlError};
use crate::output::OutputConfig;

/// Command-to-response-type mapping entry.
struct SchemaEntry {
    /// Commands that share this response type.
    commands: &'static [&'static str],
    /// Human-readable type name.
    type_name: &'static str,
}

/// All command-to-type mappings, ordered by category.
const SCHEMA_ENTRIES: &[SchemaEntry] = &[
    SchemaEntry {
        commands: &["post", "reply", "quote", "read"],
        type_name: "ApiResponse<Tweet>",
    },
    SchemaEntry {
        commands: &["search", "timeline", "mentions", "bookmarks", "likes"],
        type_name: "ApiResponse<Vec<Tweet>>",
    },
    SchemaEntry {
        commands: &["whoami", "user"],
        type_name: "ApiResponse<User>",
    },
    SchemaEntry {
        commands: &["following", "followers"],
        type_name: "ApiResponse<Vec<User>>",
    },
    SchemaEntry {
        commands: &["like", "unlike"],
        type_name: "ApiResponse<LikedResult>",
    },
    SchemaEntry {
        commands: &["follow", "unfollow"],
        type_name: "ApiResponse<FollowingResult>",
    },
    SchemaEntry {
        commands: &["delete"],
        type_name: "ApiResponse<DeletedResult>",
    },
    SchemaEntry {
        commands: &["repost", "unrepost"],
        type_name: "ApiResponse<RetweetedResult>",
    },
    SchemaEntry {
        commands: &["bookmark", "unbookmark"],
        type_name: "ApiResponse<BookmarkedResult>",
    },
    SchemaEntry {
        commands: &["block", "unblock"],
        type_name: "ApiResponse<BlockingResult>",
    },
    SchemaEntry {
        commands: &["mute", "unmute"],
        type_name: "ApiResponse<MutingResult>",
    },
    SchemaEntry {
        commands: &["dm"],
        type_name: "ApiResponse<DmEvent>",
    },
    SchemaEntry {
        commands: &["dms"],
        type_name: "ApiResponse<Vec<DmEvent>>",
    },
    SchemaEntry {
        commands: &["usage"],
        type_name: "ApiResponse<UsageData>",
    },
];

/// Returns the JSON Schema for a given command name.
fn schema_for_command(command: &str) -> Result<Value> {
    // schema_for!() requires monomorphized types at compile time,
    // so this must be a match expression, not a data-driven lookup.
    let schema = match command {
        "post" | "reply" | "quote" | "read" => schema_for!(ApiResponse<Tweet>),
        "search" | "timeline" | "mentions" | "bookmarks" | "likes" => {
            schema_for!(ApiResponse<Vec<Tweet>>)
        }
        "whoami" | "user" => schema_for!(ApiResponse<User>),
        "following" | "followers" => schema_for!(ApiResponse<Vec<User>>),
        "like" | "unlike" => schema_for!(ApiResponse<LikedResult>),
        "follow" | "unfollow" => schema_for!(ApiResponse<FollowingResult>),
        "delete" => schema_for!(ApiResponse<DeletedResult>),
        "repost" | "unrepost" => schema_for!(ApiResponse<RetweetedResult>),
        "bookmark" | "unbookmark" => schema_for!(ApiResponse<BookmarkedResult>),
        "block" | "unblock" => schema_for!(ApiResponse<BlockingResult>),
        "mute" | "unmute" => schema_for!(ApiResponse<MutingResult>),
        "dm" => schema_for!(ApiResponse<DmEvent>),
        "dms" => schema_for!(ApiResponse<Vec<DmEvent>>),
        "usage" => schema_for!(ApiResponse<UsageData>),
        "auth" | "media" | "completions" | "version" | "schema" => {
            return Err(XurlError::validation(format!(
                "schema not available for '{command}' (no typed response)"
            )));
        }
        _ => {
            let valid: Vec<&str> = SCHEMA_ENTRIES
                .iter()
                .flat_map(|e| e.commands.iter())
                .copied()
                .collect();
            return Err(XurlError::validation(format!(
                "unknown command '{command}'. Valid commands: {}",
                valid.join(", ")
            )));
        }
    };
    Ok(serde_json::to_value(schema)?)
}

/// Runs the schema subcommand.
///
/// JSON-emitting paths route through `OutputConfig::print_response` so a
/// schema body is not double-wrapped in `{"message": "..."}` under
/// `--output json`. The human-readable `--list` path routes through
/// `OutputConfig::print_message`.
///
/// # Errors
///
/// Returns an error if the requested command is unknown, has no typed
/// response, or no argument/`--list`/`--all` is supplied.
pub fn run_schema(
    command: Option<&str>,
    list: bool,
    all: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
) -> Result<()> {
    if all {
        return print_all_schemas(out, stdout);
    }
    if list {
        return print_schema_list(out, stdout);
    }
    match command {
        Some(cmd) => {
            let schema = schema_for_command(cmd)?;
            out.print_response(stdout, &schema);
            Ok(())
        }
        None => {
            // No argument: show help text (same as `xr schema --help`)
            Err(XurlError::validation(
                "usage: xr schema <COMMAND> | xr schema --list | xr schema --all",
            ))
        }
    }
}

/// Writes all commands and their response type names to `stdout`.
fn print_schema_list(out: &OutputConfig, stdout: &mut dyn Write) -> Result<()> {
    let mut entries: Vec<(&str, &str)> = Vec::new();
    for entry in SCHEMA_ENTRIES {
        for &cmd in entry.commands {
            entries.push((cmd, entry.type_name));
        }
    }
    // Sort by command name for consistent output
    entries.sort_by_key(|(cmd, _)| *cmd);

    let max_cmd_len = entries.iter().map(|(cmd, _)| cmd.len()).max().unwrap_or(0);
    for (cmd, type_name) in &entries {
        out.print_message(stdout, &format!("{cmd:<max_cmd_len$}  {type_name}"));
    }
    Ok(())
}

/// Writes all schemas as a single JSON object keyed by command name to `stdout`.
fn print_all_schemas(out: &OutputConfig, stdout: &mut dyn Write) -> Result<()> {
    let mut all: BTreeMap<String, Value> = BTreeMap::new();
    for entry in SCHEMA_ENTRIES {
        for &cmd in entry.commands {
            let schema = schema_for_command(cmd)?;
            all.insert(cmd.to_string(), schema);
        }
    }
    let value = serde_json::to_value(&all)?;
    out.print_response(stdout, &value);
    Ok(())
}
