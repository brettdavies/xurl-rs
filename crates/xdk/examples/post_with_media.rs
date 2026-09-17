//! Uploads one media file and publishes a post that carries it, as a user.
//!
//! ```bash
//! CLIENT_ID=... CLIENT_SECRET=... ACCESS_TOKEN=... \
//!   cargo run -p xdk-rs --example post_with_media -- photo.png "hello from xdk"
//! ```

mod common;

use xdk::api::{Client, execute_media_upload};
use xdk::auth::OAuth2Credential;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        common::exit_with(err);
    }
}

async fn run() -> xdk::Result<()> {
    let mut args = std::env::args().skip(1);
    let file = args.next().unwrap_or_else(|| "photo.png".to_string());
    let text = args.next().unwrap_or_else(|| "hello from xdk".to_string());
    let media_type = match file.rsplit('.').next() {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("mp4") => "video/mp4",
        _ => "image/png",
    };
    let category = if media_type.starts_with("video/") {
        "tweet_video"
    } else {
        "tweet_image"
    };

    let client = Client::builder()
        .oauth2(OAuth2Credential {
            client_id: common::required_env("CLIENT_ID")?,
            client_secret: common::required_env("CLIENT_SECRET")?,
            access_token: common::required_env("ACCESS_TOKEN")?,
            refresh_token: None,
            expires_at: None,
        })
        .build()?;

    let upload = execute_media_upload(
        &file,
        media_type,
        category,
        "",
        "",
        false,
        true,
        &[],
        &client,
    )
    .await?;
    let media_id = upload.init.data.id.clone();
    println!("uploaded {file} as media {media_id}");

    let post = client.create_post(&text, &[media_id]).send().await?;
    println!("posted {}: {}", post.data.id, post.data.text);
    Ok(())
}
