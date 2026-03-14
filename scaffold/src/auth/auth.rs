// WARNING: Generated code could not be parsed by syn for formatting.
// Run `cargo fmt` manually after fixing any syntax issues.

use std::collections::HashMap;
use base64;
use serde_json;
use reqwest;
use url;

#[derive(Debug, Clone, Default)]
pub struct Auth {
    pub token_store: Box<TokenStore /* todo: store.TokenStore */>,
    pub(crate) info_url: String,
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
    pub(crate) auth_url: String,
    pub(crate) token_url: String,
    pub(crate) redirect_uri: String,
    pub(crate) app_name: String,
}

/// NewAuth creates a new Auth object.
/// Credentials are resolved in order: env-var config → active app in .xurl store.
/// If env var credentials are present, they're also backfilled into any migrated
/// app that has tokens but no stored credentials.
pub fn new_auth(cfg: Box<Config /* todo: config.Config */>) -> Box<Auth /* todo: auth.Auth */> {
    let mut ts = store.new_token_store_with_credentials(cfg.client_id, cfg.client_secret);
    let mut client_id = cfg.client_id;
    let mut client_secret = cfg.client_secret;
    let mut app_name = cfg.app_name;
    let mut app = ts.resolve_app(app_name);
    if client_id == "" && app.is_some() {
        client_id = app.client_id;
    }
    if client_secret == "" && app.is_some() {
        client_secret = app.client_secret;
    }
    Box::new(Auth /* todo: auth.Auth */ { token_store: ts, info_url: cfg.info_url, client_id: client_id, client_secret: client_secret, auth_url: cfg.auth_url, token_url: cfg.token_url, redirect_uri: cfg.redirect_uri, app_name: app_name, ..Default::default() })
}

fn generate_signature(method: &str, url_str: &str, params: std::collections::HashMap<String, String>, consumer_secret: &str, token_secret: &str) -> anyhow::Result<String> {
    let mut parsed_url = url.parse(url_str)?;
    // if err != nil { ... } — handled by ? above
    let mut base_url = format!("{}://{}{}", parsed_url.scheme, parsed_url.host, parsed_url.path);
    let mut keys: Vec<String> = Default::default();
    for key in 0..params.len() {
        keys = { keys.push(key); keys.clone() };
    }
    keys.sort();
    let mut param_pairs: Vec<String> = Default::default();
    for key in keys.iter() {
        param_pairs = { param_pairs.push(format!("{}={}", encode(key), encode(params[key]))); param_pairs.clone() };
    }
    let mut param_string = param_pairs.join("&");
    let mut signature_base_string = format!("{}&{}&{}", method.to_uppercase(), encode(base_url), encode(param_string));
    let mut signing_key = format!("{}&{}", encode(consumer_secret), encode(token_secret));
    let mut h = hmac.new(sha1.new, signing_key.as_bytes().to_vec());
    h.write(signature_base_string.as_bytes().to_vec());
    let mut signature = base64.std_encoding.encode_to_string(h.sum(None));
    Ok(signature)
}

fn generate_nonce() -> String {
    let (n, _) = rand.int(rand.reader, big.new_int(1000000000));
    n.string()
}

fn generate_timestamp() -> String {
    format!("{}", chrono::Utc::now().unix())
}

fn encode(s: &str) -> String {
    url.query_escape(s)
}

fn generate_code_verifier_and_challenge() -> (String, String) {
    let mut b = vec![Default::default(); 32];
    rand.read(b);
    let mut verifier = base64.raw_url_encoding.encode_to_string(b);
    let mut h = sha256.new();
    h.write(verifier.as_bytes().to_vec());
    let mut challenge = base64.raw_url_encoding.encode_to_string(h.sum(None));
    (verifier, challenge)
}

fn get_o_auth2_scopes() -> Vec<String> {
    let mut read_scopes = vec!["tweet.read", "users.read", "bookmark.read", "follows.read", "list.read", "block.read", "mute.read", "like.read", "users.email", "dm.read"];
    let mut write_scopes = vec!["tweet.write", "tweet.moderate.write", "follows.write", "bookmark.write", "block.write", "mute.write", "like.write", "list.write", "media.write", "dm.write"];
    let mut other_scopes = vec!["offline.access", "space.read"];
    let mut scopes: Vec<String> = Default::default();
    scopes = { scopes.push(read_scopes); scopes.clone() };
    scopes = { scopes.push(write_scopes); scopes.clone() };
    scopes = { scopes.push(other_scopes); scopes.clone() };
    scopes
}

fn open_browser(url: &str) -> anyhow::Result<()> {
    let mut cmd: String = Default::default();
    let mut args: Vec<String> = Default::default();
    match runtime.goos {
        "windows" => {
            cmd = "cmd";
            args = vec!["/c", "start", url];
        }
        "darwin" => {
            cmd = "open";
            args = vec![url];
        }
        _ => {
            cmd = "xdg-open";
            args = vec![url];
        }
    }
    Ok(exec.command(cmd, args).start())
}


impl Auth {
    /// WithTokenStore sets the token store for the Auth object
    pub fn with_token_store(&mut self, token_store: Box<TokenStore /* todo: store.TokenStore */>) -> Box<Auth /* todo: auth.Auth */> {
        self.token_store = token_store;
        self
    }

    /// WithAppName sets the explicit app name override.
    pub fn with_app_name(&mut self, app_name: &str) -> Box<Auth /* todo: auth.Auth */> {
        self.app_name = app_name;
        let mut app = self.token_store.resolve_app(app_name);
        if app.is_some() {
            if self.client_id == "" {
                self.client_id = app.client_id;
            }
            if self.client_secret == "" {
                self.client_secret = app.client_secret;
            }
        }
        self
    }

    /// GetOAuth1Header gets the OAuth1 header for a request
    pub fn get_o_auth1_header(&mut self, method: &str, url_str: &str, additional_params: std::collections::HashMap<String, String>) -> anyhow::Result<String> {
        let mut token = self.token_store.get_o_auth1_tokens();
        if token.is_none() || token.o_auth1.is_none() {
            return Err((xurl_errors.new_auth_error("TokenNotFound", anyhow::anyhow!("OAuth1 token not found"))).into());
        }
        let mut oauth1_token = token.o_auth1;
        let mut parsed_url = url.parse(url_str)?;
        // if err != nil { ... } — handled by ? above
        let mut params = std::collections::HashMap::<String, String>::new();
        let mut query = parsed_url.query();
        for key in 0..query.len() {
            params.insert(key, query.get(key));
        }
        for (key, value) in additional_params.iter().enumerate() {
            params.insert(key, value);
        }
        params.insert("oauth_consumer_key".to_string(), oauth1_token.consumer_key);
        params.insert("oauth_nonce".to_string(), generate_nonce());
        params.insert("oauth_signature_method".to_string(), "HMAC-SHA1");
        params.insert("oauth_timestamp".to_string(), generate_timestamp());
        params.insert("oauth_token".to_string(), oauth1_token.access_token);
        params.insert("oauth_version".to_string(), "1.0");
        let mut signature = generate_signature(method, url_str, params, oauth1_token.consumer_secret, oauth1_token.token_secret)?;
        // if err != nil { ... } — handled by ? above
        let mut oauth_params: Vec<String> = Default::default();
        oauth_params = { oauth_params.push(format!("oauth_consumer_key=\"{}\"", encode(oauth1_token.consumer_key))); oauth_params.clone() };
        oauth_params = { oauth_params.push(format!("oauth_nonce=\"{}\"", encode(params["oauth_nonce"]))); oauth_params.clone() };
        oauth_params = { oauth_params.push(format!("oauth_signature=\"{}\"", encode(signature))); oauth_params.clone() };
        oauth_params = { oauth_params.push(format!("oauth_signature_method=\"{}\"", encode("HMAC-SHA1"))); oauth_params.clone() };
        oauth_params = { oauth_params.push(format!("oauth_timestamp=\"{}\"", encode(params["oauth_timestamp"]))); oauth_params.clone() };
        oauth_params = { oauth_params.push(format!("oauth_token=\"{}\"", encode(oauth1_token.access_token))); oauth_params.clone() };
        oauth_params = { oauth_params.push(format!("oauth_version=\"{}\"", encode("1.0"))); oauth_params.clone() };
        Ok("OAuth " + oauth_params.join(", "))
    }

    /// GetOAuth2Token gets or refreshes an OAuth2 token
    pub fn get_o_auth2_header(&mut self, username: &str) -> anyhow::Result<String> {
        let mut token: Box<Token /* todo: store.Token */> = Default::default();
        if username != "" {
            token = self.token_store.get_o_auth2_token(username);
        } else {
            token = self.token_store.get_first_o_auth2_token();
        }
        if token.is_none() {
            Ok(self.o_auth2_flow(username))
        }
        let mut access_token = self.refresh_o_auth2_token(username)?;
        // if err != nil { ... } — handled by ? above
        Ok("Bearer " + access_token)
    }

    /// OAuth2Flow starts the OAuth2 flow
    pub fn o_auth2_flow(&mut self, username: &str) -> anyhow::Result<String> {
        let mut config = Box::new(oauth2::Config { client_id: self.client_id, client_secret: self.client_secret, endpoint: Endpoint /* todo: oauth2.Endpoint */ { auth_url: self.auth_url, token_url: self.token_url, ..Default::default() }, redirect_url: self.redirect_uri, scopes: get_o_auth2_scopes(), ..Default::default() });
        let mut b = vec![Default::default(); 32];
        rand.read(b)?;
        // if err != nil { ... } — handled by ? above
        let mut state = base64.std_encoding.encode_to_string(b);
        let (verifier, challenge) = generate_code_verifier_and_challenge();
        let mut auth_url = config.auth_code_url(state, oauth2.set_auth_url_param("code_challenge", challenge), oauth2.set_auth_url_param("code_challenge_method", "S256"));
        let mut err = open_browser(auth_url);
        // if err != nil { ... } — handled by ? above
        let mut code_chan = tokio::sync::mpsc::channel(1);
        let mut callback = |code: String, received_state: String| -> anyhow::Error {
        if received_state != state {
            Ok(xurl_errors.new_auth_error("InvalidState", anyhow::anyhow!("invalid state parameter")))
        }
        if code == "" {
            Ok(xurl_errors.new_auth_error("InvalidCode", anyhow::anyhow!("empty authorization code")))
        }
        code_chan.send(code).await;
        Ok(())
    };
        std::thread::spawn(move || {
        let mut parsed_url = url.parse(a.redirect_uri)?;
        // if err != nil { ... } — handled by ? above
        let mut port = 8080;
        if parsed_url.port() != "" {
            fmt.sscanf(parsed_url.port(), "%d", &port);
        }
        let mut err = start_listener(port, callback);
        // if err != nil { ... } — handled by ? above
        });
        let mut code: String = Default::default();
        // select { } — requires tokio::select!
        todo!("select");
        let mut token = config.exchange(context.background(), code, oauth2.set_auth_url_param("code_verifier", verifier))?;
        // if err != nil { ... } — handled by ? above
        let mut username_str: String = Default::default();
        if username != "" {
            username_str = username;
        } else {
            let mut fetched_username = a.fetch_username(token.access_token)?;
            // if err != nil { ... } — handled by ? above
            username_str = fetched_username;
        }
        let mut expiration_time = chrono::Utc::now().add(time.duration(token.expiry.unix() - chrono::Utc::now().unix()) * time.second).unix() as u64;
        err = a.token_store.save_o_auth2_token(username_str, token.access_token, token.refresh_token, expiration_time);
        // if err != nil { ... } — handled by ? above
        Ok(token.access_token)
    }

    /// RefreshOAuth2Token validates and refreshes an OAuth2 token if needed
    pub fn refresh_o_auth2_token(&mut self, username: &str) -> anyhow::Result<String> {
        let mut token: Box<Token /* todo: store.Token */> = Default::default();
        if username != "" {
            token = self.token_store.get_o_auth2_token(username);
        } else {
            token = self.token_store.get_first_o_auth2_token();
        }
        if token.is_none() || token.o_auth2.is_none() {
            return Err((xurl_errors.new_auth_error("TokenNotFound", anyhow::anyhow!("oauth2 token not found"))).into());
        }
        let mut current_time = chrono::Utc::now().unix();
        if current_time as u64 < token.o_auth2.expiration_time {
            Ok(token.o_auth2.access_token)
        }
        let mut config = Box::new(oauth2::Config { client_id: self.client_id, client_secret: self.client_secret, endpoint: Endpoint /* todo: oauth2.Endpoint */ { token_url: self.token_url, ..Default::default() }, ..Default::default() });
        let mut token_source = config.token_source(context.background(), Box::new(oauth2::Token { refresh_token: token.o_auth2.refresh_token, ..Default::default() }));
        let mut new_token = token_source.token()?;
        // if err != nil { ... } — handled by ? above
        let mut username_str: String = Default::default();
        if username != "" {
            username_str = username;
        } else {
            let mut fetched_username = self.fetch_username(new_token.access_token)?;
            // if err != nil { ... } — handled by ? above
            username_str = fetched_username;
        }
        let mut expiration_time = chrono::Utc::now().add(time.duration(new_token.expiry.unix() - chrono::Utc::now().unix()) * time.second).unix() as u64;
        err = self.token_store.save_o_auth2_token(username_str, new_token.access_token, new_token.refresh_token, expiration_time);
        // if err != nil { ... } — handled by ? above
        Ok(new_token.access_token)
    }

    /// GetBearerTokenHeader gets the bearer token from the token store
    pub fn get_bearer_token_header(&mut self) -> anyhow::Result<String> {
        let mut token = self.token_store.get_bearer_token();
        if token.is_none() {
            return Err((xurl_errors.new_auth_error("TokenNotFound", anyhow::anyhow!("bearer token not found"))).into());
        }
        Ok("Bearer " + token.bearer)
    }

    fn fetch_username(&mut self, access_token: &str) -> anyhow::Result<String> {
        let mut req = http.new_request("GET", self.info_url, None)?;
        // if err != nil { ... } — handled by ? above
        req.header.add("Authorization", "Bearer " + access_token);
        let mut client = Box::new(reqwest::Client::default());
        let mut resp = client.do(req)?;
        // if err != nil { ... } — handled by ? above
        // defer: resp.body.close()
        let _defer = scopeguard::guard((), |_| { resp.body.close(); });
        let mut body = { let mut buf = String::new(); resp.body.read_to_string(&mut buf)?; buf }?;
        // if err != nil { ... } — handled by ? above
        let mut data: std::collections::HashMap<String, Box<dyn std::any::Any>> = Default::default();
        let mut err = serde_json::from_str(&body);
        // if err != nil { ... } — handled by ? above
        if data["data"].is_some() {
            let (user_data, ok) = /* type assert */ data["data"].downcast_ref::<std::collections::HashMap<String, Box<dyn std::any::Any>>>();
            if ok {
                let (username, ok) = /* type assert */ user_data["username"].downcast_ref::<String>();
                if ok {
                    Ok(username)
                }
            }
        }
        return Err((xurl_errors.new_auth_error("UsernameNotFound", anyhow::anyhow!("username not found when fetching username"))).into());
    }

}
