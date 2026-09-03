//! Live smoke gate for the typed wire vocabulary.
//!
//! The mocked suite proves the response types match the vendored spec; only a
//! live read proves they match what X sends. Each run spends one post read and
//! one user read on a pay-as-you-go credential, so the test is ignored by
//! default and refuses to run unless `XURL_LIVE_SMOKE=1`.
//!
//! ```sh
//! XURL_LIVE_SMOKE=1 cargo test --test live_smoke -- --ignored
//! ```
//!
//! `XURL_APP` selects the store app, `XURL_LIVE_SMOKE_AUTH` pins the scheme
//! (`app`, `oauth1`, `oauth2`), and `XURL_LIVE_SMOKE_POST_ID` replaces the
//! default reply post with any other reply or quote post.

use xurl::api::{ApiClient, CallOptions};
use xurl::auth::Auth;
use xurl::config::Config;

const DEFAULT_POST_ID: &str = "2042810767666483622";
const USERNAME: &str = "jack";

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn live_client() -> (ApiClient, CallOptions) {
    assert_eq!(
        env("XURL_LIVE_SMOKE").as_deref(),
        Some("1"),
        "refusing to spend a post read and a user read on the live X API without XURL_LIVE_SMOKE=1"
    );
    let cfg = Config::new();
    let mut auth = Auth::new(&cfg);
    if let Some(app) = env("XURL_APP") {
        auth.with_app_name(&app);
    }
    let opts = CallOptions {
        auth_type: env("XURL_LIVE_SMOKE_AUTH").unwrap_or_default(),
        ..CallOptions::default()
    };
    (ApiClient::new(&cfg, auth), opts)
}

#[test]
#[ignore = "spends one post read and one user read on the live X API; see RELEASES-PREFLIGHT.md"]
fn live_wire_vocabulary_matches_typed_structs() {
    let (mut client, opts) = live_client();

    let post_id = env("XURL_LIVE_SMOKE_POST_ID").unwrap_or_else(|| DEFAULT_POST_ID.to_string());
    let post = client.read_post(&post_id, &opts).expect(
        "post read must succeed; pass XURL_LIVE_SMOKE_POST_ID=<any reply or quote post id> if the default was deleted",
    );
    let metrics = post
        .data
        .public_metrics
        .as_ref()
        .expect("post.public_metrics must be present");
    assert!(
        metrics.extra.is_empty(),
        "X sent post metric keys no typed field reads: {:?}",
        metrics.extra.keys().collect::<Vec<_>>()
    );
    let total = metrics.repost_count
        + metrics.reply_count
        + metrics.like_count
        + metrics.quote_count
        + metrics.bookmark_count
        + metrics.impression_count;
    assert!(
        total > 0,
        "every typed post metric read as zero: {metrics:?}"
    );
    for legacy in ["referenced_tweets", "edit_history_tweet_ids"] {
        assert!(
            !post.data.extra.contains_key(legacy),
            "X answered a post.fields request with the legacy key {legacy}"
        );
    }
    assert!(
        post.data
            .referenced_posts
            .as_ref()
            .is_some_and(|refs| !refs.is_empty()),
        "referenced_posts absent; the post must be a reply or quote"
    );
    assert!(
        post.includes
            .as_ref()
            .and_then(|inc| inc.posts.as_ref())
            .is_some_and(|posts| !posts.is_empty()),
        "includes.posts absent; the referenced_posts expansion did not land"
    );

    let user = client
        .lookup_user(USERNAME, &opts)
        .expect("user read must succeed");
    let metrics = user
        .data
        .public_metrics
        .as_ref()
        .expect("user.public_metrics must be present");
    assert!(
        metrics.post_count > 0,
        "post_count read as zero; X's user post metric matches neither post_count nor tweet_count: {metrics:?}"
    );
    assert!(
        metrics.followers_count > 0 && metrics.following_count > 0 && metrics.listed_count > 0,
        "a typed user metric read as zero: {metrics:?}"
    );
    assert!(
        !metrics.extra.contains_key("tweet_count"),
        "tweet_count landed in extra instead of post_count"
    );
}
