//! Schema subcommand — outputs JSON Schema for command response types.

use std::collections::BTreeMap;
use std::io::Write;

use schemars::schema_for;
use serde_json::Value;

use crate::cli::commands::auth::{AppStatusEntry, RedirectUriGetResponse, RedirectUriSetResponse};
use crate::cli::output::OutputConfig;
use crate::cli::skill_install::{InstallEnvelope, InstallMultiEnvelope};
use xdk::api::{
    ApiResponse, BlockingResult, BookmarkedResult, ChatModeratorsResult, DeletedResult, DmEvent,
    DmSentResult, FollowingResult, LikedResult, MutingResult, Post, RepostedResult,
    UsageCreditsData, UsageData, User,
};
use xdk::error::{Error, Result};

/// Command-to-response-type mapping entry.
struct SchemaEntry {
    /// Commands that share this response type.
    commands: &'static [&'static str],
    /// Human-readable type name.
    type_name: &'static str,
    /// Builds the response type's JSON Schema. `schema_for!` needs the type
    /// at its expansion site, so each row carries its own.
    schema: fn() -> Value,
}

/// All command-to-type mappings, ordered by category.
const SCHEMA_ENTRIES: &[SchemaEntry] = &[
    SchemaEntry {
        commands: &["post", "reply", "quote", "read"],
        type_name: "ApiResponse<Post>",
        schema: || schema_for!(ApiResponse<Post>).into(),
    },
    SchemaEntry {
        commands: &["search", "timeline", "mentions", "bookmarks", "likes"],
        type_name: "ApiResponse<Vec<Post>>",
        schema: || schema_for!(ApiResponse<Vec<Post>>).into(),
    },
    SchemaEntry {
        commands: &["whoami", "user"],
        type_name: "ApiResponse<User>",
        schema: || schema_for!(ApiResponse<User>).into(),
    },
    SchemaEntry {
        commands: &["following", "followers", "muted", "blocked"],
        type_name: "ApiResponse<Vec<User>>",
        schema: || schema_for!(ApiResponse<Vec<User>>).into(),
    },
    SchemaEntry {
        commands: &["like", "unlike"],
        type_name: "ApiResponse<LikedResult>",
        schema: || schema_for!(ApiResponse<LikedResult>).into(),
    },
    SchemaEntry {
        commands: &["follow", "unfollow"],
        type_name: "ApiResponse<FollowingResult>",
        schema: || schema_for!(ApiResponse<FollowingResult>).into(),
    },
    SchemaEntry {
        commands: &["delete"],
        type_name: "ApiResponse<DeletedResult>",
        schema: || schema_for!(ApiResponse<DeletedResult>).into(),
    },
    SchemaEntry {
        commands: &["repost", "unrepost"],
        type_name: "ApiResponse<RepostedResult>",
        schema: || schema_for!(ApiResponse<RepostedResult>).into(),
    },
    SchemaEntry {
        commands: &["bookmark", "unbookmark"],
        type_name: "ApiResponse<BookmarkedResult>",
        schema: || schema_for!(ApiResponse<BookmarkedResult>).into(),
    },
    SchemaEntry {
        commands: &["mute", "unmute"],
        type_name: "ApiResponse<MutingResult>",
        schema: || schema_for!(ApiResponse<MutingResult>).into(),
    },
    SchemaEntry {
        commands: &["block", "unblock"],
        type_name: "ApiResponse<BlockingResult>",
        schema: || schema_for!(ApiResponse<BlockingResult>).into(),
    },
    SchemaEntry {
        commands: &["dm"],
        type_name: "ApiResponse<DmSentResult>",
        schema: || schema_for!(ApiResponse<DmSentResult>).into(),
    },
    SchemaEntry {
        commands: &["dms"],
        type_name: "ApiResponse<Vec<DmEvent>>",
        schema: || schema_for!(ApiResponse<Vec<DmEvent>>).into(),
    },
    SchemaEntry {
        commands: &["usage"],
        type_name: "ApiResponse<UsageData>",
        schema: || schema_for!(ApiResponse<UsageData>).into(),
    },
    SchemaEntry {
        commands: &["usage-credits"],
        type_name: "ApiResponse<UsageCreditsData>",
        schema: || schema_for!(ApiResponse<UsageCreditsData>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts-moderators-list"],
        type_name: "ApiResponse<Vec<User>>",
        schema: || schema_for!(ApiResponse<Vec<User>>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts-moderators-add", "broadcasts-moderators-remove"],
        type_name: "ApiResponse<ChatModeratorsResult>",
        schema: || schema_for!(ApiResponse<ChatModeratorsResult>).into(),
    },
    SchemaEntry {
        commands: &["auth-status", "auth-apps-list"],
        type_name: "Vec<AppStatusEntry>",
        schema: || schema_for!(Vec<AppStatusEntry>).into(),
    },
    SchemaEntry {
        commands: &["redirect-uri-get"],
        type_name: "RedirectUriGetResponse",
        schema: || schema_for!(RedirectUriGetResponse).into(),
    },
    SchemaEntry {
        commands: &["redirect-uri-set"],
        type_name: "RedirectUriSetResponse",
        schema: || schema_for!(RedirectUriSetResponse).into(),
    },
    SchemaEntry {
        commands: &["skill-install"],
        type_name: "InstallEnvelope",
        schema: || schema_for!(InstallEnvelope).into(),
    },
    SchemaEntry {
        commands: &["skill-install-all"],
        type_name: "InstallMultiEnvelope",
        schema: || schema_for!(InstallMultiEnvelope).into(),
    },
];

/// Commands clap parses that emit no typed response, so `xr schema` has no
/// document for them. Every command is in exactly one of this set and
/// `SCHEMA_ENTRIES`.
pub const SCHEMA_LESS_COMMANDS: &[&str] = &[
    "auth",
    "auth-app",
    "auth-apps",
    "auth-apps-add",
    "auth-apps-redirect-uri",
    "auth-apps-remove",
    "auth-apps-update",
    "auth-clear",
    "auth-default",
    "auth-oauth1",
    "auth-oauth2",
    "broadcasts",
    "broadcasts-moderators",
    "completions",
    "examples",
    "media",
    "media-status",
    "media-upload",
    "schema",
    "skill",
    "skill-update",
    "validate",
    "version",
];

/// Commands whose `xr schema` name is not their clap path joined with `-`:
/// the joined path, then the name the registry uses.
pub const SCHEMA_NAME_OVERRIDES: &[(&str, &str)] = &[
    ("auth-apps-redirect-uri-get", "redirect-uri-get"),
    ("auth-apps-redirect-uri-set", "redirect-uri-set"),
];

/// Registry names that describe a flag form of a command rather than a
/// command of their own, each with the command it belongs to.
pub const FLAG_FORMS: &[(&str, &str)] = &[("skill-install-all", "skill-install")];

/// The name `xr schema` accepts for the clap command at `path`.
pub fn schema_name_for_path(path: &[&str]) -> String {
    let joined = path.join("-");
    SCHEMA_NAME_OVERRIDES
        .iter()
        .find(|(clap_path, _)| *clap_path == joined)
        .map_or(joined, |(_, name)| (*name).to_string())
}

/// Every response type the registry names, one per row, in table order.
pub fn registered_types() -> impl Iterator<Item = &'static str> {
    SCHEMA_ENTRIES.iter().map(|entry| entry.type_name)
}

/// Every command name with a registry row, in table order.
pub fn registered_commands() -> impl Iterator<Item = &'static str> {
    SCHEMA_ENTRIES
        .iter()
        .flat_map(|entry| entry.commands.iter().copied())
}

/// Returns the JSON Schema for a given command name.
fn schema_for_command(command: &str) -> Result<Value> {
    if let Some(entry) = SCHEMA_ENTRIES
        .iter()
        .find(|entry| entry.commands.contains(&command))
    {
        return Ok((entry.schema)());
    }
    if SCHEMA_LESS_COMMANDS.contains(&command) {
        return Err(Error::validation(format!(
            "schema not available for '{command}' (no typed response)"
        )));
    }
    let valid: Vec<&str> = registered_commands().collect();
    Err(Error::validation(format!(
        "unknown command '{command}'. Valid commands: {}",
        valid.join(", ")
    )))
}

/// Runs the schema subcommand.
///
/// JSON-emitting paths route through `OutputConfig::print_response` so a
/// schema body is not double-wrapped in `{"message": "..."}` under
/// `--output json`. The human-readable `--list` path routes through
/// `OutputConfig::print_message`. The `envelope` flag (or `command =
/// "envelope"`) dumps the canonical output envelope schema instead of a
/// response-type schema.
///
/// # Errors
///
/// Returns an error if the requested command is unknown, has no typed
/// response, or no argument/`--list`/`--all`/`--envelope` is supplied.
pub fn run_schema(
    command: Option<&str>,
    list: bool,
    all: bool,
    envelope: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
) -> Result<()> {
    if all {
        return print_all_schemas(out, stdout);
    }
    if envelope || command == Some("envelope") {
        let schema = crate::cli::envelope::envelope_schema();
        out.print_response(stdout, &schema);
        return Ok(());
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
            Err(Error::validation(
                "usage: xr schema <COMMAND> | xr schema envelope | xr schema --list | xr schema --all",
            ))
        }
    }
}

/// Writes all commands and their response type names to `stdout`.
///
/// The trailing `envelope` row advertises the canonical agent-native
/// output envelope schema; consumers query it via `xr schema envelope`.
fn print_schema_list(out: &OutputConfig, stdout: &mut dyn Write) -> Result<()> {
    let mut entries: Vec<(&str, &str)> = Vec::new();
    for entry in SCHEMA_ENTRIES {
        for &cmd in entry.commands {
            entries.push((cmd, entry.type_name));
        }
    }
    // Sort by command name for consistent output
    entries.sort_by_key(|(cmd, _)| *cmd);
    entries.push(("envelope", "Envelope"));

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
