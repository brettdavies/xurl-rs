//! `auth_preflight` reports the credential a send would go out with, and
//! stops there: no request, no refresh, no write to the store.

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tempfile::TempDir;
use wiremock::MockServer;
use xdk::api::auth_matrix::{SHORTCUT_TEMPLATES, WireScheme};
use xdk::api::{Client, RequestOptions, RequestTarget};
use xdk::auth::{Auth, OAuth2Credential};
use xdk::config::{Config, EnvOverrides};
use xdk::error::Error;
use xdk::store::TokenStore;

const APP: &str = "preflight-app";
const USER: &str = "preflight-user";

/// An expiry an hour from now, or one long past.
fn expiry(expired: bool) -> u64 {
    if expired {
        return 1;
    }
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock is past the epoch")
        .as_secs()
        + 3600
}

/// What a test stores under the one app.
type Fill = fn(&mut TokenStore);

/// A store at `path` holding one app, filled by `fill`.
fn seed(path: &Path, fill: impl FnOnce(&mut TokenStore)) {
    let mut store = TokenStore::new_with_path(&path.to_string_lossy());
    store
        .add_app(APP, "CLIENT-ID", "CLIENT-SECRET")
        .expect("add app");
    fill(&mut store);
}

fn oauth2(store: &mut TokenStore, username: &str, expired: bool) {
    store
        .save_oauth2_token_for_app(APP, username, "ACCESS", "REFRESH", expiry(expired))
        .expect("save oauth2");
}

fn oauth1(store: &mut TokenStore) {
    store
        .save_oauth1_tokens_for_app(APP, "ACCESS", "SECRET", "KEY", "CONSUMER")
        .expect("save oauth1");
}

fn bearer(store: &mut TokenStore) {
    store
        .save_bearer_token_for_app(APP, "BEARER")
        .expect("save bearer");
}

/// A client over the store at `path`, with every URL it could reach pointed
/// at `server`, so a request the preflight must not make is one the server
/// records.
fn client(path: &Path, server: &MockServer) -> Client {
    client_with_env_bearer(path, server, None)
}

/// [`client`], with `env_bearer` as the bearer token the environment
/// supplies. The environment is passed in, so the process's own is not read.
fn client_with_env_bearer(path: &Path, server: &MockServer, env_bearer: Option<&str>) -> Client {
    let overrides = EnvOverrides {
        api_base_url: Some(server.uri()),
        auth_url: Some(format!("{}/authorize", server.uri())),
        token_url: Some(format!("{}/token", server.uri())),
        info_url: Some(format!("{}/me", server.uri())),
        bearer_token: env_bearer.map(str::to_string),
        ..EnvOverrides::default()
    };
    let cfg = Config::from_overrides(&overrides);
    let mut auth = Auth::new_with_store_path_and_overrides(&cfg, path, &overrides);
    auth.with_app_name(APP);
    Client::new(&cfg, auth).expect("client builds")
}

fn request(method: &str, path: &str) -> RequestOptions {
    RequestOptions {
        method: method.to_string(),
        target: RequestTarget::Template {
            path: path.to_string(),
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

#[tokio::test]
async fn an_expired_oauth2_token_is_reported_and_not_refreshed() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, |store| oauth2(store, USER, true));
    let before = std::fs::read(&path).expect("read the store");

    let found = client(&path, &server)
        .get_me()
        .auth_preflight()
        .await
        .expect("a stored token serves the call")
        .expect("the client attaches a credential");

    assert_eq!(found.scheme, WireScheme::OAuth2);
    assert_eq!(found.app.as_deref(), Some(APP));
    assert_eq!(found.username.as_deref(), Some(USER));
    assert!(found.token_expired);
    assert_eq!(requests(&server).await, 0, "the preflight sent a request");
    assert_eq!(
        std::fs::read(&path).expect("read the store"),
        before,
        "the preflight wrote to the store"
    );
}

#[tokio::test]
async fn a_live_oauth2_token_names_its_user() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, |store| oauth2(store, USER, false));

    let found = client(&path, &server)
        .get_me()
        .auth_preflight()
        .await
        .expect("a stored token serves the call")
        .expect("the client attaches a credential");

    assert_eq!(found.scheme, WireScheme::OAuth2);
    assert_eq!(found.username.as_deref(), Some(USER));
    assert!(!found.token_expired);
}

#[tokio::test]
async fn a_token_stored_without_a_name_reports_no_username() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, |store| {
        store
            .save_oauth2_token_unnamed_for_app(APP, "ACCESS", "REFRESH", expiry(false))
            .expect("save the unnamed token");
    });

    let found = client(&path, &server)
        .get_me()
        .auth(WireScheme::OAuth2)
        .auth_preflight()
        .await
        .expect("the unnamed token serves the call")
        .expect("the client attaches a credential");

    assert_eq!(found.scheme, WireScheme::OAuth2);
    assert_eq!(found.username, None);
}

#[tokio::test]
async fn a_username_with_no_token_is_refused_as_a_send_would_refuse_it() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, |store| oauth2(store, USER, false));

    let error = client(&path, &server)
        .get_me()
        .username("someone-else")
        .auth_preflight()
        .await
        .expect_err("no token is stored under that name");

    assert_eq!(error.kind(), "auth-required", "{error:?}");
    assert_eq!(requests(&server).await, 0);
}

#[tokio::test]
async fn a_scheme_the_endpoint_does_not_take_is_a_mismatch() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, bearer);

    let error = client(&path, &server)
        .get_me()
        .auth_preflight()
        .await
        .expect_err("GET /2/users/me takes no app-only bearer");

    assert!(
        matches!(error, Error::AuthMethodMismatch(_)),
        "expected a mismatch, got {error:?}"
    );
}

#[tokio::test]
async fn an_empty_store_has_no_credential_to_report() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, |_| {});

    let error = client(&path, &server)
        .get_me()
        .auth_preflight()
        .await
        .expect_err("nothing is stored");

    assert_eq!(error.kind(), "auth-required", "{error:?}");
}

#[tokio::test]
async fn a_request_the_client_attaches_no_credential_to_reports_none() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, |_| {});
    let client = client(&path, &server);

    let unauthenticated = client.get_me().no_auth(true).auth_preflight().await;
    assert_eq!(unauthenticated.expect("no selection runs"), None);

    let own_header = client
        .get_me()
        .header("authorization", "Bearer CALLERS-OWN")
        .auth_preflight()
        .await;
    assert_eq!(own_header.expect("no selection runs"), None);
}

#[tokio::test]
async fn a_client_built_from_credentials_reports_its_token_without_a_name() {
    let server = MockServer::start().await;
    let client = Client::builder()
        .base_url(server.uri())
        .token_url(format!("{}/token", server.uri()))
        .oauth2(OAuth2Credential {
            client_id: "CLIENT-ID".to_string(),
            client_secret: "CLIENT-SECRET".to_string(),
            access_token: "ACCESS".to_string(),
            refresh_token: Some("REFRESH".to_string()),
            expires_at: Some(SystemTime::now() - Duration::from_secs(60)),
        })
        .build()
        .expect("client builds");

    let found = client
        .get_me()
        .auth_preflight()
        .await
        .expect("the credential serves the call")
        .expect("the client attaches a credential");

    assert_eq!(found.scheme, WireScheme::OAuth2);
    assert_eq!(found.app, None);
    assert_eq!(found.username, None);
    assert!(found.token_expired);
    assert_eq!(requests(&server).await, 0, "the preflight sent a request");
}

/// The app named is the one whose stored credential is sent. A bearer token
/// from the environment outranks the stored one and belongs to no stored
/// app, so it names none.
#[tokio::test]
async fn a_stored_bearer_names_its_app_and_an_environment_bearer_names_none() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, bearer);
    let search = |client: Client| async move {
        client
            .search_posts("rust", 10)
            .auth_preflight()
            .await
            .expect("a bearer serves a search")
            .expect("the client attaches a credential")
    };

    let stored = search(client(&path, &server)).await;
    assert_eq!(stored.scheme, WireScheme::App);
    assert_eq!(stored.app.as_deref(), Some(APP));

    let from_env = search(client_with_env_bearer(&path, &server, Some("ENV-BEARER"))).await;
    assert_eq!(from_env.scheme, WireScheme::App);
    assert_eq!(from_env.app, None);

    let header = client_with_env_bearer(&path, &server, Some("ENV-BEARER"))
        .get_auth_header_public(&request("GET", "/2/tweets/search/recent"))
        .await
        .expect("the environment's bearer is the one sent");
    assert_eq!(header, "Bearer ENV-BEARER");
}

/// The preflight and the header a send computes come from one selection, so
/// they agree on every endpoint a shortcut calls, under every scheme a
/// caller can name, over every mix of stored credentials: both succeed on
/// the same scheme, or both fail for the same reason.
#[tokio::test]
async fn the_preflight_agrees_with_the_header_a_send_would_carry() {
    let server = MockServer::start().await;
    let stores: [(&str, Fill); 5] = [
        ("nothing", |_| {}),
        ("oauth2", |store| oauth2(store, USER, false)),
        ("oauth1", oauth1),
        ("bearer", bearer),
        ("all three", |store| {
            oauth2(store, USER, false);
            oauth1(store);
            bearer(store);
        }),
    ];
    let mut compared = 0;
    for (label, fill) in stores {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("auth.yml");
        seed(&path, fill);
        let client = client(&path, &server);
        for (method, endpoint) in SHORTCUT_TEMPLATES {
            for auth_type in ["", "oauth2", "oauth1", "app"] {
                let mut options = request(method, endpoint);
                // A path parameter left unfilled cannot be signed; the
                // matrix reads only the template.
                if let RequestTarget::Template { path_params, .. } = &mut options.target {
                    for name in endpoint
                        .split('{')
                        .skip(1)
                        .filter_map(|s| s.split('}').next())
                    {
                        path_params.insert(name.to_string(), "1".to_string());
                    }
                }
                options.auth_type = auth_type.to_string();
                let case = format!("{method} {endpoint} --auth {auth_type:?} with {label}");

                let header = client.get_auth_header_public(&options).await;
                let found = client.auth_preflight(&options).await;
                match (header, found) {
                    (Ok(header), Ok(Some(found))) => {
                        let sent = match header.as_str() {
                            "Bearer ACCESS" => WireScheme::OAuth2,
                            "Bearer BEARER" => WireScheme::App,
                            signed if signed.starts_with("OAuth ") => WireScheme::OAuth1,
                            other => panic!("{case}: unexpected header {other:?}"),
                        };
                        assert_eq!(found.scheme, sent, "{case}");
                    }
                    (Err(sent), Err(reported)) => {
                        assert_eq!(sent.kind(), reported.kind(), "{case}");
                    }
                    (header, found) => panic!(
                        "{case}: a send {} and the preflight {}",
                        outcome(header.is_ok()),
                        outcome(found.is_ok()),
                    ),
                }
                compared += 1;
            }
        }
    }
    assert_eq!(compared, 5 * SHORTCUT_TEMPLATES.len() * 4);
    assert_eq!(requests(&server).await, 0, "a live token needs no refresh");
}

fn outcome(ok: bool) -> &'static str {
    if ok {
        "finds a credential"
    } else {
        "finds none"
    }
}
