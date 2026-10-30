//! `xr communities search` and `xr communities get` search X's communities
//! and read one by id. Searching takes a user login only, while reading one
//! also takes an app-only bearer token; the auth matrix decides, with no
//! per-command code. An id the spec's pattern rejects is refused before any
//! request.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const COMMUNITY_ID: &str = "1146654567674912769";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn bearer_store(dir: &Path) -> PathBuf {
    let store = dir.join(".xurl");
    let mut ts = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_bearer_token_for_app("myapp", "BEARER-TOKEN")
        .expect("save bearer");
    ts.set_default_app("myapp").expect("set_default_app");
    store
}

#[tokio::test]
async fn search_sends_the_query_and_prints_the_communities() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/communities/search"))
        .and(query_param("query", "rust"))
        .and(query_param("max_results", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": COMMUNITY_ID, "name": "Rustaceans", "member_count": 4200}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["communities", "search", "rust", "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["member_count"], 4200);
}

#[tokio::test]
async fn get_goes_out_with_the_bearer_token_alone() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/communities/{COMMUNITY_ID}")))
        .and(header("authorization", "Bearer BEARER-TOKEN"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": COMMUNITY_ID, "name": "Rustaceans", "access": "Public"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &["communities", "get", COMMUNITY_ID, "--output", "json"],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["access"], "Public");
}

#[tokio::test]
async fn search_with_only_a_bearer_is_an_auth_mismatch() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = bearer_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "communities", "search", "rust"],
    )
    .await;

    assert_ne!(code, 0, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "auth-method-mismatch");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn get_refuses_an_id_outside_the_spec_pattern_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());

    for id in ["", "rustaceans", "12345678901234567890"] {
        let (code, stdout, stderr) = run(
            &store,
            &server.uri(),
            &["--dry-run", "--output", "json", "communities", "get", id],
        )
        .await;

        assert_eq!(code, 0, "{id:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{id:?}");
        assert_eq!(body["rule"], "invalid-community-id", "{id:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
