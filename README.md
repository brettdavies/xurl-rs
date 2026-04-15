# xurl-rs

A fast, ergonomic CLI for the X (Twitter) API. OAuth1, OAuth2 PKCE, Bearer auth. Media upload. Streaming. Agent-native.

Rust port of [xurl](https://github.com/xdevplatform/xurl) — faster, type-safe, with shell completions and
machine-readable output.

## Install

### Homebrew

```bash
brew tap brettdavies/tap
brew install xurl-rs
```

### Pre-built Binary

Download from [GitHub Releases](https://github.com/brettdavies/xurl-rs/releases) for Linux, macOS, and Windows.

### Cargo

```bash
cargo install xurl-rs
```

### From Source

```bash
git clone https://github.com/brettdavies/xurl-rs
cd xurl-rs
cargo build --release
# Binary at ./target/release/xr
```

## Quick Start

```bash
# Set up OAuth2 (browser-based, 30 seconds)
xr auth apps add myapp --client-id YOUR_ID --client-secret YOUR_SECRET
xr auth oauth2

# Post
xr post "Hello from xurl-rs!"

# Read
xr read 1234567890

# Search
xr search "rust programming" -n 20

# Check your profile
xr whoami
```

## Commands

### Posting

```bash
xr post "Hello world!"                        # Post
xr post "With media" --media-id 12345          # Post with media
xr reply 1234567890 "Nice!"                    # Reply
xr reply https://x.com/user/status/123 "Nice!" # Reply by URL
xr quote 1234567890 "My take"                  # Quote
xr delete 1234567890                           # Delete
```

### Reading

```bash
xr read 1234567890                             # Read a post
xr search "golang" -n 20                       # Search (10-100 results)
xr whoami                                      # Your profile
xr user @elonmusk                              # Look up user
xr timeline                                    # Home timeline
xr mentions                                    # Your mentions
```

### Engagement

```bash
xr like 1234567890                             # Like
xr unlike 1234567890                           # Unlike
xr repost 1234567890                           # Repost
xr bookmark 1234567890                         # Bookmark
xr bookmarks                                   # List bookmarks
xr likes                                       # List likes
```

### Social Graph

```bash
xr follow @user                                # Follow
xr unfollow @user                              # Unfollow
xr following                                   # Who you follow
xr followers                                   # Your followers
xr block @user                                 # Block
xr mute @user                                  # Mute
```

### Direct Messages

```bash
xr dm @user "Hey!"                             # Send DM
xr dms                                         # List DMs
```

### Schema Discovery

```bash
xr schema post                                 # JSON Schema for post response
xr schema whoami                               # JSON Schema for whoami response
xr schema --list                               # All commands and response types
xr schema --all                                # All schemas as one JSON document
```

Generate typed clients from schema output:

```bash
# TypeScript
xr schema post | bunx json-schema-to-typescript > types.ts

# Python
xr schema post | uvx --from datamodel-code-generator datamodel-codegen --output models.py
```

### Raw API Access

```bash
xr /2/users/me                                 # GET request
xr -X POST /2/tweets -d '{"text":"Hello!"}'    # POST with JSON body
xr --auth oauth1 /2/users/me                   # Explicit auth type
xr -s /2/tweets/search/stream                  # Streaming
```

### Media Upload

```bash
xr media upload video.mp4                      # Upload media
xr media status 1234567890                     # Check status
```

## Authentication

### OAuth2 (Recommended)

```bash
xr auth apps add myapp --client-id ID --client-secret SECRET
xr auth oauth2                                 # Opens browser
```

### OAuth1

```bash
xr auth oauth1 \
  --consumer-key CK \
  --consumer-secret CS \
  --access-token AT \
  --token-secret TS
```

### Bearer Token (App-Only)

```bash
xr auth app --bearer-token YOUR_TOKEN
```

### Multi-App Management

```bash
xr auth apps add prod --client-id ... --client-secret ...
xr auth apps add dev --client-id ... --client-secret ...
xr auth apps list
xr auth default prod                           # Set default
xr --app dev whoami                             # Per-request override
```

## Agent-Native Features

Built for AI agents and automation:

### Response Schema Discovery

```bash
xr schema --list                               # Discover all commands + response types
xr schema post                                 # Get JSON Schema for any command's output
xr schema --all                                # All schemas for MCP tool definitions
```

### Machine-Readable Output

```bash
xr --output json whoami                        # Raw JSON, no color
xr --output jsonl search "topic"               # JSON Lines for streaming
export XURL_OUTPUT=json                          # Default to JSON
```

### Quiet Mode

```bash
xr --quiet post "Hello"                        # No progress indicators
xr -q search "topic"                           # Short form
```

### Non-Interactive Mode

```bash
xr --no-interactive whoami                     # Error instead of prompt
# Exit code 2 if auth needed: "authentication required: run xr auth login"
```

### Structured Exit Codes

| Code | Meaning | Agent Action |
|------|---------|-------------|
| 0 | Success | Continue |
| 1 | General error | Log and handle |
| 2 | Auth required | Run `xr auth oauth2` |
| 3 | Rate limited | Retry with backoff |
| 4 | Not found | Resource doesn't exist |
| 5 | Network error | Check connectivity |

### NO_COLOR Support

```bash
NO_COLOR=1 xr whoami                           # Disable color (no-color.org)
```

## Shell Completions

```bash
# Bash
xr completions bash > ~/.local/share/bash-completion/completions/xr

# Zsh (writes to the first directory on your fpath)
xr completions zsh > "${fpath[1]}/_xr"

# Fish
xr completions fish > ~/.config/fish/completions/xr.fish

# PowerShell
xr completions powershell > xr.ps1

# Elvish
xr completions elvish > xr.elv
```

Pre-generated scripts are also available in `completions/`.

## Library Usage

xurl-rs is also a Rust library. Add it to your `Cargo.toml`:

```toml
[dependencies]
xurl-rs = "1"
```

All 29 shortcut commands return typed responses via `ApiResponse<T>`:

```rust
use xurl::api::{ApiResponse, Tweet, User, LikedResult, deserialize_response};

// Typed response from deserialization
let resp: ApiResponse<Tweet> = deserialize_response(json_value)?;
println!("{}", resp.data.text);

// List responses
let resp: ApiResponse<Vec<Tweet>> = deserialize_response(json_value)?;
for tweet in &resp.data {
    println!("{}: {}", tweet.id, tweet.text);
}
```

Available types: `Tweet`, `User`, `DmEvent`, `UsageData`, `LikedResult`, `FollowingResult`, `DeletedResult`,
`RetweetedResult`, `BookmarkedResult`, `BlockingResult`, `MutingResult`, `MediaUploadResponse`, `Includes`,
`ResponseMeta`, `ApiError`.

All structs include `#[serde(flatten)] extra: BTreeMap<String, Value>` for forward compatibility with new API fields.

## vs Go Original

| Feature | Go xurl | xurl-rs |
|---------|---------|---------|
| Language | Go | Rust |
| Memory safety | GC | Compile-time |
| Binary size | ~15 MB | ~8 MB |
| Shell completions | Built-in (cobra) | Built-in (clap_complete) |
| `--output json` | ❌ | ✅ |
| `--quiet` | ❌ | ✅ |
| `--no-interactive` | ❌ | ✅ |
| Structured exit codes | ❌ | ✅ |
| `NO_COLOR` support | ❌ | ✅ |
| `XURL_OUTPUT` env var | ❌ | ✅ |
| Typed response structs | ❌ | ✅ |
| `xr schema` (JSON Schema) | ❌ | ✅ |

## Contributing

```bash
git clone https://github.com/brettdavies/xurl-rs
cd xurl-rs
cargo test
cargo clippy
```

See [RELEASING.md](RELEASING.md) for release procedures.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
