//! Verifies `OutputConfig` print methods write to the supplied `&mut dyn Write`
//! (U1 of the library-CLI-entrypoint plan).

use xdk::Error;
use xurl::cli::output::{OutputConfig, OutputFormat};

/// Compile-time assertion: `OutputConfig` must remain a `Send + Sync` config
/// object so it can be shared across threads / tasks in the planned async
/// `Client` (see `feedback_async_multithread_first_party`).
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
    };
    let mut err_buf: Vec<u8> = Vec::new();
    let err = Error::auth("token expired");
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
        no_interactive: false,
    };
    let mut err_buf: Vec<u8> = Vec::new();
    let err = Error::auth("bad token");
    cfg.print_error(&mut err_buf, &err, 77);
    let s = String::from_utf8(err_buf).expect("utf8");
    let parsed: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON");
    assert_eq!(parsed["status"], "error");
    assert_eq!(parsed["reason"], "auth-required");
    assert_eq!(parsed["exit_code"], 77);
    assert!(parsed["message"].as_str().unwrap().contains("bad token"));
}

/// R10: `AuthMethodMismatch` JSON envelope folds in `endpoint`, `method`,
/// `requested`, `supported`, and `message`. The U6 explicit-mismatch shape
/// MUST omit `available_in_app` (U7's empty-intersection shape adds it).
#[test]
fn print_error_auth_method_mismatch_envelope_shape_r10() {
    let cfg = OutputConfig {
        format: OutputFormat::Json,
        quiet: false,
        no_color: false,
        use_color: true,
        verbose: false,
        raw: false,
        no_interactive: false,
    };
    let mut err_buf: Vec<u8> = Vec::new();
    let err = Error::from(xdk::error::AuthMismatch {
        endpoint: "/2/media/upload".to_string(),
        rendered_url: None,
        method: "POST".to_string(),
        requested: Some("app".to_string()),
        supported: vec!["oauth1".to_string(), "oauth2".to_string()],
        available_in_app: None,
        app: None,
        other_apps_with_creds: None,
    });
    cfg.print_error(&mut err_buf, &err, 2);
    let s = String::from_utf8(err_buf).expect("utf8");
    let parsed: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON");

    assert_eq!(parsed["status"], "error");
    assert_eq!(parsed["reason"], "auth-method-mismatch");
    assert_eq!(parsed["exit_code"], 2);
    assert_eq!(parsed["endpoint"], "/2/media/upload");
    assert_eq!(parsed["method"], "POST");
    assert_eq!(parsed["requested"], "app");
    assert_eq!(
        parsed["supported"],
        serde_json::json!(["oauth1", "oauth2"]),
        "supported list must preserve order from validate()"
    );
    assert!(
        parsed.get("available_in_app").is_none(),
        "explicit-mismatch envelope must omit available_in_app"
    );
    assert!(
        parsed.get("rendered_url").is_none(),
        "rendered_url is optional and omitted when None"
    );
    assert!(
        parsed.get("app").is_none(),
        "app is optional and omitted when None"
    );

    let msg = parsed["message"].as_str().expect("message string");
    assert!(
        msg.contains("Bearer (app)"),
        "envelope message must pretty-print scheme: {msg}"
    );
    assert!(
        msg.contains("POST /2/media/upload"),
        "envelope message must reference method + endpoint: {msg}"
    );
    assert!(
        msg.contains("--auth oauth1") && msg.contains("--auth oauth2"),
        "envelope message must list both alternatives: {msg}"
    );
}

/// U7 forward-compat: the empty-intersection shape (`requested = None`,
/// `available_in_app = Some([...])`) must produce a `requested: null` JSON
/// value and include the `available_in_app` array. U6 commits to this
/// envelope shape so U7 doesn't have to rewrite the serializer.
#[test]
fn print_error_auth_method_mismatch_envelope_empty_intersection_shape() {
    let cfg = OutputConfig {
        format: OutputFormat::Json,
        quiet: false,
        no_color: false,
        use_color: true,
        verbose: false,
        raw: false,
        no_interactive: false,
    };
    let mut err_buf: Vec<u8> = Vec::new();
    let err = Error::from(xdk::error::AuthMismatch {
        endpoint: "/2/media/upload".to_string(),
        rendered_url: None,
        method: "POST".to_string(),
        requested: None,
        supported: vec!["oauth2".to_string()],
        available_in_app: Some(vec!["oauth1".to_string()]),
        app: Some("default".to_string()),
        other_apps_with_creds: None,
    });
    cfg.print_error(&mut err_buf, &err, 2);
    let s = String::from_utf8(err_buf).expect("utf8");
    let parsed: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON");

    assert_eq!(parsed["reason"], "auth-method-mismatch");
    assert_eq!(parsed["requested"], serde_json::Value::Null);
    assert_eq!(parsed["available_in_app"], serde_json::json!(["oauth1"]));
    let msg = parsed["message"].as_str().expect("message string");
    assert!(
        msg.contains("No stored auth method")
            && msg.contains("App has: oauth1")
            && msg.contains("Endpoint accepts: oauth2"),
        "empty-intersection message must surface what app has vs endpoint accepts: {msg}"
    );
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
        no_interactive: false,
    };
    let mut stdout_buf: Vec<u8> = Vec::new();
    let mut err_buf: Vec<u8> = Vec::new();
    let err = Error::validation("nope");
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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
        no_interactive: false,
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

// ── U13: csv / tsv / yaml / ndjson format coverage ──────────────────

fn fmt_cfg(format: OutputFormat) -> OutputConfig {
    OutputConfig {
        format,
        quiet: false,
        no_color: true,
        use_color: false,
        verbose: false,
        raw: false,
        no_interactive: false,
    }
}

#[test]
fn yaml_format_writes_yaml_document() {
    let cfg = fmt_cfg(OutputFormat::Yaml);
    let value = serde_json::json!({"id": "1", "text": "hi"});
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("id: '1'") || s.contains("id: \"1\""), "got: {s}");
    assert!(s.contains("text: hi"), "got: {s}");
}

#[test]
fn ndjson_format_emits_one_compact_json_line() {
    let cfg = fmt_cfg(OutputFormat::Ndjson);
    let value = serde_json::json!({"id": "1", "text": "hi"});
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).unwrap();
    let trimmed = s.trim_end_matches('\n');
    assert!(!trimmed.contains('\n'), "expected one line: {s:?}");
    assert!(s.ends_with('\n'));
}

#[test]
fn csv_format_writes_header_and_row() {
    let cfg = fmt_cfg(OutputFormat::Csv);
    let value = serde_json::json!({"id": "1", "text": "hi"});
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).unwrap();
    let mut lines = s.lines();
    let header = lines.next().expect("header line");
    let row = lines.next().expect("row line");
    assert!(header.contains("id"));
    assert!(header.contains("text"));
    assert!(row.contains("hi"));
    assert!(row.contains('1'));
}

#[test]
fn csv_format_handles_array_with_union_of_keys() {
    let cfg = fmt_cfg(OutputFormat::Csv);
    let value = serde_json::json!([
        {"id": "1", "text": "hi"},
        {"id": "2", "extra": "x"},
    ]);
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).unwrap();
    let header = s.lines().next().unwrap();
    assert!(header.contains("id"));
    assert!(header.contains("text"));
    assert!(header.contains("extra"));
}

#[test]
fn csv_format_quotes_cells_with_commas() {
    let cfg = fmt_cfg(OutputFormat::Csv);
    let value = serde_json::json!({"text": "hi, world"});
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("\"hi, world\""), "expected quoted cell: {s}");
}

#[test]
fn csv_format_quotes_cells_with_newlines() {
    let cfg = fmt_cfg(OutputFormat::Csv);
    let value = serde_json::json!({"text": "first\nsecond"});
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).unwrap();
    assert!(
        s.contains("\"first\nsecond\""),
        "expected an RFC 4180 quoted cell: {s:?}"
    );
}

#[test]
fn tsv_format_replaces_newlines_with_spaces() {
    let cfg = fmt_cfg(OutputFormat::Tsv);
    let value = serde_json::json!({"id": "1", "text": "first\nsecond"});
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).unwrap();
    assert!(
        s.contains("first second"),
        "expected the newline replaced by a space: {s:?}"
    );
    assert_eq!(
        s.lines().count(),
        2,
        "TSV has no quoting rule, so a row must stay one line: {s:?}"
    );
}

#[test]
fn tsv_format_uses_tab_delimiter() {
    let cfg = fmt_cfg(OutputFormat::Tsv);
    let value = serde_json::json!({"id": "1", "text": "hi"});
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains('\t'), "expected tab in TSV output: {s:?}");
}

#[test]
fn csv_format_stringifies_nested_values_with_warning_column() {
    let cfg = fmt_cfg(OutputFormat::Csv);
    let value = serde_json::json!({"id": "1", "nested": {"a": 1}});
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_response(&mut buf, &value);
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("_warning"), "expected _warning column: {s}");
    assert!(s.contains("JSON-stringified"));
}

#[test]
fn warning_suppressed_under_yaml_and_csv() {
    for fmt in [
        OutputFormat::Yaml,
        OutputFormat::Csv,
        OutputFormat::Tsv,
        OutputFormat::Ndjson,
    ] {
        let cfg = fmt_cfg(fmt.clone());
        let mut buf: Vec<u8> = Vec::new();
        cfg.warning(&mut buf, "rate limited");
        assert!(
            buf.is_empty(),
            "warnings on stderr must be suppressed under {fmt:?}: {buf:?}"
        );
    }
}

#[test]
fn print_error_under_yaml_emits_yaml_envelope() {
    let cfg = fmt_cfg(OutputFormat::Yaml);
    let mut buf: Vec<u8> = Vec::new();
    cfg.print_error_envelope(&mut buf, "no-tty", 1, "stdin is not a terminal");
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("status: error"), "got: {s}");
    assert!(s.contains("reason: no-tty"), "got: {s}");
    assert!(s.contains("exit_code: 1"), "got: {s}");
}

#[test]
fn info_suppressed_under_every_structured_format() {
    for fmt in [
        OutputFormat::Json,
        OutputFormat::Jsonl,
        OutputFormat::Ndjson,
        OutputFormat::Yaml,
        OutputFormat::Csv,
        OutputFormat::Tsv,
    ] {
        let cfg = fmt_cfg(fmt.clone());
        let mut buf: Vec<u8> = Vec::new();
        cfg.info(&mut buf, "hi");
        assert!(buf.is_empty(), "info() must be silent under {fmt:?}");
    }
}

// ── NO_COLOR edge proof ─────────────────────────────────────────────────────

// ALLOWLISTED ENV MUTATION (see tests/env_mutation_guard.rs).
//
// The binary reads `NO_COLOR` once, in `xurl::cli::env::from_process`, and
// hands the flag to `new_with_no_color`, which every other color test drives
// directly. This is the only test covering that read, so it exports the
// variable. It is the sole mutation in this binary, so nothing races it.
#[test]
fn no_color_env_reaches_the_resolved_color_decision() {
    let prior = std::env::var_os("NO_COLOR");
    unsafe {
        std::env::set_var("NO_COLOR", "1");
    }
    let overrides = xurl::cli::env::from_process();
    let cfg = OutputConfig::new_with_no_color(
        OutputFormat::Text,
        false,
        false,
        xurl::cli::ColorChoice::Always,
        false,
        overrides.no_color,
    );
    unsafe {
        match prior {
            Some(v) => std::env::set_var("NO_COLOR", v),
            None => std::env::remove_var("NO_COLOR"),
        }
    }

    assert!(
        !cfg.use_color,
        "an exported NO_COLOR must reach the constructor and defeat --color always"
    );
}

// ── Typed-envelope round trip ──────────────────────────────────────────────

/// Deserializes one emitted envelope back into [`xurl::cli::envelope::ErrorBody`],
/// which denies unknown fields, so a key no field declares fails the test.
fn assert_round_trips(emitted: &str, expected_reason: &str) {
    let mut value: serde_json::Value = serde_json::from_str(emitted.trim())
        .unwrap_or_else(|e| panic!("envelope must parse ({e}): {emitted}"));
    let obj = value.as_object_mut().expect("envelope is an object");
    assert_eq!(
        obj.remove("status"),
        Some(serde_json::Value::String("error".to_string())),
        "every error envelope carries status=error: {emitted}"
    );
    let body: xurl::cli::envelope::ErrorBody = serde_json::from_value(value)
        .unwrap_or_else(|e| panic!("undeclared key in the envelope ({e}): {emitted}"));
    assert_eq!(body.reason, expected_reason);
}

fn json_config() -> OutputConfig {
    OutputConfig::new(
        OutputFormat::Json,
        false,
        false,
        xurl::cli::ColorChoice::Never,
    )
}

#[test]
fn every_emitted_error_envelope_round_trips_with_unknown_fields_denied() {
    // 1. A plain error through `print_error`.
    let mut buf: Vec<u8> = Vec::new();
    json_config().print_error(&mut buf, &Error::auth("NoAuthMethod: none"), 77);
    assert_round_trips(&String::from_utf8_lossy(&buf), "auth-required");

    // 2. The auth-method-mismatch shape, with its eight extra fields.
    let mut buf: Vec<u8> = Vec::new();
    let mismatch = Error::from(xdk::error::AuthMismatch {
        endpoint: "/2/users/{id}/likes".to_string(),
        rendered_url: Some("/2/users/12345/likes".to_string()),
        method: "POST".to_string(),
        requested: None,
        supported: vec!["oauth1".to_string()],
        available_in_app: Some(vec!["app".to_string()]),
        app: Some("default".to_string()),
        other_apps_with_creds: Some(vec!["work".to_string()]),
    });
    json_config().print_error(&mut buf, &mismatch, 2);
    assert_round_trips(&String::from_utf8_lossy(&buf), "auth-method-mismatch");

    // 3. A reason-and-message envelope through `print_error_envelope`.
    let mut buf: Vec<u8> = Vec::new();
    json_config().print_error_envelope(&mut buf, "no-tty", 1, "stdin is not a terminal");
    assert_round_trips(&String::from_utf8_lossy(&buf), "no-tty");

    // 4. Confirmation-required, whose verb context is declared too.
    let mut buf: Vec<u8> = Vec::new();
    let ctx = serde_json::json!({
        "command": "auth-clear",
        "all": true,
        "oauth1": false,
        "oauth2_username": serde_json::Value::Null,
        "bearer": false,
    });
    json_config().print_confirmation_required(&mut buf, &ctx, 1);
    assert_round_trips(&String::from_utf8_lossy(&buf), "confirmation-required");

    // 5. A hint-bearing envelope: `next_step` must round-trip too. The CLI
    // builds this shape for the sign-in refusal.
    let mut body = xurl::cli::envelope::ErrorBody::default();
    body.reason = "client-credentials-missing".to_string();
    body.exit_code = 2;
    body.message = Some("no app carries client credentials.".to_string());
    body.app = Some("blank".to_string());
    body.next_step = Some(xurl::cli::hints::NextStep::select_app(
        "xr auth oauth2 --app work".to_string(),
    ));
    let emitted = body.into_value().to_string();
    assert_round_trips(&emitted, "client-credentials-missing");
    assert!(
        emitted.contains("\"action\":\"select-app\""),
        "the hint survives the round trip: {emitted}"
    );

    // 6. The destructive-post context shape.
    let mut buf: Vec<u8> = Vec::new();
    let ctx = serde_json::json!({"command": "delete", "post_id": "12345"});
    json_config().print_confirmation_required(&mut buf, &ctx, 1);
    assert_round_trips(&String::from_utf8_lossy(&buf), "confirmation-required");
}

#[test]
fn an_undeclared_key_fails_the_round_trip() {
    // The guard itself must bite: a key no field declares is rejected.
    let emitted = r#"{"status":"error","reason":"auth-required","exit_code":77,"surprise":1}"#;
    let mut value: serde_json::Value = serde_json::from_str(emitted).unwrap();
    value.as_object_mut().unwrap().remove("status");
    assert!(
        serde_json::from_value::<xurl::cli::envelope::ErrorBody>(value).is_err(),
        "deny_unknown_fields must reject an undeclared key"
    );
}
