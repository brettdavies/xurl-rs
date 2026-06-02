//! Verifies `OutputConfig` print methods write to the supplied `&mut dyn Write`
//! (U1 of the library-CLI-entrypoint plan).

use xurl::error::XurlError;
use xurl::output::{OutputConfig, OutputFormat};

/// Compile-time assertion: `OutputConfig` must remain a `Send + Sync` config
/// object so it can be shared across threads / tasks in the planned async
/// `ApiClient` (see `feedback_async_multithread_first_party`).
#[test]
fn output_config_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<OutputConfig>();
}

#[test]
fn print_message_writes_to_supplied_writer_text() {
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_message(&mut buf, "hi");
    let s = String::from_utf8(buf).expect("utf8");
    assert_eq!(s, "hi\n");
}

#[test]
fn print_message_json_wraps_as_envelope() {
    let cfg = OutputConfig {
        format: OutputFormat::Json,
        quiet: false,
        no_color: false,
        use_color: true,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_message(&mut buf, "hello");
    let s = String::from_utf8(buf).expect("utf8");
    assert!(s.contains("\"message\""), "expected JSON envelope: {s}");
    assert!(s.contains("hello"), "expected payload: {s}");
    assert!(s.ends_with('\n'));
}

#[test]
fn info_writes_nothing_when_quiet() {
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: true,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.info(&mut buf, "should be suppressed");
    assert!(buf.is_empty(), "quiet mode must produce no output");
}

#[test]
fn status_writes_nothing_when_quiet() {
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: true,
        no_color: false,
        use_color: true,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.status(&mut buf, "should be suppressed");
    assert!(buf.is_empty(), "quiet mode must produce no output");
}

#[test]
fn info_writes_nothing_when_format_is_json() {
    let cfg = OutputConfig {
        format: OutputFormat::Json,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.info(&mut buf, "machine path");
    assert!(buf.is_empty(), "json mode must suppress info()");
}

#[test]
fn status_writes_nothing_when_format_is_json() {
    let cfg = OutputConfig {
        format: OutputFormat::Json,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.status(&mut buf, "machine path");
    assert!(buf.is_empty(), "json mode must suppress status()");
}

#[test]
fn info_writes_message_when_text_and_not_quiet() {
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.info(&mut buf, "fyi");
    let s = String::from_utf8(buf).expect("utf8");
    assert_eq!(s, "fyi\n");
}

#[test]
fn print_response_emits_json_in_json_format() {
    let cfg = OutputConfig {
        format: OutputFormat::Json,
        quiet: false,
        no_color: false,
        use_color: true,
        verbose: false,
        raw: false,
    };
    let value = serde_json::json!({"id": "abc", "n": 7});
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).expect("utf8");
    let parsed: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON");
    assert_eq!(parsed["id"], "abc");
    assert_eq!(parsed["n"], 7);
}

#[test]
fn print_response_no_ansi_in_json_format() {
    let cfg = OutputConfig {
        format: OutputFormat::Json,
        quiet: false,
        no_color: false,
        use_color: true,
        verbose: false,
        raw: false,
    };
    let value = serde_json::json!({"id": "abc"});
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).expect("utf8");
    assert!(
        !s.contains('\x1b'),
        "JSON path must skip ANSI colorization: {s:?}"
    );
}

#[test]
fn print_response_text_no_color_writes_pretty_json() {
    // Text + no_color must go through the writer (the colorized text path
    // still calls into format.rs's println!-based functions per the U1/U2
    // boundary; U2 fixes that).
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let value = serde_json::json!({"k": "v"});
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).expect("utf8");
    assert!(s.contains("\"k\""));
    assert!(s.contains("\"v\""));
}

#[test]
fn print_stream_line_writes_with_trailing_newline() {
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: false,
        no_color: false,
        use_color: true,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_stream_line(&mut buf, "event-payload");
    let s = String::from_utf8(buf).expect("utf8");
    assert_eq!(s, "event-payload\n");
}

#[test]
fn print_error_writes_to_error_writer_only() {
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let mut err_buf: Vec<u8> = Vec::new();
    let err = XurlError::auth("token expired");
    cfg.print_error(&mut err_buf, &err, 2);
    let s = String::from_utf8(err_buf).expect("utf8");
    assert!(s.contains("token expired"), "expected error text: {s:?}");
    assert!(s.contains("Error:"), "expected Error prefix: {s:?}");
}

#[test]
fn print_error_emits_structured_json_when_format_is_json() {
    let cfg = OutputConfig {
        format: OutputFormat::Json,
        quiet: false,
        no_color: false,
        use_color: true,
        verbose: false,
        raw: false,
    };
    let mut err_buf: Vec<u8> = Vec::new();
    let err = XurlError::auth("bad token");
    cfg.print_error(&mut err_buf, &err, 77);
    let s = String::from_utf8(err_buf).expect("utf8");
    let parsed: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON");
    assert_eq!(parsed["status"], "error");
    assert_eq!(parsed["reason"], "auth-required");
    assert_eq!(parsed["exit_code"], 77);
    assert!(parsed["message"].as_str().unwrap().contains("bad token"));
}

#[test]
fn print_error_does_not_write_to_unrelated_stdout_buffer() {
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let mut stdout_buf: Vec<u8> = Vec::new();
    let mut err_buf: Vec<u8> = Vec::new();
    let err = XurlError::validation("nope");
    cfg.print_error(&mut err_buf, &err, 1);
    // Caller did not pass stdout_buf — it must remain untouched.
    assert!(stdout_buf.is_empty());
    let _ = std::io::Write::write_all(&mut stdout_buf, b"sanity");
    assert!(!err_buf.is_empty());
}

// U8: verbose/warning/progress contract — naked stdio elsewhere is barred,
// so these are the only legal channels for diagnostics.

#[test]
fn verbose_writes_under_text_when_verbose_flag_on() {
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: true,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.verbose(&mut buf, "> GET /2/users/me");
    let s = String::from_utf8(buf).expect("utf8");
    assert_eq!(s, "> GET /2/users/me\n");
}

#[test]
fn verbose_suppressed_under_json_even_when_verbose_on() {
    // U8 requirement: agents parsing structured output must not see verbose
    // request/response prefixes on stderr.
    let cfg = OutputConfig {
        format: OutputFormat::Json,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: true,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.verbose(&mut buf, "> GET /2/users/me");
    assert!(
        buf.is_empty(),
        "verbose() must not emit under JSON: {buf:?}"
    );
}

#[test]
fn verbose_suppressed_under_quiet() {
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: true,
        no_color: true,
        use_color: false,
        verbose: true,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.verbose(&mut buf, "should be silent");
    assert!(buf.is_empty());
}

#[test]
fn verbose_suppressed_when_verbose_flag_off() {
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.verbose(&mut buf, "noop");
    assert!(buf.is_empty());
}

#[test]
fn warning_writes_under_text() {
    let cfg = OutputConfig {
        format: OutputFormat::Text,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.warning(&mut buf, "token near expiry");
    let s = String::from_utf8(buf).expect("utf8");
    assert!(
        s.contains("warning: token near expiry"),
        "unexpected: {s:?}"
    );
}

#[test]
fn warning_suppressed_under_json() {
    // Per agent-native semantic-fields-over-stderr-warnings: warnings under
    // JSON modes are not emitted on stderr (envelope promotion is the future
    // home for them — plan U8 deferred).
    let cfg = OutputConfig {
        format: OutputFormat::Json,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.warning(&mut buf, "token near expiry");
    assert!(
        buf.is_empty(),
        "warnings on stderr must be suppressed under JSON: {buf:?}"
    );
}

#[test]
fn warning_under_jsonl_also_suppressed() {
    let cfg = OutputConfig {
        format: OutputFormat::Jsonl,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
    };
    let mut buf: Vec<u8> = Vec::new();
    cfg.warning(&mut buf, "near expiry");
    assert!(buf.is_empty());
}

#[test]
fn output_config_default_is_text_no_verbose() {
    let cfg = OutputConfig::default();
    assert_eq!(cfg.format, OutputFormat::Text);
    assert!(!cfg.verbose);
    assert!(!cfg.quiet);
    assert!(!cfg.raw);
    assert!(!cfg.use_color);
}
