/// Output formatting helpers for `--output`, `--quiet`, `--color`, and
/// `NO_COLOR` support.
///
/// `OutputConfig` is a pure `Send + Sync + Clone` configuration object — it
/// owns no I/O handles. Print methods accept `&mut dyn Write` at the call site
/// so the same config can drive real stdout, real stderr, or a captured
/// `Vec<u8>` in library tests.
use std::io::{IsTerminal, Write};

use clap::ValueEnum;
use serde_json::Value;

use crate::cli::ColorChoice;
use crate::error::XurlError;

/// Output format for machine/human consumption.
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    /// Default: colored, human-readable
    Text,
    /// Machine-readable JSON, no color
    Json,
    /// JSON Lines (useful for streaming)
    Jsonl,
}

/// Output configuration threaded through command handlers.
///
/// `OutputConfig` is intentionally a pure data carrier — no I/O handles, no
/// interior mutability. This keeps it `Send + Sync + Clone`, which is required
/// for the planned async/concurrent `ApiClient` (see `project_async_requirement`).
///
/// `use_color` is the resolved color decision after combining `--color`, the
/// `NO_COLOR` env var, and stderr's TTY-ness. `no_color` is preserved as the
/// negation (`!use_color`) for source-compatibility with existing call sites
/// and tests that constructed `OutputConfig { … }` directly.
///
/// `raw` (from `--raw`) forces compact JSON (no pretty-printing) and strips
/// ANSI styling from text output. Useful for pipelines that line-buffer.
#[derive(Clone, Debug)]
pub struct OutputConfig {
    pub format: OutputFormat,
    pub quiet: bool,
    pub no_color: bool,
    pub use_color: bool,
    pub verbose: bool,
    pub raw: bool,
}

impl OutputConfig {
    /// Creates an `OutputConfig` from resolved CLI flags and environment.
    ///
    /// `use_color` is computed from `color` together with `NO_COLOR` and
    /// `std::io::stderr().is_terminal()`:
    /// - `NO_COLOR` is absolute (per <https://no-color.org/>): when set,
    ///   color is disabled regardless of `--color`.
    /// - `--color always` overrides the TTY check (still loses to `NO_COLOR`).
    /// - `--color never` disables color unconditionally.
    /// - `--color auto` enables color only when stderr is a TTY.
    ///
    /// `raw` forces `use_color = false` and switches JSON output to compact
    /// form (no pretty-printing).
    #[must_use]
    pub fn new(format: OutputFormat, quiet: bool, verbose: bool, color: ColorChoice) -> Self {
        Self::new_with_raw(format, quiet, verbose, color, false)
    }

    /// Like [`new`], with an explicit `raw` flag.
    ///
    /// [`new`]: Self::new
    #[must_use]
    pub fn new_with_raw(
        format: OutputFormat,
        quiet: bool,
        verbose: bool,
        color: ColorChoice,
        raw: bool,
    ) -> Self {
        let no_color_env = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        let use_color = if raw || no_color_env {
            false
        } else {
            match color {
                ColorChoice::Always => true,
                ColorChoice::Never => false,
                ColorChoice::Auto => std::io::stderr().is_terminal(),
            }
        };
        Self {
            format,
            quiet,
            no_color: !use_color,
            use_color,
            verbose,
            raw,
        }
    }

    /// Prints an informational message (suppressed by --quiet or --output json/jsonl).
    ///
    /// The runner passes a stderr writer here in the binary path; tests pass a `Vec<u8>`.
    pub fn info(&self, err: &mut dyn Write, msg: &str) {
        if self.quiet || self.format != OutputFormat::Text {
            return;
        }
        let _ = writeln!(err, "{msg}");
    }

    /// Prints a success/status message with optional color.
    ///
    /// The runner passes a stderr writer here in the binary path; tests pass a `Vec<u8>`.
    pub fn status(&self, err: &mut dyn Write, msg: &str) {
        if self.quiet || self.format != OutputFormat::Text {
            return;
        }
        if self.no_color {
            let _ = writeln!(err, "{msg}");
        } else {
            let _ = writeln!(err, "\x1b[32m{msg}\x1b[0m");
        }
    }

    /// Prints an API response according to the configured output format.
    ///
    /// I/O errors are intentionally swallowed (best-effort posture) so a
    /// closed downstream pipe doesn't abort the program — the SIGPIPE
    /// restoration in `main` handles the more general case.
    ///
    /// Under `--raw`, JSON output is emitted compactly (one line, no
    /// whitespace) rather than pretty-printed.
    pub fn print_response(&self, out: &mut dyn Write, value: &serde_json::Value) {
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let body = if self.raw {
                    serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
                } else {
                    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
                };
                let _ = writeln!(out, "{body}");
            }
            OutputFormat::Text => {
                if self.no_color {
                    let pretty =
                        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
                    let _ = writeln!(out, "{pretty}");
                } else {
                    let _ = crate::api::response::format_response(out, value);
                }
            }
        }
    }

    /// Prints a streaming line according to the configured output format.
    pub fn print_stream_line(&self, out: &mut dyn Write, line: &str) {
        let _ = writeln!(out, "{line}");
    }

    /// Formats and prints an error to the supplied stderr writer.
    /// Under `--output json|jsonl`, emits the canonical envelope shape:
    /// `{"status":"error","reason":<kind>,"exit_code":<code>,"message":<display>}`.
    pub fn print_error(&self, err: &mut dyn Write, error: &XurlError, exit_code: i32) {
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let reason = error.kind();
                let msg = error.to_string();
                let envelope = serde_json::json!({
                    "status": "error",
                    "reason": reason,
                    "exit_code": exit_code,
                    "message": msg,
                });
                let rendered = if self.raw {
                    serde_json::to_string(&envelope).unwrap_or_else(|_| envelope.to_string())
                } else {
                    envelope.to_string()
                };
                let _ = writeln!(err, "{rendered}");
            }
            OutputFormat::Text => {
                if self.no_color {
                    let _ = writeln!(err, "Error: {error}");
                } else {
                    let _ = writeln!(err, "\x1b[31mError: {error}\x1b[0m");
                }
            }
        }
    }

    /// Emits a canonical success envelope under JSON modes.
    ///
    /// Wraps `payload` (treated as a JSON object whose keys flatten in at
    /// the top level) with `{"status":"ok", ...payload}`. Under text mode
    /// the payload is passed through to [`Self::print_response`] so existing
    /// formatters keep their shape.
    pub fn print_success(&self, out: &mut dyn Write, payload: &Value) {
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let mut obj = serde_json::Map::new();
                obj.insert("status".into(), Value::String("ok".into()));
                if let Some(map) = payload.as_object() {
                    for (k, v) in map {
                        obj.insert(k.clone(), v.clone());
                    }
                } else {
                    obj.insert("payload".into(), payload.clone());
                }
                let envelope = Value::Object(obj);
                let body = if self.raw {
                    serde_json::to_string(&envelope).unwrap_or_else(|_| envelope.to_string())
                } else {
                    serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| envelope.to_string())
                };
                let _ = writeln!(out, "{body}");
            }
            OutputFormat::Text => self.print_response(out, payload),
        }
    }

    /// Emits a canonical dry-run envelope under JSON modes.
    ///
    /// Shape: `{"status":"dry_run","would_succeed":<bool>,"exit_code":<int>, ...ctx}`.
    /// Under text mode, falls back to a pass-through of `ctx`.
    pub fn print_dry_run(
        &self,
        out: &mut dyn Write,
        would_succeed: bool,
        exit_code: i32,
        ctx: &Value,
    ) {
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let mut obj = serde_json::Map::new();
                obj.insert("status".into(), Value::String("dry_run".into()));
                obj.insert("would_succeed".into(), Value::Bool(would_succeed));
                obj.insert("exit_code".into(), Value::from(exit_code));
                if let Some(map) = ctx.as_object() {
                    for (k, v) in map {
                        obj.insert(k.clone(), v.clone());
                    }
                }
                let envelope = Value::Object(obj);
                let body = if self.raw {
                    serde_json::to_string(&envelope).unwrap_or_else(|_| envelope.to_string())
                } else {
                    serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| envelope.to_string())
                };
                let _ = writeln!(out, "{body}");
            }
            OutputFormat::Text => self.print_response(out, ctx),
        }
    }

    /// Prints a canonical confirmation-required error envelope (U7).
    ///
    /// Emitted when a destructive op was invoked under `--no-interactive`
    /// without `--force`. `ctx` carries verb-context fields; the helper folds
    /// `status: "error"`, `reason: "confirmation-required"`, and `exit_code`
    /// into the same object on stderr.
    pub fn print_confirmation_required(
        &self,
        err: &mut dyn Write,
        ctx: &serde_json::Value,
        exit_code: i32,
    ) {
        let mut obj = if let serde_json::Value::Object(m) = ctx {
            m.clone()
        } else {
            serde_json::Map::new()
        };
        obj.insert(
            "status".to_string(),
            serde_json::Value::String("error".to_string()),
        );
        obj.insert(
            "reason".to_string(),
            serde_json::Value::String("confirmation-required".to_string()),
        );
        obj.insert(
            "exit_code".to_string(),
            serde_json::Value::Number(serde_json::Number::from(exit_code)),
        );
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let value = serde_json::Value::Object(obj);
                let pretty =
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());
                let _ = writeln!(err, "{pretty}");
            }
            OutputFormat::Text => {
                let cmd = obj
                    .get("command")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("operation");
                let line = if self.no_color {
                    format!(
                        "Error: confirmation required for {cmd} — pass --force or run interactively"
                    )
                } else {
                    format!(
                        "\x1b[31mError: confirmation required for {cmd} — pass --force or run interactively\x1b[0m"
                    )
                };
                let _ = writeln!(err, "{line}");
            }
        }
    }

    /// Prints a simple text message (e.g. version, auth status) to the supplied writer.
    /// Respects --output json by wrapping in a JSON object.
    pub fn print_message(&self, out: &mut dyn Write, msg: &str) {
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let clean = strip_ansi(msg);
                let json = serde_json::json!({"message": clean});
                let _ = writeln!(out, "{json}");
            }
            OutputFormat::Text => {
                if self.no_color {
                    let _ = writeln!(out, "{}", strip_ansi(msg));
                } else {
                    let _ = writeln!(out, "{msg}");
                }
            }
        }
    }
}

/// Strips ANSI escape codes from a string.
fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until 'm' (end of ANSI escape)
            for inner in chars.by_ref() {
                if inner == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_removes_color_codes() {
        assert_eq!(strip_ansi("\x1b[32mhello\x1b[0m"), "hello");
        assert_eq!(strip_ansi("\x1b[1;31mError\x1b[0m"), "Error");
        assert_eq!(strip_ansi("no codes here"), "no codes here");
    }

    #[test]
    fn test_xurl_error_kind_mapping() {
        assert_eq!(XurlError::Auth("test".into()).kind(), "auth-required");
        assert_eq!(XurlError::Http("test".into()).kind(), "network-error");
        assert_eq!(XurlError::api(400, "test").kind(), "network-error");
        assert_eq!(XurlError::api(401, "x").kind(), "auth-required");
        assert_eq!(XurlError::api(404, "x").kind(), "not-found");
        assert_eq!(XurlError::api(429, "x").kind(), "rate-limited");
        assert_eq!(XurlError::validation("test").kind(), "validation");
        assert_eq!(XurlError::Io("test".into()).kind(), "io");
        assert_eq!(XurlError::Json("test".into()).kind(), "serialization");
        assert_eq!(
            XurlError::InvalidMethod("X".into()).kind(),
            "invalid-method"
        );
        assert_eq!(XurlError::token_store("x").kind(), "token-store");
    }

    #[test]
    fn test_output_config_json_format() {
        let cfg = OutputConfig {
            format: OutputFormat::Json,
            quiet: false,
            no_color: false,
            use_color: true,
            verbose: false,
            raw: false,
        };
        assert!(!cfg.quiet);
    }

    #[test]
    fn test_no_color_env_overrides_color_always() {
        // Save and clear any existing NO_COLOR so the assertion is deterministic
        // in the unlikely case the test runner has it set.
        // SAFETY: tests in this module are single-threaded under cargo test by default;
        // no other thread reads NO_COLOR concurrently.
        let prior = std::env::var_os("NO_COLOR");
        // SAFETY: see above.
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
        let cfg = OutputConfig::new(OutputFormat::Text, false, false, ColorChoice::Always);
        assert!(!cfg.use_color, "NO_COLOR must defeat --color always");
        assert!(cfg.no_color, "no_color mirrors !use_color");
        // SAFETY: see above.
        unsafe {
            match prior {
                Some(v) => std::env::set_var("NO_COLOR", v),
                None => std::env::remove_var("NO_COLOR"),
            }
        }
    }

    #[test]
    fn test_color_never_disables_color() {
        // Force NO_COLOR off so the --color flag is the only driver here.
        // SAFETY: tests in this module are single-threaded under cargo test by default.
        let prior = std::env::var_os("NO_COLOR");
        // SAFETY: see above.
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
        let cfg = OutputConfig::new(OutputFormat::Text, false, false, ColorChoice::Never);
        assert!(!cfg.use_color);
        // SAFETY: see above.
        unsafe {
            if let Some(v) = prior {
                std::env::set_var("NO_COLOR", v);
            }
        }
    }

    #[test]
    fn test_color_always_enables_color_when_no_color_unset() {
        // SAFETY: tests in this module are single-threaded under cargo test by default.
        let prior = std::env::var_os("NO_COLOR");
        // SAFETY: see above.
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
        let cfg = OutputConfig::new(OutputFormat::Text, false, false, ColorChoice::Always);
        assert!(cfg.use_color, "--color always must enable color");
        // SAFETY: see above.
        unsafe {
            if let Some(v) = prior {
                std::env::set_var("NO_COLOR", v);
            }
        }
    }

    #[test]
    fn test_raw_forces_color_off() {
        let cfg =
            OutputConfig::new_with_raw(OutputFormat::Text, false, false, ColorChoice::Always, true);
        assert!(!cfg.use_color, "--raw must force use_color = false");
        assert!(cfg.raw);
    }
}
