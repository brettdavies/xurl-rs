# xurl-rs

[![Crates.io](https://img.shields.io/crates/v/xurl-rs.svg)](https://crates.io/crates/xurl-rs)
[![CI](https://github.com/brettdavies/xurl-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/brettdavies/xurl-rs/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue.svg)](#license)

A fast, ergonomic CLI for the X (Twitter) API. OAuth1, OAuth2 PKCE, Bearer auth. Media upload. Streaming. Agent-native.

Rust port of [xurl](https://github.com/xdevplatform/xurl): faster, type-safe, with shell completions and
machine-readable output. An independent project, not affiliated with, endorsed by, or maintained by X.

## Install

Every method installs the same binary, named `xr`. Verify with `xr --version`.

### Homebrew

```bash
brew tap brettdavies/tap
brew install xurl-rs
```

The formula links `xurl-rs` as an alias, so the formula name runs too. The documentation and the shell completions use
`xr`.

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

## Before you start

`xr` calls the X API v2 with an app of your own. The repository README walks through creating it under
[Before you start: an X app](https://github.com/brettdavies/xurl-rs#before-you-start-an-x-app); the OAuth2 redirect URI
to register is `http://localhost:8080/callback`, which is where `xr auth oauth2` listens. The X API is pay-per-use:
every request draws credits at the rates on X's [pricing page](https://docs.x.com/x-api/getting-started/pricing). `xr`
is free; the calls it makes are not.

## Quick Start

```bash
# Set up OAuth2 (browser-based, 30 seconds)
xr auth apps add myapp --client-id YOUR_ID --client-secret YOUR_SECRET
xr auth oauth2

# Post
xr post "Hello from xurl-rs!"

# Read
xr read 1585341984679469056

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
xr reply 1585341984679469056 "Nice!"           # Reply
xr reply https://x.com/elonmusk/status/1585341984679469056 "Nice!" # Reply by URL
xr quote 1585341984679469056 "My take"         # Quote
xr delete 1585341984679469056                  # Delete
```

### Reading

```bash
xr read 1585341984679469056                    # Read a post
xr search "golang" -n 20                       # Search (1-100 results)
xr whoami                                      # Your profile
xr user @elonmusk                              # Look up user
xr timeline                                    # Home timeline
xr mentions                                    # Your mentions
```

### Engagement

```bash
xr like 1585341984679469056                    # Like
xr unlike 1585341984679469056                  # Unlike
xr repost 1585341984679469056                  # Repost
xr unrepost 1585341984679469056                # Undo repost
xr bookmark 1585341984679469056                # Bookmark
xr unbookmark 1585341984679469056              # Remove bookmark
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
xr unmute @user                                # Unmute
xr muted                                       # Users you have muted
xr block @user                                 # Block
xr unblock @user                               # Unblock
xr blocked                                     # Users you have blocked
```

### Direct Messages

```bash
xr dm @user "Hey!"                             # Send DM
xr dms                                         # List DMs
```

### Usage and Validation

```bash
xr usage                                       # Post-cap usage for the project
xr usage credits                               # Credits-based usage
xr validate --schema post < post.json          # Validate JSON against a response schema
xr examples                                    # Curated invocation examples
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

Inspect the effective redirect URI for an app (or the default app when `NAME` is omitted). The output shows the resolved
URI, its source (`env-var` | `app-config` | `built-in-default`), and the stored URI when an env var is overriding it:

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

### Token Store Location

Credentials live in `~/.xurl`. Set `XURL_TOKEN_STORE=<path>` to point `xr` at another file; the OAuth2 headless pending
state (`<path>.pending`) follows it. The variable applies to the binary only: a program using `xdk` passes the path to
`Auth::new_with_store_path` and builds its client with `Client::new`.

## Agent-Native Features

Built for AI agents and automation:

### Skill Bundle

```bash
xr skill install claude_code                   # Clone the bundle into a host's skills directory
xr skill update --all                          # Refresh every install in place
```

`xr skill install <host>` clones the [skill bundle](https://github.com/brettdavies/xurl-rs-skill) into a host's
canonical skills directory, so the command surface, the auth paths, and the error contract are discoverable without a
prompt. Hosts: `claude_code`, `codex`, `cursor`, `factory`, `kiro`, `opencode`. `xr skill update --all` refreshes every
install in place, and both verbs take `--dry-run`.

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

`xr --output json auth status` and `xr --output json auth apps list` emit `{"status": "ok", "apps": [...]}`, carrying
one object per registered app under `apps`. An empty store emits `"apps": []`, so the array is iterable without a
zero-app special case. Per-app fields:

- `name`: app name.
- `client_id_hint`: first eight characters of the `client_id`, for visual identification without leaking the full ID.
- `redirect_uri`: the effective redirect URI for this app.
- `redirect_uri_source`: kebab-case provenance: `env-var` | `app-config` | `built-in-default`.
- `redirect_uri_stored`: only present when the `REDIRECT_URI` environment variable overrides a stored app value; carries
  the stored value so precedence is auditable.
- `oauth2_users`: array of usernames with OAuth2 tokens stored under this app.
- `oauth1`: boolean: OAuth1 credentials are stored for this app.
- `bearer`: boolean: a bearer token is stored for this app.
- `default`: boolean: this is the default app.
- `oauth2_unnamed`: only present when `true`; indicates an unnamed-user OAuth2 token is stored after a refresh where
  `/2/users/me` failed and no username was supplied.

```bash
xr --output json auth status | jq '.apps[] | select(.default) | .name'
```

### Quiet Mode

```bash
xr --quiet post "Hello"                        # No progress indicators
xr -q search "topic"                           # Short form
```

### Non-Interactive Mode

```bash
xr whoami --no-interactive                     # Error instead of prompt
# Exit code 77 with reason `auth-required` when no credentials are stored
```

### Structured Exit Codes

| Code | Meaning                                                     | Agent Action                                                  |
| ---- | ----------------------------------------------------------- | ------------------------------------------------------------- |
| 0    | Success                                                     | Continue                                                      |
| 1    | General error                                               | Log and handle                                                |
| 2    | Invalid arguments, unknown command, or auth-method mismatch | Fix the flag, read `suggestion`, or pick an accepted `--auth` |
| 3    | Rate limited                                                | Retry with backoff                                            |
| 4    | Not found                                                   | Resource doesn't exist                                        |
| 5    | Network error                                               | Check connectivity                                            |
| 77   | Auth required                                               | See Authentication; agents: read `next_step`                  |

### Recovering From an Auth Failure

Every structured error carries a `next_step` object an agent can act on without parsing prose. Exit 77 looks like this
on a machine with nothing registered:

```json
{
  "status": "error",
  "reason": "auth-required",
  "exit_code": 77,
  "message": "Auth Error: NoAuthMethod: no authentication method available",
  "next_step": {
    "action": "register-app",
    "template": "xr auth apps add <name> --client-id <client-id> --client-secret <client-secret>"
  }
}
```

`action` comes from a closed set: `register-app`, `sign-in`, `select-app`, `inspect-store`, `enroll-app`, and
`show-help`, which an `unknown-command` envelope carries with the help of the nearest command (`xr auth status --help`
for `xr auth statsu`). A step carries either a `command`, runnable verbatim, or a `template` with angle-bracket
placeholders only the caller can fill. Text mode prints the same advice as prose instead; the two need not match word
for word.

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

## Library

Upgrading from a 3.x library dependency:
[`docs/migrating/v4.0.0.md`](https://github.com/brettdavies/xurl-rs/blob/main/docs/migrating/v4.0.0.md).

The X API client behind `xr` is its own crate, `xdk-rs`: credentials in code, one typed call per endpoint, and the same
`~/.xurl` token store this tool writes. See
[crates/xdk/README.md](https://github.com/brettdavies/xurl-rs/blob/main/crates/xdk/README.md) and
[docs.rs](https://docs.rs/xdk-rs).

## Troubleshooting

### X Platform Enrollment

If OAuth succeeds but reads like `xr whoami` fail with an error body containing `client-forbidden` or
`client-not-enrolled`, the app needs X's enrollment step, which is a developer-console setting rather than anything in
`xr`. The recipe is in the repository README under
[X Platform Enrollment](https://github.com/brettdavies/xurl-rs#x-platform-enrollment).

## Relationship to xurl

`xr` is an independent Rust port of [`xdevplatform/xurl`](https://github.com/xdevplatform/xurl), X's own Go CLI. It
keeps that tool's shape: curl-style raw requests, the same auth flows, chunked media upload, and shortcut commands over
the common endpoints.

Where it goes further is the machine-readable side: seven output formats, a typed error envelope with structured exit
codes, and `xr schema` for response types. It does not port the webhook and `ngrok` surface.

Where behavior diverges on purpose,
[`KNOWN_DIFFERENCES.md`](https://github.com/brettdavies/xurl-rs/blob/main/KNOWN_DIFFERENCES.md) names each case and why.

| Feature                   | Go xurl          | xurl-rs                  |
| ------------------------- | ---------------- | ------------------------ |
| Language                  | Go               | Rust                     |
| Memory safety             | GC               | Compile-time             |
| Binary size               | ~15 MB           | ~8 MB                    |
| Shell completions         | Built-in (cobra) | Built-in (clap_complete) |
| `--output json`           | No               | Yes                      |
| `--quiet`                 | No               | Yes                      |
| `--no-interactive`        | No               | Yes                      |
| Structured exit codes     | No               | Yes                      |
| `NO_COLOR` support        | No               | Yes                      |
| `XURL_OUTPUT` env var     | No               | Yes                      |
| Typed response structs    | No               | Yes                      |
| `xr schema` (JSON Schema) | No               | Yes                      |

## Contributing

See [CONTRIBUTING.md](https://github.com/brettdavies/xurl-rs/blob/main/CONTRIBUTING.md) for dev setup, the branch and PR
flow, the error contract, and a recipe for exercising `xr` against X's API Playground without an account. Release
procedures live in [RELEASES.md](https://github.com/brettdavies/xurl-rs/blob/main/RELEASES.md).

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/brettdavies/xurl-rs/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/brettdavies/xurl-rs/blob/main/LICENSE-MIT) at your option.
