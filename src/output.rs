/// Output formatting helpers for `--output`, `--quiet`, and `NO_COLOR` support.
///
/// Centralizes all output decisions so command handlers don't need to
/// know about output formats directly.
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

    /// Prints an informational message to stderr (suppressed by --quiet or --output json/jsonl).
    pub fn info(&self, msg: &str) {
        if self.quiet || self.format != OutputFormat::Text {
            return;
        }
        eprintln!("{msg}");
    }

    /// Prints a success/status message with optional color to stderr.
    pub fn status(&self, msg: &str) {
        if self.quiet || self.format != OutputFormat::Text {
            return;
        }
        if self.no_color {
            eprintln!("{msg}");
        } else {
            eprintln!("\x1b[32m{msg}\x1b[0m");
        }
    }

    /// Prints an API response according to the configured output format.
    pub fn print_response(&self, value: &serde_json::Value) {
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                println!("{value}");
            }
            OutputFormat::Text => {
                if self.no_color {
                    let pretty =
                        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
                    println!("{pretty}");
                } else {
                    crate::api::response::format_and_print_response(value);
                }
            }
        }
    }

    /// Prints a streaming line according to the configured output format.
    #[allow(clippy::unused_self)]
    pub fn print_stream_line(&self, line: &str) {
        println!("{line}");
    }

    /// Formats and prints an error to stderr. When --output json/jsonl, emits structured JSON.
    pub fn print_error(&self, error: &XurlError, exit_code: i32) {
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let kind = error_kind(error);
                let msg = error.to_string();
                let json = serde_json::json!({
                    "error": msg,
                    "kind": kind,
                    "code": exit_code,
                });
                eprintln!("{json}");
            }
            OutputFormat::Text => {
                if self.no_color {
                    eprintln!("Error: {error}");
                } else {
                    eprintln!("\x1b[31mError: {error}\x1b[0m");
                }
            }
        }
    }

    /// Prints a simple text message to stdout (e.g. version, auth status).
    /// Respects --output json by wrapping in a JSON object.
    pub fn print_message(&self, msg: &str) {
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let clean = strip_ansi(msg);
                let json = serde_json::json!({"message": clean});
                println!("{json}");
            }
            OutputFormat::Text => {
                if self.no_color {
                    println!("{}", strip_ansi(msg));
                } else {
                    println!("{msg}");
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
        XurlError::Api(_) => "api",
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
        assert_eq!(error_kind(&XurlError::Api("test".into())), "api");
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
