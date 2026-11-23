//! An app name the store does not hold names no credential: a request under it
//! is refused before anything is sent, and nothing reads or writes the default
//! app in its place.

use std::collections::HashMap;
use std::path::Path;

use tempfile::TempDir;
use wiremock::MockServer;
use xdk::api::auth_matrix::WireScheme;
use xdk::api::{Client, RequestOptions, RequestTarget};
use xdk::auth::Auth;
use xdk::config::{Config, EnvOverrides};
use xdk::error::{Error, NextAction};
use xdk::store::TokenStore;

const APP: &str = "main-app";
const OTHER: &str = "other-app";
const GHOST: &str = "ghost-app";
const USER: &str = "main-user";

/// `APP`, the default, holds every kind of credential; `OTHER` holds none.
fn seed(path: &Path) {
    let mut store = TokenStore::new_with_path(&path.to_string_lossy());
    store
        .add_app(APP, "CLIENT-ID", "CLIENT-SECRET")
        .expect("add app");
    store
        .save_oauth2_token_for_app(APP, USER, "ACCESS", "REFRESH", 9_999_999_999)
        .expect("save oauth2");
    store
        .save_oauth1_tokens_for_app(APP, "ACCESS", "SECRET", "KEY", "CONSUMER")
        .expect("save oauth1");
    store
        .save_bearer_token_for_app(APP, "BEARER")
        .expect("save bearer");
    store
        .add_app(OTHER, "OTHER-ID", "OTHER-SECRET")
        .expect("add the other app");
}

fn overrides(server: &MockServer, env_bearer: Option<&str>) -> EnvOverrides {
    EnvOverrides {
        api_base_url: Some(server.uri()),
        token_url: Some(format!("{}/token", server.uri())),
        info_url: Some(format!("{}/me", server.uri())),
        bearer_token: env_bearer.map(str::to_string),
        ..EnvOverrides::default()
    }
}

/// A client over the store at `path` that names `app`, which an empty name
/// leaves to the store's default.
fn client(path: &Path, server: &MockServer, app: &str, env_bearer: Option<&str>) -> Client {
    let overrides = overrides(server, env_bearer);
    let cfg = Config::from_overrides(&overrides);
    let mut auth = Auth::new_with_store_path_and_overrides(&cfg, path, &overrides);
    auth.with_app_name(app);
    Client::new(&cfg, auth).expect("client builds")
}

fn request() -> RequestOptions {
    RequestOptions {
        method: "GET".to_string(),
        target: RequestTarget::Template {
            path: "/2/users/me".to_string(),
            path_params: HashMap::new(),
            query: Vec::new(),
        },
        ..RequestOptions::default()
    }
}

async fn requests(server: &MockServer) -> usize {
    server
        .received_requests()
        .await
        .expect("the server records requests")
        .len()
}

fn assert_refuses_the_app(error: &Error, name: &str) {
    assert_eq!(error.kind(), "token-store", "{error:?}");
    assert_eq!(error.exit_code(), 77, "{error:?}");
    assert_eq!(error.next_action(), Some(NextAction::FixInput), "{error:?}");
    assert_eq!(error.docs_url(), None, "{error:?}");
    assert_eq!(
        error.to_string(),
        format!("the store holds no app named {name:?}")
    );
}

#[rstest::rstest]
#[case::no_env_bearer(None)]
#[case::env_bearer(Some("ENV-BEARER"))]
#[tokio::test]
async fn a_request_under_an_app_the_store_does_not_hold_is_refused_before_it_is_sent(
    #[case] env_bearer: Option<&str>,
) {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path);
    let client = client(&path, &server, GHOST, env_bearer);

    let preflight = client
        .get_me()
        .auth_preflight()
        .await
        .expect_err("a preflight under a missing app is refused");
    assert_refuses_the_app(&preflight, GHOST);

    let header = client
        .get_auth_header_public(&request())
        .await
        .expect_err("a header under a missing app is refused");
    assert_refuses_the_app(&header, GHOST);

    let sent = client
        .get_me()
        .send()
        .await
        .expect_err("a send under a missing app is refused");
    assert_refuses_the_app(&sent, GHOST);

    assert_eq!(requests(&server).await, 0, "a request went out");
}

#[tokio::test]
async fn the_empty_name_and_a_held_app_are_served_as_they_always_were() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path);

    for app in ["", APP] {
        let found = client(&path, &server, app, None)
            .get_me()
            .auth_preflight()
            .await
            .expect("the default app's token serves the call")
            .expect("the client attaches a credential");
        assert_eq!(found.scheme, WireScheme::OAuth2, "app {app:?}");
        assert_eq!(found.app.as_deref(), Some(APP), "app {app:?}");
    }

    let error = client(&path, &server, OTHER, None)
        .get_me()
        .auth_preflight()
        .await
        .expect_err("a held app with no credential serves nothing");
    assert_ne!(error.kind(), "token-store", "{error:?}");
    assert_eq!(requests(&server).await, 0);
}

#[tokio::test]
async fn an_app_the_store_does_not_hold_reads_nothing_from_the_default_app() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path);
    let store = TokenStore::new_with_path(&path.to_string_lossy());

    assert!(store.get_oauth2_token_for_app(GHOST, USER).is_none());
    assert!(store.get_first_oauth2_token_for_app(GHOST).is_none());
    assert!(store.get_oauth2_token_unnamed_for_app(GHOST).is_none());
    assert!(store.get_oauth1_tokens_for_app(GHOST).is_none());
    assert!(store.get_bearer_token_for_app(GHOST).is_none());
    assert_eq!(store.get_default_user(GHOST), "");
    assert_eq!(store.get_app_redirect_uri(GHOST), None);
    assert!(store.resolve_app(GHOST).client_id.is_empty());
    assert!(store.resolve_app("").oauth1_token.is_some());

    let mut cfg = Config::from_overrides(&overrides(&server, None));
    cfg.app_name = GHOST.to_string();
    let auth = Auth::new_with_store_path_and_overrides(&cfg, &path, &overrides(&server, None));
    assert_eq!(auth.client_id(), "", "the default app's client id");
    assert_eq!(auth.client_secret(), "", "the default app's client secret");
    assert!(!auth.has_oauth2_token(USER));
    assert!(!auth.has_oauth2_token(""));
    assert!(auth.unexpired_oauth2_access_token(USER).is_none());
    assert!(
        auth.get_oauth1_header("GET", "https://x.test/", None)
            .is_err()
    );
}

#[tokio::test]
async fn a_write_naming_an_app_the_store_does_not_hold_changes_nothing() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path);
    let before = std::fs::read(&path).expect("read the store");
    let mut store = TokenStore::new_with_path(&path.to_string_lossy());

    let error = store
        .set_default_user(GHOST, USER)
        .expect_err("the default user of a missing app is not set");
    assert_refuses_the_app(&error, GHOST);

    let oauth1 = store.set_oauth1_account_for_app(GHOST, "1234", Some("handle"), Some("Name"));
    assert!(!matches!(oauth1, Ok(true)), "{oauth1:?}");
    let oauth2 =
        store.set_oauth2_account_for_app(GHOST, USER, "1234", Some("handle"), Some("Name"));
    assert!(!matches!(oauth2, Ok(true)), "{oauth2:?}");

    assert_eq!(
        std::fs::read(&path).expect("read the store"),
        before,
        "a write for a missing app reached the store"
    );
}

fn auth_under(path: &Path, app: &str) -> Auth {
    let overrides = EnvOverrides::default();
    let cfg = Config::from_overrides(&overrides);
    let mut auth = Auth::new_with_store_path_and_overrides(&cfg, path, &overrides);
    auth.with_app_name(app);
    auth
}

#[test]
fn a_named_login_is_looked_up_in_the_app_the_auth_names() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path);

    assert!(
        !auth_under(&path, OTHER).has_oauth2_token(USER),
        "the default app's login served an app that holds none"
    );

    let mut store = TokenStore::new_with_path(&path.to_string_lossy());
    store
        .save_oauth2_token_for_app(OTHER, "other-user", "ACCESS", "REFRESH", 9_999_999_999)
        .expect("save the other app's login");
    let other = auth_under(&path, OTHER);
    assert!(other.has_oauth2_token("other-user"));
    assert!(
        other.has_oauth2_token(USER),
        "a named caller falls back to the app's first login"
    );
    assert!(auth_under(&path, APP).has_oauth2_token(USER));
    assert!(auth_under(&path, "").has_oauth2_token(USER));
}
