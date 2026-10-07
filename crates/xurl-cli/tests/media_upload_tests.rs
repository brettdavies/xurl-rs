//! `xr media upload --wait` against a mock whose processing fails: the
//! upload itself completed at FINALIZE, so the caller still gets the media id
//! on stdout before the processing error ends the run.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::store::TokenStore;

/// A store whose default app holds a far-future OAuth2 token for `alice`, so
/// no refresh happens before the upload.
fn oauth2_store(tmp: &TempDir) -> std::path::PathBuf {
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_oauth2_token_for_app("myapp", "alice", "user-at", "user-rt", 9_999_999_999)
        .expect("save_oauth2");
    ts.set_default_app("myapp").expect("set_default_app");
    let _ = ts.remove_app("default");
    store
}

#[tokio::test(flavor = "multi_thread")]
async fn a_processing_failure_after_finalize_still_prints_the_media_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/upload/initialize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "777", "media_key": "7_777", "expires_after_secs": 86400}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/media/upload/777/append"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/media/upload/777/finalize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "777", "processing_info": {"state": "pending", "check_after_secs": 1}}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/media/upload"))
        .and(query_param("command", "STATUS"))
        .and(query_param("media_id", "777"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "777", "processing_info": {"state": "failed"}}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(&tmp);
    let video = tmp.path().join("clip.mp4");
    std::fs::write(&video, b"not really a video").expect("write clip");
    let base_url = server.uri();
    let output = tokio::task::spawn_blocking(move || {
        let mut cmd = common::xr_with_store(&store);
        cmd.env("API_BASE_URL", &base_url)
            .args(["media", "upload", video.to_str().expect("utf-8 path")])
            .output()
    })
    .await
    .expect("spawn_blocking joins")
    .expect("xr runs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(output.status.code(), Some(0), "processing failed: {stderr}");
    assert!(
        stdout.contains("\"777\""),
        "the FINALIZE envelope with the media id reaches stdout first: {stdout}"
    );
    assert!(
        stderr.contains("media processing failed"),
        "the processing error is reported: {stderr}"
    );
}

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
        oauth2_store(&tmp),
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
        oauth2_store(&tmp),
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
        oauth2_store(&tmp),
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
        oauth2_store(&tmp),
        &["--output", "json", "media", "status", "777", "--wait=5"],
    )
    .await;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(stderr.trim()).expect("a JSON envelope");
    assert_eq!(v["reason"], "validation", "got: {v}");
    assert_eq!(v["message"], "media processing failed", "got: {v}");
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
        oauth2_store(&tmp),
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
        oauth2_store(&tmp),
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

/// `--wait` takes its value after `=`. A value after a space is a stray
/// argument, and the error says how to write it.
#[tokio::test(flavor = "multi_thread")]
async fn a_wait_value_after_a_space_points_at_the_equals_form() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");

    let output = run_xr(
        &server,
        oauth2_store(&tmp),
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
