// WARNING: Generated code could not be parsed by syn for formatting.
// Run `cargo fmt` manually after fixing any syntax issues.

use std::collections::HashMap;
use serde_json;
use reqwest;

/// RequestOptions contains common options for API requests
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

/// MultipartOptions contains options specific to multipart requests
#[derive(Debug, Clone, Default)]
pub struct MultipartOptions {
    // embedded: github.com/xdevplatform/xurl/api.RequestOptions
    pub github_com_xdevplatform_xurl_api_request_options: RequestOptions /* todo: api.RequestOptions */,
    pub form_fields: std::collections::HashMap<String, String>,
    pub file_field: String,
    pub file_path: String,
    pub file_name: String,
    pub file_data: Vec<u8>,
}

/// ApiClient handles API requests
#[derive(Debug, Clone, Default)]
pub struct ApiClient {
    pub(crate) url: String,
    pub(crate) client: Box<reqwest::Client>,
    pub(crate) auth: Box<Auth /* todo: auth.Auth */>,
}

/// Client is an interface for API clients
pub trait Client {
    fn build_request(&self, request_options: RequestOptions /* todo: api.RequestOptions */) -> anyhow::Result<Box<reqwest::Request>>;
    fn build_multipart_request(&self, options: MultipartOptions /* todo: api.MultipartOptions */) -> anyhow::Result<Box<reqwest::Request>>;
    fn send_request(&self, options: RequestOptions /* todo: api.RequestOptions */) -> anyhow::Result<serde_json::Value>;
    fn stream_request(&self, options: RequestOptions /* todo: api.RequestOptions */) -> anyhow::Result<()>;
    fn send_multipart_request(&self, options: MultipartOptions /* todo: api.MultipartOptions */) -> anyhow::Result<serde_json::Value>;
}

/// NewApiClient creates a new ApiClient
pub fn new_api_client(config: Box<Config /* todo: config.Config */>, auth: Box<Auth /* todo: auth.Auth */>) -> Box<ApiClient /* todo: api.ApiClient */> {
    Box::new(ApiClient /* todo: api.ApiClient */ { url: config.api_base_url, client: Box::new(reqwest::Client { timeout: 30 * time.second, ..Default::default() }), auth: auth, ..Default::default() })
}


impl Client for ApiClient {
    /// BuildRequest builds an HTTP request
    fn build_request(&mut self, request_options: RequestOptions /* todo: api.RequestOptions */) -> anyhow::Result<Box<reqwest::Request>> {
        let mut http_method = request_options.method.to_uppercase();
        let mut body: Box<dyn std::io::Read> = Default::default();
        let mut content_type = "";
        if request_options.data != "" && (http_method == "POST" || http_method == "PUT" || http_method == "PATCH") {
            body = bytes.new_buffer_string(request_options.data);
            let mut js: serde_json::Value = Default::default();
            if serde_json::from_str(&request_options.data.as_bytes().to_vec()).is_none() {
                content_type = "application/json";
            } else {
                content_type = "application/x-www-form-urlencoded";
            }
        }
        Ok(self.build_base_request(request_options.method, request_options.endpoint, body, content_type, request_options.headers, request_options.auth_type, request_options.username, request_options.trace))
    }

    /// BuildMultipartRequest builds an HTTP request with multipart form data
    fn build_multipart_request(&mut self, options: MultipartOptions /* todo: api.MultipartOptions */) -> anyhow::Result<Box<reqwest::Request>> {
        let mut body = Box::new(Vec<u8>::default());
        let mut writer = multipart.new_writer(body);
        if options.file_field != "" && options.file_path != "" {
            let mut file = os.open(options.file_path)?;
            // if err != nil { ... } — handled by ? above
            // defer: file.close()
            let _defer = scopeguard::guard((), |_| { file.close(); });
            let mut part = writer.create_form_file(options.file_field, std::path::Path::new(&options.file_path).file_name().map(|n| n.to_string_lossy().to_string()))?;
            // if err != nil { ... } — handled by ? above
            io.copy(part, file)?;
            // if err != nil { ... } — handled by ? above
        } else if options.file_field != "" && options.file_data.len() > 0 {
            let mut part = writer.create_form_file(options.file_field, options.file_name)?;
            // if err != nil { ... } — handled by ? above
            part.write(options.file_data)?;
            // if err != nil { ... } — handled by ? above
        }
        for (key, value) in options.form_fields.iter().enumerate() {
            let mut err = writer.write_field(key, value);
            // if err != nil { ... } — handled by ? above
        }
        let mut err = writer.close();
        // if err != nil { ... } — handled by ? above
        Ok(self.build_base_request(options.method, options.endpoint, body, writer.form_data_content_type(), options.headers, options.auth_type, options.username, options.trace))
    }

    /// SendRequest sends an HTTP request
    fn send_request(&mut self, options: RequestOptions /* todo: api.RequestOptions */) -> anyhow::Result<serde_json::Value> {
        let mut req = self.build_request(options)?;
        // if err != nil { ... } — handled by ? above
        self.log_request(req, options.verbose);
        let mut resp = self.client.do(req)?;
        // if err != nil { ... } — handled by ? above
        // defer: resp.body.close()
        let _defer = scopeguard::guard((), |_| { resp.body.close(); });
        Ok(self.process_response(resp, options.verbose))
    }

    /// SendMultipartRequest sends an HTTP request with multipart form data
    fn send_multipart_request(&mut self, options: MultipartOptions /* todo: api.MultipartOptions */) -> anyhow::Result<serde_json::Value> {
        let mut req = self.build_multipart_request(options)?;
        // if err != nil { ... } — handled by ? above
        self.log_request(req, options.verbose);
        let mut resp = self.client.do(req)?;
        // if err != nil { ... } — handled by ? above
        // defer: resp.body.close()
        let _defer = scopeguard::guard((), |_| { resp.body.close(); });
        Ok(self.process_response(resp, options.verbose))
    }

    /// StreamRequest sends an HTTP request and streams the response
    fn stream_request(&mut self, options: RequestOptions /* todo: api.RequestOptions */) -> anyhow::Result<()> {
        let mut req = self.build_request(options)?;
        // if err != nil { ... } — handled by ? above
        if options.verbose {
            print!("\033[1;34m> {}\033[0m {}\n", req.method, req.url);
            for (key, values) in req.header.iter().enumerate() {
                for value in values.iter() {
                    print!("\033[1;36m> {}\033[0m: {}\n", key, value);
                }
            }
            println!();
        }
        let mut client = Box::new(reqwest::Client { timeout: 0, ..Default::default() });
        print!("\033[1;32mConnecting to streaming endpoint: {}\033[0m\n", options.endpoint);
        let mut resp = client.do(req)?;
        // if err != nil { ... } — handled by ? above
        // defer: resp.body.close()
        let _defer = scopeguard::guard((), |_| { resp.body.close(); });
        if options.verbose {
            print!("\033[1;31m< {}\033[0m\n", resp.status);
            for (key, values) in resp.header.iter().enumerate() {
                for value in values.iter() {
                    print!("\033[1;32m< {}\033[0m: {}\n", key, value);
                }
            }
            println!();
        }
        if resp.status_code >= 400 {
            let mut body = { let mut buf = String::new(); resp.body.read_to_string(&mut buf)?; buf }?;
            // if err != nil { ... } — handled by ? above
            let mut js: serde_json::Value = Default::default();
            let mut err = serde_json::from_str(&body);
            // if err != nil { ... } — handled by ? above
            Ok(xurl_errors.new_api_error(js))
        }
        let mut scanner = bufio.new_scanner(resp.body);
        // decl: const
        let mut buf = vec![Default::default(); max_scan_token_size];
        scanner.buffer(buf, max_scan_token_size);
        println!("{:?}", "\033[1;32m--- Streaming response started ---\033[0m");
        println!("{:?}", "\033[1;32m--- Press Ctrl+C to stop ---\033[0m");
        while scanner.scan() {
            let mut line = scanner.text();
            if line == "" {
                continue;
            }
            println!("{:?}", line);
        }
        let mut err = scanner.err();
        // if err != nil { ... } — handled by ? above
        println!("{:?}", "\033[1;32m--- End of stream ---\033[0m");
        Ok(())
    }

}

impl ApiClient {
    /// buildBaseRequest creates the base HTTP request with common headers and settings
    fn build_base_request(&mut self, method: &str, endpoint: &str, body: Box<dyn std::io::Read>, content_type: &str, headers: &[String], auth_type: &str, username: &str, trace: bool) -> anyhow::Result<Box<reqwest::Request>> {
        let mut http_method = method.to_uppercase();
        let mut url = endpoint;
        if !endpoint.to_lowercase().starts_with("http") {
            url = self.url;
            if !url.ends_with("/") {
                url += "/";
            }
            if endpoint.starts_with("/") {
                url += endpoint[1..];
            } else {
                url += endpoint;
            }
        }
        let mut req = http.new_request(http_method, url, body)?;
        // if err != nil { ... } — handled by ? above
        for header in headers.iter() {
            let mut parts = strings.split_n(header, ":", 2);
            if parts.len() == 2 {
                req.header.add(parts[0].trim(), parts[1].trim());
            }
        }
        if content_type != "" {
            req.header.set("Content-Type", content_type);
        }
        if req.header.get("Authorization") == "" {
            let mut auth_header = self.get_auth_header(http_method, url, auth_type, username)?;
            req.header.add("Authorization", auth_header);
        }
        req.header.add("User-Agent", "xurl/" + version.version);
        if trace {
            req.header.add("X-B3-Flags", "1");
        }
        Ok(req)
    }

    /// GetAuthHeader gets the authorization header for a request
    fn get_auth_header(&mut self, method: &str, url: &str, auth_type: &str, username: &str) -> anyhow::Result<String> {
        if self.auth.is_none() {
            return Err((xurl_errors.new_auth_error("AuthNotSet", anyhow::anyhow!("auth not set"))).into());
        }
        if auth_type != "" {
            match auth_type.to_lowercase() {
                "oauth1" => {
                    Ok(self.auth.get_o_auth1_header(method, url, None))
                }
                "oauth2" => {
                    Ok(self.auth.get_o_auth2_header(username))
                }
                "app" => {
                    Ok(self.auth.get_bearer_token_header())
                }
                _ => {
                    return Err((xurl_errors.new_auth_error("InvalidAuthType", anyhow::anyhow!("invalid auth type: {}", auth_type))).into());
                }
            }
        }
        let mut token = self.auth.token_store.get_first_o_auth2_token();
        if token.is_some() {
            let mut access_token = self.auth.get_o_auth2_header(username)?;
            Ok(access_token)
        }
        token = self.auth.token_store.get_o_auth1_tokens();
        if token.is_some() {
            let mut auth_header = self.auth.get_o_auth1_header(method, url, None)?;
            Ok(auth_header)
        }
        let mut bearer_token = self.auth.get_bearer_token_header()?;
        Ok(bearer_token)
        return Err((xurl_errors.new_auth_error("NoAuthMethod", anyhow::anyhow!("no authentication method available"))).into());
    }

    /// logRequest logs request details if verbose mode is enabled
    fn log_request(&mut self, req: Box<reqwest::Request>, verbose: bool) {
        if verbose {
            print!("\033[1;34m> {}\033[0m {}\n", req.method, req.url);
            for (key, values) in req.header.iter().enumerate() {
                for value in values.iter() {
                    print!("\033[1;36m> {}\033[0m: {}\n", key, value);
                }
            }
            println!();
        }
    }

    /// processResponse handles common response processing logic
    fn process_response(&mut self, resp: Box<reqwest::Response>, verbose: bool) -> anyhow::Result<serde_json::Value> {
        let mut response_body = { let mut buf = String::new(); resp.body.read_to_string(&mut buf)?; buf }?;
        // if err != nil { ... } — handled by ? above
        if verbose {
            print!("\033[1;31m< {}\033[0m\n", resp.status);
            for (key, values) in resp.header.iter().enumerate() {
                for value in values.iter() {
                    print!("\033[1;32m< {}\033[0m: {}\n", key, value);
                }
            }
            println!();
        }
        let mut js: serde_json::Value = Default::default();
        if response_body.len() > 0 {
            let mut err = serde_json::from_str(&response_body);
            // if err != nil { ... } — handled by ? above
        } else {
            js = json.raw_message("{}");
        }
        if resp.status_code >= 400 {
            return Err((xurl_errors.new_api_error(js)).into());
        }
        Ok(js)
    }

}
