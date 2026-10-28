//! `xr webhooks listen`: a local receiver that answers X's CRC check and
//! prints each delivered event as one JSON line.

use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use xdk::error::{EXIT_AUTH_REQUIRED, Error};
use xdk::webhooks::{
    Receiver, ReceiverConfig, RetryBudget, SignedSample, SigningSecrets, UrlTarget,
    validate_forward_url,
};

use super::Run;
use super::webhooks_checkpoint::Progress;
use super::webhooks_forward::{Finished, Forwarding, Target};
use super::webhooks_program::ProgramTarget;
use crate::cli::SigningSecret;
use crate::cli::failure::{CommandResult, Failure};
use crate::cli::hints::shell_word;

/// What `xr webhooks listen` was asked for.
pub(super) struct Listener {
    pub(super) bind: SocketAddr,
    pub(super) path: String,
    pub(super) secret: Option<SigningSecret>,
    pub(super) allow_unsigned: bool,
    pub(super) max_events: Option<u64>,
    pub(super) forward: Option<String>,
    pub(super) headers: Vec<String>,
    pub(super) forward_timeout: Option<u64>,
    pub(super) program: Vec<String>,
    pub(super) checkpoint: Option<PathBuf>,
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
    let budget = listener
        .forward_timeout
        .map_or_else(RetryBudget::default, |secs| {
            RetryBudget::new(Duration::from_secs(secs))
        });
    let targets = forward_targets(&listener);
    let mut ctx = json!({
        "command": "webhooks-listen",
        "bind": listener.bind.to_string(),
        "path": listener.path,
        "secret": signs_with,
    });
    if let Some(url) = &listener.forward {
        // A refused header may hold its value where its name belongs, so the
        // names are shown only once every header parsed.
        let headers = if targets.is_ok() {
            header_names(&listener.headers)
        } else {
            Vec::new()
        };
        ctx["forward"] = json!({
            "url": url,
            "headers": headers,
            "timeout_secs": budget.attempt_timeout().as_secs(),
        });
    }
    if let Some(checkpoint) = &listener.checkpoint {
        ctx["checkpoint"] = json!(checkpoint.display().to_string());
    }
    if !listener.program.is_empty() {
        ctx["program"] = json!(listener.program);
        ctx["timeout_secs"] = json!(budget.attempt_timeout().as_secs());
    }
    if flags.dry_run {
        if let Err(error) = targets {
            return Err(Failure::DryRunRefusal { ctx, error });
        }
        if signs_with.is_some() {
            out.print_dry_run(stdout, true, 0, &ctx);
        } else {
            ctx["reason"] = json!("no-signing-secret");
            out.print_dry_run(stdout, false, EXIT_AUTH_REQUIRED, &ctx);
        }
        return Ok(());
    }
    let targets = targets.map_err(|error| {
        if error.is_validation() {
            Failure::refused(error)
        } else {
            Failure::from(error)
        }
    })?;
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

    let mut forwarding = (!targets.is_empty()).then(|| Forwarding::start(targets, budget));
    let mut progress = Progress::new(listener.checkpoint.as_deref());
    let mut printed: u64 = 0;
    loop {
        tokio::select! {
            biased;
            Some(finished) = next_finished(&mut forwarding, stderr) => {
                if let Some(forwarding) = forwarding.as_mut() {
                    forwarding.record(out, stderr, &finished);
                    progress.advance(forwarding.oldest_pending(), out, stderr);
                }
            }
            event = receiver.next_event() => {
                let Some(event) = event else {
                    break;
                };
                let line = event_line(&event.body);
                out.print_stream_line(stdout, &line);
                let _ = stdout.flush();
                progress.arrived(event.received_at);
                if let Some(forwarding) = forwarding.as_mut() {
                    forwarding.queue(&Arc::new(event)).await;
                }
                let oldest = forwarding.as_ref().and_then(Forwarding::oldest_pending);
                progress.advance(oldest, out, stderr);
                printed += 1;
                if listener.max_events.is_some_and(|max| printed >= max) {
                    break;
                }
            }
        }
    }
    drop(receiver);
    if let Some(forwarding) = forwarding {
        forwarding.drain(out, stderr, &mut progress).await;
    }
    Ok(())
}

/// The next delivery a forward target finished, writing a program's output
/// to `stderr` as it comes; never, with no target.
async fn next_finished(
    forwarding: &mut Option<Forwarding>,
    stderr: &mut dyn Write,
) -> Option<Finished> {
    match forwarding {
        Some(forwarding) => forwarding.next_finished(stderr).await,
        None => std::future::pending().await,
    }
}

/// The targets `--forward` and the program after `--` name, validated
/// before anything binds and refused in `xr`'s own words. A header is named
/// by its position among the `-H` flags, never by its value.
fn forward_targets(listener: &Listener) -> Result<Vec<Target>, Error> {
    let mut targets = Vec::new();
    if let Some(url) = &listener.forward {
        targets.push(Target::Url(url_target(url, &listener.headers)?));
    }
    if !listener.program.is_empty() {
        let search = crate::cli::env::program_search();
        targets.push(Target::Program(ProgramTarget::resolve(
            &listener.program,
            &search,
        )?));
    }
    Ok(targets)
}

/// `--forward` with its `-H` headers.
fn url_target(url: &str, headers: &[String]) -> Result<UrlTarget, Error> {
    validate_forward_url(url).map_err(|reason| {
        Error::validation(match reason {
            "forward-url-has-credentials" => {
                "--forward takes a URL with no username or password, which every record of a \
                 failed forward would repeat; pass the credential with -H 'Authorization: ...' \
                 instead"
            }
            "forward-url-not-http" => "--forward takes an http or https URL",
            _ => "--forward takes an absolute http or https URL",
        })
    })?;
    UrlTarget::new(url, headers).map_err(|error| match error {
        Error::Validation(message) => Error::validation(format!("-H: {message}")),
        other => other,
    })
}

/// The name of each `-H` header, for a dry run: never a value.
fn header_names(headers: &[String]) -> Vec<&str> {
    headers
        .iter()
        .map(|line| {
            line.split_once(':')
                .map_or(line.as_str(), |(name, _)| name)
                .trim()
        })
        .collect()
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
