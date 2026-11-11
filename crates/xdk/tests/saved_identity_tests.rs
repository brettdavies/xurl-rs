//! What the store keeps of the account behind a login, so that the request
//! that asks X who it is goes out once per login, not once per command.
//!
//! The account id is stored beside the credential `/2/users/me` was asked
//! with, and nowhere else, so it cannot name another account. The answer's
//! username and display name go beside every credential holding that id. A
//! login stored without a username gets its name from the same answer.

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
    let login = after
        .get_oauth2_token_for_app(APP, "work")
        .and_then(|token| token.oauth2.as_ref())
        .expect("the login is stored as work");
    assert_eq!(login.account_username.as_deref(), Some("alice"));
    assert_eq!(login.display_name.as_deref(), Some("Alice"));
    let found = client(&store, &server)
        .get_me()
        .auth_preflight()
        .await
        .expect("the login serves the call")
        .expect("the client attaches a credential");
    assert_eq!(found.account_username.as_deref(), Some("alice"));
    assert_eq!(found.display_name.as_deref(), Some("Alice"));
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
    assert_eq!(pair.account_username.as_deref(), Some("alice"));
    assert_eq!(pair.display_name.as_deref(), Some("Alice"));
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
    assert_eq!(found.display_name.as_deref(), Some("Alice"));
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
    assert_eq!(token.account_username.as_deref(), Some("alice"));
    assert_eq!(token.display_name.as_deref(), Some("Alice"));
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

/// The username and display name are stored only beside the id they came
/// with: the same id keeps them, and a new id drops them, so a stored name
/// never belongs to another account.
#[test]
fn a_new_id_drops_the_stored_names_and_the_same_id_keeps_them() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, |store| {
        named(store, "work", false);
        oauth1(store);
    });
    let mut store = reloaded(&path);
    let names = |store: &TokenStore| {
        let token = store
            .get_oauth2_token_for_app(APP, "work")
            .and_then(|token| token.oauth2.as_ref())
            .expect("the login is stored");
        (token.account_username.clone(), token.display_name.clone())
    };

    assert!(
        store
            .set_oauth2_account_for_app(APP, "work", "42", Some("alice"), Some("Alice"))
            .expect("store the account")
    );
    assert!(
        !store
            .set_oauth2_account_for_app(APP, "work", "42", Some("alice"), Some("Alice"))
            .expect("the same answer"),
        "the same answer changes nothing"
    );
    assert!(
        !store
            .set_oauth2_user_id_for_app(APP, "work", "42")
            .expect("same id")
    );
    assert_eq!(
        names(&reloaded(&path)),
        (Some("alice".to_string()), Some("Alice".to_string()))
    );

    assert!(
        store
            .set_oauth2_user_id_for_app(APP, "work", "7")
            .expect("a new id")
    );
    assert_eq!(names(&reloaded(&path)), (None, None));

    assert!(
        store
            .set_oauth1_account_for_app(APP, "42", Some("alice"), Some("Alice"))
            .expect("store the pair's account")
    );
    assert!(
        store
            .set_oauth1_user_id_for_app(APP, "7")
            .expect("a new id")
    );
    let pair = reloaded(&path);
    let pair = pair
        .get_oauth1_tokens_for_app(APP)
        .and_then(|token| token.oauth1.as_ref())
        .expect("the pair is stored");
    assert_eq!(pair.user_id.as_deref(), Some("7"));
    assert_eq!(pair.account_username, None);
    assert_eq!(pair.display_name, None);
}

// ── An account's names follow its newest answer ────────────────────────

const OTHER_APP: &str = "other-app";

/// A second app holding a login `alice` for the account 42, recorded with
/// the names it had before a rename.
fn other_app_with_alice(store: &mut TokenStore) {
    store
        .add_app(OTHER_APP, "OTHER-CLIENT", "OTHER-SECRET")
        .expect("add the other app");
    store
        .save_oauth2_token_for_app(OTHER_APP, "alice", "OTHER-ACCESS", "REFRESH", expiry(false))
        .expect("save the other app's login");
    store
        .set_oauth2_account_for_app(OTHER_APP, "alice", "42", Some("alice"), Some("Alice Old"))
        .expect("record the old names");
}

fn names_in(store: &TokenStore, app: &str, login: &str) -> (Option<String>, Option<String>) {
    let token = store
        .get_oauth2_token_for_app(app, login)
        .and_then(|token| token.oauth2.as_ref())
        .unwrap_or_else(|| panic!("{app} holds {login}"));
    (token.account_username.clone(), token.display_name.clone())
}

fn renamed() -> (Option<String>, Option<String>) {
    (Some("alice2".to_string()), Some("Alice New".to_string()))
}

/// An answer for an account names it on every credential that carries its
/// id, in every app: filled where the names were missing, replaced where
/// they were stale. A credential with another id, or with none, keeps what
/// it had, since only an answer under it says whose it is.
#[test]
fn an_answer_names_its_account_on_every_credential_that_carries_its_id() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("auth.yml");
    seed(&path, |store| {
        named(store, "work", false);
        named(store, "unasked", false);
        other_app_with_alice(store);
        store
            .save_oauth2_token_for_app(OTHER_APP, "bob", "BOB-ACCESS", "REFRESH", expiry(false))
            .expect("save bob");
        store
            .set_oauth2_account_for_app(OTHER_APP, "bob", "7", Some("bob"), Some("Bob"))
            .expect("record bob");
        store
            .save_oauth1_tokens_for_app(OTHER_APP, "ACCESS", "SECRET", "KEY", "CONSUMER")
            .expect("save the pair");
        store
            .set_oauth1_user_id_for_app(OTHER_APP, "42")
            .expect("record the pair's id");
    });
    let mut store = reloaded(&path);

    assert!(
        store
            .set_oauth2_account_for_app(APP, "work", "42", Some("alice2"), Some("Alice New"))
            .expect("record the answer")
    );

    let after = reloaded(&path);
    assert_eq!(names_in(&after, APP, "work"), renamed());
    assert_eq!(names_in(&after, OTHER_APP, "alice"), renamed());
    let pair = after
        .get_oauth1_tokens_for_app(OTHER_APP)
        .and_then(|token| token.oauth1.as_ref())
        .expect("the pair is stored");
    assert_eq!(
        (pair.account_username.clone(), pair.display_name.clone()),
        renamed()
    );
    assert_eq!(
        names_in(&after, OTHER_APP, "bob"),
        (Some("bob".to_string()), Some("Bob".to_string()))
    );
    let unasked = after
        .get_oauth2_token_for_app(APP, "unasked")
        .and_then(|token| token.oauth2.as_ref())
        .expect("the login is stored");
    assert_eq!(unasked.user_id, None, "an id is never copied");
    assert_eq!(unasked.account_username, None);

    let before = std::fs::read(&path).expect("read the store");
    assert!(
        !store
            .set_oauth2_account_for_app(APP, "work", "42", Some("alice2"), Some("Alice New"))
            .expect("the same answer"),
        "an answer every credential already reflects changes nothing"
    );
    assert_eq!(std::fs::read(&path).expect("read the store"), before);
}

/// Records a newer answer for the account 42 through the other app's login
/// while the refresh grant is in flight, then answers the grant.
struct NewerAnswerDuringTheGrant {
    store: std::path::PathBuf,
}

impl wiremock::Respond for NewerAnswerDuringTheGrant {
    fn respond(&self, _: &wiremock::Request) -> ResponseTemplate {
        reloaded(&self.store)
            .set_oauth2_account_for_app(OTHER_APP, "alice", "42", Some("alice2"), Some("Alice New"))
            .expect("a newer answer");
        ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "REFRESHED-ACCESS",
            "refresh_token": "REFRESHED-REFRESH",
            "expires_in": 7200
        }))
    }
}

/// A refresh that asks X nothing carries the names of the token it read
/// before its grant went out. A newer answer recorded while the grant was
/// in flight, here through another app's login for the same account,
/// stands: the refresh neither writes the old names back nor shares them.
#[tokio::test]
async fn a_refresh_that_asks_nothing_keeps_an_answer_recorded_during_its_grant() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| {
        named(store, "work", true);
        store
            .set_oauth2_account_for_app(APP, "work", "42", Some("alice"), Some("Alice Old"))
            .expect("record the old names");
        other_app_with_alice(store);
    });
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(NewerAnswerDuringTheGrant {
            store: store.clone(),
        })
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/users/by/username/alice"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"data": {"id": "42", "name": "Alice", "username": "alice"}}),
        ))
        .mount(&server)
        .await;

    client(&store, &server)
        .lookup_user("alice")
        .send()
        .await
        .expect("the refreshed token is sent");

    assert_eq!(
        asked(&server).await,
        ["POST /token", "GET /2/users/by/username/alice"]
    );
    let after = reloaded(&store);
    let token = after
        .get_oauth2_token_for_app(APP, "work")
        .and_then(|token| token.oauth2.as_ref())
        .expect("the login is stored");
    assert_eq!(token.access_token, "REFRESHED-ACCESS");
    assert_eq!(names_in(&after, APP, "work"), renamed());
    assert_eq!(names_in(&after, OTHER_APP, "alice"), renamed());
}

/// A refresh that asks X who the token belongs to has a fresh answer, and
/// names the account with it in every app.
#[tokio::test]
async fn a_refresh_that_asks_names_the_account_in_every_app() {
    let server = x_answering_as_alice().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| {
        unnamed(store, true);
        other_app_with_alice(store);
    });

    client(&store, &server)
        .lookup_user("alice")
        .send()
        .await
        .expect("the refreshed token is sent");

    let after = reloaded(&store);
    let answered = (Some("alice".to_string()), Some("Alice".to_string()));
    assert_eq!(names_in(&after, APP, "alice"), answered);
    assert_eq!(names_in(&after, OTHER_APP, "alice"), answered);
}

/// Changes the store as another process would while a request is out, then
/// answers it: the change lands before the answer, which orders the race.
struct ChangeThenAnswer {
    store: std::path::PathBuf,
    change: fn(&Path),
    body: serde_json::Value,
}

impl wiremock::Respond for ChangeThenAnswer {
    fn respond(&self, _: &wiremock::Request) -> ResponseTemplate {
        (self.change)(&self.store);
        ResponseTemplate::new(200).set_body_json(self.body.clone())
    }
}

fn alice_answer() -> serde_json::Value {
    serde_json::json!({"data": {"id": "42", "name": "Alice", "username": "alice"}})
}

fn refreshed_pair() -> serde_json::Value {
    serde_json::json!({
        "access_token": "REFRESHED-ACCESS",
        "refresh_token": "REFRESHED-REFRESH",
        "expires_in": 7200
    })
}

/// Another process signs bob, account 99, in under the label `work`.
fn sign_bob_in_as_work(store: &Path) {
    let mut store = reloaded(store);
    store
        .save_oauth2_token_for_app(APP, "work", "BOB-ACCESS", "BOB-REFRESH", expiry(false))
        .expect("bob signs in");
    store
        .set_oauth2_account_for_app(APP, "work", "99", Some("bob"), Some("Bob"))
        .expect("bob's account");
}

async fn answering(route: (&str, &str), responder: ChangeThenAnswer) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method(route.0))
        .and(path(route.1))
        .respond_with(responder)
        .mount(&server)
        .await;
    server
}

/// `xr whoami` under `work` while bob signs in under that label: the
/// answer is alice's and says nothing about bob's token.
#[tokio::test]
async fn a_me_answer_is_not_recorded_on_a_login_stored_under_its_label_meanwhile() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| named(store, "work", false));
    let server = answering(
        ("GET", "/2/users/me"),
        ChangeThenAnswer {
            store: store.clone(),
            change: sign_bob_in_as_work,
            body: alice_answer(),
        },
    )
    .await;

    client(&store, &server)
        .get_me()
        .username("work")
        .send_saving_identity()
        .await
        .expect("X answers");

    let after = reloaded(&store);
    let token = after
        .get_oauth2_token_for_app(APP, "work")
        .and_then(|token| token.oauth2.as_ref())
        .expect("bob's login is stored");
    assert_eq!(token.access_token, "BOB-ACCESS");
    assert_eq!(
        (token.user_id.as_deref(), token.account_username.as_deref()),
        (Some("99"), Some("bob")),
        "bob's token is labelled with the account the answer named"
    );
}

/// The same race through the unnamed slot: a token another process stores
/// there while the request is out is not moved under the answer's name.
#[tokio::test]
async fn a_me_answer_does_not_name_an_unnamed_login_stored_meanwhile() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| unnamed(store, false));
    let server = answering(
        ("GET", "/2/users/me"),
        ChangeThenAnswer {
            store: store.clone(),
            change: |store| {
                reloaded(store)
                    .save_oauth2_token_unnamed_for_app(
                        APP,
                        "BOB-ACCESS",
                        "BOB-REFRESH",
                        expiry(false),
                    )
                    .expect("bob's unnamed login");
            },
            body: alice_answer(),
        },
    )
    .await;

    client(&store, &server)
        .get_me()
        .send_saving_identity()
        .await
        .expect("X answers");

    let after = reloaded(&store);
    assert!(
        after.get_oauth2_token_for_app(APP, "alice").is_none(),
        "bob's token was stored under alice"
    );
    let unnamed = after
        .get_oauth2_token_unnamed_for_app(APP)
        .and_then(|token| token.oauth2.as_ref())
        .expect("bob's token stays unnamed");
    assert_eq!(
        (unnamed.access_token.as_str(), unnamed.user_id.as_deref()),
        ("BOB-ACCESS", None)
    );
}

/// The same race under OAuth1: an access pair stored while the request is
/// out does not take the answer's account.
#[tokio::test]
async fn a_me_answer_under_oauth1_is_not_recorded_on_a_pair_stored_meanwhile() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, oauth1);
    let server = answering(
        ("GET", "/2/users/me"),
        ChangeThenAnswer {
            store: store.clone(),
            change: |store| {
                reloaded(store)
                    .save_oauth1_tokens_for_app(APP, "BOB-ACCESS", "BOB-SECRET", "KEY", "CONSUMER")
                    .expect("bob's access pair");
            },
            body: alice_answer(),
        },
    )
    .await;

    client(&store, &server)
        .get_me()
        .auth(WireScheme::OAuth1)
        .send_saving_identity()
        .await
        .expect("X answers");

    let after = reloaded(&store);
    let pair = after
        .get_oauth1_tokens_for_app(APP)
        .and_then(|token| token.oauth1.as_ref())
        .expect("bob's pair is stored");
    assert_eq!(
        (pair.access_token.as_str(), pair.user_id.as_deref()),
        ("BOB-ACCESS", None),
        "bob's pair took the account the answer named"
    );
}

/// A refresh of alice's expired `work` login while bob signs in under that
/// label: bob's login stands, and the refreshed pair is not stored.
#[tokio::test]
async fn a_refresh_leaves_a_login_signed_in_during_its_grant() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| {
        named(store, "work", true);
        store
            .set_oauth2_account_for_app(APP, "work", "42", Some("alice"), Some("Alice"))
            .expect("alice's account");
    });
    let server = answering(
        ("POST", "/token"),
        ChangeThenAnswer {
            store: store.clone(),
            change: sign_bob_in_as_work,
            body: refreshed_pair(),
        },
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(alice_answer()))
        .mount(&server)
        .await;

    client(&store, &server)
        .get_me()
        .username("work")
        .send()
        .await
        .expect("the refreshed token is sent");

    let after = reloaded(&store);
    let token = after
        .get_oauth2_token_for_app(APP, "work")
        .and_then(|token| token.oauth2.as_ref())
        .expect("a login under work");
    assert_eq!(
        (token.access_token.as_str(), token.user_id.as_deref()),
        ("BOB-ACCESS", Some("99")),
        "the refresh put alice back under work"
    );
}

/// A refresh of the expired `work` login while another process clears it:
/// the login stays cleared.
#[tokio::test]
async fn a_refresh_leaves_a_login_cleared_during_its_grant_cleared() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| {
        named(store, "work", true);
        store
            .set_oauth2_account_for_app(APP, "work", "42", Some("alice"), Some("Alice"))
            .expect("alice's account");
    });
    let server = answering(
        ("POST", "/token"),
        ChangeThenAnswer {
            store: store.clone(),
            change: |store| {
                reloaded(store)
                    .clear_oauth2_token_for_app(APP, "work")
                    .expect("clear work");
            },
            body: refreshed_pair(),
        },
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(alice_answer()))
        .mount(&server)
        .await;

    client(&store, &server)
        .get_me()
        .username("work")
        .send()
        .await
        .expect("the refreshed token is sent");

    assert!(
        reloaded(&store)
            .get_oauth2_token_for_app(APP, "work")
            .is_none(),
        "the cleared login came back"
    );
}

/// A refresh of the unnamed login while another process signs bob in under
/// `alice`, the name X answers the refresh with: bob's login stands, and the
/// refreshed pair stays unnamed.
#[tokio::test]
async fn a_refresh_does_not_name_an_unnamed_login_over_one_signed_in_under_that_name() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| unnamed(store, true));
    let server = answering(
        ("POST", "/token"),
        ChangeThenAnswer {
            store: store.clone(),
            change: |store| {
                let mut store = reloaded(store);
                store
                    .save_oauth2_token_for_app(
                        APP,
                        "alice",
                        "BOB-ACCESS",
                        "BOB-REFRESH",
                        expiry(false),
                    )
                    .expect("bob signs in");
                store
                    .set_oauth2_account_for_app(APP, "alice", "99", Some("bob"), Some("Bob"))
                    .expect("bob's account");
            },
            body: refreshed_pair(),
        },
    )
    .await;
    for route in ["/me", "/2/users/me"] {
        Mock::given(method("GET"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200).set_body_json(alice_answer()))
            .mount(&server)
            .await;
    }

    client(&store, &server)
        .get_me()
        .send()
        .await
        .expect("the refreshed token is sent");

    let after = reloaded(&store);
    let under_alice = after
        .get_oauth2_token_for_app(APP, "alice")
        .and_then(|token| token.oauth2.as_ref())
        .expect("a login under alice");
    assert_eq!(
        (
            under_alice.access_token.as_str(),
            under_alice.user_id.as_deref()
        ),
        ("BOB-ACCESS", Some("99")),
        "the refresh replaced the login signed in under alice"
    );
    let kept = after
        .get_oauth2_token_unnamed_for_app(APP)
        .and_then(|token| token.oauth2.as_ref())
        .expect("the refreshed pair is kept");
    assert_eq!(kept.access_token, "REFRESHED-ACCESS");
}

/// A refresh of the unnamed login while another process's `/2/users/me`
/// answer under the same login stores it under `alice`: the refreshed pair
/// replaces it there, so the refresh token the grant spent is not left
/// behind.
#[tokio::test]
async fn a_refresh_follows_an_unnamed_login_named_during_its_grant() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| unnamed(store, true));
    let server = answering(
        ("POST", "/token"),
        ChangeThenAnswer {
            store: store.clone(),
            change: |store| {
                reloaded(store)
                    .name_unnamed_oauth2_token_for_app(APP, "alice")
                    .expect("the answer names the login");
            },
            body: refreshed_pair(),
        },
    )
    .await;
    for route in ["/me", "/2/users/me"] {
        Mock::given(method("GET"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200).set_body_json(alice_answer()))
            .mount(&server)
            .await;
    }

    client(&store, &server)
        .get_me()
        .send()
        .await
        .expect("the refreshed token is sent");

    let after = reloaded(&store);
    let token = after
        .get_oauth2_token_for_app(APP, "alice")
        .and_then(|token| token.oauth2.as_ref())
        .expect("a login under alice");
    assert_eq!(
        (token.access_token.as_str(), token.refresh_token.as_str()),
        ("REFRESHED-ACCESS", "REFRESHED-REFRESH"),
        "the login under alice holds the refresh token the grant spent"
    );
}

/// A refresh of the unnamed login while another process stores bob's login
/// in the unnamed slot: bob's login stands, and nothing is stored under the
/// name the refresh learned.
#[tokio::test]
async fn a_refresh_of_the_unnamed_login_leaves_one_stored_there_during_its_grant() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| unnamed(store, true));
    let server = answering(
        ("POST", "/token"),
        ChangeThenAnswer {
            store: store.clone(),
            change: |store| {
                reloaded(store)
                    .save_oauth2_token_unnamed_for_app(
                        APP,
                        "BOB-ACCESS",
                        "BOB-REFRESH",
                        expiry(false),
                    )
                    .expect("bob signs in unnamed");
            },
            body: refreshed_pair(),
        },
    )
    .await;
    for route in ["/me", "/2/users/me"] {
        Mock::given(method("GET"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200).set_body_json(alice_answer()))
            .mount(&server)
            .await;
    }

    client(&store, &server)
        .get_me()
        .send()
        .await
        .expect("the refreshed token is sent");

    let after = reloaded(&store);
    let unnamed = after
        .get_oauth2_token_unnamed_for_app(APP)
        .and_then(|token| token.oauth2.as_ref())
        .expect("a login in the unnamed slot");
    assert_eq!(
        unnamed.access_token, "BOB-ACCESS",
        "bob's login was replaced"
    );
    assert!(
        after.get_oauth2_token_for_app(APP, "alice").is_none(),
        "the refreshed pair was stored under the name it learned"
    );
}

/// An expired `work` login without an id, refreshed inside the send: the
/// answer is recorded on the token the request carried, the refreshed one.
#[tokio::test]
async fn a_me_answer_is_recorded_on_the_token_a_refresh_inside_the_send_minted() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join("auth.yml");
    seed(&store, |store| named(store, "work", true));
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(refreshed_pair()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(alice_answer()))
        .mount(&server)
        .await;

    client(&store, &server)
        .get_me()
        .username("work")
        .send_saving_identity()
        .await
        .expect("the refreshed token is sent");

    let after = reloaded(&store);
    let token = after
        .get_oauth2_token_for_app(APP, "work")
        .and_then(|token| token.oauth2.as_ref())
        .expect("a login under work");
    assert_eq!(
        (token.access_token.as_str(), token.user_id.as_deref()),
        ("REFRESHED-ACCESS", Some("42"))
    );
}
