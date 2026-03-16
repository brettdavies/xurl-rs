# xurl

A fast, ergonomic CLI for the X (Twitter) API. OAuth1, OAuth2 PKCE, Bearer auth. Media upload. Streaming. Agent-native.

Rust port of [xurl](https://github.com/xdevplatform/xurl) — faster, type-safe, with shell completions and machine-readable output.

## Install

```bash
cargo install xurl
```

Or via Homebrew:

```bash
brew tap brettdavies/tap
brew install xurl
```

## Quick Start

```bash
# Set up OAuth2 (browser-based, 30 seconds)
xurl auth apps add myapp --client-id YOUR_ID --client-secret YOUR_SECRET
xurl auth oauth2

# Post
xurl post "Hello from xurl-rs!"

# Read
xurl read 1234567890

# Search
xurl search "rust programming" -n 20

# Check your profile
xurl whoami
```

## Commands

### Posting

```bash
xurl post "Hello world!"                        # Post
xurl post "With media" --media-id 12345          # Post with media
xurl reply 1234567890 "Nice!"                    # Reply
xurl reply https://x.com/user/status/123 "Nice!" # Reply by URL
xurl quote 1234567890 "My take"                  # Quote
xurl delete 1234567890                           # Delete
```

### Reading

```bash
xurl read 1234567890                             # Read a post
xurl search "golang" -n 20                       # Search (10-100 results)
xurl whoami                                      # Your profile
xurl user @elonmusk                              # Look up user
xurl timeline                                    # Home timeline
xurl mentions                                    # Your mentions
```

### Engagement

```bash
xurl like 1234567890                             # Like
xurl unlike 1234567890                           # Unlike
xurl repost 1234567890                           # Repost
xurl bookmark 1234567890                         # Bookmark
xurl bookmarks                                   # List bookmarks
xurl likes                                       # List likes
```

### Social Graph

```bash
xurl follow @user                                # Follow
xurl unfollow @user                              # Unfollow
xurl following                                   # Who you follow
xurl followers                                   # Your followers
xurl block @user                                 # Block
xurl mute @user                                  # Mute
```

### Direct Messages

```bash
xurl dm @user "Hey!"                             # Send DM
xurl dms                                         # List DMs
```

### Raw API Access

```bash
xurl /2/users/me                                 # GET request
xurl -X POST /2/tweets -d '{"text":"Hello!"}'    # POST with JSON body
xurl --auth oauth1 /2/users/me                   # Explicit auth type
xurl -s /2/tweets/search/stream                  # Streaming
```

### Media Upload

```bash
xurl media upload video.mp4                      # Upload media
xurl media status 1234567890                     # Check status
```

## Authentication

### OAuth2 (Recommended)

```bash
xurl auth apps add myapp --client-id ID --client-secret SECRET
xurl auth oauth2                                 # Opens browser
```

### OAuth1

```bash
xurl auth oauth1 \
  --consumer-key CK \
  --consumer-secret CS \
  --access-token AT \
  --token-secret TS
```

### Bearer Token (App-Only)

```bash
xurl auth app --bearer-token YOUR_TOKEN
```

### Multi-App Management

```bash
xurl auth apps add prod --client-id ... --client-secret ...
xurl auth apps add dev --client-id ... --client-secret ...
xurl auth apps list
xurl auth default prod                           # Set default
xurl --app dev whoami                             # Per-request override
```

## Agent-Native Features

Built for AI agents and automation:

### Machine-Readable Output

```bash
xurl --output json whoami                        # Raw JSON, no color
xurl --output jsonl search "topic"               # JSON Lines for streaming
export XURL_OUTPUT=json                          # Default to JSON
```

### Quiet Mode

```bash
xurl --quiet post "Hello"                        # No progress indicators
xurl -q search "topic"                           # Short form
```

### Non-Interactive Mode

```bash
xurl --no-interactive whoami                     # Error instead of prompt
# Exit code 2 if auth needed: "authentication required: run xurl auth login"
```

### Structured Exit Codes

| Code | Meaning | Agent Action |
|------|---------|-------------|
| 0 | Success | Continue |
| 1 | General error | Log and handle |
| 2 | Auth required | Run `xurl auth oauth2` |
| 3 | Rate limited | Retry with backoff |
| 4 | Not found | Resource doesn't exist |
| 5 | Network error | Check connectivity |

### NO_COLOR Support

```bash
NO_COLOR=1 xurl whoami                           # Disable color (no-color.org)
```

## Shell Completions

```bash
# Bash
xurl --generate-completion bash > ~/.bash_completion.d/xurl

# Zsh
xurl --generate-completion zsh > ~/.zfunc/_xurl

# Fish
xurl --generate-completion fish > ~/.config/fish/completions/xurl.fish

# PowerShell
xurl --generate-completion powershell > xurl.ps1

# Elvish
xurl --generate-completion elvish > xurl.elv
```

Pre-generated scripts are also available in `completions/`.

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

## Contributing

```bash
git clone https://github.com/brettdavies/xurl-rs
cd xurl-rs
cargo test
cargo clippy
```

See [RELEASING.md](RELEASING.md) for release procedures.

## License

MIT
