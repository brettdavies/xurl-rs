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
}

impl Config {
    /// Creates a new `Config` from environment variables, falling back to defaults.
    #[must_use]
    pub fn new() -> Self {
        let client_id = env_or_default("CLIENT_ID", "");
        let client_secret = env_or_default("CLIENT_SECRET", "");
        let redirect_uri = env_or_default("REDIRECT_URI", "http://localhost:8080/callback");
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
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns an environment variable's value, or `default` if unset.
fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
