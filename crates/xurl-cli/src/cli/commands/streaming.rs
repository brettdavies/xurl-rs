//! Streaming request handler: SSE / chunked transfer support.
use std::io::Write;

use crate::cli::envelope::Envelope;
use crate::cli::output::OutputConfig;
use crate::cli::shutdown::{ShutdownSignal, shutdown_signal};
use xdk::api::{Client, RequestOptions};
use xdk::error::Result;

/// Sends a streaming request with output-format awareness.
///
/// The stream runs on the client's shared transport, so its wire diagnostics
/// are the same `tracing` events every request emits. A shutdown signal
/// mid-stream flushes stdout, emits the cancellation envelope under
/// structured output, and returns cleanly; dropping the stream closes the
/// connection. A signal that arrives while the request is still waiting on
/// its response headers ends the run the same way.
pub(super) async fn stream_request_with_output(
    client: &Client,
    options: &RequestOptions,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<()> {
    let url = client.build_url_public(&options.target)?;
    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);
    let mut lines = tokio::select! {
        // `shutdown` registers its handlers on its first poll, so it goes first: no byte of the request goes out
        // before Ctrl+C and SIGTERM are caught, and a signal during connect is not left to the default disposition.
        biased;
        signal = &mut shutdown => {
            report_cancelled(signal, out, stdout, stderr);
            return Ok(());
        }
        opened = client.stream_request(options) => opened?,
    };
    out.status(stderr, &format!("Connecting to streaming endpoint: {url}"));

    out.status(stderr, "--- Streaming response started ---");
    out.status(stderr, "--- Press Ctrl+C to stop ---");

    loop {
        tokio::select! {
            biased;
            signal = &mut shutdown => {
                report_cancelled(signal, out, stdout, stderr);
                return Ok(());
            }
            next = lines.next_line() => match next? {
                Some(line) => out.print_stream_line(stdout, &line),
                None => break,
            },
        }
    }

    out.status(stderr, "--- End of stream ---");
    Ok(())
}

/// Flushes what was printed, writes the `cancelled` document under a
/// structured format, and says on stderr that a signal ended the stream.
fn report_cancelled(
    signal: ShutdownSignal,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) {
    let _ = stdout.flush();
    if out.format.is_structured() {
        let envelope = Envelope::Cancelled {
            exit_code: None,
            message: None,
            reason: Some(signal.reason().into()),
            context: serde_json::Map::new(),
            warnings: None,
        };
        out.print_response(stdout, &envelope.into_value());
    }
    out.status(stderr, "--- Stream cancelled by signal ---");
}
