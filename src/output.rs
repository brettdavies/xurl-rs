/// Output formatting helpers for `--output`, `--quiet`, and `NO_COLOR` support.
///
/// `OutputConfig` is a pure `Send + Sync + Clone` configuration object — it
/// owns no I/O handles. Print methods accept `&mut dyn Write` at the call site
/// so the same config can drive real stdout, real stderr, or a captured
/// `Vec<u8>` in library tests.
use std::io::Write;

use clap::ValueEnum;

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
#[derive(Clone, Debug)]
pub struct OutputConfig {
    pub format: OutputFormat,
    pub quiet: bool,
    pub no_color: bool,
}

impl OutputConfig {
    /// Creates an `OutputConfig` from CLI flags and environment.
    #[must_use]
    pub fn new(format: OutputFormat, quiet: bool) -> Self {
        let no_color = std::env::var("NO_COLOR").is_ok();
        Self {
            format,
            quiet,
            no_color,
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
    pub fn print_response(&self, out: &mut dyn Write, value: &serde_json::Value) {
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let pretty =
                    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
                let _ = writeln!(out, "{pretty}");
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
    /// When --output json/jsonl, emits structured JSON.
    pub fn print_error(&self, err: &mut dyn Write, error: &XurlError, exit_code: i32) {
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let kind = error_kind(error);
                let msg = error.to_string();
                let json = serde_json::json!({
                    "error": msg,
                    "kind": kind,
                    "code": exit_code,
                });
                let _ = writeln!(err, "{json}");
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

/// Returns a string category for an error variant.
fn error_kind(e: &XurlError) -> &'static str {
    match e {
        XurlError::Auth(_) => "auth",
        XurlError::Http(_) => "http",
        XurlError::Api { .. } => "api",
        XurlError::Validation(_) => "validation",
        XurlError::Io(_) => "io",
        XurlError::Json(_) => "json",
        XurlError::InvalidMethod(_) => "invalid_method",
        XurlError::TokenStore(_) => "token_store",
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
    fn test_error_kind_mapping() {
        assert_eq!(error_kind(&XurlError::Auth("test".into())), "auth");
        assert_eq!(error_kind(&XurlError::Http("test".into())), "http");
        assert_eq!(error_kind(&XurlError::api(400, "test")), "api");
        assert_eq!(error_kind(&XurlError::validation("test")), "validation");
        assert_eq!(error_kind(&XurlError::Io("test".into())), "io");
    }

    #[test]
    fn test_output_config_json_format() {
        let cfg = OutputConfig {
            format: OutputFormat::Json,
            quiet: false,
            no_color: false,
        };
        assert!(!cfg.quiet);
    }
}
