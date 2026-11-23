//! A run that names an app the store does not hold is refused as `token-store`
//! with the app's name as `value`, before any request goes out, and its dry
//! run says so instead of naming a credential from another app.

mod common;

use serde_json::{Value, json};
use tempfile::TempDir;
use wiremock::MockServer;

use common::{oauth1_store, run_in_process};

const GHOST: &str = "ghost";

/// `xr --output json <args>` against a store whose default app, `myapp`, holds
/// an `OAuth1` credential, and a mock server that must receive nothing: the
/// exit code, stdout, and stderr.
async fn invoke(args: &[&str]) -> (i32, String, String) {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let (code, stdout, stderr) = run_in_process(&store, &server.uri(), args).await;
    assert_eq!(
        server.received_requests().await.expect("recorded").len(),
        0,
        "{args:?}: a request went out"
    );
    (code, stdout, stderr)
}

fn json_of(text: &str) -> Value {
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("not JSON ({e}): {text}"))
}

fn refusal_step() -> Value {
    json!({"action": "fix-input", "command": "xr --output json auth status"})
}

#[rstest::rstest]
#[case::whoami(&["whoami"])]
#[case::search(&["search", "rust"])]
#[case::raw(&["/2/users/me"])]
#[case::post(&["post", "hello"])]
#[case::followers(&["followers"])]
#[tokio::test]
async fn a_request_under_an_app_the_store_does_not_hold_is_refused(#[case] command: &[&str]) {
    let mut args = vec!["--output", "json", "--app", GHOST];
    args.extend_from_slice(command);

    let (code, stdout, stderr) = invoke(&args).await;

    assert_eq!(code, 77, "{args:?}: {stderr}");
    assert_eq!(stdout, "", "{args:?}");
    let envelope = json_of(&stderr);
    assert_eq!(envelope["reason"], "token-store", "{args:?}: {envelope}");
    assert_eq!(envelope["exit_code"], 77, "{args:?}: {envelope}");
    assert_eq!(envelope["value"], GHOST, "{args:?}: {envelope}");
    assert_eq!(
        envelope["next_step"],
        refusal_step(),
        "{args:?}: {envelope}"
    );
    assert_eq!(
        envelope["message"], "Token Store Error: the store holds no app named \"ghost\"",
        "{args:?}"
    );
}

#[rstest::rstest]
#[case::whoami(&["whoami"])]
#[case::search(&["search", "rust"])]
#[case::raw(&["/2/users/me"])]
#[case::post(&["post", "hello"])]
#[case::followers(&["followers"])]
#[case::delete(&["delete", "12345"])]
#[case::webhooks_listen(&["webhooks", "listen"])]
#[tokio::test]
async fn a_dry_run_under_an_app_the_store_does_not_hold_refuses_and_names_no_credential(
    #[case] command: &[&str],
) {
    let mut args = vec!["--output", "json", "--dry-run", "--app", GHOST];
    args.extend_from_slice(command);

    let (code, stdout, stderr) = invoke(&args).await;

    assert_eq!(code, 0, "{args:?}: {stderr}");
    let answer = json_of(&stdout);
    assert_eq!(answer["status"], "dry_run", "{args:?}: {answer}");
    assert_eq!(answer["would_succeed"], false, "{args:?}: {answer}");
    assert_eq!(answer["reason"], "token-store", "{args:?}: {answer}");
    assert_eq!(answer["exit_code"], 77, "{args:?}: {answer}");
    assert_eq!(answer["value"], GHOST, "{args:?}: {answer}");
    assert_eq!(answer["next_step"], refusal_step(), "{args:?}: {answer}");
    assert!(
        answer.get("auth").is_none(),
        "{args:?}: the dry run names a credential: {answer}"
    );
    assert!(
        answer.get("confirmation_required").is_none(),
        "{args:?}: the dry run says it would ask: {answer}"
    );
}

#[tokio::test]
async fn a_real_webhooks_listen_under_an_app_the_store_does_not_hold_is_refused() {
    let (code, stdout, stderr) =
        invoke(&["--output", "json", "--app", GHOST, "webhooks", "listen"]).await;

    assert_eq!(code, 77, "{stderr}");
    assert_eq!(stdout, "");
    let envelope = json_of(&stderr);
    assert_eq!(envelope["reason"], "token-store", "{envelope}");
    assert_eq!(envelope["value"], GHOST, "{envelope}");
}

#[tokio::test]
async fn the_text_refusal_names_the_app_and_the_apps_the_store_holds() {
    let (code, _stdout, stderr) = invoke(&["--app", GHOST, "whoami"]).await;

    assert_eq!(code, 77, "{stderr}");
    assert!(
        stderr.contains("the store holds no app named \"ghost\""),
        "{stderr}"
    );
    assert!(stderr.contains("myapp"), "{stderr}");
    assert!(stderr.contains("xr auth status"), "{stderr}");
}

#[rstest::rstest]
#[case::no_app(&[])]
#[case::empty_name(&["--app", ""])]
#[case::held_app(&["--app", "myapp"])]
#[tokio::test]
async fn the_default_app_and_a_held_app_still_name_their_credential(#[case] app: &[&str]) {
    let mut args = vec!["--output", "json", "--dry-run"];
    args.extend_from_slice(app);
    args.push("whoami");

    let (code, stdout, stderr) = invoke(&args).await;

    assert_eq!(code, 0, "{args:?}: {stderr}");
    let answer = json_of(&stdout);
    assert_eq!(answer["would_succeed"], true, "{args:?}: {answer}");
    assert_eq!(answer["auth"]["app"], "myapp", "{args:?}: {answer}");
    assert_eq!(answer["auth"]["scheme"], "oauth1", "{args:?}: {answer}");
}

#[tokio::test]
async fn a_store_that_could_not_be_loaded_checks_no_app_name_and_leaves_the_bearer_to_serve() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    std::fs::write(&store, "not: [valid yaml").expect("write a store that does not parse");
    let overrides = xdk::config::EnvOverrides {
        api_base_url: Some(server.uri()),
        bearer_token: Some("env-bearer-value".to_string()),
        ..xdk::config::EnvOverrides::default()
    };
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();

    let code = xurl::cli::runner::run_with_overrides(
        [
            "xr",
            "--output",
            "json",
            "--dry-run",
            "--app",
            GHOST,
            "search",
            "rust",
        ],
        &mut stdout,
        &mut stderr,
        &store,
        &overrides,
    )
    .await;

    let stdout = String::from_utf8_lossy(&stdout);
    assert_eq!(code, 0, "{}", String::from_utf8_lossy(&stderr));
    let answer = json_of(&stdout);
    assert_eq!(answer["would_succeed"], true, "{answer}");
    assert_eq!(answer["auth"]["scheme"], "app", "{answer}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

/// `xr <args>` as a child process against the store `oauth1_store` builds,
/// with `environment` set on the child and every X endpoint at a closed port:
/// the exit code, stdout, and stderr.
fn child(args: &[&str], environment: &[(&str, &str)]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let mut command = common::xr_with_store(&store);
    command.env("API_BASE_URL", common::CLOSED);
    for (key, value) in environment {
        command.env(key, value);
    }
    let output = command.args(args).output().expect("xr runs");
    (
        output.status.code().expect("exit code"),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[rstest::rstest]
#[case::request(&["--output", "json", "--app", GHOST, "whoami"], false)]
#[case::store_command(&["--output", "json", "auth", "default", GHOST], false)]
#[case::dry_run(&["--output", "json", "--dry-run", "--app", GHOST, "whoami"], true)]
#[tokio::test]
async fn the_envelope_names_the_apps_the_store_holds(#[case] args: &[&str], #[case] dry_run: bool) {
    let (code, stdout, stderr) = invoke(args).await;

    let envelope = json_of(if dry_run { &stdout } else { &stderr });
    assert_eq!(code, if dry_run { 0 } else { 77 }, "{args:?}: {stderr}");
    assert_eq!(envelope["reason"], "token-store", "{args:?}: {envelope}");
    assert_eq!(envelope["value"], GHOST, "{args:?}: {envelope}");
    assert_eq!(envelope["held"], json!(["myapp"]), "{args:?}: {envelope}");
}

#[tokio::test]
async fn a_store_that_holds_no_app_carries_an_empty_list() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let (code, _stdout, stderr) = run_in_process(
        &store,
        &server.uri(),
        &["--output", "json", "--app", GHOST, "whoami"],
    )
    .await;

    assert_eq!(code, 77, "{stderr}");
    let envelope = json_of(&stderr);
    assert_eq!(envelope["reason"], "token-store", "{envelope}");
    assert_eq!(envelope["held"], json!([]), "{envelope}");
}

#[test]
fn another_token_store_failure_carries_no_list_of_apps() {
    let (code, _stdout, stderr) = child(&["--output", "json", "auth", "apps", "add", "myapp"], &[]);
    let envelope = json_of(&stderr);
    assert_ne!(code, 0, "{envelope}");
    assert!(envelope.get("held").is_none(), "{envelope}");
}

#[rstest::rstest]
#[case::request(&["--output", "json", "whoami"], 77, false)]
#[case::dry_run(&["--output", "json", "--dry-run", "whoami"], 0, true)]
fn a_name_that_came_from_xurl_app_says_so(
    #[case] args: &[&str],
    #[case] exit_code: i32,
    #[case] on_stdout: bool,
) {
    let (code, stdout, stderr) = child(args, &[("XURL_APP", GHOST)]);

    assert_eq!(code, exit_code, "{args:?}: {stderr}");
    let envelope = json_of(if on_stdout { &stdout } else { &stderr });
    assert_eq!(
        envelope["message"],
        "Token Store Error: the store holds no app named \"ghost\" (from XURL_APP)",
        "{args:?}: {envelope}"
    );
}

#[test]
fn the_text_refusal_says_when_the_name_came_from_xurl_app() {
    let (code, _stdout, stderr) = child(&["whoami"], &[("XURL_APP", GHOST)]);

    assert_eq!(code, 77, "{stderr}");
    assert!(
        stderr.contains("the store holds no app named \"ghost\" (from XURL_APP)"),
        "{stderr}"
    );
}

#[rstest::rstest]
#[case::flag_over_the_variable(&["--output", "json", "--app", GHOST, "whoami"], &[("XURL_APP", "other")])]
#[case::flag_matching_the_variable(&["--output", "json", "--app", GHOST, "whoami"], &[("XURL_APP", GHOST)])]
#[case::name_given_to_a_store_command(&["--output", "json", "auth", "default", GHOST], &[("XURL_APP", "myapp")])]
#[case::name_given_to_a_store_command_and_the_variable(&["--output", "json", "auth", "default", GHOST], &[("XURL_APP", GHOST)])]
#[case::name_removed_by_a_store_command_and_given_by_the_variable(&["--output", "json", "auth", "apps", "remove", GHOST, "--force"], &[("XURL_APP", GHOST)])]
#[case::no_variable(&["--output", "json", "--app", GHOST, "whoami"], &[])]
fn a_name_that_did_not_come_from_xurl_app_does_not_say_so(
    #[case] args: &[&str],
    #[case] environment: &[(&str, &str)],
) {
    let (code, _stdout, stderr) = child(args, environment);

    assert_eq!(code, 77, "{args:?}: {stderr}");
    let envelope = json_of(&stderr);
    assert_eq!(
        envelope["message"], "Token Store Error: the store holds no app named \"ghost\"",
        "{args:?}: {envelope}"
    );
}

#[test]
fn the_app_flag_help_says_a_name_the_store_does_not_hold_is_refused() {
    let (code, stdout, stderr) = child(&["--help"], &[]);

    assert_eq!(code, 0, "{stderr}");
    let line = stdout
        .lines()
        .skip_while(|line| !line.trim_start().starts_with("--app"))
        .take(3)
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        line.contains("name the store does not hold is refused"),
        "{line}"
    );
}
