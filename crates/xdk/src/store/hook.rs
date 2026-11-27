//! The token store as a refresh hook: a rotated `OAuth2` pair lands in a
//! named app, under the user the next load would read.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use crate::auth::oauth2::epoch_secs;
use crate::auth::{BoxError, OAuth2Credential, OnTokenRefreshed};
use crate::error::Error;

use super::TokenStore;

/// Persists rotated `OAuth2` pairs into one app of a token store.
///
/// Built by [`TokenStore::refresh_hook_for`]; the store itself implements
/// the hook for its default app. A pair whose client is not the app's is
/// refused, since that app's client could not refresh it.
#[derive(Clone, Debug)]
pub struct StoreRefreshHook {
    path: PathBuf,
    /// Empty means the store's default app at the time of the write.
    app: String,
}

impl OnTokenRefreshed for StoreRefreshHook {
    fn on_token_refreshed<'a>(
        &'a self,
        credential: &'a OAuth2Credential,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), BoxError>> + Send + 'a>> {
        let path = self.path.clone();
        let app = self.app.clone();
        let client_id = credential.client_id.clone();
        let access_token = credential.access_token.clone();
        let refresh_token = credential.refresh_token.clone().unwrap_or_default();
        // An unknown expiry is stored as already expired, so the next load
        // refreshes and learns the real one instead of sending a token X may
        // have stopped accepting.
        let expiration_time = credential.expires_at.map_or(0, epoch_secs);
        Box::pin(async move {
            // The app and user are resolved against the store as re-read under
            // the lock, so the write follows a default another process changed
            // since this hook was built, to an app whose client minted the pair.
            TokenStore::update_at(path, move |store| {
                let app = if app.is_empty() {
                    store.default_app.clone()
                } else {
                    app
                };
                let stored_client_id = &store.resolve_app(&app).client_id;
                if !stored_client_id.is_empty() && *stored_client_id != client_id {
                    return Err(Error::token_store(format!(
                        "app {app:?} signs in through another client than the one that refreshed \
                         this OAuth2 pair, so the pair is not stored there"
                    )));
                }
                let user = store
                    .first_oauth2_user_for_app(&app)
                    .map(|(username, _)| username.to_string());
                match user {
                    Some(username) => store.save_oauth2_token_for_app(
                        &app,
                        &username,
                        &access_token,
                        &refresh_token,
                        expiration_time,
                    ),
                    None => store.save_oauth2_token_unnamed_for_app(
                        &app,
                        &access_token,
                        &refresh_token,
                        expiration_time,
                    ),
                }
            })
            .await
            .map_err(BoxError::from)
        })
    }
}

impl OnTokenRefreshed for TokenStore {
    /// Persists into the store's default app; [`TokenStore::refresh_hook_for`]
    /// targets another.
    fn on_token_refreshed<'a>(
        &'a self,
        credential: &'a OAuth2Credential,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), BoxError>> + Send + 'a>> {
        let hook = self.refresh_hook_for("");
        Box::pin(async move { hook.on_token_refreshed(credential).await })
    }
}

impl TokenStore {
    /// A refresh hook that persists into `app_name`; an empty name means the
    /// store's default app at the time of each write.
    #[must_use]
    pub fn refresh_hook_for(&self, app_name: &str) -> StoreRefreshHook {
        StoreRefreshHook {
            path: self.file_path.clone(),
            app: app_name.to_string(),
        }
    }
}
