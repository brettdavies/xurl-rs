/// Token persistence layer — multi-app YAML store at `~/.xurl`.
///
/// Supports:
/// - Multi-app credential and token management
/// - `OAuth2`, `OAuth1`, and Bearer token types
/// - Legacy JSON migration (auto-converts old format)
/// - `.twurlrc` import (legacy Twitter CLI compatibility)
/// - Credential backfill from environment variables
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Result, XurlError};

// ── Token types ──────────────────────────────────────────────────────

/// `OAuth1` authentication tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth1Token {
    pub access_token: String,
    pub token_secret: String,
    pub consumer_key: String,
    pub consumer_secret: String,
}

/// `OAuth2` authentication tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expiration_time: u64,
}

/// Token type discriminator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Bearer,
    Oauth2,
    Oauth1,
}

/// A token with its type and payload.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    #[serde(rename = "type")]
    pub token_type: TokenType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2: Option<OAuth2Token>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth1: Option<OAuth1Token>,
}

// ── App ──────────────────────────────────────────────────────────────

/// Holds credentials and tokens for a single registered X API application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub client_id: String,
    pub client_secret: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub default_user: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub oauth2_tokens: BTreeMap<String, Token>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth1_token: Option<Token>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<Token>,
}

impl App {
    fn new() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            default_user: String::new(),
            oauth2_tokens: BTreeMap::new(),
            oauth1_token: None,
            bearer_token: None,
        }
    }

    fn with_credentials(client_id: &str, client_secret: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            ..Self::new()
        }
    }

    fn has_tokens(&self) -> bool {
        !self.oauth2_tokens.is_empty()
            || self.oauth1_token.is_some()
            || self.bearer_token.is_some()
    }
}

// ── On-disk YAML structure ───────────────────────────────────────────

/// Serialised YAML layout of `~/.xurl`.
#[derive(Debug, Serialize, Deserialize)]
struct StoreFile {
    apps: BTreeMap<String, App>,
    default_app: String,
}

// ── Legacy JSON structure (for migration) ────────────────────────────

#[derive(Debug, Deserialize)]
struct LegacyStore {
    oauth2_tokens: Option<BTreeMap<String, Token>>,
    #[serde(rename = "oauth1_tokens")]
    oauth1_token: Option<Token>,
    bearer_token: Option<Token>,
}

// ── Twurlrc structures ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TwurlrcProfile {
    username: Option<String>,
    consumer_key: Option<String>,
    consumer_secret: Option<String>,
    token: Option<String>,
    secret: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TwurlrcConfig {
    profiles: Option<BTreeMap<String, BTreeMap<String, TwurlrcProfile>>>,
    bearer_tokens: Option<BTreeMap<String, String>>,
}

// ── TokenStore ───────────────────────────────────────────────────────

/// Manages authentication tokens across multiple apps.
pub struct TokenStore {
    pub apps: BTreeMap<String, App>,
    pub default_app: String,
    pub file_path: PathBuf,
}

#[allow(dead_code)]
impl Default for TokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl TokenStore {
    /// Creates a new `TokenStore`, loading from `~/.xurl` (auto-migrating legacy JSON).
    #[must_use] 
    pub fn new() -> Self {
        Self::with_credentials("", "")
    }

    /// Creates a `TokenStore` and backfills the given client credentials into any
    /// app that was migrated without them.
    #[must_use] 
    pub fn with_credentials(client_id: &str, client_secret: &str) -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let file_path = home_dir.join(".xurl");

        let mut store = TokenStore {
            apps: BTreeMap::new(),
            default_app: String::new(),
            file_path,
        };

        if let Ok(data) = fs::read(&store.file_path) {
            store.load_from_data(&data);
        }

        // Backfill credentials into any app that has tokens but no client ID/secret
        if !client_id.is_empty() || !client_secret.is_empty() {
            let mut dirty = false;
            for app in store.apps.values_mut() {
                if app.has_tokens() {
                    if app.client_id.is_empty() && !client_id.is_empty() {
                        app.client_id = client_id.to_string();
                        dirty = true;
                    }
                    if app.client_secret.is_empty() && !client_secret.is_empty() {
                        app.client_secret = client_secret.to_string();
                        dirty = true;
                    }
                }
            }
            if dirty {
                let _ = store.save_to_file();
            }
        }

        // Import from .twurlrc if we have no apps or the default app is missing OAuth1/Bearer
        let needs_import = match store.active_app() {
            None => true,
            Some(app) => app.oauth1_token.is_none() || app.bearer_token.is_none(),
        };
        if needs_import {
            let twurlrc_path = home_dir.join(".twurlrc");
            if twurlrc_path.exists()
                && let Err(e) = store.import_from_twurlrc(&twurlrc_path) {
                    eprintln!("Error importing from .twurlrc: {e}");
                }
        }

        store
    }

    /// Creates a `TokenStore` from a specific file path (no auto-import).
    #[must_use] 
    pub fn new_with_path(path: &str) -> Self {
        let file_path = PathBuf::from(path);
        let mut store = TokenStore {
            apps: BTreeMap::new(),
            default_app: String::new(),
            file_path,
        };
        if let Ok(data) = fs::read(&store.file_path) {
            store.load_from_data(&data);
        }
        if store.apps.is_empty() {
            store.apps.insert("default".to_string(), App::new());
            store.default_app = "default".to_string();
        }
        store
    }

    /// Creates a `TokenStore` from a specific file path with credential backfill.
    #[must_use] 
    pub fn new_with_credentials_and_path(
        client_id: &str,
        client_secret: &str,
        path: &str,
    ) -> Self {
        let mut store = Self::new_with_path(path);
        if !client_id.is_empty() || !client_secret.is_empty() {
            for app in store.apps.values_mut() {
                if app.has_tokens() || app.client_id.is_empty() {
                    if app.client_id.is_empty() && !client_id.is_empty() {
                        app.client_id = client_id.to_string();
                    }
                    if app.client_secret.is_empty() && !client_secret.is_empty() {
                        app.client_secret = client_secret.to_string();
                    }
                }
            }
            let _ = store.save_to_file();
        }
        store
    }

    /// Creates a `TokenStore` using a custom home directory (for testing).
    #[must_use] 
    pub fn new_with_home(home: &str) -> Self {
        let home_path = PathBuf::from(home);
        let file_path = home_path.join(".xurl");
        let mut store = TokenStore {
            apps: BTreeMap::new(),
            default_app: String::new(),
            file_path,
        };
        if let Ok(data) = fs::read(&store.file_path) {
            store.load_from_data(&data);
        }
        if store.apps.is_empty() {
            store.apps.insert("default".to_string(), App::new());
            store.default_app = "default".to_string();
        }
        // Auto-import from .twurlrc if needed
        let needs_import = match store.active_app() {
            None => true,
            Some(app) => app.oauth1_token.is_none(),
        };
        if needs_import {
            let twurlrc_path = home_path.join(".twurlrc");
            if twurlrc_path.exists() {
                let _ = store.import_from_twurlrc(&twurlrc_path);
            }
        }
        store
    }

    /// Loads a `TokenStore` from a specific file path (alias for `new_with_path`).
    #[must_use] 
    pub fn load_from_path(path: &str) -> Self {
        Self::new_with_path(path)
    }

    /// Tries YAML first, then falls back to legacy JSON migration.
    fn load_from_data(&mut self, data: &[u8]) {
        // Try new YAML format first
        if let Ok(sf) = serde_yaml::from_slice::<StoreFile>(data)
            && !sf.apps.is_empty() {
                self.apps = sf.apps;
                self.default_app = sf.default_app;
                return;
            }

        // Fall back to legacy JSON
        if let Ok(legacy) = serde_json::from_slice::<LegacyStore>(data) {
            let app = App {
                client_id: String::new(),
                client_secret: String::new(),
                default_user: String::new(),
                oauth2_tokens: legacy.oauth2_tokens.unwrap_or_default(),
                oauth1_token: legacy.oauth1_token,
                bearer_token: legacy.bearer_token,
            };
            self.apps.insert("default".to_string(), app);
            self.default_app = "default".to_string();
            // Persist in new YAML format immediately
            let _ = self.save_to_file();
        }
    }

    // ── App management ───────────────────────────────────────────────

    /// Registers a new application. If it's the only app it becomes default.
    ///
    /// # Errors
    ///
    /// Returns an error if the app name already exists or the store cannot be saved.
    pub fn add_app(&mut self, name: &str, client_id: &str, client_secret: &str) -> Result<()> {
        if self.apps.contains_key(name) {
            return Err(XurlError::token_store(format!("app {name:?} already exists")));
        }
        self.apps
            .insert(name.to_string(), App::with_credentials(client_id, client_secret));
        if self.apps.len() == 1 {
            self.default_app = name.to_string();
        }
        self.save_to_file()
    }

    /// Updates the credentials of an existing application.
    ///
    /// # Errors
    ///
    /// Returns an error if the app is not found or the store cannot be saved.
    pub fn update_app(&mut self, name: &str, client_id: &str, client_secret: &str) -> Result<()> {
        let app = self
            .apps
            .get_mut(name)
            .ok_or_else(|| XurlError::token_store(format!("app {name:?} not found")))?;
        if !client_id.is_empty() {
            app.client_id = client_id.to_string();
        }
        if !client_secret.is_empty() {
            app.client_secret = client_secret.to_string();
        }
        self.save_to_file()
    }

    /// Removes a registered application and its tokens.
    ///
    /// # Errors
    ///
    /// Returns an error if the app is not found or the store cannot be saved.
    pub fn remove_app(&mut self, name: &str) -> Result<()> {
        if !self.apps.contains_key(name) {
            return Err(XurlError::token_store(format!("app {name:?} not found")));
        }
        self.apps.remove(name);
        if self.default_app == name {
            self.default_app = self.apps.keys().next().cloned().unwrap_or_default();
        }
        self.save_to_file()
    }

    /// Sets the default application by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the app is not found or the store cannot be saved.
    pub fn set_default_app(&mut self, name: &str) -> Result<()> {
        if !self.apps.contains_key(name) {
            return Err(XurlError::token_store(format!("app {name:?} not found")));
        }
        self.default_app = name.to_string();
        self.save_to_file()
    }

    /// Returns sorted app names.
    #[must_use] 
    pub fn list_apps(&self) -> Vec<String> {
        self.apps.keys().cloned().collect()
    }

    /// Returns an app by name.
    #[must_use] 
    pub fn get_app(&self, name: &str) -> Option<&App> {
        self.apps.get(name)
    }

    /// Sets the default `OAuth2` user for the named (or default) app.
    ///
    /// # Errors
    ///
    /// Returns an error if the username is not found in the app or the store cannot be saved.
    pub fn set_default_user(&mut self, app_name: &str, username: &str) -> Result<()> {
        let app = self.resolve_app_mut(app_name);
        if !app.oauth2_tokens.contains_key(username) {
            return Err(XurlError::token_store(format!(
                "user {username:?} not found in app"
            )));
        }
        app.default_user = username.to_string();
        self.save_to_file()
    }

    /// Returns the default `OAuth2` user for the named (or default) app.
    #[must_use] 
    pub fn get_default_user(&self, app_name: &str) -> &str {
        let app = self.resolve_app(app_name);
        &app.default_user
    }

    /// Returns the default app name.
    #[must_use] 
    pub fn get_default_app(&self) -> &str {
        &self.default_app
    }

    /// Returns the name of the active app (explicit or default).
    #[must_use] 
    pub fn get_active_app_name<'a>(&'a self, explicit: &'a str) -> &'a str {
        if explicit.is_empty() {
            &self.default_app
        } else {
            explicit
        }
    }

    /// Returns the current default App, or None.
    fn active_app(&self) -> Option<&App> {
        self.apps.get(&self.default_app)
    }

    /// Returns the active app; creates "default" if none exist.
    fn active_app_or_create(&mut self) -> &mut App {
        if !self.apps.contains_key(&self.default_app) {
            self.apps.insert("default".to_string(), App::new());
            if self.default_app.is_empty() {
                self.default_app = "default".to_string();
            }
        }
        let key = if self.apps.contains_key(&self.default_app) {
            self.default_app.clone()
        } else {
            "default".to_string()
        };
        self.apps.get_mut(&key).expect("just inserted")
    }

    /// Returns the app for the given name, or the default app.
    #[must_use] 
    pub fn resolve_app(&self, name: &str) -> &App {
        if !name.is_empty()
            && let Some(app) = self.apps.get(name) {
                return app;
            }
        // Fall back to default app, or a static empty app
        self.apps
            .get(&self.default_app)
            .unwrap_or_else(|| {
                // This is a fallback — should rarely happen
                static EMPTY: std::sync::LazyLock<App> = std::sync::LazyLock::new(App::new);
                &EMPTY
            })
    }

    /// Returns the app for the given name (mutable), or the default app.
    ///
    /// # Panics
    ///
    /// Panics if the internal app map is in an inconsistent state (should never
    /// happen as `active_app_or_create` always inserts a default).
    pub fn resolve_app_mut(&mut self, name: &str) -> &mut App {
        if !name.is_empty() && self.apps.contains_key(name) {
            return self.apps.get_mut(name).expect("just checked");
        }
        self.active_app_or_create()
    }

    // ── Twurlrc import ───────────────────────────────────────────────

    /// Imports tokens from a `.twurlrc` file into the active app.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, parsed, or the store cannot be saved.
    pub fn import_from_twurlrc(&mut self, file_path: &std::path::Path) -> Result<()> {
        let data = fs::read(file_path)?;
        let twurlrc: TwurlrcConfig = serde_yaml::from_slice(&data)?;

        let app = self.active_app_or_create();

        // Import the first OAuth1 tokens from twurlrc
        if let Some(profiles) = &twurlrc.profiles {
            'outer: for consumer_keys in profiles.values() {
                if let Some((consumer_key, profile)) = consumer_keys.iter().next() {
                    if app.oauth1_token.is_none() {
                        app.oauth1_token = Some(Token {
                            token_type: TokenType::Oauth1,
                            bearer: None,
                            oauth2: None,
                            oauth1: Some(OAuth1Token {
                                access_token: profile.token.clone().unwrap_or_default(),
                                token_secret: profile.secret.clone().unwrap_or_default(),
                                consumer_key: consumer_key.clone(),
                                consumer_secret: profile
                                    .consumer_secret
                                    .clone()
                                    .unwrap_or_default(),
                            }),
                        });
                    }
                    break 'outer;
                }
            }
        }

        // Import the first bearer token from twurlrc
        if let Some(bearer_tokens) = &twurlrc.bearer_tokens
            && let Some(bearer_token) = bearer_tokens.values().next()
        {
            app.bearer_token = Some(Token {
                token_type: TokenType::Bearer,
                bearer: Some(bearer_token.clone()),
                oauth2: None,
                oauth1: None,
            });
        }

        self.save_to_file()
    }

    // ── Token operations ─────────────────────────────────────────────

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
        self.save_oauth1_tokens_for_app("", access_token, token_secret, consumer_key, consumer_secret)
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
            && let Some(token) = app.oauth2_tokens.get(&app.default_user) {
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

    // ── Persistence ──────────────────────────────────────────────────

    /// Saves the token store to `~/.xurl` in YAML format.
    fn save_to_file(&self) -> Result<()> {
        let sf = StoreFile {
            apps: self.apps.clone(),
            default_app: self.default_app.clone(),
        };
        let data = serde_yaml::to_string(&sf).map_err(|e| XurlError::Json(e.to_string()))?;
        fs::write(&self.file_path, data)?;

        // Match Go's 0600 permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.file_path, perms)?;
        }

        Ok(())
    }
}
