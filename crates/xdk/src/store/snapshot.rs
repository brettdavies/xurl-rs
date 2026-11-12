//! Secret-free view of the token store, taken before dispatch.
//!
//! The CLI needs to choose a recovery hint at the point an error is about to
//! print, which is after the `Auth` value has moved into the command
//! handlers. This snapshot carries the few facts that choice needs, and no
//! secret: presence flags and login names per app, the default and active
//! app names, whether the environment supplied a client id, and how the file
//! loaded.
//!
//! ```text
//! file missing        -> Fresh       apps empty; saves allowed; hint: register
//! file read fails     -> Unreadable  apps empty; saves refused; hint: inspect
//! file parsed         -> Loaded      apps as stored; saves allowed
//! neither parser ok   -> Unparseable apps empty; saves refused; hint: inspect
//! ```

use std::collections::BTreeMap;

use super::{LoadState, TokenStore};
use crate::api::auth_matrix::WireScheme;

/// What one app carries, without carrying any of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppFacts {
    /// Whether the app has a non-empty `client_id`.
    pub has_client_id: bool,
    /// Whether the app holds any token: `OAuth2`, `OAuth1`, or bearer.
    pub has_tokens: bool,
    /// Whether the app holds an `OAuth2` login, named or not.
    pub has_oauth2: bool,
    /// Whether the app holds `OAuth1` tokens.
    pub has_oauth1: bool,
    /// Whether the app holds a bearer token.
    pub has_bearer: bool,
}

/// Secret-free snapshot of the store as one run observed it.
#[derive(Debug, Clone)]
pub struct StoreSnapshot {
    /// Presence flags per app name, in the store's stable ordering.
    pub apps: BTreeMap<String, AppFacts>,
    /// The store's default app name; empty when nothing is registered.
    pub default_app: String,
    /// The app this run targets: `--app`, `XURL_APP`, or the default.
    pub active_app: String,
    /// Whether `CLIENT_ID` supplied a client id for this run.
    pub env_client_id_present: bool,
    /// Whether `XURL_BEARER_TOKEN` supplied a bearer token for this run,
    /// which serves a request without being stored in any app.
    pub env_bearer_present: bool,
    /// How the backing file resolved.
    pub load_state: LoadState,
    /// Path of the backing file, so a hint can name it.
    pub store_path: String,
    /// The named `OAuth2` logins each app holds, the names `-u` selects, in
    /// stable order.
    pub oauth2_logins: BTreeMap<String, Vec<String>>,
    /// The login a request with no `-u` goes out as, per app that holds a
    /// named one: the default user, else the app's first login.
    pub oauth2_default_logins: BTreeMap<String, String>,
}

impl StoreSnapshot {
    /// Builds a snapshot from the loaded store and the run's resolved
    /// active app and environment client id.
    #[must_use]
    pub fn new(store: &TokenStore, active_app: &str, env_client_id_present: bool) -> Self {
        let apps = store
            .apps
            .iter()
            .map(|(name, app)| {
                (
                    name.clone(),
                    AppFacts {
                        has_client_id: !app.client_id.is_empty(),
                        has_tokens: app.has_tokens(),
                        has_oauth2: !app.oauth2_tokens.is_empty()
                            || app.unnamed_oauth2_token.is_some(),
                        has_oauth1: app.oauth1_token.is_some(),
                        has_bearer: app.bearer_token.is_some(),
                    },
                )
            })
            .collect();
        Self {
            apps,
            default_app: store.default_app.clone(),
            active_app: store.get_active_app_name(active_app).to_string(),
            env_client_id_present,
            env_bearer_present: false,
            load_state: store.load_state,
            store_path: store.file_path.display().to_string(),
            oauth2_logins: store
                .apps
                .iter()
                .map(|(name, app)| (name.clone(), app.oauth2_tokens.keys().cloned().collect()))
                .collect(),
            oauth2_default_logins: store
                .apps
                .keys()
                .filter_map(|name| {
                    let (login, _) = store.first_oauth2_user_for_app(name)?;
                    Some((name.clone(), login.to_string()))
                })
                .collect(),
        }
    }

    /// This snapshot with `XURL_BEARER_TOKEN` marked as supplying a bearer.
    #[must_use]
    pub fn with_env_bearer(mut self, present: bool) -> Self {
        self.env_bearer_present = present;
        self
    }

    /// The schemes `app` can serve a request with, in auto-detect preference
    /// order: what it stores, and the environment's bearer for the active
    /// app.
    #[must_use]
    pub fn schemes_held(&self, app: &str) -> Vec<WireScheme> {
        let facts = self.apps.get(app);
        let env_bearer = self.env_bearer_present && app == self.active_app;
        WireScheme::ALL_BY_PREFERENCE
            .into_iter()
            .filter(|scheme| match scheme {
                WireScheme::OAuth2 => facts.is_some_and(|facts| facts.has_oauth2),
                WireScheme::OAuth1 => facts.is_some_and(|facts| facts.has_oauth1),
                WireScheme::App => env_bearer || facts.is_some_and(|facts| facts.has_bearer),
            })
            .collect()
    }

    /// Names of apps carrying a client id, in stable order.
    #[must_use]
    pub fn apps_with_client_id(&self) -> Vec<String> {
        self.apps
            .iter()
            .filter(|(_, facts)| facts.has_client_id)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Names of apps carrying a client id but no token, in stable order.
    #[must_use]
    pub fn apps_ready_to_sign_in(&self) -> Vec<String> {
        self.apps
            .iter()
            .filter(|(_, facts)| facts.has_client_id && !facts.has_tokens)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Names of apps holding an `OAuth2` login, named or not, in stable
    /// order.
    #[must_use]
    pub fn apps_with_login(&self) -> Vec<String> {
        self.apps
            .iter()
            .filter(|(_, facts)| facts.has_oauth2)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Names of apps holding the named `OAuth2` login `login`, in stable
    /// order.
    #[must_use]
    pub fn apps_holding_login(&self, login: &str) -> Vec<String> {
        self.oauth2_logins
            .iter()
            .filter(|(_, logins)| logins.iter().any(|held| held == login))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Whether the file existed but could not be read or parsed.
    #[must_use]
    pub fn load_failed(&self) -> bool {
        matches!(
            self.load_state,
            LoadState::Unreadable | LoadState::Unparseable
        )
    }
}
