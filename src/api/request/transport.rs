//! HTTP transport for the three response shapes (JSON, multipart, and
//! streaming) with request-header assembly and wire diagnostics emitted as
//! `tracing` events.

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_core::Stream;
use reqwest::multipart;

use crate::error::{Error, Result};

use super::{Client, MultipartOptions, RequestOptions};

/// Target of the wire diagnostics: one `DEBUG` event per request line
/// (`kind = "request"`, `method`, `url`), response status (`kind =
/// "status"`, `status`), response header (`kind = "header"`, `name`,
/// `value`), the end of a response (`kind = "end"`), and each header-override
/// note (`kind = "note"`, message), in the order a subscriber prints them.
pub const WIRE_TARGET: &str = "xurl::wire";

impl Client {
    /// Sends a regular API request and returns the JSON response.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP method is invalid, the request fails,
    /// or the API returns an error status (>= 400).
    pub async fn send_request(&self, options: &RequestOptions) -> Result<serde_json::Value> {
        self.send_request_with(options, self.request_timeout())
            .await
    }

    /// [`Self::send_request`] with `timeout` bounding this one request in
    /// place of the client-level bound.
    pub(crate) async fn send_request_with(
        &self,
        options: &RequestOptions,
        timeout: std::time::Duration,
    ) -> Result<serde_json::Value> {
        let method = options.method.to_uppercase();
        let method = if method.is_empty() { "GET" } else { &method };
        // Auth-matrix validation lives inside `get_auth_header` (called
        // below) so each request performs one matrix lookup, not two. The
        // explicit-auth branch there rejects with
        // `Error::AuthMethodMismatch` before any header is produced;
        // `no_auth: true` short-circuits past the call site entirely.
        let url = self.build_url(&options.target)?;

        // Build the request
        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| Error::InvalidMethod(method.to_string()))?;

        let mut builder = self
            .http()
            .request(req_method.clone(), &url)
            .timeout(timeout);

        // Add body for POST/PUT/PATCH. Content-Type is xurl's auto-detect
        // unless the caller already supplied one; the body itself is always
        // attached regardless.
        let xurl_would_set_content_type =
            !options.data.is_empty() && (method == "POST" || method == "PUT" || method == "PATCH");
        if xurl_would_set_content_type {
            if !user_supplied_header(&options.headers, "Content-Type") {
                let content_type =
                    if serde_json::from_str::<serde_json::Value>(&options.data).is_ok() {
                        "application/json"
                    } else {
                        "application/x-www-form-urlencoded"
                    };
                builder = builder.header("Content-Type", content_type);
            }
            builder = builder.body(options.data.clone());
        }

        // Add custom headers
        for header in &options.headers {
            if let Some((key, value)) = header.split_once(':') {
                builder = builder.header(key.trim(), value.trim());
            }
        }

        // Add auth header (skip if no_auth is set, or if the caller already
        // supplied an `Authorization` header in `options.headers`). When auth
        // resolution fails (e.g., TokenNotFound for the resolved app),
        // propagate the error so the user sees the real problem instead of
        // letting the request go out unauthenticated and surfacing as a
        // confusing 401 from upstream. The older "silently skip on Err" form
        // let auth bugs masquerade as upstream auth rejections.
        if !options.no_auth && !user_supplied_header(&options.headers, "Authorization") {
            let auth_header = self.get_auth_header(options).await?;
            builder = builder.header("Authorization", auth_header);
        }

        // Add common headers (skip when the caller already supplied them).
        if !user_supplied_header(&options.headers, "User-Agent") {
            builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));
        }

        if options.trace && !user_supplied_header(&options.headers, "X-B3-Flags") {
            builder = builder.header("X-B3-Flags", "1");
        }

        note_header_overrides(
            &options.headers,
            xurl_would_set_content_type,
            !options.no_auth,
            options.trace,
        );
        trace_request(method, &url);

        let resp = builder.send().await?;
        trace_response(resp.status(), resp.headers());

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

        let json: serde_json::Value = if body.is_empty() {
            serde_json::json!({})
        } else if let Ok(v) = serde_json::from_str(&body) {
            v
        } else {
            if status.as_u16() >= 400 {
                return Err(Error::api(status.as_u16(), format!("HTTP error: {status}")));
            }
            serde_json::json!({})
        };

        if status.as_u16() >= 400 {
            return Err(Error::api(status.as_u16(), json.to_string()));
        }

        Ok(json)
    }

    /// Sends a multipart request (used for media upload chunks).
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP method is invalid, file I/O fails,
    /// the request fails, or the API returns an error status (>= 400).
    pub async fn send_multipart_request(
        &self,
        options: &MultipartOptions,
    ) -> Result<serde_json::Value> {
        let method = options.request.method.to_uppercase();
        let method = if method.is_empty() { "POST" } else { &method };
        // Auth-matrix validation lives inside `get_auth_header` (called
        // below). `no_auth: true` short-circuits past the call site.
        let url = self.build_url(&options.request.target)?;

        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| Error::InvalidMethod(method.to_string()))?;

        let mut form = multipart::Form::new();

        // Add file from path or data
        if !options.file_field.is_empty() && !options.file_path.is_empty() {
            let part = multipart::Part::file(&options.file_path)
                .await
                .map_err(|e| Error::Io(format!("error opening file: {e}")))?;
            form = form.part(options.file_field.clone(), part);
        } else if !options.file_field.is_empty() && !options.file_data.is_empty() {
            let part = multipart::Part::bytes(options.file_data.clone())
                .file_name(options.file_name.clone());
            form = form.part(options.file_field.clone(), part);
        }

        // Add form fields
        for (key, value) in &options.form_fields {
            form = form.text(key.clone(), value.clone());
        }

        let mut builder = self
            .http()
            .request(req_method, &url)
            .timeout(self.request_timeout())
            .multipart(form);

        // Add custom headers
        for header in &options.request.headers {
            if let Some((key, value)) = header.split_once(':') {
                builder = builder.header(key.trim(), value.trim());
            }
        }

        // Add auth header (skip if no_auth is set, or if the caller already
        // supplied an `Authorization` header). Propagate auth errors rather
        // than silently sending the request unauthenticated; see the matching
        // propagation site in `send_request` for the rationale.
        if !options.request.no_auth
            && !user_supplied_header(&options.request.headers, "Authorization")
        {
            let auth_header = self.get_auth_header(&options.request).await?;
            builder = builder.header("Authorization", auth_header);
        }

        if !user_supplied_header(&options.request.headers, "User-Agent") {
            builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));
        }

        if options.request.trace && !user_supplied_header(&options.request.headers, "X-B3-Flags") {
            builder = builder.header("X-B3-Flags", "1");
        }

        // Multipart's Content-Type is owned by `reqwest`'s multipart builder
        // (it carries the boundary), so the override note skips Content-Type.
        note_header_overrides(
            &options.request.headers,
            false,
            !options.request.no_auth,
            options.request.trace,
        );
        trace_request(method, &url);

        let resp = builder.send().await?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

        let json: serde_json::Value = if body.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&body).unwrap_or(serde_json::json!({}))
        };

        if status.as_u16() >= 400 {
            return Err(Error::api(status.as_u16(), json.to_string()));
        }

        Ok(json)
    }

    /// Opens a streaming request and returns its lines as they arrive.
    ///
    /// The connection stays open until the returned stream is dropped or
    /// the server ends it; the caller decides what to print. Streaming
    /// requests carry no total timeout.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP method is invalid, the request fails,
    /// or the API returns an error status (>= 400).
    pub async fn stream_request(&self, options: &RequestOptions) -> Result<StreamLines> {
        let method = options.method.to_uppercase();
        let method = if method.is_empty() { "GET" } else { &method };
        // Auth-matrix validation lives inside `get_auth_header` (called
        // below). Streaming honours the same fail-fast rule: an explicit
        // `--auth X` against an endpoint that doesn't accept `X` rejects
        // via `get_auth_header` before `builder.send()` opens any socket.
        let url = self.build_url(&options.target)?;

        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| Error::InvalidMethod(method.to_string()))?;

        let mut builder = self.http().request(req_method, &url);

        let xurl_would_set_content_type = !options.data.is_empty();
        if xurl_would_set_content_type {
            if !user_supplied_header(&options.headers, "Content-Type") {
                let content_type =
                    if serde_json::from_str::<serde_json::Value>(&options.data).is_ok() {
                        "application/json"
                    } else {
                        "application/x-www-form-urlencoded"
                    };
                builder = builder.header("Content-Type", content_type);
            }
            builder = builder.body(options.data.clone());
        }

        for header in &options.headers {
            if let Some((key, value)) = header.split_once(':') {
                builder = builder.header(key.trim(), value.trim());
            }
        }

        if !options.no_auth && !user_supplied_header(&options.headers, "Authorization") {
            let auth_header = self.get_auth_header(options).await?;
            builder = builder.header("Authorization", auth_header);
        }

        if !user_supplied_header(&options.headers, "User-Agent") {
            builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));
        }

        if options.trace && !user_supplied_header(&options.headers, "X-B3-Flags") {
            builder = builder.header("X-B3-Flags", "1");
        }

        note_header_overrides(
            &options.headers,
            xurl_would_set_content_type,
            !options.no_auth,
            options.trace,
        );
        trace_request(method, &url);

        let resp = builder.send().await?;
        trace_response(resp.status(), resp.headers());

        let resp_status = resp.status();
        if resp_status.as_u16() >= 400 {
            let body = resp.text().await.unwrap_or_default();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                return Err(Error::api(resp_status.as_u16(), json.to_string()));
            }
            return Err(Error::api(resp_status.as_u16(), body));
        }

        Ok(StreamLines {
            body: Box::pin(resp.bytes_stream()),
            buf: Vec::new(),
            done: false,
        })
    }
}

type ByteStream = Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send>>;

/// Lines of a streaming response, delivered as they arrive.
///
/// A [`Stream`] of lines; [`Self::next_line`] is the same thing without a
/// stream combinator. Empty keep-alive lines are skipped. Dropping the
/// stream closes the connection.
pub struct StreamLines {
    body: ByteStream,
    buf: Vec<u8>,
    done: bool,
}

impl StreamLines {
    /// The next line, or `None` once the server has ended the stream.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] when the connection fails mid-stream.
    pub async fn next_line(&mut self) -> Result<Option<String>> {
        std::future::poll_fn(|cx| Pin::new(&mut *self).poll_next(cx))
            .await
            .transpose()
    }

    /// Removes and returns the first complete line in the buffer, without
    /// its `\n` and any `\r` before it.
    fn take_line(&mut self) -> Option<String> {
        let end = self.buf.iter().position(|&b| b == b'\n')?;
        let mut line: Vec<u8> = self.buf.drain(..=end).collect();
        line.pop();
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        Some(String::from_utf8_lossy(&line).into_owned())
    }
}

impl std::fmt::Debug for StreamLines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamLines").finish_non_exhaustive()
    }
}

impl Stream for StreamLines {
    type Item = Result<String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(line) = self.take_line() {
                if line.is_empty() {
                    continue;
                }
                return Poll::Ready(Some(Ok(line)));
            }
            if self.done {
                let rest = std::mem::take(&mut self.buf);
                let line = String::from_utf8_lossy(&rest).into_owned();
                let line = line.trim_end_matches('\r').to_string();
                return Poll::Ready((!line.is_empty()).then_some(Ok(line)));
            }
            match self.body.as_mut().poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => self.done = true,
                Poll::Ready(Some(Ok(chunk))) => self.buf.extend_from_slice(&chunk),
                Poll::Ready(Some(Err(e))) => {
                    self.done = true;
                    return Poll::Ready(Some(Err(Error::Io(e.to_string()))));
                }
            }
        }
    }
}

/// Returns `true` when any `"Name: Value"` entry in `headers` matches `name`
/// on a case-insensitive (ASCII) compare of the trimmed key.
///
/// Used by the three send paths to suppress xurl's own append of any header
/// the caller already supplied. `reqwest::RequestBuilder::header` uses
/// `HeaderMap::append`, so without this guard a header the caller passed
/// (Authorization, User-Agent, Content-Type, X-B3-Flags) would go out
/// alongside xurl's own value. HTTP allows multi-valued headers in the
/// abstract; many servers (and HTTP signatures like Authorization) treat
/// duplicates as undefined.
fn user_supplied_header(headers: &[String], name: &str) -> bool {
    headers
        .iter()
        .filter_map(|h| h.split_once(':'))
        .any(|(key, _)| key.trim().eq_ignore_ascii_case(name))
}

/// Emits one wire note for each xurl-added header that was suppressed because
/// the caller already supplied it via `options.headers`.
///
/// The four xurl-added headers are `Content-Type` (only when there's a body
/// to send), `Authorization` (only when `no_auth` is false), `User-Agent`
/// (always), and `X-B3-Flags` (only when `trace` is true). The corresponding
/// `would_*` booleans gate which headers are eligible for a note at this
/// call site; `send_multipart_request` passes `would_set_content_type: false`
/// because reqwest's multipart builder owns the Content-Type.
fn note_header_overrides(
    headers: &[String],
    would_set_content_type: bool,
    would_set_auth: bool,
    would_set_trace: bool,
) {
    let candidates: [(&str, bool); 4] = [
        ("Content-Type", would_set_content_type),
        ("Authorization", would_set_auth),
        ("User-Agent", true),
        ("X-B3-Flags", would_set_trace),
    ];
    for (name, xurl_wanted) in candidates {
        if xurl_wanted && user_supplied_header(headers, name) {
            tracing::debug!(
                target: WIRE_TARGET,
                kind = "note",
                "info: user-supplied {name} detected; skipping xurl append"
            );
        }
    }
}

/// Emits the request line as one wire event.
fn trace_request(method: &str, url: &str) {
    tracing::debug!(target: WIRE_TARGET, kind = "request", method, url);
}

/// Emits the response status, each header, and the end marker as wire
/// events, in the order `xr --verbose` prints them.
fn trace_response(status: reqwest::StatusCode, headers: &reqwest::header::HeaderMap) {
    tracing::debug!(target: WIRE_TARGET, kind = "status", status = %status);
    for (key, value) in headers {
        tracing::debug!(
            target: WIRE_TARGET,
            kind = "header",
            name = %key,
            value = value.to_str().unwrap_or("")
        );
    }
    tracing::debug!(target: WIRE_TARGET, kind = "end");
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── user_supplied_header tests ───────────────────────────────

    #[test]
    fn user_supplied_header_detects_canonical_case() {
        let headers = vec!["Authorization: Bearer foo".to_string()];
        assert!(user_supplied_header(&headers, "Authorization"));
    }

    #[test]
    fn user_supplied_header_is_case_insensitive_on_input_key() {
        for raw in [
            "authorization: Bearer foo",
            "AUTHORIZATION: Bearer foo",
            "aUtHoRiZaTiOn: Bearer foo",
        ] {
            assert!(
                user_supplied_header(&[raw.to_string()], "Authorization"),
                "did not detect: {raw}"
            );
        }
    }

    #[test]
    fn user_supplied_header_is_case_insensitive_on_query_name() {
        let headers = vec!["Authorization: Bearer foo".to_string()];
        for query in ["authorization", "AUTHORIZATION", "aUtHoRiZaTiOn"] {
            assert!(
                user_supplied_header(&headers, query),
                "did not detect with query: {query}"
            );
        }
    }

    #[test]
    fn user_supplied_header_ignores_surrounding_whitespace_on_key() {
        let headers = vec!["  Authorization  : Bearer foo".to_string()];
        assert!(user_supplied_header(&headers, "Authorization"));
    }

    #[test]
    fn user_supplied_header_false_for_empty_list() {
        let headers: Vec<String> = Vec::new();
        assert!(!user_supplied_header(&headers, "Authorization"));
        assert!(!user_supplied_header(&headers, "User-Agent"));
    }

    #[test]
    fn user_supplied_header_does_not_match_substring_keys() {
        let headers = vec![
            "Cookie: session=abc".to_string(),
            "X-Authorization-Hint: ignored".to_string(),
        ];
        assert!(!user_supplied_header(&headers, "Authorization"));
    }

    #[test]
    fn user_supplied_header_false_for_unparseable_entry() {
        let headers = vec!["malformed-no-colon".to_string()];
        assert!(!user_supplied_header(&headers, "Authorization"));
    }

    #[test]
    fn user_supplied_header_detects_each_xurl_added_header() {
        let headers = vec![
            "Content-Type: application/xml".to_string(),
            "Authorization: Bearer foo".to_string(),
            "User-Agent: custom/1.0".to_string(),
            "X-B3-Flags: 0".to_string(),
        ];
        assert!(user_supplied_header(&headers, "Content-Type"));
        assert!(user_supplied_header(&headers, "Authorization"));
        assert!(user_supplied_header(&headers, "User-Agent"));
        assert!(user_supplied_header(&headers, "X-B3-Flags"));
    }
}
