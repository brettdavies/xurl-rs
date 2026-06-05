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
use crate::output::OutputConfig;

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
///   via [`crate::output::warn_stderr`] so the access token isn't discarded
///   along with the lookup failure.
///
/// # Errors
///
/// Returns an error if the token-exchange request fails or the response is
/// missing an access token. `fetch_username` failures no longer propagate.
pub(crate) fn exchange_code_for_token(
    auth: &mut Auth,
    code: &str,
    verifier: &str,
    username: &str,
) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(auth.http_timeout_secs()))
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

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expiration_time = now + expires_in;

    let app_name = auth.app_name().to_string();

    if username.is_empty() {
        match auth.fetch_username(&access_token) {
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
                crate::output::warn_stderr(
                    "token exchange succeeded but /2/users/me lookup failed; token stored under unnamed slot",
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
/// Browser-failure prompts are written to `out`/`stdout` via
/// `OutputConfig::print_message`, so library callers can capture them
/// instead of seeing real-process stdout pollution.
///
/// `browser_opener` is the injection point for the listener-before-browser
/// ordering test (KTD13). The production caller passes `open::that`; tests
/// pass a recording closure. The opener is invoked after the listener has
/// bound and entered its accept loop, guaranteeing the browser cannot reach
/// the callback URL before the socket is actively draining.
///
/// # Errors
///
/// Returns an error if the authorization URL is invalid, the callback server
/// fails, the token exchange fails, or the username cannot be resolved.
pub fn run_oauth2_flow(
    auth: &mut Auth,
    username: &str,
    out: &OutputConfig,
    stdout: &mut dyn std::io::Write,
    browser_opener: fn(&str) -> std::io::Result<()>,
) -> Result<String> {
    // Generate state parameter
    let state_bytes: [u8; 32] = rand::random();
    let state = BASE64_STANDARD.encode(state_bytes);

    let (verifier, challenge) = generate_code_verifier_and_challenge();

    let auth_url_str = build_auth_url(auth, &state, &challenge)?;

    // Parse the resolved redirect URI; the listener binds host, port, and path
    // from it (KTD6 + R25). Validation already accepted https or http+loopback
    // at U2/R8 write/resolve time, so a parse failure here is a programmer error.
    let redirect_parsed = Url::parse(auth.redirect_uri())
        .map_err(|e| XurlError::auth_with_cause("InvalidURL", &e))?;

    // Browser-open closure: runs on the listener's bind-success hook so the
    // socket is actively in `accept().await` before the URL is dispatched
    // (KTD5). Stdout messages on opener failure remain user-visible because
    // the closure can capture the OutputConfig; here we keep the closure
    // signature simple by returning io::Result and handling the message
    // OUTSIDE the closure via the recorded outcome.
    //
    // The shared cell carries the opener's io::Result back to the parent so
    // the failure-message print happens once the runtime exits — this avoids
    // holding `&mut dyn Write` across the runtime's lifetime.
    let opener_outcome = std::sync::Arc::new(std::sync::Mutex::new(None::<std::io::Result<()>>));
    let opener_outcome_for_closure = std::sync::Arc::clone(&opener_outcome);
    let auth_url_for_closure = auth_url_str.clone();
    let cancel = tokio_util::sync::CancellationToken::new();
    let cancel_for_closure = cancel.clone();
    let on_bound = move || {
        let result = browser_opener(&auth_url_for_closure);
        let failed = result.is_err();
        if let Ok(mut slot) = opener_outcome_for_closure.lock() {
            *slot = Some(result);
        }
        if failed {
            // Auto-open chose this code path; if open fails there is no
            // mechanism here for the user to paste the URL back. Cancel the
            // listener immediately rather than waiting out the 5-minute
            // CALLBACK_TIMEOUT. Callers wanting a paste flow use --no-browser
            // (cli/commands/auth.rs), which routes through remote_oauth2_step1.
            cancel_for_closure.cancel();
        }
    };

    let code_result = callback::wait_for_callback_with(&redirect_parsed, &state, cancel, on_bound);

    let opener_err = opener_outcome.lock().ok().and_then(|mut s| s.take());
    if let Some(Err(_e)) = &opener_err {
        out.print_message(
            stdout,
            "Failed to open browser automatically. Re-run with --no-browser \
             to use the paste-the-URL flow:",
        );
        out.print_message(stdout, &auth_url_str);
        return Err(XurlError::auth(
            "browser-open failed; re-run with --no-browser to paste the URL manually",
        ));
    }

    let code = code_result?;
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
        // Deferred per U4 / KTD6 plan task description. Plumbing a writer
        // through `remote_oauth2_step1` (called from `cli/commands/auth.rs`)
        // would require expanding the `Auth::remote_oauth2_step1` signature
        // for a single warning line. Library tests covering remote OAuth2
        // step 1 don't exercise the overwrite branch.
        crate::output::warn_stderr("overwriting previous pending auth flow");
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
///   via [`crate::output::warn_stderr`] (the function lacks an
///   `OutputConfig`; the persisted store state is the load-bearing observable).
///
/// # Errors
///
/// Returns an error when no cached token is found or the refresh-token POST
/// itself fails. `fetch_username` failures no longer propagate.
pub fn refresh_oauth2_token(auth: &mut Auth, username: &str) -> Result<String> {
    let app_name_lookup = auth.app_name().to_string();
    let token = if username.is_empty() {
        // Empty-caller precedence mirrors `Auth::get_oauth2_header` (KTD5):
        // default_user/arbitrary-first in the named map first, then the
        // unnamed (`/me`-failed salvage) slot. Both lookups MUST be
        // scoped to `app_name_lookup` so a `--app NAME` invocation reads
        // NAME's tokens, not whichever app happens to be the default. The
        // previous `get_first_oauth2_token()` call (no arg) defaulted the
        // lookup to the empty app name, which `resolve_app` resolved to
        // the default app, causing a freshly minted token in a named app
        // to be invisible to the refresh path. Fixed alongside the
        // `--app NAME` client_id regression (#51).
        auth.token_store
            .get_first_oauth2_token_for_app(&app_name_lookup)
            .cloned()
            .or_else(|| {
                auth.token_store
                    .get_oauth2_token_unnamed_for_app(&app_name_lookup)
                    .cloned()
            })
    } else {
        auth.token_store
            .get_oauth2_token_for_app(&app_name_lookup, username)
            .cloned()
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
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(auth.http_timeout_secs()))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new());
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

    let new_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expiration_time = new_now + expires_in;

    let app_name = auth.app_name().to_string();

    if username.is_empty() {
        match auth.fetch_username(&new_access_token) {
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
                crate::output::warn_stderr(
                    "refresh succeeded but /2/users/me lookup failed; token stored under unnamed slot",
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
