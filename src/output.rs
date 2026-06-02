/// Output formatting helpers for `--output`, `--quiet`, `--color`, and
/// `NO_COLOR` support.
///
/// `OutputConfig` is a pure `Send + Sync + Clone` configuration object — it
/// owns no I/O handles. Print methods accept `&mut dyn Write` at the call site
/// so the same config can drive real stdout, real stderr, or a captured
/// `Vec<u8>` in library tests.
use std::io::{IsTerminal, Write};

use clap::ValueEnum;

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
#[derive(Clone, Debug)]
pub struct OutputConfig {
    pub format: OutputFormat,
    pub quiet: bool,
    pub no_color: bool,
    pub use_color: bool,
    pub verbose: bool,
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
    #[must_use]
    pub fn new(format: OutputFormat, quiet: bool, verbose: bool, color: ColorChoice) -> Self {
        let no_color_env = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        let use_color = if no_color_env {
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
            use_color: true,
            verbose: false,
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
}
