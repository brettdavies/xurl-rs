//! Runs to completion with no X app, no credential, and no network: the
//! `testing` feature's in-process mock answers the search from the crate's
//! fixtures, and the assertions fail the process if it does not.
//!
//! ```bash
//! cargo run -p xdk-rs --example offline_search --features testing
//! ```

// `xdk::Error` is wide, which the crate allows for itself; a `main` returning
// `xdk::Result` inherits it.
#![allow(clippy::result_large_err)]

use xdk::testing::MockX;

#[tokio::main]
async fn main() -> xdk::Result<()> {
    let mock = MockX::start().await;
    println!("mock X API listening at {}", mock.base_url());
    let client = mock.app_client()?;

    let posts = client.search_posts("rust", 10).send().await?;
    for post in &posts.data {
        println!("{}: {}", post.id, post.text);
    }
    assert_eq!(
        posts.data.len(),
        2,
        "the post_list fixture carries two posts"
    );

    let window = client
        .last_rate_limit()
        .expect("every mock response reports a rate-limit window");
    println!(
        "{:?} of {:?} requests left in the window",
        window.remaining, window.limit
    );

    let seen = mock.requests().await;
    assert_eq!(seen.len(), 1, "one request reached the mock");
    assert_eq!(seen[0].method, "GET");
    assert_eq!(seen[0].path, "/2/tweets/search/recent");
    assert!(
        seen[0]
            .headers
            .iter()
            .any(|(name, value)| name == "authorization" && value.starts_with("Bearer ")),
        "the bearer token reached the wire"
    );
    println!("offline search completed against the mock");
    Ok(())
}
