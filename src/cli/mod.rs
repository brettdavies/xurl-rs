/// CLI definition — clap derive with subcommands.
///
/// Mirrors the Go cobra command tree: root (raw mode) + shortcuts +
/// auth/media/webhook/version subcommands.
pub mod commands;
pub mod exit_codes;

use clap::{Parser, Subcommand};

pub use crate::output::OutputFormat;

/// Auth-enabled curl-like interface for the X API.
#[derive(Parser, Debug)]
#[command(
    name = "xr",
    about = "Auth enabled curl-like interface for the X API",
    long_about = r#"A command-line tool for making authenticated requests to the X API.

Shortcut commands (agent-friendly):
  xr post "Hello world!"                        Post to X
  xr reply 1234567890 "Nice!"                   Reply to a post
  xr read 1234567890                             Read a post
  xr search "golang" -n 20                       Search posts
  xr whoami                                      Show your profile
  xr like 1234567890                             Like a post
  xr repost 1234567890                           Repost
  xr follow @user                                Follow a user
  xr dm @user "Hey!"                             Send a DM
  xr timeline                                    Home timeline
  xr mentions                                    Your mentions

Raw API access (curl-style):
  basic requests        xr /2/users/me
                        xr -X POST /2/tweets -d '{"text":"Hello world!"}'
                        xr -H "Content-Type: application/json" /2/tweets
  authentication        xr --auth oauth2 /2/users/me
                        xr --auth oauth1 /2/users/me
                        xr --auth app /2/users/me
  media and streaming   xr media upload path/to/video.mp4
                        xr /2/tweets/search/stream --auth app
                        xr -s /2/users/me

Multi-app management:
  xr auth apps add my-app --client-id ... --client-secret ...
  xr auth apps list
  xr auth default                                # interactive picker
  xr auth default my-app                         # set by name
  xr --app my-app /2/users/me                    # per-request override

Run 'xr --help' to see all available commands."#,
    version
)]
pub struct Cli {
    /// HTTP method (GET by default)
    #[arg(short = 'X', long = "method", global = false)]
    pub method: Option<String>,

    /// Request headers
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    /// Request body data
    #[arg(short = 'd', long = "data")]
    pub data: Option<String>,

    /// Authentication type (oauth1, oauth2, app)
    #[arg(long = "auth")]
    pub auth_type: Option<String>,

    /// Username for `OAuth2` authentication
    #[arg(short = 'u', long = "username")]
    pub username: Option<String>,

    /// Print verbose information
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Add trace header to request
    #[arg(short = 't', long = "trace")]
    pub trace: bool,

    /// Force streaming mode
    #[arg(short = 's', long = "stream")]
    pub stream: bool,

    /// File to upload (for multipart requests)
    #[arg(short = 'F', long = "file")]
    pub file: Option<String>,

    /// Use a specific registered app (overrides default)
    #[arg(long = "app", global = true)]
    pub app: Option<String>,

    /// Output format: text (default), json (machine-readable), jsonl (streaming)
    #[arg(
        long,
        global = true,
        default_value = "text",
        value_enum,
        env = "XURL_OUTPUT"
    )]
    pub output: OutputFormat,

    /// Suppress all non-essential output (errors still go to stderr)
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Disable interactive prompts; fail with error instead
    #[arg(long, global = true)]
    pub no_interactive: bool,

    /// Request timeout in seconds
    #[arg(long, global = true, default_value = "30")]
    pub timeout: u64,

    /// Generate shell completion script and exit
    #[arg(long = "generate-completion", value_name = "SHELL", hide = true)]
    pub generate_completion: Option<clap_complete::Shell>,

    /// Subcommand to run
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// URL for raw mode (positional, only when no subcommand)
    pub url: Option<String>,
}

/// All subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ── Posting ──────────────────────────────────────────────────────
    /// Post to X
    Post {
        /// The text to post
        text: String,
        /// Media ID(s) to attach (repeatable)
        #[arg(long = "media-id")]
        media_ids: Vec<String>,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Reply to a post
    Reply {
        /// Post ID or URL to reply to
        post_id: String,
        /// The reply text
        text: String,
        /// Media ID(s) to attach (repeatable)
        #[arg(long = "media-id")]
        media_ids: Vec<String>,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Quote a post
    Quote {
        /// Post ID or URL to quote
        post_id: String,
        /// The quote text
        text: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Delete a post
    Delete {
        /// Post ID or URL to delete
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Reading ──────────────────────────────────────────────────────
    /// Read a post
    Read {
        /// Post ID or URL to read
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Search recent posts
    Search {
        /// Search query
        query: String,
        /// Number of results (min 10, max 100)
        #[arg(short = 'n', long = "max-results", default_value = "10")]
        max_results: i32,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── User Info ────────────────────────────────────────────────────
    /// Show the authenticated user's profile
    Whoami {
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Look up a user by username
    User {
        /// Username to look up
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Timeline & Mentions ──────────────────────────────────────────
    /// Show your home timeline
    Timeline {
        /// Number of results (1-100)
        #[arg(short = 'n', long = "max-results", default_value = "10")]
        max_results: i32,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Show your recent mentions
    Mentions {
        /// Number of results (5-100)
        #[arg(short = 'n', long = "max-results", default_value = "10")]
        max_results: i32,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Engagement ───────────────────────────────────────────────────
    /// Like a post
    Like {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Unlike a post
    Unlike {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Repost a post
    Repost {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Undo a repost
    Unrepost {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Bookmark a post
    Bookmark {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Remove a bookmark
    Unbookmark {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List your bookmarks
    Bookmarks {
        /// Number of results (1-100)
        #[arg(short = 'n', long = "max-results", default_value = "10")]
        max_results: i32,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List your liked posts
    Likes {
        /// Number of results (1-100)
        #[arg(short = 'n', long = "max-results", default_value = "10")]
        max_results: i32,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Social Graph ─────────────────────────────────────────────────
    /// Follow a user
    Follow {
        /// Username to follow
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Unfollow a user
    Unfollow {
        /// Username to unfollow
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List users you follow
    Following {
        /// Number of results (1-1000)
        #[arg(short = 'n', long = "max-results", default_value = "10")]
        max_results: i32,
        /// Username to list following for (default: you)
        #[arg(long = "of")]
        of: Option<String>,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List your followers
    Followers {
        /// Number of results (1-1000)
        #[arg(short = 'n', long = "max-results", default_value = "10")]
        max_results: i32,
        /// Username to list followers for (default: you)
        #[arg(long = "of")]
        of: Option<String>,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Block a user
    Block {
        /// Username to block
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Unblock a user
    Unblock {
        /// Username to unblock
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Mute a user
    Mute {
        /// Username to mute
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Unmute a user
    Unmute {
        /// Username to unmute
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Direct Messages ──────────────────────────────────────────────
    /// Send a direct message
    Dm {
        /// Username to DM
        #[arg(value_name = "USERNAME")]
        target_username: String,
        /// Message text
        text: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List recent direct messages
    Dms {
        /// Number of results (1-100)
        #[arg(short = 'n', long = "max-results", default_value = "10")]
        max_results: i32,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Auth ─────────────────────────────────────────────────────────
    /// Authentication management
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },

    // ── Media ────────────────────────────────────────────────────────
    /// Media upload operations
    Media {
        #[command(subcommand)]
        command: MediaCommands,
    },

    // ── Version ──────────────────────────────────────────────────────
    /// Show xurl version information
    Version,
}

/// Common flags shared by shortcut commands.
#[derive(clap::Args, Debug, Clone)]
pub struct CommonFlags {
    /// Authentication type (oauth1, oauth2, app)
    #[arg(long = "auth")]
    pub auth_type: Option<String>,

    /// `OAuth2` username to act as
    #[arg(short = 'u', long = "username")]
    pub username: Option<String>,

    /// Print verbose request/response info
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Add X-B3-Flags trace header
    #[arg(short = 't', long = "trace")]
    pub trace: bool,
}

impl CommonFlags {
    /// Converts to `RequestOptions`.
    pub fn to_request_options(&self) -> crate::api::RequestOptions {
        crate::api::RequestOptions {
            auth_type: self.auth_type.clone().unwrap_or_default(),
            username: self.username.clone().unwrap_or_default(),
            verbose: self.verbose,
            trace: self.trace,
            ..Default::default()
        }
    }
}

/// Auth subcommands.
#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Configure `OAuth2` authentication
    Oauth2,
    /// Configure `OAuth1` authentication
    Oauth1 {
        /// Consumer key
        #[arg(long = "consumer-key")]
        consumer_key: String,
        /// Consumer secret
        #[arg(long = "consumer-secret")]
        consumer_secret: String,
        /// Access token
        #[arg(long = "access-token")]
        access_token: String,
        /// Token secret
        #[arg(long = "token-secret")]
        token_secret: String,
    },
    /// Configure app-auth (bearer token)
    App {
        /// Bearer token
        #[arg(long = "bearer-token")]
        bearer_token: String,
    },
    /// Show authentication status
    Status,
    /// Clear authentication tokens
    Clear {
        /// Clear all authentication
        #[arg(long)]
        all: bool,
        /// Clear `OAuth1` tokens
        #[arg(long)]
        oauth1: bool,
        /// Clear `OAuth2` token for username
        #[arg(long = "oauth2-username")]
        oauth2_username: Option<String>,
        /// Clear bearer token
        #[arg(long)]
        bearer: bool,
    },
    /// Manage registered X API apps
    Apps {
        #[command(subcommand)]
        command: AppCommands,
    },
    /// Set default app and/or user
    Default {
        /// App name (optional)
        app_name: Option<String>,
        /// Username (optional)
        username: Option<String>,
    },
}

/// App management subcommands.
#[derive(Subcommand, Debug)]
pub enum AppCommands {
    /// Register a new X API app
    Add {
        /// App name
        name: String,
        /// `OAuth2` client ID
        #[arg(long = "client-id")]
        client_id: String,
        /// `OAuth2` client secret
        #[arg(long = "client-secret")]
        client_secret: String,
    },
    /// Update credentials for an existing app
    Update {
        /// App name
        name: String,
        /// `OAuth2` client ID
        #[arg(long = "client-id")]
        client_id: Option<String>,
        /// `OAuth2` client secret
        #[arg(long = "client-secret")]
        client_secret: Option<String>,
    },
    /// Remove a registered app
    Remove {
        /// App name
        name: String,
    },
    /// List registered apps
    List,
}

/// Media subcommands.
#[derive(Subcommand, Debug)]
pub enum MediaCommands {
    /// Upload media file
    Upload {
        /// File path
        file: String,
        /// Media type (e.g., video/mp4)
        #[arg(long = "media-type", default_value = "video/mp4")]
        media_type: String,
        /// Media category (e.g., `amplify_video`)
        #[arg(long = "category", default_value = "amplify_video")]
        category: String,
        /// Wait for media processing to complete
        #[arg(long = "wait", default_value = "true")]
        wait: bool,
        /// Authentication type
        #[arg(long = "auth")]
        auth_type: Option<String>,
        /// Username
        #[arg(short = 'u', long = "username")]
        username: Option<String>,
        /// Verbose output
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
        /// Trace header
        #[arg(short = 't', long = "trace")]
        trace: bool,
        /// Request headers
        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
    },
    /// Check media upload status
    Status {
        /// Media ID
        media_id: String,
        /// Authentication type
        #[arg(long = "auth")]
        auth_type: Option<String>,
        /// Username
        #[arg(short = 'u', long = "username")]
        username: Option<String>,
        /// Verbose output
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
        /// Wait for processing
        #[arg(short = 'w', long = "wait")]
        wait: bool,
        /// Trace header
        #[arg(short = 't', long = "trace")]
        trace: bool,
        /// Request headers
        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
    },
}
