//! Token-store and pending-auth files in the layout `serde_yaml` 0.9 writes
//! (plain `yes` scalars and keys, single-quoted numeric strings, long tokens on
//! one line) load with every value intact. Both fixtures were written by
//! `serde_yaml` from the store and pending-state types.

use std::fs;

use xdk::auth::pending;
use xdk::store::TokenStore;

const STORE: &str = include_str!("fixtures/yaml/store-serde-yaml.yml");
const PENDING: &str = include_str!("fixtures/yaml/pending-serde-yaml.yml");

const LONG_BEARER: &str = "AAAAAAAAAAAAAAAAAAAAAMLheAAAAAAA0%2BuSeid%2BULvsea4JtiGRiSDSJSI%3DEUifiRBkKG5E2XzMDjRfl76ZC9Ub0wnz4XsNiRVBChTYbJcE3F";

fn assert_fixture_store(store: &TokenStore) {
    assert!(!store.load_failed(), "the store parsed");
    assert_eq!(store.get_default_app(), "second");
    let app = store.get_app("second").expect("app `second`");
    assert_eq!(app.client_id, "Q2xpZW50SWQtZXhhbXBsZS0wMDAx");
    assert_eq!(app.client_secret, "yes", "a plain `yes` stays a string");
    assert_eq!(app.default_user, "yes");
    assert_eq!(app.redirect_uri, "http://127.0.0.1:8739/callback");

    let named = store
        .get_oauth2_token_for_app("second", "yes")
        .and_then(|t| t.oauth2.as_ref())
        .expect("the OAuth2 token under the key `yes`");
    assert_eq!(
        named.access_token,
        "VGhpcy1pcy1hLWxvbmctZXhhbXBsZS1hY2Nlc3MtdG9rZW4tdGhhdC1ydW5zLXBhc3QtZWlnaHR5LWNoYXJhY3RlcnMtMDAwMQ"
    );
    assert_eq!(named.refresh_token, "cmVmcmVzaC10b2tlbi1leGFtcGxlLTAwMDE");
    assert_eq!(named.expiration_time, 1_893_456_000);
    assert!(
        store
            .get_oauth2_token_for_app("second", "example_user")
            .is_some()
    );

    let oauth1 = store
        .get_oauth1_tokens_for_app("second")
        .and_then(|t| t.oauth1.as_ref())
        .expect("the OAuth1 token");
    assert_eq!(oauth1.token_secret, "token%2Bsecret%3D");
    assert_eq!(oauth1.consumer_secret, "consumer secret with spaces");

    let bearer = store
        .get_bearer_token_for_app("second")
        .and_then(|t| t.bearer.as_deref())
        .expect("the bearer token");
    assert_eq!(bearer, LONG_BEARER);
}

#[test]
fn a_store_in_the_serde_yaml_layout_loads_every_value() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.yml");
    fs::write(&path, STORE).unwrap();

    let store = TokenStore::load_from_path(&path.to_string_lossy());

    assert_fixture_store(&store);
}

#[test]
fn a_rewritten_store_keeps_every_value_and_long_tokens_on_one_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.yml");
    fs::write(&path, STORE).unwrap();
    let path = path.to_string_lossy().into_owned();

    let mut store = TokenStore::load_from_path(&path);
    store.set_default_user("second", "yes").unwrap();
    let written = fs::read_to_string(&path).unwrap();

    assert!(
        written
            .lines()
            .any(|line| line.trim() == format!("bearer: {LONG_BEARER}")),
        "the bearer token is one plain line:\n{written}"
    );
    assert_fixture_store(&TokenStore::load_from_path(&path));
}

#[test]
fn a_pending_state_in_the_serde_yaml_layout_loads() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pending.yml");
    // `load` enforces a 15-minute window, so the fixture's timestamp becomes
    // now; every other line is as `serde_yaml` wrote it.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let body = PENDING.replace("created_at: 1893456000", &format!("created_at: {now}"));
    fs::write(&path, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    }

    let state = pending::load(&path).expect("the pending state loads");

    assert_eq!(
        state.code_verifier,
        "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
    );
    assert_eq!(state.state, "af0ifjsldkj");
    assert_eq!(
        state.client_id, "1234567890123456789",
        "a quoted number stays a string"
    );
    assert_eq!(state.app_name, "default");
    assert_eq!(state.created_at, now);
    assert_eq!(state.scopes, ["tweet.read", "users.read", "offline.access"]);
}
