//! Credentials an embedder holds in code, and the hook that receives a
//! rotated `OAuth2` pair.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::api::auth_matrix::WireScheme;
use crate::error::{Error, Result};
use crate::store::OAuth1Token;

use super::oauth1;
use super::oauth2;

/// The error a token-refresh hook may return.
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// An `OAuth2` user token pair plus the app credentials that can refresh it.
///
/// `refresh_token: None` means the client never refreshes and an expired
/// token surfaces as [`Error::Auth`]; `expires_at: None` means the token is
/// used until X rejects it.
#[derive(Clone, PartialEq, Eq)]
pub struct OAuth2Credential {
    /// The app's `OAuth2` client ID.
    pub client_id: String,
    /// The app's `OAuth2` client secret; the refresh request sends it as
    /// HTTP Basic auth alongside `client_id` in the form.
    pub client_secret: String,
    /// The bearer access token sent on every user-context request.
    pub access_token: String,
    /// The refresh token X issued alongside the access token.
    pub refresh_token: Option<String>,
    /// When the access token stops being accepted.
    pub expires_at: Option<SystemTime>,
}

impl OAuth2Credential {
    /// Whether the declared expiry has passed.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|at| SystemTime::now() >= at)
    }
}

impl fmt::Debug for OAuth2Credential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OAuth2Credential")
            .field("client_id", &self.client_id)
            .field("client_secret", &REDACTED)
            .field("access_token", &REDACTED)
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| REDACTED),
            )
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

/// `OAuth1` HMAC-SHA1 credentials: the per-app consumer pair and the
/// per-user access pair.
#[derive(Clone, PartialEq, Eq)]
pub struct OAuth1Credential {
    /// The app's consumer key.
    pub consumer_key: String,
    /// The app's consumer secret.
    pub consumer_secret: String,
    /// The user's access token.
    pub access_token: String,
    /// The secret paired with [`Self::access_token`].
    pub token_secret: String,
}

impl fmt::Debug for OAuth1Credential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OAuth1Credential")
            .field("consumer_key", &self.consumer_key)
            .field("consumer_secret", &REDACTED)
            .field("access_token", &REDACTED)
            .field("token_secret", &REDACTED)
            .finish()
    }
}

/// Stands in for a secret in `Debug` output so a logged credential never
/// carries the value.
pub(crate) const REDACTED: &str = "<redacted>";

/// Receives every rotated `OAuth2` pair the client installs.
///
/// X rotates the refresh token on every refresh, so the only live copy of
/// the new pair is the one the client holds in memory until this hook
/// persists it. The client calls the hook after installing the new state
/// and before answering the request that triggered the refresh; an `Err`
/// fails that request with [`Error::TokenStore`] while the new token stays
/// installed, so a retry does not refresh a second time.
///
/// The future is boxed so the client can hold the hook as a trait object.
pub trait OnTokenRefreshed: Send + Sync + 'static {
    /// Persists `credential`, the pair the client now sends.
    fn on_token_refreshed<'a>(
        &'a self,
        credential: &'a OAuth2Credential,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), BoxError>> + Send + 'a>>;
}

/// The credentials a client built with [`crate::api::ClientBuilder`] holds.
pub(crate) struct DirectCredentials {
    pub(crate) bearer: Option<String>,
    pub(crate) oauth1: Option<OAuth1Credential>,
    pub(crate) oauth2: Option<OAuth2Credential>,
    pub(crate) hook: Option<Arc<dyn OnTokenRefreshed>>,
    pub(crate) token_url: String,
    pub(crate) timeout: Duration,
}

impl DirectCredentials {
    pub(crate) fn is_empty(&self) -> bool {
        self.bearer.is_none() && self.oauth1.is_none() && self.oauth2.is_none()
    }

    /// Schemes with a credential behind them, in auto-detect preference
    /// order.
    pub(crate) fn available(&self) -> Vec<WireScheme> {
        WireScheme::ALL_BY_PREFERENCE
            .into_iter()
            .filter(|scheme| match scheme {
                WireScheme::OAuth2 => self.oauth2.is_some(),
                WireScheme::OAuth1 => self.oauth1.is_some(),
                WireScheme::App => self.bearer.is_some(),
            })
            .collect()
    }

    pub(crate) fn oauth1_header(&self, method: &str, url: &str) -> Result<String> {
        let credential = self
            .oauth1
            .as_ref()
            .ok_or_else(|| Error::auth("TokenNotFound: OAuth1 token not found"))?;
        let token = OAuth1Token {
            access_token: credential.access_token.clone(),
            token_secret: credential.token_secret.clone(),
            consumer_key: credential.consumer_key.clone(),
            consumer_secret: credential.consumer_secret.clone(),
        };
        oauth1::build_oauth1_header(method, url, &token, None)
    }

    pub(crate) fn bearer_header(&self) -> Result<String> {
        self.bearer
            .as_ref()
            .map(|token| format!("Bearer {token}"))
            .ok_or_else(|| Error::auth("TokenNotFound: bearer token not found"))
    }

    pub(crate) fn has_oauth2(&self) -> bool {
        self.oauth2.is_some()
    }

    pub(crate) fn unexpired_oauth2_access_token(&self) -> Option<String> {
        self.oauth2
            .as_ref()
            .filter(|credential| !credential.is_expired())
            .map(|credential| credential.access_token.clone())
    }

    /// The access token to send, refreshing an expired one through the
    /// token endpoint and handing the rotated pair to the hook.
    pub(crate) async fn refresh_oauth2(&mut self, http: &reqwest::Client) -> Result<String> {
        let token_url = self.token_url.clone();
        let timeout = self.timeout;
        let hook = self.hook.clone();
        let credential = self
            .oauth2
            .as_mut()
            .ok_or_else(|| Error::auth(crate::error::NO_OAUTH2_TOKEN))?;
        if !credential.is_expired() {
            return Ok(credential.access_token.clone());
        }
        let refresh_token = credential.refresh_token.as_deref().ok_or_else(|| {
            Error::auth("TokenExpired: oauth2 access token expired and no refresh token was given")
        })?;
        let refreshed = oauth2::refresh_grant(
            http,
            &token_url,
            timeout,
            &credential.client_id,
            &credential.client_secret,
            refresh_token,
        )
        .await?;
        credential.access_token = refreshed.access_token;
        if let Some(rotated) = refreshed.refresh_token {
            credential.refresh_token = Some(rotated);
        }
        credential.expires_at = Some(refreshed.expires_at);
        if let Some(hook) = hook {
            hook.on_token_refreshed(credential)
                .await
                .map_err(|e| Error::token_store(format!("token refresh hook failed: {e}")))?;
        }
        Ok(credential.access_token.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_carries_no_secret() {
        let oauth2 = OAuth2Credential {
            client_id: "cid".to_string(),
            client_secret: "hush-secret".to_string(),
            access_token: "hush-access".to_string(),
            refresh_token: Some("hush-refresh".to_string()),
            expires_at: None,
        };
        let rendered = format!("{oauth2:?}");
        assert!(rendered.contains("cid"));
        assert!(!rendered.contains("hush"), "{rendered}");

        let oauth1 = OAuth1Credential {
            consumer_key: "ck".to_string(),
            consumer_secret: "hush-consumer".to_string(),
            access_token: "hush-access".to_string(),
            token_secret: "hush-token".to_string(),
        };
        let rendered = format!("{oauth1:?}");
        assert!(rendered.contains("ck"));
        assert!(!rendered.contains("hush"), "{rendered}");
    }

    #[test]
    fn expiry_is_read_against_the_clock() {
        let mut credential = OAuth2Credential {
            client_id: String::new(),
            client_secret: String::new(),
            access_token: String::new(),
            refresh_token: None,
            expires_at: None,
        };
        assert!(!credential.is_expired(), "no expiry means never expired");
        credential.expires_at = Some(SystemTime::now() - Duration::from_secs(1));
        assert!(credential.is_expired());
        credential.expires_at = Some(SystemTime::now() + Duration::from_secs(60));
        assert!(!credential.is_expired());
    }
}
