# Feature Parity: xurl (Go) → xurl-rs (Rust)

Tracks every feature in the Go xurl original and its status in the Rust port.

**Legend:** ✅ Implemented & Tested | 🔄 Partial | ❌ Missing

## CLI Framework

| Feature | Go | Rust | Notes |
|---|---|---|---|
| `--help` flag | ✅ | ✅ | Different format (cobra vs clap) — both exit 0 |
| `--version` flag | ✅ | ✅ | Go uses `version` subcommand; Rust has both `--version` and `version` |
| `--app` global flag | ✅ | ✅ | Per-request app override |
| `--auth` flag | ✅ | ✅ | oauth1/oauth2/app |
| `-X` / `--method` | ✅ | ✅ | |
| `-H` / `--header` | ✅ | ✅ | |
| `-d` / `--data` | ✅ | ✅ | |
| `-v` / `--verbose` | ✅ | ✅ | |
| `-t` / `--trace` | ✅ | ✅ | X-B3-Flags header |
| `-s` / `--stream` | ✅ | ✅ | Force streaming mode |
| `-F` / `--file` | ✅ | ✅ | Multipart file upload |
| `-u` / `--username` | ✅ | ✅ | OAuth2 username |

## Raw API Mode

| Feature | Go | Rust | Notes |
|---|---|---|---|
| Raw GET requests | ✅ | ✅ | `xurl /2/users/me` |
| Raw POST requests | ✅ | ✅ | `xurl -X POST /2/tweets -d '{...}'` |
| Absolute URL support | ✅ | ✅ | `xurl https://api.x.com/2/users/me` |
| Custom headers | ✅ | ✅ | |
| Streaming endpoint detection | ✅ | ✅ | Auto-detects /stream paths |
| Multipart media append | ✅ | ✅ | |

## Shortcut Commands — Posting

| Command | Go | Rust | Notes |
|---|---|---|---|
| `post "text"` | ✅ | ✅ | |
| `post "text" --media-id X` | ✅ | ✅ | |
| `reply <id> "text"` | ✅ | ✅ | |
| `reply <url> "text"` | ✅ | ✅ | URL → ID extraction |
| `quote <id> "text"` | ✅ | ✅ | |
| `delete <id>` | ✅ | ✅ | |

## Shortcut Commands — Reading

| Command | Go | Rust | Notes |
|---|---|---|---|
| `read <id>` | ✅ | ✅ | With field expansions |
| `search "query"` | ✅ | ✅ | |
| `search "query" -n 20` | ✅ | ✅ | Clamped 10–100 |

## Shortcut Commands — User Info

| Command | Go | Rust | Notes |
|---|---|---|---|
| `whoami` | ✅ | ✅ | /2/users/me |
| `user <username>` | ✅ | ✅ | Go uses `user`; Rust uses `user` |
| `timeline` | ✅ | ✅ | |
| `mentions` | ✅ | ✅ | |

## Shortcut Commands — Engagement

| Command | Go | Rust | Notes |
|---|---|---|---|
| `like <id>` | ✅ | ✅ | |
| `unlike <id>` | ✅ | ✅ | |
| `repost <id>` | ✅ | ✅ | |
| `unrepost <id>` | ✅ | ✅ | |
| `bookmark <id>` | ✅ | ✅ | |
| `unbookmark <id>` | ✅ | ✅ | |
| `bookmarks` | ✅ | ✅ | |
| `likes` | ✅ | ✅ | |

## Shortcut Commands — Social Graph

| Command | Go | Rust | Notes |
|---|---|---|---|
| `follow @user` | ✅ | ✅ | |
| `unfollow @user` | ✅ | ✅ | |
| `following` | ✅ | ✅ | With `--of` flag |
| `followers` | ✅ | ✅ | With `--of` flag |
| `block @user` | ✅ | ✅ | |
| `unblock @user` | ✅ | ✅ | |
| `mute @user` | ✅ | ✅ | |
| `unmute @user` | ✅ | ✅ | |

## Shortcut Commands — Direct Messages

| Command | Go | Rust | Notes |
|---|---|---|---|
| `dm @user "text"` | ✅ | ✅ | |
| `dms` | ✅ | ✅ | |

## Authentication

| Feature | Go | Rust | Notes |
|---|---|---|---|
| OAuth2 PKCE flow | ✅ | ✅ | Browser-based auth |
| OAuth2 token refresh | ✅ | ✅ | Automatic refresh |
| OAuth1 HMAC-SHA1 | ✅ | ✅ | Full RFC 5849 |
| Bearer token (app-only) | ✅ | ✅ | |
| `auth oauth2` | ✅ | ✅ | Interactive flow |
| `auth oauth1` | ✅ | ✅ | Direct credential input |
| `auth app` | ✅ | ✅ | Bearer token setup |
| `auth status` | ✅ | ✅ | |
| `auth clear` | ✅ | ✅ | With --all, --oauth1, --oauth2-username, --bearer |
| `auth default` | ✅ | ✅ | Interactive picker |
| `auth default <name>` | ✅ | ✅ | Set by name |

## Multi-App Management

| Feature | Go | Rust | Notes |
|---|---|---|---|
| `auth apps add` | ✅ | ✅ | |
| `auth apps update` | ✅ | ✅ | |
| `auth apps remove` | ✅ | ✅ | |
| `auth apps list` | ✅ | ✅ | |
| Default app management | ✅ | ✅ | |
| Per-app token isolation | ✅ | ✅ | |

## Media Upload

| Feature | Go | Rust | Notes |
|---|---|---|---|
| `media upload <file>` | ✅ | ✅ | Chunked 4MB uploads |
| `media status <id>` | ✅ | ✅ | |
| Processing wait/poll | ✅ | ✅ | With backoff |
| Media type detection | ✅ | ✅ | |
| Segment index tracking | ✅ | ✅ | |

## Token Store

| Feature | Go | Rust | Notes |
|---|---|---|---|
| YAML persistence (~/.xurl) | ✅ | ✅ | |
| Multi-app credential store | ✅ | ✅ | |
| Legacy JSON migration | ✅ | ✅ | Auto-converts |
| .twurlrc import | ✅ | ✅ | Legacy Twitter CLI |
| Credential backfill from env | ✅ | ✅ | |

## Output

| Feature | Go | Rust | Notes |
|---|---|---|---|
| JSON syntax highlighting | ✅ | ✅ | colored crate |
| Pretty-printed JSON | ✅ | ✅ | |
| Verbose request/response info | ✅ | ✅ | |
| Streaming output | ✅ | ✅ | Line-by-line |

## Intentionally Different

| Feature | Go | Rust | Reason |
|---|---|---|---|
| `--version` flag | ❌ (uses `version` cmd) | ✅ (both) | clap derives --version automatically |
| `completion` subcommand | ✅ (cobra built-in) | ❌ | Will add via clap_complete (Phase 6) |
| `webhook` subcommand | ✅ | ❌ | Deferred — low priority, rarely used |
| `lookup` alias | ✅ | ❌ | Renamed to `user` for clarity |
| Exit code 1 for usage errors | ✅ | ❌ (exit 2) | Rust/clap follows UNIX convention: 2 = usage error |
| Help text format | cobra style | clap style | Framework difference — content equivalent |

## Missing (Deferred)

| Feature | Priority | Reason |
|---|---|---|
| `webhook` subcommand | Low | Rarely used in practice |
| `completion` subcommand | Medium | Adding in Phase 6 via clap_complete |
| Shell completion scripts | Medium | Adding in Phase 6 |
