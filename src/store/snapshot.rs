//! Secret-free view of the token store, taken before dispatch.
//!
//! The CLI needs to choose a recovery hint at the point an error is about to
//! print, which is after the `Auth` value has moved into the command
//! handlers. This snapshot carries the few facts that choice needs, and no
//! secret: presence flags per app, the default and active app names, whether
//! the environment supplied a client id, and how the file loaded.
//!
//! ```text
//! file missing        -> Fresh       apps empty; saves allowed; hint: register
//! file read fails     -> Unreadable  apps empty; saves refused; hint: inspect
//! file parsed         -> Loaded      apps as stored; saves allowed
//! neither parser ok   -> Unparseable apps empty; saves refused; hint: inspect
//! ```

use std::collections::BTreeMap;

use super::{LoadState, TokenStore};

/// What one app carries, without carrying any of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppFacts {
    /// Whether the app has a non-empty `client_id`.
    pub has_client_id: bool,
    /// Whether the app holds any token: `OAuth2`, `OAuth1`, or bearer.
    pub has_tokens: bool,
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
    /// How the backing file resolved.
    pub load_state: LoadState,
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
                    },
                )
            })
            .collect();
        Self {
            apps,
            default_app: store.default_app.clone(),
            active_app: store.get_active_app_name(active_app).to_string(),
            env_client_id_present,
            load_state: store.load_state,
        }
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

    /// Names of apps already holding a token, in stable order.
    #[must_use]
    pub fn apps_with_tokens(&self) -> Vec<String> {
        self.apps
            .iter()
            .filter(|(_, facts)| facts.has_tokens)
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
