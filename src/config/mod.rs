//! Application configuration resolved from environment variables.
//!
//! `Config` mirrors the Go `config.Config` struct. All fields come from
//! env vars with sensible defaults for the X API; the redirect URI is
//! resolved per app at construction.

use std::path::Path;

use serde::Serialize;
use url::Url;

use crate::error::XurlError;

/// Application configuration resolved from environment variables.
///
/// Mirrors the Go `config.Config` struct — all fields come from env vars
/// with sensible defaults for the X API.
///
/// Holds the application configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// `OAuth2` client ID (may come from env or the active app in `.xurl`).
    pub client_id: String,
    /// `OAuth2` client secret.
    pub client_secret: String,
    /// `OAuth2` PKCE redirect URI.
    pub redirect_uri: String,
    /// `OAuth2` authorization URL.
    pub auth_url: String,
    /// `OAuth2` token exchange URL.
    pub token_url: String,
    /// API base URL.
    pub api_base_url: String,
    /// User info endpoint URL.
    pub info_url: String,
    /// Explicit `--app` override; empty means "use default".
    pub app_name: String,
    /// Precedence level that produced the current [`Self::redirect_uri`].
    ///
    /// `Config::new()` cannot consult any token store, so it only ever
    /// emits [`ResolveSource::EnvVar`] or [`ResolveSource::BuiltInDefault`].
    /// [`Auth::new_with_store_path`](crate::auth::Auth::new_with_store_path)
    /// overwrites this with the full three-level resolution.
    pub(crate) redirect_uri_source: ResolveSource,
    /// Convenience predicate mirroring `redirect_uri_source.is_env_var()`.
    ///
    /// Stored separately to keep `Auth`-consuming hot paths free of the
    /// `match` on the enum variant.
    pub(crate) redirect_uri_from_env: bool,
    /// Per-request HTTP timeout in seconds for all reqwest-backed paths
    /// (API client, OAuth2 token exchange/refresh, `fetch_username`).
    ///
    /// Sourced from `--timeout` / `XURL_TIMEOUT` via the CLI runner;
    /// `Config::new()` defaults to [`crate::api::DEFAULT_TIMEOUT_SECS`].
    pub http_timeout_secs: u64,
}

/// Built-in default `OAuth2` redirect URI used when neither the
/// `REDIRECT_URI` env var nor a stored per-app value is set.
pub const DEFAULT_REDIRECT_URI: &str = "http://localhost:8080/callback";

/// The values `xurl` reads from the process environment, as data.
///
/// [`EnvOverrides::from_env`] is the single place in the crate that reads
/// these variables; every other layer receives an already-resolved value.
/// A caller that embeds the library — or a test that must stay isolated from
/// whatever else the process is doing — builds this directly and passes it to
/// [`Config::from_overrides`] or to
/// [`run_with_overrides`](crate::cli::runner::run_with_overrides).
///
/// `None` means the variable was unset, which is distinct from `Some(String::new())`
/// for `redirect_uri`: an unset value falls through to the next precedence
/// level, while a set-but-empty one is an env-sourced value that fails
/// validation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvOverrides {
    /// `CLIENT_ID` — `OAuth2` client ID.
    pub client_id: Option<String>,
    /// `CLIENT_SECRET` — `OAuth2` client secret.
    pub client_secret: Option<String>,
    /// `REDIRECT_URI` — `OAuth2` PKCE redirect URI, top of the three-level precedence.
    pub redirect_uri: Option<String>,
    /// `AUTH_URL` — `OAuth2` authorization endpoint.
    pub auth_url: Option<String>,
    /// `TOKEN_URL` — `OAuth2` token exchange endpoint.
    pub token_url: Option<String>,
    /// `API_BASE_URL` — API origin every request is built against.
    pub api_base_url: Option<String>,
    /// `INFO_URL` — user-info endpoint; derived from `api_base_url` when unset.
    pub info_url: Option<String>,
    /// `XURL_BEARER_TOKEN` — app-only bearer token, top of the bearer precedence.
    ///
    /// Consumed by [`Auth::get_bearer_token_header`](crate::auth::Auth::get_bearer_token_header)
    /// rather than by [`Config`], so it has no corresponding `Config` field.
    pub bearer_token: Option<String>,
    /// `XURL_OUTPUT` — output format, read before clap parsing so a parse
    /// error can pick its envelope shape.
    ///
    /// Consumed by the CLI runner rather than by [`Config`].
    pub output: Option<String>,
}

impl EnvOverrides {
    /// Reads every supported variable from the process environment.
    ///
    /// The binary calls this once per run. Nothing else in the crate reads
    /// these variables, so a caller that supplies its own `EnvOverrides` is
    /// unaffected by the process environment.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            client_id: std::env::var("CLIENT_ID").ok(),
            client_secret: std::env::var("CLIENT_SECRET").ok(),
            redirect_uri: std::env::var("REDIRECT_URI").ok(),
            auth_url: std::env::var("AUTH_URL").ok(),
            token_url: std::env::var("TOKEN_URL").ok(),
            api_base_url: std::env::var("API_BASE_URL").ok(),
            info_url: std::env::var("INFO_URL").ok(),
            bearer_token: std::env::var("XURL_BEARER_TOKEN").ok(),
            output: std::env::var("XURL_OUTPUT").ok(),
        }
    }
}

impl Config {
    /// Creates a new `Config` from environment variables, falling back to defaults.
    ///
    /// Reads the process environment through [`EnvOverrides::from_env`] and
    /// delegates to [`Config::from_overrides`]; the two paths are equivalent
    /// for identical inputs.
    ///
    /// `redirect_uri` resolution is env-only here: `REDIRECT_URI` if set,
    /// otherwise [`DEFAULT_REDIRECT_URI`]. The token-store-aware three-level
    /// precedence (env > app-stored > default) is run by
    /// [`Auth::new_with_store_path`](crate::auth::Auth::new_with_store_path),
    /// which overwrites `redirect_uri`, `redirect_uri_source`, and
    /// `redirect_uri_from_env` on the owned `Config`. The R12 audit confirms
    /// no consumer reads `redirect_uri` from a pre-resolution `Config`.
    #[must_use]
    pub fn new() -> Self {
        Self::from_overrides(&EnvOverrides::from_env())
    }

    /// Creates a `Config` from explicitly supplied values, reading no environment.
    ///
    /// An absent field takes the same built-in default `Config::new()` applies
    /// when its variable is unset. An absent `info_url` derives from the
    /// resolved `api_base_url`, so overriding only the base URL keeps the
    /// user-info endpoint pointed at the same origin.
    ///
    /// A present `redirect_uri` records [`ResolveSource::EnvVar`] provenance,
    /// matching an exported `REDIRECT_URI`; an absent one records the built-in
    /// default. That distinction decides whether a stored per-app value can win
    /// during the three-level resolution, so supplying a value here is not the
    /// same as leaving it out.
    #[must_use]
    pub fn from_overrides(overrides: &EnvOverrides) -> Self {
        let redirect_uri_from_env = overrides.redirect_uri.is_some();
        let redirect_uri = overrides
            .redirect_uri
            .clone()
            .unwrap_or_else(|| DEFAULT_REDIRECT_URI.to_string());
        let redirect_uri_source = if redirect_uri_from_env {
            ResolveSource::EnvVar
        } else {
            ResolveSource::BuiltInDefault
        };
        let api_base_url = overrides
            .api_base_url
            .clone()
            .unwrap_or_else(|| "https://api.x.com".to_string());
        let info_url = overrides
            .info_url
            .clone()
            .unwrap_or_else(|| format!("{api_base_url}/2/users/me"));

        Self {
            client_id: overrides.client_id.clone().unwrap_or_default(),
            client_secret: overrides.client_secret.clone().unwrap_or_default(),
            redirect_uri,
            auth_url: overrides
                .auth_url
                .clone()
                .unwrap_or_else(|| "https://x.com/i/oauth2/authorize".to_string()),
            token_url: overrides
                .token_url
                .clone()
                .unwrap_or_else(|| "https://api.x.com/2/oauth2/token".to_string()),
            api_base_url,
            info_url,
            app_name: String::new(),
            redirect_uri_source,
            redirect_uri_from_env,
            http_timeout_secs: crate::api::DEFAULT_TIMEOUT_SECS,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Returns the legacy default token-store path: `~/.xurl`.
    ///
    /// Falls back to `./.xurl` when the home directory cannot be resolved.
    /// This is the canonical legacy path resolver — the binary uses it; tests
    /// pass explicit tempdir paths to `Auth::new_with_store_path` instead.
    #[must_use]
    pub fn default_store_path() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".xurl")
    }

    /// Validates an `OAuth2` redirect URI.
    ///
    /// Enforces the project's https-or-loopback policy: accept any `https`
    /// URL, or `http` only when the host is one of `localhost`, `127.0.0.1`,
    /// or `::1`. All other schemes (including `ftp`, `file`) and `http`
    /// against a non-loopback host are rejected.
    ///
    /// Returns the parsed [`Url`] on success so callers that already need it
    /// (e.g., the listener bind logic) can avoid a second parse.
    ///
    /// # Errors
    ///
    /// Returns [`XurlError::Validation`] when parsing fails or the URI does
    /// not satisfy the https-or-loopback rule.
    pub fn validate_redirect_uri(uri: &str) -> crate::error::Result<Url> {
        let parsed = Url::parse(uri)
            .map_err(|e| XurlError::validation(format!("invalid redirect URI: {e}")))?;

        let scheme = parsed.scheme();
        if scheme == "https" {
            return Ok(parsed);
        }

        if scheme == "http"
            && let Some(host) = parsed.host_str()
            && matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]")
        {
            return Ok(parsed);
        }

        Err(XurlError::validation(format!(
            "redirect URI must be https, or http on loopback (localhost / 127.0.0.1 / [::1]); got: {uri}"
        )))
    }
}

// ── Resolver ─────────────────────────────────────────────────────────────────

/// Origin of a resolved redirect URI.
///
/// Mirrors the upstream Go xurl labels at `config/config.go:62-72`.
/// The `#[serde(rename_all = "kebab-case")]` directive produces
/// `"env-var"`, `"app-config"`, and `"built-in-default"` in JSON output
/// (the machine-readable shape consumed by `--output json`); the
/// human-readable text rendering uses [`ResolveSource::as_text_label`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ResolveSource {
    /// Resolved from the `REDIRECT_URI` environment variable.
    EnvVar,
    /// Resolved from the per-app value stored in `~/.xurl`.
    AppConfig,
    /// Fell through to [`DEFAULT_REDIRECT_URI`].
    BuiltInDefault,
}

#[allow(dead_code)] // Consumed by U3 (Auth integration) and U5 (status rendering)
impl ResolveSource {
    /// Returns the upstream-verbatim human label for this source.
    ///
    /// Used by `auth status` and `auth apps redirect-uri get` text mode.
    pub(crate) fn as_text_label(&self) -> &'static str {
        match self {
            Self::EnvVar => "REDIRECT_URI environment variable",
            Self::AppConfig => "app config",
            Self::BuiltInDefault => "built-in default",
        }
    }

    /// Convenience predicate: was the URI resolved from the env var?
    pub(crate) fn is_env_var(&self) -> bool {
        matches!(self, Self::EnvVar)
    }
}

/// A resolved redirect URI plus the precedence level that produced it.
#[allow(dead_code)] // Consumed by U3 (Auth integration) and U4 (CLI redirect-uri get handler)
pub(crate) struct ResolvedRedirectUri {
    /// The effective URI to use for the `OAuth2` flow.
    pub uri: String,
    /// The precedence level that produced [`Self::uri`].
    pub source: ResolveSource,
}

/// Pure precedence helper: `REDIRECT_URI` env var > stored app value > built-in default.
///
/// `env_value` is the raw `Option<String>` produced by `std::env::var("REDIRECT_URI").ok()`.
/// `stored` is the per-app value from `TokenStore::get_app_redirect_uri`.
///
/// When `env_value` is set but fails [`Config::validate_redirect_uri`](Config::validate_redirect_uri), the helper
/// emits a one-line warning to stderr (via [`crate::output::warn_stderr`])
/// and falls through to the next precedence level. The pure helper has no
/// `OutputConfig` available, so the warning shape is intentionally minimal;
/// the binary's `OutputConfig::print_message` equivalent would be redundant
/// here since callers cannot suppress the env-var rejection in any meaningful
/// way.
///
/// Stored values are assumed valid — validation is enforced at `set_app_redirect_uri`
/// write time per R2.
pub(crate) fn resolve_redirect_uri_from(
    env_value: Option<String>,
    stored: Option<&str>,
) -> ResolvedRedirectUri {
    if let Some(v) = env_value {
        if Config::validate_redirect_uri(&v).is_ok() {
            return ResolvedRedirectUri {
                uri: v,
                source: ResolveSource::EnvVar,
            };
        }
        crate::output::warn_stderr(
            "REDIRECT_URI env value rejected by validation; falling through to next precedence level",
        );
    }

    if let Some(s) = stored
        && !s.is_empty()
    {
        return ResolvedRedirectUri {
            uri: s.to_string(),
            source: ResolveSource::AppConfig,
        };
    }

    ResolvedRedirectUri {
        uri: DEFAULT_REDIRECT_URI.to_string(),
        source: ResolveSource::BuiltInDefault,
    }
}

/// Thin wrapper around the pure precedence helper that opens the token
/// store at `store_path` and looks up the per-app stored URI for `app_name`.
///
/// Callers that already hold a `TokenStore` should call the pure helper
/// directly with the env var and the result of
/// `store.get_app_redirect_uri(app_name)` to avoid a second disk read.
#[must_use]
#[allow(private_interfaces)] // ResolvedRedirectUri is pub(crate) per KTD9; the plan keeps this resolver pub
pub fn resolve_redirect_uri(store_path: &Path, app_name: &str) -> ResolvedRedirectUri {
    let env = std::env::var("REDIRECT_URI").ok();
    let store = crate::store::TokenStore::new_with_path(store_path.to_str().unwrap_or("."));
    let stored = store.get_app_redirect_uri(app_name).map(str::to_string);
    resolve_redirect_uri_from(env, stored.as_deref())
}

// In-source unit tests cover the `pub(crate)` resolver internals that the
// external integration-test crate cannot reach without breaking visibility.
// Tests touching only the public API (`resolve_redirect_uri`,
// `validate_redirect_uri`, `DEFAULT_REDIRECT_URI`) live in
// `tests/config_tests.rs`.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_overrides_present_redirect_uri_records_env_provenance() {
        let cfg = Config::from_overrides(&EnvOverrides {
            redirect_uri: Some("https://example.com/cb".to_string()),
            ..EnvOverrides::default()
        });

        assert_eq!(cfg.redirect_uri_source, ResolveSource::EnvVar);
        assert!(cfg.redirect_uri_from_env);
    }

    #[test]
    fn from_overrides_absent_redirect_uri_records_builtin_provenance() {
        let cfg = Config::from_overrides(&EnvOverrides::default());

        assert_eq!(cfg.redirect_uri_source, ResolveSource::BuiltInDefault);
        assert!(!cfg.redirect_uri_from_env);
        assert_eq!(cfg.redirect_uri, DEFAULT_REDIRECT_URI);
    }

    #[test]
    fn resolve_redirect_uri_from_env_wins_over_stored() {
        let resolved = resolve_redirect_uri_from(
            Some("https://example.com/cb".to_string()),
            Some("http://stored.example.com/cb"),
        );
        assert_eq!(resolved.source, ResolveSource::EnvVar);
        assert_eq!(resolved.uri, "https://example.com/cb");
    }

    #[test]
    fn resolve_redirect_uri_from_stored_wins_over_default() {
        let resolved = resolve_redirect_uri_from(None, Some("http://localhost:9090/cb"));
        assert_eq!(resolved.source, ResolveSource::AppConfig);
        assert_eq!(resolved.uri, "http://localhost:9090/cb");
    }

    #[test]
    fn resolve_redirect_uri_from_default_fallback() {
        let resolved = resolve_redirect_uri_from(None, None);
        assert_eq!(resolved.source, ResolveSource::BuiltInDefault);
        assert_eq!(resolved.uri, DEFAULT_REDIRECT_URI);
    }

    #[test]
    fn resolve_redirect_uri_from_empty_stored_falls_through_to_default() {
        let resolved = resolve_redirect_uri_from(None, Some(""));
        assert_eq!(resolved.source, ResolveSource::BuiltInDefault);
        assert_eq!(resolved.uri, DEFAULT_REDIRECT_URI);
    }

    #[test]
    fn resolve_redirect_uri_from_invalid_env_falls_through_to_stored() {
        let resolved = resolve_redirect_uri_from(
            Some("not-a-url".to_string()),
            Some("http://localhost:9090/cb"),
        );
        assert_eq!(resolved.source, ResolveSource::AppConfig);
        assert_eq!(resolved.uri, "http://localhost:9090/cb");
    }

    // Exhaustive mapping locks both rendering paths so a new variant trips
    // the compiler (via the inner match) at the same time as a serde-mapping
    // assertion failure.
    #[test]
    fn resolve_source_as_text_label_exhaustive() {
        for variant in [
            ResolveSource::EnvVar,
            ResolveSource::AppConfig,
            ResolveSource::BuiltInDefault,
        ] {
            let label = variant.as_text_label();
            match variant {
                ResolveSource::EnvVar => {
                    assert_eq!(label, "REDIRECT_URI environment variable");
                }
                ResolveSource::AppConfig => {
                    assert_eq!(label, "app config");
                }
                ResolveSource::BuiltInDefault => {
                    assert_eq!(label, "built-in default");
                }
            }
        }
    }

    #[test]
    fn resolve_source_serialize_kebab_case_exhaustive() {
        for variant in [
            ResolveSource::EnvVar,
            ResolveSource::AppConfig,
            ResolveSource::BuiltInDefault,
        ] {
            let json = serde_json::to_string(&variant).expect("serialize ResolveSource");
            match variant {
                ResolveSource::EnvVar => assert_eq!(json, "\"env-var\""),
                ResolveSource::AppConfig => assert_eq!(json, "\"app-config\""),
                ResolveSource::BuiltInDefault => assert_eq!(json, "\"built-in-default\""),
            }
        }
    }

    #[test]
    fn resolve_source_is_env_var_predicate() {
        assert!(ResolveSource::EnvVar.is_env_var());
        assert!(!ResolveSource::AppConfig.is_env_var());
        assert!(!ResolveSource::BuiltInDefault.is_env_var());
    }

    // ── Thin-wrapper resolve_redirect_uri tests ─────────────────────────
    //
    // These exercise the disk-I/O wrapper. The env-var leg uses serial_test
    // to avoid races with other env-mutating tests in the same crate.

    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    fn write_store_with_redirect_uri(path: &std::path::Path, app: &str, uri: &str) {
        let yaml = format!(
            "apps:\n  {app}:\n    client_id: ''\n    client_secret: ''\n    redirect_uri: '{uri}'\n    oauth2_tokens: {{}}\ndefault_app: {app}\n"
        );
        fs::write(path, yaml).expect("write tempdir store");
    }

    fn write_empty_store(path: &std::path::Path, app: &str) {
        let yaml = format!(
            "apps:\n  {app}:\n    client_id: ''\n    client_secret: ''\n    oauth2_tokens: {{}}\ndefault_app: {app}\n"
        );
        fs::write(path, yaml).expect("write tempdir store");
    }

    #[test]
    #[serial]
    fn resolve_redirect_uri_env_wins() {
        let tmp = TempDir::new().expect("create tempdir for redirect_uri test");
        let store_path = tmp.path().join(".xurl");
        write_store_with_redirect_uri(&store_path, "app1", "http://localhost:7777/cb");

        unsafe {
            std::env::set_var("REDIRECT_URI", "https://example.com/cb");
        }
        let resolved = resolve_redirect_uri(&store_path, "app1");
        unsafe {
            std::env::remove_var("REDIRECT_URI");
        }

        assert_eq!(resolved.source, ResolveSource::EnvVar);
        assert_eq!(resolved.uri, "https://example.com/cb");
    }

    #[test]
    #[serial]
    fn resolve_redirect_uri_stored_when_no_env() {
        let tmp = TempDir::new().expect("create tempdir for redirect_uri test");
        let store_path = tmp.path().join(".xurl");
        write_store_with_redirect_uri(&store_path, "app1", "http://localhost:9090/cb");

        unsafe {
            std::env::remove_var("REDIRECT_URI");
        }
        let resolved = resolve_redirect_uri(&store_path, "app1");

        assert_eq!(resolved.source, ResolveSource::AppConfig);
        assert_eq!(resolved.uri, "http://localhost:9090/cb");
    }

    #[test]
    #[serial]
    fn resolve_redirect_uri_default_fallback() {
        let tmp = TempDir::new().expect("create tempdir for redirect_uri test");
        let store_path = tmp.path().join(".xurl");
        write_empty_store(&store_path, "app1");

        unsafe {
            std::env::remove_var("REDIRECT_URI");
        }
        let resolved = resolve_redirect_uri(&store_path, "app1");

        assert_eq!(resolved.source, ResolveSource::BuiltInDefault);
        assert_eq!(resolved.uri, DEFAULT_REDIRECT_URI);
    }
}
