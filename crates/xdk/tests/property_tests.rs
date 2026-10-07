//! Properties of the OAuth1 percent-encoder and of the token store's file
//! round trip, each checked against generated inputs.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use proptest::prelude::*;
use xdk::auth::oauth1::encode;
use xdk::store::{App, TokenStore};

/// The bytes RFC 3986 section 2.3 calls unreserved.
fn unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"-._~".contains(&byte)
}

proptest! {
    /// RFC 5849 section 3.6: an unreserved byte is written as it is, and
    /// every other byte as `%` and two uppercase hex digits.
    #[test]
    fn the_encoder_leaves_exactly_the_unreserved_bytes_bare(input in any::<String>()) {
        let mut expected = String::new();
        for byte in input.bytes() {
            if unreserved(byte) {
                expected.push(char::from(byte));
            } else {
                write!(expected, "%{byte:02X}").unwrap();
            }
        }
        prop_assert_eq!(encode(&input), expected);
    }
}

/// One app's credentials, as the store's save functions take them.
#[derive(Debug, Clone, PartialEq)]
struct AppSpec {
    client_id: String,
    client_secret: String,
    /// Username to access token, refresh token, and expiry.
    oauth2: BTreeMap<String, (String, String, u64)>,
    /// Access token, token secret, consumer key, consumer secret.
    oauth1: Option<(String, String, String, String)>,
    bearer: Option<String>,
}

/// A printable string, the shape of every credential X issues.
fn text() -> impl Strategy<Value = String> {
    "\\PC{0,24}"
}

fn app_spec() -> impl Strategy<Value = AppSpec> {
    (
        text(),
        text(),
        prop::collection::btree_map("\\PC{1,12}", (text(), text(), any::<u64>()), 0..3),
        prop::option::of((text(), text(), text(), text())),
        prop::option::of(text()),
    )
        .prop_map(
            |(client_id, client_secret, oauth2, oauth1, bearer)| AppSpec {
                client_id,
                client_secret,
                oauth2,
                oauth1,
                bearer,
            },
        )
}

/// What `app` holds, read field by field, so the comparison does not pass
/// through the serializer the round trip is testing.
fn spec_of(app: &App) -> AppSpec {
    AppSpec {
        client_id: app.client_id.clone(),
        client_secret: app.client_secret.clone(),
        oauth2: app
            .oauth2_tokens
            .iter()
            .filter_map(|(username, token)| {
                let token = token.oauth2.as_ref()?;
                let fields = (
                    token.access_token.clone(),
                    token.refresh_token.clone(),
                    token.expiration_time,
                );
                Some((username.clone(), fields))
            })
            .collect(),
        oauth1: app
            .oauth1_token
            .as_ref()
            .and_then(|token| token.oauth1.as_ref())
            .map(|token| {
                (
                    token.access_token.clone(),
                    token.token_secret.clone(),
                    token.consumer_key.clone(),
                    token.consumer_secret.clone(),
                )
            }),
        bearer: app
            .bearer_token
            .as_ref()
            .and_then(|token| token.bearer.clone()),
    }
}

proptest! {
    // Each case writes and reads a file, so the count stays small.
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Whatever apps and tokens are saved, a second store opened on the same
    /// path holds every one of them, under the same default app.
    #[test]
    fn a_saved_store_loads_back_equal(
        apps in prop::collection::btree_map("[A-Za-z0-9_.-]{1,12}", app_spec(), 0..4),
    ) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(".xurl");
        let path = path.to_str().expect("utf-8 path");

        let mut saved = TokenStore::new_with_path(path);
        for (name, spec) in &apps {
            saved
                .add_app(name, &spec.client_id, &spec.client_secret)
                .expect("add_app");
            for (username, (access, refresh, expires)) in &spec.oauth2 {
                saved
                    .save_oauth2_token_for_app(name, username, access, refresh, *expires)
                    .expect("save_oauth2");
            }
            if let Some((access, secret, consumer_key, consumer_secret)) = &spec.oauth1 {
                saved
                    .save_oauth1_tokens_for_app(name, access, secret, consumer_key, consumer_secret)
                    .expect("save_oauth1");
            }
            if let Some(bearer) = &spec.bearer {
                saved
                    .save_bearer_token_for_app(name, bearer)
                    .expect("save_bearer");
            }
        }

        let loaded = TokenStore::new_with_path(path);
        let held: BTreeMap<String, AppSpec> = loaded
            .apps
            .iter()
            .map(|(name, app)| (name.clone(), spec_of(app)))
            .collect();
        prop_assert_eq!(held, apps);
        prop_assert_eq!(loaded.default_app, saved.default_app);
    }
}
