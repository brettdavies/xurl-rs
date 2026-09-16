//! HTTP transport for the three response shapes — JSON, multipart, and
//! streaming — with request-header assembly and verbose wire diagnostics.

use std::io::{BufRead, BufReader};

use reqwest::blocking::{Client, multipart};

use crate::cli::output::OutputConfig;
use crate::error::{Result, XurlError};

use super::{ApiClient, MultipartOptions, RequestOptions};

impl ApiClient {
    /// Sends a regular API request and returns the JSON response.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP method is invalid, the request fails,
    /// or the API returns an error status (>= 400).
    pub fn send_request(&mut self, options: &RequestOptions) -> Result<serde_json::Value> {
        let method = options.method.to_uppercase();
        let method = if method.is_empty() { "GET" } else { &method };
        // Auth-matrix validation lives inside `get_auth_header` (called
        // below) so each request performs one matrix lookup, not two. The
        // explicit-auth branch there rejects with
        // `XurlError::AuthMethodMismatch` before any header is produced;
        // `no_auth: true` short-circuits past the call site entirely.
        let url = self.build_url(&options.target)?;

        // Build the request
        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| XurlError::InvalidMethod(method.to_string()))?;

        let mut builder = self.client.request(req_method.clone(), &url);

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
            let auth_header = self.get_auth_header(options)?;
            builder = builder.header("Authorization", auth_header);
        }

        // Add common headers (skip when the caller already supplied them).
        if !user_supplied_header(&options.headers, "User-Agent") {
            builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));
        }

        if options.trace && !user_supplied_header(&options.headers, "X-B3-Flags") {
            builder = builder.header("X-B3-Flags", "1");
        }

        if options.verbose {
            let mut err = std::io::stderr().lock();
            log_header_overrides(
                &self.out,
                &mut err,
                &options.headers,
                xurl_would_set_content_type,
                !options.no_auth,
                options.trace,
            );
        }

        if options.verbose {
            let mut err = std::io::stderr().lock();
            if self.out.use_color {
                self.out
                    .verbose(&mut err, &format!("\x1b[1;34m> {method}\x1b[0m {url}"));
            } else {
                self.out.verbose(&mut err, &format!("> {method} {url}"));
            }
        }

        let resp = builder.send()?;

        if options.verbose {
            let mut err = std::io::stderr().lock();
            log_response_headers(&self.out, &mut err, resp.status(), resp.headers());
        }

        let status = resp.status();
        let body = resp.text().unwrap_or_default();

        let json: serde_json::Value = if body.is_empty() {
            serde_json::json!({})
        } else if let Ok(v) = serde_json::from_str(&body) {
            v
        } else {
            if status.as_u16() >= 400 {
                return Err(XurlError::api(
                    status.as_u16(),
                    format!("HTTP error: {status}"),
                ));
            }
            serde_json::json!({})
        };

        if status.as_u16() >= 400 {
            return Err(XurlError::api(status.as_u16(), json.to_string()));
        }

        Ok(json)
    }

    /// Sends a multipart request (used for media upload chunks).
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP method is invalid, file I/O fails,
    /// the request fails, or the API returns an error status (>= 400).
    pub fn send_multipart_request(
        &mut self,
        options: &MultipartOptions,
    ) -> Result<serde_json::Value> {
        let method = options.request.method.to_uppercase();
        let method = if method.is_empty() { "POST" } else { &method };
        // Auth-matrix validation lives inside `get_auth_header` (called
        // below). `no_auth: true` short-circuits past the call site.
        let url = self.build_url(&options.request.target)?;

        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| XurlError::InvalidMethod(method.to_string()))?;

        let mut form = multipart::Form::new();

        // Add file from path or data
        if !options.file_field.is_empty() && !options.file_path.is_empty() {
            let part = multipart::Part::file(&options.file_path)
                .map_err(|e| XurlError::Io(format!("error opening file: {e}")))?;
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

        let mut builder = self.client.request(req_method, &url).multipart(form);

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
            let auth_header = self.get_auth_header(&options.request)?;
            builder = builder.header("Authorization", auth_header);
        }

        if !user_supplied_header(&options.request.headers, "User-Agent") {
            builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));
        }

        if options.request.trace && !user_supplied_header(&options.request.headers, "X-B3-Flags") {
            builder = builder.header("X-B3-Flags", "1");
        }

        if options.request.verbose {
            let mut err = std::io::stderr().lock();
            // Multipart's Content-Type is owned by `reqwest`'s multipart
            // builder (carries the boundary); xurl never explicitly sets it
            // here, so the override advisory skips Content-Type for this path.
            log_header_overrides(
                &self.out,
                &mut err,
                &options.request.headers,
                false,
                !options.request.no_auth,
                options.request.trace,
            );
            if self.out.use_color {
                self.out
                    .verbose(&mut err, &format!("\x1b[1;34m> {method}\x1b[0m {url}"));
            } else {
                self.out.verbose(&mut err, &format!("> {method} {url}"));
            }
        }

        let resp = builder.send()?;
        let status = resp.status();
        let body = resp.text().unwrap_or_default();

        let json: serde_json::Value = if body.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&body).unwrap_or(serde_json::json!({}))
        };

        if status.as_u16() >= 400 {
            return Err(XurlError::api(status.as_u16(), json.to_string()));
        }

        Ok(json)
    }

    /// Sends a streaming request — reads lines until EOF.
    ///
    /// All output flows through this client's configured `OutputConfig`
    /// (set via [`ApiClient::set_output`]); the CLI binary calls the
    /// `stream_request_with_output` helper in `cli::commands` which threads
    /// the runner's `OutputConfig` and writers in directly. Library callers
    /// pass their own `stdout`/`stderr` here so a streaming session can be
    /// captured in tests or redirected to a custom sink.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP method is invalid, the request fails,
    /// the API returns an error status (>= 400), or a read error occurs.
    #[allow(dead_code)] // Public library API — used by consumers and integration tests
    pub fn stream_request(
        &mut self,
        options: &RequestOptions,
        stdout: &mut dyn std::io::Write,
        stderr: &mut dyn std::io::Write,
    ) -> Result<()> {
        let method = options.method.to_uppercase();
        let method = if method.is_empty() { "GET" } else { &method };
        // Auth-matrix validation lives inside `get_auth_header` (called
        // below). Streaming honours the same fail-fast rule: an explicit
        // `--auth X` against an endpoint that doesn't accept `X` rejects
        // via `get_auth_header` before `builder.send()` opens any socket.
        let url = self.build_url(&options.target)?;

        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| XurlError::InvalidMethod(method.to_string()))?;

        let mut builder = Client::builder()
            .timeout(None)
            .build()
            .unwrap_or_else(|_| Client::new())
            .request(req_method, &url);

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
            let auth_header = self.get_auth_header(options)?;
            builder = builder.header("Authorization", auth_header);
        }

        if !user_supplied_header(&options.headers, "User-Agent") {
            builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));
        }

        if options.trace && !user_supplied_header(&options.headers, "X-B3-Flags") {
            builder = builder.header("X-B3-Flags", "1");
        }

        if options.verbose {
            log_header_overrides(
                &self.out,
                stderr,
                &options.headers,
                xurl_would_set_content_type,
                !options.no_auth,
                options.trace,
            );
            if self.out.use_color {
                self.out
                    .verbose(stderr, &format!("\x1b[1;34m> {method}\x1b[0m {url}"));
            } else {
                self.out.verbose(stderr, &format!("> {method} {url}"));
            }
        }

        self.out
            .status(stderr, &format!("Connecting to streaming endpoint: {url}"));

        let resp = builder.send()?;

        if options.verbose {
            log_response_headers(&self.out, stderr, resp.status(), resp.headers());
        }

        let resp_status = resp.status();
        if resp_status.as_u16() >= 400 {
            let body = resp.text().unwrap_or_default();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                return Err(XurlError::api(resp_status.as_u16(), json.to_string()));
            }
            return Err(XurlError::api(resp_status.as_u16(), body));
        }

        self.out
            .status(stderr, "--- Streaming response started ---");
        self.out.status(stderr, "--- Press Ctrl+C to stop ---");

        let reader = BufReader::with_capacity(1024 * 1024, resp);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if line.is_empty() {
                        continue;
                    }
                    self.out.print_stream_line(stdout, &line);
                }
                Err(e) => {
                    return Err(XurlError::Io(e.to_string()));
                }
            }
        }

        self.out.status(stderr, "--- End of stream ---");
        Ok(())
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

/// Emits an `info:` advisory for each xurl-added header that was suppressed
/// because the caller already supplied one via `options.headers`.
///
/// The four xurl-added headers are `Content-Type` (only when there's a body
/// to send), `Authorization` (only when `no_auth` is false), `User-Agent`
/// (always), and `X-B3-Flags` (only when `trace` is true). The corresponding
/// `would_*` booleans gate which headers are eligible for advisory in this
/// call site — `send_multipart_request` passes `would_set_content_type: false`
/// because reqwest's multipart builder owns the Content-Type.
fn log_header_overrides(
    out: &OutputConfig,
    err: &mut dyn std::io::Write,
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
            out.verbose(
                err,
                &format!("info: user-supplied {name} detected; skipping xurl append"),
            );
        }
    }
}

/// Emits the verbose response-header dump (`< STATUS`, `< key: value`, blank
/// line) through the supplied `OutputConfig`. Lives at module scope so
/// `send_request`, `send_multipart_request`, and `stream_request` share one
/// implementation.
fn log_response_headers(
    out: &OutputConfig,
    err: &mut dyn std::io::Write,
    status: reqwest::StatusCode,
    headers: &reqwest::header::HeaderMap,
) {
    if out.use_color {
        out.verbose(err, &format!("\x1b[1;31m< {status}\x1b[0m"));
        for (key, value) in headers {
            out.verbose(
                err,
                &format!("\x1b[1;32m< {key}\x1b[0m: {}", value.to_str().unwrap_or("")),
            );
        }
    } else {
        out.verbose(err, &format!("< {status}"));
        for (key, value) in headers {
            out.verbose(err, &format!("< {key}: {}", value.to_str().unwrap_or("")));
        }
    }
    out.verbose(err, "");
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
