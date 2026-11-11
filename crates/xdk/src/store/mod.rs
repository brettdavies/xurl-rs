//! Token persistence layer — multi-app YAML store at `~/.xurl/auth.yml`.
//!
//! Supports:
//! - Multi-app credential and token management
//! - `OAuth2`, `OAuth1`, and Bearer token types
//! - Legacy JSON migration (auto-converts old format)
//! - `.twurlrc` import (legacy Twitter CLI compatibility)
//! - Credential backfill from environment variables

mod account_names;
pub(crate) mod atomic;
mod hook;
mod layout;
mod lock;
mod migration;
// Facts the binary renders in `xr auth status` and its hints; not a store API.
#[doc(hidden)]
pub mod snapshot;
mod tokens;
pub mod types;

pub(crate) use atomic::write_atomically;
pub(crate) use layout::ensure_store_dir;
use layout::store_path_under;
pub use layout::{
    AUTH_FILE_NAME, STORE_TARGET, adopt_directory_layout, default_store_path, locate_store,
};
pub(crate) use lock::RefreshLock;
pub(crate) use tokens::MintedFrom;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(unused_imports)] // Re-exported for library consumers and integration tests
pub use types::{App, LoadState, OAuth1Token, OAuth2Token, Token, TokenType};

use crate::error::{Error, Result};

// ── TokenStore ───────────────────────────────────────────────────────

/// Manages authentication tokens across multiple apps.
///
/// The in-memory shape mirrors the on-disk YAML at `~/.xurl/auth.yml`. Library
/// consumers construct via [`TokenStore::new`] (the default path),
/// [`TokenStore::new_with_path`] (explicit path, no auto-import), or
/// [`TokenStore::with_credentials`] (auto-backfill).
///
/// # Example
///
/// ```rust,no_run
/// use xdk::store::TokenStore;
///
/// let store = TokenStore::new_with_path("/tmp/my-xurl-store.yaml");
/// let active = store.get_default_app();
/// if let Some(app) = store.get_app(active) {
///     println!("active app {active} has {} oauth2 users", app.oauth2_tokens.len());
/// }
/// ```
pub struct TokenStore {
    /// All registered apps, keyed by name.
    pub apps: BTreeMap<String, App>,
    /// Name of the default app selected when `--app` is not supplied.
    pub default_app: String,
    /// Path to the YAML file backing this store.
    pub file_path: PathBuf,
    /// How the backing file resolved when this store was constructed.
    ///
    /// [`LoadState::Unreadable`] and [`LoadState::Unparseable`] make every
    /// save refuse, so a file the loader could not understand is reported
    /// rather than overwritten.
    pub load_state: LoadState,
}

impl std::fmt::Debug for TokenStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenStore")
            .field("file_path", &self.file_path)
            .field("default_app", &self.default_app)
            .field("apps", &self.apps.keys().collect::<Vec<_>>())
            .field("load_state", &self.load_state)
            .finish()
    }
}

impl Default for TokenStore {
    /// Constructs a `TokenStore` from the default location, identical to
    /// calling [`TokenStore::new`].
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)] // Public library API — used by consumers and integration tests
impl TokenStore {
    /// Creates a new `TokenStore`, loading from `~/.xurl/auth.yml` (auto-migrating
    /// legacy JSON).
    ///
    /// A single-file store still at `~/.xurl` is opened where it is; nothing is
    /// moved. See [`locate_store`] and [`adopt_directory_layout`].
    #[must_use]
    pub fn new() -> Self {
        Self::with_credentials("", "")
    }

    /// Creates a `TokenStore` and backfills the given client credentials into any
    /// app that was migrated without them.
    ///
    /// Backfilled values stay in memory until the next explicit save, so loading
    /// a store in the current format never rewrites it and concurrent readers
    /// cannot clobber each other. Loading a legacy-format file rewrites it once
    /// as the current YAML.
    #[must_use]
    pub fn with_credentials(client_id: &str, client_secret: &str) -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::from_home(&home_dir, client_id, client_secret)
    }

    fn from_home(home_dir: &Path, client_id: &str, client_secret: &str) -> Self {
        let file_path = locate_store(&store_path_under(home_dir));

        let mut store = TokenStore {
            apps: BTreeMap::new(),
            default_app: String::new(),
            file_path,
            load_state: LoadState::Fresh,
        };

        store.load_backing_file();

        // Backfill credentials into any app that has tokens but no client ID/secret
        for app in store.apps.values_mut() {
            if app.has_tokens() {
                if app.client_id.is_empty() && !client_id.is_empty() {
                    app.client_id = client_id.to_string();
                }
                if app.client_secret.is_empty() && !client_secret.is_empty() {
                    app.client_secret = client_secret.to_string();
                }
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
                && let Err(e) = store.import_from_twurlrc(&twurlrc_path)
            {
                tracing::warn!(target: STORE_TARGET, "error importing from .twurlrc: {e}");
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
            load_state: LoadState::Fresh,
        };
        store.load_backing_file();
        store
    }

    /// Creates a `TokenStore` from a specific file path with credential backfill.
    ///
    /// Like [`TokenStore::with_credentials`], the backfill stays in memory until
    /// the next explicit save; loading writes the file only to upgrade a
    /// legacy-format one.
    #[must_use]
    pub fn new_with_credentials_and_path(client_id: &str, client_secret: &str, path: &str) -> Self {
        let mut store = Self::new_with_path(path);
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
        store
    }

    /// Reads the backing file and records how it resolved.
    ///
    /// The view's apps and default app become the file's, so a reload keeps
    /// no app another process removed, or a file another process deleted.
    /// A missing file is a fresh store, not a damaged one, and the next save
    /// is allowed. A file with nothing but whitespace in it leaves the view
    /// as it was: a writer that truncates the file before writing it, rather
    /// than replacing it by rename, leaves it empty for a moment, and a save
    /// from a view that read it then would write every app away. A view that
    /// held nothing yet loads such a file as a fresh store. A file that exists
    /// but cannot be read or parsed leaves `apps` empty and records the
    /// failure, which makes every save refuse.
    fn load_backing_file(&mut self) {
        let read = fs::read(&self.file_path);
        if read
            .as_ref()
            .is_ok_and(|data| data.iter().all(u8::is_ascii_whitespace))
        {
            return;
        }
        self.apps.clear();
        self.default_app.clear();
        self.load_state = match read {
            Ok(data) => {
                if self.load_from_data(&data) {
                    LoadState::Loaded
                } else {
                    LoadState::Unparseable
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => LoadState::Fresh,
            Err(_) => LoadState::Unreadable,
        };
    }

    /// Loads a `TokenStore` from a specific file path (alias for `new_with_path`).
    #[must_use]
    pub fn load_from_path(path: &str) -> Self {
        Self::new_with_path(path)
    }

    // ── App management ───────────────────────────────────────────────

    /// Registers a new application.
    ///
    /// The new app becomes the default when it is the only one, or when the
    /// current default has neither a client id nor any token. Registration
    /// operates on the loaded apps alone and never materializes `default`.
    ///
    /// # Errors
    ///
    /// Returns an error if the app name already exists, carries a character
    /// outside `[A-Za-z0-9_.-]`, or the store cannot be saved.
    pub fn add_app(&mut self, name: &str, client_id: &str, client_secret: &str) -> Result<()> {
        self.refuse_if_app_present(name)?;
        validate_app_name(name)?;
        self.update(|store| {
            store.refuse_if_app_present(name)?;
            let promote = store.apps.is_empty() || store.default_lacks_credentials();
            store.apps.insert(
                name.to_string(),
                App::with_credentials(client_id, client_secret),
            );
            if promote {
                store.default_app = name.to_string();
            }
            Ok(())
        })
    }

    /// The not-found error for `name`, checked on this view before the lock
    /// is taken and again on disk truth inside it, so a store whose
    /// directory cannot hold a lock still answers with the app's absence.
    fn require_app(&self, name: &str) -> Result<()> {
        if self.apps.contains_key(name) {
            Ok(())
        } else {
            Err(Error::token_store(format!("app {name:?} not found")))
        }
    }

    /// The not-found error for `username` in `app`, checked on the caller's
    /// view before the lock is taken and again on disk truth inside it, as
    /// [`Self::require_app`] is.
    fn require_user(app: &App, username: &str) -> Result<()> {
        if app.oauth2_tokens.contains_key(username) {
            Ok(())
        } else {
            Err(Error::token_store(format!(
                "user {username:?} not found in app"
            )))
        }
    }

    /// The already-exists error for `name`, the registration counterpart of
    /// [`Self::require_app`].
    fn refuse_if_app_present(&self, name: &str) -> Result<()> {
        if self.apps.contains_key(name) {
            Err(Error::token_store(format!("app {name:?} already exists")))
        } else {
            Ok(())
        }
    }

    /// Whether the current default is a name the new registration should
    /// displace: absent, or present with neither a client id nor any token.
    ///
    /// A default holding a bearer or `OAuth1` token keeps its place, so a
    /// working app-only setup survives a later registration.
    fn default_lacks_credentials(&self) -> bool {
        match self.apps.get(&self.default_app) {
            None => true,
            Some(app) => app.client_id.is_empty() && !app.has_tokens(),
        }
    }

    /// Updates the credentials of an existing application.
    ///
    /// # Errors
    ///
    /// Returns an error if the app is not found or the store cannot be saved.
    pub fn update_app(&mut self, name: &str, client_id: &str, client_secret: &str) -> Result<()> {
        self.require_app(name)?;
        self.update(|store| {
            let app = store
                .apps
                .get_mut(name)
                .ok_or_else(|| Error::token_store(format!("app {name:?} not found")))?;
            if !client_id.is_empty() {
                app.client_id = client_id.to_string();
            }
            if !client_secret.is_empty() {
                app.client_secret = client_secret.to_string();
            }
            Ok(())
        })
    }

    /// Removes a registered application and its tokens.
    ///
    /// # Errors
    ///
    /// Returns an error if the app is not found or the store cannot be saved.
    pub fn remove_app(&mut self, name: &str) -> Result<()> {
        self.require_app(name)?;
        self.update(|store| {
            store.require_app(name)?;
            store.apps.remove(name);
            if store.default_app == name {
                store.default_app = store.apps.keys().next().cloned().unwrap_or_default();
            }
            Ok(())
        })
    }

    /// Sets the default application by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the app is not found or the store cannot be saved.
    pub fn set_default_app(&mut self, name: &str) -> Result<()> {
        self.require_app(name)?;
        self.update(|store| {
            store.require_app(name)?;
            store.default_app = name.to_string();
            Ok(())
        })
    }

    /// Returns `true` when the currently-resolved default app holds no
    /// credentials of any kind: no `OAuth2` user tokens, no `OAuth1` token,
    /// no bearer token, and no unnamed `OAuth2` salvage token. The signal
    /// used by [`Self::promote_to_default_if_first_credentialed`] to detect
    /// the placeholder default that a fresh install starts with so the very
    /// first authenticated app can transparently take over.
    #[must_use]
    pub fn default_app_is_uninitialized(&self) -> bool {
        let app = self.resolve_app("");
        app.oauth2_tokens.is_empty()
            && app.oauth1_token.is_none()
            && app.bearer_token.is_none()
            && app.unnamed_oauth2_token.is_none()
    }

    /// Promotes `candidate_app` to the default app iff the current default
    /// is uninitialized (per [`Self::default_app_is_uninitialized`]) and
    /// `candidate_app` is registered and different from the current default.
    /// Returns the new default name when promotion happened, `None` otherwise.
    ///
    /// Idempotent: callers can invoke unconditionally after any
    /// authentication save. No-op on already-credentialed defaults, on
    /// unknown candidates, on empty candidate names, and when the candidate
    /// already IS the default.
    ///
    /// Drives the "first signed-in app becomes the default" UX so a fresh
    /// `xr auth oauth2 --app NAME` (or `oauth1`, or `auth app --bearer-token
    /// --app NAME`) does not leave the placeholder `default` ahead of NAME
    /// in the resolution chain. Users who want a different default later
    /// can still run `xr auth default <name>` explicitly; this helper only
    /// fires on the first authenticated save.
    ///
    /// # Errors
    ///
    /// Returns an error if [`Self::set_default_app`] fails to persist.
    pub fn promote_to_default_if_first_credentialed(
        &mut self,
        candidate_app: &str,
    ) -> Result<Option<String>> {
        if candidate_app.is_empty() || candidate_app == self.default_app {
            return Ok(None);
        }
        if !self.apps.contains_key(candidate_app) {
            return Ok(None);
        }
        if !self.default_app_is_uninitialized() {
            return Ok(None);
        }
        self.update(|store| {
            if !store.apps.contains_key(candidate_app) || !store.default_app_is_uninitialized() {
                return Ok(None);
            }
            store.default_app = candidate_app.to_string();
            Ok(Some(candidate_app.to_string()))
        })
    }

    /// Returns sorted app names.
    #[must_use]
    pub fn list_apps(&self) -> Vec<String> {
        self.apps.keys().cloned().collect()
    }

    /// Sets the default application and that app's default `OAuth2` user in
    /// one write.
    ///
    /// Both names are checked before anything is saved, so a user the app
    /// does not hold leaves the default app where it was, and an app that is
    /// not registered leaves every default user where it was.
    ///
    /// # Errors
    ///
    /// Returns an error if the app is not found, the username is not found
    /// in that app, or the store cannot be saved.
    pub fn set_default_app_and_user(&mut self, name: &str, username: &str) -> Result<()> {
        self.require_user_in_app(name, username)?;
        self.update(|store| {
            store.require_user_in_app(name, username)?;
            store.default_app = name.to_string();
            if let Some(app) = store.apps.get_mut(name) {
                app.default_user = username.to_string();
            }
            Ok(())
        })
    }

    /// The not-found error for `name`, or for `username` in the app of that
    /// name. Unlike [`Self::resolve_app`], an unregistered name is an error
    /// here and never the default app.
    fn require_user_in_app(&self, name: &str, username: &str) -> Result<()> {
        self.require_app(name)?;
        match self.apps.get(name) {
            Some(app) => Self::require_user(app, username),
            None => Ok(()),
        }
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
        Self::require_user(self.resolve_app(app_name), username)?;
        self.update(|store| {
            let app = store.resolve_app_mut(app_name);
            Self::require_user(app, username)?;
            app.default_user = username.to_string();
            Ok(())
        })
    }

    /// Returns the default `OAuth2` user for the named (or default) app.
    #[must_use]
    pub fn get_default_user(&self, app_name: &str) -> &str {
        let app = self.resolve_app(app_name);
        &app.default_user
    }

    /// Sets the stored `OAuth2` redirect URI for the named (or default) app.
    ///
    /// An empty `uri` clears the stored value; the next serialize omits the
    /// field thanks to `#[serde(skip_serializing_if = "String::is_empty")]`.
    /// A non-empty `uri` is validated via [`crate::config::Config::validate_redirect_uri`]
    /// before persisting; on validation failure the store is not modified.
    ///
    /// # Errors
    ///
    /// Returns an error if the URI fails validation or the store cannot be saved.
    pub fn set_app_redirect_uri(&mut self, name: &str, uri: &str) -> Result<()> {
        if !name.is_empty() {
            self.require_app(name)?;
        }
        if !uri.is_empty() {
            let _ = crate::config::Config::validate_redirect_uri(uri)?;
        }
        self.update(|store| {
            if !name.is_empty() {
                store.require_app(name)?;
            }
            let app = store.resolve_app_mut(name);
            app.redirect_uri = uri.to_string();
            Ok(())
        })
    }

    /// Returns the stored `OAuth2` redirect URI for the named (or default) app.
    ///
    /// Returns `None` when the app is absent or its stored URI is empty.
    #[must_use]
    pub fn get_app_redirect_uri(&self, name: &str) -> Option<&str> {
        let app = self.resolve_app(name);
        if app.redirect_uri.is_empty() {
            None
        } else {
            Some(app.redirect_uri.as_str())
        }
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

    /// Applies `f` to the on-disk state and persists the result, holding the
    /// sidecar lock across the whole read-modify-write.
    ///
    /// The file is re-read once the lock is held, so `f` sees every write
    /// another process or another loaded view of this store has made. A
    /// backfilled client id or secret is the one piece of in-memory state a
    /// reload keeps: an app whose on-disk credential is empty keeps the value
    /// this store holds for it, as [`Self::new_with_credentials_and_path`]
    /// promises. Mutators called from inside `f` run under the same lock.
    ///
    /// # Errors
    ///
    /// Returns an error when the lock cannot be taken, the file exists but
    /// cannot be loaded, `f` fails, or the save fails. When `f` fails, the
    /// file is left as it was.
    pub fn update<R>(&mut self, f: impl FnOnce(&mut Self) -> Result<R>) -> Result<R> {
        self.ensure_dir()?;
        let lock = lock::StoreLock::acquire(&self.file_path)?;
        if lock.is_reentrant() {
            return f(self);
        }
        self.reload_locked();
        self.refuse_if_load_failed()?;
        let out = f(self)?;
        self.save_to_file()?;
        Ok(out)
    }

    /// Runs [`Self::update`] for the store at `path` on tokio's blocking
    /// pool, so a lock another process holds never parks the runtime thread.
    ///
    /// # Errors
    ///
    /// Everything [`Self::update`] returns, plus an internal error when the
    /// blocking task cannot be joined.
    pub async fn update_at<R, F>(path: PathBuf, f: F) -> Result<R>
    where
        F: FnOnce(&mut Self) -> Result<R> + Send + 'static,
        R: Send + 'static,
    {
        // A blocking-pool thread has no thread-local dispatcher, so the
        // caller's is carried in; otherwise the store's warnings vanish.
        let dispatch = tracing::Dispatch::default();
        tokio::task::spawn_blocking(move || {
            tracing::dispatcher::with_default(&dispatch, || {
                let mut store = Self::new_with_path(&path.to_string_lossy());
                store.update(f)
            })
        })
        .await
        .map_err(|e| Error::Internal(format!("store update task failed: {e}")))?
    }

    /// Re-reads the backing file so this view carries what another process
    /// has written since it loaded. Saves replace the file by rename, so a
    /// read needs no lock to see a whole store.
    pub(crate) fn reload(&mut self) {
        self.reload_locked();
    }

    /// Re-reads the backing file, keeping a backfilled credential where the
    /// file carries none.
    fn reload_locked(&mut self) {
        let held: Vec<(String, String, String)> = self
            .apps
            .iter()
            .map(|(name, app)| {
                (
                    name.clone(),
                    app.client_id.clone(),
                    app.client_secret.clone(),
                )
            })
            .collect();
        self.load_backing_file();
        for (name, client_id, client_secret) in held {
            if let Some(app) = self.apps.get_mut(&name) {
                if app.client_id.is_empty() {
                    app.client_id = client_id;
                }
                if app.client_secret.is_empty() {
                    app.client_secret = client_secret;
                }
            }
        }
    }

    /// Writes the store under the sidecar lock without re-reading the file
    /// first: the in-memory state is the whole truth, as it is right after a
    /// legacy file has been migrated.
    ///
    /// # Errors
    ///
    /// Returns an error when the lock cannot be taken or the save fails.
    pub(crate) fn save_locked(&self) -> Result<()> {
        self.ensure_dir()?;
        let _lock = lock::StoreLock::acquire(&self.file_path)?;
        self.save_to_file()
    }

    /// Writes the store to its file as YAML, atomically and `0600`.
    pub(crate) fn save_to_file(&self) -> Result<()> {
        self.refuse_if_load_failed()?;
        let sf = types::StoreFile {
            apps: self.apps.clone(),
            default_app: self.default_app.clone(),
        };
        let data = crate::yaml::to_string(&sf)
            .map_err(|e| Error::token_store(e.to_string()).with_source(e))?;
        write_atomically(&self.file_path, data.as_bytes())?;
        Ok(())
    }
}

impl TokenStore {
    /// Whether the backing file existed but could not be read or parsed.
    #[must_use]
    pub fn load_failed(&self) -> bool {
        matches!(
            self.load_state,
            LoadState::Unreadable | LoadState::Unparseable
        )
    }

    /// Creates the store's own directory before a write needs it.
    fn ensure_dir(&self) -> Result<()> {
        ensure_store_dir(&self.file_path).map_err(|e| self.dir_refused(e))
    }

    fn dir_refused(&self, e: std::io::Error) -> Error {
        Error::token_store(format!(
            "cannot create the directory of {}: {e}",
            self.file_path.display()
        ))
        .with_source(e)
    }

    /// The refusal every write path shares.
    ///
    /// # Errors
    ///
    /// Returns a token-store error naming the path when the file exists but
    /// could not be loaded, so a store the loader did not understand is
    /// never overwritten.
    pub(crate) fn refuse_if_load_failed(&self) -> Result<()> {
        if self.load_failed() {
            return Err(Error::token_store(format!(
                "refusing to write {}: the file exists but could not be loaded; fix or move it, then retry",
                self.file_path.display()
            )));
        }
        Ok(())
    }

    /// The refusals a save meets that do not depend on what it writes, met
    /// ahead of the work that ends in the save: the file exists but could not
    /// be loaded, the store's directory cannot be created, or the save's temp
    /// file cannot be created beside the store. The probe for the last two
    /// creates an empty file beside the store, or beside the directory the
    /// save would create, and removes it at once, so the disk is left as it
    /// was found.
    ///
    /// # Errors
    ///
    /// The token-store error of a store that could not be loaded or whose
    /// directory cannot be created, as the save returns them, and an I/O error
    /// naming the store when the temp file cannot be created.
    // `xr auth oauth2` and its dry run meet this before the credential check
    // (`crates/xurl-cli/src/cli/commands/auth/signin.rs`): a client id the
    // unloadable file holds is not one the store can be said to lack.
    #[doc(hidden)]
    pub fn refuse_unless_writable(&self) -> Result<()> {
        self.refuse_if_load_failed()?;
        if let Some(dir) = layout::outermost_store_dir_to_create(&self.file_path) {
            return atomic::probe_temp_file(dir).map_err(|e| self.dir_refused(e));
        }
        let target = atomic::resolve(&self.file_path);
        atomic::probe_temp_file(&target).map_err(|e| {
            Error::io(format!("cannot write {}: {e}", target.display())).with_source(e)
        })
    }
}

/// Whether `c` may appear in an app name.
///
/// The one definition of the set. Names reach shell commands in hint text
/// and `next_step.command`, so it is narrow enough to need no quoting;
/// registration rejects anything else and the hint builder quotes the
/// grandfathered names an older store can still hold.
#[must_use]
#[doc(hidden)]
pub fn is_app_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')
}

/// Rejects a new app name carrying a character outside the set
/// [`is_app_name_char`] defines. Existing names load unchanged; only
/// registration validates.
fn validate_app_name(name: &str) -> Result<()> {
    if name.is_empty() || !name.chars().all(is_app_name_char) {
        return Err(Error::validation_of("invalid-app-name", name));
    }
    Ok(())
}
