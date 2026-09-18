//! Reads recent posts with a bearer token from the environment, against
//! `API_BASE_URL` when set, and exits with the library's own exit code.

use xdk::api::Client;
use xdk_consumer_check::{describe_failure, recent_posts};

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    let Ok(token) = std::env::var("XURL_BEARER_TOKEN") else {
        eprintln!("set XURL_BEARER_TOKEN to an app-only bearer token");
        return std::process::ExitCode::from(2);
    };
    let mut builder = Client::builder()
        .bearer(token)
        .user_agent("xdk-consumer-check/0.0.0");
    if let Ok(base_url) = std::env::var("API_BASE_URL") {
        builder = builder.base_url(base_url);
    }
    let client = match builder.build() {
        Ok(client) => client,
        Err(error) => {
            eprintln!("{error}");
            return std::process::ExitCode::from(1);
        }
    };
    match recent_posts(&client, "rustlang").await {
        Ok((lines, window)) => {
            for line in lines {
                println!("{line}");
            }
            if let Some(window) = window {
                eprintln!(
                    "rate limit: {:?} left of {:?}",
                    window.remaining, window.limit
                );
            }
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            let (kind, next, code) = describe_failure(&error);
            eprintln!("{kind}: {error} (next: {next:?})");
            std::process::ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
    }
}
