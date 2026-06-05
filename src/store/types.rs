//! Token and app type definitions for the multi-app credential store.
//!
//! These types form the on-disk YAML schema for `~/.xurl` and the in-memory
//! shape that [`crate::store::TokenStore`] exposes to library consumers.
//! `App` groups one set of X API client credentials with the user / bearer
//! tokens authorized against them; `Token` is the polymorphic envelope that
//! carries an `OAuth1`, `OAuth2`, or bearer payload alongside its
//! discriminator.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ── Token types ──────────────────────────────────────────────────────

/// `OAuth1` HMAC-SHA1 access-token bundle.
///
/// Carries the four secrets needed to sign an `OAuth1` request: the
/// per-user access pair (`access_token` / `token_secret`) and the
/// per-app consumer pair (`consumer_key` / `consumer_secret`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth1Token {
    /// `OAuth1` user access token (the per-user secret).
    pub access_token: String,
    /// `OAuth1` user token secret, paired with [`Self::access_token`].
    pub token_secret: String,
    /// `OAuth1` consumer key (the per-app identifier).
    pub consumer_key: String,
    /// `OAuth1` consumer secret, paired with [`Self::consumer_key`].
    pub consumer_secret: String,
}

/// `OAuth2` PKCE access + refresh token pair with expiration.
///
/// `expiration_time` is a Unix epoch second; the refresh path treats any
/// `expiration_time <= now` as expired and POSTs the refresh-grant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Token {
    /// Bearer access token returned by the `OAuth2` token endpoint.
    pub access_token: String,
    /// Refresh token used to mint a new access token after expiry.
    pub refresh_token: String,
    /// Unix epoch second at which [`Self::access_token`] expires.
    pub expiration_time: u64,
}

/// Token-type discriminator carried alongside [`Token`].
///
/// Serialised lowercase in YAML (`bearer` / `oauth2` / `oauth1`) so the
/// on-disk store remains compatible with the upstream Go xurl format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    /// App-only bearer-token authentication.
    Bearer,
    /// `OAuth2` PKCE user-authorized token.
    Oauth2,
    /// `OAuth1` HMAC-SHA1 user-authorized token.
    Oauth1,
}

/// Polymorphic token envelope: a [`TokenType`] discriminator plus exactly
/// one populated payload field.
///
/// Only the variant matching [`Self::token_type`] is populated; the other
/// two are `None` and skipped on serialize. Callers select the right
/// helper on [`crate::store::TokenStore`] (`get_oauth1_tokens`,
/// `get_oauth2_token`, `get_bearer_token`) rather than reading these fields
/// directly.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Discriminator for which payload field is populated.
    #[serde(rename = "type")]
    pub token_type: TokenType,
    /// Raw bearer-token string when [`Self::token_type`] is
    /// [`TokenType::Bearer`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer: Option<String>,
    /// `OAuth2` payload when [`Self::token_type`] is [`TokenType::Oauth2`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2: Option<OAuth2Token>,
    /// `OAuth1` payload when [`Self::token_type`] is [`TokenType::Oauth1`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth1: Option<OAuth1Token>,
}

// ── App ──────────────────────────────────────────────────────────────

/// Credentials and tokens for a single registered X API application.
///
/// One `App` corresponds to one X API client (one `client_id` /
/// `client_secret` pair); it holds every `OAuth2` user token authorized
/// against that app keyed by username, plus optional `OAuth1` and bearer
/// tokens. The token store carries an arbitrary number of apps and a
/// `default_app` name; the `--app NAME` flag selects between them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    /// `OAuth2` client ID issued by X for this app.
    pub client_id: String,
    /// `OAuth2` client secret paired with [`Self::client_id`].
    pub client_secret: String,
    /// Default `OAuth2` username for this app. When non-empty, lookups
    /// without an explicit username prefer this entry over arbitrary-first.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub default_user: String,
    /// Stored `OAuth2` redirect URI override; empty means "fall through to
    /// `REDIRECT_URI` env or the built-in default".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub redirect_uri: String,
    /// `OAuth2` user tokens, keyed by username (the value returned by
    /// `/2/users/me`, or the explicit name passed to `xr auth oauth2 NAME`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub oauth2_tokens: BTreeMap<String, Token>,
    /// `OAuth1` token for this app, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth1_token: Option<Token>,
    /// Bearer token for this app, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<Token>,
    /// Salvage slot for an `OAuth2` token whose `/2/users/me` lookup failed
    /// during exchange. Lets the token-exchange path persist the access
    /// token rather than discarding it when the username cannot be
    /// resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unnamed_oauth2_token: Option<Token>,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            default_user: String::new(),
            redirect_uri: String::new(),
            oauth2_tokens: BTreeMap::new(),
            oauth1_token: None,
            bearer_token: None,
            unnamed_oauth2_token: None,
        }
    }

    pub(crate) fn with_credentials(client_id: &str, client_secret: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            ..Self::new()
        }
    }

    pub(crate) fn has_tokens(&self) -> bool {
        !self.oauth2_tokens.is_empty()
            || self.oauth1_token.is_some()
            || self.bearer_token.is_some()
            || self.unnamed_oauth2_token.is_some()
    }
}

// ── On-disk YAML structure ───────────────────────────────────────────

/// Serialised YAML layout of `~/.xurl`.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StoreFile {
    pub apps: BTreeMap<String, App>,
    pub default_app: String,
}
