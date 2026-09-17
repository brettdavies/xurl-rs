//! Building a client from credentials held in code.

use std::sync::Arc;
use std::time::Duration;

use crate::auth::{DirectCredentials, OAuth1Credential, OAuth2Credential, OnTokenRefreshed};
use crate::config::{DEFAULT_API_BASE_URL, DEFAULT_TOKEN_URL};
use crate::error::{Error, Result};

use super::source::CredentialSource;
use super::{Client, DEFAULT_TIMEOUT_SECS};

/// Builds a [`Client`] from the credentials an embedder holds.
///
/// More than one scheme may be set; a call with no explicit scheme picks
/// per the endpoint's auth matrix in `OAuth2`, `OAuth1`, Bearer order.
///
/// ```rust,no_run
/// use xurl::api::Client;
///
/// # fn run() -> xurl::Result<()> {
/// let client = Client::builder().bearer("app-only-token").build()?;
/// # let _ = client;
/// # Ok(()) }
/// ```
#[must_use = "a ClientBuilder does nothing until build() is called"]
pub struct ClientBuilder {
    bearer: Option<String>,
    oauth2: Option<OAuth2Credential>,
    oauth1: Option<OAuth1Credential>,
    hook: Option<Arc<dyn OnTokenRefreshed>>,
    base_url: String,
    token_url: String,
    timeout: Duration,
}

impl ClientBuilder {
    pub(crate) fn new() -> Self {
        Self {
            bearer: None,
            oauth2: None,
            oauth1: None,
            hook: None,
            base_url: DEFAULT_API_BASE_URL.to_string(),
            token_url: DEFAULT_TOKEN_URL.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    /// An app-only bearer token: reads public data, cannot act as a user.
    pub fn bearer(mut self, token: impl Into<String>) -> Self {
        self.bearer = Some(token.into());
        self
    }

    /// An `OAuth2` user token pair; the client refreshes it when it expires
    /// and hands the rotated pair to the [`Self::on_token_refreshed`] hook.
    pub fn oauth2(mut self, credential: OAuth2Credential) -> Self {
        self.oauth2 = Some(credential);
        self
    }

    /// `OAuth1` user credentials; every request under them is HMAC-SHA1
    /// signed.
    pub fn oauth1(mut self, credential: OAuth1Credential) -> Self {
        self.oauth1 = Some(credential);
        self
    }

    /// Receives every rotated `OAuth2` pair; see [`OnTokenRefreshed`].
    pub fn on_token_refreshed(mut self, hook: impl OnTokenRefreshed) -> Self {
        self.hook = Some(Arc::new(hook));
        self
    }

    /// The API origin every path is built on.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// The `OAuth2` token endpoint a refresh posts to.
    pub fn token_url(mut self, url: impl Into<String>) -> Self {
        self.token_url = url.into();
        self
    }

    /// Bounds every non-streaming request from connect to body end; a
    /// refresh uses the same bound. [`super::Call::timeout`] overrides it
    /// for one call.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when no credential was given and
    /// [`Error::Http`] when the HTTP client cannot be built.
    pub fn build(self) -> Result<Client> {
        let credentials = DirectCredentials {
            bearer: self.bearer,
            oauth1: self.oauth1,
            oauth2: self.oauth2,
            hook: self.hook,
            token_url: self.token_url,
            timeout: self.timeout,
        };
        if credentials.is_empty() {
            return Err(Error::validation(
                "no credential given: set a bearer token, an OAuth2 credential, or OAuth1 credentials before build()",
            ));
        }
        Client::from_source(
            self.base_url,
            CredentialSource::Direct(credentials),
            self.timeout,
        )
    }
}
