/// Streaming request handler — SSE / chunked transfer support.
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::api::{ApiClient, RequestOptions};
use crate::auth::callback::shutdown_signal;
use crate::error::{Result, XurlError};
use crate::output::OutputConfig;

/// Spawns a background thread that waits for SIGINT/SIGTERM and flips the
/// returned `AtomicBool` to true. The thread holds its own current-thread
/// tokio runtime so the synchronous streaming path can observe signals
/// without itself being async.
fn spawn_shutdown_watcher() -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    let flag_for_thread = Arc::clone(&flag);
    std::thread::spawn(move || {
        let Ok(rt) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        rt.block_on(async move {
            shutdown_signal().await;
            flag_for_thread.store(true, Ordering::SeqCst);
        });
    });
    flag
}

/// Sends a streaming request with output-format awareness.
pub(super) fn stream_request_with_output(
    client: &mut ApiClient,
    options: &RequestOptions,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<()> {
    use std::io::{BufRead, BufReader};

    let method = options.method.to_uppercase();
    let method = if method.is_empty() { "GET" } else { &method };
    // Fail-fast auth-matrix validation before any URL render or socket open.
    // ApiClient::stream_request runs the same gate; the CLI streaming
    // wrapper otherwise bypasses it because it builds the request inline
    // rather than delegating to ApiClient::stream_request. Gated on
    // !no_auth so explicit auth-skip invocations still work even when the
    // matrix would reject the auth_type.
    if !options.no_auth {
        crate::api::auth_matrix::validate(
            &options.target,
            method,
            &options.auth_type,
            Some(client.auth_app_name()),
        )?;
    }
    let url = client.build_url_public(&options.target)?;

    let req_method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|_| XurlError::InvalidMethod(method.to_string()))?;

    let mut builder = reqwest::blocking::Client::builder()
        .timeout(None)
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
        .request(req_method, &url);

    if !options.data.is_empty() {
        if serde_json::from_str::<serde_json::Value>(&options.data).is_ok() {
            builder = builder
                .header("Content-Type", "application/json")
                .body(options.data.clone());
        } else {
            builder = builder
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(options.data.clone());
        }
    }

    for header in &options.headers {
        if let Some((key, value)) = header.split_once(':') {
            builder = builder.header(key.trim(), value.trim());
        }
    }

    // Propagate auth errors rather than silently sending unauthenticated.
    // Mirrors send_request's policy: an AuthMethodMismatch envelope (or any
    // other auth failure) must reach the caller, not get swallowed into a
    // request the user's app cannot satisfy.
    if !options.no_auth {
        let auth_header = client.get_auth_header_public(options)?;
        builder = builder.header("Authorization", auth_header);
    }

    builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));

    if options.trace {
        builder = builder.header("X-B3-Flags", "1");
    }

    if options.verbose {
        if out.use_color {
            out.verbose(stderr, &format!("\x1b[1;34m> {method}\x1b[0m {url}"));
        } else {
            out.verbose(stderr, &format!("> {method} {url}"));
        }
    }

    out.status(stderr, &format!("Connecting to streaming endpoint: {url}"));

    let resp = builder.send()?;

    if options.verbose {
        let status = resp.status();
        if out.use_color {
            out.verbose(stderr, &format!("\x1b[1;31m< {status}\x1b[0m"));
            for (key, value) in resp.headers() {
                out.verbose(
                    stderr,
                    &format!("\x1b[1;32m< {key}\x1b[0m: {}", value.to_str().unwrap_or("")),
                );
            }
        } else {
            out.verbose(stderr, &format!("< {status}"));
            for (key, value) in resp.headers() {
                out.verbose(
                    stderr,
                    &format!("< {key}: {}", value.to_str().unwrap_or("")),
                );
            }
        }
        out.verbose(stderr, "");
    }

    let resp_status = resp.status();
    if resp_status.as_u16() >= 400 {
        let body = resp.text().unwrap_or_default();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            return Err(XurlError::api(resp_status.as_u16(), json.to_string()));
        }
        return Err(XurlError::api(resp_status.as_u16(), body));
    }

    out.status(stderr, "--- Streaming response started ---");
    out.status(stderr, "--- Press Ctrl+C to stop ---");

    let shutdown = spawn_shutdown_watcher();
    let reader = BufReader::with_capacity(1024 * 1024, resp);
    for line in reader.lines() {
        if shutdown.load(Ordering::SeqCst) {
            // Flush buffered output, emit a cancellation envelope under JSON
            // modes, and exit cleanly. Text mode keeps stdout silent — the
            // status banner on stderr already signals shutdown.
            let _ = stdout.flush();
            if out.format.is_structured() {
                let envelope = serde_json::json!({
                    "status": "cancelled",
                    "reason": "sigterm",
                });
                out.print_response(stdout, &envelope);
            }
            out.status(stderr, "--- Stream cancelled by signal ---");
            return Ok(());
        }
        match line {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                out.print_stream_line(stdout, &line);
            }
            Err(e) => {
                return Err(XurlError::Io(e.to_string()));
            }
        }
    }

    out.status(stderr, "--- End of stream ---");
    Ok(())
}
