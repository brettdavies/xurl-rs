//! Authentication orchestration — `OAuth2` PKCE, `OAuth1` HMAC-SHA1, Bearer.
//!
//! Mirrors the Go `auth.Auth` struct. Credentials are resolved in order:
//! env-var config -> active app in `.xurl` store.

pub mod callback;
pub mod oauth1;
pub mod oauth2;
pub mod pending;

use crate::config::Config;
use crate::error::{Result, XurlError};
use crate::output::OutputConfig;
use crate::store::TokenStore;

/// Manages authentication for X API requests.
#[allow(clippy::struct_field_names)]
pub struct Auth {
    pub token_store: TokenStore,
    /// Owned application configuration. The redirect URI here is the
    /// resolver output (env > app-stored > built-in default), written by
    /// [`Auth::new_with_store_path`] and re-resolved by
    /// [`Auth::with_app_name`]. This is the single source of truth per
    /// KTD2 — no parallel `redirect_uri` field lives on `Auth`.
    config: Config,
    client_id: String,
    client_secret: String,
    /// `true` iff [`Self::client_id`] was supplied via the `CLIENT_ID` env
    /// var at construction. Preserved across [`Self::with_app_name`] switches
    /// so env precedence holds even after the active app changes. Without
    /// this flag, the older "preserve if non-empty" check could not
    /// distinguish env-supplied values from values copied off the previous
    /// app's store entry, so subsequent `--app NAME` switches silently
    /// re-used the previous app's stored client_id.
    client_id_from_env: bool,
    /// Counterpart to [`Self::client_id_from_env`] for client_secret.
    client_secret_from_env: bool,
    app_name: String,
}

impl Auth {
    /// Creates a new `Auth` object using the legacy `~/.xurl` token-store path.
    ///
    /// Shim over [`Auth::new_with_store_path`] resolving to
    /// [`Config::default_store_path`]. Credentials are resolved: env vars -> active app.
    #[must_use]
    pub fn new(cfg: &Config) -> Self {
        Self::new_with_store_path(cfg, &Config::default_store_path())
    }

    /// Creates a new `Auth` object backed by an explicit token-store path.
    ///
    /// The canonical constructor. Tests pass a `TempDir`-rooted path to avoid
    /// touching the real `~/.xurl`; the binary calls [`Auth::new`] which
    /// resolves to [`Config::default_store_path`]. Credentials are resolved:
    /// env vars -> active app at `store_path`.
    ///
    /// Runs the three-level redirect URI resolver (env > app-stored >
    /// built-in default) against the constructed token store and writes the
    /// result back into the owned `Config`, satisfying KTD2's single source
    /// of truth invariant.
    #[must_use]
    pub fn new_with_store_path(cfg: &Config, store_path: &std::path::Path) -> Self {
        let path_str = store_path.to_str().unwrap_or(".");
        let ts =
            TokenStore::new_with_credentials_and_path(&cfg.client_id, &cfg.client_secret, path_str);

        // Env-origin is recorded BEFORE falling back to the store so a later
        // `with_app_name` switch can correctly preserve env-supplied values
        // and re-resolve store-derived ones from the new app's entry.
        let client_id_from_env = !cfg.client_id.is_empty();
        let client_secret_from_env = !cfg.client_secret.is_empty();

        let mut client_id = cfg.client_id.clone();
        let mut client_secret = cfg.client_secret.clone();
        let app_name = cfg.app_name.clone();

        let app = ts.resolve_app(&app_name);
        if !client_id_from_env {
            client_id.clone_from(&app.client_id);
        }
        if !client_secret_from_env {
            client_secret.clone_from(&app.client_secret);
        }

        let mut config = cfg.clone();
        let resolved = crate::config::resolve_redirect_uri_from(
            std::env::var("REDIRECT_URI").ok(),
            ts.get_app_redirect_uri(&app_name),
        );
        config.redirect_uri = resolved.uri;
        config.redirect_uri_source = resolved.source;
        config.redirect_uri_from_env = resolved.source.is_env_var();

        Self {
            token_store: ts,
            config,
            client_id,
            client_secret,
            client_id_from_env,
            client_secret_from_env,
            app_name,
        }
    }

    /// Sets the explicit app name override and re-resolves the redirect URI.
    ///
    /// Credentials honor env precedence per the internal
    /// `client_id_from_env` / `client_secret_from_env` flags:
    /// env-supplied values survive the switch; store-derived values
    /// get re-resolved from the new app's
    /// entry, even when the previous app's stored value was non-empty.
    /// The redirect URI is always re-resolved (env-precedence is enforced
    /// inside the resolver itself, so re-running unconditionally produces
    /// the right value per KTD3).
    pub fn with_app_name(&mut self, app_name: &str) {
        self.app_name = app_name.to_string();
        let app = self.token_store.resolve_app(app_name);
        if !self.client_id_from_env {
            self.client_id = app.client_id.clone();
        }
        if !self.client_secret_from_env {
            self.client_secret = app.client_secret.clone();
        }

        let resolved = crate::config::resolve_redirect_uri_from(
            std::env::var("REDIRECT_URI").ok(),
            self.token_store.get_app_redirect_uri(app_name),
        );
        self.config.redirect_uri = resolved.uri;
        self.config.redirect_uri_source = resolved.source;
        self.config.redirect_uri_from_env = resolved.source.is_env_var();
    }

    /// Gets the `OAuth1` Authorization header for a request.
    ///
    /// # Errors
    ///
    /// Returns an error if no `OAuth1` token is found or signature generation fails.
    pub fn get_oauth1_header(
        &self,
        method: &str,
        url_str: &str,
        additional_params: Option<&std::collections::BTreeMap<String, String>>,
    ) -> Result<String> {
        // Multi-app resolution: read the OAuth1 token from the active app
        // (set by `with_app_name` from the `--app NAME` CLI flag) rather
        // than from the legacy no-arg `get_oauth1_tokens()` which falls
        // back to the default app. Without this scoping, a token saved
        // under NAME via `xr auth oauth1 --app NAME` would be invisible
        // to subsequent `--app NAME --auth oauth1` invocations.
        let token = self
            .token_store
            .get_oauth1_tokens_for_app(&self.app_name)
            .ok_or_else(|| XurlError::auth("TokenNotFound: OAuth1 token not found"))?;

        let oauth1_token = token
            .oauth1
            .as_ref()
            .ok_or_else(|| XurlError::auth("TokenNotFound: OAuth1 token not found"))?;

        oauth1::build_oauth1_header(method, url_str, oauth1_token, additional_params)
    }

    /// Gets or refreshes an `OAuth2` token and returns the Authorization header.
    ///
    /// Lookup precedence is intent-split per `KTD5`:
    ///
    /// - non-empty `username` (named caller): try the username's own token
    ///   first; on miss, fall through to
    ///   [`TokenStore::get_first_oauth2_token_for_app`] (which itself prefers
    ///   `default_user`, then arbitrary-first); on total miss, trigger the
    ///   full OAuth2 flow. The unnamed (`/me`-failed salvage) slot is
    ///   **never** consulted on this branch — a caller who supplies a
    ///   username has explicit identity intent and must not silently receive
    ///   a salvage-state token under their name.
    /// - empty `username`: try `get_first_oauth2_token_for_app` (which
    ///   prefers `default_user`, then arbitrary-first); on miss, fall through
    ///   to the unnamed slot via
    ///   [`TokenStore::get_oauth2_token_unnamed_for_app`]; on total miss,
    ///   trigger the OAuth2 flow.
    ///
    /// In both branches the cached-token path delegates to
    /// [`Auth::refresh_oauth2_token`], which is a no-op if the token is still
    /// valid.
    ///
    /// # Errors
    ///
    /// Returns an error if the `OAuth2` flow fails or token refresh fails.
    pub fn get_oauth2_header(&mut self, username: &str) -> Result<String> {
        let app_name = self.app_name.clone();

        if username.is_empty() {
            // Empty-caller precedence: default_user (via get_first) -> unnamed -> flow.
            if self
                .token_store
                .get_first_oauth2_token_for_app(&app_name)
                .is_some()
            {
                let access_token = self.refresh_oauth2_token(username)?;
                return Ok(format!("Bearer {access_token}"));
            }

            if self
                .token_store
                .get_oauth2_token_unnamed_for_app(&app_name)
                .is_some()
            {
                // Empty-caller + unnamed-hit: refresh delegates with empty username;
                // if /me fails again the refresh path writes back to the unnamed slot.
                let access_token = self.refresh_oauth2_token(username)?;
                return Ok(format!("Bearer {access_token}"));
            }
        } else {
            // Named-caller precedence: own token -> get_first fallback -> flow.
            // The unnamed slot is never consulted here.
            if self.token_store.get_oauth2_token(username).is_some()
                || self
                    .token_store
                    .get_first_oauth2_token_for_app(&app_name)
                    .is_some()
            {
                let access_token = self.refresh_oauth2_token(username)?;
                return Ok(format!("Bearer {access_token}"));
            }
        }

        // Deferred per U4 / KTD6: this site is the mid-request token-refresh
        // path inside `ApiClient::send_request`. Plumbing writers from the
        // runner all the way through `ApiClient::send_request` →
        // `get_auth_header` → `get_oauth2_header` would expand 6 method
        // signatures (send_request, send_multipart_request, stream_request,
        // get_auth_header, get_auth_header_public, get_oauth2_header) and
        // every call site in `cli/commands/` and `api/media.rs` for a path
        // that runs only on the first request when no token is cached.
        //
        // The vast majority of OAuth2 flows run at explicit `xr auth oauth2`
        // time (which U4 wires correctly via the runner's writers). This
        // stub is the only remaining direct-stdio site after U4.
        let out = OutputConfig::new(
            crate::output::OutputFormat::Text,
            false,
            false,
            crate::cli::ColorChoice::Auto,
        );
        let mut stdout = std::io::stdout();
        let access_token = self.oauth2_flow(username, &out, &mut stdout)?;
        Ok(format!("Bearer {access_token}"))
    }

    /// Starts the `OAuth2` PKCE flow with the default `open::that` browser opener.
    ///
    /// Browser-failure prompts are written to `stdout` via `out`'s
    /// `print_message`, so callers can capture them in tests. Library
    /// consumers needing a custom opener (recording / headless / tests for
    /// the listener-before-browser ordering) call
    /// [`oauth2::run_oauth2_flow`] directly with their own `fn(&str) ->
    /// io::Result<()>` opener.
    ///
    /// # Errors
    ///
    /// Returns an error if the authorization flow, token exchange, or username
    /// resolution fails.
    pub fn oauth2_flow(
        &mut self,
        username: &str,
        out: &OutputConfig,
        stdout: &mut dyn std::io::Write,
    ) -> Result<String> {
        oauth2::run_oauth2_flow(self, username, out, stdout, |url| open::that(url))
    }

    /// Validates and refreshes an `OAuth2` token if needed.
    ///
    /// # Errors
    ///
    /// Returns an error if no token is found or the refresh request fails.
    pub fn refresh_oauth2_token(&mut self, username: &str) -> Result<String> {
        oauth2::refresh_oauth2_token(self, username)
    }

    /// Runs step 1 of the remote `OAuth2` PKCE flow.
    ///
    /// Returns the authorization URL that the user should open in a browser
    /// on another machine.
    ///
    /// # Errors
    ///
    /// Returns an error if the authorization URL is invalid or the pending
    /// state file cannot be written.
    pub fn remote_oauth2_step1(&self, pending_path: &std::path::Path) -> Result<String> {
        oauth2::run_remote_step1(self, pending_path)
    }

    /// Runs step 2 of the remote `OAuth2` PKCE flow.
    ///
    /// Takes the redirect URL from the browser, extracts the authorization
    /// code, exchanges it for an access token, and saves the token.
    ///
    /// # Errors
    ///
    /// Returns an error if the pending state is missing/expired/invalid,
    /// the token exchange fails, or the username cannot be resolved.
    pub fn remote_oauth2_step2(
        &mut self,
        redirect_url: &str,
        username: &str,
        pending_path: &std::path::Path,
    ) -> Result<String> {
        oauth2::run_remote_step2(self, redirect_url, username, pending_path)
    }

    /// Gets the bearer token Authorization header.
    ///
    /// Resolution order: `XURL_BEARER_TOKEN` env var first (one-shot agent
    /// flows that pipe a secret without persisting it to disk), then the
    /// token store entry on the resolved app. Empty env values fall through
    /// to the store. The env var matches the precedence shape used by every
    /// other agentic flag (`XURL_VERBOSE`, `XURL_OUTPUT`, `XURL_NO_BROWSER`,
    /// etc.) and pairs with `XURL_OUTPUT=json xr ... --auth app` for stateless
    /// container invocations.
    ///
    /// Production code reads `XURL_BEARER_TOKEN` from the process environment.
    /// The resolution logic is factored into [`resolve_bearer_token`] so unit
    /// tests can exercise every precedence path without mutating the global
    /// environment (the project policy in MEMORY: no env mutation in tests,
    /// no `#[serial]`).
    ///
    /// # Errors
    ///
    /// Returns an error if `XURL_BEARER_TOKEN` is unset (or empty) AND no
    /// bearer token is stored for the resolved app.
    pub fn get_bearer_token_header(&self) -> Result<String> {
        resolve_bearer_token(
            std::env::var("XURL_BEARER_TOKEN").ok(),
            &self.token_store,
            &self.app_name,
        )
    }

    /// Fetches the username for an access token from the /2/users/me endpoint.
    pub(crate) fn fetch_username(&self, access_token: &str) -> Result<String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(
                self.config.http_timeout_secs,
            ))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        let resp = client
            .get(&self.config.info_url)
            .header("Authorization", format!("Bearer {access_token}"))
            .send()
            .map_err(|e| XurlError::auth_with_cause("NetworkError", &e))?;

        let body: serde_json::Value = resp
            .json()
            .map_err(|e| XurlError::auth_with_cause("JSONDeserializationError", &e))?;

        body.get("data")
            .and_then(|d| d.get("username"))
            .and_then(|u| u.as_str())
            .map(std::string::ToString::to_string)
            .ok_or_else(|| {
                XurlError::auth("UsernameNotFound: username not found when fetching username")
            })
    }

    /// Replaces the token store (used in integration tests).
    ///
    /// Honors the internal env-precedence flags
    /// (`client_id_from_env` / `client_secret_from_env`):
    /// env-supplied values survive the swap; store-derived values get
    /// re-resolved from the new store's
    /// active app. Replaces the older equality-based heuristic
    /// (`self.client_id == old_app.client_id`) which produced false
    /// negatives when the old and new stores happened to share a value.
    #[allow(dead_code)] // Public library API — used by consumers and integration tests
    #[must_use]
    pub fn with_token_store(mut self, token_store: TokenStore) -> Self {
        let new_app = token_store.resolve_app(&self.app_name);
        if !self.client_id_from_env {
            self.client_id = new_app.client_id.clone();
        }
        if !self.client_secret_from_env {
            self.client_secret = new_app.client_secret.clone();
        }
        self.token_store = token_store;
        self
    }

    /// Returns a reference to the token store.
    #[allow(dead_code)] // Public library API — used by consumers and integration tests
    #[must_use]
    pub fn token_store(&self) -> &TokenStore {
        &self.token_store
    }

    // Accessors
    #[must_use]
    pub fn app_name(&self) -> &str {
        &self.app_name
    }
    #[must_use]
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
    #[must_use]
    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }
    #[must_use]
    pub fn auth_url(&self) -> &str {
        &self.config.auth_url
    }
    #[must_use]
    pub fn token_url(&self) -> &str {
        &self.config.token_url
    }
    #[must_use]
    pub fn redirect_uri(&self) -> &str {
        &self.config.redirect_uri
    }
    /// Per-request HTTP timeout in seconds for OAuth2 token exchange,
    /// refresh, and `/2/users/me` lookups. Mirrors the `--timeout` flag.
    #[must_use]
    pub fn http_timeout_secs(&self) -> u64 {
        self.config.http_timeout_secs
    }
}

/// Pure resolver for the bearer Authorization header.
///
/// Encapsulates the precedence between an env-supplied bearer (typically
/// `XURL_BEARER_TOKEN`) and the active app's stored bearer in the token
/// store. Factored out of [`Auth::get_bearer_token_header`] so unit tests
/// can exercise every branch (env-only, store-only, env-overrides-store,
/// env-empty-falls-through, neither-set) without touching the process
/// environment, per the test-isolation policy.
///
/// `env_token` is `None` when the env var is unset, `Some("")` when it is set
/// to the empty string (which falls through to the store), and `Some(value)`
/// when it carries a non-empty token that wins.
///
/// `app_name` scopes the store lookup so multi-app callers reading a bearer
/// via `xr <cmd> --app NAME --auth app` correctly target NAME's stored
/// bearer rather than the default app's. Pass an empty string to fall back
/// to the default app (the legacy no-arg behavior).
///
/// # Errors
///
/// Returns an error if `env_token` is `None` or `Some("")` AND no bearer
/// token is stored for the resolved app.
pub fn resolve_bearer_token(
    env_token: Option<String>,
    store: &TokenStore,
    app_name: &str,
) -> Result<String> {
    if let Some(token) = env_token
        && !token.is_empty()
    {
        return Ok(format!("Bearer {token}"));
    }

    let token = store
        .get_bearer_token_for_app(app_name)
        .ok_or_else(|| XurlError::auth("TokenNotFound: bearer token not found"))?;

    let bearer = token
        .bearer
        .as_ref()
        .ok_or_else(|| XurlError::auth("TokenNotFound: bearer token not found"))?;

    Ok(format!("Bearer {bearer}"))
}

// Compile-time guarantee: `Auth` is `Send + Sync` so it can be shared across
// threads in the planned async/concurrent `ApiClient` (see plan KTD8). Any
// future change that introduces a `!Send` or `!Sync` field will fail this
// assertion at compile time.
const _: fn() = || {
    fn _assert_send_sync<T: Send + Sync>() {}
    _assert_send_sync::<Auth>();
};
