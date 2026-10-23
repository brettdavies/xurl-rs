//! Receives X's webhook deliveries on a local port, forwards each verified
//! event to an HTTP endpoint of yours under the retry rule `xr` uses, and
//! keeps a restart checkpoint. At start it prints the replay window a
//! checkpoint left by an earlier run would request; it sends nothing to X.
//!
//! ```bash
//! CLIENT_SECRET=... FORWARD_URL=http://127.0.0.1:3000/events \
//!     cargo run -p xdk-rs --example webhook_forward
//! ```
//!
//! Expose `127.0.0.1:8080` at a public HTTPS URL and register it with X, as
//! the `xdk::webhooks` docs describe.

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::time::SystemTime;

use tokio_util::sync::CancellationToken;
use xdk::webhooks::{
    CheckpointWriter, Receiver, ReceiverConfig, RetryBudget, SigningSecrets, UrlTarget,
    read_checkpoint, replay_window,
};

const CHECKPOINT: &str = "webhook-forward-checkpoint.json";

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        common::exit_with(err);
    }
}

async fn run() -> xdk::Result<()> {
    let now = SystemTime::now();
    if let Ok(stopped_at) = read_checkpoint(Path::new(CHECKPOINT), now) {
        let window = replay_window(stopped_at, now)?;
        println!(
            "a replay of {} to {} would recover what the last run missed (clamped: {})",
            window.from, window.to, window.clamped
        );
    }

    let secrets = SigningSecrets {
        oauth2_client_secret: Some(common::required_env("CLIENT_SECRET")?),
        oauth1_consumer_secret: None,
    };
    let target = UrlTarget::new(&common::required_env("FORWARD_URL")?, &[] as &[&str])?;
    let budget = RetryBudget::default();
    let mut checkpoint = CheckpointWriter::new(CHECKPOINT);

    let bind = SocketAddr::from(([127, 0, 0, 1], 8080));
    let mut receiver =
        Receiver::bind(ReceiverConfig::new(bind, secrets), CancellationToken::new()).await?;
    println!("listening on {}", receiver.local_addr());
    while let Some(event) = receiver.next_event().await {
        let outcome = target.forward(&event, &budget).await;
        println!(
            "{} after {} attempt(s): {:?}",
            event.sha256(),
            outcome.attempts,
            outcome.result
        );
        // One target, forwarded in arrival order: once this forward has
        // finished, no earlier event is waiting, so the checkpoint moves on.
        checkpoint.record(event.received_at)?;
    }
    Ok(())
}
