//! The store shapes a walk runs every command over: what the active app
//! stores, and what a second app stores.

use std::time::{SystemTime, UNIX_EPOCH};

use xdk::api::auth_matrix::WireScheme;
use xdk::store::TokenStore;

use super::invoke::{Fill, SAMPLE, Seed, bearer_only, live_oauth2_only, nothing, oauth1_only};

pub const OTHER: &str = "other-app";

fn register_other(ts: &mut TokenStore) {
    ts.add_app(OTHER, "CLIENT-ID", "CLIENT-SECRET")
        .expect("add app");
}

fn other_with_bearer(ts: &mut TokenStore) {
    register_other(ts);
    ts.save_bearer_token_for_app(OTHER, "BEARER")
        .expect("save bearer");
}

fn other_with_oauth1(ts: &mut TokenStore) {
    register_other(ts);
    ts.save_oauth1_tokens_for_app(OTHER, "ACCESS", "SECRET", "KEY", "CONSUMER")
        .expect("save oauth1");
}

fn other_with_login(ts: &mut TokenStore) {
    register_other(ts);
    let in_an_hour = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock is past the epoch")
        .as_secs()
        + 3600;
    ts.save_oauth2_token_for_app(OTHER, SAMPLE, "LIVE-ACCESS", "REFRESH", in_an_hour)
        .expect("save oauth2");
}

/// One thing a store can hold: what the active app stores (`holds`), and the
/// scheme the other registered app stores, if any.
pub struct Shape {
    pub name: &'static str,
    pub seed: Seed,
    pub holds: &'static [WireScheme],
}

pub fn shapes() -> Vec<Shape> {
    let fills: [(&str, Fill, &[WireScheme]); 7] = [
        ("an empty app", nothing, &[]),
        ("a login", live_oauth2_only, &[WireScheme::OAuth2]),
        ("OAuth1 keys", oauth1_only, &[WireScheme::OAuth1]),
        ("a bearer token", bearer_only, &[WireScheme::App]),
        (
            "nothing, another app with a bearer token",
            other_with_bearer,
            &[],
        ),
        (
            "nothing, another app with OAuth1 keys",
            other_with_oauth1,
            &[],
        ),
        ("nothing, another app with a login", other_with_login, &[]),
    ];
    fills
        .into_iter()
        .map(|(name, fill, holds)| Shape {
            name,
            seed: Seed::of(fill),
            holds,
        })
        .collect()
}
