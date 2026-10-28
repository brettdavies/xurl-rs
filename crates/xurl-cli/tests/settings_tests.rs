//! `xr settings` reaches `/2/users/me/settings` through the runner: `get`
//! prints the account's settings, `update` sends only the settings given,
//! and inputs the spec bounds are refused before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

async fn run_json(server: &MockServer, args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let mut argv = args.to_vec();
    argv.extend(["--output", "json"]);
    run(&store, &server.uri(), &argv).await
}

#[tokio::test]
async fn get_prints_the_accounts_settings() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/me/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "2244994945", "protected": false, "allow_dms_from": "all"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(&server, &["settings", "get"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["data"]["protected"], false);
    assert_eq!(body["data"]["allow_dms_from"], "all");
}

#[tokio::test]
async fn update_sends_only_the_settings_given() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/2/users/me/settings"))
        .and(body_json(serde_json::json!({
            "allow_dms_from": "following",
            "protected": true,
            "quiet_hours": {"start_hour": 22, "end_hour": 7},
            "time_zone": {"name": "America/Chicago"}
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"updated": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "settings",
            "update",
            "--allow-dms-from",
            "following",
            "--protected",
            "true",
            "--quiet-hours-start",
            "22",
            "--quiet-hours-end",
            "7",
            "--time-zone",
            "America/Chicago",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["updated"], true);
}

#[tokio::test]
async fn inputs_outside_the_spec_bounds_are_refused_before_any_request() {
    let server = MockServer::start().await;

    for (args, reason) in [
        (vec!["settings", "update"], "no-settings-changes"),
        (
            vec!["settings", "update", "--language", "e"],
            "invalid-settings-language",
        ),
        (
            vec!["settings", "update", "--time-zone", ""],
            "invalid-settings-time-zone",
        ),
        (
            vec!["settings", "update", "--quiet-hours-end", "24"],
            "invalid-quiet-hour",
        ),
    ] {
        let mut argv = vec!["--dry-run"];
        argv.extend(&args);
        let (code, stdout, stderr) = run_json(&server, &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{args:?}");
        assert_eq!(body["reason"], reason, "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_setting_takes_only_the_values_the_spec_lists() {
    let server = MockServer::start().await;

    for args in [
        vec!["settings", "update", "--protected", "yes"],
        vec!["settings", "update", "--allow-dms-from", "everyone"],
        vec!["settings", "update", "--allow-media-tagging", "verified"],
    ] {
        let (code, _stdout, stderr) = run_json(&server, &args).await;

        assert_eq!(code, 2, "{args:?}: {stderr}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
