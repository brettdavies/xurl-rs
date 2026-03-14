/// OAuth2 PKCE flow and token refresh.
///
/// Implements the browser-based OAuth2 authorization code flow with PKCE
/// (Proof Key for Code Exchange) as used by the X API.
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use url::Url;

use super::Auth;
use super::callback;
use crate::error::{Result, XurlError};

/// OAuth2 scopes requested for xurl.
fn get_oauth2_scopes() -> Vec<&'static str> {
    vec![
        // Read scopes
        "tweet.read",
        "users.read",
        "bookmark.read",
        "follows.read",
        "list.read",
        "block.read",
        "mute.read",
        "like.read",
        "users.email",
        "dm.read",
        // Write scopes
        "tweet.write",
        "tweet.moderate.write",
        "follows.write",
        "bookmark.write",
        "block.write",
        "mute.write",
        "like.write",
        "list.write",
        "media.write",
        "dm.write",
        // Other scopes
        "offline.access",
        "space.read",
    ]
}

/// Generates a PKCE code verifier and its S256 challenge.
fn generate_code_verifier_and_challenge() -> (String, String) {
    let b: [u8; 32] = rand::random();
    let verifier = URL_SAFE_NO_PAD.encode(b);
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());
    (verifier, challenge)
}

/// Runs the full OAuth2 PKCE authorization flow.
pub fn run_oauth2_flow(auth: &mut Auth, username: &str) -> Result<String> {
    // Generate state parameter
    let state_bytes: [u8; 32] = rand::random();
    let state = BASE64_STANDARD.encode(state_bytes);

    let (verifier, challenge) = generate_code_verifier_and_challenge();

    // Build authorization URL
    let scopes = get_oauth2_scopes().join(" ");
    let mut auth_url = Url::parse(auth.auth_url())
        .map_err(|e| XurlError::auth_with_cause("InvalidURL", &e))?;
    auth_url
        .query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", auth.client_id())
        .append_pair("redirect_uri", auth.redirect_uri())
        .append_pair("scope", &scopes)
        .append_pair("state", &state)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256");

    let auth_url_str = auth_url.to_string();

    // Try to open browser
    if let Err(_e) = open::that(&auth_url_str) {
        println!("Failed to open browser automatically. Please visit this URL manually:");
        println!("{auth_url_str}");
    }

    // Parse redirect URI to get callback port
    let redirect_parsed = Url::parse(auth.redirect_uri())
        .map_err(|e| XurlError::auth_with_cause("InvalidURL", &e))?;
    let port = redirect_parsed.port().unwrap_or(8080);

    // Start callback server and wait for code
    let code = callback::wait_for_callback(port, &state)?;

    // Exchange code for token
    let client = reqwest::blocking::Client::new();
    let token_resp = client
        .post(auth.token_url())
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", auth.redirect_uri()),
            ("client_id", auth.client_id()),
            ("code_verifier", &verifier),
        ])
        .basic_auth(auth.client_id(), Some(auth.client_secret()))
        .send()
        .map_err(|e| XurlError::auth_with_cause("TokenExchangeError", &e))?;

    let token_data: serde_json::Value = token_resp
        .json()
        .map_err(|e| XurlError::auth_with_cause("TokenExchangeError", &e))?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| XurlError::auth("TokenExchangeError: no access_token in response"))?
        .to_string();

    let refresh_token = token_data["refresh_token"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let expires_in = token_data["expires_in"].as_u64().unwrap_or(7200);

    // Resolve username
    let username_str = if !username.is_empty() {
        username.to_string()
    } else {
        auth.fetch_username(&access_token)?
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expiration_time = now + expires_in;

    auth.token_store.save_oauth2_token(
        &username_str,
        &access_token,
        &refresh_token,
        expiration_time,
    )?;

    Ok(access_token)
}

/// Refreshes an OAuth2 token if expired.
pub fn refresh_oauth2_token(auth: &mut Auth, username: &str) -> Result<String> {
    let token = if !username.is_empty() {
        auth.token_store.get_oauth2_token(username).cloned()
    } else {
        auth.token_store.get_first_oauth2_token().cloned()
    };

    let token = token.ok_or_else(|| XurlError::auth("TokenNotFound: oauth2 token not found"))?;
    let oauth2 = token
        .oauth2
        .as_ref()
        .ok_or_else(|| XurlError::auth("TokenNotFound: oauth2 token not found"))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();

    // Token is still valid
    if now < oauth2.expiration_time {
        return Ok(oauth2.access_token.clone());
    }

    // Token is expired, refresh it
    let client = reqwest::blocking::Client::new();
    let token_resp = client
        .post(auth.token_url())
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", &oauth2.refresh_token),
            ("client_id", auth.client_id()),
        ])
        .basic_auth(auth.client_id(), Some(auth.client_secret()))
        .send()
        .map_err(|e| XurlError::auth_with_cause("RefreshTokenError", &e))?;

    let token_data: serde_json::Value = token_resp
        .json()
        .map_err(|e| XurlError::auth_with_cause("RefreshTokenError", &e))?;

    let new_access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| XurlError::auth("RefreshTokenError: no access_token in response"))?
        .to_string();

    let new_refresh_token = token_data["refresh_token"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let expires_in = token_data["expires_in"].as_u64().unwrap_or(7200);

    // Resolve username
    let username_str = if !username.is_empty() {
        username.to_string()
    } else {
        auth.fetch_username(&new_access_token)
            .map_err(|e| XurlError::auth_with_cause("UsernameFetchError", &e))?
    };

    let new_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expiration_time = new_now + expires_in;

    auth.token_store.save_oauth2_token(
        &username_str,
        &new_access_token,
        &new_refresh_token,
        expiration_time,
    )?;

    Ok(new_access_token)
}
