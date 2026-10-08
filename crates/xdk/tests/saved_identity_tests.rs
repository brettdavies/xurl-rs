//! What the store keeps of the account behind a login, so that the request
//! that asks X who it is goes out once per login, not once per command.
//!
//! The account id is stored beside the credential `/2/users/me` was asked
//! with, and nowhere else, so it cannot name another account. A login stored
//! without a username gets its name from the same answer.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::auth_matrix::WireScheme;
use xdk::auth::Auth;
use xdk::config::{Config, EnvOverrides};
use xdk::store::TokenStore;

const APP: &str = "identity-app";

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

/// A store at `path` holding one app, filled by `fill`.
fn seed(path: &Path, fill: impl FnOnce(&mut TokenStore)) {
    let mut store = TokenStore::new_with_path(&path.to_string_lossy());
    store
        .add_app(APP, "CLIENT-ID", "CLIENT-SECRET")
        .expect("add app");
    fill(&mut store);
}

fn unnamed(store: &mut TokenStore, expired: bool) {
    store
        .save_oauth2_token_unnamed_for_app(APP, "UNNAMED-ACCESS", "REFRESH", expiry(expired))
        .expect("save the unnamed token");
}

fn named(store: &mut TokenStore, name: &str, expired: bool) {
    store
        .save_oauth2_token_for_app(APP, name, "NAMED-ACCESS", "REFRESH", expiry(expired))
        .expect("save the named token");
}

fn oauth1(store: &mut TokenStore) {
    store
        .save_oauth1_tokens_for_app(APP, "ACCESS", "SECRET", "KEY", "CONSUMER")
        .expect("save oauth1");
}

/// A server that answers as the account `alice`, id 42: `/2/users/me`, the
/// identity lookup a sign-in or a refresh makes, a lookup by handle, and a
/// refresh grant.
async fn x_answering_as_alice() -> MockServer {
    let server = MockServer::start().await;
    let alice = serde_json::json!({"data": {"id": "42", "name": "Alice", "username": "alice"}});
    for route in ["/2/users/me", "/me", "/2/users/by/username/alice"] {
        Mock::given(method("GET"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200).set_body_json(alice.clone()))
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "REFRESHED-ACCESS",
            "refresh_token": "REFRESHED-REFRESH",
            "expires_in": 7200
        })))
        .mount(&server)
        .await;
    server
}

fn client(store: &Path, server: &MockServer) -> Client {
    let overrides = EnvOverrides {
        api_base_url: Some(server.uri()),
        auth_url: Some(format!("{}/authorize", server.uri())),
        token_url: Some(format!("{}/token", server.uri())),
        info_url: Some(format!("{}/me", server.uri())),
        ..EnvOverrides::default()
    };
    let cfg = Config::from_overrides(&overrides);
    let mut auth = Auth::new_with_store_path_and_overrides(&cfg, store, &overrides);
    auth.with_app_name(APP);
    Client::new(&cfg, auth).expect("client builds")
}

fn reloaded(store: &Path) -> TokenStore {
    TokenStore::new_with_path(&store.to_string_lossy())
}

/// The paths the server was asked for, in order.
async fn asked(server: &MockServer) -> Vec<String> {
    server
        .received_requests()
        .await
        .expect("the server records requests")
        .iter()
        .map(|request| format!("{} {}", request.method, request.url.path()))
        .collect()
}

fn oauth2_id(store: &TokenStore, name: &str) -> Option<String> {
    store
        .get_oauth2_token_for_app(APP, name)
        .and_then(|token| token.oauth2.as_ref())
        .and_then(|token| token.user_id.clone())
}

#[tokio::test]
async fn a_me_lookup_under_an_unnamed_login_stores_its_name_and_id() {
    let server = x_answering_as_alice().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| unnamed(store, false));
    let client = client(&store, &server);

    let me = client
        .get_me()
        .send_saving_identity()
        .await
        .expect("X answers");
    assert_eq!(me.data.username, "alice");

    let after = reloaded(&store);
    let token = after
        .get_oauth2_token_for_app(APP, "alice")
        .and_then(|token| token.oauth2.as_ref())
        .expect("the token is stored under alice");
    assert_eq!(token.access_token, "UNNAMED-ACCESS");
    assert_eq!(token.user_id.as_deref(), Some("42"));
    assert!(
        after.get_oauth2_token_unnamed_for_app(APP).is_none(),
        "the unnamed slot is empty once the token has a name"
    );
    let found = client
        .get_me()
        .auth_preflight()
        .await
        .expect("the login serves the call")
        .expect("the client attaches a credential");
    assert_eq!(found.username.as_deref(), Some("alice"));
    assert_eq!(found.user_id.as_deref(), Some("42"));
}

/// The key a login is stored under is the caller's label for it, which
/// `xr auth oauth2 NAME` lets them choose. The answer adds the id and
/// leaves the label alone.
#[tokio::test]
async fn a_me_lookup_under_a_named_login_stores_the_id_and_keeps_the_label() {
    let server = x_answering_as_alice().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| named(store, "work", false));

    client(&store, &server)
        .get_me()
        .send_saving_identity()
        .await
        .expect("X answers");

    let after = reloaded(&store);
    assert_eq!(oauth2_id(&after, "work").as_deref(), Some("42"));
    assert!(after.get_oauth2_token_for_app(APP, "alice").is_none());
}

#[tokio::test]
async fn a_me_lookup_under_oauth1_stores_the_id_beside_the_access_pair() {
    let server = x_answering_as_alice().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| {
        oauth1(store);
        unnamed(store, false);
    });
    let client = client(&store, &server);

    client
        .get_me()
        .auth(WireScheme::OAuth1)
        .send_saving_identity()
        .await
        .expect("X answers the OAuth1 request");

    let after = reloaded(&store);
    let pair = after
        .get_oauth1_tokens_for_app(APP)
        .and_then(|token| token.oauth1.as_ref())
        .expect("the access pair is stored");
    assert_eq!(pair.user_id.as_deref(), Some("42"));
    assert!(
        after.get_oauth2_token_unnamed_for_app(APP).is_some(),
        "an answer under OAuth1 says nothing about the OAuth2 token"
    );
    let found = client
        .get_me()
        .auth(WireScheme::OAuth1)
        .auth_preflight()
        .await
        .expect("the pair serves the call")
        .expect("the client attaches a credential");
    assert_eq!(found.user_id.as_deref(), Some("42"));
}

/// A new access pair may be another account's, so it starts without an id.
#[tokio::test]
async fn storing_a_new_credential_drops_the_id_of_the_one_it_replaces() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, |store| {
        oauth1(store);
        named(store, "alice", false);
    });
    let mut store = reloaded(&path);
    assert!(store.set_oauth1_user_id_for_app(APP, "42").expect("store"));
    assert!(
        store
            .set_oauth2_user_id_for_app(APP, "alice", "42")
            .expect("store")
    );

    oauth1(&mut store);
    named(&mut store, "alice", false);

    let after = reloaded(&path);
    let pair = after
        .get_oauth1_tokens_for_app(APP)
        .and_then(|token| token.oauth1.as_ref())
        .expect("the access pair is stored");
    assert_eq!(pair.user_id, None);
    assert_eq!(oauth2_id(&after, "alice"), None);
}

/// A lookup by handle answers about whoever holds the handle now, which is
/// not a statement about the token it was sent with.
#[tokio::test]
async fn a_lookup_by_handle_stores_nothing() {
    let server = x_answering_as_alice().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| named(store, "work", false));
    let before = std::fs::read(&store).expect("read the store");

    client(&store, &server)
        .lookup_user("alice")
        .send_saving_identity()
        .await
        .expect("X answers the lookup");

    assert_eq!(std::fs::read(&store).expect("read the store"), before);
}

/// A token already stored under the username is the newer one, so it stays.
#[tokio::test]
async fn a_username_that_already_holds_a_token_keeps_it() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, |store| {
        unnamed(store, false);
        named(store, "alice", false);
    });
    let mut store = reloaded(&path);

    let moved = store
        .name_unnamed_oauth2_token_for_app(APP, "alice")
        .expect("the store is readable");

    assert!(!moved);
    let token = store
        .get_oauth2_token_for_app(APP, "alice")
        .and_then(|token| token.oauth2.as_ref())
        .expect("alice keeps a token");
    assert_eq!(token.access_token, "NAMED-ACCESS");
}

/// A refresh mints the new pair from the old one, so it replaces it where it
/// is stored. The lookup it makes is for what the store lacks: here the id.
#[tokio::test]
async fn a_refresh_asks_once_for_an_id_it_lacks_and_keeps_the_login_in_place() {
    let server = x_answering_as_alice().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| named(store, "work", true));
    let client = client(&store, &server);

    client
        .get_me()
        .send()
        .await
        .expect("the refreshed token is sent");

    let after = reloaded(&store);
    let token = after
        .get_oauth2_token_for_app(APP, "work")
        .and_then(|token| token.oauth2.as_ref())
        .expect("the login is still stored as work");
    assert_eq!(token.access_token, "REFRESHED-ACCESS");
    assert_eq!(token.user_id.as_deref(), Some("42"));
    assert!(
        after.get_oauth2_token_for_app(APP, "alice").is_none(),
        "no second entry appears under the name X answered with"
    );
    assert_eq!(
        asked(&server).await,
        ["POST /token", "GET /me", "GET /2/users/me"]
    );
}

/// With a name and an id stored, X has nothing to add, so the refresh sends
/// the grant and no lookup.
#[tokio::test]
async fn a_refresh_of_a_login_with_a_name_and_an_id_makes_no_lookup() {
    let server = x_answering_as_alice().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| {
        named(store, "alice", true);
        store
            .set_oauth2_user_id_for_app(APP, "alice", "42")
            .expect("store the id");
    });

    client(&store, &server)
        .search_posts("rust", 10)
        .send()
        .await
        .expect_err("the server has no search route; the refresh is what is under test");

    assert_eq!(
        asked(&server).await,
        ["POST /token", "GET /2/tweets/search/recent"]
    );
    let after = reloaded(&store);
    let token = after
        .get_oauth2_token_for_app(APP, "alice")
        .and_then(|token| token.oauth2.as_ref())
        .expect("the login is stored");
    assert_eq!(token.access_token, "REFRESHED-ACCESS");
    assert_eq!(
        token.user_id.as_deref(),
        Some("42"),
        "the id outlives the refresh"
    );
}

#[tokio::test]
async fn a_refresh_of_an_unnamed_login_names_it_and_empties_the_slot() {
    let server = x_answering_as_alice().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| unnamed(store, true));

    client(&store, &server)
        .get_me()
        .send()
        .await
        .expect("the refreshed token is sent");

    let after = reloaded(&store);
    let token = after
        .get_oauth2_token_for_app(APP, "alice")
        .and_then(|token| token.oauth2.as_ref())
        .expect("the login is stored under the name X answered with");
    assert_eq!(token.access_token, "REFRESHED-ACCESS");
    assert_eq!(token.user_id.as_deref(), Some("42"));
    assert!(
        after.get_oauth2_token_unnamed_for_app(APP).is_none(),
        "the pair the refresh replaced does not stay behind"
    );
}
