//! `xr broadcasts moderators` reaches the chat moderator endpoints through
//! the runner: the list is a GET, add resolves the handle and POSTs the user
//! id, and both print the typed response as JSON.

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use xdk::config::EnvOverrides;
use xdk::store::TokenStore;
use xurl::cli;

/// A store whose default app carries an `OAuth1` token, a scheme every
/// chat moderator endpoint accepts.
fn oauth1_store(tmp: &TempDir) -> std::path::PathBuf {
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_oauth1_tokens_for_app(
        "myapp",
        "OA1-ACCESS-TOKEN",
        "TOKEN-SECRET",
        "OA1-CONSUMER-KEY",
        "CONSUMER-SECRET",
    )
    .expect("save_oauth1");
    ts.set_default_app("myapp").expect("set_default_app");
    let _ = ts.remove_app("default");
    store
}

async fn run(store: &std::path::Path, base_url: &str, args: &[&str]) -> (i32, String, String) {
    let overrides = EnvOverrides {
        api_base_url: Some(base_url.to_string()),
        ..EnvOverrides::default()
    };
    let mut argv = vec!["xr"];
    argv.extend_from_slice(args);
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code =
        cli::runner::run_with_overrides(argv, &mut stdout, &mut stderr, store, &overrides).await;
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

#[tokio::test]
async fn moderators_list_prints_the_users_as_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/broadcasts/chat/moderators"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "2244994945", "name": "Helper", "username": "helper"}],
            "meta": {"result_count": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(&tmp);

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["broadcasts", "moderators", "list", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"][0]["username"], "helper");
}

#[tokio::test]
async fn moderators_add_resolves_the_handle_and_posts_the_user_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by/username/helper"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "2244994945", "name": "Helper", "username": "helper"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/2/broadcasts/chat/moderators"))
        .and(body_json(serde_json::json!({"user_id": "2244994945"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"moderator_user_ids": ["2244994945"]}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(&tmp);

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "moderators",
            "add",
            "@helper",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["data"]["moderator_user_ids"][0], "2244994945");
}

#[tokio::test]
async fn moderators_remove_deletes_by_the_resolved_user_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by/username/helper"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "2244994945", "name": "Helper", "username": "helper"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/2/broadcasts/chat/moderators/2244994945"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"moderator_user_ids": []}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(&tmp);

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "broadcasts",
            "moderators",
            "remove",
            "@helper",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn moderators_add_dry_run_validates_without_a_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(&tmp);

    let (code, stdout, _stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "broadcasts",
            "moderators",
            "add",
            "@helper",
        ],
    )
    .await;

    assert_eq!(code, 0);
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON on stdout");
    assert_eq!(body["would_succeed"], true, "{body}");
    assert_eq!(body["command"], "broadcasts-moderators-add", "{body}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
