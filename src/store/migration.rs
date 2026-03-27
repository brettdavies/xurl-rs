/// Legacy format migration — JSON and `.twurlrc` import.
use std::collections::BTreeMap;

use serde::Deserialize;

use super::TokenStore;
use super::types::{App, OAuth1Token, StoreFile, Token, TokenType};
use crate::error::Result;

// ── Legacy JSON structure (for migration) ────────────────────────────

#[derive(Debug, Deserialize)]
struct LegacyStore {
    oauth2_tokens: Option<BTreeMap<String, Token>>,
    #[serde(rename = "oauth1_tokens")]
    oauth1_token: Option<Token>,
    bearer_token: Option<Token>,
}

// ── Twurlrc structures ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields populated by serde deserialization
struct TwurlrcProfile {
    username: Option<String>,
    consumer_key: Option<String>,
    consumer_secret: Option<String>,
    token: Option<String>,
    secret: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TwurlrcConfig {
    profiles: Option<BTreeMap<String, BTreeMap<String, TwurlrcProfile>>>,
    bearer_tokens: Option<BTreeMap<String, String>>,
}

// ── Migration methods ───────────────────────────────────────────────

impl TokenStore {
    /// Tries YAML first, then falls back to legacy JSON migration.
    pub(crate) fn load_from_data(&mut self, data: &[u8]) {
        // Try new YAML format first
        if let Ok(sf) = serde_yaml::from_slice::<StoreFile>(data)
            && !sf.apps.is_empty()
        {
            self.apps = sf.apps;
            self.default_app = sf.default_app;
            return;
        }

        // Fall back to legacy JSON
        if let Ok(legacy) = serde_json::from_slice::<LegacyStore>(data) {
            let app = App {
                client_id: String::new(),
                client_secret: String::new(),
                default_user: String::new(),
                oauth2_tokens: legacy.oauth2_tokens.unwrap_or_default(),
                oauth1_token: legacy.oauth1_token,
                bearer_token: legacy.bearer_token,
            };
            self.apps.insert("default".to_string(), app);
            self.default_app = "default".to_string();
            // Persist in new YAML format immediately
            let _ = self.save_to_file();
        }
    }

    /// Imports tokens from a `.twurlrc` file into the active app.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, parsed, or the store cannot be saved.
    pub fn import_from_twurlrc(&mut self, file_path: &std::path::Path) -> Result<()> {
        let data = std::fs::read(file_path)?;
        let twurlrc: TwurlrcConfig = serde_yaml::from_slice(&data)?;

        let app = self.active_app_or_create();

        // Import the first OAuth1 tokens from twurlrc
        if let Some(profiles) = &twurlrc.profiles {
            'outer: for consumer_keys in profiles.values() {
                if let Some((consumer_key, profile)) = consumer_keys.iter().next() {
                    if app.oauth1_token.is_none() {
                        app.oauth1_token = Some(Token {
                            token_type: TokenType::Oauth1,
                            bearer: None,
                            oauth2: None,
                            oauth1: Some(OAuth1Token {
                                access_token: profile.token.clone().unwrap_or_default(),
                                token_secret: profile.secret.clone().unwrap_or_default(),
                                consumer_key: consumer_key.clone(),
                                consumer_secret: profile
                                    .consumer_secret
                                    .clone()
                                    .unwrap_or_default(),
                            }),
                        });
                    }
                    break 'outer;
                }
            }
        }

        // Import the first bearer token from twurlrc
        if let Some(bearer_tokens) = &twurlrc.bearer_tokens
            && let Some(bearer_token) = bearer_tokens.values().next()
        {
            app.bearer_token = Some(Token {
                token_type: TokenType::Bearer,
                bearer: Some(bearer_token.clone()),
                oauth2: None,
                oauth1: None,
            });
        }

        self.save_to_file()
    }
}
