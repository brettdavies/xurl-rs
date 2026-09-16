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
//! default post with any other post that carries media.

use xurl::api::Client;
use xurl::api::auth_matrix::WireScheme;
use xurl::auth::Auth;
use xurl::config::Config;

const DEFAULT_POST_ID: &str = "1585341984679469056";
const USERNAME: &str = "elonmusk";

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn live_client() -> (Client, Option<WireScheme>) {
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
    let scheme = env("XURL_LIVE_SMOKE_AUTH").map(|raw| {
        WireScheme::from_wire(&raw)
            .unwrap_or_else(|| panic!("XURL_LIVE_SMOKE_AUTH={raw} is not app, oauth1, or oauth2"))
    });
    (Client::new(&cfg, auth).expect("client builds"), scheme)
}

#[tokio::test]
#[ignore = "spends one post read and one user read on the live X API; see RELEASES-PREFLIGHT.md"]
async fn live_wire_vocabulary_matches_typed_structs() {
    let (client, scheme) = live_client();

    let post_id = env("XURL_LIVE_SMOKE_POST_ID").unwrap_or_else(|| DEFAULT_POST_ID.to_string());
    let mut read = client.read_post(&post_id);
    if let Some(scheme) = scheme {
        read = read.auth(scheme);
    }
    let post = read.send().await.expect(
        "post read must succeed; pass XURL_LIVE_SMOKE_POST_ID=<any post id with media> if the default was deleted",
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
            .attachments
            .as_ref()
            .and_then(|a| a.get("media_keys"))
            .and_then(|k| k.as_array())
            .is_some_and(|keys| !keys.is_empty()),
        "attachments.media_keys absent; the post must carry media"
    );
    if post
        .data
        .referenced_posts
        .as_ref()
        .is_some_and(|refs| !refs.is_empty())
    {
        assert!(
            post.includes
                .as_ref()
                .and_then(|inc| inc.posts.as_ref())
                .is_some_and(|posts| !posts.is_empty()),
            "includes.posts absent; the referenced_posts expansion did not land"
        );
    }

    let mut lookup = client.lookup_user(USERNAME);
    if let Some(scheme) = scheme {
        lookup = lookup.auth(scheme);
    }
    let user = lookup.send().await.expect("user read must succeed");
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
