//! The signatures X puts on webhook requests, and the app secrets behind them.

use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::auth::Auth;
use crate::error::{Error, Result};

type HmacSha256 = Hmac<Sha256>;

/// The header X signs an event in when the app has an `OAuth2` client
/// secret.
pub const OAUTH2_SIGNATURE_HEADER: &str = "x-twitter-webhooks-signature-oauth2";
/// The header X signs an event in, under the `OAuth1` consumer secret, when
/// the app has no `OAuth2` client secret.
pub const LEGACY_SIGNATURE_HEADER: &str = "x-twitter-webhooks-signature";
const SIGNATURE_PREFIX: &str = "sha256=";

/// The body every [`SignedSample`] carries. It marks itself as a test, so a
/// target can tell it from a delivery X made.
const SAMPLE_BODY: &str = r#"{"xr_test_event":true,"detail":"A signed sample for testing a webhook receiver. X did not send it."}"#;

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

/// A sample event signed the way X signs a delivery, for proving that a
/// receiver and whatever stands in front of it pass a signed event through
/// before X sends one.
///
/// The signature covers the sample's own body only, so whoever holds it can
/// deliver that body and no other. `Debug` prints the signature, which
/// reveals nothing about the secret behind it.
///
/// # Example
///
/// ```
/// use xdk::webhooks::{OAUTH2_SIGNATURE_HEADER, SignedSample, SigningSecrets};
///
/// let secrets = SigningSecrets {
///     oauth2_client_secret: Some("the app's client secret".to_string()),
///     oauth1_consumer_secret: None,
/// };
/// let sample = SignedSample::new(&secrets)?;
/// assert_eq!(sample.signature_header, OAUTH2_SIGNATURE_HEADER);
/// assert!(sample.signature.starts_with("sha256="));
/// for (name, value) in sample.headers() {
///     println!("{name}: {value}");
/// }
/// # Ok::<(), xdk::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SignedSample {
    /// The event body: a JSON object whose `xr_test_event` is `true`.
    pub body: &'static str,
    /// The header the signature travels in: [`OAUTH2_SIGNATURE_HEADER`]
    /// under a client secret, [`LEGACY_SIGNATURE_HEADER`] under a consumer
    /// secret alone.
    pub signature_header: &'static str,
    /// `sha256=` and the base64 HMAC-SHA256 of the body.
    pub signature: String,
}

impl SignedSample {
    /// Signs the sample with the secret a [`Receiver`](super::Receiver)
    /// built from `secrets` verifies with: the client secret when there is
    /// one, the consumer secret otherwise.
    ///
    /// # Errors
    ///
    /// Returns a `validation` error when `secrets` holds no secret.
    pub fn new(secrets: &SigningSecrets) -> Result<Self> {
        let (secret, signature_header) = if let Some(secret) =
            present(secrets.oauth2_client_secret.as_deref())
        {
            (secret, OAUTH2_SIGNATURE_HEADER)
        } else if let Some(secret) = present(secrets.oauth1_consumer_secret.as_deref()) {
            (secret, LEGACY_SIGNATURE_HEADER)
        } else {
            return Err(Error::validation(
                "a signed sample needs the app's OAuth2 client secret or OAuth1 consumer secret",
            ));
        };
        Ok(Self {
            body: SAMPLE_BODY,
            signature_header,
            signature: sign(secret, SAMPLE_BODY.as_bytes())?,
        })
    }

    /// The headers a delivery of the sample carries: `content-type` and the
    /// signature.
    #[must_use]
    pub fn headers(&self) -> [(&'static str, &str); 2] {
        [
            ("content-type", "application/json"),
            (self.signature_header, &self.signature),
        ]
    }
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
