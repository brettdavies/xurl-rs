//! A request X answers 401 names the credential it went out under and carries
//! the step that gets that credential again.
//!
//! Each case runs `xr` in process against a local server that answers the
//! endpoint 401. The `auth` object is the one the command's dry run reports,
//! without `token_expired`.

mod common;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tempfile::TempDir;
use xdk::config::EnvOverrides;
use xdk::store::TokenStore;
use xdk::testing::MockX;

use common::invoke::{Fill, SAMPLE, bearer_only, live_oauth2_only, nothing, oauth1_only};

const OTHER: &str = "other-app";

fn x_body() -> Value {
    json!({"title": "Unauthorized", "type": "about:blank", "status": 401, "detail": "Unauthorized"})
}

fn store_in(dir: &Path, fill: Fill) -> PathBuf {
    let store = dir.join(".xurl");
    let mut ts = TokenStore::new_with_path(&store.to_string_lossy());
    ts.add_app(SAMPLE, "CLIENT-ID", "CLIENT-SECRET")
        .expect("add app");
    fill(&mut ts);
    store
}

fn other_app_with_a_login(ts: &mut TokenStore) {
    oauth1_only(ts);
    ts.add_app(OTHER, "CLIENT-ID", "CLIENT-SECRET")
        .expect("add app");
    let in_an_hour = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock is past the epoch")
        .as_secs()
        + 3600;
    ts.save_oauth2_token_for_app(OTHER, "alice", "LIVE-ACCESS", "REFRESH", in_an_hour)
        .expect("save oauth2");
}

/// Runs `args` against `mock`, with `bearer` as `XURL_BEARER_TOKEN` when given.
async fn run(
    mock: &MockX,
    fill: Fill,
    bearer: Option<&str>,
    args: &[&str],
) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = store_in(tmp.path(), fill);
    let overrides = EnvOverrides {
        api_base_url: Some(mock.base_url()),
        bearer_token: bearer.map(str::to_string),
        ..EnvOverrides::default()
    };
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code =
        xurl::cli::runner::run_with_overrides(args, &mut stdout, &mut stderr, &store, &overrides)
            .await;
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

/// The JSON envelope of a run X answered 401.
async fn refused(mock: &MockX, fill: Fill, bearer: Option<&str>, args: &[&str]) -> Value {
    let (code, stdout, stderr) = run(mock, fill, bearer, args).await;
    assert_eq!(code, 77, "stdout: {stdout} stderr: {stderr}");
    let envelope: Value = serde_json::from_str(stderr.trim()).expect("one JSON envelope");
    assert_eq!(envelope["reason"], "auth-required", "{envelope}");
    assert_eq!(envelope["message"], x_body().to_string(), "{envelope}");
    envelope
}

async fn mock_answering(method: &str, path: &str, status: u16, body: Value) -> MockX {
    let mock = MockX::start().await;
    mock.stub(method, path, status, body).await;
    mock
}

const WHOAMI: &[&str] = &["xr", "--output", "json", "whoami"];
const SEARCH: &[&str] = &["xr", "--output", "json", "search", "hello"];

#[tokio::test]
async fn a_401_under_a_login_names_it_and_signs_in_again() {
    let mock = mock_answering("GET", "^/2/users/me$", 401, x_body()).await;

    let envelope = refused(&mock, live_oauth2_only, None, WHOAMI).await;

    assert_eq!(
        envelope["auth"],
        json!({"scheme": "oauth2", "app": SAMPLE, "username": SAMPLE}),
        "{envelope}"
    );
    assert_eq!(
        envelope["next_step"],
        json!({
            "action": "sign-in",
            "command": format!("xr --output json auth oauth2 {SAMPLE} --no-browser --step 1 --app {SAMPLE}"),
        }),
        "{envelope}"
    );
}

#[tokio::test]
async fn a_401_under_oauth1_keys_names_them_and_stores_a_pair_again() {
    let mock = mock_answering("GET", "^/2/users/me$", 401, x_body()).await;

    let envelope = refused(&mock, oauth1_only, None, WHOAMI).await;

    assert_eq!(
        envelope["auth"],
        json!({"scheme": "oauth1", "app": SAMPLE}),
        "{envelope}"
    );
    assert_eq!(
        envelope["next_step"],
        json!({
            "action": "fix-input",
            "template": format!(
                "xr --output json auth oauth1 --app {SAMPLE} --consumer-key <consumer-key> --consumer-secret-file <path> --access-token-file <path> --token-secret-file <path>"
            ),
        }),
        "{envelope}"
    );
}

#[tokio::test]
async fn a_401_under_a_stored_bearer_token_names_it_and_stores_one_again() {
    let mock = mock_answering("GET", "^/2/tweets/search/recent$", 401, x_body()).await;

    let envelope = refused(&mock, bearer_only, None, SEARCH).await;

    assert_eq!(
        envelope["auth"],
        json!({"scheme": "app", "app": SAMPLE}),
        "{envelope}"
    );
    assert_eq!(
        envelope["next_step"],
        json!({
            "action": "fix-input",
            "template": format!("xr --output json auth app --app {SAMPLE} --bearer-token-file <path>"),
        }),
        "{envelope}"
    );
}

/// No `xr` command replaces an environment variable, so the step names none.
#[tokio::test]
async fn a_401_under_the_environment_bearer_token_names_the_variable() {
    let mock = mock_answering("GET", "^/2/tweets/search/recent$", 401, x_body()).await;

    let envelope = refused(&mock, nothing, Some("ENV-BEARER"), SEARCH).await;

    assert_eq!(envelope["auth"], json!({"scheme": "app"}), "{envelope}");
    let step = &envelope["next_step"];
    assert_eq!(step["action"], "fix-input", "{envelope}");
    assert!(
        step["command"].is_null() && step["template"].is_null(),
        "{envelope}"
    );
    assert!(
        step["docs"]
            .as_str()
            .is_some_and(|docs| docs.starts_with("https://docs.x.com/")),
        "{envelope}"
    );
}

/// The refused credential is the one of the app `--app` names, and the sign-in
/// names that app and the login.
#[tokio::test]
async fn a_401_under_a_named_app_names_that_app() {
    let mock = mock_answering("GET", "^/2/users/me$", 401, x_body()).await;

    let envelope = refused(
        &mock,
        other_app_with_a_login,
        None,
        &["xr", "--app", OTHER, "--output", "json", "whoami"],
    )
    .await;

    assert_eq!(
        envelope["auth"],
        json!({"scheme": "oauth2", "app": OTHER, "username": "alice"}),
        "{envelope}"
    );
    assert_eq!(
        envelope["next_step"]["command"],
        format!("xr --output json auth oauth2 alice --no-browser --step 1 --app {OTHER}"),
        "{envelope}"
    );
}

#[tokio::test]
async fn a_401_on_a_raw_request_names_the_credential() {
    let mock = mock_answering("GET", "^/2/users/me$", 401, x_body()).await;

    let envelope = refused(
        &mock,
        oauth1_only,
        None,
        &["xr", "--output", "json", "/2/users/me"],
    )
    .await;

    assert_eq!(
        envelope["auth"],
        json!({"scheme": "oauth1", "app": SAMPLE}),
        "{envelope}"
    );
    assert_eq!(envelope["next_step"]["action"], "fix-input", "{envelope}");
}

/// The caller's id is resolved first, so the refusal can come from the second
/// request; the credential is the one that request went out under.
#[tokio::test]
async fn a_401_on_the_second_request_of_a_command_names_its_credential() {
    let mock = mock_answering("POST", "^/2/users/[^/]+/likes$", 401, x_body()).await;

    let envelope = refused(
        &mock,
        live_oauth2_only,
        None,
        &["xr", "--output", "json", "like", "1234567890"],
    )
    .await;

    assert_eq!(
        envelope["auth"],
        json!({"scheme": "oauth2", "app": SAMPLE, "username": SAMPLE}),
        "{envelope}"
    );
    assert_eq!(envelope["next_step"]["action"], "sign-in", "{envelope}");
}

#[tokio::test]
async fn text_output_names_the_credential_and_the_step() {
    let mock = mock_answering("GET", "^/2/users/me$", 401, x_body()).await;

    let (code, stdout, stderr) = run(&mock, live_oauth2_only, None, &["xr", "whoami"]).await;

    assert_eq!(code, 77, "stdout: {stdout} stderr: {stderr}");
    assert!(stderr.contains(&x_body().to_string()), "{stderr}");
    assert!(
        stderr.contains(&format!(
            "X refused the OAuth 2.0 credentials of app \"{SAMPLE}\", the login \"{SAMPLE}\". Sign in again. Run: xr auth oauth2 {SAMPLE}"
        )),
        "{stderr}"
    );
}

/// X's 403 for an app-only bearer token on a user-context endpoint, as
/// captured: the one 403 whose problem `type` names the credential.
const UNSUPPORTED_AUTHENTICATION: &str = r#"{"title":"Unsupported Authentication","detail":"Authenticating with OAuth 2.0 Application-Only is forbidden for this endpoint.  Supported authentication types are [OAuth 1.0a User Context, OAuth 2.0 User Context].","type":"https://api.twitter.com/2/problems/unsupported-authentication","status":403}"#;

/// X's 403 for a login without the scope an endpoint needs, as captured: the
/// generic answer, which names no credential.
const GENERIC_FORBIDDEN: &str =
    r#"{"detail":"Forbidden","status":403,"title":"Forbidden","type":"about:blank"}"#;

const UNLISTED: &[&str] = &[
    "xr",
    "--output",
    "json",
    "--auth",
    "app",
    "/2/unlisted/thing",
];

fn oauth1_and_bearer(ts: &mut TokenStore) {
    oauth1_only(ts);
    bearer_only(ts);
}

fn body(raw: &str) -> Value {
    serde_json::from_str(raw).expect("captured body is JSON")
}

/// The JSON envelope of a run X answered 403 with `raw`.
async fn forbidden(raw: &str, fill: Fill, args: &[&str]) -> Value {
    let mock = mock_answering("GET", "^/2/(users/me|unlisted/thing)$", 403, body(raw)).await;
    let (code, stdout, stderr) = run(&mock, fill, None, args).await;
    assert_eq!(code, 1, "stdout: {stdout} stderr: {stderr}");
    let envelope: Value = serde_json::from_str(stderr.trim()).expect("one JSON envelope");
    assert_eq!(envelope["reason"], "forbidden", "{envelope}");
    assert_eq!(envelope["message"], body(raw).to_string(), "{envelope}");
    envelope
}

/// The app stores a scheme the endpoint takes, so the step reruns with it.
#[tokio::test]
async fn a_403_refusing_the_credential_reruns_under_an_accepted_scheme_the_app_stores() {
    let envelope = forbidden(UNSUPPORTED_AUTHENTICATION, oauth1_and_bearer, UNLISTED).await;

    assert_eq!(
        envelope["auth"],
        json!({"scheme": "app", "app": SAMPLE}),
        "{envelope}"
    );
    assert_eq!(envelope["next_step"]["action"], "fix-input", "{envelope}");
    assert_eq!(
        envelope["next_step"]["command"], "xr --output json --auth oauth1 /2/unlisted/thing",
        "{envelope}"
    );
}

/// The app stores nothing the endpoint takes, so the step gets one.
#[tokio::test]
async fn a_403_refusing_the_credential_gets_a_scheme_the_app_stores_none_of() {
    let envelope = forbidden(UNSUPPORTED_AUTHENTICATION, bearer_only, UNLISTED).await;

    assert_eq!(
        envelope["auth"],
        json!({"scheme": "app", "app": SAMPLE}),
        "{envelope}"
    );
    assert_eq!(envelope["next_step"]["action"], "sign-in", "{envelope}");
}

/// A scope the login lacks is X's generic 403, which names no credential.
#[tokio::test]
async fn a_403_about_the_request_keeps_its_envelope() {
    let envelope = forbidden(GENERIC_FORBIDDEN, live_oauth2_only, WHOAMI).await;

    assert!(envelope.get("auth").is_none(), "{envelope}");
    assert!(envelope.get("next_step").is_none(), "{envelope}");
}

/// An enrollment refusal is about the app, whatever else its body names.
#[tokio::test]
async fn an_enrollment_403_keeps_its_step() {
    let raw = r#"{"title":"Client Forbidden","detail":"Your client is not enrolled: client-not-enrolled","reason":"client-not-enrolled","type":"https://api.twitter.com/2/problems/unsupported-authentication","status":403}"#;
    let mock = mock_answering("GET", "^/2/users/me$", 403, body(raw)).await;

    let (code, _stdout, stderr) = run(&mock, live_oauth2_only, None, WHOAMI).await;

    let envelope: Value = serde_json::from_str(stderr.trim()).expect("one JSON envelope");
    assert_eq!(code, 1, "{envelope}");
    assert_eq!(envelope["next_step"]["action"], "enroll-app", "{envelope}");
    assert!(envelope.get("auth").is_none(), "{envelope}");
}

/// The text names what X refused before it names the step.
#[tokio::test]
async fn text_output_after_a_403_names_the_refused_credential() {
    let mock = mock_answering(
        "GET",
        "^/2/unlisted/thing$",
        403,
        body(UNSUPPORTED_AUTHENTICATION),
    )
    .await;

    let (code, _stdout, stderr) = run(
        &mock,
        oauth1_and_bearer,
        None,
        &["xr", "--auth", "app", "/2/unlisted/thing"],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(
        stderr.contains(&format!(
            "X refused the Bearer (app) credentials of app \"{SAMPLE}\": this endpoint does not take it."
        )),
        "{stderr}"
    );
}
