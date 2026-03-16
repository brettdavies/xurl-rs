/// HTTP request building and execution for the X API.
///
/// Mirrors the Go `ApiClient` — builds requests with auth headers,
/// handles regular/streaming/multipart responses.
use std::io::{BufRead, BufReader};
use std::time::Duration;

use reqwest::blocking::{Client, multipart};

use crate::auth::Auth;
use crate::config::Config;
use crate::error::{Result, XurlError};

/// Common options for API requests.
#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    pub method: String,
    pub endpoint: String,
    pub headers: Vec<String>,
    pub data: String,
    pub auth_type: String,
    pub username: String,
    pub verbose: bool,
    pub trace: bool,
}

/// Options specific to multipart requests.
#[derive(Debug, Clone)]
pub struct MultipartOptions {
    pub request: RequestOptions,
    pub form_fields: std::collections::HashMap<String, String>,
    pub file_field: String,
    pub file_path: String,
    pub file_name: String,
    pub file_data: Vec<u8>,
}

/// Handles API requests with authentication.
pub struct ApiClient<'a> {
    base_url: String,
    client: Client,
    auth: &'a mut Auth,
}

impl<'a> ApiClient<'a> {
    /// Creates a new `ApiClient`.
    pub fn new(config: &Config, auth: &'a mut Auth) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url: config.api_base_url.clone(),
            client,
            auth,
        }
    }

    /// Builds the full URL from an endpoint (public accessor for command layer).
    #[must_use]
    pub fn build_url_public(&self, endpoint: &str) -> String {
        self.build_url(endpoint)
    }

    /// Builds the full URL from an endpoint.
    fn build_url(&self, endpoint: &str) -> String {
        if endpoint.to_lowercase().starts_with("http") {
            return endpoint.to_string();
        }

        let mut url = self.base_url.clone();
        if !url.ends_with('/') {
            url.push('/');
        }
        if let Some(stripped) = endpoint.strip_prefix('/') {
            url.push_str(stripped);
        } else {
            url.push_str(endpoint);
        }
        url
    }

    /// Sends a regular API request and returns the JSON response.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP method is invalid, the request fails,
    /// or the API returns an error status (>= 400).
    pub fn send_request(&mut self, options: &RequestOptions) -> Result<serde_json::Value> {
        let method = options.method.to_uppercase();
        let method = if method.is_empty() { "GET" } else { &method };
        let url = self.build_url(&options.endpoint);

        // Build the request
        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| XurlError::InvalidMethod(method.to_string()))?;

        let mut builder = self.client.request(req_method.clone(), &url);

        // Add body for POST/PUT/PATCH
        if !options.data.is_empty() && (method == "POST" || method == "PUT" || method == "PATCH") {
            // Detect content type
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

        // Add custom headers
        for header in &options.headers {
            if let Some((key, value)) = header.split_once(':') {
                builder = builder.header(key.trim(), value.trim());
            }
        }

        // Add auth header
        if let Ok(auth_header) =
            self.get_auth_header(method, &url, &options.auth_type, &options.username)
        {
            builder = builder.header("Authorization", auth_header);
        }

        // Add common headers
        builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));

        if options.trace {
            builder = builder.header("X-B3-Flags", "1");
        }

        if options.verbose {
            eprintln!("\x1b[1;34m> {method}\x1b[0m {url}");
        }

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

        let status = resp.status();
        let body = resp.text().unwrap_or_default();

        let json: serde_json::Value = if body.is_empty() {
            serde_json::json!({})
        } else if let Ok(v) = serde_json::from_str(&body) {
            v
        } else {
            if status.as_u16() >= 400 {
                return Err(XurlError::Http(format!("HTTP error: {status}")));
            }
            serde_json::json!({})
        };

        if status.as_u16() >= 400 {
            return Err(XurlError::api(json.to_string()));
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
        let url = self.build_url(&options.request.endpoint);

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

        // Add auth header
        if let Ok(auth_header) = self.get_auth_header(
            method,
            &url,
            &options.request.auth_type,
            &options.request.username,
        ) {
            builder = builder.header("Authorization", auth_header);
        }

        builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));

        if options.request.trace {
            builder = builder.header("X-B3-Flags", "1");
        }

        if options.request.verbose {
            eprintln!("\x1b[1;34m> {method}\x1b[0m {url}");
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
            return Err(XurlError::api(json.to_string()));
        }

        Ok(json)
    }

    /// Sends a streaming request — reads lines until EOF.
    ///
    /// Note: The binary uses `stream_request_with_output` in `cli::commands`
    /// for output-format awareness. This method is retained for library usage
    /// and tests.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP method is invalid, the request fails,
    /// the API returns an error status (>= 400), or a read error occurs.
    #[allow(dead_code)] // Public library API — used by consumers and integration tests
    pub fn stream_request(&mut self, options: &RequestOptions) -> Result<()> {
        let method = options.method.to_uppercase();
        let method = if method.is_empty() { "GET" } else { &method };
        let url = self.build_url(&options.endpoint);

        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| XurlError::InvalidMethod(method.to_string()))?;

        let mut builder = Client::builder()
            .timeout(None)
            .build()
            .unwrap_or_else(|_| Client::new())
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

        if let Ok(auth_header) =
            self.get_auth_header(method, &url, &options.auth_type, &options.username)
        {
            builder = builder.header("Authorization", auth_header);
        }

        builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));

        if options.trace {
            builder = builder.header("X-B3-Flags", "1");
        }

        if options.verbose {
            eprintln!("\x1b[1;34m> {method}\x1b[0m {url}");
        }

        eprintln!("Connecting to streaming endpoint: {}", options.endpoint);

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

        if resp.status().as_u16() >= 400 {
            let body = resp.text().unwrap_or_default();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                return Err(XurlError::api(json.to_string()));
            }
            return Err(XurlError::api(body));
        }

        eprintln!("--- Streaming response started ---");
        eprintln!("--- Press Ctrl+C to stop ---");

        let reader = BufReader::with_capacity(1024 * 1024, resp);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if line.is_empty() {
                        continue;
                    }
                    println!("{line}");
                }
                Err(e) => {
                    return Err(XurlError::Io(e.to_string()));
                }
            }
        }

        eprintln!("--- End of stream ---");
        Ok(())
    }

    /// Gets the authorization header for a request (public accessor for command layer).
    ///
    /// # Errors
    ///
    /// Returns an error if no valid auth method is found.
    pub fn get_auth_header_public(
        &mut self,
        method: &str,
        url: &str,
        auth_type: &str,
        username: &str,
    ) -> Result<String> {
        self.get_auth_header(method, url, auth_type, username)
    }

    /// Gets the authorization header for a request.
    fn get_auth_header(
        &mut self,
        method: &str,
        url: &str,
        auth_type: &str,
        username: &str,
    ) -> Result<String> {
        if !auth_type.is_empty() {
            return match auth_type.to_lowercase().as_str() {
                "oauth1" => self.auth.get_oauth1_header(method, url, None),
                "oauth2" => self.auth.get_oauth2_header(username),
                "app" => self.auth.get_bearer_token_header(),
                _ => Err(XurlError::auth(format!("invalid auth type: {auth_type}"))),
            };
        }

        // Try OAuth2 first — propagate errors if a token exists
        if self.auth.token_store.get_first_oauth2_token().is_some() {
            return self.auth.get_oauth2_header(username);
        }

        // Try OAuth1 — propagate errors if a token exists
        if self.auth.token_store.get_oauth1_tokens().is_some() {
            return self.auth.get_oauth1_header(method, url, None);
        }

        // Try Bearer
        if self.auth.token_store.has_bearer_token() {
            return self.auth.get_bearer_token_header();
        }

        Err(XurlError::auth(
            "NoAuthMethod: no authentication method available",
        ))
    }
}
