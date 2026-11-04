//! Under a structured format a library warning rides inside the one document
//! a run writes on stderr: the error envelope when the run fails, and a
//! status-ok document of its own when it succeeds. Stdout stays what the
//! command prints with no warning at all.
//!
//! A text line from `Diagnostics` goes to the process's own stderr, which the
//! runner's injected writer never sees, so a buffer captured in process parses
//! whatever `Diagnostics` wrote. The tests that read stderr as a caller does
//! spawn the built binary, with the variable set on the child alone. The
//! in-process tests pin what reaches the injected writer.

mod common;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::config::EnvOverrides;
use xdk::store::TokenStore;

/// What `REDIRECT_URI` holds to fire the warning: no scheme, so no URL.
const NOT_A_URL: &str = "notaurl";
const WARNING: &str =
    "REDIRECT_URI env value rejected by validation; falling through to next precedence level";
const WARNING_LINE: &str = "warning: REDIRECT_URI env value rejected by validation; falling through to next precedence level";
const CLOSED_PORT: &str = "http://127.0.0.1:1";
const BEARER: &str = "bearer-token";
const SEARCH_PATH: &str = "/2/tweets/search/recent";

fn the_warning() -> Value {
    json!([{"message": WARNING, "target": "xdk::config"}])
}

/// Parses all of `text` as one JSON document, the way `jaq` or `json.loads`
/// reads a captured stream.
fn one_document(stream: &str, text: &str) -> Value {
    serde_json::from_str(text)
        .unwrap_or_else(|e| panic!("{stream} is not one JSON document ({e}): {text:?}"))
}

/// `xr <args>` in process, with the warning's variable injected.
async fn in_process(
    store: &Path,
    overrides: &EnvOverrides,
    args: &[&str],
) -> (i32, String, String) {
    let mut argv = vec!["xr"];
    argv.extend_from_slice(args);
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code =
        xurl::cli::runner::run_with_overrides(argv, &mut stdout, &mut stderr, store, overrides)
            .await;
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

fn warned(base_url: &str) -> EnvOverrides {
    EnvOverrides {
        redirect_uri: Some(NOT_A_URL.to_string()),
        api_base_url: Some(base_url.to_string()),
        ..EnvOverrides::default()
    }
}

fn warned_with_bearer(base_url: &str) -> EnvOverrides {
    EnvOverrides {
        bearer_token: Some(BEARER.to_string()),
        ..warned(base_url)
    }
}

/// The built `xr` with the warning's variable set on the child, off the
/// runtime thread so a mock keeps serving.
async fn spawned(
    store: PathBuf,
    base_url: &str,
    bearer: bool,
    args: &[&str],
) -> (i32, String, String) {
    let base_url = base_url.to_string();
    let args: Vec<String> = args.iter().map(|a| (*a).to_string()).collect();
    let output = tokio::task::spawn_blocking(move || {
        let mut cmd = common::xr_with_store(&store);
        cmd.env("REDIRECT_URI", NOT_A_URL)
            .env("API_BASE_URL", &base_url);
        if bearer {
            cmd.env("XURL_BEARER_TOKEN", BEARER);
        }
        cmd.args(&args).output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

async fn search_server(body: Value) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(SEARCH_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    server
}

fn found_posts() -> Value {
    json!({"data": [{"id": "1", "text": "hi"}]})
}

#[tokio::test]
async fn a_failed_run_carries_the_warning_in_its_envelope() {
    let tmp = TempDir::new().expect("tempdir");
    let (code, stdout, stderr) = in_process(
        &tmp.path().join(".xurl"),
        &warned(CLOSED_PORT),
        &["--output", "json", "whoami"],
    )
    .await;

    assert_eq!(code, 77, "stderr: {stderr}");
    assert_eq!(stdout, "");
    let envelope = one_document("stderr", &stderr);
    assert_eq!(envelope["status"], "error");
    assert_eq!(envelope["reason"], "auth-required");
    assert_eq!(envelope["warnings"], the_warning());
    assert!(
        envelope["next_step"].is_object(),
        "the rest of the envelope stands: {envelope}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_failed_run_writes_one_document_on_stderr_in_every_json_shaped_format() {
    for format in ["json", "jsonl", "ndjson", "csv", "tsv"] {
        let tmp = TempDir::new().expect("tempdir");
        let (code, _, stderr) = spawned(
            tmp.path().join(".xurl"),
            CLOSED_PORT,
            false,
            &["--output", format, "whoami"],
        )
        .await;

        assert_eq!(code, 77, "{format}: {stderr}");
        let envelope = one_document("stderr", &stderr);
        assert_eq!(envelope["reason"], "auth-required", "{format}");
        assert_eq!(envelope["warnings"], the_warning(), "{format}");
        if format != "json" {
            assert_eq!(stderr.lines().count(), 1, "{format} is one line: {stderr}");
        }
    }
}

/// A `warning:` line is itself a YAML mapping key, so a YAML reader would not
/// catch it as a parse failure: it would hand back a key no envelope declares.
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_run_under_yaml_writes_no_warning_line() {
    let tmp = TempDir::new().expect("tempdir");
    let (code, _, stderr) = spawned(
        tmp.path().join(".xurl"),
        CLOSED_PORT,
        false,
        &["--output", "yaml", "whoami"],
    )
    .await;

    assert_eq!(code, 77, "stderr: {stderr}");
    assert!(
        !stderr.lines().any(|line| line.starts_with("warning:")),
        "no text line ahead of the document: {stderr}"
    );
    assert!(
        stderr.lines().any(|line| line.starts_with("warnings:")) && stderr.contains(WARNING),
        "the warning rides in the document: {stderr}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_run_that_warns_and_succeeds_leaves_stdout_as_it_was() {
    let server = search_server(found_posts()).await;
    let tmp = TempDir::new().expect("tempdir");
    let (code, stdout, stderr) = in_process(
        &tmp.path().join(".xurl"),
        &warned_with_bearer(&server.uri()),
        &["--output", "json", "search", "hi"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let response = one_document("stdout", &stdout);
    assert!(response.get("warnings").is_none(), "stdout: {stdout}");
    assert_eq!(
        one_document("stderr", &stderr),
        json!({"status": "ok", "warnings": the_warning()})
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_run_that_warns_and_succeeds_keeps_each_stream_parseable_as_a_caller_reads_it() {
    let server = search_server(found_posts()).await;
    for format in ["json", "jsonl"] {
        let tmp = TempDir::new().expect("tempdir");
        let (code, stdout, stderr) = spawned(
            tmp.path().join(".xurl"),
            &server.uri(),
            true,
            &["--output", format, "search", "hi"],
        )
        .await;

        assert_eq!(code, 0, "{format}: {stderr}");
        let response = one_document("stdout", &stdout);
        assert!(response.get("warnings").is_none(), "{format}: {stdout}");
        assert_eq!(
            one_document("stderr", &stderr),
            json!({"status": "ok", "warnings": the_warning()}),
            "{format}"
        );
        if format == "jsonl" {
            assert_eq!(stderr.lines().count(), 1, "one line: {stderr}");
        }
    }
}

/// A flattened row has no room for a nested key: a `warnings` column would
/// appear only on the runs that warned.
#[tokio::test(flavor = "multi_thread")]
async fn a_delimited_response_gains_no_warnings_column() {
    let server = search_server(found_posts()).await;
    for format in ["csv", "tsv"] {
        let tmp = TempDir::new().expect("tempdir");
        let (code, stdout, stderr) = spawned(
            tmp.path().join(".xurl"),
            &server.uri(),
            true,
            &["--output", format, "search", "hi"],
        )
        .await;

        assert_eq!(code, 0, "{format}: {stderr}");
        assert!(!stdout.contains("warnings"), "{format}: {stdout}");
        assert_eq!(
            one_document("stderr", &stderr),
            json!({"status": "ok", "warnings": the_warning()}),
            "{format}"
        );
    }
}

/// Raw mode prints what X sent, and that need not be an object a key could
/// go in.
#[tokio::test(flavor = "multi_thread")]
async fn a_raw_response_stays_the_body_x_sent() {
    let body = json!(["a", "b"]);
    let server = search_server(body.clone()).await;
    let tmp = TempDir::new().expect("tempdir");
    let url = format!("{}{SEARCH_PATH}", server.uri());
    let (code, stdout, stderr) = in_process(
        &tmp.path().join(".xurl"),
        &warned_with_bearer(&server.uri()),
        &["--output", "json", &url],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(one_document("stdout", &stdout), body);
    assert_eq!(
        one_document("stderr", &stderr),
        json!({"status": "ok", "warnings": the_warning()})
    );
}

#[tokio::test]
async fn a_dry_run_keeps_its_envelope_on_stdout_and_its_warning_on_stderr() {
    for (overrides, would_succeed) in [
        (warned_with_bearer(CLOSED_PORT), true),
        (warned(CLOSED_PORT), false),
    ] {
        let tmp = TempDir::new().expect("tempdir");
        let (code, stdout, stderr) = in_process(
            &tmp.path().join(".xurl"),
            &overrides,
            &["--output", "json", "--dry-run", "search", "hi"],
        )
        .await;

        assert_eq!(code, 0, "stderr: {stderr}");
        let dry_run = one_document("stdout", &stdout);
        assert_eq!(dry_run["status"], "dry_run");
        assert_eq!(dry_run["would_succeed"], would_succeed, "stdout: {stdout}");
        assert!(dry_run.get("warnings").is_none(), "stdout: {stdout}");
        assert_eq!(
            one_document("stderr", &stderr),
            json!({"status": "ok", "warnings": the_warning()})
        );
    }
}

/// `--quiet` drops chatter. A warning is not chatter, in either rendering.
#[tokio::test]
async fn quiet_leaves_the_warning_in_the_document() {
    let tmp = TempDir::new().expect("tempdir");
    let (_, _, stderr) = in_process(
        &tmp.path().join(".xurl"),
        &warned(CLOSED_PORT),
        &["--output", "json", "--quiet", "whoami"],
    )
    .await;
    assert_eq!(one_document("stderr", &stderr)["warnings"], the_warning());
}

/// `auth status` resolves the redirect URI once loading the store and once
/// per app it reports, so the same warning fires more than once in one run.
#[tokio::test(flavor = "multi_thread")]
async fn a_warning_that_fires_twice_appears_once() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("work", "CLIENT-ID", "SECRET").expect("add_app");

    let (code, stdout, stderr) = spawned(
        store,
        CLOSED_PORT,
        false,
        &["--output", "json", "auth", "status"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(one_document("stdout", &stdout)["status"], "ok");
    assert_eq!(
        one_document("stderr", &stderr),
        json!({"status": "ok", "warnings": the_warning()})
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn text_output_keeps_its_warning_line() {
    for flags in [&[][..], &["--quiet"]] {
        let tmp = TempDir::new().expect("tempdir");
        let mut args = flags.to_vec();
        args.push("whoami");
        let (code, _, stderr) = spawned(tmp.path().join(".xurl"), CLOSED_PORT, false, &args).await;

        assert_eq!(code, 77, "{flags:?}: {stderr}");
        assert_eq!(stderr.lines().next(), Some(WARNING_LINE), "{flags:?}");
        assert!(!stderr.contains("warnings"), "{flags:?}: {stderr}");
    }
}

/// An undeclared key would make the schema wrong about what callers receive,
/// and the error variant closes its object.
#[tokio::test]
async fn the_envelope_schema_declares_warnings_on_both_variants_that_carry_them() {
    let schema = xurl::cli::envelope::envelope_schema();
    let variants = schema["oneOf"].as_array().expect("oneOf");
    let variant = |status: &str| {
        variants
            .iter()
            .find(|variant| variant["properties"]["status"]["const"] == status)
            .unwrap_or_else(|| panic!("no {status} variant: {schema}"))
    };
    let items = |status: &str| &variant(status)["properties"]["warnings"]["items"];
    assert!(items("error").is_object(), "{}", variant("error"));
    assert_eq!(items("error"), items("ok"));
    let warning = &schema["$defs"]["Warning"];
    assert_eq!(
        warning["required"],
        json!(["message", "target"]),
        "{warning}"
    );
    assert_eq!(warning["additionalProperties"], false, "{warning}");
}

/// Each document a warned run writes passes the validator `xr validate
/// --schema envelope` runs.
#[tokio::test]
async fn the_error_document_passes_the_envelope_validator() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let (_, _, stderr) = in_process(
        &store,
        &warned(CLOSED_PORT),
        &["--output", "json", "whoami"],
    )
    .await;
    assert!(
        one_document("stderr", &stderr)["warnings"].is_array(),
        "stderr: {stderr}"
    );
    let document = tmp.path().join("envelope.json");
    std::fs::write(&document, &stderr).expect("write the envelope");

    let (code, stdout, refusal) = in_process(
        &store,
        &EnvOverrides::default(),
        &[
            "--output",
            "json",
            "validate",
            document.to_str().expect("utf-8 path"),
            "--schema",
            "envelope",
        ],
    )
    .await;
    assert_eq!(code, 0, "stdout: {stdout}\nstderr: {refusal}");
}
