//! `xr validate` — input-side schema validation for X API response shapes.
//!
//! Reads a JSON document from stdin or a file path, attempts to deserialize
//! it into the requested typed response struct, and emits an `ok` /
//! `validation-failed` envelope. The schema catalog mirrors the response
//! types exported from `src/api/response/types.rs` so the CLI's input-
//! validation surface is the same shape downstream agents see when they
//! deserialize an API response themselves.

use std::io::{Read, Write};
use std::path::Path;

use serde::de::DeserializeOwned;

use crate::cli::output::OutputConfig;
use xdk::api::{
    ApiResponse, BlockingResult, BookmarkedResult, ChatModeratorsResult, DeletedResult, DmEvent,
    FollowingResult, LikedResult, MutingResult, Post, RepostedResult, UsageCreditsData, UsageData,
    User,
};
use xdk::error::{EXIT_GENERAL_ERROR, EXIT_SUCCESS};

/// Canonical exit code for an input-validation failure, the same code the
/// other handlers hand back after writing their own envelope.
const EXIT_VALIDATION_FAILED: i32 = 1;

/// Auto-detection sentinel returned by [`detect_schema`] when the document
/// shape doesn't map cleanly to a known type.
const SCHEMA_UNKNOWN: &str = "unknown";

/// One name `--schema` accepts and the validator it dispatches to.
struct SchemaAlias {
    name: &'static str,
    validate: fn(&serde_json::Value) -> Result<(), String>,
    /// The response type the validator deserializes, by the compiler's
    /// name; `None` for a structural validator with no type.
    response_type: Option<fn() -> &'static str>,
}

/// An alias whose validator deserializes the API envelope around `T` or a
/// bare `T`.
const fn typed_alias<T: DeserializeOwned + Default>(name: &'static str) -> SchemaAlias {
    SchemaAlias {
        name,
        validate: typed::<T>,
        response_type: Some(std::any::type_name::<ApiResponse<T>>),
    }
}

/// Registry response types `xr validate` has no alias for, each with the
/// reason. Every one is a shape the CLI builds itself rather than an API
/// response, so nothing an agent pipes in would carry it.
pub const UNVALIDATED_TYPES: &[(&str, &str)] = &[
    (
        "Vec<AppStatusEntry>",
        "the auth status table the CLI renders from its own store",
    ),
    (
        "RedirectUriGetResponse",
        "the CLI's own redirect-uri lookup",
    ),
    (
        "RedirectUriSetResponse",
        "the CLI's own redirect-uri update",
    ),
    ("InstallEnvelope", "the skill installer's own report"),
    (
        "InstallMultiEnvelope",
        "the skill installer's own multi-host report",
    ),
];

/// Every schema `xr validate` accepts, in the order the `--schema` help and
/// the `unknown-schema` envelope list them. Singular and plural aliases
/// both resolve to the typed-response variant that round-trips a non-empty
/// value; auto-detection (`detect_schema`) follows the same precedence.
const SCHEMA_ALIASES: &[SchemaAlias] = &[
    typed_alias::<Post>("post"),
    typed_alias::<Vec<Post>>("posts"),
    typed_alias::<User>("user"),
    typed_alias::<Vec<User>>("users"),
    typed_alias::<DmEvent>("dm"),
    typed_alias::<Vec<DmEvent>>("dms"),
    typed_alias::<UsageData>("usage"),
    typed_alias::<UsageCreditsData>("credits"),
    SchemaAlias {
        name: "envelope",
        validate: validate_envelope,
        response_type: None,
    },
    typed_alias::<LikedResult>("like"),
    typed_alias::<FollowingResult>("follow"),
    typed_alias::<DeletedResult>("delete"),
    typed_alias::<RepostedResult>("repost"),
    typed_alias::<BookmarkedResult>("bookmark"),
    typed_alias::<MutingResult>("mute"),
    typed_alias::<BlockingResult>("block"),
    typed_alias::<ChatModeratorsResult>("moderators"),
];

/// Every name `--schema` accepts, in declaration order.
pub fn schema_names() -> impl Iterator<Item = &'static str> {
    SCHEMA_ALIASES.iter().map(|alias| alias.name)
}

/// Every response type an alias validates, written the way the schema
/// registry writes it (`ApiResponse<Vec<Post>>`, no module paths).
pub fn validated_types() -> impl Iterator<Item = String> {
    SCHEMA_ALIASES
        .iter()
        .filter_map(|alias| alias.response_type)
        .map(|response_type| short_type_name(response_type()))
}

/// Strips every module path from a `std::any::type_name` rendering, so
/// `xdk::api::response::types::ApiResponse<alloc::vec::Vec<...::Post>>`
/// reads `ApiResponse<Vec<Post>>`.
fn short_type_name(full: &str) -> String {
    let mut out = String::new();
    let mut rest = full;
    while let Some(idx) = rest.find("::") {
        let head = &rest[..idx];
        let keep = head
            .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
            .map_or(0, |i| i + 1);
        out.push_str(&head[..keep]);
        rest = &rest[idx + 2..];
    }
    out.push_str(rest);
    out
}

/// The `--schema` argument's help text, listing every accepted name.
pub fn schema_arg_help() -> String {
    let names: Vec<String> = schema_names().map(|name| format!("`{name}`")).collect();
    format!(
        "Schema name to validate against ({}). Omit for auto-detection",
        names.join(", ")
    )
}

/// Detects the most likely schema from the document's top-level shape.
///
/// The X API envelope is `{"data": ..., "meta": ..., "includes": ...}`. When
/// that shape is present we dispatch on `data`'s type. For bare payloads
/// (an agent calling `validate` on the `data` value directly) we fall back
/// to the same key/value heuristics.
fn detect_schema(value: &serde_json::Value) -> &'static str {
    let data = value.get("data").unwrap_or(value);
    match data {
        serde_json::Value::Array(arr) => {
            if let Some(first) = arr.first() {
                if first.get("text").is_some() {
                    "posts"
                } else if first.get("username").is_some() || first.get("name").is_some() {
                    "users"
                } else if first.get("event_type").is_some() {
                    "dms"
                } else {
                    SCHEMA_UNKNOWN
                }
            } else {
                "posts"
            }
        }
        serde_json::Value::Object(map) => {
            if map.contains_key("status") && map.contains_key("reason") {
                "envelope"
            } else if map.contains_key("text") {
                "post"
            } else if map.contains_key("username") || map.contains_key("name") {
                "user"
            } else if map.contains_key("event_type") {
                "dm"
            } else if map.contains_key("project_cap") || map.contains_key("cap_reset_day") {
                "usage"
            } else if map.contains_key("total_balance") || map.contains_key("free_balance") {
                "credits"
            } else {
                SCHEMA_UNKNOWN
            }
        }
        _ => SCHEMA_UNKNOWN,
    }
}

/// Entrypoint for the `xr validate` subcommand.
///
/// Returns the process exit code. All output goes through `out` so library
/// callers passing `Vec<u8>` writers can capture the envelope.
pub fn run_validate(
    file: Option<&str>,
    schema_arg: Option<&str>,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let raw = match read_input(file) {
        Ok(s) => s,
        Err(reason) => {
            out.print_error_envelope(stderr, "io", EXIT_GENERAL_ERROR, &reason);
            return EXIT_GENERAL_ERROR;
        }
    };

    let value: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            let payload = serde_json::json!({
                "status": "error",
                "reason": "invalid-json",
                "exit_code": EXIT_VALIDATION_FAILED,
                "message": format!("input is not valid JSON: {e}"),
            });
            out.print_response(stderr, &payload);
            return EXIT_VALIDATION_FAILED;
        }
    };

    let schema = match schema_arg {
        Some(s) => s.to_string(),
        None => detect_schema(&value).to_string(),
    };

    let Some(alias) = SCHEMA_ALIASES.iter().find(|alias| alias.name == schema) else {
        let known: Vec<&str> = schema_names().collect();
        let payload = serde_json::json!({
            "status": "error",
            "reason": "unknown-schema",
            "exit_code": EXIT_VALIDATION_FAILED,
            "schema": schema,
            "known_schemas": known,
            "message": format!(
                "unknown schema {schema:?}; pass one of {} or omit --schema for auto-detection",
                known.join(", ")
            ),
        });
        out.print_response(stderr, &payload);
        return EXIT_VALIDATION_FAILED;
    };

    match (alias.validate)(&value) {
        Ok(()) => {
            let payload = serde_json::json!({
                "status": "ok",
                "schema": schema,
                "valid": true,
            });
            out.print_response(stdout, &payload);
            EXIT_SUCCESS
        }
        Err(msg) => {
            let payload = serde_json::json!({
                "status": "error",
                "reason": "validation-failed",
                "exit_code": EXIT_VALIDATION_FAILED,
                "schema": schema,
                "valid": false,
                "message": msg,
            });
            out.print_response(stderr, &payload);
            EXIT_VALIDATION_FAILED
        }
    }
}

/// Reads the input document from `path` (or stdin if `None` or `"-"`).
fn read_input(path: Option<&str>) -> Result<String, String> {
    match path {
        None | Some("-") => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("reading stdin: {e}"))?;
            Ok(buf)
        }
        Some(p) => std::fs::read_to_string(Path::new(p)).map_err(|e| format!("reading {p}: {e}")),
    }
}

/// Accepts `value` as either the API envelope around `T` or a bare `T`.
fn typed<T: DeserializeOwned + Default>(value: &serde_json::Value) -> Result<(), String> {
    try_into::<ApiResponse<T>>(value).or_else(|_| try_into::<T>(value))
}

/// Validates that `value` matches the canonical xurl error / success envelope.
///
/// The envelope shape is intentionally untyped at the API boundary (errors
/// from every layer flatten in via `print_error`), so we validate structurally
/// rather than by `serde_json::from_value::<T>` into a single struct.
fn validate_envelope(value: &serde_json::Value) -> Result<(), String> {
    let map = value
        .as_object()
        .ok_or_else(|| "envelope must be a JSON object".to_string())?;
    let status = map
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "envelope missing required string field `status`".to_string())?;
    match status {
        "ok" | "dry_run" => Ok(()),
        "error" => {
            for field in ["reason", "exit_code", "message"] {
                if !map.contains_key(field) {
                    return Err(format!("error envelope missing required field {field:?}"));
                }
            }
            if !map["exit_code"].is_i64() {
                return Err("error envelope `exit_code` must be an integer".into());
            }
            Ok(())
        }
        "awaiting_callback" | "cancelled" => Ok(()),
        other => Err(format!(
            "envelope `status` must be one of ok / dry_run / error / awaiting_callback / cancelled; got {other:?}"
        )),
    }
}

/// Wrapper around `serde_json::from_value` returning the failure message.
fn try_into<T: DeserializeOwned>(value: &serde_json::Value) -> Result<(), String> {
    serde_json::from_value::<T>(value.clone())
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ColorChoice;
    use crate::cli::output::OutputFormat;

    fn cfg(format: OutputFormat) -> OutputConfig {
        OutputConfig::new(format, false, false, ColorChoice::Never)
    }

    fn run(
        value: &serde_json::Value,
        schema: Option<&str>,
        format: OutputFormat,
    ) -> (i32, String, String) {
        let out = cfg(format);
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        // Drive the validate path via a temp file so we don't poll stdin.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("in.json");
        std::fs::write(&path, serde_json::to_string(value).unwrap()).unwrap();
        let code = run_validate(
            Some(path.to_str().unwrap()),
            schema,
            &out,
            &mut stdout,
            &mut stderr,
        );
        (
            code,
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
        )
    }

    #[test]
    fn short_type_name_drops_every_module_path() {
        assert_eq!(
            short_type_name(
                "xdk::api::response::types::ApiResponse<alloc::vec::Vec<xdk::api::response::types::Post>>"
            ),
            "ApiResponse<Vec<Post>>"
        );
        assert_eq!(short_type_name("Plain"), "Plain");
    }

    #[test]
    fn validates_a_post_envelope() {
        let v = serde_json::json!({
            "data": {"id": "1", "text": "hi"},
        });
        let (code, stdout, _) = run(&v, Some("post"), OutputFormat::Json);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"valid\": true"), "got: {stdout}");
        assert!(stdout.contains("\"schema\": \"post\""), "got: {stdout}");
    }

    #[test]
    fn auto_detects_user_shape() {
        let v = serde_json::json!({
            "data": {"id": "1", "username": "elonmusk", "name": "Elon Musk"},
        });
        let (code, stdout, _) = run(&v, None, OutputFormat::Json);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"schema\": \"user\""), "got: {stdout}");
    }

    #[test]
    fn rejects_malformed_envelope() {
        // `text` field missing → ApiResponse<Post> fails to deserialize.
        let v = serde_json::json!({
            "data": {"id": "1"},
        });
        let (code, _stdout, stderr) = run(&v, Some("post"), OutputFormat::Json);
        assert_eq!(code, 1);
        assert!(
            stderr.contains("validation-failed"),
            "expected validation-failed reason in stderr: {stderr}"
        );
    }

    #[test]
    fn unknown_schema_returns_envelope_with_catalog() {
        let v = serde_json::json!({"data": {"id": "1"}});
        let (code, _stdout, stderr) = run(&v, Some("not-a-thing"), OutputFormat::Json);
        assert_eq!(code, 1);
        assert!(stderr.contains("unknown-schema"), "got: {stderr}");
        assert!(stderr.contains("known_schemas"), "got: {stderr}");
    }

    #[test]
    fn validates_canonical_error_envelope() {
        let v = serde_json::json!({
            "status": "error",
            "reason": "auth-required",
            "exit_code": 2,
            "message": "no token",
        });
        let (code, stdout, _stderr) = run(&v, Some("envelope"), OutputFormat::Json);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"valid\": true"), "got: {stdout}");
    }

    #[test]
    fn envelope_rejects_missing_required_field() {
        let v = serde_json::json!({
            "status": "error",
            "reason": "auth-required",
            "exit_code": 2,
            // missing `message`
        });
        let (code, _stdout, stderr) = run(&v, Some("envelope"), OutputFormat::Json);
        assert_eq!(code, 1);
        assert!(stderr.contains("missing required field"), "got: {stderr}");
    }
}
