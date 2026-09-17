//! Searches recent posts with an app-only bearer token and reads the
//! rate-limit window the response reported.
//!
//! ```bash
//! XURL_BEARER_TOKEN=... cargo run -p xdk-rs --example search -- "rust lang"
//! ```

mod common;

use xdk::api::Client;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        common::exit_with(err);
    }
}

async fn run() -> xdk::Result<()> {
    let query = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "rust".to_string());
    let client = Client::builder()
        .bearer(common::required_env("XURL_BEARER_TOKEN")?)
        .build()?;

    let posts = client.search_posts(&query, 10).send().await?;
    for post in &posts.data {
        println!("{}: {}", post.id, post.text);
    }
    if let Some(window) = client.last_rate_limit() {
        println!(
            "{} of {} requests left until {:?}",
            window.remaining.map_or("?".to_string(), |n| n.to_string()),
            window.limit.map_or("?".to_string(), |n| n.to_string()),
            window.reset_at
        );
    }
    Ok(())
}
