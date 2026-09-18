//! An embedder-shaped consumer of `xdk`: every call here goes through the
//! documented surface (`xdk::api`, `xdk::auth`, `xdk::error`, `xdk::store`),
//! so hiding or removing something an embedder relies on fails this crate's
//! build before it fails a user's.
#![deny(missing_docs)]

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use xdk::api::{Client, RateLimit};
use xdk::auth::{BoxError, OAuth2Credential, OnTokenRefreshed};
use xdk::error::NextAction;

/// The most recent posts matching `query`, as `id: text` lines, with the
/// rate-limit window the response reported.
///
/// # Errors
///
/// Returns the library's [`xdk::Error`] unchanged so the caller can branch on
/// [`xdk::Error::next_action`].
pub async fn recent_posts(
    client: &Client,
    query: &str,
) -> xdk::Result<(Vec<String>, Option<RateLimit>)> {
    let response = client.search_posts(query, 10).send().await?;
    let lines = response
        .data
        .iter()
        .map(|post| format!("{}: {}", post.id, post.text))
        .collect();
    Ok((lines, client.last_rate_limit()))
}

/// What an embedder does with a failed call: name the kind, take the
/// library's next action, and map it to an exit code.
#[must_use]
pub fn describe_failure(error: &xdk::Error) -> (String, Option<NextAction>, i32) {
    (
        error.kind().to_string(),
        error.next_action(),
        error.exit_code(),
    )
}

/// An access token and the refresh token X rotated alongside it, which is
/// absent when the credential carried none.
pub type TokenPair = (String, Option<String>);

/// A hook that keeps the rotated pair in memory, standing in for the
/// secrets manager an embedder would persist to.
///
/// The builder takes a hook by value, so every clone shares one cell: hand
/// a clone to the client and keep one to read the rotated pair back.
#[derive(Debug, Default, Clone)]
pub struct RememberedPair {
    latest: Arc<Mutex<Option<TokenPair>>>,
}

impl RememberedPair {
    /// The access and refresh tokens the last refresh delivered.
    #[must_use]
    pub fn latest(&self) -> Option<TokenPair> {
        self.latest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl OnTokenRefreshed for RememberedPair {
    fn on_token_refreshed<'a>(
        &'a self,
        credential: &'a OAuth2Credential,
    ) -> Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'a>> {
        Box::pin(async move {
            *self
                .latest
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((
                credential.access_token.clone(),
                credential.refresh_token.clone(),
            ));
            Ok(())
        })
    }
}
