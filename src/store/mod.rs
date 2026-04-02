/// Token persistence layer — multi-app YAML store at `~/.xurl`.
///
/// Supports:
/// - Multi-app credential and token management
/// - `OAuth2`, `OAuth1`, and Bearer token types
/// - Legacy JSON migration (auto-converts old format)
/// - `.twurlrc` import (legacy Twitter CLI compatibility)
/// - Credential backfill from environment variables
mod migration;
mod tokens;
pub mod types;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[allow(unused_imports)] // Re-exported for library consumers and integration tests
pub use types::{App, OAuth1Token, OAuth2Token, Token, TokenType};

use crate::error::{Result, XurlError};

// ── TokenStore ───────────────────────────────────────────────────────

/// Manages authentication tokens across multiple apps.
pub struct TokenStore {
    pub apps: BTreeMap<String, App>,
    pub default_app: String,
    pub file_path: PathBuf,
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)] // Public library API — used by consumers and integration tests
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

        // Ensure a default app exists (matches Go: NewTokenStore always returns a usable store)
        if store.apps.is_empty() {
            store.default_app = "default".to_string();
            store.apps.insert("default".to_string(), App::new());
        }

        // Import from .twurlrc if we have no apps or the default app is missing OAuth1/Bearer
        let needs_import = match store.active_app() {
            None => true,
            Some(app) => app.oauth1_token.is_none() || app.bearer_token.is_none(),
        };
        if needs_import {
            let twurlrc_path = home_dir.join(".twurlrc");
            if twurlrc_path.exists()
                && let Err(e) = store.import_from_twurlrc(&twurlrc_path)
            {
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
    pub fn new_with_credentials_and_path(client_id: &str, client_secret: &str, path: &str) -> Self {
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

    // ── App management ───────────────────────────────────────────────

    /// Registers a new application. If it's the only app it becomes default.
    ///
    /// # Errors
    ///
    /// Returns an error if the app name already exists or the store cannot be saved.
    pub fn add_app(&mut self, name: &str, client_id: &str, client_secret: &str) -> Result<()> {
        if self.apps.contains_key(name) {
            return Err(XurlError::token_store(format!(
                "app {name:?} already exists"
            )));
        }
        self.apps.insert(
            name.to_string(),
            App::with_credentials(client_id, client_secret),
        );
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
    pub(crate) fn active_app(&self) -> Option<&App> {
        self.apps.get(&self.default_app)
    }

    /// Returns the active app; creates "default" if none exist.
    pub(crate) fn active_app_or_create(&mut self) -> &mut App {
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
            && let Some(app) = self.apps.get(name)
        {
            return app;
        }
        // Fall back to default app, or a static empty app
        self.apps.get(&self.default_app).unwrap_or_else(|| {
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

    // ── Persistence ──────────────────────────────────────────────────

    /// Saves the token store to `~/.xurl` in YAML format.
    pub(crate) fn save_to_file(&self) -> Result<()> {
        let sf = types::StoreFile {
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
