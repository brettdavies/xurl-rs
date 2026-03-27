/// Token and app type definitions for the multi-app credential store.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ── Token types ──────────────────────────────────────────────────────

/// `OAuth1` authentication tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth1Token {
    pub access_token: String,
    pub token_secret: String,
    pub consumer_key: String,
    pub consumer_secret: String,
}

/// `OAuth2` authentication tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expiration_time: u64,
}

/// Token type discriminator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Bearer,
    Oauth2,
    Oauth1,
}

/// A token with its type and payload.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    #[serde(rename = "type")]
    pub token_type: TokenType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2: Option<OAuth2Token>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth1: Option<OAuth1Token>,
}

// ── App ──────────────────────────────────────────────────────────────

/// Holds credentials and tokens for a single registered X API application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub client_id: String,
    pub client_secret: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub default_user: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub oauth2_tokens: BTreeMap<String, Token>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth1_token: Option<Token>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<Token>,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            default_user: String::new(),
            oauth2_tokens: BTreeMap::new(),
            oauth1_token: None,
            bearer_token: None,
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
        !self.oauth2_tokens.is_empty() || self.oauth1_token.is_some() || self.bearer_token.is_some()
    }
}

// ── On-disk YAML structure ───────────────────────────────────────────

/// Serialised YAML layout of `~/.xurl`.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StoreFile {
    pub apps: BTreeMap<String, App>,
    pub default_app: String,
}
