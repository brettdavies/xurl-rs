//! Output formatting helpers for `--output`, `--quiet`, `--color`, and
//! `NO_COLOR` support.
//!
//! `OutputConfig` is a pure `Send + Sync + Clone` configuration object — it
//! owns no I/O handles. Print methods accept `&mut dyn Write` at the call site
//! so the same config can drive real stdout, real stderr, or a captured
//! `Vec<u8>` in library tests.
//!
//! Nothing under `src/` outside `src/cli/` writes to a terminal: library
//! code routes its diagnostics through `tracing` events that the
//! `Diagnostics` subscriber renders, and every printed line goes through one
//! of [`OutputConfig`]'s methods. `scripts/lint-stdio.sh` enforces both.

mod delimited;
mod diagnostics;
mod format;
mod message;

pub(crate) use diagnostics::Diagnostics;

use std::io::{IsTerminal, Write};

use clap::ValueEnum;
use serde_json::Value;

use crate::cli::ColorChoice;
use crate::cli::envelope::ErrorBody;
use delimited::write_flattened;
use xdk::error::Error;

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
/// interior mutability. This keeps it `Send + Sync + Clone`, which the
/// async `Client` requires.
///
/// `use_color` is the resolved color decision after combining `--color`, the
/// `NO_COLOR` env var, and stderr's TTY-ness. `no_color` is preserved as the
/// negation (`!use_color`) for source-compatibility with existing call sites
/// and tests that constructed `OutputConfig { … }` directly.
///
/// `raw` (from `--raw`) forces compact JSON (no pretty-printing) and strips
/// ANSI styling from text output. Useful for pipelines that line-buffer.
///
/// # Example
///
/// ```rust,no_run
/// use xurl::cli::ColorChoice;
/// use xurl::cli::output::{OutputConfig, OutputFormat};
///
/// let cfg = OutputConfig::new(OutputFormat::Json, false, false, ColorChoice::Never);
///
/// let mut buf: Vec<u8> = Vec::new();
/// let payload = serde_json::json!({ "status": "ok" });
/// cfg.print_response(&mut buf, &payload);
///
/// let rendered = String::from_utf8(buf).unwrap();
/// assert!(rendered.contains("\"status\""));
/// ```
#[derive(Clone, Debug)]
pub struct OutputConfig {
    /// Resolved output format from `--output` / `--json` / `--jsonl` /
    /// `XURL_OUTPUT`.
    pub format: OutputFormat,
    /// Suppress non-essential human chatter (`--quiet` / `XURL_QUIET`).
    pub quiet: bool,
    /// Negation of [`Self::use_color`], kept for source compatibility with
    /// call sites that pattern-match on the negative form.
    pub no_color: bool,
    /// Resolved color decision after combining `--color`, `NO_COLOR`, and
    /// stderr's TTY-ness. The single source of truth for "should I emit
    /// ANSI escapes".
    pub use_color: bool,
    /// Enable verbose request/response logging
    /// (`--verbose` / `-v` / `XURL_VERBOSE`).
    pub verbose: bool,
    /// Emit unstyled, compact output (`--raw` / `XURL_RAW`). Strips ANSI in
    /// text mode and forces compact JSON in machine modes.
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
    /// Creates an `OutputConfig` from resolved CLI flags, with `raw` off and
    /// no `NO_COLOR` override.
    ///
    /// `use_color` is computed from `color` and `std::io::stderr().is_terminal()`:
    /// - `--color always` overrides the TTY check.
    /// - `--color never` disables color unconditionally.
    /// - `--color auto` enables color only when stderr is a TTY.
    ///
    /// The runner reads `NO_COLOR` from the process and calls
    /// [`new_with_no_color`]; this constructor never consults the environment.
    ///
    /// [`new_with_no_color`]: Self::new_with_no_color
    #[must_use]
    pub fn new(format: OutputFormat, quiet: bool, verbose: bool, color: ColorChoice) -> Self {
        Self::new_with_no_color(format, quiet, verbose, color, false, false)
    }

    /// The full constructor, with the `raw` flag and the `NO_COLOR` decision
    /// supplied.
    ///
    /// `no_color_env` is `true` when `NO_COLOR` is set to a non-empty value,
    /// which disables colour regardless of `color` (per
    /// <https://no-color.org/>). `raw` forces `use_color = false` and switches
    /// JSON output to compact form. Callers that already resolved their
    /// environment, and tests, pass the flag directly rather than exporting
    /// the variable.
    #[must_use]
    pub fn new_with_no_color(
        format: OutputFormat,
        quiet: bool,
        verbose: bool,
        color: ColorChoice,
        raw: bool,
        no_color_env: bool,
    ) -> Self {
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
    /// (e.g. `"no-tty"`) rather than reading it from `Error::kind()`.
    /// Under text mode falls back to a plain "Error: …" line.
    pub fn print_error_envelope(
        &self,
        err: &mut dyn Write,
        reason: &str,
        exit_code: i32,
        message: &str,
    ) {
        self.emit_error_envelope(
            err,
            ErrorBody {
                reason: reason.to_string(),
                exit_code,
                message: Some(message.to_string()),
                ..ErrorBody::default()
            },
        );
    }

    /// Prints an error with a recovery hint attached.
    ///
    /// Text mode puts the hint lines after the error line, outside the color
    /// wrap, and drops them under `--quiet`, which suppresses advice and
    /// never the error. Structured modes fold the typed step into the
    /// envelope instead, leaving every other key exactly as it was.
    pub(crate) fn print_error_with_hint(
        &self,
        err: &mut dyn Write,
        error: &Error,
        exit_code: i32,
        hint: &crate::cli::hints::Hint,
    ) {
        if self.format.is_structured() {
            let display = message::render(error);
            let body = ErrorBody {
                reason: error.kind().to_string(),
                exit_code,
                message: Some(display),
                next_step: Some(hint.next_step.clone()),
                ..ErrorBody::default()
            };
            self.emit_error_envelope(err, body);
            return;
        }
        self.print_error(err, error, exit_code);
        if self.quiet {
            return;
        }
        for line in &hint.text_lines {
            let _ = writeln!(err, "{line}");
        }
    }

    /// The one path every error envelope takes.
    ///
    /// ```text
    ///   print_error ─┐
    ///   print_error_envelope ─┤
    ///   print_confirmation_required ─┼─> ErrorBody ─> text "Error:" line
    ///   emit_invalid_args_envelope ─┘                 or structured document
    /// ```
    ///
    /// Building the typed [`ErrorBody`] here is what keeps the generated
    /// schema honest: a key no field declares cannot be emitted.
    pub(crate) fn emit_error_envelope(&self, err: &mut dyn Write, body: ErrorBody) {
        let display = body.message.clone().unwrap_or_default();
        let envelope = body.into_value();
        self.write_envelope_or_text_error(err, &envelope, &display);
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
                    let _ = format::format_response(out, value);
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
    pub fn print_error(&self, err: &mut dyn Write, error: &Error, exit_code: i32) {
        let display = message::render(error);
        let mut body = ErrorBody {
            reason: error.kind().to_string(),
            exit_code,
            message: Some(display),
            ..ErrorBody::default()
        };
        // `AuthMethodMismatch` carries structured fields: the envelope folds
        // `endpoint` (template), `rendered_url` (substituted), `method`,
        // `requested`, `supported`, `available_in_app`, `app`, and
        // `other_apps_with_creds` alongside the standard `message`. Agents
        // pattern-match on these without re-parsing the human message.
        if let Error::AuthMethodMismatch {
            endpoint,
            rendered_url,
            method,
            requested,
            supported,
            available_in_app,
            app,
            other_apps_with_creds,
        } = error
        {
            body.endpoint = Some(endpoint.clone());
            body.rendered_url = rendered_url.clone();
            body.method = Some(method.clone());
            body.requested = Some(match requested {
                Some(s) => Value::String(s.clone()),
                None => Value::Null,
            });
            body.supported = Some(supported.clone());
            body.available_in_app = available_in_app.clone();
            body.app = app.clone();
            body.other_apps_with_creds = other_apps_with_creds.clone();
        }
        self.emit_error_envelope(err, body);
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

    /// Prints a canonical confirmation-required error envelope.
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
        let map = match ctx {
            serde_json::Value::Object(m) => m.clone(),
            _ => serde_json::Map::new(),
        };
        let get_str = |k: &str| map.get(k).and_then(Value::as_str).map(str::to_string);
        let get_bool = |k: &str| map.get(k).and_then(Value::as_bool);
        let body = ErrorBody {
            reason: "confirmation-required".to_string(),
            exit_code,
            message: None,
            command: get_str("command"),
            post_id: get_str("post_id"),
            name: get_str("name"),
            all: get_bool("all"),
            oauth1: get_bool("oauth1"),
            oauth2_username: map.get("oauth2_username").cloned(),
            bearer: get_bool("bearer"),
            ..ErrorBody::default()
        };
        if self.format.is_structured() {
            self.emit_error_envelope(err, body);
        } else {
            let cmd = map
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
    /// channel for structured warnings).
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

    /// Prints a success message: a status-ok envelope for a structured
    /// caller, the message itself for a human.
    ///
    /// The message-shaped verbs share one success contract this way, so an
    /// agent branches on `status` across the whole surface rather than
    /// inferring success from the absence of an error.
    pub fn print_ok_message(&self, out: &mut dyn Write, msg: &str) {
        if self.format.is_structured() {
            let clean = strip_ansi(msg);
            self.print_success(out, &serde_json::json!({"message": clean}));
            return;
        }
        self.print_message(out, msg);
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
    /// when a `Client` is constructed before the runner has resolved the
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
        assert_eq!(Error::Auth("test".into()).kind(), "auth-required");
        assert_eq!(Error::Http("test".into()).kind(), "network-error");
        assert_eq!(Error::api(400, "test").kind(), "network-error");
        assert_eq!(Error::api(401, "x").kind(), "auth-required");
        assert_eq!(Error::api(404, "x").kind(), "not-found");
        assert_eq!(Error::api(429, "x").kind(), "rate-limited");
        assert_eq!(Error::validation("test").kind(), "validation");
        assert_eq!(Error::Io("test".into()).kind(), "io");
        assert_eq!(Error::Json("test".into()).kind(), "serialization");
        assert_eq!(Error::InvalidMethod("X".into()).kind(), "invalid-method");
        assert_eq!(Error::token_store("x").kind(), "token-store");
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
        let cfg = OutputConfig::new_with_no_color(
            OutputFormat::Text,
            false,
            false,
            ColorChoice::Always,
            false,
            true,
        );
        assert!(!cfg.use_color, "NO_COLOR must defeat --color always");
        assert!(cfg.no_color, "no_color mirrors !use_color");
    }

    #[test]
    fn test_color_never_disables_color() {
        let cfg = OutputConfig::new_with_no_color(
            OutputFormat::Text,
            false,
            false,
            ColorChoice::Never,
            false,
            false,
        );
        assert!(!cfg.use_color);
    }

    #[test]
    fn test_color_always_enables_color_when_no_color_unset() {
        let cfg = OutputConfig::new_with_no_color(
            OutputFormat::Text,
            false,
            false,
            ColorChoice::Always,
            false,
            false,
        );
        assert!(cfg.use_color, "--color always must enable color");
    }

    #[test]
    fn test_raw_forces_color_off() {
        let cfg = OutputConfig::new_with_no_color(
            OutputFormat::Text,
            false,
            false,
            ColorChoice::Always,
            true,
            false,
        );
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
