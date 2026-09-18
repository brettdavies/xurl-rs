//! Acts as a user with an OAuth2 credential held in code, and registers the
//! hook that receives every rotated token pair.
//!
//! X rotates the refresh token on every refresh, so the pair the hook
//! receives is the only live copy; this example prints where a real hook
//! would persist it instead of writing anything. The access token's expiry
//! comes from `EXPIRES_IN` (seconds, the token response's own field); with
//! no value the token is treated as already expired, so a run with a
//! `REFRESH_TOKEN` refreshes once and the hook fires.
//!
//! ```bash
//! CLIENT_ID=... CLIENT_SECRET=... ACCESS_TOKEN=... REFRESH_TOKEN=... EXPIRES_IN=7200 \
//!   cargo run -p xdk-rs --example authenticate
//! ```

mod common;

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::{Duration, SystemTime};

use xdk::api::Client;
use xdk::auth::{BoxError, OAuth2Credential, OnTokenRefreshed};

/// Where the rotated pair would go; a real hook writes it there atomically.
struct PrintPersistLocation {
    path: PathBuf,
}

impl OnTokenRefreshed for PrintPersistLocation {
    fn on_token_refreshed<'a>(
        &'a self,
        credential: &'a OAuth2Credential,
    ) -> Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'a>> {
        Box::pin(async move {
            println!(
                "token pair rotated: a real hook would write it to {} (expires at {:?}, refresh token present: {})",
                self.path.display(),
                credential.expires_at,
                credential.refresh_token.is_some()
            );
            Ok(())
        })
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        common::exit_with(err);
    }
}

async fn run() -> xdk::Result<()> {
    let credential = OAuth2Credential {
        client_id: common::required_env("CLIENT_ID")?,
        client_secret: common::required_env("CLIENT_SECRET")?,
        access_token: common::required_env("ACCESS_TOKEN")?,
        refresh_token: std::env::var("REFRESH_TOKEN")
            .ok()
            .filter(|v| !v.is_empty()),
        expires_at: Some(
            std::env::var("EXPIRES_IN")
                .ok()
                .and_then(|secs| secs.parse::<u64>().ok())
                .map_or(SystemTime::UNIX_EPOCH, |secs| {
                    SystemTime::now() + Duration::from_secs(secs)
                }),
        ),
    };
    let token_file = std::env::var_os("XDK_TOKEN_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("x-tokens.json"));
    println!(
        "a rotated pair would be persisted to {}",
        token_file.display()
    );

    let client = Client::builder()
        .oauth2(credential)
        .on_token_refreshed(PrintPersistLocation { path: token_file })
        .build()?;

    let me = client.get_me().send().await?;
    println!("signed in as @{} ({})", me.data.username, me.data.name);
    Ok(())
}
