//! Streaming request handler: SSE / chunked transfer support.
use std::io::Write;

use crate::api::{Client, RequestOptions};
use crate::cli::output::OutputConfig;
use crate::cli::shutdown::shutdown_signal;
use crate::error::Result;

/// Sends a streaming request with output-format awareness.
///
/// The stream runs on the client's shared transport, so its wire diagnostics
/// are the same `tracing` events every request emits. A shutdown signal
/// mid-stream flushes stdout, emits the cancellation envelope under
/// structured output, and returns cleanly; dropping the stream closes the
/// connection.
pub(super) async fn stream_request_with_output(
    client: &Client,
    options: &RequestOptions,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<()> {
    let url = client.build_url_public(&options.target)?;
    out.status(stderr, &format!("Connecting to streaming endpoint: {url}"));

    let mut lines = client.stream_request(options).await?;

    out.status(stderr, "--- Streaming response started ---");
    out.status(stderr, "--- Press Ctrl+C to stop ---");

    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            () = &mut shutdown => {
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
            next = lines.next_line() => match next? {
                Some(line) => out.print_stream_line(stdout, &line),
                None => break,
            },
        }
    }

    out.status(stderr, "--- End of stream ---");
    Ok(())
}
