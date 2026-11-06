//! `xr broadcasts sources` reaches the stream-source endpoints and the
//! ingest region through the runner: each verb sends the request the spec
//! gives its endpoint and prints the typed response, a `remove` asks before
//! it deletes, and an empty name is refused before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth1_store, run_in_process as run};

const SOURCE_ID: &str = "9012345678901234567";
const STREAM_KEY: &str = "example-stream-key-4f2a";

/// A source as X describes one to a list, a read, or a rename: it carries no
/// stream key.
fn source(name: &str) -> serde_json::Value {
    serde_json::json!({
        "id": SOURCE_ID,
        "name": name,
        "region": "pdxa",
        "rtmps_url": "rtmps://va.pscp.tv:443/x",
        "is_stream_active": false,
        "created_at_ms": "1761148800000"
    })
}

/// A source as X answers its creation, the one answer that carries the key.
fn created_source(name: &str) -> serde_json::Value {
    let mut created = source(name);
    created["stream_key"] = serde_json::json!(STREAM_KEY);
    created
}

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

#[tokio::test]
async fn sources_list_prints_the_sources_as_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/sources"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"data": [source("Studio A")], "meta": {"result_count": 1}}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["broadcasts", "sources", "list", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"][0]["name"], "Studio A");
    assert!(body["data"][0].get("stream_key").is_none());
    assert_eq!(body["meta"]["result_count"], 1);
}

#[tokio::test]
async fn sources_region_prints_the_nearest_ingest_region() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/region"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"region": "us-west-2"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["broadcasts", "sources", "region", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["region"], "us-west-2");
}

#[tokio::test]
async fn sources_add_posts_the_name_and_region() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/sources"))
        .and(body_json(
            serde_json::json!({"name": "Studio A", "region": "eu-west-1"}),
        ))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({"data": created_source("Studio A")})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "sources",
            "add",
            "Studio A",
            "--region",
            "eu-west-1",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"]["id"], SOURCE_ID);
    assert_eq!(body["data"]["stream_key"], STREAM_KEY);
}

#[tokio::test]
async fn sources_get_reads_one_source() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/sources/{SOURCE_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": source("Studio A")})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "sources",
            "get",
            SOURCE_ID,
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["is_stream_active"], false);
}

fn stream_key_answer() -> serde_json::Value {
    serde_json::json!({"data": {"id": SOURCE_ID, "stream_key": STREAM_KEY}})
}

#[tokio::test]
async fn sources_stream_key_reads_the_key_of_one_source() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/sources/{SOURCE_ID}/stream_key")))
        .respond_with(ResponseTemplate::new(200).set_body_json(stream_key_answer()))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "sources",
            "stream-key",
            SOURCE_ID,
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"]["id"], SOURCE_ID);
    assert_eq!(body["data"]["stream_key"], STREAM_KEY);
    assert_eq!(
        stdout.matches(STREAM_KEY).count(),
        1,
        "the key appears once, in data.stream_key: {stdout}"
    );
}

/// `--verbose` prints through the binary's own subscriber, so this runs the
/// built `xr`, off the runtime thread so the mock keeps serving. Under a
/// structured format `--verbose` prints nothing, so the run uses text.
#[tokio::test]
async fn sources_stream_key_keeps_the_key_out_of_verbose_text_output() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/sources/{SOURCE_ID}/stream_key")))
        .respond_with(ResponseTemplate::new(200).set_body_json(stream_key_answer()))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let base_url = server.uri();

    let output = tokio::task::spawn_blocking(move || {
        common::xr_with_store(&store)
            .env("API_BASE_URL", base_url)
            .args([
                "--verbose",
                "broadcasts",
                "sources",
                "stream-key",
                SOURCE_ID,
            ])
            .output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(0), "stderr: {stderr}");
    assert!(stdout.contains(STREAM_KEY), "stdout: {stdout}");
    assert!(
        stderr.contains("/stream_key"),
        "--verbose prints the request lines in text output: {stderr}"
    );
    assert!(!stderr.contains(STREAM_KEY), "stderr: {stderr}");
}

#[tokio::test]
async fn sources_stream_key_stores_nothing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/sources/{SOURCE_ID}/stream_key")))
        .respond_with(ResponseTemplate::new(200).set_body_json(stream_key_answer()))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let before = std::fs::read(&store).expect("the seeded store");

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "sources",
            "stream-key",
            SOURCE_ID,
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let after = std::fs::read(&store).expect("the store");
    assert_eq!(after, before, "the store is as the command found it");
    assert!(
        !String::from_utf8_lossy(&after).contains(STREAM_KEY),
        "the key is not written to the store"
    );
}

#[tokio::test]
async fn sources_stream_key_under_dry_run_sends_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "broadcasts",
            "sources",
            "stream-key",
            SOURCE_ID,
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "broadcasts-sources-stream-key");
    assert_eq!(body["source_id"], SOURCE_ID);
    assert_eq!(body["would_succeed"], true);
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

/// The help of a stream-source verb, as clap holds it.
fn sources_verb_help(verb: &str) -> String {
    use clap::CommandFactory;

    let mut root = xurl::cli::Cli::command();
    root.build();
    let command = ["broadcasts", "sources", verb]
        .iter()
        .fold(&root, |cmd, name| {
            cmd.find_subcommand(name)
                .unwrap_or_else(|| panic!("clap has `{name}`"))
        });
    command
        .get_long_about()
        .or_else(|| command.get_about())
        .map(ToString::to_string)
        .expect("the verb has help")
}

#[test]
fn sources_stream_key_help_calls_the_output_a_credential() {
    let help = sources_verb_help("stream-key");
    assert!(
        help.contains("credential"),
        "the help warns what the key is: {help}"
    );
}

#[test]
fn sources_add_help_names_the_command_that_reads_the_key_again() {
    let help = sources_verb_help("add");
    assert!(
        help.contains("xr broadcasts sources stream-key"),
        "the help names the way to read the key later: {help}"
    );
}

#[tokio::test]
async fn sources_update_puts_the_new_name() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/sources/{SOURCE_ID}")))
        .and(body_json(serde_json::json!({"name": "Studio B"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": source("Studio B")})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "sources",
            "update",
            SOURCE_ID,
            "--name",
            "Studio B",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["name"], "Studio B");
}

/// The label is a flag, as `lists update --name` takes it, and `update`
/// sends nothing without one.
#[tokio::test]
async fn sources_update_needs_the_name_flag() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    for args in [
        &["broadcasts", "sources", "update", SOURCE_ID][..],
        &["broadcasts", "sources", "update", SOURCE_ID, "Studio B"][..],
    ] {
        let mut argv = vec!["--output", "json"];
        argv.extend(args);
        let (code, _stdout, stderr) = run(&store, &server.uri(), &argv).await;
        assert_eq!(code, 2, "{args:?}: {stderr}");
        let envelope: serde_json::Value =
            serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
        assert_eq!(envelope["reason"], "invalid-args", "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn sources_remove_without_force_asks_first_and_deletes_nothing() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--no-interactive",
            "--output",
            "json",
            "broadcasts",
            "sources",
            "remove",
            SOURCE_ID,
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "confirmation-required");
    assert_eq!(envelope["next_step"]["action"], "confirm");
    assert!(
        envelope["next_step"]["template"]
            .as_str()
            .is_some_and(|t| t.contains("--force")),
        "{envelope}"
    );
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn sources_remove_with_force_deletes_the_source() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/sources/{SOURCE_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "sources",
            "remove",
            SOURCE_ID,
            "--force",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["deleted"], true);
}

#[tokio::test]
async fn sources_remove_under_dry_run_says_the_real_run_would_ask() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "broadcasts",
            "sources",
            "remove",
            SOURCE_ID,
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "broadcasts-sources-remove");
    assert_eq!(body["source_id"], SOURCE_ID);
    assert_eq!(body["confirmation_required"], true);
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn sources_add_refuses_an_empty_name_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "broadcasts",
            "sources",
            "add",
            " ",
            "--region",
            "us-east-1",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["would_succeed"], false);
    assert_eq!(body["rule"], "empty-source-name");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

/// The region names in `text`, such as `us-east-1` or `ap-northeast-2`.
fn region_names(text: &str) -> std::collections::BTreeSet<String> {
    regex::Regex::new(r"\b[a-z]{2}-[a-z]+-[0-9]\b")
        .expect("a pattern")
        .find_iter(text)
        .map(|found| found.as_str().to_string())
        .collect()
}

#[test]
fn sources_add_region_help_names_the_regions_the_spec_accepts() {
    use clap::CommandFactory;

    let path = common::workspace_root().join("crates/xdk/vendor/x-api-openapi.json");
    let spec: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("the vendored spec"))
            .expect("the vendored spec is JSON");
    let accepted = region_names(
        spec.pointer("/components/schemas/CreateSourceRequest/properties/region/description")
            .and_then(serde_json::Value::as_str)
            .expect("the spec describes the region a new source takes"),
    );

    let mut root = xurl::cli::Cli::command();
    root.build();
    let add = ["broadcasts", "sources", "add"]
        .iter()
        .fold(&root, |cmd, name| {
            cmd.find_subcommand(name).expect("clap has the command")
        });
    let help = add
        .get_arguments()
        .find(|arg| arg.get_id() == "region")
        .and_then(|arg| arg.get_help())
        .map(ToString::to_string)
        .expect("--region has help");

    assert_eq!(
        region_names(&help),
        accepted,
        "`xr broadcasts sources add --region` help and the spec's CreateSourceRequest.region \
         description name different regions; edit the help in crates/xurl-cli/src/cli/mod.rs"
    );
}
