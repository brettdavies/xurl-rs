//! No `Debug` output of a credential-bearing type carries a secret.
//!
//! Every value below is built with one recognizable secret, and its `Debug`
//! rendering is searched for that secret; a derived `Debug` that reappears on
//! any of these types fails here.

use std::collections::BTreeMap;

use tempfile::TempDir;
use xdk::api::Client;
use xdk::auth::{Auth, OAuth1Credential, OAuth2Credential};
use xdk::config::{Config, EnvOverrides};
use xdk::store::{App, OAuth1Token, OAuth2Token, Token, TokenStore, TokenType};

const SECRET: &str = "SECRET-VALUE-9f3c1a";

fn assert_redacted(label: &str, rendered: &str) {
    assert!(
        !rendered.contains(SECRET),
        "{label} leaks a secret in Debug output: {rendered}"
    );
    assert!(!rendered.is_empty(), "{label} rendered nothing");
}

#[test]
fn store_token_types_redact_every_secret() {
    let oauth1 = OAuth1Token {
        access_token: SECRET.into(),
        token_secret: SECRET.into(),
        consumer_key: "consumer-key".into(),
        consumer_secret: SECRET.into(),
    };
    let oauth2 = OAuth2Token {
        access_token: SECRET.into(),
        refresh_token: SECRET.into(),
        expiration_time: 1,
    };
    let bearer = Token {
        token_type: TokenType::Bearer,
        bearer: Some(SECRET.into()),
        oauth2: None,
        oauth1: None,
    };
    let user = Token {
        token_type: TokenType::Oauth2,
        bearer: None,
        oauth2: Some(oauth2.clone()),
        oauth1: Some(oauth1.clone()),
    };
    let mut oauth2_tokens = BTreeMap::new();
    oauth2_tokens.insert("alice".to_string(), user.clone());
    let app = App {
        client_id: "client-id".into(),
        client_secret: SECRET.into(),
        default_user: "alice".into(),
        redirect_uri: "http://localhost:8080/callback".into(),
        oauth2_tokens,
        oauth1_token: Some(user.clone()),
        bearer_token: Some(bearer.clone()),
        unnamed_oauth2_token: Some(user.clone()),
    };
    assert_redacted("OAuth1Token", &format!("{oauth1:?}"));
    assert_redacted("OAuth2Token", &format!("{oauth2:?}"));
    assert_redacted("Token (bearer)", &format!("{bearer:?}"));
    assert_redacted("Token (user)", &format!("{user:?}"));
    assert_redacted("App", &format!("{app:?}"));
    assert!(
        format!("{app:?}").contains("alice"),
        "the shape (app users) is still visible"
    );
}

#[test]
fn store_auth_and_client_types_redact_every_secret() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl");
    let mut store = TokenStore::new_with_path(path.to_str().unwrap());
    store.add_app("myapp", "client-id", SECRET).unwrap();
    store
        .save_oauth2_token_for_app("myapp", "alice", SECRET, SECRET, 1)
        .unwrap();
    store.set_default_app("myapp").unwrap();
    assert_redacted("TokenStore", &format!("{store:?}"));
    assert_redacted("TokenStore::apps", &format!("{:?}", store.apps));

    let mut cfg = Config::new();
    cfg.client_secret = SECRET.into();
    let auth = Auth::new_with_store_path(&cfg, &path);
    assert_redacted("Auth", &format!("{auth:?}"));

    let builder = Client::builder()
        .bearer(SECRET)
        .oauth2(OAuth2Credential {
            client_id: "client-id".into(),
            client_secret: SECRET.into(),
            access_token: SECRET.into(),
            refresh_token: Some(SECRET.into()),
            expires_at: None,
        })
        .oauth1(OAuth1Credential {
            consumer_key: "consumer-key".into(),
            consumer_secret: SECRET.into(),
            access_token: SECRET.into(),
            token_secret: SECRET.into(),
        });
    assert_redacted("ClientBuilder", &format!("{builder:?}"));
    let client = builder.build().unwrap();
    assert_redacted("Client", &format!("{client:?}"));
}

#[test]
fn config_and_env_overrides_redact_every_secret() {
    let mut cfg = Config::from_overrides(&EnvOverrides::default());
    cfg.client_secret = SECRET.into();
    assert_redacted("Config", &format!("{cfg:?}"));

    let overrides = EnvOverrides {
        client_secret: Some(SECRET.into()),
        bearer_token: Some(SECRET.into()),
        ..EnvOverrides::default()
    };
    assert_redacted("EnvOverrides", &format!("{overrides:?}"));
}
