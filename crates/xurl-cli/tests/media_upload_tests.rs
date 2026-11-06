//! `xr media upload` and `xr media status` against a mock X: what a wait on
//! processing ends on, and which stream each outcome reaches. Stdout carries
//! the command's result, so a run that fails leaves it empty and names the
//! media id on stderr.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::oauth2_store;

// ── A wait on media processing always ends ─────────────────────────────

/// The STATUS endpoint for media `777`, answering `data`.
fn status_mock(data: serde_json::Value) -> Mock {
    Mock::given(method("GET"))
        .and(path("/2/media/upload"))
        .and(query_param("command", "STATUS"))
        .and(query_param("media_id", "777"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": data })))
}

/// Runs `xr <args>` against `server` off the runtime thread, killed after ten
/// seconds so a wait that never ends fails the test instead of hanging it.
async fn run_xr(
    server: &MockServer,
    store: std::path::PathBuf,
    args: &[&str],
) -> std::process::Output {
    let base_url = server.uri();
    let args: Vec<String> = args.iter().map(|a| (*a).to_string()).collect();
    tokio::task::spawn_blocking(move || {
        let mut cmd = common::xr_with_store(&store);
        cmd.env("API_BASE_URL", &base_url)
            .args(&args)
            .timeout(std::time::Duration::from_secs(10))
            .output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs")
}

async fn status_calls(server: &MockServer) -> usize {
    server
        .received_requests()
        .await
        .expect("request recording is on")
        .iter()
        .filter(|request| {
            request.method == wiremock::http::Method::GET && request.url.path() == "/2/media/upload"
        })
        .count()
}

/// An image, or any media X has finished with, reports a status with no
/// `processing_info`: there is nothing left to wait for.
#[tokio::test(flavor = "multi_thread")]
async fn a_status_without_processing_info_ends_the_wait_after_one_call() {
    let server = MockServer::start().await;
    status_mock(serde_json::json!({"id": "777", "media_key": "3_777"}))
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");

    let output = run_xr(
        &server,
        oauth2_store(tmp.path()),
        &["--output", "json", "media", "status", "777", "--wait"],
    )
    .await;

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(status_calls(&server).await, 1);
}

/// A job still in progress when the deadline passes ends with its own
/// `reason`, the media id, and the command that resumes the wait for twice
/// as long.
#[tokio::test(flavor = "multi_thread")]
async fn a_wait_past_its_deadline_ends_with_processing_timeout_and_a_resume_command() {
    let server = MockServer::start().await;
    status_mock(serde_json::json!({
        "id": "777",
        "processing_info": {"state": "in_progress", "check_after_secs": 1, "progress_percent": 10}
    }))
    .mount(&server)
    .await;
    let tmp = TempDir::new().expect("tempdir");

    let output = run_xr(
        &server,
        oauth2_store(tmp.path()),
        &["--output", "json", "media", "status", "777", "--wait=2"],
    )
    .await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "processing-timeout", "got: {v}");
    assert_eq!(v["exit_code"], 1, "got: {v}");
    assert_eq!(v["media_id"], "777", "got: {v}");
    assert_eq!(v["next_step"]["action"], "resume-wait", "got: {v}");
    assert_eq!(
        v["next_step"]["command"], "xr media status 777 --wait=4",
        "got: {v}"
    );
    let calls = status_calls(&server).await;
    assert!(
        (1..=3).contains(&calls),
        "a two-second deadline at one check a second allows at most three status calls, saw {calls}"
    );
}

/// The same timeout in text mode names the resume command a person can run.
#[tokio::test(flavor = "multi_thread")]
async fn a_processing_timeout_in_text_mode_prints_the_resume_command() {
    let server = MockServer::start().await;
    status_mock(serde_json::json!({
        "id": "777",
        "processing_info": {"state": "in_progress", "check_after_secs": 1}
    }))
    .mount(&server)
    .await;
    let tmp = TempDir::new().expect("tempdir");

    let output = run_xr(
        &server,
        oauth2_store(tmp.path()),
        &["media", "status", "777", "--wait=1"],
    )
    .await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(
        stderr.contains("xr media status 777 --wait=2"),
        "the resume command is printed: {stderr}"
    );
}

/// A failed job keeps its own error and is not reported as a timeout.
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_status_is_not_a_timeout() {
    let server = MockServer::start().await;
    status_mock(serde_json::json!({"id": "777", "processing_info": {"state": "failed"}}))
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");

    let output = run_xr(
        &server,
        oauth2_store(tmp.path()),
        &["--output", "json", "media", "status", "777", "--wait=5"],
    )
    .await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "invalid-request", "got: {v}");
    assert_eq!(v["media_id"], "777", "got: {v}");
    assert_eq!(v["next_step"]["action"], "fix-input", "got: {v}");
    assert!(
        v["message"]
            .as_str()
            .is_some_and(|m| m.contains("failed processing")),
        "got: {v}"
    );
    assert_eq!(status_calls(&server).await, 1);
}

/// Bare `--wait` and `--wait=true` both wait, here until the job succeeds on
/// its second status.
#[rstest::rstest]
#[case::bare("--wait")]
#[case::explicit_true("--wait=true")]
#[tokio::test(flavor = "multi_thread")]
async fn a_bare_or_true_wait_polls_until_the_job_succeeds(#[case] wait: &str) {
    let server = MockServer::start().await;
    status_mock(serde_json::json!({
        "id": "777",
        "processing_info": {"state": "in_progress", "check_after_secs": 1}
    }))
    .up_to_n_times(1)
    .mount(&server)
    .await;
    status_mock(serde_json::json!({"id": "777", "processing_info": {"state": "succeeded"}}))
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");

    let output = run_xr(
        &server,
        oauth2_store(tmp.path()),
        &["--output", "json", "media", "status", "777", wait],
    )
    .await;

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(status_calls(&server).await, 2);
}

/// `--wait=false` and `--wait=0` return after FINALIZE without a status call,
/// even for a video X is still processing.
#[rstest::rstest]
#[case::explicit_false("--wait=false")]
#[case::zero("--wait=0")]
#[tokio::test(flavor = "multi_thread")]
async fn an_upload_told_not_to_wait_returns_after_finalize(#[case] wait: &str) {
    let server = MockServer::start().await;
    let created = serde_json::json!({
        "data": {"id": "777", "media_key": "7_777", "expires_after_secs": 86400}
    });
    Mock::given(method("POST"))
        .and(path("/2/media/upload/initialize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&created))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/media/upload/777/append"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/media/upload/777/finalize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "777", "processing_info": {"state": "pending", "check_after_secs": 1}}
        })))
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let video = tmp.path().join("clip.mp4");
    std::fs::write(&video, b"not really a video").expect("write clip");

    let output = run_xr(
        &server,
        oauth2_store(tmp.path()),
        &[
            "--output",
            "json",
            "media",
            "upload",
            video.to_str().expect("utf-8 path"),
            wait,
        ],
    )
    .await;

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(status_calls(&server).await, 0);
}

// ── A waited upload answers one document ───────────────────────────────

/// Mounts INIT, APPEND, and a FINALIZE answering `finalize` for media `777`.
async fn mount_upload(server: &MockServer, finalize: serde_json::Value) {
    Mock::given(method("POST"))
        .and(path("/2/media/upload/initialize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "777", "media_key": "7_777", "expires_after_secs": 86400}
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/media/upload/777/append"))
        .respond_with(ResponseTemplate::new(204))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/media/upload/777/finalize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": finalize })),
        )
        .mount(server)
        .await;
}

/// An upload whose FINALIZE reports processing under way, answered by
/// `status` when it waits, run with `args` after the file. Returns the output
/// and the test's scratch directory, which holds the store.
async fn upload_then_status(
    status: ResponseTemplate,
    args: &[&str],
) -> (std::process::Output, TempDir) {
    let server = MockServer::start().await;
    mount_upload(
        &server,
        serde_json::json!({
            "id": "777",
            "processing_info": {"state": "pending", "check_after_secs": 1}
        }),
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/2/media/upload"))
        .and(query_param("command", "STATUS"))
        .and(query_param("media_id", "777"))
        .respond_with(status)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let video = tmp.path().join("clip.mp4");
    std::fs::write(&video, b"not really a video").expect("write clip");
    let video = video.to_str().expect("utf-8 path").to_string();
    let mut full = vec!["media", "upload", video.as_str()];
    full.extend_from_slice(args);
    let output = run_xr(&server, oauth2_store(tmp.path()), &full).await;
    (output, tmp)
}

fn failed_status() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "data": {"id": "777", "processing_info": {"state": "failed"}}
    }))
}

/// A job X failed is a failed command, so FINALIZE's pending document stays
/// off stdout in every format, and stderr names the media.
#[rstest::rstest]
#[case::text(&[])]
#[case::json(&["--output", "json"])]
#[case::jsonl(&["--output", "jsonl"])]
#[case::yaml(&["--output", "yaml"])]
#[case::csv(&["--output", "csv"])]
#[case::tsv(&["--output", "tsv"])]
#[case::verbose_text(&["--verbose"])]
#[case::verbose_json(&["--verbose", "--output", "json"])]
#[tokio::test(flavor = "multi_thread")]
async fn a_job_x_failed_leaves_stdout_empty_in_every_format(#[case] args: &[&str]) {
    let (output, _tmp) = upload_then_status(failed_status(), args).await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert_eq!(stdout, "", "a failed command prints no result");
    assert!(stderr.contains("777"), "stderr names the media: {stderr}");
    assert!(
        stderr.contains("failed processing"),
        "the processing error is reported: {stderr}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_job_x_failed_carries_the_media_id_in_its_envelope() {
    let (output, _tmp) = upload_then_status(failed_status(), &["--output", "json"]).await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "invalid-request", "got: {v}");
    assert_eq!(v["media_id"], "777", "got: {v}");
    assert_eq!(v["next_step"]["action"], "fix-input", "got: {v}");
    let message = v["message"].as_str().expect("a message");
    assert!(!message.contains(AFTER_UPLOAD), "the read worked: {v}");
}

/// A wait that runs out is a failed command too, with the upload intact.
#[tokio::test(flavor = "multi_thread")]
async fn an_upload_wait_past_its_deadline_leaves_stdout_empty() {
    let still_running = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "data": {"id": "777", "processing_info": {"state": "in_progress", "check_after_secs": 1}}
    }));
    let (output, _tmp) = upload_then_status(still_running, &["--wait=1", "--output", "json"]).await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "processing-timeout", "got: {v}");
    assert_eq!(v["media_id"], "777", "got: {v}");
    assert_eq!(v["next_step"]["action"], "resume-wait", "got: {v}");
    assert_eq!(
        v["next_step"]["command"], "xr media status 777 --wait=2",
        "a wait whose deadline ran out resumes for twice as long: {v}"
    );
    assert_eq!(
        v["message"], "media 777 was still processing when the 1-second wait ended",
        "the read worked, so the message names the deadline: {v}"
    );
}

/// A status read that fails ends the wait with the upload complete and billed,
/// so the id reaches the caller, whose other copy was the document on stdout.
#[rstest::rstest]
#[case::json(&["--output", "json"])]
#[case::jsonl(&["--output", "jsonl"])]
#[tokio::test(flavor = "multi_thread")]
async fn a_status_read_that_fails_names_the_media_id_in_the_envelope(#[case] args: &[&str]) {
    let (output, _tmp) = upload_then_status(ResponseTemplate::new(500), args).await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(output.status.code(), Some(0), "stderr: {stderr}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "server-error", "got: {v}");
    assert_eq!(v["media_id"], "777", "got: {v}");
    let message = v["message"].as_str().expect("a message");
    assert!(message.starts_with(AFTER_UPLOAD), "says what failed: {v}");
}

/// What a status read's error opens with: the operation that failed and that
/// the upload finished, which X's answer alone does not say. An empty 5xx
/// otherwise reads `Error: {}`.
const AFTER_UPLOAD: &str = "media 777 finished uploading, but reading its status failed: ";

#[rstest::rstest]
#[case::plain(&[], true)]
#[case::quiet(&["--quiet"], false)]
#[tokio::test(flavor = "multi_thread")]
async fn a_status_read_that_fails_says_what_failed_in_text(
    #[case] args: &[&str],
    #[case] resume_line: bool,
) {
    let (output, _tmp) = upload_then_status(ResponseTemplate::new(503), args).await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(output.status.code(), Some(0), "stderr: {stderr}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert!(
        stderr.contains(&format!("Error: {AFTER_UPLOAD}")),
        "the error says the upload finished and names it: {stderr}"
    );
    assert_eq!(
        stderr.matches("Media ID").count(),
        0,
        "the error line already names the media: {stderr}"
    );
    assert_eq!(
        stderr.contains("Resume the wait. Run: xr media status 777 --wait=60"),
        resume_line,
        "the resume line prints unless --quiet: {stderr}"
    );
}

/// A refused credential on the status read fails the same way until it
/// changes, which no `xr` command does, so the step is `fix-input`. What to
/// run once the credential works is the resumed wait, as a template: an agent
/// that runs every `command` it is handed does not send a read X will refuse.
#[rstest::rstest]
#[case::default_credential(&[], "xr media status 777 --wait=60")]
#[case::own_login(&["--app", "myapp", "-u", "alice"], "xr --app myapp media status 777 --wait=60 -u alice")]
#[tokio::test(flavor = "multi_thread")]
async fn a_status_read_whose_credential_x_refused_resumes_once_it_works(
    #[case] flags: &[&str],
    #[case] template: &str,
) {
    let mut args = vec!["--output", "json"];
    args.extend_from_slice(flags);
    let (output, _tmp) = upload_then_status(ResponseTemplate::new(401), &args).await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(77), "stderr: {stderr}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "auth-required", "got: {v}");
    assert_eq!(v["media_id"], "777", "got: {v}");
    let message = v["message"].as_str().expect("a message");
    assert!(message.starts_with(AFTER_UPLOAD), "says what failed: {v}");
    assert_eq!(v["next_step"]["action"], "fix-input", "got: {v}");
    assert_eq!(v["next_step"]["template"], template, "got: {v}");
    assert_eq!(v["next_step"].get("command"), None, "never a command: {v}");
    assert_eq!(
        v["next_step"]["docs"], "https://docs.x.com/resources/fundamentals/authentication",
        "got: {v}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_refused_credential_after_an_upload_says_in_text_how_to_resume() {
    let (output, _tmp) = upload_then_status(ResponseTemplate::new(401), &[]).await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr
            .contains("once the credential works, resume the wait: xr media status 777 --wait=60"),
        "{stderr}"
    );
}

/// A status read that fails after the upload finished waits again rather than
/// inviting another upload, whose file X would bill for again. The new wait
/// has the deadline of the one that failed, which had not run out.
#[rstest::rstest]
#[case::default_wait(&[], "xr media status 777 --wait=60")]
#[case::own_wait(&["--wait=300"], "xr media status 777 --wait=300")]
#[tokio::test(flavor = "multi_thread")]
async fn a_status_read_x_failed_resumes_the_wait(#[case] wait: &[&str], #[case] command: &str) {
    let mut args = vec!["--output", "json"];
    args.extend_from_slice(wait);
    let (output, _tmp) = upload_then_status(ResponseTemplate::new(500), &args).await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "server-error", "got: {v}");
    assert_eq!(v["media_id"], "777", "got: {v}");
    assert_eq!(v["next_step"]["action"], "resume-wait", "got: {v}");
    assert_eq!(v["next_step"]["command"], command, "got: {v}");
    assert_eq!(
        v["next_step"]["docs"], "https://docs.x.com/x-api/fundamentals/response-codes-and-errors",
        "X's page on response codes stays with the step that replaced retry: {v}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_status_read_x_failed_prints_the_resume_command_in_text() {
    let (output, _tmp) = upload_then_status(ResponseTemplate::new(500), &[]).await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("xr media status 777 --wait=60"),
        "the resume command is printed: {stderr}"
    );
}

/// The command that resumes the wait reads under the credential the upload
/// used, so run verbatim it reads the same account's media. A header's value
/// can be a credential, so with `-H` the step is a template that hides it.
#[rstest::rstest]
#[case::own_app_and_login(
    &["--app", "myapp", "--auth", "oauth2", "-u", "alice"],
    "xr --app myapp media status 777 --wait=60 --auth oauth2 -u alice",
    None,
)]
#[case::joined_spellings(
    &["--app=myapp", "--username=alice"],
    "xr --app myapp media status 777 --wait=60 -u alice",
    None,
)]
#[case::header(
    &["-u", "alice", "-H", "X-Trace: secret-123"],
    "xr media status 777 --wait=60 -u alice -H 'X-Trace: <value>'",
    Some("secret-123"),
)]
#[tokio::test(flavor = "multi_thread")]
async fn a_resumed_wait_runs_under_the_credential_the_upload_used(
    #[case] flags: &[&str],
    #[case] invocation: &str,
    #[case] hidden: Option<&str>,
) {
    let mut args = vec!["--output", "json"];
    args.extend_from_slice(flags);
    let (output, _tmp) = upload_then_status(ResponseTemplate::new(500), &args).await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["next_step"]["action"], "resume-wait", "got: {v}");
    let (key, absent) = if hidden.is_some() {
        ("template", "command")
    } else {
        ("command", "template")
    };
    assert_eq!(v["next_step"][key], invocation, "got: {v}");
    assert_eq!(v["next_step"].get(absent), None, "never both: {v}");
    if let Some(hidden) = hidden {
        assert!(
            !stderr.contains(hidden),
            "a header value never echoes: {stderr}"
        );
    }
}

/// A wait whose deadline ran out resumes under the same credential too,
/// whichever media command waited.
#[tokio::test(flavor = "multi_thread")]
async fn an_upload_wait_past_its_deadline_resumes_under_its_credential() {
    let still_running = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "data": {"id": "777", "processing_info": {"state": "in_progress", "check_after_secs": 1}}
    }));
    let (output, _tmp) = upload_then_status(
        still_running,
        &[
            "--wait=1", "--output", "json", "--app", "myapp", "-u", "alice",
        ],
    )
    .await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "processing-timeout", "got: {v}");
    assert_eq!(
        v["next_step"]["command"], "xr --app myapp media status 777 --wait=2 -u alice",
        "got: {v}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_status_wait_past_its_deadline_resumes_under_its_credential() {
    let server = MockServer::start().await;
    status_mock(serde_json::json!({
        "id": "777",
        "processing_info": {"state": "in_progress", "check_after_secs": 1}
    }))
    .mount(&server)
    .await;
    let tmp = TempDir::new().expect("tempdir");

    let output = run_xr(
        &server,
        oauth2_store(tmp.path()),
        &[
            "--output",
            "json",
            "--app",
            "myapp",
            "media",
            "status",
            "777",
            "--wait=1",
            "--auth",
            "oauth2",
            "-H",
            "X-Trace: secret-123",
        ],
    )
    .await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "processing-timeout", "got: {v}");
    assert_eq!(
        v["next_step"]["template"],
        "xr --app myapp media status 777 --wait=2 --auth oauth2 -H 'X-Trace: <value>'",
        "got: {v}"
    );
    assert_eq!(v["next_step"].get("command"), None, "never both: {v}");
    assert!(
        !stderr.contains("secret-123"),
        "a header value never echoes: {stderr}"
    );
}

/// Text output prints the same invocation the envelope carries.
#[tokio::test(flavor = "multi_thread")]
async fn text_output_prints_the_resume_invocation_with_its_credential() {
    let (output, _tmp) = upload_then_status(
        ResponseTemplate::new(500),
        &["--app", "myapp", "-u", "alice"],
    )
    .await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Run: xr --app myapp media status 777 --wait=60 -u alice"),
        "{stderr}"
    );
}

/// Answers an upload's requests, one connection each, and drops the connection
/// of any status read without a byte of answer: what a network failure looks
/// like from the client.
fn serve_upload_then_drop_status(listener: &std::net::TcpListener) {
    use std::io::{BufRead, BufReader, Read, Write};
    for stream in listener.incoming() {
        let Ok(stream) = stream else { return };
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            continue;
        }
        let request = line.clone();
        let mut length = 0usize;
        loop {
            line.clear();
            if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
                break;
            }
            let lower = line.to_ascii_lowercase();
            if let Some(value) = lower.strip_prefix("content-length:") {
                length = value.trim().parse().unwrap_or(0);
            }
        }
        let mut body = vec![0u8; length];
        let _ = reader.read_exact(&mut body);
        if request.starts_with("GET ") {
            continue;
        }
        let answer = if request.contains("/initialize") {
            r#"{"data":{"id":"777","media_key":"7_777","expires_after_secs":86400}}"#
        } else {
            r#"{"data":{"id":"777","processing_info":{"state":"pending","check_after_secs":1}}}"#
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
             Connection: close\r\n\r\n{answer}",
            answer.len()
        );
        let _ = reader.into_inner().write_all(response.as_bytes());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_status_read_the_network_dropped_resumes_the_wait() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a free port");
    let base_url = format!("http://{}", listener.local_addr().expect("an address"));
    std::thread::spawn(move || serve_upload_then_drop_status(&listener));
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let video = tmp.path().join("clip.mp4");
    std::fs::write(&video, b"not really a video").expect("write clip");
    let output = tokio::task::spawn_blocking(move || {
        common::xr_with_store(&store)
            .env("API_BASE_URL", &base_url)
            .args(["--output", "json", "media", "upload"])
            .arg(&video)
            .timeout(std::time::Duration::from_secs(30))
            .output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "", "{stderr}");
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "network-error", "got: {v}");
    assert_eq!(v["media_id"], "777", "got: {v}");
    let message = v["message"].as_str().expect("a message");
    assert!(message.starts_with(AFTER_UPLOAD), "says what failed: {v}");
    assert_eq!(v["next_step"]["action"], "resume-wait", "got: {v}");
    assert_eq!(
        v["next_step"]["command"], "xr media status 777 --wait=60",
        "got: {v}"
    );
    assert_eq!(v["next_step"].get("docs"), None, "X never answered: {v}");
}

/// A failure before the upload finished leaves nothing to wait on, so running
/// the upload again is the right step, and there is no id to name.
#[rstest::rstest]
#[case::at_initialize("/2/media/upload/initialize")]
#[case::at_finalize("/2/media/upload/777/finalize")]
#[tokio::test(flavor = "multi_thread")]
async fn a_failure_before_the_upload_finished_keeps_retry(#[case] failing: &str) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(failing))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    mount_upload(&server, serde_json::json!({"id": "777"})).await;
    let tmp = TempDir::new().expect("tempdir");
    let video = tmp.path().join("clip.mp4");
    std::fs::write(&video, b"not really a video").expect("write clip");
    let video = video.to_str().expect("utf-8 path").to_string();

    let output = run_xr(
        &server,
        oauth2_store(tmp.path()),
        &["--output", "json", "media", "upload", video.as_str()],
    )
    .await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "server-error", "got: {v}");
    assert_eq!(v["next_step"]["action"], "retry", "got: {v}");
    assert_eq!(v.get("media_id"), None, "got: {v}");
}

/// A 429 on the status read keeps `wait-and-retry`, which says to wait for the
/// window first, whether or not X named its reset. What to send again is the
/// read, not the caller's own request, which was the upload: the step carries
/// the command that resumes the wait.
#[rstest::rstest]
#[case::with_reset(Some("1767225599"), &[], "xr media status 777 --wait=60")]
#[case::without_reset(None, &[], "xr media status 777 --wait=60")]
#[case::own_login(
    Some("1767225599"),
    &["--app", "myapp", "-u", "alice"],
    "xr --app myapp media status 777 --wait=60 -u alice",
)]
#[tokio::test(flavor = "multi_thread")]
async fn a_rate_limited_status_read_resumes_the_wait_once_the_window_passes(
    #[case] reset: Option<&str>,
    #[case] flags: &[&str],
    #[case] command: &str,
) {
    let mut limited = ResponseTemplate::new(429);
    if let Some(reset) = reset {
        limited = limited.insert_header("x-rate-limit-reset", reset);
    }
    let mut args = vec!["--output", "json"];
    args.extend_from_slice(flags);
    let (output, _tmp) = upload_then_status(limited, &args).await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(3), "stderr: {stderr}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "rate-limited", "got: {v}");
    assert_eq!(v["media_id"], "777", "got: {v}");
    assert_eq!(v["next_step"]["action"], "wait-and-retry", "got: {v}");
    assert_eq!(v["next_step"]["command"], command, "got: {v}");
    assert_eq!(
        v["next_step"]["docs"], "https://docs.x.com/resources/fundamentals/rate-limits",
        "got: {v}"
    );
    assert_eq!(
        v.get("retry_at").is_some(),
        reset.is_some(),
        "the time keys appear only when X named the reset: {v}"
    );
}

#[rstest::rstest]
#[case::with_reset(
    Some("1767225599"),
    "resume the wait in 0 seconds, at 2025-12-31T23:59:59Z. Run: xr media status 777 --wait=60"
)]
#[case::without_reset(
    None,
    "once the window passes, resume the wait. Run: xr media status 777 --wait=60"
)]
#[tokio::test(flavor = "multi_thread")]
async fn a_rate_limited_status_read_says_in_text_how_to_resume(
    #[case] reset: Option<&str>,
    #[case] line: &str,
) {
    let mut limited = ResponseTemplate::new(429);
    if let Some(reset) = reset {
        limited = limited.insert_header("x-rate-limit-reset", reset);
    }
    let (output, _tmp) = upload_then_status(limited, &[]).await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(line), "{stderr}");
}

/// Every JSON document on `stdout`, in order.
fn documents(stdout: &[u8]) -> Vec<serde_json::Value> {
    serde_json::Deserializer::from_slice(stdout)
        .into_iter::<serde_json::Value>()
        .collect::<Result<_, _>>()
        .expect("stdout holds only JSON documents")
}

/// The answer to a waited upload is FINALIZE's document carrying the final
/// processing state, so a caller parses stdout once and reads the media id
/// and the state from the same object. `--verbose` adds nothing to stdout
/// under a structured format.
#[rstest::rstest]
#[case::plain(&[])]
#[case::verbose(&["--verbose"])]
#[tokio::test(flavor = "multi_thread")]
async fn a_waited_upload_prints_one_document_with_the_final_state(#[case] extra: &[&str]) {
    let server = MockServer::start().await;
    mount_upload(
        &server,
        serde_json::json!({
            "id": "777",
            "expires_after_secs": 86400,
            "processing_info": {"state": "pending", "check_after_secs": 1}
        }),
    )
    .await;
    status_mock(serde_json::json!({
        "id": "777",
        "media_key": "7_777",
        "processing_info": {"state": "succeeded", "progress_percent": 100}
    }))
    .mount(&server)
    .await;
    let tmp = TempDir::new().expect("tempdir");
    let video = tmp.path().join("clip.mp4");
    std::fs::write(&video, b"not really a video").expect("write clip");

    let mut args = vec!["--output", "json"];
    args.extend_from_slice(extra);
    args.extend_from_slice(&["media", "upload", video.to_str().expect("utf-8 path")]);
    let output = run_xr(&server, oauth2_store(tmp.path()), &args).await;

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let documents = documents(&output.stdout);
    assert_eq!(
        documents.len(),
        1,
        "one document, not one per phase: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let data = &documents[0]["data"];
    assert_eq!(data["id"], "777");
    assert_eq!(
        data["expires_after_secs"], 86400,
        "FINALIZE's own field stays"
    );
    assert_eq!(
        data["media_key"], "7_777",
        "a field only the status carried is kept"
    );
    assert_eq!(
        data["processing_info"],
        serde_json::json!({"state": "succeeded", "progress_percent": 100}),
        "the final state replaces the pending one FINALIZE reported"
    );
}

/// X drops `processing_info` from the status of media it has finished with.
/// The pending state FINALIZE reported is then stale, and the one document
/// carries none.
#[tokio::test(flavor = "multi_thread")]
async fn a_finished_status_without_processing_info_clears_the_pending_state() {
    let server = MockServer::start().await;
    mount_upload(
        &server,
        serde_json::json!({
            "id": "777",
            "expires_after_secs": 86400,
            "processing_info": {"state": "pending", "check_after_secs": 1}
        }),
    )
    .await;
    status_mock(serde_json::json!({"id": "777", "media_key": "7_777"}))
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let video = tmp.path().join("clip.mp4");
    std::fs::write(&video, b"not really a video").expect("write clip");

    let output = run_xr(
        &server,
        oauth2_store(tmp.path()),
        &[
            "--output",
            "json",
            "media",
            "upload",
            video.to_str().expect("utf-8 path"),
        ],
    )
    .await;

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let documents = documents(&output.stdout);
    assert_eq!(
        documents.len(),
        1,
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(documents[0]["data"]["id"], "777");
    assert_eq!(
        documents[0]["data"].get("processing_info"),
        None,
        "no state is reported once X reports none"
    );
}

// ── The wait follows what X reports, not only the category ─────────────

/// Uploads a GIF whose FINALIZE answers `finalize`, with a STATUS that
/// reports success, and returns the run with the number of status calls.
async fn upload_a_gif(finalize: serde_json::Value) -> (std::process::Output, usize) {
    let server = MockServer::start().await;
    mount_upload(&server, finalize).await;
    status_mock(serde_json::json!({
        "id": "777",
        "processing_info": {"state": "succeeded", "progress_percent": 100}
    }))
    .mount(&server)
    .await;
    let tmp = TempDir::new().expect("tempdir");
    let gif = tmp.path().join("loop.gif");
    std::fs::write(&gif, b"not really a gif").expect("write gif");

    let output = run_xr(
        &server,
        oauth2_store(tmp.path()),
        &[
            "--output",
            "json",
            "media",
            "upload",
            gif.to_str().expect("utf-8 path"),
            "--media-type",
            "image/gif",
            "--category",
            "tweet_gif",
        ],
    )
    .await;
    let calls = status_calls(&server).await;
    (output, calls)
}

/// X says an upload needs processing by putting an unfinished
/// `processing_info` on FINALIZE's answer. A GIF that carries one is waited
/// for, though its category is not a video one.
#[rstest::rstest]
#[case::pending("pending")]
#[case::in_progress("in_progress")]
#[tokio::test(flavor = "multi_thread")]
async fn an_upload_x_reports_as_processing_is_waited_for(#[case] state: &str) {
    let (output, status_calls) = upload_a_gif(serde_json::json!({
        "id": "777",
        "processing_info": {"state": state, "check_after_secs": 1}
    }))
    .await;

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(status_calls, 1, "the status is read until it is final");
    let documents = documents(&output.stdout);
    assert_eq!(
        documents[0]["data"]["processing_info"]["state"], "succeeded",
        "the answer carries the final state"
    );
}

/// Without that signal, or with one already final, a GIF is ready at
/// FINALIZE and no status is read.
#[rstest::rstest]
#[case::no_processing_info(serde_json::json!({"id": "777"}))]
#[case::already_succeeded(serde_json::json!({
    "id": "777",
    "processing_info": {"state": "succeeded"}
}))]
#[tokio::test(flavor = "multi_thread")]
async fn an_upload_x_reports_as_ready_is_not_waited_for(#[case] finalize: serde_json::Value) {
    let (output, status_calls) = upload_a_gif(finalize).await;

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(status_calls, 0, "nothing to wait for");
}

/// `--wait` takes its value after `=`. A value after a space is a stray
/// argument, and the error says how to write it.
#[tokio::test(flavor = "multi_thread")]
async fn a_wait_value_after_a_space_points_at_the_equals_form() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");

    let output = run_xr(
        &server,
        oauth2_store(tmp.path()),
        &["--output", "json", "media", "status", "777", "--wait", "60"],
    )
    .await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "invalid-args", "got: {v}");
    assert!(
        v["message"]
            .as_str()
            .expect("message")
            .contains("--wait=60"),
        "the message shows the form that works: {v}"
    );
    assert_eq!(status_calls(&server).await, 0);
}
