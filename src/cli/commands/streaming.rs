/// Streaming request handler — SSE / chunked transfer support.
use crate::api::{ApiClient, RequestOptions};
use crate::error::{Result, XurlError};
use crate::output::OutputConfig;

/// Sends a streaming request with output-format awareness.
pub(super) fn stream_request_with_output(
    client: &mut ApiClient,
    options: &RequestOptions,
    out: &OutputConfig,
) -> Result<()> {
    use std::io::{BufRead, BufReader};

    let method = options.method.to_uppercase();
    let method = if method.is_empty() { "GET" } else { &method };
    let url = client.build_url_public(&options.endpoint);

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

    if !options.no_auth {
        if let Ok(auth_header) =
            client.get_auth_header_public(method, &url, &options.auth_type, &options.username)
        {
            builder = builder.header("Authorization", auth_header);
        }
    }

    builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));

    if options.trace {
        builder = builder.header("X-B3-Flags", "1");
    }

    if options.verbose {
        eprintln!("\x1b[1;34m> {method}\x1b[0m {url}");
    }

    out.status(&format!(
        "Connecting to streaming endpoint: {}",
        options.endpoint
    ));

    let resp = builder.send()?;

    if options.verbose {
        eprintln!("\x1b[1;31m< {}\x1b[0m", resp.status());
        for (key, value) in resp.headers() {
            eprintln!(
                "\x1b[1;32m< {}\x1b[0m: {}",
                key,
                value.to_str().unwrap_or("")
            );
        }
        eprintln!();
    }

    let resp_status = resp.status();
    if resp_status.as_u16() >= 400 {
        let body = resp.text().unwrap_or_default();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            return Err(XurlError::api(resp_status.as_u16(), json.to_string()));
        }
        return Err(XurlError::api(resp_status.as_u16(), body));
    }

    out.status("--- Streaming response started ---");
    out.status("--- Press Ctrl+C to stop ---");

    let reader = BufReader::with_capacity(1024 * 1024, resp);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                out.print_stream_line(&line);
            }
            Err(e) => {
                return Err(XurlError::Io(e.to_string()));
            }
        }
    }

    out.status("--- End of stream ---");
    Ok(())
}
