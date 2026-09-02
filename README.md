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

Most shortcut commands honor `-u USERNAME` to bypass the `/2/users/me` lookup. When set, the user-ID resolver hits
`/2/users/by/username/<u>` instead, which is useful when `/me` is temporarily failing on X. Example: `xr like POST_ID -u
alice`.

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
xr auth oauth2 alice                           # Skip /2/users/me; save under "alice"
xr auth oauth2 --app myapp alice               # Same, against a specific app
```

`xr auth oauth2` accepts an optional `[USERNAME]` positional. If X's `/2/users/me` endpoint is unreliable, supplying the
handle explicitly skips that lookup and stores the resulting token under the known username so shortcut commands resolve
without `/me`.

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

Register an app with a custom OAuth2 callback URL via `--redirect-uri`:

```bash
xr auth apps add prod \
  --client-id ID \
  --client-secret SECRET \
  --redirect-uri http://localhost:8080/callback
```

Update credentials or the stored redirect URI on an existing app:

```bash
xr auth apps update prod --client-id NEW_ID --client-secret NEW_SECRET
xr auth apps update prod --redirect-uri http://localhost:8080/callback
```

The `REDIRECT_URI` environment variable still overrides the stored app value at runtime, so `auth apps update
--redirect-uri` is best for setting your default per-app callback while env vars stay the temporary override path.

Inspect the effective redirect URI for an app (or the default app when `NAME` is omitted) — the output shows the
resolved URI, its source (`env-var` | `app-config` | `built-in-default`), and the stored URI when an env var is
overriding it:

```bash
xr auth apps redirect-uri get               # Default app
xr auth apps redirect-uri get prod          # Named app
```

Write the per-app stored redirect URI. The scheme must be `https`, or `http` with a host in `{localhost, 127.0.0.1,
::1}`:

```bash
xr auth apps redirect-uri set prod http://localhost:8080/callback
```

If you run `xr auth oauth2` without `--app`, the default app has no `client_id` set, and another registered app does
have credentials, the CLI prints a warning suggesting `xr auth oauth2 --app NAME` so the token lands on the right app
instead of the credential-less default.

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

`xr --output json auth status` and `xr --output json auth apps list` emit a structured array with one object per
registered app. Per-app fields:

- `name` — app name.
- `client_id_hint` — first eight characters of the `client_id`, for visual identification without leaking the full ID.
- `redirect_uri` — the effective redirect URI for this app.
- `redirect_uri_source` — kebab-case provenance: `env-var` | `app-config` | `built-in-default`.
- `redirect_uri_stored` — only present when the `REDIRECT_URI` environment variable overrides a stored app value;
  carries the stored value so precedence is auditable.
- `oauth2_users` — array of usernames with OAuth2 tokens stored under this app.
- `oauth1` — boolean: OAuth1 credentials are stored for this app.
- `bearer` — boolean: a bearer token is stored for this app.
- `default` — boolean: this is the default app.
- `oauth2_unnamed` — only present when `true`; indicates an unnamed-user OAuth2 token is stored after a refresh where
  `/2/users/me` failed and no username was supplied.

```bash
xr --output json auth status | jq '.[] | select(.default) | .name'
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

| Code | Meaning       | Agent Action           |
| ---- | ------------- | ---------------------- |
| 0    | Success       | Continue               |
| 1    | General error | Log and handle         |
| 2    | Auth required | Run `xr auth oauth2`   |
| 3    | Rate limited  | Retry with backoff     |
| 4    | Not found     | Resource doesn't exist |
| 5    | Network error | Check connectivity     |

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

All 30 shortcut commands return typed responses via `ApiResponse<T>`:

```rust
use xurl::api::{ApiResponse, Post, User, LikedResult, deserialize_response};

// Typed response from deserialization
let resp: ApiResponse<Post> = deserialize_response(json_value)?;
println!("{}", resp.data.text);

// List responses
let resp: ApiResponse<Vec<Post>> = deserialize_response(json_value)?;
for post in &resp.data {
    println!("{}: {}", post.id, post.text);
}
```

Available types: `Post`, `User`, `DmEvent`, `UsageData`, `UsageCreditsData`, `LikedResult`, `FollowingResult`, `DeletedResult`,
`RepostedResult`, `BookmarkedResult`, `BlockingResult`, `MutingResult`, `MediaUploadResponse`, `Includes`,
`ResponseMeta`, `ApiError`.

All structs include `#[serde(flatten)] extra: BTreeMap<String, Value>` for forward compatibility with new API fields.

## Troubleshooting

### X Platform Enrollment

If OAuth succeeds but reads like `xr whoami` fail with an error body containing `client-forbidden` or
`client-not-enrolled`, the current X platform fix is to move the app into the `Pay-per-use` package and use the
`Production` environment in the developer console. This is an X platform enrollment issue, not a local callback-listener
issue in `xr`.

The working recipe in the X developer console:

1. Go to `Apps` -> `Manage apps`.
2. Open the app.
3. Use `Move to package`.
4. Choose `Pay-per-use`.
5. Move the app to the `Production` environment.

Without that enrollment step, `xr whoami` and other `/2/*` reads can fail even when the OAuth callback and tokens are
valid.

## vs Go Original

| Feature                   | Go xurl          | xurl-rs                  |
| ------------------------- | ---------------- | ------------------------ |
| Language                  | Go               | Rust                     |
| Memory safety             | GC               | Compile-time             |
| Binary size               | ~15 MB           | ~8 MB                    |
| Shell completions         | Built-in (cobra) | Built-in (clap_complete) |
| `--output json`           | ❌                | ✅                        |
| `--quiet`                 | ❌                | ✅                        |
| `--no-interactive`        | ❌                | ✅                        |
| Structured exit codes     | ❌                | ✅                        |
| `NO_COLOR` support        | ❌                | ✅                        |
| `XURL_OUTPUT` env var     | ❌                | ✅                        |
| Typed response structs    | ❌                | ✅                        |
| `xr schema` (JSON Schema) | ❌                | ✅                        |

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
