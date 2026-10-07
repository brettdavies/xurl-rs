//! CLI definition — clap derive with subcommands.
//!
//! Mirrors the Go cobra command tree: root (raw mode) + shortcuts +
//! auth/media/webhook/version subcommands.

mod classify;
pub mod commands;
mod failure;
pub mod hints;
pub mod runner;
pub(crate) mod shutdown;

pub use runner::{run, run_argv, run_with_store_path};

use clap::builder::FalseyValueParser;
use clap::{Parser, Subcommand, ValueEnum};

pub mod env;
pub mod envelope;
pub mod family_help;
pub mod output;
#[cfg(test)]
mod parse_tests;
mod reparse;
pub mod skill_install;

pub use output::OutputFormat;
use skill_install::KNOWN_HOSTS;
pub use skill_install::SkillHost;
use xdk::api::VideoCategory;

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
/// `cli-env-vars-must-appear-in-help-2026-04-20.md`). The skill hosts'
/// config- and base-directory lines are generated from the skill manifest by
/// `build.rs`.
const ROOT_HELP: &str = concat!(
    "\
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
  XURL_WAIT_ON_RATE_LIMIT  Wait out a rate limit and retry once (same as --wait-on-rate-limit)
  XURL_RATE_LIMIT_MAX_WAIT Longest rate-limit wait in seconds (same as --rate-limit-max-wait)
  XURL_COLOR             Color control: auto, always, never (same as --color)
  XURL_VERBOSE           Request/response lines and legacy-vocabulary notes (same as -v/--verbose)
  XURL_APP               Override default app (same as --app)
  XURL_JSON              Shorthand for XURL_OUTPUT=json (same as --json)
  XURL_JSONL             Shorthand for XURL_OUTPUT=jsonl (same as --jsonl)
  XURL_NO_BROWSER        Skip browser-open on `auth oauth2` (same as --no-browser)
  XURL_TOKEN_STORE       Token-store file to use instead of ~/.xurl (OAuth2 pending state sits beside it)
  XURL_SKILL_HOME        Directory ~ means in skill install destinations; wins over HOME
",
    include_str!(concat!(env!("OUT_DIR"), "/skill_env_help.txt")),
    "  XURL_BEARER_TOKEN      App-only bearer token; wins over the bearer stored for the active app
  CLIENT_ID              OAuth2 client ID; wins over the active app's stored value
  CLIENT_SECRET          OAuth2 client secret; wins over the active app's stored value
  REDIRECT_URI           OAuth2 redirect URI override for the active app
  AUTH_URL               OAuth2 authorization endpoint override
  TOKEN_URL              OAuth2 token exchange endpoint override
  API_BASE_URL           API origin every request is built against
  INFO_URL               User-info endpoint override; derived from API_BASE_URL when unset

Flags override env vars when both are set. NO_COLOR=1 always wins over
--color/XURL_COLOR (https://no-color.org). --no-pager is a documented
no-op so agents can pass it unconditionally; xr never invokes $PAGER.

INPUT FROM STDIN:
  Subcommands that accept JSON input (currently `xr validate`) read from
  stdin when no file argument is given or when `-` is passed as the path.
  This matches the standard CLI convention for piping data:
    cat post.json | xr validate --schema post --output json
    cat post.json | xr validate - --schema post --output json

EXIT CODES:
  0    success
  1    general error
  2    invalid arguments or auth-method mismatch
  3    rate-limited (HTTP 429)
  4    not found (HTTP 404)
  5    network error
  77   authentication required

TTY behavior:
  When stdout is not a TTY, color output is auto-stripped (unless
  --color always is set) and human-only banners are suppressed, so piping
  to jaq or redirecting to a file produces clean machine-readable output
  without any extra flags.
"
);

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
    xr reply 1585341984679469056 \"Congrats!\"
  Reply with JSON output for scripting:
    xr reply 1585341984679469056 \"Congrats!\" --output json
  Reply to a post URL (xr accepts either):
    xr reply https://x.com/elonmusk/status/1585341984679469056 \"Nice thread.\"
  Reply with a media attachment:
    xr reply 1585341984679469056 \"Here's a photo\" --media-id 222 --output json
";

/// `xr quote` examples — paired text + JSON.
const QUOTE_HELP: &str = "\
Examples:
  Quote-post by ID:
    xr quote 1585341984679469056 \"Worth a read.\"
  Quote-post with JSON envelope:
    xr quote 1585341984679469056 \"Worth a read.\" --output json
  Quote-post from a URL:
    xr quote https://x.com/elonmusk/status/1585341984679469056 \"Thread.\"
";

/// `xr delete` examples — destructive op; advertise non-interactive shape.
const DELETE_HELP: &str = "\
Examples:
  Delete a post by ID (text):
    xr delete 1585341984679469056
  Delete with JSON envelope:
    xr delete 1585341984679469056 --output json
  Delete in a non-interactive context (CI, agent); --force confirms it:
    xr delete 1585341984679469056 --force --no-interactive --output json
";

/// `xr read` examples — paired text + JSON, plus a pipe-to-jaq invocation.
const READ_HELP: &str = "\
Examples:
  Read a post (text):
    xr read 1585341984679469056
  Read a post (JSON):
    xr read 1585341984679469056 --output json
  Read a post URL:
    xr read https://x.com/elonmusk/status/1585341984679469056 --output json
  Extract a single field via jaq:
    xr read 1585341984679469056 --output json | jaq '.data.text'
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
  The id of each result, one per line:
    xr search \"rustlang\" --output json | jaq -r '.data[]?.id'
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
    xr user elonmusk
  Same, as JSON:
    xr user elonmusk --output json
  Use the @ prefix (xr accepts either):
    xr user @elonmusk --output json
";

/// `xr timeline` examples — paired text + JSON, plus JSONL pipeline.
const TIMELINE_HELP: &str = "\
Examples:
  Home timeline (last 10, text):
    xr timeline
  Home timeline (50 results, JSON envelope):
    xr timeline -n 50 --output json
  The id of each post, one per line:
    xr timeline -n 100 --output json | jaq -r '.data[]?.id'
";

/// `xr mentions` examples — paired text + JSON.
const MENTIONS_HELP: &str = "\
Examples:
  Your mentions (text):
    xr mentions
  Last 25, JSON envelope:
    xr mentions -n 25 --output json
  The id of each mention, one per line:
    xr mentions -n 100 --output json | jaq -r '.data[]?.id'
";

/// `xr like` examples — paired text + JSON.
const LIKE_HELP: &str = "\
Examples:
  Like a post (text):
    xr like 1585341984679469056
  Like a post (JSON envelope):
    xr like 1585341984679469056 --output json
  Idempotent: re-liking is a server-side no-op.
";

/// `xr unlike` examples — paired text + JSON.
const UNLIKE_HELP: &str = "\
Examples:
  Unlike a post (text):
    xr unlike 1585341984679469056
  Unlike (JSON envelope):
    xr unlike 1585341984679469056 --output json
";

/// `xr repost` examples — paired text + JSON.
const REPOST_HELP: &str = "\
Examples:
  Repost a post (text):
    xr repost 1585341984679469056
  Repost (JSON envelope):
    xr repost 1585341984679469056 --output json
";

/// `xr unrepost` examples — paired text + JSON.
const UNREPOST_HELP: &str = "\
Examples:
  Undo a repost (text):
    xr unrepost 1585341984679469056
  Undo a repost (JSON envelope):
    xr unrepost 1585341984679469056 --output json
";

/// `xr bookmark` examples — paired text + JSON.
const BOOKMARK_HELP: &str = "\
Examples:
  Bookmark a post (text):
    xr bookmark 1585341984679469056
  Bookmark (JSON envelope):
    xr bookmark 1585341984679469056 --output json
";

/// `xr unbookmark` examples — paired text + JSON.
const UNBOOKMARK_HELP: &str = "\
Examples:
  Remove a bookmark (text):
    xr unbookmark 1585341984679469056
  Remove a bookmark (JSON envelope):
    xr unbookmark 1585341984679469056 --output json
";

/// `xr bookmarks` examples — paired text + JSON, JSONL pipeline.
const BOOKMARKS_HELP: &str = "\
Examples:
  List your bookmarks (text):
    xr bookmarks
  100 results, JSON envelope:
    xr bookmarks -n 100 --output json
  The id of each bookmark, one per line:
    xr bookmarks -n 100 --output json | jaq -r '.data[]?.id'
";

/// `xr likes` examples — paired text + JSON, JSONL pipeline.
const LIKES_HELP: &str = "\
Examples:
  List your liked posts (text):
    xr likes
  100 results, JSON envelope:
    xr likes -n 100 --output json
  The id of each liked post, one per line:
    xr likes -n 100 --output json | jaq -r '.data[]?.id'
";

/// `xr follow` examples — paired text + JSON.
const FOLLOW_HELP: &str = "\
Examples:
  Follow a user (text):
    xr follow @elonmusk
  Follow (JSON envelope):
    xr follow @elonmusk --output json
  Without the @ prefix:
    xr follow elonmusk --output json
";

/// `xr unfollow` examples — paired text + JSON.
const UNFOLLOW_HELP: &str = "\
Examples:
  Unfollow a user (text):
    xr unfollow @elonmusk
  Unfollow (JSON envelope):
    xr unfollow @elonmusk --output json
";

/// `xr following` examples — paired text + JSON, `--of` for another user.
const FOLLOWING_HELP: &str = "\
Examples:
  Users you follow (text):
    xr following
  100 results, JSON envelope:
    xr following -n 100 --output json
  Who someone else follows:
    xr following --of @elonmusk --output json
";

/// `xr followers` examples — paired text + JSON, `--of` for another user.
const FOLLOWERS_HELP: &str = "\
Examples:
  Your followers (text):
    xr followers
  100 results, JSON envelope:
    xr followers -n 100 --output json
  Someone else's followers:
    xr followers --of @elonmusk --output json
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

/// `xr muted` examples — paired text + JSON.
const MUTED_HELP: &str = "\
Examples:
  List the users you have muted (text):
    xr muted
  First 100, as JSON Lines:
    xr muted -n 100 --output jsonl
";

/// `xr block` examples — paired text + JSON.
const BLOCK_HELP: &str = "\
Examples:
  Block a user (text):
    xr block @spammer
  Block (JSON envelope):
    xr block @spammer --output json
";

/// `xr unblock` examples — paired text + JSON.
const UNBLOCK_HELP: &str = "\
Examples:
  Unblock a user (text):
    xr unblock @spammer
  Unblock (JSON envelope):
    xr unblock @spammer --output json
";

/// `xr blocked` examples — paired text + JSON.
const BLOCKED_HELP: &str = "\
Examples:
  List the users you have blocked (text):
    xr blocked
  First 100, as JSON Lines:
    xr blocked -n 100 --output jsonl
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
  The id of each event, one per line:
    xr dms -n 100 --output json | jaq -r '.data[]?.id'
";

/// `xr auth` parent help — points to subcommands.
const AUTH_HELP: &str = "\
Examples:
  Browser-based OAuth2 (default):
    xr auth oauth2
  Headless OAuth2 (servers, containers):
    xr auth oauth2 --no-browser --step 1
  Bearer token for read-only / search, piped so it never reaches argv:
    op read 'op://<vault>/<item>/bearer_token' | xr auth app --bearer-token-file -
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
    xr media status 1585341984679469056 --output json
  Describe an uploaded image for screen readers:
    xr media alt-text 1585341984679469056 \"A dog asleep on a beach towel\"
  Add an English subtitle track to an uploaded video:
    xr media subtitles add 1585341984679469056 1585341984679469057 --language en --name English
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
    cat post.json | xr validate --output json
  Same, written as `xr validate -` for explicit stdin:
    cat post.json | xr validate - --schema post --output json
  Validate a file against a specific schema:
    xr validate posts.json --schema posts --output json
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
  Sign in with only the scopes a task needs (offline.access is always added):
    xr auth oauth2 --scopes tweet.read,users.read
  Same, headless:
    xr auth oauth2 --no-browser --step 1 --scopes tweet.read,users.read --output json
";

/// `xr auth oauth1` — non-interactive OAuth1 setup.
const AUTH_OAUTH1_HELP: &str = "\
Examples:
  Configure OAuth1 with one secret piped from a vault and the others read
  from files, so none reaches argv (stdin carries one value):
    op read 'op://<vault>/<item>/token_secret' | xr auth oauth1 --consumer-key CK \\
      --consumer-secret-file consumer-secret.txt \\
      --access-token-file access-token.txt --token-secret-file -
  Every secret from a file, with JSON envelope for scripted setup:
    xr auth oauth1 --consumer-key CK \\
      --consumer-secret-file consumer-secret.txt \\
      --access-token-file access-token.txt \\
      --token-secret-file token-secret.txt --output json
";

/// `xr auth app` — bearer-token configuration.
const AUTH_APP_HELP: &str = "\
Examples:
  Store the bearer token, piped from a vault so it never reaches argv:
    op read 'op://<vault>/<item>/bearer_token' | xr auth app --bearer-token-file -
  Same from an env var, JSON envelope:
    printenv XURL_BEARER_TOKEN | xr auth app --bearer-token-file - --output json
  Read it from a file:
    xr auth app --bearer-token-file bearer-token.txt
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
  Non-interactive (CI, agent); --force confirms it:
    xr auth clear --all --force --no-interactive --output json
";

/// `xr auth apps` parent help — points to subcommands.
const AUTH_APPS_HELP: &str = "\
Examples:
  Register a new app, with the secret piped from a vault so it never reaches argv:
    op read 'op://<vault>/<item>/client_secret' | xr auth apps add my-app --client-id ID --client-secret-file -
  List registered apps (JSON):
    xr auth apps list --output json
  Rotate the secret of an existing app from a file:
    xr auth apps update my-app --client-secret-file client-secret.txt
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
  Register a new app, with the secret piped from a vault so it never reaches argv:
    op read 'op://<vault>/<item>/client_secret' | xr auth apps add my-app --client-id ID --client-secret-file -
  Read the secret from a file:
    xr auth apps add my-app --client-id ID --client-secret-file client-secret.txt
  Register with a custom redirect URI:
    xr auth apps add my-app --client-id ID --client-secret-file client-secret.txt \\
      --redirect-uri https://localhost:8443/callback
  Register, JSON envelope for scripted setup:
    op read 'op://<vault>/<item>/client_secret' | xr auth apps add my-app --client-id ID --client-secret-file - --output json
";

/// `xr auth apps update` examples.
const APPS_UPDATE_HELP: &str = "\
Examples:
  Rotate the client secret, piped from a vault so it never reaches argv:
    op read 'op://<vault>/<item>/client_secret' | xr auth apps update my-app --client-secret-file -
  Update the redirect URI:
    xr auth apps update my-app --redirect-uri https://localhost:8443/callback
  Clear the stored redirect URI (pass empty string):
    xr auth apps update my-app --redirect-uri \"\"
  Rotate from a file, JSON envelope:
    xr auth apps update my-app --client-secret-file client-secret.txt --output json
";

/// `xr auth apps remove` — destructive op; advertise non-interactive shape.
const APPS_REMOVE_HELP: &str = "\
Examples:
  Remove a registered app (text):
    xr auth apps remove my-app
  Remove (JSON envelope):
    xr auth apps remove my-app --output json
  Non-interactive removal in CI; --force confirms it:
    xr auth apps remove my-app --force --no-interactive --output json
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
  Upload a video and wait for processing, up to 60 seconds (JSON envelope):
    xr media upload ./clip.mp4 --wait --output json
  Wait up to five minutes for a long video:
    xr media upload ./clip.mp4 --wait=300 --output json
  Upload using a specific auth method:
    xr media upload ./photo.png --auth oauth2 --output json
  Skip waiting (returns immediately after FINALIZE):
    xr media upload ./clip.mp4 --wait=false --output json
";

/// `xr media status` examples.
const MEDIA_STATUS_HELP: &str = "\
Examples:
  Check upload status (text):
    xr media status 1585341984679469056
  Check status (JSON envelope):
    xr media status 1585341984679469056 --output json
  Poll until processing completes, up to 60 seconds:
    xr media status 1585341984679469056 --wait --output json
  Resume a wait that timed out, for up to two minutes:
    xr media status 1585341984679469056 --wait=120 --output json
";

/// `xr media alt-text` examples.
const MEDIA_ALT_TEXT_HELP: &str = "\
Examples:
  Set the alt text of an uploaded image (text):
    xr media alt-text 1585341984679469056 \"A dog asleep on a beach towel\"
  Same, JSON envelope:
    xr media alt-text 1585341984679469056 \"A dog asleep on a beach towel\" --output json
  Upload, describe, then post the image:
    xr media upload ./dog.png --media-type image/png --category tweet_image
    xr media alt-text <media_id> \"A dog asleep on a beach towel\"
    xr post \"Beach day\" --media-id <media_id>
";

/// Auth-enabled curl-like interface for the X API.
#[derive(Parser, Debug)]
#[command(
    name = "xr",
    about = "Auth enabled curl-like interface for the X API",
    long_about = r#"A command-line tool for making authenticated requests to the X API.

Shortcut commands (agent-friendly):
  xr post "Hello world!"                        Post to X
  xr reply 1585341984679469056 "Nice!"                   Reply to a post
  xr read 1585341984679469056                             Read a post
  xr search "golang" -n 20                       Search posts
  xr whoami                                      Show your profile
  xr like 1585341984679469056                             Like a post
  xr repost 1585341984679469056                           Repost
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
  xr auth apps add my-app --client-id ... --client-secret-file ...
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

    /// Print request and response lines, and a note for each key X sent in its legacy post
    /// vocabulary
    #[arg(
        short = 'v',
        long = "verbose",
        global = true,
        env = "XURL_VERBOSE",
        value_parser = FalseyValueParser::new(),
        num_args = 0..=1,
        default_value_t = false,
        default_missing_value = "true",
        require_equals = true,
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
    /// yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is a usage
    /// error at exit 2.
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

    /// Emit unstyled, compact output. Strips ANSI in text mode; prints
    /// `json` output on one line, as `jsonl` and `ndjson` always are.
    #[arg(
        long,
        global = true,
        env = "XURL_RAW",
        value_parser = FalseyValueParser::new(),
        num_args = 0..=1,
        default_value_t = false,
        default_missing_value = "true",
        require_equals = true,
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
        require_equals = true,
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
        require_equals = true,
    )]
    pub no_interactive: bool,

    /// Request timeout in seconds
    #[arg(long, global = true, default_value = "30", env = "XURL_TIMEOUT")]
    pub timeout: u64,

    /// Wait out a rate limit and retry once, instead of failing with `rate-limited`
    ///
    /// Applies when X answers 429 and that response names its reset, and the
    /// wait fits `--rate-limit-max-wait`. A longer wait, or a 429 that names
    /// no reset, fails at once as it does without this flag, and the second
    /// response is final whatever it is.
    #[arg(
        long,
        global = true,
        env = "XURL_WAIT_ON_RATE_LIMIT",
        value_parser = FalseyValueParser::new(),
        num_args = 0..=1,
        default_value_t = false,
        default_missing_value = "true",
        require_equals = true,
    )]
    pub wait_on_rate_limit: bool,

    /// Longest `--wait-on-rate-limit` waits before its retry, in seconds
    #[arg(
        long,
        global = true,
        value_name = "SECS",
        default_value = "60",
        env = "XURL_RATE_LIMIT_MAX_WAIT"
    )]
    pub rate_limit_max_wait: u64,

    /// Colorize output: auto (TTY-aware), always, or never
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = ColorChoice::Auto,
        env = "XURL_COLOR"
    )]
    pub color: ColorChoice,

    /// Validate inputs and skip the API call.
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
        require_equals = true,
    )]
    pub dry_run: bool,

    /// Global result-set limit, clamped to 1..=100.
    ///
    /// Applies to `search`, `timeline`, `mentions`, `bookmarks`, `likes`,
    /// `following`, `followers`, `muted`, `blocked`, and `dms`; other commands
    /// ignore it. The per-command `-n/--max-results` flag takes precedence when
    /// both are set.
    #[arg(long = "limit", global = true, env = "XURL_LIMIT")]
    pub limit: Option<i32>,

    /// Pagination cursor / `pagination_token` for list endpoints.
    ///
    /// The X API uses cursor-based pagination: each list response carries a
    /// `meta.next_token` field, and the next page is fetched by re-running
    /// the same command with `--cursor <token>` (or `XURL_CURSOR=<token>`).
    /// Threads through to the `pagination_token` query parameter on every
    /// `search`, `timeline`, `mentions`, `bookmarks`, `likes`, `following`,
    /// `followers`, `muted`, `blocked`, and `dms` invocation; other commands
    /// ignore it.
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
#[non_exhaustive]
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
        /// Shortcut flags shared with every other shortcut command.
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
        /// Shortcut flags shared with every other shortcut command.
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
        /// Shortcut flags shared with every other shortcut command.
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
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Reading ──────────────────────────────────────────────────────
    /// Read a post
    #[command(after_help = READ_HELP)]
    Read {
        /// Post ID or URL to read
        post_id: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Search recent posts
    #[command(after_help = SEARCH_HELP)]
    Search {
        /// Search query
        query: String,
        /// Number of results (10-100; a lower value is raised to 10, X's
        /// minimum). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── User Info ────────────────────────────────────────────────────
    /// Show the authenticated user's profile
    #[command(after_help = WHOAMI_HELP)]
    Whoami {
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Look up a user by username
    #[command(after_help = USER_HELP)]
    User {
        /// Username to look up
        #[arg(value_name = "USERNAME")]
        target_username: String,
        /// Shortcut flags shared with every other shortcut command.
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
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Show your recent mentions
    #[command(after_help = MENTIONS_HELP)]
    Mentions {
        /// Number of results (5-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Engagement ───────────────────────────────────────────────────
    /// Like a post
    #[command(after_help = LIKE_HELP)]
    Like {
        /// Post ID or URL
        post_id: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Unlike a post
    #[command(after_help = UNLIKE_HELP)]
    Unlike {
        /// Post ID or URL
        post_id: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Repost a post
    #[command(after_help = REPOST_HELP)]
    Repost {
        /// Post ID or URL
        post_id: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Undo a repost
    #[command(after_help = UNREPOST_HELP)]
    Unrepost {
        /// Post ID or URL
        post_id: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Bookmark a post
    #[command(after_help = BOOKMARK_HELP)]
    Bookmark {
        /// Post ID or URL
        post_id: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Remove a bookmark
    #[command(after_help = UNBOOKMARK_HELP)]
    Unbookmark {
        /// Post ID or URL
        post_id: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List your bookmarks
    #[command(after_help = BOOKMARKS_HELP)]
    Bookmarks {
        /// Number of results (1-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List your liked posts
    #[command(after_help = LIKES_HELP)]
    Likes {
        /// Number of results (5-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        /// Shortcut flags shared with every other shortcut command.
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
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Unfollow a user
    #[command(after_help = UNFOLLOW_HELP)]
    Unfollow {
        /// Username to unfollow
        #[arg(value_name = "USERNAME")]
        target_username: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List users you follow
    #[command(after_help = FOLLOWING_HELP)]
    Following {
        /// Number of results (1-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        /// Username to list following for (default: you)
        #[arg(long = "of")]
        of: Option<String>,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List your followers
    #[command(after_help = FOLLOWERS_HELP)]
    Followers {
        /// Number of results (1-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        /// Username to list followers for (default: you)
        #[arg(long = "of")]
        of: Option<String>,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Mute a user
    #[command(after_help = MUTE_HELP)]
    Mute {
        /// Username to mute
        #[arg(value_name = "USERNAME")]
        target_username: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Unmute a user
    #[command(after_help = UNMUTE_HELP)]
    Unmute {
        /// Username to unmute
        #[arg(value_name = "USERNAME")]
        target_username: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List users you have muted
    #[command(after_help = MUTED_HELP)]
    Muted {
        /// Number of results (1-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Block a user
    #[command(after_help = BLOCK_HELP)]
    Block {
        /// Username to block
        #[arg(value_name = "USERNAME")]
        target_username: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Unblock a user
    #[command(after_help = UNBLOCK_HELP)]
    Unblock {
        /// Username to unblock
        #[arg(value_name = "USERNAME")]
        target_username: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List users you have blocked
    #[command(after_help = BLOCKED_HELP)]
    Blocked {
        /// Number of results (1-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Usage ─────────────────────────────────────────────────────────
    /// Show API usage (post caps, daily breakdown)
    #[command(after_help = USAGE_HELP)]
    Usage {
        /// Usage family subcommand; bare `xr usage` reads `/2/usage/tweets`.
        #[command(subcommand)]
        target: Option<UsageCommands>,

        /// Shortcut flags shared with every other shortcut command.
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
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// List recent direct messages
    #[command(after_help = DMS_HELP)]
    Dms {
        /// Number of results (1-100). Overrides global `--limit` when set.
        #[arg(short = 'n', long = "max-results")]
        max_results: Option<i32>,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },

    // ── Broadcasts ───────────────────────────────────────────────────
    /// Broadcast chat moderation
    #[command(after_help = family_help::broadcasts::root_page())]
    Broadcasts {
        /// `broadcasts` family subcommand.
        #[command(subcommand)]
        target: BroadcastsCommands,
    },

    // ── Auth ─────────────────────────────────────────────────────────
    /// Authentication management
    #[command(after_help = AUTH_HELP)]
    Auth {
        /// `auth` subcommand to dispatch (`oauth2`, `oauth1`, `app`, …).
        #[command(subcommand)]
        command: AuthCommands,
    },

    // ── Media ────────────────────────────────────────────────────────
    /// Media upload, alt text, and subtitles
    #[command(after_help = MEDIA_HELP)]
    Media {
        /// `media` subcommand to dispatch.
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
        /// `skill` subcommand to dispatch (`install` or `update`).
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
    /// envelope. Use `--schema` to pin a specific shape (e.g. `post`,
    /// `user`); without it the command auto-detects from the top-level
    /// shape.
    #[command(after_help = VALIDATE_HELP)]
    Validate {
        /// File path to read JSON from. Pass `-` or omit to read from stdin.
        #[arg(value_name = "FILE")]
        file: Option<String>,

        /// Schema name to validate against; the list comes from the alias
        /// table in `commands::validate`.
        #[arg(long = "schema", value_name = "NAME", help = commands::validate::schema_arg_help())]
        schema: Option<String>,
    },
}

/// Examples block for `xr usage credits --help`.
const USAGE_CREDITS_HELP: &str = "\
Examples:
  Your credits-based usage:
    xr usage credits --output json

  With an explicit app:
    xr usage credits --app mydev --output json
";

/// `xr usage` family subcommands.
#[derive(Subcommand, Debug)]
pub enum UsageCommands {
    /// Show credits-based usage for the project
    #[command(after_help = USAGE_CREDITS_HELP)]
    Credits {
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
}

/// `xr broadcasts` family subcommands.
#[derive(Subcommand, Debug)]
pub enum BroadcastsCommands {
    /// Manage who moderates your broadcast chats
    #[command(after_help = family_help::broadcasts::moderators_page())]
    Moderators {
        /// `moderators` verb to dispatch.
        #[command(subcommand)]
        action: ModeratorsCommands,
    },
}

/// `xr broadcasts moderators` verbs.
#[derive(Subcommand, Debug)]
pub enum ModeratorsCommands {
    /// List your broadcast chat moderators
    #[command(after_help = family_help::broadcasts::FAMILY.verb_page(&family_help::broadcasts::LIST))]
    List {
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Add a broadcast chat moderator
    #[command(after_help = family_help::broadcasts::FAMILY.verb_page(&family_help::broadcasts::ADD))]
    Add {
        /// Username to add
        #[arg(value_name = "USERNAME")]
        target_username: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Remove a broadcast chat moderator
    #[command(after_help = family_help::broadcasts::FAMILY.verb_page(&family_help::broadcasts::REMOVE))]
    Remove {
        /// Username to remove
        #[arg(value_name = "USERNAME")]
        target_username: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
}

/// Parses a `<host>` argument: the possible-values list is what help and
/// completions render, and what `KNOWN_HOSTS` accepts is exactly what
/// `SkillHost::from_key` maps, so the fallible step never fires.
fn skill_host_parser() -> impl clap::builder::TypedValueParser<Value = SkillHost> {
    use clap::builder::TypedValueParser as _;

    clap::builder::PossibleValuesParser::new(KNOWN_HOSTS).try_map(|name: String| {
        SkillHost::from_key(&name).ok_or_else(|| format!("unknown host {name:?}"))
    })
}

/// `skill` subcommand variants.
#[derive(Subcommand, Debug)]
pub enum SkillCmd {
    /// Install the skill bundle into a host's canonical skills directory.
    ///
    /// Shallow-clones the xurl-rs repository so the bundled `AGENTS.md` is
    /// discoverable to local agents. The destination is the host's skills
    /// directory: under the host's own config-directory variable when it is
    /// set, else under `XURL_SKILL_HOME`, else under a base-directory variable
    /// the host follows or `~` (ENVIRONMENT VARIABLES in `xr --help` lists
    /// every variable).
    #[command(after_help = "Examples:
  xr skill install claude_code                     # install bundle to Claude Code
  xr skill install claude_code --dry-run           # print the resolved git command without spawning
  xr skill install --all                           # install across every known host
  xr skill install codex --output json             # JSON envelope for agent consumption
  xr skill install --all --dry-run --output json   # multi-host dry-run envelope")]
    Install {
        /// Target host (e.g. claude_code, codex, cursor). Required unless `--all`.
        #[arg(value_parser = skill_host_parser())]
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
    /// the bundle picks up upstream changes. The destination and hardening
    /// surface are identical to `install`. A copy at a location the host still
    /// reads but xr does not install to, such as Codex's
    /// `~/.codex/skills/xurl-rs`, is removed too and named in
    /// `legacy_install_dir`. The envelope's `action` is `"skill-update"` so
    /// agents can distinguish from a first-time install.
    #[command(after_help = "Examples:
  xr skill update claude_code                      # refresh Claude Code's xurl-rs bundle
  xr skill update claude_code --dry-run            # show the resolved plan without touching disk
  xr skill update --all                            # refresh every known host
  xr skill update codex --output json              # JSON envelope for agent consumption")]
    Update {
        /// Target host (e.g. claude_code, codex, cursor). Required unless `--all`.
        #[arg(value_parser = skill_host_parser())]
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

/// A required secret's two sources, exactly one of which is given: the plain
/// flag, or its file twin that keeps the value out of argv.
fn secret_source(plain: &'static str, file: &'static str) -> clap::ArgGroup {
    clap::ArgGroup::new(format!("{plain}_source"))
        .args([plain, file])
        .required(true)
        .multiple(false)
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
            require_equals = true,
        )]
        no_browser: bool,
        /// Step number: 1 (generate auth URL) or 2 (complete exchange)
        #[arg(long, requires = "no_browser", value_parser = clap::value_parser!(u8).range(1..=2))]
        step: Option<u8>,
        /// Redirect URL from browser (step 2). Use '-' to read from stdin (recommended on shared machines)
        #[arg(long = "auth-url", requires = "step")]
        auth_url: Option<String>,
        /// Request only these comma-separated scopes, plus offline.access (default: every scope)
        ///
        /// A sign-in asks for every scope `xr` can use unless this names a
        /// subset. `offline.access` is always added, because without a refresh
        /// token the login ends within hours. A name X does not define is
        /// rejected with the list of valid ones. Step 2 of the headless flow
        /// takes the scopes step 1 saved, so the flag belongs on step 1.
        #[arg(long, value_name = "SCOPES", value_delimiter = ',')]
        scopes: Option<Vec<String>>,
        /// Username to label the saved token (bypasses `/2/users/me` lookup when supplied)
        #[arg(value_name = "USERNAME")]
        username: Option<String>,
    },
    /// Configure `OAuth1` authentication
    #[command(
        after_help = AUTH_OAUTH1_HELP,
        group(secret_source("consumer_secret", "consumer_secret_file")),
        group(secret_source("access_token", "access_token_file")),
        group(secret_source("token_secret", "token_secret_file")),
    )]
    Oauth1 {
        /// Consumer key
        #[arg(long = "consumer-key")]
        consumer_key: String,
        /// Consumer secret
        #[arg(long = "consumer-secret")]
        consumer_secret: Option<String>,
        /// File holding the consumer secret; '-' reads it from stdin
        #[arg(long = "consumer-secret-file", value_name = "PATH")]
        consumer_secret_file: Option<String>,
        /// Access token
        #[arg(long = "access-token")]
        access_token: Option<String>,
        /// File holding the access token; '-' reads it from stdin
        #[arg(long = "access-token-file", value_name = "PATH")]
        access_token_file: Option<String>,
        /// Token secret
        #[arg(long = "token-secret")]
        token_secret: Option<String>,
        /// File holding the token secret; '-' reads it from stdin
        #[arg(long = "token-secret-file", value_name = "PATH")]
        token_secret_file: Option<String>,
    },
    /// Configure app-auth (bearer token)
    #[command(
        after_help = AUTH_APP_HELP,
        group(secret_source("bearer_token", "bearer_token_file")),
    )]
    App {
        /// Bearer token
        #[arg(long = "bearer-token")]
        bearer_token: Option<String>,
        /// File holding the bearer token; '-' reads it from stdin
        #[arg(long = "bearer-token-file", value_name = "PATH")]
        bearer_token_file: Option<String>,
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
        /// `apps` subcommand to dispatch (`add`, `list`, `update`, …).
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
    #[command(
        after_help = APPS_ADD_HELP,
        group(secret_source("client_secret", "client_secret_file")),
    )]
    Add {
        /// App name
        name: String,
        /// `OAuth2` client ID
        #[arg(long = "client-id")]
        client_id: String,
        /// `OAuth2` client secret
        #[arg(long = "client-secret")]
        client_secret: Option<String>,
        /// File holding the `OAuth2` client secret; '-' reads it from stdin
        #[arg(long = "client-secret-file", value_name = "PATH")]
        client_secret_file: Option<String>,
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
        /// File holding the `OAuth2` client secret; '-' reads it from stdin
        #[arg(
            long = "client-secret-file",
            value_name = "PATH",
            conflicts_with = "client_secret"
        )]
        client_secret_file: Option<String>,
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
        /// `redirect-uri` subcommand to dispatch (`get` or `set`).
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

/// How long a media command waits for X to finish processing: `None` does
/// not wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessingWait(pub Option<std::time::Duration>);

/// Reads a `--wait` value: `true` is the default deadline, `false` or `0` is
/// no wait, and a number is that many seconds.
fn parse_processing_wait(value: &str) -> Result<ProcessingWait, String> {
    match value {
        "true" => Ok(ProcessingWait(Some(xdk::api::DEFAULT_PROCESSING_WAIT))),
        "false" | "0" => Ok(ProcessingWait(None)),
        secs => secs
            .parse::<u64>()
            .map(|secs| ProcessingWait(Some(std::time::Duration::from_secs(secs))))
            .map_err(|_| "expected a number of seconds, 'true', or 'false'".to_string()),
    }
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
        /// Wait for X to finish processing the upload before returning
        ///
        /// On by default, for up to 60 seconds. It applies to a video, and to
        /// any other upload X reports as still processing when it is
        /// finalized, such as an animated GIF; an upload X reports as ready
        /// returns at once. `--wait=<SECS>` waits that long, and
        /// `--wait=false` or `--wait=0` returns after FINALIZE. The value
        /// follows `=`. A wait that reaches its deadline exits 1 with reason
        /// `processing-timeout`; the upload itself is intact, and the error
        /// names the `xr media status` command that resumes the wait.
        #[arg(
            long = "wait",
            value_name = "SECS",
            num_args = 0..=1,
            require_equals = true,
            default_value = "true",
            default_missing_value = "true",
            value_parser = parse_processing_wait,
        )]
        wait: ProcessingWait,
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
        /// Wait for X to finish processing instead of reading the status once
        ///
        /// Bare `--wait` or `--wait=true` waits up to 60 seconds, and
        /// `--wait=<SECS>` that long; `--wait=false` or `--wait=0` reads the
        /// status once, which is the default. The value follows `=`. A wait
        /// that reaches its deadline exits 1 with reason `processing-timeout`
        /// and names the command that resumes it for twice as long.
        #[arg(
            short = 'w',
            long = "wait",
            value_name = "SECS",
            num_args = 0..=1,
            require_equals = true,
            default_value = "false",
            default_missing_value = "true",
            value_parser = parse_processing_wait,
        )]
        wait: ProcessingWait,
        /// Trace header
        #[arg(short = 't', long = "trace")]
        trace: bool,
        /// Request headers
        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
    },
    /// Set the alt text shown for an uploaded image or video
    #[command(after_help = MEDIA_ALT_TEXT_HELP)]
    AltText {
        /// Media id from `xr media upload`
        #[arg(value_name = "MEDIA_ID")]
        media_id: String,
        /// Alt text, up to 1000 characters
        #[arg(value_name = "TEXT")]
        text: String,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Add or remove the subtitle tracks of an uploaded video
    #[command(after_help = family_help::media_subtitles::page())]
    Subtitles {
        /// `subtitles` verb to dispatch.
        #[command(subcommand)]
        action: SubtitlesCommands,
    },
}

/// `xr media subtitles` verbs.
#[derive(Subcommand, Debug)]
pub enum SubtitlesCommands {
    /// Add a subtitle track to an uploaded video
    #[command(after_help = family_help::media_subtitles::FAMILY.verb_page(&family_help::media_subtitles::ADD))]
    Add {
        /// Media id of the video
        #[arg(value_name = "VIDEO_ID")]
        video_id: String,
        /// Media id of the subtitle file, uploaded with `--category subtitles`
        #[arg(value_name = "SUBTITLES_ID")]
        subtitles_id: String,
        /// Two-letter language code of the track (e.g. en)
        #[arg(long = "language", value_name = "CODE")]
        language: String,
        /// Language name viewers pick the track by (e.g. English)
        #[arg(long = "name", value_name = "NAME")]
        display_name: Option<String>,
        /// Category the video was uploaded with
        #[arg(long = "category", value_parser = video_category_parser(), default_value = "amplify_video")]
        category: VideoCategory,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
    /// Remove a subtitle track from an uploaded video
    #[command(after_help = family_help::media_subtitles::FAMILY.verb_page(&family_help::media_subtitles::REMOVE))]
    Remove {
        /// Media id of the video
        #[arg(value_name = "VIDEO_ID")]
        video_id: String,
        /// Two-letter language code of the track to remove (e.g. en)
        #[arg(long = "language", value_name = "CODE")]
        language: String,
        /// Category the video was uploaded with
        #[arg(long = "category", value_parser = video_category_parser(), default_value = "amplify_video")]
        category: VideoCategory,
        /// Shortcut flags shared with every other shortcut command.
        #[command(flatten)]
        common: CommonFlags,
    },
}

/// Parses a `--category` for a subtitled video: the possible values are the
/// upload names `VideoCategory::from_upload_name` maps, so the fallible step
/// never fires.
fn video_category_parser() -> impl clap::builder::TypedValueParser<Value = VideoCategory> {
    use clap::builder::TypedValueParser as _;

    clap::builder::PossibleValuesParser::new(VideoCategory::ALL.map(VideoCategory::upload_name))
        .try_map(|name: String| {
            VideoCategory::from_upload_name(&name)
                .ok_or_else(|| format!("unknown category {name:?}"))
        })
}
