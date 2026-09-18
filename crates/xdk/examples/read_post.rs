//! Reads one post with an app-only bearer token.
//!
//! ```bash
//! XURL_BEARER_TOKEN=... cargo run -p xdk-rs --example read_post -- 20
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
    let post_id = std::env::args().nth(1).unwrap_or_else(|| "20".to_string());
    let client = Client::builder()
        .bearer(common::required_env("XURL_BEARER_TOKEN")?)
        .build()?;

    let post = client.read_post(&post_id).send().await?;
    println!("{}: {}", post.data.id, post.data.text);
    if let Some(created_at) = &post.data.created_at {
        println!("posted {created_at}");
    }
    if let Some(author) = post
        .includes
        .as_ref()
        .and_then(|inc| inc.users.as_ref())
        .and_then(|users| users.first())
    {
        println!("by @{}", author.username);
    }
    Ok(())
}
