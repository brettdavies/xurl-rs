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

use crate::api::{
    ApiResponse, BookmarkedResult, DeletedResult, DmEvent, FollowingResult, LikedResult,
    MutingResult, Post, RepostedResult, UsageCreditsData, UsageData, User,
};
use crate::error::{EXIT_GENERAL_ERROR, EXIT_SUCCESS};
use crate::output::OutputConfig;

/// Canonical exit code for an input-validation failure (matches the
/// envelope-already-emitted dispatch path used elsewhere in the CLI).
const EXIT_VALIDATION_FAILED: i32 = 1;

/// Auto-detection sentinel returned by [`detect_schema`] when the document
/// shape doesn't map cleanly to a known type.
const SCHEMA_UNKNOWN: &str = "unknown";

/// Schema-name aliases.
///
/// Singular and plural aliases both resolve to the typed-response variant
/// that round-trips a non-empty value. The order in the auto-detection
/// fallback (`detect_schema`) follows the same precedence: object-with-id
/// looks like a `post`, an array → `posts`, etc.
fn known_schemas() -> &'static [&'static str] {
    &[
        "post", "posts", "user", "users", "dm", "dms", "usage", "credits", "envelope", "like",
        "follow", "delete", "repost", "bookmark", "mute",
    ]
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

    if !known_schemas().iter().any(|k| *k == schema) {
        let payload = serde_json::json!({
            "status": "error",
            "reason": "unknown-schema",
            "exit_code": EXIT_VALIDATION_FAILED,
            "schema": schema,
            "known_schemas": known_schemas(),
            "message": format!(
                "unknown schema {schema:?}; pass one of {} or omit --schema for auto-detection",
                known_schemas().join(", ")
            ),
        });
        out.print_response(stderr, &payload);
        return EXIT_VALIDATION_FAILED;
    }

    match validate_against(&schema, &value) {
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

/// Dispatches `value` against the requested schema name.
fn validate_against(schema: &str, value: &serde_json::Value) -> Result<(), String> {
    match schema {
        "post" => try_into::<ApiResponse<Post>>(value).or_else(|_| try_into::<Post>(value)),
        "posts" => {
            try_into::<ApiResponse<Vec<Post>>>(value).or_else(|_| try_into::<Vec<Post>>(value))
        }
        "user" => try_into::<ApiResponse<User>>(value).or_else(|_| try_into::<User>(value)),
        "users" => {
            try_into::<ApiResponse<Vec<User>>>(value).or_else(|_| try_into::<Vec<User>>(value))
        }
        "dm" => try_into::<ApiResponse<DmEvent>>(value).or_else(|_| try_into::<DmEvent>(value)),
        "dms" => try_into::<ApiResponse<Vec<DmEvent>>>(value)
            .or_else(|_| try_into::<Vec<DmEvent>>(value)),
        "usage" => {
            try_into::<ApiResponse<UsageData>>(value).or_else(|_| try_into::<UsageData>(value))
        }
        "credits" => try_into::<ApiResponse<UsageCreditsData>>(value)
            .or_else(|_| try_into::<UsageCreditsData>(value)),
        "envelope" => validate_envelope(value),
        "like" => {
            try_into::<ApiResponse<LikedResult>>(value).or_else(|_| try_into::<LikedResult>(value))
        }
        "follow" => try_into::<ApiResponse<FollowingResult>>(value)
            .or_else(|_| try_into::<FollowingResult>(value)),
        "delete" => try_into::<ApiResponse<DeletedResult>>(value)
            .or_else(|_| try_into::<DeletedResult>(value)),
        "repost" => try_into::<ApiResponse<RepostedResult>>(value)
            .or_else(|_| try_into::<RepostedResult>(value)),
        "bookmark" => try_into::<ApiResponse<BookmarkedResult>>(value)
            .or_else(|_| try_into::<BookmarkedResult>(value)),
        "mute" => try_into::<ApiResponse<MutingResult>>(value)
            .or_else(|_| try_into::<MutingResult>(value)),
        SCHEMA_UNKNOWN => Err(format!(
            "could not auto-detect schema from document shape; pass --schema explicitly (known: {})",
            known_schemas().join(", ")
        )),
        other => Err(format!("schema {other:?} has no validator wired up")),
    }
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
    use crate::output::OutputFormat;

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
            "data": {"id": "1", "username": "jack", "name": "jack"},
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
