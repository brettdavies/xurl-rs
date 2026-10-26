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
            account_username: None,
            display_name: None,
        };
        self.put_oauth2_token(app_name, Some(username), token, false)
    }

    /// Stores `token` in the named app: under `name`, or in the unnamed slot
    /// when there is none. `drop_unnamed` empties the unnamed slot in the
    /// same write, for a token that just left it for a name.
    ///
    /// The token's names were not asked of X for this write: a refresh
    /// carries them from the token it replaces, read before its grant went
    /// out. Where the slot holds the same account, the token takes the names
    /// the store holds now, so a newer answer recorded meanwhile stands.
    pub(crate) fn put_oauth2_token(
        &mut self,
        app_name: &str,
        name: Option<&str>,
        token: OAuth2Token,
        drop_unnamed: bool,
    ) -> Result<()> {
        self.write_oauth2_token(app_name, name, token, drop_unnamed, false)
    }

    /// [`Self::put_oauth2_token`] for a token whose names `/2/users/me` just
    /// gave, under that token: they are written beside it, and onto every
    /// credential the store holds for the same account.
    pub(crate) fn put_answered_oauth2_token(
        &mut self,
        app_name: &str,
        name: Option<&str>,
        token: OAuth2Token,
        drop_unnamed: bool,
    ) -> Result<()> {
        self.write_oauth2_token(app_name, name, token, drop_unnamed, true)
    }

    fn write_oauth2_token(
        &mut self,
        app_name: &str,
        name: Option<&str>,
        mut token: OAuth2Token,
        drop_unnamed: bool,
        answered: bool,
    ) -> Result<()> {
        self.update(|store| {
            if answered && let Some(user_id) = &token.user_id {
                store.share_account_names(
                    user_id,
                    token.account_username.as_deref(),
                    token.display_name.as_deref(),
                );
            }
            let app = store.resolve_app_mut(app_name);
            let held = match name {
                Some(name) => app.oauth2_tokens.get(name),
                None => app.unnamed_oauth2_token.as_ref(),
            }
            .and_then(|held| held.oauth2.as_ref());
            if !answered
                && let Some(held) = held
                && token.user_id.is_some()
                && held.user_id == token.user_id
            {
                token.account_username.clone_from(&held.account_username);
                token.display_name.clone_from(&held.display_name);
            }
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
    /// `username` is the unnamed slot. A new id drops the account's stored
    /// username and display name; the same id keeps them.
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
        self.set_oauth2_account_for_app(app_name, username, user_id, None, None)
    }

    /// Records the account the `OAuth2` token stored under `username`
    /// belongs to, as one `/2/users/me` answer under that token named it:
    /// its id, and its username and display name when the answer carried
    /// them. Returns whether the store changed. An empty `username` is the
    /// unnamed slot.
    ///
    /// A name passed replaces the stored one. A name left `None` keeps the
    /// stored one while the id is unchanged, and a new id drops it, so a
    /// stored name never belongs to another account. A name passed is also
    /// written onto every other credential the store holds for `user_id`, in
    /// every app; the id itself is never copied.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn set_oauth2_account_for_app(
        &mut self,
        app_name: &str,
        username: &str,
        user_id: &str,
        account_username: Option<&str>,
        display_name: Option<&str>,
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
        if user_id.is_empty() {
            return Ok(false);
        }
        let recorded = |token: &OAuth2Token| {
            let stored = StoredAccount::of_oauth2(token);
            let next = stored.recorded(user_id, account_username, display_name);
            (next != stored).then_some(next)
        };
        let current = if username.is_empty() {
            self.get_oauth2_token_unnamed_for_app(app_name)
        } else {
            self.get_oauth2_token_for_app(app_name, username)
        }
        .and_then(|token| token.oauth2.as_ref());
        if current.and_then(recorded).is_none()
            && !self.has_account_names_to_share(user_id, account_username, display_name)
        {
            return Ok(false);
        }
        self.update(|store| {
            let mut changed = false;
            if let Some(token) = slot(store, app_name, username)
                && let Some(next) = recorded(token)
            {
                next.write_oauth2(token);
                changed = true;
            }
            let shared = store.share_account_names(user_id, account_username, display_name);
            Ok(changed || shared)
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
            account_username: None,
            display_name: None,
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
                    account_username: None,
                    display_name: None,
                }),
            });
            Ok(())
        })
    }

    /// Records `user_id` as the account the named app's `OAuth1` access pair
    /// belongs to, returning whether the store changed. As with
    /// [`Self::set_oauth2_user_id_for_app`], the caller passes an id only
    /// when `/2/users/me` answered under that pair, and a new id drops the
    /// stored username and display name.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn set_oauth1_user_id_for_app(&mut self, app_name: &str, user_id: &str) -> Result<bool> {
        self.set_oauth1_account_for_app(app_name, user_id, None, None)
    }

    /// Records the account the named app's `OAuth1` access pair belongs to,
    /// as [`Self::set_oauth2_account_for_app`] does for an `OAuth2` login,
    /// names written onto every credential held for the same account
    /// included.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be saved to disk.
    pub fn set_oauth1_account_for_app(
        &mut self,
        app_name: &str,
        user_id: &str,
        account_username: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<bool> {
        if user_id.is_empty() {
            return Ok(false);
        }
        let recorded = |token: &OAuth1Token| {
            let stored = StoredAccount::of_oauth1(token);
            let next = stored.recorded(user_id, account_username, display_name);
            (next != stored).then_some(next)
        };
        let current = self
            .get_oauth1_tokens_for_app(app_name)
            .and_then(|token| token.oauth1.as_ref());
        if current.and_then(recorded).is_none()
            && !self.has_account_names_to_share(user_id, account_username, display_name)
        {
            return Ok(false);
        }
        self.update(|store| {
            let mut changed = false;
            if let Some(token) = store
                .resolve_app_mut(app_name)
                .oauth1_token
                .as_mut()
                .and_then(|token| token.oauth1.as_mut())
                && let Some(next) = recorded(token)
            {
                next.write_oauth1(token);
                changed = true;
            }
            let shared = store.share_account_names(user_id, account_username, display_name);
            Ok(changed || shared)
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

/// The account fields a stored user credential carries.
#[derive(PartialEq)]
struct StoredAccount {
    user_id: Option<String>,
    account_username: Option<String>,
    display_name: Option<String>,
}

impl StoredAccount {
    fn of_oauth2(token: &OAuth2Token) -> Self {
        Self {
            user_id: token.user_id.clone(),
            account_username: token.account_username.clone(),
            display_name: token.display_name.clone(),
        }
    }

    fn of_oauth1(token: &OAuth1Token) -> Self {
        Self {
            user_id: token.user_id.clone(),
            account_username: token.account_username.clone(),
            display_name: token.display_name.clone(),
        }
    }

    /// The fields after recording an answer: a name given replaces the
    /// stored one, a name left out stays while the id is the same, and a
    /// new id drops both.
    fn recorded(
        &self,
        user_id: &str,
        account_username: Option<&str>,
        display_name: Option<&str>,
    ) -> Self {
        let same_account = self.user_id.as_deref() == Some(user_id);
        let name = |given: Option<&str>, stored: &Option<String>| match given {
            Some(given) => Some(given.to_string()),
            None if same_account => stored.clone(),
            None => None,
        };
        Self {
            user_id: Some(user_id.to_string()),
            account_username: name(account_username, &self.account_username),
            display_name: name(display_name, &self.display_name),
        }
    }

    fn write_oauth2(self, token: &mut OAuth2Token) {
        token.user_id = self.user_id;
        token.account_username = self.account_username;
        token.display_name = self.display_name;
    }

    fn write_oauth1(self, token: &mut OAuth1Token) {
        token.user_id = self.user_id;
        token.account_username = self.account_username;
        token.display_name = self.display_name;
    }
}
