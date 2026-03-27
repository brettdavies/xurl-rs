/// `OAuth2` PKCE flow and token refresh.
///
/// Implements the browser-based `OAuth2` authorization code flow with PKCE
/// (Proof Key for Code Exchange) as used by the X API.
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use url::Url;

use super::Auth;
use super::callback;
use super::pending;
use crate::error::{Result, XurlError};

/// `OAuth2` scopes requested for xurl.
#[must_use]
pub fn get_oauth2_scopes() -> Vec<&'static str> {
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
#[must_use]
pub fn generate_code_verifier_and_challenge() -> (String, String) {
    let b: [u8; 32] = rand::random();
    let verifier = URL_SAFE_NO_PAD.encode(b);
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());
    (verifier, challenge)
}

/// Builds the `OAuth2` authorization URL with all required query parameters.
///
/// # Errors
///
/// Returns an error if the base authorization URL cannot be parsed.
pub(crate) fn build_auth_url(auth: &Auth, state: &str, challenge: &str) -> Result<String> {
    let scopes = get_oauth2_scopes().join(" ");
    let mut auth_url =
        Url::parse(auth.auth_url()).map_err(|e| XurlError::auth_with_cause("InvalidURL", &e))?;
    auth_url
        .query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", auth.client_id())
        .append_pair("redirect_uri", auth.redirect_uri())
        .append_pair("scope", &scopes)
        .append_pair("state", state)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256");

    Ok(auth_url.to_string())
}

/// Exchanges an authorization code for an access token and saves it.
///
/// Performs the full post-authorization pipeline: POST to token endpoint,
/// parse response, resolve username (fetching from API if empty), compute
/// expiration, and save to the token store.
///
/// # Errors
///
/// Returns an error if the token exchange request fails, the response is
/// missing an access token, or the username cannot be resolved.
pub(crate) fn exchange_code_for_token(
    auth: &mut Auth,
    code: &str,
    verifier: &str,
    username: &str,
) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new());
    let token_resp = client
        .post(auth.token_url())
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", auth.redirect_uri()),
            ("client_id", auth.client_id()),
            ("code_verifier", verifier),
        ])
        .basic_auth(auth.client_id(), Some(auth.client_secret()))
        .send()
        .map_err(|e| XurlError::auth_with_cause("TokenExchangeError", &e))?;

    let status = token_resp.status();
    let token_data: serde_json::Value = token_resp
        .json()
        .map_err(|e| XurlError::auth_with_cause("TokenExchangeError", &e))?;

    if !status.is_success() {
        let api_error = token_data["error"].as_str().unwrap_or("unknown");
        let api_desc = token_data["error_description"].as_str().unwrap_or("");
        return Err(XurlError::auth(format!(
            "TokenExchangeError: HTTP {status} — {api_error}: {api_desc}"
        )));
    }

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
    let username_str = if username.is_empty() {
        auth.fetch_username(&access_token)?
    } else {
        username.to_string()
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

/// Runs the full `OAuth2` PKCE authorization flow.
///
/// # Errors
///
/// Returns an error if the authorization URL is invalid, the callback server
/// fails, the token exchange fails, or the username cannot be resolved.
pub fn run_oauth2_flow(auth: &mut Auth, username: &str) -> Result<String> {
    // Generate state parameter
    let state_bytes: [u8; 32] = rand::random();
    let state = BASE64_STANDARD.encode(state_bytes);

    let (verifier, challenge) = generate_code_verifier_and_challenge();

    let auth_url_str = build_auth_url(auth, &state, &challenge)?;

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

    exchange_code_for_token(auth, &code, &verifier, username)
}

/// Runs step 1 of the remote `OAuth2` PKCE flow (headless machines).
///
/// Generates the PKCE verifier/challenge and state nonce, builds the
/// authorization URL, and persists the PKCE state to `pending_path` so
/// that a subsequent call to [`run_remote_step2`] can complete the exchange.
///
/// Returns the authorization URL that the user should open in a browser
/// on another machine.
///
/// # Errors
///
/// Returns an error if the authorization URL is invalid or the pending
/// state file cannot be written.
pub fn run_remote_step1(auth: &Auth, pending_path: &std::path::Path) -> Result<String> {
    if pending_path.exists() {
        eprintln!("Warning: Overwriting previous pending auth flow");
    }

    let state_bytes: [u8; 32] = rand::random();
    let state = BASE64_STANDARD.encode(state_bytes);
    let (verifier, challenge) = generate_code_verifier_and_challenge();

    let auth_url_str = build_auth_url(auth, &state, &challenge)?;

    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let pending_state = pending::PendingOAuth2State {
        code_verifier: verifier,
        state,
        client_id: auth.client_id().to_string(),
        app_name: auth.app_name().to_string(),
        created_at: now,
    };

    pending::save(&pending_state, pending_path)?;

    Ok(auth_url_str)
}

/// Runs step 2 of the remote `OAuth2` PKCE flow (headless machines).
///
/// Loads the pending PKCE state from `pending_path`, validates the state
/// and client ID, extracts the authorization code from `redirect_url`,
/// exchanges it for an access token, and saves the token to the store.
///
/// The pending state file is deleted only on success — on any error the
/// file is preserved so the user can retry.
///
/// # Errors
///
/// Returns an error if the pending state is missing/expired/invalid,
/// the client ID doesn't match, the state parameter doesn't match,
/// the redirect URL is missing the code, or the token exchange fails.
pub fn run_remote_step2(
    auth: &mut Auth,
    redirect_url: &str,
    username: &str,
    pending_path: &std::path::Path,
) -> Result<String> {
    let pending_state = pending::load(pending_path)?;

    // Validate client_id matches runtime context
    if pending_state.client_id != auth.client_id() {
        return Err(XurlError::auth(format!(
            "AppMismatch: pending state was created for app {:?} (client_id: {}), \
             but current context uses client_id: {}. Re-run step 1 with the correct --app",
            pending_state.app_name,
            pending_state.client_id,
            auth.client_id()
        )));
    }

    // Parse redirect URL to extract query parameters
    let parsed = Url::parse(redirect_url).map_err(|e| {
        XurlError::auth_with_cause("InvalidRedirectURL: failed to parse redirect URL", &e)
    })?;

    let params: std::collections::HashMap<String, String> = parsed
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    // Validate state first (CSRF check before revealing anything about code)
    let state = params.get("state").ok_or_else(|| {
        XurlError::auth("MissingState: no 'state' parameter found in redirect URL")
    })?;

    if *state != pending_state.state {
        return Err(XurlError::auth(
            "StateMismatch: the state parameter in the redirect URL does not match \
             the pending auth flow. This may indicate a CSRF attack or that step 1 \
             was re-run. Please start over with step 1",
        ));
    }

    // Extract authorization code
    let code = params.get("code").ok_or_else(|| {
        XurlError::auth(
            "MissingCode: no 'code' parameter found in redirect URL. \
             Make sure you copied the full URL from your browser's address bar",
        )
    })?;

    // Exchange code for token
    let access_token = exchange_code_for_token(auth, code, &pending_state.code_verifier, username)?;

    // Only delete on success
    pending::delete(pending_path)?;

    Ok(access_token)
}

/// Refreshes an `OAuth2` token if expired.
///
/// # Errors
///
/// Returns an error if no token is found, the refresh request fails, or the
/// username cannot be resolved after refresh.
pub fn refresh_oauth2_token(auth: &mut Auth, username: &str) -> Result<String> {
    let token = if username.is_empty() {
        auth.token_store.get_first_oauth2_token().cloned()
    } else {
        auth.token_store.get_oauth2_token(username).cloned()
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
    let username_str = if username.is_empty() {
        auth.fetch_username(&new_access_token)
            .map_err(|e| XurlError::auth_with_cause("UsernameFetchError", &e))?
    } else {
        username.to_string()
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
