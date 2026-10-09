//! The signatures X puts on webhook requests, and the app secrets behind them.

use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::auth::Auth;
use crate::error::{Error, Result};

type HmacSha256 = Hmac<Sha256>;

pub(super) const OAUTH2_SIGNATURE_HEADER: &str = "x-twitter-webhooks-signature-oauth2";
pub(super) const LEGACY_SIGNATURE_HEADER: &str = "x-twitter-webhooks-signature";
const SIGNATURE_PREFIX: &str = "sha256=";

/// The secrets X signs with for one app.
///
/// `Debug` prints which secrets are present and never their values.
#[derive(Clone, Default)]
pub struct SigningSecrets {
    /// The app's `OAuth2` client secret. X signs with it when the app has one.
    pub oauth2_client_secret: Option<String>,
    /// The app's `OAuth1` consumer secret (API Secret Key), which X signs
    /// with for an app that has no `OAuth2` client secret.
    pub oauth1_consumer_secret: Option<String>,
}

impl SigningSecrets {
    /// The signing secrets of `auth`'s active app: the client secret as
    /// `auth` resolved it (the `CLIENT_SECRET` environment variable over the
    /// stored one) and the consumer secret of the app's stored `OAuth1`
    /// credentials.
    #[must_use]
    pub fn for_app(auth: &Auth) -> Self {
        let consumer_secret = auth
            .token_store
            .resolve_app(auth.app_name())
            .oauth1_token
            .as_ref()
            .and_then(|token| token.oauth1.as_ref())
            .map(|oauth1| oauth1.consumer_secret.clone());
        Self {
            oauth2_client_secret: present(Some(auth.client_secret())).map(str::to_string),
            oauth1_consumer_secret: consumer_secret.filter(|secret| !secret.is_empty()),
        }
    }

    /// Whether neither secret is present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.crc_secret().is_none()
    }

    /// The secret the CRC response token is keyed by: the client secret
    /// when present, the consumer secret otherwise.
    pub(super) fn crc_secret(&self) -> Option<&str> {
        present(self.oauth2_client_secret.as_deref())
            .or_else(|| present(self.oauth1_consumer_secret.as_deref()))
    }
}

impl fmt::Debug for SigningSecrets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown = |secret: &Option<String>| {
            if present(secret.as_deref()).is_some() {
                "<redacted>"
            } else {
                "<none>"
            }
        };
        f.debug_struct("SigningSecrets")
            .field("oauth2_client_secret", &shown(&self.oauth2_client_secret))
            .field(
                "oauth1_consumer_secret",
                &shown(&self.oauth1_consumer_secret),
            )
            .finish()
    }
}

pub(super) fn present(secret: Option<&str>) -> Option<&str> {
    secret.filter(|s| !s.is_empty())
}

/// Whether `given` is the signature of `body` under `secret`, compared in
/// constant time. A missing secret verifies nothing.
pub(super) fn verified(secret: Option<&str>, body: &[u8], given: &str) -> bool {
    let Some(secret) = present(secret) else {
        return false;
    };
    let Some(encoded) = given.strip_prefix(SIGNATURE_PREFIX) else {
        return false;
    };
    let Ok(digest) = BASE64_STANDARD.decode(encoded) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    mac.verify_slice(&digest).is_ok()
}

/// The signature X computes over `message` under `secret`:
/// `sha256=` followed by the base64 of the HMAC-SHA256.
///
/// This is both the CRC `response_token` for a `crc_token` and the value of
/// an event's signature header for its raw body.
///
/// # Errors
///
/// Returns an auth error if the HMAC cannot be keyed with `secret`.
pub fn sign(secret: &str, message: &[u8]) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| Error::auth_with_cause("SignatureGenerationError", &e).with_source(e))?;
    mac.update(message);
    Ok(format!(
        "{SIGNATURE_PREFIX}{}",
        BASE64_STANDARD.encode(mac.finalize().into_bytes())
    ))
}
