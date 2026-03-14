use std::collections::HashMap;
use serde_json;
pub type TokenType = String;
pub const BEARER_TOKEN_TYPE: TokenType = "bearer";
pub const O_AUTH2_TOKEN_TYPE: TokenType = "oauth2";
pub const O_AUTH1_TOKEN_TYPE: TokenType = "oauth1";
/// Represents OAuth1 authentication tokens
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OAuth1Token {
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "token_secret")]
    pub token_secret: String,
    #[serde(rename = "consumer_key")]
    pub consumer_key: String,
    #[serde(rename = "consumer_secret")]
    pub consumer_secret: String,
}
/// Represents OAuth2 authentication tokens
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OAuth2Token {
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "refresh_token")]
    pub refresh_token: String,
    #[serde(rename = "expiration_time")]
    pub expiration_time: u64,
}
/// Token represents an authentication token
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Token {
    #[serde(rename = "type")]
    pub r#type: TokenType,
    #[serde(rename = "bearer")]
    #[serde(default)]
    pub bearer: String,
    #[serde(rename = "oauth2")]
    #[serde(default)]
    pub o_auth2: Box<OAuth2Token>,
    #[serde(rename = "oauth1")]
    #[serde(default)]
    pub o_auth1: Box<OAuth1Token>,
}
/// App holds the credentials and tokens for a single registered X API application.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct App {
    pub client_id: String,
    pub client_secret: String,
    #[serde(default)]
    pub default_user: String,
    #[serde(default)]
    pub o_auth2_tokens: std::collections::HashMap<String, Token>,
    #[serde(default)]
    pub o_auth1_token: Box<Token>,
    #[serde(default)]
    pub bearer_token: Box<Token>,
}
/// storeFile is the serialised YAML layout of ~/.xurl
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct storeFile {
    pub apps: std::collections::HashMap<String, Box<App>>,
    pub default_app: String,
}
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct legacyStore {
    #[serde(rename = "oauth2_tokens")]
    pub o_auth2_tokens: std::collections::HashMap<String, Token>,
    #[serde(rename = "oauth1_tokens")]
    #[serde(default)]
    pub o_auth1_token: Box<Token>,
    #[serde(rename = "bearer_token")]
    #[serde(default)]
    pub bearer_token: Box<Token>,
}
/// Manages authentication tokens across multiple apps.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TokenStore {
    pub apps: std::collections::HashMap<String, Box<App>>,
    pub default_app: String,
    pub file_path: String,
}
/// Creates a new TokenStore, loading from ~/.xurl (auto-migrating legacy JSON).
pub fn new_token_store() -> Box<TokenStore> {
    new_token_store_with_credentials("", "")
}
/// NewTokenStoreWithCredentials creates a TokenStore and backfills the given
/// client credentials into any app that was migrated without them (i.e. legacy
/// JSON migration where CLIENT_ID / CLIENT_SECRET came from env vars).
pub fn new_token_store_with_credentials(
    client_id: &str,
    client_secret: &str,
) -> Box<TokenStore> {
    let mut home_dir = std::env::var("HOME").unwrap();
    let mut file_path = std::path::PathBuf::from(home_dir).join(".xurl");
    let mut store = Box::new(TokenStore {
        apps: std::collections::HashMap::<String, Box<App>>::new(),
        file_path: file_path,
        ..Default::default()
    });
    os.stat(file_path).unwrap();
    let mut data = std::fs::read_to_string(file_path).unwrap();
    store.load_from_data(data);
    if client_id != "" || client_secret != "" {
        let mut dirty = false;
        for app in store.apps.iter() {
            let mut has_tokens = app.o_auth2_tokens.len() > 0
                || app.o_auth1_token.is_some() || app.bearer_token.is_some();
            if has_tokens && app.client_id == "" && client_id != "" {
                app.client_id = client_id;
                dirty = true;
            }
            if has_tokens && app.client_secret == "" && client_secret != "" {
                app.client_secret = client_secret;
                dirty = true;
            }
        }
        if dirty {
            _ = store.save_to_file();
        }
    }
    let mut app = store.active_app();
    if app.is_none() || app.o_auth1_token.is_none() || app.bearer_token.is_none() {
        let mut twurl_path = std::path::PathBuf::from(home_dir).join(".twurlrc");
        os.stat(twurl_path).unwrap();
        let mut err = store.import_from_twurlrc(twurl_path);
    }
    store
}
impl TokenStore {
    /// loadFromData tries YAML first, then falls back to legacy JSON migration.
    fn load_from_data(&mut self, data: Vec<u8>) {
        let mut sf: storeFile = Default::default();
        let mut err = yaml.unmarshal(data, &sf);
        if err.is_none() && sf.apps.len() > 0 {
            self.apps = sf.apps;
            self.default_app = sf.default_app;
            for app in self.apps.iter() {
                if app.o_auth2_tokens.is_none() {
                    app.o_auth2_tokens = std::collections::HashMap::<
                        String,
                        Token,
                    >::new();
                }
            }
            return;
        }
        let mut legacy: legacyStore = Default::default();
        let mut err = serde_json::from_str(&data);
        let mut oauth2 = legacy.o_auth2_tokens;
        if oauth2.is_none() {
            oauth2 = std::collections::HashMap::<String, Token>::new();
        }
        self.apps
            .insert(
                "default".to_string(),
                Box::new(App {
                    o_auth2_tokens: oauth2,
                    o_auth1_token: legacy.o_auth1_token,
                    bearer_token: legacy.bearer_token,
                    ..Default::default()
                }),
            );
        self.default_app = "default";
        _ = self.save_to_file();
    }
    /// AddApp registers a new application. If it's the only app it becomes default.
    pub fn add_app(
        &mut self,
        name: &str,
        client_id: &str,
        client_secret: &str,
    ) -> anyhow::Result<()> {
        let (_, exists) = self.apps[name];
        if exists {
            Ok(errors.new_token_store_error(format!("app {:?} already exists", name)))
        }
        self.apps
            .insert(
                name,
                Box::new(App {
                    client_id: client_id,
                    client_secret: client_secret,
                    o_auth2_tokens: std::collections::HashMap::<String, Token>::new(),
                    ..Default::default()
                }),
            );
        if self.apps.len() == 1 {
            self.default_app = name;
        }
        Ok(self.save_to_file())
    }
    /// UpdateApp updates the credentials of an existing application.
    pub fn update_app(
        &mut self,
        name: &str,
        client_id: &str,
        client_secret: &str,
    ) -> anyhow::Result<()> {
        let (app, exists) = self.apps[name];
        if !exists {
            Ok(errors.new_token_store_error(format!("app {:?} not found", name)))
        }
        if client_id != "" {
            app.client_id = client_id;
        }
        if client_secret != "" {
            app.client_secret = client_secret;
        }
        Ok(self.save_to_file())
    }
    /// RemoveApp removes a registered application and its tokens.
    pub fn remove_app(&mut self, name: &str) -> anyhow::Result<()> {
        let (_, exists) = self.apps[name];
        if !exists {
            Ok(errors.new_token_store_error(format!("app {:?} not found", name)))
        }
        self.apps.remove(name);
        if self.default_app == name {
            self.default_app = "";
            for n in 0..self.apps.len() {
                self.default_app = n;
                break;
            }
        }
        Ok(self.save_to_file())
    }
    /// SetDefaultApp sets the default application by name.
    pub fn set_default_app(&mut self, name: &str) -> anyhow::Result<()> {
        let (_, exists) = self.apps[name];
        if !exists {
            Ok(errors.new_token_store_error(format!("app {:?} not found", name)))
        }
        self.default_app = name;
        Ok(self.save_to_file())
    }
    /// ListApps returns sorted app names.
    pub fn list_apps(&mut self) -> Vec<String> {
        let mut names = Vec::with_capacity(self.apps.len());
        for name in 0..self.apps.len() {
            names = {
                names.push(name);
                names.clone()
            };
        }
        names.sort();
        names
    }
    /// GetApp returns an app by name.
    pub fn get_app(&mut self, name: &str) -> Box<App> {
        self.apps[name]
    }
    /// SetDefaultUser sets the default OAuth2 user for the named (or default) app.
    pub fn set_default_user(
        &mut self,
        app_name: &str,
        username: &str,
    ) -> anyhow::Result<()> {
        let mut app = self.resolve_app(app_name);
        let (_, ok) = app.o_auth2_tokens[username];
        if !ok {
            Ok(
                errors
                    .new_token_store_error(
                        format!("user {:?} not found in app", username),
                    ),
            )
        }
        app.default_user = username;
        Ok(self.save_to_file())
    }
    /// GetDefaultUser returns the default OAuth2 user for the named (or default) app.
    pub fn get_default_user(&mut self, app_name: &str) -> String {
        let mut app = self.resolve_app(app_name);
        app.default_user
    }
    /// GetDefaultApp returns the default app name.
    pub fn get_default_app(&mut self) -> String {
        self.default_app
    }
    /// GetActiveAppName returns the name of the active app (explicit or default).
    pub fn get_active_app_name(&mut self, explicit: &str) -> String {
        if explicit != "" {
            explicit
        }
        self.default_app
    }
    /// activeApp returns the current default App, or nil.
    fn active_app(&mut self) -> Box<App> {
        self.apps[self.default_app]
    }
    /// activeAppOrCreate returns the active app; creates "default" if none exist.
    fn active_app_or_create(&mut self) -> Box<App> {
        let mut app = self.active_app();
        if app.is_some() {
            app
        }
        self.apps
            .insert(
                "default".to_string(),
                Box::new(App {
                    o_auth2_tokens: std::collections::HashMap::<String, Token>::new(),
                    ..Default::default()
                }),
            );
        if self.default_app == "" {
            self.default_app = "default";
        }
        self.apps["default"]
    }
    /// ResolveApp returns the app for the given name, or the default app.
    pub fn resolve_app(&mut self, name: &str) -> Box<App> {
        if name != "" {
            let (app, ok) = self.apps[name];
            if ok {
                app
            }
        }
        self.active_app_or_create()
    }
    /// Imports tokens from a twurlrc file into the active app.
    fn import_from_twurlrc(&mut self, file_path: &str) -> anyhow::Result<()> {
        let mut data = std::fs::read_to_string(file_path)?;
        let mut twurl_config: struct_Profiles_map_string_map_string_struct_Username_string__yaml___username_____ConsumerKey_string__yaml___consumer_key_____ConsumerSecret_string__yaml___consumer_secret_____Token_string__yaml___token_____Secret_string__yaml___secret______yaml___profiles_____Configuration_struct_DefaultProfile___string__yaml___default_profile______yaml___configuration_____BearerTokens_map_string_string__yaml___bearer_tokens____ = Default::default();
        let mut err = yaml.unmarshal(data, &twurl_config);
        let mut app = self.active_app_or_create();
        for consumer_keys in twurl_config.profiles.iter() {
            for (consumer_key, profile) in consumer_keys.iter().enumerate() {
                if app.o_auth1_token.is_none() {
                    app.o_auth1_token = Box::new(Token {
                        r#type: OAuth1TokenType,
                        o_auth1: Box::new(OAuth1Token {
                            access_token: profile.token,
                            token_secret: profile.secret,
                            consumer_key: consumer_key,
                            consumer_secret: profile.consumer_secret,
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
                break;
            }
            break;
        }
        if twurl_config.bearer_tokens.len() > 0 {
            for bearer_token in twurl_config.bearer_tokens.iter() {
                app.bearer_token = Box::new(Token {
                    r#type: BearerTokenType,
                    bearer: bearer_token,
                    ..Default::default()
                });
                break;
            }
        }
        Ok(self.save_to_file())
    }
    /// SaveBearerToken saves a bearer token into the resolved app.
    pub fn save_bearer_token(&mut self, token: &str) -> anyhow::Result<()> {
        Ok(self.save_bearer_token_for_app("", token))
    }
    /// SaveBearerTokenForApp saves a bearer token into the named app.
    pub fn save_bearer_token_for_app(
        &mut self,
        app_name: &str,
        token: &str,
    ) -> anyhow::Result<()> {
        let mut app = self.resolve_app(app_name);
        app.bearer_token = Box::new(Token {
            r#type: BearerTokenType,
            bearer: token,
            ..Default::default()
        });
        Ok(self.save_to_file())
    }
    /// SaveOAuth2Token saves an OAuth2 token into the resolved app.
    pub fn save_o_auth2_token(
        &mut self,
        username: &str,
        access_token: &str,
        refresh_token: &str,
        expiration_time: u64,
    ) -> anyhow::Result<()> {
        Ok(
            self
                .save_o_auth2_token_for_app(
                    "",
                    username,
                    access_token,
                    refresh_token,
                    expiration_time,
                ),
        )
    }
    /// SaveOAuth2TokenForApp saves an OAuth2 token into the named app.
    pub fn save_o_auth2_token_for_app(
        &mut self,
        app_name: &str,
        username: &str,
        access_token: &str,
        refresh_token: &str,
        expiration_time: u64,
    ) -> anyhow::Result<()> {
        let mut app = self.resolve_app(app_name);
        if app.o_auth2_tokens.is_none() {
            app.o_auth2_tokens = std::collections::HashMap::<String, Token>::new();
        }
        app.o_auth2_tokens
            .insert(
                username,
                Token {
                    r#type: OAuth2TokenType,
                    o_auth2: Box::new(OAuth2Token {
                        access_token: access_token,
                        refresh_token: refresh_token,
                        expiration_time: expiration_time,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            );
        Ok(self.save_to_file())
    }
    /// SaveOAuth1Tokens saves OAuth1 tokens into the resolved app.
    pub fn save_o_auth1_tokens(
        &mut self,
        access_token: &str,
        token_secret: &str,
        consumer_key: &str,
        consumer_secret: &str,
    ) -> anyhow::Result<()> {
        Ok(
            self
                .save_o_auth1_tokens_for_app(
                    "",
                    access_token,
                    token_secret,
                    consumer_key,
                    consumer_secret,
                ),
        )
    }
    /// SaveOAuth1TokensForApp saves OAuth1 tokens into the named app.
    pub fn save_o_auth1_tokens_for_app(
        &mut self,
        app_name: &str,
        access_token: &str,
        token_secret: &str,
        consumer_key: &str,
        consumer_secret: &str,
    ) -> anyhow::Result<()> {
        let mut app = self.resolve_app(app_name);
        app.o_auth1_token = Box::new(Token {
            r#type: OAuth1TokenType,
            o_auth1: Box::new(OAuth1Token {
                access_token: access_token,
                token_secret: token_secret,
                consumer_key: consumer_key,
                consumer_secret: consumer_secret,
                ..Default::default()
            }),
            ..Default::default()
        });
        Ok(self.save_to_file())
    }
    /// GetOAuth2Token gets an OAuth2 token for a username from the resolved app.
    pub fn get_o_auth2_token(&mut self, username: &str) -> Box<Token> {
        self.get_o_auth2_token_for_app("", username)
    }
    /// GetOAuth2TokenForApp gets an OAuth2 token for a username from the named app.
    pub fn get_o_auth2_token_for_app(
        &mut self,
        app_name: &str,
        username: &str,
    ) -> Box<Token> {
        let mut app = self.resolve_app(app_name);
        let (token, ok) = app.o_auth2_tokens[username];
        if ok {
            &token
        }
        None
    }
    /// GetFirstOAuth2Token gets the first OAuth2 token from the resolved app.
    pub fn get_first_o_auth2_token(&mut self) -> Box<Token> {
        self.get_first_o_auth2_token_for_app("")
    }
    /// GetFirstOAuth2TokenForApp gets the default user's token, or the first OAuth2 token from the named app.
    pub fn get_first_o_auth2_token_for_app(&mut self, app_name: &str) -> Box<Token> {
        let mut app = self.resolve_app(app_name);
        if app.default_user != "" {
            let (token, ok) = app.o_auth2_tokens[app.default_user];
            if ok {
                &token
            }
        }
        for token in app.o_auth2_tokens.iter() {
            &token
        }
        None
    }
    /// GetOAuth1Tokens gets OAuth1 tokens from the resolved app.
    pub fn get_o_auth1_tokens(&mut self) -> Box<Token> {
        self.get_o_auth1_tokens_for_app("")
    }
    /// GetOAuth1TokensForApp gets OAuth1 tokens from the named app.
    pub fn get_o_auth1_tokens_for_app(&mut self, app_name: &str) -> Box<Token> {
        let mut app = self.resolve_app(app_name);
        app.o_auth1_token
    }
    /// GetBearerToken gets the bearer token from the resolved app.
    pub fn get_bearer_token(&mut self) -> Box<Token> {
        self.get_bearer_token_for_app("")
    }
    /// GetBearerTokenForApp gets the bearer token from the named app.
    pub fn get_bearer_token_for_app(&mut self, app_name: &str) -> Box<Token> {
        let mut app = self.resolve_app(app_name);
        app.bearer_token
    }
    /// ClearOAuth2Token clears an OAuth2 token for a username from the resolved app.
    pub fn clear_o_auth2_token(&mut self, username: &str) -> anyhow::Result<()> {
        Ok(self.clear_o_auth2_token_for_app("", username))
    }
    /// ClearOAuth2TokenForApp clears an OAuth2 token for a username from the named app.
    pub fn clear_o_auth2_token_for_app(
        &mut self,
        app_name: &str,
        username: &str,
    ) -> anyhow::Result<()> {
        let mut app = self.resolve_app(app_name);
        app.o_auth2_tokens.remove(username);
        Ok(self.save_to_file())
    }
    /// ClearOAuth1Tokens clears OAuth1 tokens from the resolved app.
    pub fn clear_o_auth1_tokens(&mut self) -> anyhow::Result<()> {
        Ok(self.clear_o_auth1_tokens_for_app(""))
    }
    /// ClearOAuth1TokensForApp clears OAuth1 tokens from the named app.
    pub fn clear_o_auth1_tokens_for_app(
        &mut self,
        app_name: &str,
    ) -> anyhow::Result<()> {
        let mut app = self.resolve_app(app_name);
        app.o_auth1_token = None;
        Ok(self.save_to_file())
    }
    /// ClearBearerToken clears the bearer token from the resolved app.
    pub fn clear_bearer_token(&mut self) -> anyhow::Result<()> {
        Ok(self.clear_bearer_token_for_app(""))
    }
    /// ClearBearerTokenForApp clears the bearer token from the named app.
    pub fn clear_bearer_token_for_app(&mut self, app_name: &str) -> anyhow::Result<()> {
        let mut app = self.resolve_app(app_name);
        app.bearer_token = None;
        Ok(self.save_to_file())
    }
    /// ClearAll clears all tokens from the resolved app.
    pub fn clear_all(&mut self) -> anyhow::Result<()> {
        Ok(self.clear_all_for_app(""))
    }
    /// ClearAllForApp clears all tokens from the named app.
    pub fn clear_all_for_app(&mut self, app_name: &str) -> anyhow::Result<()> {
        let mut app = self.resolve_app(app_name);
        app.o_auth2_tokens = std::collections::HashMap::<String, Token>::new();
        app.o_auth1_token = None;
        app.bearer_token = None;
        Ok(self.save_to_file())
    }
    /// GetOAuth2Usernames gets all OAuth2 usernames from the resolved app.
    pub fn get_o_auth2_usernames(&mut self) -> Vec<String> {
        self.get_o_auth2_usernames_for_app("")
    }
    /// GetOAuth2UsernamesForApp gets all OAuth2 usernames from the named app.
    pub fn get_o_auth2_usernames_for_app(&mut self, app_name: &str) -> Vec<String> {
        let mut app = self.resolve_app(app_name);
        let mut usernames = Vec::with_capacity(app.o_auth2_tokens.len());
        for username in 0..app.o_auth2_tokens.len() {
            usernames = {
                usernames.push(username);
                usernames.clone()
            };
        }
        usernames.sort();
        usernames
    }
    /// HasOAuth1Tokens checks if OAuth1 tokens exist in the resolved app.
    pub fn has_o_auth1_tokens(&mut self) -> bool {
        let mut app = self.active_app();
        app.is_some() && app.o_auth1_token.is_some()
    }
    /// HasBearerToken checks if a bearer token exists in the resolved app.
    pub fn has_bearer_token(&mut self) -> bool {
        let mut app = self.active_app();
        app.is_some() && app.bearer_token.is_some()
    }
    /// Saves the token store to ~/.xurl in YAML format.
    fn save_to_file(&mut self) -> anyhow::Result<()> {
        let mut sf = storeFile {
            apps: self.apps,
            default_app: self.default_app,
            ..Default::default()
        };
        let mut data = yaml.marshal(&sf)?;
        err = std::fs::write(self.file_path, data);
        Ok(())
    }
}
