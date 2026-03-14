/// Config holds the application configuration
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub auth_url: String,
    pub token_url: String,
    pub api_base_url: String,
    pub info_url: String,
    pub app_name: String,
}
/// NewConfig creates a new Config from environment variables
pub fn new_config() -> Box<Config> {
    let mut client_id = get_env_or_default("CLIENT_ID", "");
    let mut client_secret = get_env_or_default("CLIENT_SECRET", "");
    let mut redirect_uri = get_env_or_default(
        "REDIRECT_URI",
        "http://localhost:8080/callback",
    );
    let mut auth_url = get_env_or_default(
        "AUTH_URL",
        "https://x.com/i/oauth2/authorize",
    );
    let mut token_url = get_env_or_default(
        "TOKEN_URL",
        "https://api.x.com/2/oauth2/token",
    );
    let mut api_base_url = get_env_or_default("API_BASE_URL", "https://api.x.com");
    let mut info_url = get_env_or_default(
        "INFO_URL",
        format!("{}/2/users/me", api_base_url),
    );
    Box::new(Config {
        client_id: client_id,
        client_secret: client_secret,
        redirect_uri: redirect_uri,
        auth_url: auth_url,
        token_url: token_url,
        api_base_url: api_base_url,
        info_url: info_url,
        ..Default::default()
    })
}
/// Helper function to get environment variable with default value
fn get_env_or_default(key: &str, default_value: &str) -> String {
    let (value, exists) = os.lookup_env(key);
    if !exists {
        default_value
    }
    value
}
