//! Token CRUD operations — save, get, clear for Bearer, OAuth2, OAuth1.

use super::TokenStore;
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
        self.update(|store| {
            let app = store.resolve_app_mut(app_name);
            app.bearer_token = Some(Token {
                token_type: TokenType::Bearer,
                bearer: Some(token.to_string()),
                oauth2: None,
                oauth1: None,
            });
            Ok(())
        })
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
        let token = OAuth2Token {
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
            expiration_time,
            user_id: None,
        };
        self.put_oauth2_token(app_name, Some(username), token, false)
    }

    /// Stores `token` in the named app: under `name`, or in the unnamed slot
    /// when there is none. `drop_unnamed` empties the unnamed slot in the
    /// same write, for a token that just left it for a name.
    pub(crate) fn put_oauth2_token(
        &mut self,
        app_name: &str,
        name: Option<&str>,
        token: OAuth2Token,
        drop_unnamed: bool,
    ) -> Result<()> {
        self.update(|store| {
            let app = store.resolve_app_mut(app_name);
            let token = Token {
                token_type: TokenType::Oauth2,
                bearer: None,
                oauth2: Some(token),
                oauth1: None,
            };
            match name {
                Some(name) => {
                    app.oauth2_tokens.insert(name.to_string(), token);
                    if drop_unnamed {
                        app.unnamed_oauth2_token = None;
                    }
                }
                None => app.unnamed_oauth2_token = Some(token),
            }
            Ok(())
        })
    }

    /// Records `user_id` as the account the `OAuth2` token stored under
    /// `username` belongs to, returning whether the store changed. An empty
    /// `username` is the unnamed slot.
    ///
    /// The id is a fact about the token beside it, so the caller passes one
    /// only when `/2/users/me` answered under that token.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn set_oauth2_user_id_for_app(
        &mut self,
        app_name: &str,
        username: &str,
        user_id: &str,
    ) -> Result<bool> {
        fn slot<'a>(
            store: &'a mut TokenStore,
            app_name: &str,
            username: &str,
        ) -> Option<&'a mut OAuth2Token> {
            let app = store.resolve_app_mut(app_name);
            let token = if username.is_empty() {
                app.unnamed_oauth2_token.as_mut()
            } else {
                app.oauth2_tokens.get_mut(username)
            };
            token.and_then(|token| token.oauth2.as_mut())
        }
        let current = if username.is_empty() {
            self.get_oauth2_token_unnamed_for_app(app_name)
        } else {
            self.get_oauth2_token_for_app(app_name, username)
        }
        .and_then(|token| token.oauth2.as_ref());
        match current {
            None => return Ok(false),
            Some(token) if token.user_id.as_deref() == Some(user_id) => return Ok(false),
            Some(_) => {}
        }
        if user_id.is_empty() {
            return Ok(false);
        }
        self.update(|store| match slot(store, app_name, username) {
            Some(token) => {
                token.user_id = Some(user_id.to_string());
                Ok(true)
            }
            None => Ok(false),
        })
    }

    /// Saves an `OAuth2` token into the named app's unnamed (`/me`-failed salvage) slot.
    ///
    /// Used by the refresh and exchange paths when post-token username discovery
    /// fails: the refreshed access token is still valid and is preserved here
    /// rather than discarded. Single-occupancy, last-write-wins.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn save_oauth2_token_unnamed_for_app(
        &mut self,
        app_name: &str,
        access_token: &str,
        refresh_token: &str,
        expiration_time: u64,
    ) -> Result<()> {
        let token = OAuth2Token {
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
            expiration_time,
            user_id: None,
        };
        self.put_oauth2_token(app_name, None, token, false)
    }

    /// Stores the named app's unnamed `OAuth2` token under `username` and
    /// empties the unnamed slot, returning whether a token moved.
    ///
    /// Nothing moves when the slot is empty, or when a token is already
    /// stored under `username`: that one is the newer, written by a refresh
    /// that learned the name, so both are left as they are.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn name_unnamed_oauth2_token_for_app(
        &mut self,
        app_name: &str,
        username: &str,
    ) -> Result<bool> {
        let movable = |store: &Self| {
            let app = store.resolve_app(app_name);
            app.unnamed_oauth2_token.is_some() && !app.oauth2_tokens.contains_key(username)
        };
        if username.is_empty() || !movable(self) {
            return Ok(false);
        }
        self.update(|store| {
            if !movable(store) {
                return Ok(false);
            }
            let app = store.resolve_app_mut(app_name);
            if let Some(token) = app.unnamed_oauth2_token.take() {
                app.oauth2_tokens.insert(username.to_string(), token);
            }
            Ok(true)
        })
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
        self.update(|store| {
            let app = store.resolve_app_mut(app_name);
            app.oauth1_token = Some(Token {
                token_type: TokenType::Oauth1,
                bearer: None,
                oauth2: None,
                oauth1: Some(OAuth1Token {
                    access_token: access_token.to_string(),
                    token_secret: token_secret.to_string(),
                    consumer_key: consumer_key.to_string(),
                    consumer_secret: consumer_secret.to_string(),
                    user_id: None,
                }),
            });
            Ok(())
        })
    }

    /// Records `user_id` as the account the named app's `OAuth1` access pair
    /// belongs to, returning whether the store changed. As with
    /// [`Self::set_oauth2_user_id_for_app`], the caller passes an id only
    /// when `/2/users/me` answered under that pair.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn set_oauth1_user_id_for_app(&mut self, app_name: &str, user_id: &str) -> Result<bool> {
        let current = self
            .get_oauth1_tokens_for_app(app_name)
            .and_then(|token| token.oauth1.as_ref());
        match current {
            None => return Ok(false),
            Some(token) if token.user_id.as_deref() == Some(user_id) => return Ok(false),
            Some(_) => {}
        }
        if user_id.is_empty() {
            return Ok(false);
        }
        self.update(|store| {
            let token = store
                .resolve_app_mut(app_name)
                .oauth1_token
                .as_mut()
                .and_then(|token| token.oauth1.as_mut());
            match token {
                Some(token) => {
                    token.user_id = Some(user_id.to_string());
                    Ok(true)
                }
                None => Ok(false),
            }
        })
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
        self.first_oauth2_user_for_app(app_name)
            .map(|(_, token)| token)
    }

    /// The user a request with no username is sent as, and that user's
    /// token: the default user when one is set and still holds a token,
    /// else the first `OAuth2` user of the named app.
    #[must_use]
    pub fn first_oauth2_user_for_app(&self, app_name: &str) -> Option<(&str, &Token)> {
        let app = self.resolve_app(app_name);
        if !app.default_user.is_empty()
            && let Some((username, token)) = app.oauth2_tokens.get_key_value(&app.default_user)
        {
            return Some((username.as_str(), token));
        }
        app.oauth2_tokens
            .iter()
            .next()
            .map(|(username, token)| (username.as_str(), token))
    }

    /// Gets the unnamed (`/me`-failed salvage) `OAuth2` token from the named app.
    ///
    /// Returns `None` when the slot is empty.
    #[must_use]
    pub fn get_oauth2_token_unnamed_for_app(&self, app_name: &str) -> Option<&Token> {
        let app = self.resolve_app(app_name);
        app.unnamed_oauth2_token.as_ref()
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

    /// Returns the names of every app in the store that holds at least one
    /// stored credential (OAuth2 token, OAuth1 tokens, or bearer token).
    ///
    /// Iterates in `BTreeMap` key order so the result is deterministic.
    /// Used by the `get_auth_header` resolver to surface a "wrong-app"
    /// envelope when the active app is empty but the user has credentials
    /// stored under a different app.
    #[must_use]
    pub fn apps_with_credentials(&self) -> Vec<String> {
        self.apps
            .iter()
            .filter(|(_, app)| {
                !app.oauth2_tokens.is_empty()
                    || app.oauth1_token.is_some()
                    || app.bearer_token.is_some()
                    || app.unnamed_oauth2_token.is_some()
            })
            .map(|(name, _)| name.clone())
            .collect()
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

    /// Name of the app a clear should act on, or `None` when it does not
    /// exist.
    ///
    /// Clearing must not be the operation that materializes an app: a clear
    /// against a store with nothing in it has nothing to do, and creating a
    /// placeholder to empty it would put back a phantom app an empty store
    /// does not carry.
    fn existing_app_name(&self, app_name: &str) -> Option<String> {
        let name = self.get_active_app_name(app_name).to_string();
        self.apps.contains_key(&name).then_some(name)
    }

    /// Clears an `OAuth2` token for a username from the named app.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn clear_oauth2_token_for_app(&mut self, app_name: &str, username: &str) -> Result<()> {
        self.clear_in_app(app_name, |app| {
            app.oauth2_tokens.remove(username);
        })
    }

    /// Applies `clear` to the app a clear should act on, under the lock, and
    /// does nothing when that app does not exist.
    ///
    /// A store that could not be loaded holds no apps in memory whatever its
    /// file holds, so it is refused before the lookup can report nothing to
    /// clear.
    fn clear_in_app(&mut self, app_name: &str, clear: impl FnOnce(&mut super::App)) -> Result<()> {
        self.refuse_if_load_failed()?;
        if self.existing_app_name(app_name).is_none() {
            return Ok(());
        }
        self.update(|store| {
            if let Some(name) = store.existing_app_name(app_name)
                && let Some(app) = store.apps.get_mut(&name)
            {
                clear(app);
            }
            Ok(())
        })
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
        self.clear_in_app(app_name, |app| app.oauth1_token = None)
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
        self.clear_in_app(app_name, |app| app.bearer_token = None)
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
        self.clear_in_app(app_name, |app| {
            app.oauth2_tokens.clear();
            app.oauth1_token = None;
            app.bearer_token = None;
            app.unnamed_oauth2_token = None;
        })
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
