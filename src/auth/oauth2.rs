//! `OAuth2` PKCE flow and token refresh.
//!
//! Implements the browser-based `OAuth2` authorization code flow with PKCE
//! (Proof Key for Code Exchange) as used by the X API.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use url::Url;

use super::Auth;
use super::callback;
use super::pending;
use crate::error::{Error, Result};
use crate::store::OAuth2Token;
use tokio_util::sync::CancellationToken;

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
        Url::parse(auth.auth_url()).map_err(|e| Error::auth_with_cause("InvalidURL", &e))?;
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
/// parse response, resolve the storage key, compute expiration, and save to
/// the token store. The storage-key resolution mirrors
/// [`refresh_oauth2_token`]'s three-branch shape (KTD7):
///
/// - caller supplied non-empty `username` -> save under that username, skip
///   `fetch_username` entirely;
/// - caller supplied empty `username` and `fetch_username` succeeds -> save
///   under the discovered name;
/// - caller supplied empty `username` and `fetch_username` fails -> save into
///   the active app's unnamed (`/me`-failed salvage) slot via
///   [`crate::store::TokenStore::save_oauth2_token_unnamed_for_app`] and warn
///   with a `tracing` warning so the access token isn't discarded
///   along with the lookup failure.
///
/// # Errors
///
/// Returns an error if the token-exchange request fails or the response is
/// missing an access token. `fetch_username` failures no longer propagate.
pub(crate) async fn exchange_code_for_token(
    auth: &mut Auth,
    http: &reqwest::Client,
    code: &str,
    verifier: &str,
    username: &str,
) -> Result<String> {
    let token_resp = http
        .post(auth.token_url())
        .timeout(Duration::from_secs(auth.http_timeout_secs()))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", auth.redirect_uri()),
            ("client_id", auth.client_id()),
            ("code_verifier", verifier),
        ])
        .basic_auth(auth.client_id(), Some(auth.client_secret()))
        .send()
        .await
        .map_err(|e| Error::auth_with_cause("TokenExchangeError", &e))?;

    let status = token_resp.status();
    let token_data: serde_json::Value = token_resp
        .json()
        .await
        .map_err(|e| Error::auth_with_cause("TokenExchangeError", &e))?;

    if !status.is_success() {
        let api_error = token_data["error"].as_str().unwrap_or("unknown");
        let api_desc = token_data["error_description"].as_str().unwrap_or("");
        return Err(Error::auth(format!(
            "TokenExchangeError: HTTP {status} — {api_error}: {api_desc}"
        )));
    }

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| Error::auth("TokenExchangeError: no access_token in response"))?
        .to_string();

    let refresh_token = token_data["refresh_token"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let expires_in = token_data["expires_in"].as_u64().unwrap_or(7200);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expiration_time = now + expires_in;

    let app_name = auth.app_name().to_string();

    if username.is_empty() {
        match auth.fetch_username(http, &access_token).await {
            Ok(discovered) => {
                auth.token_store.save_oauth2_token_for_app(
                    &app_name,
                    &discovered,
                    &access_token,
                    &refresh_token,
                    expiration_time,
                )?;
            }
            Err(_) => {
                tracing::warn!(
                    target: "xurl::auth",
                    "token exchange succeeded but /2/users/me lookup failed; token stored under unnamed slot"
                );
                auth.token_store.save_oauth2_token_unnamed_for_app(
                    &app_name,
                    &access_token,
                    &refresh_token,
                    expiration_time,
                )?;
            }
        }
    } else {
        auth.token_store.save_oauth2_token_for_app(
            &app_name,
            username,
            &access_token,
            &refresh_token,
            expiration_time,
        )?;
    }

    // First-signed-in-app auto-default: if the user authenticated against a
    // named app and the previous default app holds no credentials, promote
    // the just-signed-in app to default so subsequent invocations resolve
    // it without an explicit `--app NAME`. No-op on subsequent sign-ins or
    // when the user has already configured a credentialed default.
    let _ = auth
        .token_store
        .promote_to_default_if_first_credentialed(&app_name)?;

    Ok(access_token)
}

/// Runs the full `OAuth2` PKCE authorization flow.
///
/// `browser_opener` receives the authorize URL after the callback listener
/// has bound and entered its accept loop, so a browser cannot reach the
/// callback before the socket is draining. The binary passes `open::that`;
/// tests pass a recording closure. When the opener fails the listener is
/// cancelled at once and the flow returns the browser-open error rather than
/// waiting out the callback timeout.
///
/// `cancel` is the caller's stop: the binary cancels it on a shutdown
/// signal, a library caller passes a child of its own token. The flow
/// registers no signal handler of its own.
///
/// # Errors
///
/// Returns an error if the authorization URL is invalid, the opener fails,
/// the callback server fails, or the token exchange fails.
pub async fn run_oauth2_flow<F>(
    auth: &mut Auth,
    http: &reqwest::Client,
    username: &str,
    cancel: CancellationToken,
    browser_opener: F,
) -> Result<String>
where
    F: Fn(&str) -> std::io::Result<()> + Send + Sync + 'static,
{
    // Generate state parameter
    let state_bytes: [u8; 32] = rand::random();
    let state = BASE64_STANDARD.encode(state_bytes);

    let (verifier, challenge) = generate_code_verifier_and_challenge();

    let auth_url_str = build_auth_url(auth, &state, &challenge)?;

    // Parse the resolved redirect URI; the listener binds host, port, and path
    // from it (KTD6 + R25). Validation already accepted https or http+loopback
    // at U2/R8 write/resolve time, so a parse failure here is a programmer error.
    let redirect_parsed =
        Url::parse(auth.redirect_uri()).map_err(|e| Error::auth_with_cause("InvalidURL", &e))?;

    // The opener runs on the listener's bind-success hook. A failed open
    // cancels the listener immediately rather than waiting out the callback
    // timeout, and the flag below turns that cancellation into the
    // browser-open error the caller can act on.
    let opener_failed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let opener_failed_for_closure = std::sync::Arc::clone(&opener_failed);
    let cancel_for_closure = cancel.clone();
    let on_bound = move || {
        if browser_opener(&auth_url_str).is_err() {
            opener_failed_for_closure.store(true, std::sync::atomic::Ordering::SeqCst);
            cancel_for_closure.cancel();
        }
    };

    let code_result =
        callback::wait_for_callback_with(&redirect_parsed, &state, cancel, on_bound).await;

    if opener_failed.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(Error::auth(
            "browser-open failed; re-run with --no-browser to paste the URL manually",
        ));
    }

    let code = code_result?;
    exchange_code_for_token(auth, http, &code, &verifier, username).await
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
        tracing::warn!(target: "xurl::auth", "overwriting previous pending auth flow");
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
pub async fn run_remote_step2(
    auth: &mut Auth,
    http: &reqwest::Client,
    redirect_url: &str,
    username: &str,
    pending_path: &std::path::Path,
) -> Result<String> {
    let pending_state = pending::load(pending_path)?;

    // Validate client_id matches runtime context
    if pending_state.client_id != auth.client_id() {
        return Err(Error::auth(format!(
            "AppMismatch: pending state was created for app {:?} (client_id: {}), \
             but current context uses client_id: {}. Re-run step 1 with the correct --app",
            pending_state.app_name,
            pending_state.client_id,
            auth.client_id()
        )));
    }

    // Parse redirect URL to extract query parameters
    let parsed = Url::parse(redirect_url).map_err(|e| {
        Error::auth_with_cause("InvalidRedirectURL: failed to parse redirect URL", &e)
    })?;

    let params: std::collections::HashMap<String, String> = parsed
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    // Validate state first (CSRF check before revealing anything about code)
    let state = params
        .get("state")
        .ok_or_else(|| Error::auth("MissingState: no 'state' parameter found in redirect URL"))?;

    if *state != pending_state.state {
        return Err(Error::auth(
            "StateMismatch: the state parameter in the redirect URL does not match \
             the pending auth flow. This may indicate a CSRF attack or that step 1 \
             was re-run. Please start over with step 1",
        ));
    }

    // Extract authorization code
    let code = params.get("code").ok_or_else(|| {
        Error::auth(
            "MissingCode: no 'code' parameter found in redirect URL. \
             Make sure you copied the full URL from your browser's address bar",
        )
    })?;

    // Exchange code for token
    let access_token =
        exchange_code_for_token(auth, http, code, &pending_state.code_verifier, username).await?;

    // Only delete on success
    pending::delete(pending_path)?;

    Ok(access_token)
}

/// The stored OAuth2 token a request for `username` would use, if any.
///
/// An empty `username` follows the empty-caller precedence of
/// [`Auth::get_oauth2_header`]: `default_user` or the first named token in
/// the active app, then the unnamed (`/me`-failed salvage) slot. A named
/// caller reads its own entry in the active app. Both lookups are scoped to
/// the active app, so a `--app NAME` invocation reads NAME's tokens rather
/// than whichever app happens to be the default.
pub(crate) fn stored_oauth2_token(auth: &Auth, username: &str) -> Option<OAuth2Token> {
    let app_name = auth.app_name().to_string();
    let token = if username.is_empty() {
        auth.token_store
            .get_first_oauth2_token_for_app(&app_name)
            .or_else(|| auth.token_store.get_oauth2_token_unnamed_for_app(&app_name))
    } else {
        auth.token_store
            .get_oauth2_token_for_app(&app_name, username)
    };
    token.and_then(|t| t.oauth2.clone())
}

/// Whether the stored expiry has passed.
pub(crate) fn is_expired(token: &OAuth2Token) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    now >= token.expiration_time
}

/// Refreshes an `OAuth2` token if expired.
///
/// The refresh-token POST result is the sole source of truth for "is the
/// refresh successful" (`KTD2`). The refreshed access token is persisted in
/// all three success branches:
///
/// - caller supplied non-empty `username` -> save under that username, skip
///   `fetch_username` entirely;
/// - caller supplied empty `username` and `fetch_username` succeeds -> save
///   under the discovered name;
/// - caller supplied empty `username` and `fetch_username` fails -> save into
///   the active app's unnamed (`/me`-failed salvage) slot via
///   [`crate::store::TokenStore::save_oauth2_token_unnamed_for_app`] and warn
///   with a `tracing` warning (the persisted store state is the load-bearing
///   observable).
///
/// # Errors
///
/// Returns an error when no cached token is found or the refresh-token POST
/// itself fails. `fetch_username` failures no longer propagate.
pub async fn refresh_oauth2_token(
    auth: &mut Auth,
    http: &reqwest::Client,
    username: &str,
) -> Result<String> {
    let oauth2 = stored_oauth2_token(auth, username)
        .ok_or_else(|| Error::auth(crate::error::NO_OAUTH2_TOKEN))?;

    // Token is still valid
    if !is_expired(&oauth2) {
        return Ok(oauth2.access_token.clone());
    }

    // Token is expired, refresh it
    let token_resp = http
        .post(auth.token_url())
        .timeout(Duration::from_secs(auth.http_timeout_secs()))
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", &oauth2.refresh_token),
            ("client_id", auth.client_id()),
        ])
        .basic_auth(auth.client_id(), Some(auth.client_secret()))
        .send()
        .await
        .map_err(|e| Error::auth_with_cause("RefreshTokenError", &e))?;

    let token_data: serde_json::Value = token_resp
        .json()
        .await
        .map_err(|e| Error::auth_with_cause("RefreshTokenError", &e))?;

    let new_access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| Error::auth("RefreshTokenError: no access_token in response"))?
        .to_string();

    let new_refresh_token = token_data["refresh_token"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let expires_in = token_data["expires_in"].as_u64().unwrap_or(7200);

    let new_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expiration_time = new_now + expires_in;

    let app_name = auth.app_name().to_string();

    if username.is_empty() {
        match auth.fetch_username(http, &new_access_token).await {
            Ok(discovered) => {
                auth.token_store.save_oauth2_token_for_app(
                    &app_name,
                    &discovered,
                    &new_access_token,
                    &new_refresh_token,
                    expiration_time,
                )?;
            }
            Err(_) => {
                tracing::warn!(
                    target: "xurl::auth",
                    "refresh succeeded but /2/users/me lookup failed; token stored under unnamed slot"
                );
                auth.token_store.save_oauth2_token_unnamed_for_app(
                    &app_name,
                    &new_access_token,
                    &new_refresh_token,
                    expiration_time,
                )?;
            }
        }
    } else {
        auth.token_store.save_oauth2_token_for_app(
            &app_name,
            username,
            &new_access_token,
            &new_refresh_token,
            expiration_time,
        )?;
    }

    Ok(new_access_token)
}
