//! Reads the first lines of the sampled post stream with an app-only bearer
//! token, through the raw-request path, because streaming endpoints have no
//! shortcut.
//!
//! ```bash
//! XURL_BEARER_TOKEN=... cargo run -p xdk-rs --example stream
//! ```

mod common;

use std::collections::HashMap;

use xdk::api::{Client, RequestOptions, RequestTarget};

const LINES_TO_SHOW: usize = 10;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        common::exit_with(err);
    }
}

async fn run() -> xdk::Result<()> {
    let client = Client::builder()
        .bearer(common::required_env("XURL_BEARER_TOKEN")?)
        .build()?;

    let options = RequestOptions {
        method: "GET".to_string(),
        target: RequestTarget::Template {
            path: "/2/tweets/sample/stream".to_string(),
            path_params: HashMap::new(),
            query: vec![("tweet.fields".to_string(), "created_at".to_string())],
        },
        ..RequestOptions::default()
    };

    let mut lines = client.stream_request(&options).await?;
    let mut shown = 0;
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        println!("{line}");
        shown += 1;
        if shown == LINES_TO_SHOW {
            break;
        }
    }
    Ok(())
}
