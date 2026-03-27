use super::TokenStore;
/// Token CRUD operations — save, get, clear for Bearer, OAuth2, OAuth1.
use super::types::{OAuth1Token, OAuth2Token, Token, TokenType};
use crate::error::Result;

#[allow(dead_code)] // Public library API — used by consumers and integration tests
impl TokenStore {
    // ── Save ─────────────────────────────────────────────────────────

    /// Saves a bearer token into the resolved app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn save_bearer_token(&mut self, token: &str) -> Result<()> {
        self.save_bearer_token_for_app("", token)
    }

    /// Saves a bearer token into the named app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn save_bearer_token_for_app(&mut self, app_name: &str, token: &str) -> Result<()> {
        let app = self.resolve_app_mut(app_name);
        app.bearer_token = Some(Token {
            token_type: TokenType::Bearer,
            bearer: Some(token.to_string()),
            oauth2: None,
            oauth1: None,
        });
        self.save_to_file()
    }

    /// Saves an `OAuth2` token into the resolved app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn save_oauth2_token(
        &mut self,
        username: &str,
        access_token: &str,
        refresh_token: &str,
        expiration_time: u64,
    ) -> Result<()> {
        self.save_oauth2_token_for_app("", username, access_token, refresh_token, expiration_time)
    }

    /// Saves an `OAuth2` token into the named app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn save_oauth2_token_for_app(
        &mut self,
        app_name: &str,
        username: &str,
        access_token: &str,
        refresh_token: &str,
        expiration_time: u64,
    ) -> Result<()> {
        let app = self.resolve_app_mut(app_name);
        app.oauth2_tokens.insert(
            username.to_string(),
            Token {
                token_type: TokenType::Oauth2,
                bearer: None,
                oauth2: Some(OAuth2Token {
                    access_token: access_token.to_string(),
                    refresh_token: refresh_token.to_string(),
                    expiration_time,
                }),
                oauth1: None,
            },
        );
        self.save_to_file()
    }

    /// Saves `OAuth1` tokens into the resolved app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn save_oauth1_tokens(
        &mut self,
        access_token: &str,
        token_secret: &str,
        consumer_key: &str,
        consumer_secret: &str,
    ) -> Result<()> {
        self.save_oauth1_tokens_for_app(
            "",
            access_token,
            token_secret,
            consumer_key,
            consumer_secret,
        )
    }

    /// Saves `OAuth1` tokens into the named app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn save_oauth1_tokens_for_app(
        &mut self,
        app_name: &str,
        access_token: &str,
        token_secret: &str,
        consumer_key: &str,
        consumer_secret: &str,
    ) -> Result<()> {
        let app = self.resolve_app_mut(app_name);
        app.oauth1_token = Some(Token {
            token_type: TokenType::Oauth1,
            bearer: None,
            oauth2: None,
            oauth1: Some(OAuth1Token {
                access_token: access_token.to_string(),
                token_secret: token_secret.to_string(),
                consumer_key: consumer_key.to_string(),
                consumer_secret: consumer_secret.to_string(),
            }),
        });
        self.save_to_file()
    }

    // ── Get ──────────────────────────────────────────────────────────

    /// Gets an `OAuth2` token for a username from the resolved app.
    #[must_use]
    pub fn get_oauth2_token(&self, username: &str) -> Option<&Token> {
        self.get_oauth2_token_for_app("", username)
    }

    /// Gets an `OAuth2` token for a username from the named app.
    #[must_use]
    pub fn get_oauth2_token_for_app(&self, app_name: &str, username: &str) -> Option<&Token> {
        let app = self.resolve_app(app_name);
        app.oauth2_tokens.get(username)
    }

    /// Gets the first `OAuth2` token from the resolved app.
    #[must_use]
    pub fn get_first_oauth2_token(&self) -> Option<&Token> {
        self.get_first_oauth2_token_for_app("")
    }

    /// Gets the default user's token, or the first `OAuth2` token from the named app.
    #[must_use]
    pub fn get_first_oauth2_token_for_app(&self, app_name: &str) -> Option<&Token> {
        let app = self.resolve_app(app_name);
        // Prefer the default user if one is set and still has a token
        if !app.default_user.is_empty()
            && let Some(token) = app.oauth2_tokens.get(&app.default_user)
        {
            return Some(token);
        }
        app.oauth2_tokens.values().next()
    }

    /// Gets `OAuth1` tokens from the resolved app.
    #[must_use]
    pub fn get_oauth1_tokens(&self) -> Option<&Token> {
        self.get_oauth1_tokens_for_app("")
    }

    /// Gets `OAuth1` tokens from the named app.
    #[must_use]
    pub fn get_oauth1_tokens_for_app(&self, app_name: &str) -> Option<&Token> {
        let app = self.resolve_app(app_name);
        app.oauth1_token.as_ref()
    }

    /// Gets the bearer token from the resolved app.
    #[must_use]
    pub fn get_bearer_token(&self) -> Option<&Token> {
        self.get_bearer_token_for_app("")
    }

    /// Gets the bearer token from the named app.
    #[must_use]
    pub fn get_bearer_token_for_app(&self, app_name: &str) -> Option<&Token> {
        let app = self.resolve_app(app_name);
        app.bearer_token.as_ref()
    }

    // ── Clear ────────────────────────────────────────────────────────

    /// Clears an `OAuth2` token for a username from the resolved app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn clear_oauth2_token(&mut self, username: &str) -> Result<()> {
        self.clear_oauth2_token_for_app("", username)
    }

    /// Clears an `OAuth2` token for a username from the named app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn clear_oauth2_token_for_app(&mut self, app_name: &str, username: &str) -> Result<()> {
        let app = self.resolve_app_mut(app_name);
        app.oauth2_tokens.remove(username);
        self.save_to_file()
    }

    /// Clears `OAuth1` tokens from the resolved app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn clear_oauth1_tokens(&mut self) -> Result<()> {
        self.clear_oauth1_tokens_for_app("")
    }

    /// Clears `OAuth1` tokens from the named app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn clear_oauth1_tokens_for_app(&mut self, app_name: &str) -> Result<()> {
        let app = self.resolve_app_mut(app_name);
        app.oauth1_token = None;
        self.save_to_file()
    }

    /// Clears the bearer token from the resolved app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn clear_bearer_token(&mut self) -> Result<()> {
        self.clear_bearer_token_for_app("")
    }

    /// Clears the bearer token from the named app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn clear_bearer_token_for_app(&mut self, app_name: &str) -> Result<()> {
        let app = self.resolve_app_mut(app_name);
        app.bearer_token = None;
        self.save_to_file()
    }

    /// Clears all tokens from the resolved app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn clear_all(&mut self) -> Result<()> {
        self.clear_all_for_app("")
    }

    /// Clears all tokens from the named app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn clear_all_for_app(&mut self, app_name: &str) -> Result<()> {
        let app = self.resolve_app_mut(app_name);
        app.oauth2_tokens.clear();
        app.oauth1_token = None;
        app.bearer_token = None;
        self.save_to_file()
    }

    // ── Query ────────────────────────────────────────────────────────

    /// Gets all `OAuth2` usernames from the resolved app.
    #[must_use]
    pub fn get_oauth2_usernames(&self) -> Vec<String> {
        self.get_oauth2_usernames_for_app("")
    }

    /// Gets all `OAuth2` usernames from the named app.
    #[must_use]
    pub fn get_oauth2_usernames_for_app(&self, app_name: &str) -> Vec<String> {
        let app = self.resolve_app(app_name);
        app.oauth2_tokens.keys().cloned().collect()
    }

    /// Checks if `OAuth1` tokens exist in the resolved app.
    #[must_use]
    pub fn has_oauth1_tokens(&self) -> bool {
        self.active_app()
            .is_some_and(|app| app.oauth1_token.is_some())
    }

    /// Checks if a bearer token exists in the resolved app.
    #[must_use]
    pub fn has_bearer_token(&self) -> bool {
        self.active_app()
            .is_some_and(|app| app.bearer_token.is_some())
    }
}
