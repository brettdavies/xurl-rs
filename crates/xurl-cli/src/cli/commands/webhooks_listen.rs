//! `xr webhooks listen`: a local receiver that answers X's CRC check and
//! prints each delivered event as one JSON line.

use std::net::SocketAddr;

use serde_json::json;
use xdk::error::{EXIT_AUTH_REQUIRED, Error};
use xdk::webhooks::{Receiver, ReceiverConfig, SignedSample, SigningSecrets};

use super::Run;
use crate::cli::SigningSecret;
use crate::cli::failure::CommandResult;
use crate::cli::hints::shell_word;

/// What `xr webhooks listen` was asked for.
pub(super) struct Listener {
    pub(super) bind: SocketAddr,
    pub(super) path: String,
    pub(super) secret: Option<SigningSecret>,
    pub(super) allow_unsigned: bool,
    pub(super) max_events: Option<u64>,
}

pub(super) async fn listen(run: Run<'_>, listener: Listener) -> CommandResult<()> {
    let Run {
        auth,
        flags,
        out,
        stdout,
        stderr,
        ..
    } = run;
    let mut secrets = SigningSecrets::for_app(&auth);
    match listener.secret {
        Some(SigningSecret::Oauth2) => secrets.oauth1_consumer_secret = None,
        Some(SigningSecret::Oauth1) => secrets.oauth2_client_secret = None,
        None => {}
    }
    let signs_with = if secrets.oauth2_client_secret.is_some() {
        Some("oauth2")
    } else if secrets.oauth1_consumer_secret.is_some() {
        Some("oauth1")
    } else {
        None
    };
    let mut ctx = json!({
        "command": "webhooks-listen",
        "bind": listener.bind.to_string(),
        "path": listener.path,
        "secret": signs_with,
    });
    if flags.dry_run {
        if signs_with.is_some() {
            out.print_dry_run(stdout, true, 0, &ctx);
        } else {
            ctx["reason"] = json!("no-signing-secret");
            out.print_dry_run(stdout, false, EXIT_AUTH_REQUIRED, &ctx);
        }
        return Ok(());
    }
    let Some(signs_with) = signs_with else {
        return Err(Error::auth(
            "No signing secret: X signs webhook requests with the app's OAuth2 client secret or \
             OAuth1 consumer secret, and the app has neither stored",
        )
        .into());
    };

    let sample = SignedSample::new(&secrets)?;
    let mut config = ReceiverConfig::new(listener.bind, secrets);
    config.path.clone_from(&listener.path);
    config.allow_unsigned = listener.allow_unsigned;
    let cancel = crate::cli::shutdown::cancel_on_shutdown();
    let mut receiver = Receiver::bind(config, cancel).await?;
    let url = format!("http://{}{}", receiver.local_addr(), listener.path);
    let test_command = test_command(&url, &sample);
    if out.format.is_structured() {
        let up = json!({
            "status": "listening",
            "url": url,
            "secret": signs_with,
            "test_command": test_command,
            "test_request": test_request(&url, &sample),
        });
        out.print_stream_line(stdout, &up.to_string());
    } else {
        out.info(
            stderr,
            &format!(
                "Listening on {url}. X delivers only to a public HTTPS URL with no port: expose \
                 this address through a tunnel or proxy, then register that URL with \
                 `xr webhooks add <URL>`."
            ),
        );
        out.info(
            stderr,
            &format!("Test it with a signed sample event: {test_command}"),
        );
    }
    let _ = stdout.flush();

    let mut printed: u64 = 0;
    while let Some(event) = receiver.next_event().await {
        let line = event_line(&event.body);
        out.print_stream_line(stdout, &line);
        let _ = stdout.flush();
        printed += 1;
        if listener.max_events.is_some_and(|max| printed >= max) {
            break;
        }
    }
    Ok(())
}

/// A `curl` invocation, quoted for a POSIX shell, that delivers `sample` to
/// `url` the way X delivers an event. `-f` makes a refused delivery exit
/// non-zero.
fn test_command(url: &str, sample: &SignedSample) -> String {
    let mut words = vec!["curl".to_string(), "-fsS".to_string()];
    for (name, value) in sample.headers() {
        words.push("-H".to_string());
        words.push(shell_word(&format!("{name}: {value}")));
    }
    words.push("--data-binary".to_string());
    words.push(shell_word(sample.body));
    words.push(shell_word(url));
    words.join(" ")
}

/// The same delivery as [`test_command`], as data for a caller with no POSIX
/// shell.
fn test_request(url: &str, sample: &SignedSample) -> serde_json::Value {
    let headers: serde_json::Map<String, serde_json::Value> = sample
        .headers()
        .into_iter()
        .map(|(name, value)| (name.to_string(), json!(value)))
        .collect();
    json!({"method": "POST", "url": url, "headers": headers, "body": sample.body})
}

/// One event as one line of JSON.
///
/// A body that is JSON on a single line is printed byte for byte, so what a
/// consumer reads is what X signed. A JSON body that spans lines is
/// re-serialized compactly, and a body that is not JSON is carried as a JSON
/// string, so every line parses.
fn event_line(body: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(body) {
        let text = text.trim();
        let one_line = !text.contains(['\n', '\r']);
        if one_line && serde_json::from_str::<serde::de::IgnoredAny>(text).is_ok() {
            return text.to_string();
        }
    }
    match serde_json::from_slice::<serde_json::Value>(body) {
        Ok(value) => value.to_string(),
        Err(_) => json!(String::from_utf8_lossy(body)).to_string(),
    }
}
