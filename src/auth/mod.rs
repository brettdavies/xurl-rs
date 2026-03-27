/// Authentication orchestration — `OAuth2` PKCE, `OAuth1` HMAC-SHA1, Bearer.
///
/// Mirrors the Go `auth.Auth` struct. Credentials are resolved in order:
/// env-var config -> active app in `.xurl` store.
pub(crate) mod callback;
pub mod oauth1;
pub mod oauth2;
pub mod pending;

use crate::config::Config;
use crate::error::{Result, XurlError};
use crate::store::TokenStore;

/// Manages authentication for X API requests.
#[allow(clippy::struct_field_names)]
pub struct Auth {
    pub token_store: TokenStore,
    info_url: String,
    client_id: String,
    client_secret: String,
    auth_url: String,
    token_url: String,
    redirect_uri: String,
    app_name: String,
}

impl Auth {
    /// Creates a new `Auth` object. Credentials are resolved: env vars -> active app.
    #[must_use]
    pub fn new(cfg: &Config) -> Self {
        let ts = TokenStore::with_credentials(&cfg.client_id, &cfg.client_secret);

        let mut client_id = cfg.client_id.clone();
        let mut client_secret = cfg.client_secret.clone();
        let app_name = cfg.app_name.clone();

        let app = ts.resolve_app(&app_name);
        if client_id.is_empty() {
            client_id.clone_from(&app.client_id);
        }
        if client_secret.is_empty() {
            client_secret.clone_from(&app.client_secret);
        }

        Self {
            token_store: ts,
            info_url: cfg.info_url.clone(),
            client_id,
            client_secret,
            auth_url: cfg.auth_url.clone(),
            token_url: cfg.token_url.clone(),
            redirect_uri: cfg.redirect_uri.clone(),
            app_name,
        }
    }

    /// Sets the explicit app name override.
    pub fn with_app_name(&mut self, app_name: &str) {
        self.app_name = app_name.to_string();
        let app = self.token_store.resolve_app(app_name);
        if self.client_id.is_empty() {
            self.client_id = app.client_id.clone();
        }
        if self.client_secret.is_empty() {
            self.client_secret = app.client_secret.clone();
        }
    }

    /// Gets the `OAuth1` Authorization header for a request.
    ///
    /// # Errors
    ///
    /// Returns an error if no `OAuth1` token is found or signature generation fails.
    pub fn get_oauth1_header(
        &self,
        method: &str,
        url_str: &str,
        additional_params: Option<&std::collections::BTreeMap<String, String>>,
    ) -> Result<String> {
        let token = self
            .token_store
            .get_oauth1_tokens()
            .ok_or_else(|| XurlError::auth("TokenNotFound: OAuth1 token not found"))?;

        let oauth1_token = token
            .oauth1
            .as_ref()
            .ok_or_else(|| XurlError::auth("TokenNotFound: OAuth1 token not found"))?;

        oauth1::build_oauth1_header(method, url_str, oauth1_token, additional_params)
    }

    /// Gets or refreshes an `OAuth2` token and returns the Authorization header.
    ///
    /// # Errors
    ///
    /// Returns an error if the `OAuth2` flow fails or token refresh fails.
    pub fn get_oauth2_header(&mut self, username: &str) -> Result<String> {
        let token = if username.is_empty() {
            self.token_store.get_first_oauth2_token().cloned()
        } else {
            self.token_store.get_oauth2_token(username).cloned()
        };

        if token.is_none() {
            let access_token = self.oauth2_flow(username)?;
            return Ok(format!("Bearer {access_token}"));
        }

        let access_token = self.refresh_oauth2_token(username)?;
        Ok(format!("Bearer {access_token}"))
    }

    /// Starts the `OAuth2` PKCE flow.
    ///
    /// # Errors
    ///
    /// Returns an error if the authorization flow, token exchange, or username
    /// resolution fails.
    pub fn oauth2_flow(&mut self, username: &str) -> Result<String> {
        oauth2::run_oauth2_flow(self, username)
    }

    /// Validates and refreshes an `OAuth2` token if needed.
    ///
    /// # Errors
    ///
    /// Returns an error if no token is found or the refresh request fails.
    pub fn refresh_oauth2_token(&mut self, username: &str) -> Result<String> {
        oauth2::refresh_oauth2_token(self, username)
    }

    /// Gets the bearer token Authorization header.
    ///
    /// # Errors
    ///
    /// Returns an error if no bearer token is found in the token store.
    pub fn get_bearer_token_header(&self) -> Result<String> {
        let token = self
            .token_store
            .get_bearer_token()
            .ok_or_else(|| XurlError::auth("TokenNotFound: bearer token not found"))?;

        let bearer = token
            .bearer
            .as_ref()
            .ok_or_else(|| XurlError::auth("TokenNotFound: bearer token not found"))?;

        Ok(format!("Bearer {bearer}"))
    }

    /// Fetches the username for an access token from the /2/users/me endpoint.
    pub(crate) fn fetch_username(&self, access_token: &str) -> Result<String> {
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(&self.info_url)
            .header("Authorization", format!("Bearer {access_token}"))
            .send()
            .map_err(|e| XurlError::auth_with_cause("NetworkError", &e))?;

        let body: serde_json::Value = resp
            .json()
            .map_err(|e| XurlError::auth_with_cause("JSONDeserializationError", &e))?;

        body.get("data")
            .and_then(|d| d.get("username"))
            .and_then(|u| u.as_str())
            .map(std::string::ToString::to_string)
            .ok_or_else(|| {
                XurlError::auth("UsernameNotFound: username not found when fetching username")
            })
    }

    /// Replaces the token store (used in integration tests).
    ///
    /// Re-resolves credentials from the new store's active app when
    /// they came from the old store (not from config/env vars), so
    /// stale credentials from the real `~/.xurl` don't leak into tests.
    #[allow(dead_code)] // Public library API — used by consumers and integration tests
    #[must_use]
    pub fn with_token_store(mut self, token_store: TokenStore) -> Self {
        let old_app = self.token_store.resolve_app(&self.app_name);
        let new_app = token_store.resolve_app(&self.app_name);
        if self.client_id == old_app.client_id {
            self.client_id = new_app.client_id.clone();
        }
        if self.client_secret == old_app.client_secret {
            self.client_secret = new_app.client_secret.clone();
        }
        self.token_store = token_store;
        self
    }

    /// Returns a reference to the token store.
    #[allow(dead_code)] // Public library API — used by consumers and integration tests
    #[must_use]
    pub fn token_store(&self) -> &TokenStore {
        &self.token_store
    }

    // Accessors
    #[must_use]
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
    #[must_use]
    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }
    #[must_use]
    pub fn auth_url(&self) -> &str {
        &self.auth_url
    }
    #[must_use]
    pub fn token_url(&self) -> &str {
        &self.token_url
    }
    #[must_use]
    pub fn redirect_uri(&self) -> &str {
        &self.redirect_uri
    }
}
