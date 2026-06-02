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
}

/// Built-in default `OAuth2` redirect URI used when neither the
/// `REDIRECT_URI` env var nor a stored per-app value is set.
pub const DEFAULT_REDIRECT_URI: &str = "http://localhost:8080/callback";

impl Config {
    /// Creates a new `Config` from environment variables, falling back to defaults.
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
        let client_id = env_or_default("CLIENT_ID", "");
        let client_secret = env_or_default("CLIENT_SECRET", "");
        let redirect_uri_env = std::env::var("REDIRECT_URI").ok();
        let redirect_uri_from_env = redirect_uri_env.is_some();
        let redirect_uri = redirect_uri_env.unwrap_or_else(|| DEFAULT_REDIRECT_URI.to_string());
        let redirect_uri_source = if redirect_uri_from_env {
            ResolveSource::EnvVar
        } else {
            ResolveSource::BuiltInDefault
        };
        let auth_url = env_or_default("AUTH_URL", "https://x.com/i/oauth2/authorize");
        let token_url = env_or_default("TOKEN_URL", "https://api.x.com/2/oauth2/token");
        let api_base_url = env_or_default("API_BASE_URL", "https://api.x.com");
        let info_url = env_or_default("INFO_URL", &format!("{api_base_url}/2/users/me"));

        Self {
            client_id,
            client_secret,
            redirect_uri,
            auth_url,
            token_url,
            api_base_url,
            info_url,
            app_name: String::new(),
            redirect_uri_source,
            redirect_uri_from_env,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
/// emits a one-line warning to stderr (via `eprintln!`) and falls through to the
/// next precedence level. The pure helper has no `OutputConfig` available, so the
/// warning shape is intentionally minimal; the binary's `OutputConfig::print_message`
/// equivalent would be redundant here since callers cannot suppress the env-var
/// rejection in any meaningful way.
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
        eprintln!(
            "warning: REDIRECT_URI env value rejected by validation; falling through to next precedence level"
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

/// Returns an environment variable's value, or `default` if unset.
fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
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
