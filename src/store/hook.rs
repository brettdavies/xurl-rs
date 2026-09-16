//! The token store as a refresh hook: a rotated `OAuth2` pair lands in the
//! default app, under the user the next load would read.

use std::future::Future;
use std::pin::Pin;

use crate::auth::oauth2::epoch_secs;
use crate::auth::{BoxError, OAuth2Credential, OnTokenRefreshed};

use super::TokenStore;

impl OnTokenRefreshed for TokenStore {
    fn on_token_refreshed<'a>(
        &'a self,
        credential: &'a OAuth2Credential,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), BoxError>> + Send + 'a>> {
        let path = self.file_path.clone();
        let app = self.default_app.clone();
        let username = self.refresh_target_user(&app);
        let access_token = credential.access_token.clone();
        let refresh_token = credential.refresh_token.clone().unwrap_or_default();
        // An unknown expiry is stored as already expired, so the next load
        // refreshes and learns the real one instead of sending a token X may
        // have stopped accepting.
        let expiration_time = credential.expires_at.map_or(0, epoch_secs);
        Box::pin(async move {
            TokenStore::update_at(path, move |store| match &username {
                Some(username) => store.save_oauth2_token_for_app(
                    &app,
                    username,
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
            })
            .await
            .map_err(BoxError::from)
        })
    }
}

impl TokenStore {
    /// The user a rotated pair is saved under: the app's default user, then
    /// its first stored `OAuth2` user. Mirrors the read precedence so the
    /// write lands where the next lookup looks.
    fn refresh_target_user(&self, app_name: &str) -> Option<String> {
        let app = self.resolve_app(app_name);
        if !app.default_user.is_empty() {
            return Some(app.default_user.clone());
        }
        app.oauth2_tokens.keys().next().cloned()
    }
}
