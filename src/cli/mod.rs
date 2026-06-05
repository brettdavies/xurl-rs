/// CLI definition — clap derive with subcommands.
///
/// Mirrors the Go cobra command tree: root (raw mode) + shortcuts +
/// auth/media/webhook/version subcommands.
pub mod commands;
pub mod exit_codes;
pub mod runner;

pub use runner::{run, run_argv, run_with_store_path};

use clap::builder::FalseyValueParser;
use clap::{Parser, Subcommand, ValueEnum};

pub use crate::output::OutputFormat;
pub use crate::skill_install::SkillHost;

/// Color output choice. Honored by `OutputConfig` together with `NO_COLOR`
/// and TTY detection.
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq, Default)]
#[value(rename_all = "lower")]
pub enum ColorChoice {
    /// Enable color when stderr is a TTY and `NO_COLOR` is unset.
    #[default]
    Auto,
    /// Always emit ANSI color escapes (still suppressed by `NO_COLOR`).
    Always,
    /// Never emit ANSI color escapes.
    Never,
}

/// Root `--help` appendix listing the agentic-flag matrix, env-var
/// equivalents, exit-code contract, and TTY-aware auto-quiet behavior.
///
/// Every env var the binary reads at the root level appears here so agents
/// can discover the agentic surface from `xr --help` alone (corpus doc:
/// `cli-env-vars-must-appear-in-help-2026-04-20.md`).
const ROOT_HELP: &str = "\
Examples:
  Authenticate (browser):
    xr auth oauth2
  Authenticate (headless / SSH / container):
    xr auth oauth2 --no-browser --step 1
  Post a status update (text vs JSON):
    xr post \"Hello world\"
    xr post \"Hello world\" --output json
  Search recent posts with env-var precedence:
    XURL_OUTPUT=json xr search \"rustlang\" -n 25
  Browse the full curated gallery:
    xr examples

ENVIRONMENT VARIABLES:
  XURL_OUTPUT            Output format: text, json, jsonl, ndjson, yaml, csv, tsv (same as --output)
  XURL_CURSOR            Pagination cursor / page token (same as --cursor)
  XURL_QUIET             Suppress non-essential output (same as --quiet)
  XURL_NO_INTERACTIVE    Fail instead of prompting (same as --no-interactive)
  XURL_TIMEOUT           Network timeout in seconds (same as --timeout)
  XURL_COLOR             Color control: auto, always, never (same as --color)
  XURL_VERBOSE           Verbose request/response logging (same as -v/--verbose)
  XURL_APP               Override default app (same as --app)
  XURL_JSON              Shorthand for XURL_OUTPUT=json (same as --json)
  XURL_JSONL             Shorthand for XURL_OUTPUT=jsonl (same as --jsonl)
  XURL_NO_BROWSER        Skip browser-open on `auth oauth2` (same as --no-browser)
  REDIRECT_URI           OAuth2 redirect URI override for the active app

Flags override env vars when both are set. NO_COLOR=1 always wins over
--color/XURL_COLOR (https://no-color.org). --no-pager is a documented
no-op so agents can pass it unconditionally; xr never invokes $PAGER.

INPUT FROM STDIN:
  Subcommands that accept JSON input (currently `xr validate`) read from
  stdin when no file argument is given or when `-` is passed as the path.
  This matches the standard CLI convention for piping data:
    cat tweet.json | xr validate --schema tweet --output json
    cat tweet.json | xr validate - --schema tweet --output json

EXIT CODES:
  0    success
  1    general error
  2    invalid arguments or authentication required
  3    rate-limited (HTTP 429)
  4    not found (HTTP 404)
  5    network error

TTY behavior:
  When stdout is not a TTY, color output is auto-stripped (unless
  --color always is set) and human-only banners are suppressed, so piping
  to jaq or redirecting to a file produces clean machine-readable output
  without any extra flags.
";

/// `xr post` examples — text + JSON paired, plus a reply variant and a
/// media-attached form.
const POST_HELP: &str = "\
Examples:
  Post a status update (text):
    xr post \"Hello world\"
  Post the same update (machine-readable JSON envelope):
    xr post \"Hello world\" --output json
  Post with media (run `xr media upload` first to get an ID):
    xr post \"Look at this\" --media-id 1234567890 --output json
  Post quietly (suppress human-readable banners):
    xr post \"Ship it\" --quiet --output json
";

/// `xr reply` examples — anchor by post ID, paired text + JSON.
const REPLY_HELP: &str = "\
Examples:
  Reply to a post by ID:
    xr reply 1234567890 \"Congrats!\"
  Reply with JSON output for scripting:
    xr reply 1234567890 \"Congrats!\" --output json
  Reply to a post URL (xr accepts either):
    xr reply https://x.com/jack/status/1234567890 \"Nice thread.\"
  Reply with a media attachment:
    xr reply 1234567890 \"Here's a photo\" --media-id 222 --output json
";

/// `xr quote` examples — paired text + JSON.
const QUOTE_HELP: &str = "\
Examples:
  Quote-post by ID:
    xr quote 1234567890 \"Worth a read.\"
  Quote-post with JSON envelope:
    xr quote 1234567890 \"Worth a read.\" --output json
  Quote-post from a URL:
    xr quote https://x.com/jack/status/1234567890 \"Thread.\"
";

/// `xr delete` examples — destructive op; advertise non-interactive shape.
const DELETE_HELP: &str = "\
Examples:
  Delete a post by ID (text):
    xr delete 1234567890
  Delete with JSON envelope:
    xr delete 1234567890 --output json
  Delete in a non-interactive context (CI, agent):
    xr delete 1234567890 --no-interactive --output json
";

/// `xr read` examples — paired text + JSON, plus a pipe-to-jaq invocation.
const READ_HELP: &str = "\
Examples:
  Read a post (text):
    xr read 1234567890
  Read a post (JSON):
    xr read 1234567890 --output json
  Read a post URL:
    xr read https://x.com/jack/status/1234567890 --output json
  Extract a single field via jaq:
    xr read 1234567890 --output json | jaq '.data.text'
";

/// `xr search` examples — text + JSON, env-var override, JSONL pipeline.
const SEARCH_HELP: &str = "\
Examples:
  Search recent posts (text):
    xr search \"rustlang\"
  Search with 25 results, JSON envelope:
    xr search \"rustlang\" -n 25 --output json
  Demonstrate env-var precedence (XURL_OUTPUT == --output):
    XURL_OUTPUT=json xr search \"rustlang\"
  Stream-friendly JSONL piped to jaq:
    xr search \"rustlang\" --output jsonl | jaq '.id'
";

/// `xr whoami` examples — paired text + JSON.
const WHOAMI_HELP: &str = "\
Examples:
  Show your profile (text):
    xr whoami
  Same, as a JSON envelope:
    xr whoami --output json
  Act as a specific authenticated user (multi-account):
    xr whoami --username alice --output json
";

/// `xr user` examples — paired text + JSON.
const USER_HELP: &str = "\
Examples:
  Look up a user by handle (text):
    xr user jack
  Same, as JSON:
    xr user jack --output json
  Use the @ prefix (xr accepts either):
    xr user @jack --output json
";

/// `xr timeline` examples — paired text + JSON, plus JSONL pipeline.
const TIMELINE_HELP: &str = "\
Examples:
  Home timeline (last 10, text):
    xr timeline
  Home timeline (50 results, JSON envelope):
    xr timeline -n 50 --output json
  Stream-friendly JSONL piped to jaq:
    xr timeline -n 100 --output jsonl | jaq '.id'
";

/// `xr mentions` examples — paired text + JSON.
const MENTIONS_HELP: &str = "\
Examples:
  Your mentions (text):
    xr mentions
  Last 25, JSON envelope:
    xr mentions -n 25 --output json
  JSONL pipeline:
    xr mentions -n 100 --output jsonl | jaq '.id'
";

/// `xr like` examples — paired text + JSON.
const LIKE_HELP: &str = "\
Examples:
  Like a post (text):
    xr like 1234567890
  Like a post (JSON envelope):
    xr like 1234567890 --output json
  Idempotent: re-liking is a server-side no-op.
";

/// `xr unlike` examples — paired text + JSON.
const UNLIKE_HELP: &str = "\
Examples:
  Unlike a post (text):
    xr unlike 1234567890
  Unlike (JSON envelope):
    xr unlike 1234567890 --output json
";

/// `xr repost` examples — paired text + JSON.
const REPOST_HELP: &str = "\
Examples:
  Repost a post (text):
    xr repost 1234567890
  Repost (JSON envelope):
    xr repost 1234567890 --output json
";

/// `xr unrepost` examples — paired text + JSON.
const UNREPOST_HELP: &str = "\
Examples:
  Undo a repost (text):
    xr unrepost 1234567890
  Undo a repost (JSON envelope):
    xr unrepost 1234567890 --output json
";

/// `xr bookmark` examples — paired text + JSON.
const BOOKMARK_HELP: &str = "\
Examples:
  Bookmark a post (text):
    xr bookmark 1234567890
  Bookmark (JSON envelope):
    xr bookmark 1234567890 --output json
";

/// `xr unbookmark` examples — paired text + JSON.
const UNBOOKMARK_HELP: &str = "\
Examples:
  Remove a bookmark (text):
    xr unbookmark 1234567890
  Remove a bookmark (JSON envelope):
    xr unbookmark 1234567890 --output json
";

/// `xr bookmarks` examples — paired text + JSON, JSONL pipeline.
const BOOKMARKS_HELP: &str = "\
Examples:
  List your bookmarks (text):
    xr bookmarks
  100 results, JSON envelope:
    xr bookmarks -n 100 --output json
  JSONL piped to jaq:
    xr bookmarks -n 100 --output jsonl | jaq '.id'
";

/// `xr likes` examples — paired text + JSON, JSONL pipeline.
const LIKES_HELP: &str = "\
Examples:
  List your liked posts (text):
    xr likes
  100 results, JSON envelope:
    xr likes -n 100 --output json
  JSONL piped to jaq:
    xr likes -n 100 --output jsonl | jaq '.id'
";

/// `xr follow` examples — paired text + JSON.
const FOLLOW_HELP: &str = "\
Examples:
  Follow a user (text):
    xr follow @jack
  Follow (JSON envelope):
    xr follow @jack --output json
  Without the @ prefix:
    xr follow jack --output json
";

/// `xr unfollow` examples — paired text + JSON.
const UNFOLLOW_HELP: &str = "\
Examples:
  Unfollow a user (text):
    xr unfollow @jack
  Unfollow (JSON envelope):
    xr unfollow @jack --output json
";

/// `xr following` examples — paired text + JSON, `--of` for another user.
const FOLLOWING_HELP: &str = "\
Examples:
  Users you follow (text):
    xr following
  100 results, JSON envelope:
    xr following -n 100 --output json
  Who someone else follows:
    xr following --of @jack --output json
";

/// `xr followers` examples — paired text + JSON, `--of` for another user.
const FOLLOWERS_HELP: &str = "\
Examples:
  Your followers (text):
    xr followers
  100 results, JSON envelope:
    xr followers -n 100 --output json
  Someone else's followers:
    xr followers --of @jack --output json
";

/// `xr mute` examples — paired text + JSON.
const MUTE_HELP: &str = "\
Examples:
  Mute a user (text):
    xr mute @noisy
  Mute (JSON envelope):
    xr mute @noisy --output json
";

/// `xr unmute` examples — paired text + JSON.
const UNMUTE_HELP: &str = "\
Examples:
  Unmute a user (text):
    xr unmute @noisy
  Unmute (JSON envelope):
    xr unmute @noisy --output json
";

/// `xr usage` examples — paired text + JSON.
const USAGE_HELP: &str = "\
Examples:
  Show API usage (text):
    xr usage
  Show API usage (JSON envelope, machine-parseable cap data):
    xr usage --output json
  Quiet + JSON for clean agent consumption:
    xr usage --quiet --output json
";

/// `xr dm` examples — paired text + JSON.
const DM_HELP: &str = "\
Examples:
  Send a DM (text):
    xr dm @recipient \"Hello\"
  Send a DM (JSON envelope):
    xr dm @recipient \"Hello\" --output json
  Without the @ prefix:
    xr dm recipient \"Hi\" --output json
";

/// `xr dms` examples — paired text + JSON, JSONL pipeline.
const DMS_HELP: &str = "\
Examples:
  List recent DM events (text):
    xr dms
  50 results, JSON envelope:
    xr dms -n 50 --output json
  JSONL piped to jaq:
    xr dms -n 100 --output jsonl | jaq '.id'
";

/// `xr auth` parent help — points to subcommands.
const AUTH_HELP: &str = "\
Examples:
  Browser-based OAuth2 (default):
    xr auth oauth2
  Headless OAuth2 (servers, containers):
    xr auth oauth2 --no-browser --step 1
  Bearer token for read-only / search:
    xr auth app --bearer-token \"$TOKEN\"
  Show current auth state, machine-readable:
    xr auth status --output json
";

/// `xr media` parent help — points to subcommands.
const MEDIA_HELP: &str = "\
Examples:
  Upload an image:
    xr media upload ./photo.png --media-type image/png --category tweet_image
  Upload a video and wait for processing:
    xr media upload ./clip.mp4 --wait --output json
  Check upload status:
    xr media status 1234567890 --output json
";

/// `xr schema` examples — paired text + JSON, list, all.
const SCHEMA_HELP: &str = "\
Examples:
  List all command schemas (text):
    xr schema --list
  List all command schemas (JSON):
    xr schema --list --output json
  Dump a single command's schema:
    xr schema post --output json
  Dump every schema as one JSON document:
    xr schema --all --output json
";

/// `xr completions` examples.
const COMPLETIONS_HELP: &str = "\
Examples:
  Bash:
    xr completions bash > ~/.bash_completion.d/xr
  Zsh:
    xr completions zsh > ~/.zfunc/_xr
  Fish:
    xr completions fish > ~/.config/fish/completions/xr.fish
";

/// `xr version` examples.
const VERSION_HELP: &str = "\
Examples:
  Print the binary version (text):
    xr version
  Same, JSON-friendly via the top-level flag:
    xr --version
";

/// `xr validate` — JSON-shape validation against bundled response schemas.
const VALIDATE_HELP: &str = "\
Examples:
  Read JSON from stdin (no file argument):
    cat tweet.json | xr validate --output json
  Same, written as `xr validate -` for explicit stdin:
    cat tweet.json | xr validate - --schema tweet --output json
  Validate a file against a specific schema:
    xr validate tweets.json --schema tweets --output json
  Validate the canonical xurl error envelope:
    echo '{\"status\":\"error\",\"reason\":\"x\",\"exit_code\":1,\"message\":\"y\"}' | xr validate --schema envelope --output json
";

/// `xr examples` advertises itself — paired text + JSON for the top-level
/// flag round-trip.
const EXAMPLES_HELP: &str = "\
Examples:
  Print the full curated gallery:
    xr examples
  Discover env-var precedence and exit codes:
    xr --help
  Browse a single command's curated examples:
    xr post --help
";

/// `xr auth oauth2` — browser + headless flows.
const AUTH_OAUTH2_HELP: &str = "\
Examples:
  Interactive browser flow (default):
    xr auth oauth2
  Headless step 1 (generate auth URL on a server / container):
    xr auth oauth2 --no-browser --step 1
  Headless step 2 (paste the redirect URL after authorizing):
    xr auth oauth2 --no-browser --step 2 --auth-url 'https://localhost/callback?code=...&state=...'
  Headless step 2 reading the URL from stdin (recommended on shared boxes):
    echo 'https://localhost/callback?code=...&state=...' | xr auth oauth2 --no-browser --step 2 --auth-url - --output json
  Label the saved token with a specific username (skips /2/users/me):
    xr auth oauth2 alice --output json
";

/// `xr auth oauth1` — non-interactive OAuth1 setup.
const AUTH_OAUTH1_HELP: &str = "\
Examples:
  Configure OAuth1 from app + user credentials:
    xr auth oauth1 \\
      --consumer-key CK --consumer-secret CS \\
      --access-token AT --token-secret TS
  Same, with JSON envelope for scripted setup:
    xr auth oauth1 --consumer-key CK --consumer-secret CS \\
      --access-token AT --token-secret TS --output json
";

/// `xr auth app` — bearer-token configuration.
const AUTH_APP_HELP: &str = "\
Examples:
  Set the bearer token from an env var:
    xr auth app --bearer-token \"$XURL_BEARER_TOKEN\"
  Same, JSON envelope:
    xr auth app --bearer-token \"$XURL_BEARER_TOKEN\" --output json
  Test the configured bearer:
    xr auth status --output json
";

/// `xr auth status` — paired text + JSON.
const AUTH_STATUS_HELP: &str = "\
Examples:
  Show current auth state (text):
    xr auth status
  Same, machine-readable:
    xr auth status --output json
  Quiet + JSON for agent consumption:
    xr auth status --quiet --output json
";

/// `xr auth clear` — destructive op; advertise non-interactive shape.
const AUTH_CLEAR_HELP: &str = "\
Examples:
  Clear all tokens (text):
    xr auth clear --all
  Clear only the bearer token (JSON envelope):
    xr auth clear --bearer --output json
  Clear a single OAuth2 user:
    xr auth clear --oauth2-username alice --output json
  Non-interactive, fail without an explicit selector:
    xr auth clear --all --no-interactive --output json
";

/// `xr auth apps` parent help — points to subcommands.
const AUTH_APPS_HELP: &str = "\
Examples:
  Register a new app:
    xr auth apps add my-app --client-id ID --client-secret SECRET
  List registered apps (JSON):
    xr auth apps list --output json
  Update credentials for an existing app:
    xr auth apps update my-app --client-secret NEW
  Inspect or set the stored OAuth2 redirect URI:
    xr auth apps redirect-uri get my-app --output json
";

/// `xr auth default` — paired text + JSON.
const AUTH_DEFAULT_HELP: &str = "\
Examples:
  Interactive picker (TTY):
    xr auth default
  Set default by name:
    xr auth default my-app
  Set default app + username together:
    xr auth default my-app alice
  Non-interactive fail-fast if no name is supplied:
    xr auth default --no-interactive --output json
";

/// `xr auth apps add` examples.
const APPS_ADD_HELP: &str = "\
Examples:
  Register a new app (text):
    xr auth apps add my-app --client-id ID --client-secret SECRET
  Register with a custom redirect URI:
    xr auth apps add my-app --client-id ID --client-secret SECRET \\
      --redirect-uri https://localhost:8443/callback
  Register, JSON envelope for scripted setup:
    xr auth apps add my-app --client-id ID --client-secret SECRET --output json
";

/// `xr auth apps update` examples.
const APPS_UPDATE_HELP: &str = "\
Examples:
  Rotate the client secret:
    xr auth apps update my-app --client-secret NEW
  Update the redirect URI:
    xr auth apps update my-app --redirect-uri https://localhost:8443/callback
  Clear the stored redirect URI (pass empty string):
    xr auth apps update my-app --redirect-uri \"\"
  Same, JSON envelope:
    xr auth apps update my-app --client-secret NEW --output json
";

/// `xr auth apps remove` — destructive op; advertise non-interactive shape.
const APPS_REMOVE_HELP: &str = "\
Examples:
  Remove a registered app (text):
    xr auth apps remove my-app
  Remove (JSON envelope):
    xr auth apps remove my-app --output json
  Non-interactive removal in CI:
    xr auth apps remove my-app --no-interactive --output json
";

/// `xr auth apps list` — paired text + JSON.
const APPS_LIST_HELP: &str = "\
Examples:
  List registered apps (text):
    xr auth apps list
  List (JSON envelope):
    xr auth apps list --output json
  Quiet + JSON for clean agent consumption:
    xr auth apps list --quiet --output json
";

/// `xr auth apps redirect-uri` parent help.
const APPS_REDIRECT_URI_HELP: &str = "\
Examples:
  Show the effective redirect URI and its source:
    xr auth apps redirect-uri get my-app --output json
  Set the stored redirect URI:
    xr auth apps redirect-uri set my-app https://localhost:8443/callback
  Clear the stored redirect URI (empty value):
    xr auth apps redirect-uri set my-app \"\"
";

/// `xr auth apps redirect-uri get` examples.
const REDIRECT_URI_GET_HELP: &str = "\
Examples:
  Show the effective URI for the default app (text):
    xr auth apps redirect-uri get
  Show for a specific app (JSON envelope):
    xr auth apps redirect-uri get my-app --output json
  Compare env-var override vs stored value:
    REDIRECT_URI=https://example/callback xr auth apps redirect-uri get my-app --output json
";

/// `xr auth apps redirect-uri set` examples.
const REDIRECT_URI_SET_HELP: &str = "\
Examples:
  Set a custom redirect URI:
    xr auth apps redirect-uri set my-app https://localhost:8443/callback
  Same, JSON envelope:
    xr auth apps redirect-uri set my-app https://localhost:8443/callback --output json
  Clear the stored URI (empty string):
    xr auth apps redirect-uri set my-app \"\"
";

/// `xr media upload` examples.
const MEDIA_UPLOAD_HELP: &str = "\
Examples:
  Upload an image (text):
    xr media upload ./photo.png --media-type image/png --category tweet_image
  Upload a video and wait for processing (JSON envelope):
    xr media upload ./clip.mp4 --wait --output json
  Upload using a specific auth method:
    xr media upload ./photo.png --auth oauth2 --output json
  Skip waiting (returns immediately after FINALIZE):
    xr media upload ./clip.mp4 --wait false --output json
";

/// `xr media status` examples.
const MEDIA_STATUS_HELP: &str = "\
Examples:
  Check upload status (text):
    xr media status 1234567890
  Check status (JSON envelope):
    xr media status 1234567890 --output json
  Poll until processing completes:
    xr media status 1234567890 --wait --output json
";

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

Shell completions:
  xr completions bash > ~/.bash_completion.d/xr
  xr completions zsh > ~/.zfunc/_xr
  xr completions fish > ~/.config/fish/completions/xr.fish

Run 'xr --help' to see all available commands."#,
    after_help = ROOT_HELP,
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
    #[arg(
        short = 'v',
        long = "verbose",
        global = true,
        env = "XURL_VERBOSE",
        value_parser = FalseyValueParser::new(),
        num_args = 0..=1,
        default_value_t = false,
        default_missing_value = "true",
        require_equals = false,
    )]
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
    #[arg(long = "app", global = true, env = "XURL_APP")]
    pub app: Option<String>,

    /// Output format. text (default), json, jsonl, ndjson (alias of jsonl),
    /// yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml,
    /// xml) are not supported — xurl emits a JSON envelope with reason
    /// `invalid-args` if requested.
    #[arg(
        long,
        global = true,
        default_value = "text",
        value_enum,
        env = "XURL_OUTPUT"
    )]
    pub output: OutputFormat,

    /// Shorthand for `--output json` (P2 alias).
    #[arg(
        long,
        global = true,
        conflicts_with = "output",
        conflicts_with = "jsonl",
        env = "XURL_JSON",
        value_parser = clap::builder::FalseyValueParser::new(),
        action = clap::ArgAction::SetTrue,
    )]
    pub json: bool,

    /// Shorthand for `--output jsonl` (P2 alias).
    #[arg(
        long,
        global = true,
        conflicts_with = "output",
        conflicts_with = "json",
        env = "XURL_JSONL",
        value_parser = clap::builder::FalseyValueParser::new(),
        action = clap::ArgAction::SetTrue,
    )]
    pub jsonl: bool,

    /// Emit unstyled, compact output. Strips ANSI in text mode; compact (no
    /// pretty-printing) JSON in json/jsonl modes.
    #[arg(
        long,
        global = true,
        env = "XURL_RAW",
        value_parser = FalseyValueParser::new(),
        num_args = 0..=1,
        default_value_t = false,
        default_missing_value = "true",
        require_equals = false,
    )]
    pub raw: bool,

    /// Documented no-op. `xr` writes directly to stdout and never invokes
    /// `$PAGER`; this flag is advertised so agents can pass `--no-pager`
    /// unconditionally without xr rejecting it.
    #[arg(
        long,
        global = true,
        env = "XURL_NO_PAGER",
        value_parser = FalseyValueParser::new(),
        action = clap::ArgAction::SetTrue,
    )]
    pub no_pager: bool,

    /// Suppress all non-essential output (errors still go to stderr)
    #[arg(
        long,
        short = 'q',
        global = true,
        env = "XURL_QUIET",
        value_parser = FalseyValueParser::new(),
        num_args = 0..=1,
        default_value_t = false,
        default_missing_value = "true",
        require_equals = false,
    )]
    pub quiet: bool,

    /// Disable interactive prompts; fail with error instead
    #[arg(
        long,
        global = true,
        env = "XURL_NO_INTERACTIVE",
        value_parser = FalseyValueParser::new(),
        num_args = 0..=1,
        default_value_t = false,
        default_missing_value = "true",
        require_equals = false,
    )]
    pub no_interactive: bool,

    /// Request timeout in seconds
    #[arg(long, global = true, default_value = "30", env = "XURL_TIMEOUT")]
    pub timeout: u64,

    /// Colorize output: auto (TTY-aware), always, or never
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = ColorChoice::Auto,
        env = "XURL_COLOR"
    )]
    pub color: ColorChoice,

    /// Validate inputs and skip the API call (U7).
    ///
    /// Honored by every write op; emits a canonical dry-run envelope on
    /// stdout under `--output json` / `--output jsonl`, or a "Would …" line
    /// under `--output text`. Read ops ignore it.
    #[arg(
        long = "dry-run",
        global = true,
        env = "XURL_DRY_RUN",
        value_parser = FalseyValueParser::new(),
        num_args = 0..=1,
        default_value_t = false,
        default_missing_value = "true",
        require_equals = false,
    )]
    pub dry_run: bool,

    /// Global result-set limit, clamped to 1..=100 (U7).
    ///
    /// Applies to every list-style command. The per-command `-n/--max-results`
    /// flag takes precedence when both are set.
    #[arg(long = "limit", global = true, env = "XURL_LIMIT")]
    pub limit: Option<i32>,

    /// Pagination cursor / `pagination_token` for list endpoints.
    ///
    /// The X API uses cursor-based pagination: each list response carries a
    /// `meta.next_token` field, and the next page is fetched by re-running
    /// the same command with `--cursor <token>` (or `XURL_CURSOR=<token>`).
    /// Threads through to the `pagination_token` query parameter on every
    /// `search`, `timeline`, `mentions`, `bookmarks`, `likes`, `following`,
    /// `followers`, and `dms` invocation.
    #[arg(
        long = "cursor",
        global = true,
        env = "XURL_CURSOR",
        value_name = "TOKEN"
    )]
    pub cursor: Option<String>,

    /// Documented alias for `--cursor`.
    ///
    /// X's API does not offer offset-style pagination (`--page 2` is not
    /// addressable). Passing `--page` returns a canonical
    /// `unsupported-pagination` envelope on stderr suggesting `--cursor`
    /// instead. Exposed so agents trained on offset-pagination conventions
    /// get a structured error rather than a silent no-op.
    #[arg(
        long = "page",
        global = true,
        env = "XURL_PAGE",
        value_name = "N",
        conflicts_with = "cursor"
    )]
    pub page: Option<String>,

    /// Documented alias for `--cursor` (`--after <token>`).
    ///
    /// Threads through to the same `pagination_token` query parameter as
    /// `--cursor`. Exposed so agents that picked up "after-style" pagination
    /// from other CLIs (`gh`, `kubectl`) get a working flag.
    #[arg(
        long = "after",
        global = true,
        env = "XURL_AFTER",
        value_name = "TOKEN",
        conflicts_with = "cursor",
        conflicts_with = "page"
    )]
    pub after: Option<String>,

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
    #[command(after_help = POST_HELP)]
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
    #[command(after_help = REPLY_HELP)]
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
    #[command(after_help = QUOTE_HELP)]
    Quote {
        /// Post ID or URL to quote
        post_id: String,
        /// The quote text
        text: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Delete a post
    #[command(after_help = DELETE_HELP)]
    Delete {
        /// Post ID or URL to delete
        post_id: String,
        /// Skip the confirmation prompt; required under `--no-interactive`
        #[arg(long)]
        force: bool,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Reading ──────────────────────────────────────────────────────
    /// Read a post
    #[command(after_help = READ_HELP)]
    Read {
        /// Post ID or URL to read
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Search recent posts
    #[command(after_help = SEARCH_HELP)]
    Search {
        /// Search query
        query: String,
        /// Number of results (1-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── User Info ────────────────────────────────────────────────────
    /// Show the authenticated user's profile
    #[command(after_help = WHOAMI_HELP)]
    Whoami {
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Look up a user by username
    #[command(after_help = USER_HELP)]
    User {
        /// Username to look up
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Timeline & Mentions ──────────────────────────────────────────
    /// Show your home timeline
    #[command(after_help = TIMELINE_HELP)]
    Timeline {
        /// Number of results (1-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Show your recent mentions
    #[command(after_help = MENTIONS_HELP)]
    Mentions {
        /// Number of results (5-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Engagement ───────────────────────────────────────────────────
    /// Like a post
    #[command(after_help = LIKE_HELP)]
    Like {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Unlike a post
    #[command(after_help = UNLIKE_HELP)]
    Unlike {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Repost a post
    #[command(after_help = REPOST_HELP)]
    Repost {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Undo a repost
    #[command(after_help = UNREPOST_HELP)]
    Unrepost {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Bookmark a post
    #[command(after_help = BOOKMARK_HELP)]
    Bookmark {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Remove a bookmark
    #[command(after_help = UNBOOKMARK_HELP)]
    Unbookmark {
        /// Post ID or URL
        post_id: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List your bookmarks
    #[command(after_help = BOOKMARKS_HELP)]
    Bookmarks {
        /// Number of results (1-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List your liked posts
    #[command(after_help = LIKES_HELP)]
    Likes {
        /// Number of results (1-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Social Graph ─────────────────────────────────────────────────
    /// Follow a user
    #[command(after_help = FOLLOW_HELP)]
    Follow {
        /// Username to follow
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Unfollow a user
    #[command(after_help = UNFOLLOW_HELP)]
    Unfollow {
        /// Username to unfollow
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List users you follow
    #[command(after_help = FOLLOWING_HELP)]
    Following {
        /// Number of results (1-1000). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        /// Username to list following for (default: you)
        #[arg(long = "of")]
        of: Option<String>,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List your followers
    #[command(after_help = FOLLOWERS_HELP)]
    Followers {
        /// Number of results (1-1000). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        /// Username to list followers for (default: you)
        #[arg(long = "of")]
        of: Option<String>,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Mute a user
    #[command(after_help = MUTE_HELP)]
    Mute {
        /// Username to mute
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Unmute a user
    #[command(after_help = UNMUTE_HELP)]
    Unmute {
        /// Username to unmute
        #[arg(value_name = "USERNAME")]
        target_username: String,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Usage ─────────────────────────────────────────────────────────
    /// Show API usage (tweet caps, daily breakdown)
    #[command(after_help = USAGE_HELP)]
    Usage {
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Direct Messages ──────────────────────────────────────────────
    /// Send a direct message
    #[command(after_help = DM_HELP)]
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
    #[command(after_help = DMS_HELP)]
    Dms {
        /// Number of results (1-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Auth ─────────────────────────────────────────────────────────
    /// Authentication management
    #[command(after_help = AUTH_HELP)]
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },

    // ── Media ────────────────────────────────────────────────────────
    /// Media upload operations
    #[command(after_help = MEDIA_HELP)]
    Media {
        #[command(subcommand)]
        command: MediaCommands,
    },

    // ── Skill bundle ─────────────────────────────────────────────────
    /// Install or manage the xurl-rs skill bundle
    ///
    /// Namespace for bundle operations. `xr skill install <host>` shallow-clones
    /// the xurl-rs repository into a host's canonical skills directory so the
    /// bundled `AGENTS.md` becomes discoverable to local agents.
    #[command(after_help = "Examples:
  xr skill install claude_code                 # install bundle to Claude Code
  xr skill install claude_code --dry-run       # print the resolved git command without spawning
  xr skill install --all                       # install across every known host
  xr skill install codex --output json         # JSON envelope for agent consumption
  xr skill install --all --dry-run --output json  # multi-host dry-run envelope")]
    Skill {
        #[command(subcommand)]
        cmd: SkillCmd,
    },

    // ── Meta ─────────────────────────────────────────────────────────
    /// Show JSON Schema for a command's response type
    #[command(after_help = SCHEMA_HELP)]
    Schema {
        /// Command name to get the schema for (e.g. "post", "whoami", "envelope")
        command: Option<String>,
        /// List all commands and their response types
        #[arg(long)]
        list: bool,
        /// Output all schemas as a single JSON document
        #[arg(long)]
        all: bool,
        /// Output the canonical agent-native output envelope schema
        #[arg(long)]
        envelope: bool,
    },

    /// Generate shell completion script
    #[command(after_help = COMPLETIONS_HELP)]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Show xurl version information
    #[command(after_help = VERSION_HELP)]
    Version,

    /// Print a curated gallery of invocation examples grouped by use case
    #[command(after_help = EXAMPLES_HELP)]
    Examples,

    /// Validate a JSON document against a bundled response schema.
    ///
    /// Reads JSON from stdin (when no file argument is given or `-` is
    /// passed) or from the supplied file, deserializes it into the
    /// requested typed response, and emits an `ok` / `validation-failed`
    /// envelope. Use `--schema` to pin a specific shape (e.g. `tweet`,
    /// `user`); without it the command auto-detects from the top-level
    /// shape.
    #[command(after_help = VALIDATE_HELP)]
    Validate {
        /// File path to read JSON from. Pass `-` or omit to read from stdin.
        #[arg(value_name = "FILE")]
        file: Option<String>,

        /// Schema name to validate against (`tweet`, `tweets`, `user`,
        /// `users`, `dm`, `dms`, `usage`, `envelope`). Omit for auto-detection.
        #[arg(long = "schema", value_name = "NAME")]
        schema: Option<String>,
    },
}

/// `skill` subcommand variants.
#[derive(Subcommand, Debug)]
pub enum SkillCmd {
    /// Install the skill bundle into a host's canonical skills directory.
    ///
    /// Shallow-clones the xurl-rs repository so the bundled `AGENTS.md` is
    /// discoverable to local agents. The destination is taken from the
    /// build-generated host map (`src/skill_install/skill.json`).
    #[command(after_help = "Examples:
  xr skill install claude_code                     # install bundle to Claude Code
  xr skill install claude_code --dry-run           # print the resolved git command without spawning
  xr skill install --all                           # install across every known host
  xr skill install codex --output json             # JSON envelope for agent consumption
  xr skill install --all --dry-run --output json   # multi-host dry-run envelope")]
    Install {
        /// Target host (e.g. claude_code, codex, cursor). Required unless `--all`.
        host: Option<SkillHost>,

        /// Install into every known host in one invocation.
        #[arg(long, conflicts_with = "host")]
        all: bool,

        /// Print the resolved git command without spawning.
        #[arg(long)]
        dry_run: bool,
    },

    /// Refresh an existing skill-bundle install in place.
    ///
    /// Removes the current destination and re-runs the install pipeline so
    /// the bundle picks up upstream changes. Hardening surface is identical
    /// to `install`. The envelope's `action` is `"skill-update"` so agents
    /// can distinguish from a first-time install.
    #[command(after_help = "Examples:
  xr skill update claude_code                      # refresh Claude Code's xurl-rs bundle
  xr skill update claude_code --dry-run            # show the resolved plan without touching disk
  xr skill update --all                            # refresh every known host
  xr skill update codex --output json              # JSON envelope for agent consumption")]
    Update {
        /// Target host (e.g. claude_code, codex, cursor). Required unless `--all`.
        host: Option<SkillHost>,

        /// Update every known host in one invocation.
        #[arg(long, conflicts_with = "host")]
        all: bool,

        /// Print the resolved plan without removing or cloning.
        #[arg(long)]
        dry_run: bool,
    },
}

impl Cli {
    /// Resolves the effective output format after applying `--json` /
    /// `--jsonl` aliases.
    ///
    /// `--jsonl` wins over `--json` if both were set (they conflict via
    /// clap, so at most one survives parsing); either alias overrides
    /// `--output`.
    #[must_use]
    pub fn effective_output(&self) -> OutputFormat {
        if self.jsonl {
            OutputFormat::Jsonl
        } else if self.json {
            OutputFormat::Json
        } else {
            self.output.clone()
        }
    }
}

/// Common flags shared by shortcut commands.
///
/// `--verbose` is intentionally absent here; it lives on the root [`Cli`]
/// as a global flag with `XURL_VERBOSE` env backing, so subcommands inherit
/// it without local duplication.
#[derive(clap::Args, Debug, Clone)]
pub struct CommonFlags {
    /// Authentication type (oauth1, oauth2, app)
    #[arg(long = "auth")]
    pub auth_type: Option<String>,

    /// `OAuth2` username to act as
    #[arg(short = 'u', long = "username")]
    pub username: Option<String>,

    /// Add X-B3-Flags trace header
    #[arg(short = 't', long = "trace")]
    pub trace: bool,
}

impl CommonFlags {
    /// Converts to `CallOptions` for shortcut methods.
    ///
    /// `verbose` and `timeout_secs` are sourced from the root [`Cli`] global
    /// flags rather than per-subcommand, so the caller threads them through
    /// here.
    pub fn to_call_options(&self, verbose: bool, timeout_secs: u64) -> crate::api::CallOptions {
        self.to_call_options_with_cursor(verbose, timeout_secs, None)
    }

    /// Like [`to_call_options`] but with an explicit cursor / pagination
    /// token. The runner threads the global `--cursor` (or `--after` /
    /// `XURL_CURSOR` / `XURL_AFTER`) here so list shortcuts can append it to
    /// their URL.
    ///
    /// [`to_call_options`]: Self::to_call_options
    pub fn to_call_options_with_cursor(
        &self,
        verbose: bool,
        timeout_secs: u64,
        cursor: Option<&str>,
    ) -> crate::api::CallOptions {
        crate::api::CallOptions {
            auth_type: self.auth_type.clone().unwrap_or_default(),
            username: self.username.clone().unwrap_or_default(),
            no_auth: false,
            verbose,
            trace: self.trace,
            timeout_secs,
            pagination_token: cursor.unwrap_or_default().to_string(),
        }
    }
}

/// Auth subcommands.
#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Configure `OAuth2` authentication
    #[command(after_help = AUTH_OAUTH2_HELP)]
    Oauth2 {
        /// Enable manual two-step flow for headless machines (SSH, containers)
        ///
        /// Auto-engages when stdout is not a TTY (piped runs, CI) so headless
        /// callers receive the auth URL instead of a silent `open::that` spawn
        /// that nothing will see. The `XURL_NO_BROWSER` env var sets this by
        /// default on machines that should never attempt to open a browser.
        /// Honours `1` / `true` / `yes` / `on` as truthy and `0` / `false` /
        /// `no` / `off` / empty as falsey (the same `FalseyValueParser` shape
        /// used by every other env-backed boolean flag).
        #[arg(
            long,
            env = "XURL_NO_BROWSER",
            value_parser = FalseyValueParser::new(),
            num_args = 0..=1,
            default_value_t = false,
            default_missing_value = "true",
            require_equals = false,
        )]
        no_browser: bool,
        /// Step number: 1 (generate auth URL) or 2 (complete exchange)
        #[arg(long, requires = "no_browser", value_parser = clap::value_parser!(u8).range(1..=2))]
        step: Option<u8>,
        /// Redirect URL from browser (step 2). Use '-' to read from stdin (recommended on shared machines)
        #[arg(long = "auth-url", requires = "step")]
        auth_url: Option<String>,
        /// Username to label the saved token (bypasses `/2/users/me` lookup when supplied)
        #[arg(value_name = "USERNAME")]
        username: Option<String>,
    },
    /// Configure `OAuth1` authentication
    #[command(after_help = AUTH_OAUTH1_HELP)]
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
    #[command(after_help = AUTH_APP_HELP)]
    App {
        /// Bearer token
        #[arg(long = "bearer-token")]
        bearer_token: String,
    },
    /// Show authentication status
    #[command(after_help = AUTH_STATUS_HELP)]
    Status,
    /// Clear authentication tokens
    #[command(after_help = AUTH_CLEAR_HELP)]
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
        /// Skip the confirmation prompt; required under `--no-interactive`
        #[arg(long)]
        force: bool,
    },
    /// Manage registered X API apps
    #[command(after_help = AUTH_APPS_HELP)]
    Apps {
        #[command(subcommand)]
        command: AppCommands,
    },
    /// Set default app and/or user
    #[command(after_help = AUTH_DEFAULT_HELP)]
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
    #[command(after_help = APPS_ADD_HELP)]
    Add {
        /// App name
        name: String,
        /// `OAuth2` client ID
        #[arg(long = "client-id")]
        client_id: String,
        /// `OAuth2` client secret
        #[arg(long = "client-secret")]
        client_secret: String,
        /// `OAuth2` redirect URI (https or http on loopback)
        #[arg(long = "redirect-uri")]
        redirect_uri: Option<String>,
    },
    /// Update credentials for an existing app
    #[command(after_help = APPS_UPDATE_HELP)]
    Update {
        /// App name
        name: String,
        /// `OAuth2` client ID
        #[arg(long = "client-id")]
        client_id: Option<String>,
        /// `OAuth2` client secret
        #[arg(long = "client-secret")]
        client_secret: Option<String>,
        /// `OAuth2` redirect URI (https or http on loopback); empty string clears
        #[arg(long = "redirect-uri")]
        redirect_uri: Option<String>,
    },
    /// Remove a registered app
    #[command(after_help = APPS_REMOVE_HELP)]
    Remove {
        /// App name
        name: String,
        /// Skip the confirmation prompt; required under `--no-interactive`
        #[arg(long)]
        force: bool,
    },
    /// List registered apps
    #[command(after_help = APPS_LIST_HELP)]
    List,
    /// Inspect or set the stored `OAuth2` redirect URI for an app
    #[command(after_help = APPS_REDIRECT_URI_HELP)]
    RedirectUri {
        #[command(subcommand)]
        command: RedirectUriCommands,
    },
}

/// `auth apps redirect-uri` subcommands.
#[derive(Subcommand, Debug)]
pub enum RedirectUriCommands {
    /// Show the effective redirect URI, its source, and the stored value
    #[command(after_help = REDIRECT_URI_GET_HELP)]
    Get {
        /// App name (defaults to the configured default app)
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },
    /// Set the stored redirect URI for an app (empty string clears)
    #[command(after_help = REDIRECT_URI_SET_HELP)]
    Set {
        /// App name
        #[arg(value_name = "NAME")]
        name: String,
        /// Redirect URI (https or http on loopback)
        #[arg(value_name = "URI")]
        uri: String,
    },
}

/// Media subcommands.
#[derive(Subcommand, Debug)]
pub enum MediaCommands {
    /// Upload media file
    #[command(after_help = MEDIA_UPLOAD_HELP)]
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
        /// Trace header
        #[arg(short = 't', long = "trace")]
        trace: bool,
        /// Request headers
        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
    },
    /// Check media upload status
    #[command(after_help = MEDIA_STATUS_HELP)]
    Status {
        /// Media ID
        media_id: String,
        /// Authentication type
        #[arg(long = "auth")]
        auth_type: Option<String>,
        /// Username
        #[arg(short = 'u', long = "username")]
        username: Option<String>,
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
