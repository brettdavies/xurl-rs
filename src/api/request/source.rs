//! Where a client's credentials come from: a token store the binary
//! resolves apps and users against, or credentials an embedder holds in
//! code.

use crate::api::auth_matrix::WireScheme;
use crate::auth::{Auth, DirectCredentials};
use crate::error::Result;

pub(crate) enum CredentialSource {
    Store(Auth),
    Direct(DirectCredentials),
}

/// What scheme selection reads off the credentials at hand.
pub(crate) struct SchemeFacts {
    /// Schemes with a usable credential, in auto-detect preference order.
    pub(crate) available: Vec<WireScheme>,
    /// Schemes the active app stores; an env-supplied bearer is available
    /// without being stored.
    pub(crate) stored: Vec<WireScheme>,
    /// The active app's name, for a store-backed client.
    pub(crate) app: Option<String>,
}

impl CredentialSource {
    pub(crate) fn store(&mut self) -> Option<&mut Auth> {
        match self {
            Self::Store(auth) => Some(auth),
            Self::Direct(_) => None,
        }
    }

    /// The app the credentials are read from, for a store-backed client.
    ///
    /// An empty active app name is the "use default app" convention; the
    /// envelope names the store's actual default so the user has something
    /// to act on.
    pub(crate) fn active_app(&self) -> Option<String> {
        match self {
            Self::Direct(_) => None,
            Self::Store(auth) => {
                let raw_app = auth.app_name();
                Some(if raw_app.is_empty() {
                    auth.token_store.default_app.clone()
                } else {
                    raw_app.to_string()
                })
            }
        }
    }

    pub(crate) fn scheme_facts(&self) -> SchemeFacts {
        match self {
            Self::Direct(direct) => {
                let available = direct.available();
                SchemeFacts {
                    stored: available.clone(),
                    available,
                    app: None,
                }
            }
            Self::Store(auth) => {
                let app_name = self.active_app().unwrap_or_default();
                let stored = stored_in_app(auth, &app_name);
                let mut available = stored.clone();
                if auth.env_bearer_token_present() && !available.contains(&WireScheme::App) {
                    available.push(WireScheme::App);
                }
                SchemeFacts {
                    available,
                    stored,
                    app: Some(app_name),
                }
            }
        }
    }

    /// Other apps in the store holding credentials. Only the wrong-app
    /// envelope reads this, so it walks the store on demand rather than on
    /// every request.
    pub(crate) fn other_apps_with_creds(&self, app_name: &str) -> Vec<String> {
        match self {
            Self::Store(auth) => auth
                .token_store
                .apps_with_credentials()
                .into_iter()
                .filter(|name| name != app_name)
                .collect(),
            Self::Direct(_) => Vec::new(),
        }
    }

    pub(crate) fn oauth1_header(&self, method: &str, url: &str) -> Result<String> {
        match self {
            Self::Store(auth) => auth.get_oauth1_header(method, url, None),
            Self::Direct(direct) => direct.oauth1_header(method, url),
        }
    }

    pub(crate) fn bearer_header(&self) -> Result<String> {
        match self {
            Self::Store(auth) => auth.get_bearer_token_header(),
            Self::Direct(direct) => direct.bearer_header(),
        }
    }

    pub(crate) fn has_oauth2_token(&self, username: &str) -> bool {
        match self {
            Self::Store(auth) => auth.has_oauth2_token(username),
            Self::Direct(direct) => direct.has_oauth2(),
        }
    }

    pub(crate) fn unexpired_oauth2_access_token(&self, username: &str) -> Option<String> {
        match self {
            Self::Store(auth) => auth.unexpired_oauth2_access_token(username),
            Self::Direct(direct) => direct.unexpired_oauth2_access_token(),
        }
    }

    pub(crate) async fn refresh_oauth2_token(
        &mut self,
        http: &reqwest::Client,
        username: &str,
    ) -> Result<String> {
        match self {
            Self::Store(auth) => auth.refresh_oauth2_token(http, username).await,
            Self::Direct(direct) => direct.refresh_oauth2(http).await,
        }
    }
}

/// Schemes the active app stores, in preference order.
fn stored_in_app(auth: &Auth, app_name: &str) -> Vec<WireScheme> {
    let store = &auth.token_store;
    WireScheme::ALL_BY_PREFERENCE
        .into_iter()
        .filter(|scheme| match scheme {
            WireScheme::OAuth2 => store.get_first_oauth2_token_for_app(app_name).is_some(),
            WireScheme::OAuth1 => store.get_oauth1_tokens_for_app(app_name).is_some(),
            WireScheme::App => store.get_bearer_token_for_app(app_name).is_some(),
        })
        .collect()
}
