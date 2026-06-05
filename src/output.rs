/// Output formatting helpers for `--output`, `--quiet`, `--color`, and
/// `NO_COLOR` support.
///
/// `OutputConfig` is a pure `Send + Sync + Clone` configuration object — it
/// owns no I/O handles. Print methods accept `&mut dyn Write` at the call site
/// so the same config can drive real stdout, real stderr, or a captured
/// `Vec<u8>` in library tests.
///
/// This module is the single owner of `println!` / `eprintln!`. Every other
/// `src/**/*.rs` site routes through one of [`OutputConfig`]'s methods or
/// [`warn_stderr`] for the rare deep call sites that cannot carry an
/// `OutputConfig`. A CI guard in `scripts/lint-stdio.sh` enforces the
/// invariant.
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
    /// Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name.
    Ndjson,
    /// YAML document (best-effort serialization of the JSON shape).
    Yaml,
    /// Comma-separated values (best-effort flattening of the top-level shape).
    Csv,
    /// Tab-separated values (best-effort flattening of the top-level shape).
    Tsv,
}

impl OutputFormat {
    /// Returns true for every machine-readable format. Equivalent to
    /// `*self != OutputFormat::Text`. Used by stderr-emitter methods
    /// (`info`, `status`, `warning`, `verbose`, `progress`) to suppress
    /// human-targeted chatter under any structured mode.
    #[must_use]
    pub fn is_structured(&self) -> bool {
        !matches!(self, OutputFormat::Text)
    }
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
    /// Set when the user passed `--no-interactive` (or `XURL_NO_INTERACTIVE`).
    ///
    /// Routed into `OutputConfig` so dialoguer-gating call sites can ask
    /// [`Self::is_interactive_terminal`] without re-reading the parsed `Cli`
    /// struct. Constructors default this to `false`; the runner sets it via
    /// [`Self::with_no_interactive`] right after construction.
    pub no_interactive: bool,
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
            no_interactive: false,
        }
    }

    /// Returns a copy of this config with the `no_interactive` field set.
    ///
    /// Used by the runner immediately after construction to thread the parsed
    /// `--no-interactive` (or `XURL_NO_INTERACTIVE`) flag into
    /// [`Self::is_interactive_terminal`].
    #[must_use]
    pub fn with_no_interactive(mut self, no_interactive: bool) -> Self {
        self.no_interactive = no_interactive;
        self
    }

    /// Returns `true` when the active session can drive interactive prompts.
    ///
    /// True only when:
    /// - `--no-interactive` is NOT set,
    /// - stdin is a TTY,
    /// - stderr is a TTY (dialoguer renders prompts on stderr).
    ///
    /// Call sites that drive `dialoguer::Select` / `dialoguer::Confirm`
    /// MUST gate on this — auto-engaging a prompt under a non-TTY session
    /// leaves the dialoguer state machine waiting on `/dev/null`.
    #[must_use]
    pub fn is_interactive_terminal(&self) -> bool {
        !self.no_interactive && std::io::stdin().is_terminal() && std::io::stderr().is_terminal()
    }

    /// Emits a canonical error envelope with an explicit kebab-case `reason`.
    ///
    /// Mirrors [`Self::print_error`] but lets the caller pin the `reason`
    /// (e.g. `"no-tty"`) rather than reading it from `XurlError::kind()`.
    /// Under text mode falls back to a plain "Error: …" line.
    pub fn print_error_envelope(
        &self,
        err: &mut dyn Write,
        reason: &str,
        exit_code: i32,
        message: &str,
    ) {
        let envelope = serde_json::json!({
            "status": "error",
            "reason": reason,
            "exit_code": exit_code,
            "message": message,
        });
        self.write_envelope_or_text_error(err, &envelope, message);
    }

    /// Prints an informational message (suppressed by --quiet or any
    /// structured `--output` mode).
    ///
    /// The runner passes a stderr writer here in the binary path; tests pass a `Vec<u8>`.
    pub fn info(&self, err: &mut dyn Write, msg: &str) {
        if self.quiet || self.format.is_structured() {
            return;
        }
        let _ = writeln!(err, "{msg}");
    }

    /// Prints a success/status message with optional color.
    ///
    /// The runner passes a stderr writer here in the binary path; tests pass a `Vec<u8>`.
    pub fn status(&self, err: &mut dyn Write, msg: &str) {
        if self.quiet || self.format.is_structured() {
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
            OutputFormat::Json | OutputFormat::Jsonl | OutputFormat::Ndjson => {
                let body = if self.raw || matches!(self.format, OutputFormat::Ndjson) {
                    serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
                } else {
                    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
                };
                let _ = writeln!(out, "{body}");
            }
            OutputFormat::Yaml => {
                let body = serde_yaml::to_string(value).unwrap_or_else(|_| value.to_string());
                let _ = write!(out, "{body}");
            }
            OutputFormat::Csv => {
                let _ = write_flattened(out, value, ',');
            }
            OutputFormat::Tsv => {
                let _ = write_flattened(out, value, '\t');
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
    /// Under any structured `--output`, emits the canonical envelope shape:
    /// `{"status":"error","reason":<kind>,"exit_code":<code>,"message":<display>}`.
    /// Json/Jsonl/Ndjson emit one JSON line; Yaml emits a YAML document; Csv/Tsv
    /// emit a JSON line carrying the envelope (delimited formats are not a good
    /// fit for nested error metadata).
    pub fn print_error(&self, err: &mut dyn Write, error: &XurlError, exit_code: i32) {
        let envelope = serde_json::json!({
            "status": "error",
            "reason": error.kind(),
            "exit_code": exit_code,
            "message": error.to_string(),
        });
        let display = error.to_string();
        self.write_envelope_or_text_error(err, &envelope, &display);
    }

    /// Emits a canonical success envelope under structured modes.
    ///
    /// Wraps `payload` (treated as a JSON object whose keys flatten in at
    /// the top level) with `{"status":"ok", ...payload}`. Under text mode
    /// the payload is passed through to [`Self::print_response`] so existing
    /// formatters keep their shape.
    pub fn print_success(&self, out: &mut dyn Write, payload: &Value) {
        if !self.format.is_structured() {
            self.print_response(out, payload);
            return;
        }
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
        self.write_structured(out, &envelope);
    }

    /// Emits a canonical dry-run envelope under structured modes.
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
        if !self.format.is_structured() {
            self.print_response(out, ctx);
            return;
        }
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
        self.write_structured(out, &envelope);
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
        if self.format.is_structured() {
            let value = serde_json::Value::Object(obj);
            self.write_structured(err, &value);
        } else {
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

    /// Emits a verbose diagnostic line to `err` when verbose is on, quiet is
    /// off, and the format is text.
    ///
    /// Under `--output json` or `--output jsonl`, agents parsing structured
    /// output must not encounter interleaved human text on stderr, so
    /// `verbose` is suppressed (per the agent-native semantic-fields-over-
    /// stderr-warnings principle). Mirrors the `diag!` macro pattern from the
    /// bird CLI without the macro.
    pub fn verbose(&self, err: &mut dyn Write, msg: &str) {
        if !self.verbose || self.quiet || self.format.is_structured() {
            return;
        }
        let _ = writeln!(err, "{msg}");
    }

    /// Emits a warning to `err`. Always goes to stderr in text mode; under
    /// JSON modes the line is suppressed (the canonical envelope is the
    /// channel for structured warnings — see plan U8's deferred
    /// `warnings: []` envelope promotion).
    ///
    /// Suppressed entirely under `--quiet` combined with JSON modes; under
    /// `--quiet` text mode, warnings still surface (errors and warnings are
    /// the load-bearing signals the operator must see).
    pub fn warning(&self, err: &mut dyn Write, msg: &str) {
        if self.format.is_structured() {
            return;
        }
        if self.no_color {
            let _ = writeln!(err, "warning: {msg}");
        } else {
            let _ = writeln!(err, "\x1b[1;33mwarning:\x1b[0m {msg}");
        }
    }

    /// Emits a progress / status line to `err` when the format is text and
    /// stderr is a TTY. Quiet suppresses progress unconditionally.
    pub fn progress(&self, err: &mut dyn Write, msg: &str) {
        if self.quiet || self.format.is_structured() {
            return;
        }
        if !std::io::stderr().is_terminal() {
            return;
        }
        let _ = writeln!(err, "{msg}");
    }

    /// Prints a simple text message (e.g. version, auth status) to the supplied writer.
    /// Respects --output json by wrapping in a JSON object.
    pub fn print_message(&self, out: &mut dyn Write, msg: &str) {
        if self.format.is_structured() {
            let clean = strip_ansi(msg);
            let value = serde_json::json!({"message": clean});
            self.write_structured(out, &value);
            return;
        }
        if self.no_color {
            let _ = writeln!(out, "{}", strip_ansi(msg));
        } else {
            let _ = writeln!(out, "{msg}");
        }
    }

    /// Renders `value` to `w` in the active structured format.
    ///
    /// Json/Jsonl pretty-print (compact under `--raw`); Ndjson always emits a
    /// single line; Yaml emits a YAML document; Csv/Tsv fall back to one line
    /// of JSON because nested envelope metadata isn't a good fit for a
    /// flat delimited table.
    fn write_structured(&self, w: &mut dyn Write, value: &Value) {
        match self.format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let body = if self.raw {
                    serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
                } else {
                    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
                };
                let _ = writeln!(w, "{body}");
            }
            OutputFormat::Ndjson => {
                let body = serde_json::to_string(value).unwrap_or_else(|_| value.to_string());
                let _ = writeln!(w, "{body}");
            }
            OutputFormat::Yaml => {
                let body = serde_yaml::to_string(value).unwrap_or_else(|_| value.to_string());
                let _ = write!(w, "{body}");
            }
            OutputFormat::Csv | OutputFormat::Tsv => {
                // Envelopes are nested by design; fall back to one JSON line.
                let body = serde_json::to_string(value).unwrap_or_else(|_| value.to_string());
                let _ = writeln!(w, "{body}");
            }
            OutputFormat::Text => {
                // Unreachable in practice — guarded by callers — but keep a
                // safe fallback rather than panic.
                let body =
                    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
                let _ = writeln!(w, "{body}");
            }
        }
    }

    /// Common helper for the error printers: routes the structured envelope
    /// through [`Self::write_structured`] in machine modes, falling back to a
    /// colored "Error: {display}" line in text mode.
    fn write_envelope_or_text_error(&self, err: &mut dyn Write, envelope: &Value, display: &str) {
        if self.format.is_structured() {
            self.write_structured(err, envelope);
        } else if self.no_color {
            let _ = writeln!(err, "Error: {display}");
        } else {
            let _ = writeln!(err, "\x1b[31mError: {display}\x1b[0m");
        }
    }
}

impl Default for OutputConfig {
    /// Library-friendly default: text format, color-auto, no verbose, no quiet,
    /// no raw. Matches what an interactive operator gets without flags. Used
    /// when an `ApiClient` is constructed before the runner has resolved the
    /// real `OutputConfig`.
    fn default() -> Self {
        Self {
            format: OutputFormat::Text,
            quiet: false,
            no_color: true,
            use_color: false,
            verbose: false,
            raw: false,
            no_interactive: false,
        }
    }
}

/// Emits a one-line warning to stderr. Single-owner escape hatch for deep
/// call sites that cannot reasonably carry an `OutputConfig` (token-store
/// migration, env-var rejection during config resolution, callback partial-
/// bind notices, OAuth2 salvage warnings). The CI guard at
/// `scripts/lint-stdio.sh` allow-lists this module so the discipline holds:
/// every `eprintln!` lives here.
pub fn warn_stderr(msg: &str) {
    eprintln!("warning: {msg}");
}

/// Best-effort delimited-table emitter for the `csv` / `tsv` output formats.
///
/// Strategy:
/// - Object → one header row + one value row.
/// - Array of objects → header is the union of keys (insertion-ordered by
///   first occurrence), followed by one row per element. Missing keys emit
///   an empty cell.
/// - Array of scalars → single-column `value` table, one row per element.
/// - Scalar → single-column `value` table with one row.
///
/// Nested values (objects, arrays inside a cell) are serialized as JSON so
/// each cell stays a single delimited field. A `_warning` column is appended
/// with the note "nested values JSON-stringified" so downstream tools see why
/// some cells carry JSON.
///
/// The result is forward-only and best-effort: malformed shapes degrade
/// gracefully into a one-line JSON blob in a `_payload` column rather than
/// erroring out.
fn write_flattened(out: &mut dyn Write, value: &Value, sep: char) -> std::io::Result<()> {
    let rows: Vec<&Value> = match value {
        Value::Array(arr) => arr.iter().collect(),
        other => vec![other],
    };

    // Collect the union of object keys to use as the column header.
    let mut header: Vec<String> = Vec::new();
    let mut all_objects = true;
    for row in &rows {
        if let Value::Object(map) = row {
            for k in map.keys() {
                if !header.iter().any(|h| h == k) {
                    header.push(k.clone());
                }
            }
        } else {
            all_objects = false;
        }
    }

    if !all_objects || header.is_empty() {
        // Scalar / mixed shapes: single-column "value" table.
        writeln!(out, "value")?;
        for row in &rows {
            let cell = scalar_cell(row);
            writeln!(out, "{}", delimited_escape(&cell, sep))?;
        }
        return Ok(());
    }

    // Detect whether any cell needed JSON stringification so we can emit a
    // single `_warning` column at the end, only when relevant.
    let mut needs_warning = false;
    let row_cells: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            header
                .iter()
                .map(|key| {
                    let cell =
                        row.as_object()
                            .and_then(|m| m.get(key))
                            .map_or_else(String::new, |v| {
                                if matches!(v, Value::Object(_) | Value::Array(_)) {
                                    needs_warning = true;
                                    serde_json::to_string(v).unwrap_or_else(|_| v.to_string())
                                } else {
                                    scalar_cell(v)
                                }
                            });
                    delimited_escape(&cell, sep)
                })
                .collect()
        })
        .collect();

    // Header line.
    let mut header_out: Vec<String> = header.iter().map(|h| delimited_escape(h, sep)).collect();
    if needs_warning {
        header_out.push(delimited_escape("_warning", sep));
    }
    writeln!(out, "{}", header_out.join(&sep.to_string()))?;

    for cells in &row_cells {
        let mut line = cells.clone();
        if needs_warning {
            line.push(delimited_escape("nested values JSON-stringified", sep));
        }
        writeln!(out, "{}", line.join(&sep.to_string()))?;
    }
    Ok(())
}

/// Renders a scalar [`Value`] as the unquoted cell body (the caller wraps with
/// `delimited_escape` to handle the delimiter / quote rules).
fn scalar_cell(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Object(_) | Value::Array(_) => {
            serde_json::to_string(v).unwrap_or_else(|_| v.to_string())
        }
    }
}

/// Applies RFC 4180 quoting when needed.
///
/// For comma-delimited output: wraps in double quotes when the cell contains
/// a comma, double quote, CR, or LF, and escapes embedded quotes. For tab-
/// delimited output: replaces embedded tabs and newlines with spaces (TSV
/// has no canonical quoting rule, and most consumers reject control chars
/// inside fields).
fn delimited_escape(cell: &str, sep: char) -> String {
    if sep == '\t' {
        return cell.replace(['\t', '\n', '\r'], " ");
    }
    if cell.contains(sep) || cell.contains('"') || cell.contains('\n') || cell.contains('\r') {
        let escaped = cell.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        cell.to_string()
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
            no_interactive: false,
        };
        assert!(!cfg.quiet);
    }

    #[test]
    fn test_with_no_interactive_threads_field() {
        let cfg = OutputConfig::new(OutputFormat::Text, false, false, ColorChoice::Never)
            .with_no_interactive(true);
        assert!(cfg.no_interactive);
        // is_interactive_terminal is false when no_interactive is true regardless of TTY.
        assert!(!cfg.is_interactive_terminal());
    }

    #[test]
    fn test_print_error_envelope_json_shape() {
        let cfg = OutputConfig::new(OutputFormat::Json, false, false, ColorChoice::Never);
        let mut buf: Vec<u8> = Vec::new();
        cfg.print_error_envelope(&mut buf, "no-tty", 1, "stdin is not a terminal");
        let s = String::from_utf8(buf).expect("utf8");
        let v: serde_json::Value = serde_json::from_str(s.trim()).expect("valid json");
        assert_eq!(v["status"], "error");
        assert_eq!(v["reason"], "no-tty");
        assert_eq!(v["exit_code"], 1);
        assert_eq!(v["message"], "stdin is not a terminal");
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

    #[test]
    fn test_verbose_emits_under_text_when_verbose_flag_set() {
        let cfg = OutputConfig::new(OutputFormat::Text, false, true, ColorChoice::Never);
        let mut buf = Vec::new();
        cfg.verbose(&mut buf, "hello");
        assert_eq!(buf, b"hello\n");
    }

    #[test]
    fn test_verbose_suppressed_under_json() {
        let cfg = OutputConfig::new(OutputFormat::Json, false, true, ColorChoice::Never);
        let mut buf = Vec::new();
        cfg.verbose(&mut buf, "hello");
        assert!(buf.is_empty(), "verbose must not leak under JSON");
    }

    #[test]
    fn test_verbose_suppressed_under_quiet() {
        let cfg = OutputConfig::new(OutputFormat::Text, true, true, ColorChoice::Never);
        let mut buf = Vec::new();
        cfg.verbose(&mut buf, "hello");
        assert!(buf.is_empty());
    }

    #[test]
    fn test_verbose_suppressed_when_flag_off() {
        let cfg = OutputConfig::new(OutputFormat::Text, false, false, ColorChoice::Never);
        let mut buf = Vec::new();
        cfg.verbose(&mut buf, "hello");
        assert!(buf.is_empty());
    }

    #[test]
    fn test_warning_emits_under_text() {
        let cfg = OutputConfig::new(OutputFormat::Text, false, false, ColorChoice::Never);
        let mut buf = Vec::new();
        cfg.warning(&mut buf, "rate limited");
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("warning: rate limited"));
    }

    #[test]
    fn test_warning_suppressed_under_json() {
        let cfg = OutputConfig::new(OutputFormat::Json, false, false, ColorChoice::Never);
        let mut buf = Vec::new();
        cfg.warning(&mut buf, "rate limited");
        assert!(
            buf.is_empty(),
            "warnings on stderr must be suppressed under JSON modes"
        );
    }
}
