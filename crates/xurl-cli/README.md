# xurl-rs

[![Crates.io](https://img.shields.io/crates/v/xurl-rs.svg)](https://crates.io/crates/xurl-rs)
[![CI](https://github.com/brettdavies/xurl-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/brettdavies/xurl-rs/actions/workflows/ci.yml)
[![agent-native](https://anc.dev/badge/xr.svg)](https://anc.dev/score/xr)
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
`xr`. It installs the pre-built archive this repository's release publishes for your platform, checked against the
checksum the formula pins, so nothing is compiled. The tap pins a checksum only after it has verified the archive
against the release's build-provenance attestation;
[Verifying an archive](https://github.com/brettdavies/homebrew-tap#verifying-an-archive) in the tap's README has the
command that repeats the check.

### Pre-built Binary

Download from [GitHub Releases](https://github.com/brettdavies/xurl-rs/releases) for Linux, macOS, and Windows.

Each archive carries a build-provenance attestation and an attested SBOM, signed by the release workflow. To check that
an archive you downloaded was built by it, from this repository:

```bash
gh attestation verify xurl-rs-x86_64-unknown-linux-gnu.tar.gz --repo brettdavies/xurl-rs \
  --signer-workflow brettdavies/.github/.github/workflows/rust-release.yml
```

Releases up to 4.2.1 carry no attestation; compare those against the release's `sha256sum.txt`.

### Cargo

```bash
cargo install --locked xurl-rs
```

`--locked` builds with the dependency versions the release was tested against. Without it, cargo resolves the newest
compatible versions at install time, and with cc 1.6.0 that breaks the aws-lc-sys build whenever `CFLAGS` carries an
optimization flag such as `-O2` ([aws/aws-lc-rs#1252](https://github.com/aws/aws-lc-rs/issues/1252)).

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
# Set up OAuth2 (browser-based, 30 seconds). The secret is piped from a vault, so it never reaches argv.
op read 'op://<vault>/<item>/client_secret' | xr auth apps add myapp --client-id YOUR_ID --client-secret-file -
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
xr read-posts 20 1585341984679469056           # Up to 100 posts in one request
xr search "golang" -n 20                       # Search (10-100 results)
xr search-users "rust lang"                    # Find accounts by keyword
xr search-counts rustlang --granularity day    # Matching posts per day
xr likers 1585341984679469056                  # Who liked a post
xr reposters 1585341984679469056               # Who reposted a post
xr quotes 1585341984679469056                  # Posts that quote a post
xr whoami                                      # Your profile
xr whoami --cached                             # Your id, username, and name from the store; no request
xr user @elonmusk                              # Look up user
xr user-posts @XDevelopers                     # A user's posts, newest first
xr timeline                                    # Home timeline
xr mentions                                    # Your mentions
```

### Engagement

```bash
xr like 1585341984679469056                    # Like
xr unlike 1585341984679469056                  # Unlike
xr hide 1585341984679469056                    # Hide a reply to your post
xr unhide 1585341984679469056                  # Show a hidden reply again
xr repost 1585341984679469056                  # Repost
xr unrepost 1585341984679469056                # Undo repost
xr bookmark 1585341984679469056                # Bookmark
xr unbookmark 1585341984679469056              # Remove bookmark
xr bookmarks                                   # List bookmarks
xr bookmark-folders list                       # Your bookmark folders
xr bookmark-folders get 1146654567674912769    # Posts in a folder, by ID
xr bookmark-folders add "Reading list"         # Create a folder
xr likes                                       # List likes
```

A folder lists its posts by ID alone; `xr read-posts` turns up to 100 of them into posts in one request, as in
`xr bookmark-folders get <id> --output json | jaq -r '.data[]?.id' | xargs xr read-posts`. X's API creates folders and
lists them, and has no endpoint that moves a bookmark into a folder or deletes one, so those stay in the X app.

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

These flags (`-X`, `-H`, `-d`, `--auth`, `-u`, `-t`, `-s`, `-F`) belong to a raw request. Given before a command word,
one is refused at exit 2 with reason `invalid-args`, because the command reads only its own copy and would otherwise
run without it: `xr -u work whoami` names `xr whoami -u work` as its step, and a flag the command does not take names
the command's help.

### Media Upload

```bash
xr media upload video.mp4                      # Upload media; waits up to 60 s for a video to process
xr media upload video.mp4 --wait=300           # Wait up to five minutes
xr media upload video.mp4 --wait=false         # Return after FINALIZE
xr media status 1234567890                     # Check status
xr media status 1234567890 --wait=120          # Wait on processing, up to two minutes
xr media alt-text 1234567890 "A dog asleep"    # Set alt text on an upload
xr media upload captions.srt --media-type text/srt --category subtitles
xr media subtitles add 1234567890 1234567891 --language en --name English
xr media subtitles remove 1234567890 --language en
```

`--wait` takes its value after `=`: bare or `true` waits up to 60 seconds, a number that many seconds, and `false` or `0`
not at all. `media status` does not wait unless asked. `media upload` waits by default, for a video and for any other
upload X reports as still processing when it is finalized, such as an animated GIF; an upload X reports as ready
returns at once. A wait ends when X reports the job finished or failed, or when the status carries no processing
information, as an image's does.

An upload answers one JSON document: FINALIZE's response, with `data.processing_info` holding the final state when the
upload waited for one.

A wait that reaches its deadline exits 1 with reason `processing-timeout`. The upload itself is intact, and X keeps a
media id for 24 hours, so the envelope carries the id and the command that resumes the wait for twice as long:

```json
{
  "status": "error",
  "reason": "processing-timeout",
  "exit_code": 1,
  "message": "media 1234567890 was still processing when the 60-second wait ended",
  "media_id": "1234567890",
  "next_step": {
    "action": "resume-wait",
    "command": "xr media status 1234567890 --wait=120"
  }
}
```

### Broadcasts

```bash
xr broadcasts moderators list                      # Who moderates your broadcast chats
xr broadcasts moderators add @helper               # Add a chat moderator
xr broadcasts sources list                         # Your RTMP and RTMPS ingest points
xr broadcasts sources add "Studio A" --region us-east-1
xr broadcasts sources get 1x2y3z4w5v6u             # Whether the source is receiving video
xr broadcasts sources rename 1x2y3z4w5v6u "Studio B"
xr broadcasts sources remove 1x2y3z4w5v6u          # Delete; asks first (--force skips)
```

A stream source is a persistent ingest point: point an encoder at its `rtmps_url` (or `rtmp_url`) with its `id` as the
stream key, then poll `get` until `is_stream_active` is true. The `id` is the stream key itself, so whoever holds it can
stream to the source; keep it out of logs you share. `--region` takes an ingest region, and
`xr broadcasts sources add --help` lists them. The API lists the regions in prose, so `xr` passes the value through
rather than refusing one it does not know.

### Calls

```bash
xr calls scheduled list                            # Calls on your calendar in the next 24 hours
xr calls scheduled list --from 2026-11-02T00:00:00Z --to 2026-11-09T00:00:00Z
xr calls scheduled get 1849302018273645            # One call, with its invitees and their answers
xr calls scheduled add --start 2026-11-02T16:00:00Z --title Standup --duration 15 --invite 2244994945
xr calls scheduled update 1849302018273645 --start 2026-11-02T17:00:00Z
xr calls scheduled cancel 1849302018273645         # Cancel; asks first (--force skips)
xr calls scheduled rsvp 1849302018273645 accepted  # Or declined, tentative
xr calls get 8a6f1c2e                              # Who is in a call, and whether you may join
```

Times are RFC 3339 with a zone, such as `2026-11-02T16:00:00Z`, and one that is not is refused before any request.
`--invite` takes a numeric user ID (`xr user @handle` prints one) and repeats for each person, at most 25 on a new
call; on `update` it replaces the whole invite list. Invitees get an email about a new, changed, or cancelled call
unless `--no-notify` is given. A recurring call takes `--recurrence` (an RFC 5545 RRULE such as `FREQ=WEEKLY;BYDAY=MO`)
and `--time-zone`. To change, cancel, or answer some occurrences of it, name one with `--occurrence`, its
`occurrence_start_time` as `list` prints it, and `--scope this` or `--scope this-and-following`; with neither flag, the
request applies to every occurrence. `update` with no field to change is refused with `no-call-changes`.

The calls endpoints take a user login with the `calls.read` and `calls.write` scopes, which every `xr auth oauth2`
sign-in requests. A login made before `xr` requested them lacks them, and X refuses the request until you sign in again
with `xr auth oauth2`. A call's `id` and a scheduled call's `call_id` are also the call's join secret.

`xr calls` is a preview. X marks the Calls API as in development and not yet generally available, so its requests and
the scopes it takes may change before it is; `xr calls` follows the vendored spec. Starting, joining, and leaving a live
call (`POST /2/calls`, `/2/calls/{id}/join`, `/2/calls/{id}/leave`) have no command: joining hands out a short-lived
token for a real-time audio WebSocket that `xr` does not drive. They stay reachable in raw mode.

### Webhooks

```bash
xr webhooks list                                   # Webhooks the app has registered
xr webhooks add https://example.com/webhook        # Register a URL; X checks it first
xr webhooks validate 1146654567674912769           # Ask X to check the URL again
xr webhooks remove 1146654567674912769             # Delete; asks first (--force skips)
xr webhooks replay 1146654567674912769 --from 202601150000 --to 202601151200
xr webhooks subscriptions add 1146654567674912769  # Deliver your account's activity there
xr webhooks subscriptions list 1146654567674912769 # Whose activity it receives
xr webhooks stream-links add 1146654567674912769   # Deliver your filtered stream there
xr webhooks listen --port 8080                     # Receive: answer X's check, print events
xr webhooks listen --forward http://127.0.0.1:3000/events --checkpoint listen.json
xr webhooks listen -- ./handle-event               # Run a program per event, body on stdin
xr webhooks replay 1146654567674912769 --from-checkpoint listen.json
```

A webhook is a public URL X delivers events to, in place of a poll or an open stream. Before X is involved, prove the
wiring: once `listen` is up it hands out a signed sample delivery. Under `--output json` the first line carries
`test_command`, a `curl` invocation quoted for a POSIX shell, and `test_request`, the same request as method, URL,
headers, and body for a caller with no POSIX shell; in text the `curl` invocation goes to stderr. The sample is signed
with the app's own secret, so it passes `listen`'s check and anything you forward to, and its body carries
`"xr_test_event": true`: ignore events that carry it. Run the command against the public URL too, once it is exposed.

Three things have to be in place, in this order:

1. **A receiver X can reach.** `xr webhooks listen` binds `127.0.0.1:8080`, answers X's Challenge-Response Check on
   `/webhook`, and prints each event as one JSON line on stdout. X delivers only to a public HTTPS URL with no port, so
   expose that local address through a tunnel or reverse proxy of your own, such as `cloudflared`, the `ngrok` CLI, or
   Tailscale Funnel. `xr` opens no tunnel. A port another process holds answers with reason `address-in-use` and the
   same invocation on `--port 0`.
2. **The URL registered.** `xr webhooks add <URL>` registers it and prints the webhook's id. X sends its check to the
   URL before it answers, so the listener has to be up and exposed first. A URL that is not HTTPS, or that names a port,
   is refused before any request.
3. **Something subscribed to it.** `xr webhooks subscriptions add <WEBHOOK_ID>` subscribes the signed-in account's
   activity. `xr webhooks stream-links add <WEBHOOK_ID>` delivers the posts your filtered-stream rules match.

X bills each event it delivers; its pricing page lists the rate per event type.

`listen` prints only what it has verified. X signs its check and every event with the app's OAuth2 client secret, or
with the OAuth1 consumer secret for an app that has no client secret. `listen` uses the client secret when the app has
one, and `--secret oauth1` names the consumer secret instead. An event whose signature is wrong or missing is answered
401 and not printed; `--allow-unsigned` also prints a POST that carries no signature, for a local test with `curl`. It
runs until interrupted, or exits 0 once `--max-events <N>` events have printed, and under `--output json` its first line
is `{"status":"listening","url":...}`.

#### Forward events to your app

`--forward <URL>` POSTs each verified event to an `http` or `https` URL of yours, and everything after `--` is a program
`listen` runs once per event. Give either or both: each target receives every event, in the order X delivered them, and
X's answer does not wait for a forward. A target that falls far behind fills its queue of 256 events, and from then on
X's deliveries wait for room rather than being dropped.

- **The URL** receives the event's body exactly as X sent it, with X's `Content-Type` and signature header, so it can
  verify the event with the app's secret the way `listen` does. Each `-H 'Name: Value'` is added, except that `Host`,
  `Content-Length`, and `Idempotency-Key` are `listen`'s own and a `-H` naming one is refused. A 2xx from that URL
  delivers. No redirect is followed, so a 3xx is a failed forward: name the final URL. A URL that carries a username or
  password is refused; pass the credential with `-H 'Authorization: ...'`. A malformed `-H` is refused by its position
  before anything binds, and no header value is ever printed.
- **The program** gets the body on stdin and runs with no shell, so an event cannot become a command. Its stdout and
  stderr go to `listen`'s stderr a whole line at a time, with a newline added to output it leaves unfinished, so a
  record never shares a line with them and stdout keeps one line per event. Exit 0 delivers.
- **Repeats share a key.** The URL gets `Idempotency-Key`, the SHA-256 of the event's bytes as quoted lowercase hex, and
  the program gets the same hex in `XR_EVENT_SHA256`. A retry and a replay of the same event carry the same key, so a
  target can drop what it has seen.
- **Retries.** A failed forward is tried four times in all, 1, 2, then 4 seconds apart, each attempt cut off at
  `--forward-timeout` (30 seconds unless set). A connection failure, a timeout, 408, 429, and 5xx are retried; another
  status is final. A program is retried on a non-zero exit or a timeout, and a run past the timeout is stopped with
  every process it started.
- **Records on stderr.** Each delivery writes one record, never on stdout and never changing `listen`'s exit status:

  ```json
  {"status":"forwarded","target":"url","attempts":1,"http_status":200,"elapsed_ms":4,"received_at":"2026-10-22T15:04:10Z","event_sha256":"3f9a…"}
  {"status":"forward-failed","target":"program","attempts":4,"cause":"the program exited with 3","received_at":"2026-10-22T15:04:10Z","event_sha256":"3f9a…","side_effects":"printed, not delivered","next_step":{"action":"run-command","template":"xr webhooks replay <WEBHOOK_ID> --from 202610221504 --to 202610221505"}}
  ```

  A program's record carries `exit_status` where a URL's carries `http_status`. A failed record's step is the replay of
  the event's minute and the next, with the webhook's id to fill in. Text output prints the same as prose.
- **Shutdown.** On Ctrl+C, SIGTERM, or `--max-events`, `listen` stops taking events, releases the port, and gives the
  targets one attempt timeout plus 7 seconds to deliver what is queued. A second Ctrl+C ends that at once. Whatever is
  left is counted in a `{"status":"forward-incomplete","target":...,"undelivered":N}` record.

#### Recover what a restart missed

X documents no redelivery: an event that arrives while `listen` is down is lost unless you ask X to replay it. Start
`listen` with `--checkpoint <FILE>`, and it records, at most once a minute, when the oldest event not yet delivered to
every target arrived, or the newest event when nothing is waiting. After a restart, look at the window first, then
request it:

```bash
xr --dry-run webhooks replay 1146654567674912769 --from-checkpoint listen.json   # from, to, clamped; sends nothing
xr webhooks replay 1146654567674912769 --from-checkpoint listen.json
```

The window runs from the checkpoint's minute to the current one. X replays at most the last 24 hours: an older
checkpoint is moved up to the earliest minute X can replay, and the answer says `clamped: true` with `lost_from`, the
first minute that cannot be recovered. X bills the events it delivers again, so the dry run is the place to check the
window. Replayed events reach your targets like any other, and the checkpoint covers events still queued when `listen`
stopped, so a target sees repeats; drop them by `Idempotency-Key` or `XR_EVENT_SHA256`. A checkpoint that cannot be read
is refused with reason `io`, and `--from` with `--to` names a window by hand. A checkpoint `listen` cannot write is
reported on stderr as `checkpoint-failed`, and `listen` goes on.

`list`, `add`, `validate`, and `remove` take any stored credential. `replay`, `stream-links`, and `subscriptions count`,
`list`, and `remove` take a bearer token. `subscriptions add` and `check` act for the signed-in account and take a user
login. Each `remove` asks before it deletes, and `replay` takes its window as twelve digits, `yyyymmddhhmm` in UTC.

The X Activity API (`/2/activity/...`) has no command; raw mode reaches it.

## Authentication

Every secret flag (`--client-secret`, `--consumer-secret`, `--access-token`, `--token-secret`, `--bearer-token`) has a
`--<name>-file` twin that reads the value from a file, or from stdin when the path is `-`. A secret passed that way
never appears in the process's arguments or in shell history, and its one trailing line ending is dropped. Stdin carries
one value, so a command that takes several secrets reads at most one from `-` and the rest from files. `-` is refused
when stdin is a terminal: pipe the secret in, or pass a path. A file or a pipe that holds nothing is refused too, at
exit 2: that is what a vault command prints when it fails, and nothing is stored.

### OAuth2 (Recommended)

```bash
op read 'op://<vault>/<item>/client_secret' | xr auth apps add myapp --client-id ID --client-secret-file -
xr auth oauth2                                 # Opens browser
xr auth oauth2 alice                           # Skip /2/users/me; save under "alice"
xr auth oauth2 --app myapp alice               # Same, against a specific app
xr auth oauth2 --scopes tweet.read,users.read  # Request only these scopes
```

A sign-in requests every scope `xr` can use. `--scopes` narrows it to a comma-separated subset; `offline.access` is
always added, because without a refresh token the login ends within hours. A name X does not define is rejected before
any request, with the valid names listed. In the headless flow the flag goes on step 1, and step 2 takes the scopes step
1 saved.

`xr auth oauth2` accepts an optional `[USERNAME]` positional. If X's `/2/users/me` endpoint is unreliable, supplying the
handle explicitly skips that lookup and stores the resulting token under the known username so shortcut commands resolve
without `/me`.

### OAuth1

```bash
op read 'op://<vault>/<item>/token_secret' | xr auth oauth1 \
  --consumer-key CK \
  --consumer-secret-file consumer-secret.txt \
  --access-token-file access-token.txt \
  --token-secret-file -
```

### Bearer Token (App-Only)

```bash
op read 'op://<vault>/<item>/bearer_token' | xr auth app --bearer-token-file -
```

### Multi-App Management

```bash
xr auth apps add prod --client-id ... --client-secret-file ...
xr auth apps add dev --client-id ... --client-secret-file ...
xr auth apps list
xr auth default prod                           # Set default
xr --app dev whoami                             # Per-request override
```

Register an app with a custom OAuth2 callback URL via `--redirect-uri`:

```bash
xr auth apps add prod \
  --client-id ID \
  --client-secret-file client-secret.txt \
  --redirect-uri http://localhost:8080/callback
```

Update credentials or the stored redirect URI on an existing app:

```bash
xr auth apps update prod --client-id NEW_ID --client-secret-file client-secret.txt
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

Credentials live in `~/.xurl/auth.yml`, the file Go `xurl` 1.3.0 and later use, so the two tools share one store.

A store kept as a single file at `~/.xurl`, which is where `xr` 4.3.0 and earlier and Go `xurl` before 1.3.0 put it, is
moved into that directory by the first command that opens it. In text output the command reports the move on stderr
(`Migrated <old path> to <new path>`). The move is one-way, and `xr` 4.3.0 and earlier cannot read the directory, so
upgrade every copy of `xr` that shares the store. A `~/.xurl` that is a symbolic link to a file is used where it points.

Beside each login the store keeps the id of the X account it belongs to, with the account's username and display name
from the same answer. `xr` writes them from `/2/users/me` answered under that login: at sign-in, at the refresh of a
login that has no id, on `xr whoami`, and on the first command that needs the id. A command that acts as you (`like`,
`follow`, `timeline`, `bookmarks`, and the rest of them) then reads the id from the store, where it asked X on every
run. The account sits beside the credential it was asked with and goes when that credential is replaced, so it cannot
name another account. The username and display name of an answer are also written beside every other login the store
holds for the same account id, in any app, so a rename seen through one login shows through all of them. A login stored
without a username gets its name the same way. Someone else's handle is looked up every time, because a handle can pass
to another account.

`xr whoami --cached` answers from the stored account with no request, as `{"data": {"id", "name", "username"}, "source":
"cache"}`. When the store holds no whole account for the login yet, it asks X once, answers with `"source": "api"`, and
stores the answer. `xr whoami` without the flag always asks X, for the live profile and its counts. The stored names are
what X answered when a login for the account last asked, so a rename shows once one of them asks again, as a live `xr
whoami` does. The token is not checked either: an expired login still answers, so `--cached` is no test of a credential,
and `xr --dry-run whoami` says whether the token has expired. `xr --dry-run whoami --cached` carries `source` too,
naming the one the real run would answer from.

Set `XURL_TOKEN_STORE=<path>` to point `xr` at another file; the OAuth2 headless pending state (`<path>.pending`)
follows it. The variable applies to the binary only: a program using `xdk` passes the path to
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
install in place, and both verbs take `--dry-run`. A host's destination follows that host's own config-directory
variable when it is set (for example `CLAUDE_CONFIG_DIR`; `xr --help` lists them all); otherwise it sits under
`XURL_SKILL_HOME` when that is set, and under `~` when it is not, where a base-directory variable the host follows (such
as `XDG_CONFIG_HOME` for OpenCode) relocates its part of the path. Codex's destination is `~/.agents/skills/xurl-rs`;
`xr skill update codex` also removes a copy at `~/.codex/skills/xurl-rs`, a deprecated location Codex still reads.

### Response Schema Discovery

```bash
xr schema --list                               # Discover all commands + response types
xr schema post                                 # Get JSON Schema for any command's output
xr schema --all                                # All schemas for MCP tool definitions
```

### Machine-Readable Output

```bash
xr --output json whoami                        # Raw JSON, no color
xr --output jsonl search "topic"               # The document on one line
export XURL_OUTPUT=json                          # Default to JSON
```

Shortcut commands print X's response under the names the vendored spec gives its keys. Where X still sends a key in its
legacy post vocabulary, the key is renamed to the spec's spelling and its value is left as X sent it:
`edit_history_tweet_ids` prints as `edit_history_post_ids`, `tweet_count` as `post_count`, `retweet_count` as
`repost_count`, `referenced_tweets` as `referenced_posts`, and `includes.tweets` as `includes.posts`. `--verbose` notes
each key it renamed. A raw `xr <URL>` request prints X's body unchanged.

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
- `oauth2_accounts`: one object per name in `oauth2_users`, in the same order: `login`, and once X has answered under
  that login, the account's `user_id`, `username`, and `display_name`. The text form prints the id and display name
  beside the login.
- `oauth1`: boolean: OAuth1 credentials are stored for this app.
- `oauth1_account`: only present when the store holds the id of the account the OAuth1 access pair belongs to:
  `user_id`, `username`, and `display_name`.
- `bearer`: boolean: a bearer token is available for this app, stored on it or supplied by `XURL_BEARER_TOKEN`.
- `bearer_source`: only present when `bearer` is `true`: `store` or `env`.
- `default`: boolean: this is the default app.
- `oauth2_unnamed`: only present when `true`; an OAuth2 login is stored without a username, because `/2/users/me` failed
  at the sign-in or refresh that stored it. `xr whoami`, or the next command that asks X who you are, stores it under
  the username X answers with.

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

### Dry Runs

`--dry-run` sends nothing and stores nothing, whichever command it is on: a write, a read, or a raw request with any
method. It checks the inputs and the stored credentials offline, and answers at exit 0.

```bash
xr --output json --dry-run post "Hello"
```

```json
{
  "auth": {
    "app": "myapp",
    "scheme": "oauth2",
    "token_expired": false,
    "username": "alice"
  },
  "body": "Hello",
  "command": "post",
  "exit_code": 0,
  "media_ids": [],
  "status": "dry_run",
  "would_succeed": true
}
```

- `would_succeed` is `false` when an input is invalid, or when no stored credential serves a request the command would
  send. `reason` says which, and `exit_code` is the code the real run would end on. A credential refusal also carries
  the keys of its error envelope (`message`, `endpoint`, `supported`, `available_in_app`) and the `next_step` the real
  run would name. A step that reruns the invocation keeps `--dry-run`, so following it sends nothing either.
- `auth` names the credential the request would go out under: the `app` it is stored under, its `scheme`, and for an
  OAuth2 login `username` and `token_expired`. A bearer token from `XURL_BEARER_TOKEN` outranks every stored one and
  belongs to no stored app, so it carries no `app`. An expired token is reported, not refreshed. Whether X still accepts
  the credential is not checked, since nothing is sent to find out.
- `confirmation_required: true` marks a destructive command that would stop to ask. The flag outranks `--force` and the
  prompt, so `xr --dry-run delete <id>` answers with or without `--force`.

A command that only reads local state, such as `auth status`, `schema`, or `version`, has nothing to hold back and
answers as it always does. Text output summarizes the same facts in lines: that nothing was sent, whether the real run
would succeed and, when it would not, the exit code and why, then the command's inputs, the credential, and whether it
would stop to ask. A refusal for credentials prints the advice the real run's error prints, which `--quiet` leaves out.

```text
$ xr --dry-run bookmark-folders add ""
Dry run of `bookmark-folders-add`: nothing was sent.
The real run would fail with exit code 1: The folder name is empty.
  name: ""
```

### Structured Exit Codes

| Code | Meaning                                                     | Agent Action                                                   |
| ---- | ----------------------------------------------------------- | -------------------------------------------------------------- |
| 0    | Success                                                     | Continue                                                       |
| 1    | General error                                               | Log and handle                                                 |
| 2    | Invalid arguments, unknown command, or auth-method mismatch | Fix the flag, read `suggestion`, or pick an accepted `--auth`  |
| 3    | Rate limited                                                | Wait `retry_after_secs`, or until `retry_at`, then retry       |
| 4    | Not found                                                   | Resource doesn't exist                                         |
| 5    | Network error, an unreadable local file, or a taken port    | Check connectivity, the path, or the port; `reason` says which |
| 77   | Auth required                                               | See Authentication; agents: read `next_step`                   |

A panic in any `xr` command writes the `internal` error in the active output format and exits 1: the `report-issue` step
names the issue tracker, the message keeps Rust's `panicked at <file>:<line>:<col>`, and a `backtrace` field appears
when `RUST_BACKTRACE` asks for one.

### Rate Limits

A 429 exits 3 with reason `rate-limited`. When X names the reset on that response, the envelope says when to retry, as a
delay and as a time, and the `next_step` names the step:

```json
{
  "status": "error",
  "reason": "rate-limited",
  "exit_code": 3,
  "message": "{\"status\":429,\"title\":\"Too Many Requests\"}",
  "retry_after_secs": 640,
  "retry_at": "2026-10-06T19:12:00Z",
  "next_step": {
    "action": "wait-and-retry",
    "docs": "https://docs.x.com/resources/fundamentals/rate-limits"
  }
}
```

`retry_after_secs` is the seconds left until the reset, zero once it has passed, and `retry_at` is the same moment in RFC
3339 UTC, for a caller that schedules the retry instead of sleeping. Both come from that response's own
`x-rate-limit-reset` header. A 429 that carries no such header has neither key and no `next_step`.

`xr` never retries on its own. `--wait-on-rate-limit` (`XURL_WAIT_ON_RATE_LIMIT`) makes it wait for the reset and send
the request once more, when the wait fits `--rate-limit-max-wait <SECS>` (`XURL_RATE_LIMIT_MAX_WAIT`, 60 by default). A
longer wait, or a 429 with no reset, fails at once, and the second response is final whatever it is.

```bash
xr --wait-on-rate-limit search "rustlang"                           # Wait up to 60 s, retry once
xr --wait-on-rate-limit --rate-limit-max-wait 900 search "rustlang" # Wait up to 15 minutes
```

### Recovering From an Auth Failure

A structured error that has a step the caller can take carries a `next_step` object an agent can act on without
parsing prose; [Which Errors Carry a Step](#which-errors-carry-a-step) lists them. Exit 77 looks like this on a machine
with nothing registered:

```json
{
  "status": "error",
  "reason": "auth-required",
  "exit_code": 77,
  "message": "Auth Error: NoAuthMethod: no authentication method available",
  "next_step": {
    "action": "register-app",
    "template": "<secret-command> | xr auth apps add <name> --client-id <client-id> --client-secret-file -"
  }
}
```

`action` comes from a closed set: `register-app`, `sign-in`, `select-app`, `inspect-store`, `enroll-app`, `resume-wait`,
`wait-and-retry`, `retry`, `fix-input`, `report-issue`, `confirm`, `run-command`, and `show-help`, which carries the
help to read: `xr post --help` for a usage error under `post`, and the help of the nearest command for a mistyped one
(`xr auth status --help` for `xr auth statsu`). `reason` is closed too. A newer release can add to either set, so treat
a value you do not recognize as your default branch. A step carries a `command`, runnable verbatim, or a `template` the
caller finishes or decides on first, and never both: a template holds angle-bracket placeholders only the caller can
fill, or, under `confirm`, the invocation that destroys once it runs. One whose recovery is not an `xr` invocation
carries neither, and `docs` when a page covers it: `enroll-app`, `wait-and-retry`, `retry`, `fix-input`, `report-issue`,
and `inspect-store` when a store command is what failed. Two `fix-input` steps carry an invocation: `listen` again on a
free port, under `address-in-use`, and the invocation with `-u`, `--auth`, or `-t` moved after the command word it came
before, under `invalid-args`. Text mode prints the same advice as prose instead; the two need not match word for word.

### Which Errors Carry a Step

| `reason`                     | `next_step.action`                                                                                       |
| ---------------------------- | -------------------------------------------------------------------------------------------------------- |
| `auth-required`              | `register-app`, `sign-in`, `select-app`, or `inspect-store`; none when X answers a 401                   |
| `client-credentials-missing` | `register-app` or `select-app`                                                                           |
| `token-store`                | `inspect-store` when the store file could not be loaded; none for a name it does not hold                |
| `auth-method-mismatch`       | `select-app`, `sign-in`, or `register-app` when the endpoint takes a login the app lacks; none otherwise |
| `forbidden`                  | `enroll-app` when X names enrollment; none otherwise                                                     |
| `rate-limited`               | `wait-and-retry` when the 429 names its reset; none otherwise                                            |
| `processing-timeout`         | `resume-wait`                                                                                            |
| `invalid-args`               | `show-help`; `fix-input` when `-u`, `--auth`, or `-t` came before a command word that takes it           |
| `unknown-command`            | `show-help`                                                                                              |
| `invalid-method`             | `show-help`                                                                                              |
| `invalid-url`                | `show-help`                                                                                              |
| `invalid-path-param`         | `show-help`                                                                                              |
| `unsupported-pagination`     | `show-help`                                                                                              |
| `invalid-json`               | `show-help`                                                                                              |
| `unknown-schema`             | `show-help`                                                                                              |
| `missing-host`               | `show-help`                                                                                              |
| `validation`                 | `show-help` for an argument `xr` refused; `fix-input` otherwise                                          |
| `no-tty`                     | `select-app`, as a template that takes the app's name                                                    |
| `confirmation-required`      | `confirm`, as a template: the same invocation with `--force`                                             |
| `network-error`              | `retry`                                                                                                  |
| `server-error`               | `retry`                                                                                                  |
| `git-clone-failed`           | `retry`                                                                                                  |
| `not-found`                  | `fix-input`                                                                                              |
| `invalid-request`            | `fix-input`                                                                                              |
| `validation-failed`          | `fix-input`                                                                                              |
| `io`                         | `fix-input`                                                                                              |
| `address-in-use`             | `fix-input`, carrying `listen` again on `--port 0`: a template when it hides a `-H` value                |
| `home-not-set`               | `fix-input`                                                                                              |
| `destination-is-file`        | `fix-input`                                                                                              |
| `remove-failed`              | `fix-input`                                                                                              |
| `git-not-found`              | `fix-input`                                                                                              |
| `destination-not-empty`      | `run-command`: `xr skill update <host>`                                                                  |
| `not-installed`              | `run-command`: `xr skill install <host>`                                                                 |
| `internal`                   | `report-issue`                                                                                           |
| `serialization`              | `report-issue`                                                                                           |
| `api-error`                  | `report-issue`                                                                                           |

Every reason has a row. The five actions that are not about credentials or help read as follows.

- `retry`: nothing was wrong with the request. Send it again; `docs` names X's page on response codes when X answered.
- `fix-input`: the input, or the local state the command read, is what failed, and the same invocation fails again. No
  `xr` command repairs it, with two exceptions. For a port another process holds, the step is `listen` again with
  `--port 0`, a `command` when the invocation gave no `-H` and a `template` that hides each header's value when it did.
  For `-u`, `--auth`, or `-t` given before a command word that takes it, the step is the invocation with the flag after
  the word. `docs` names X's page on response codes for a 400, a 404, or a 422.
- `report-issue`: the fault is in `xr` or its library. `docs` is the issue tracker.
- `confirm`: a destructive command ran without its confirmation. The step is a `template`, never a `command`: it is
  the invocation that destroys once it runs, so an agent that runs every `command` it is handed does not confirm by
  accident.
- `run-command`: one other command repairs this, and `command` is that invocation. A `forward-failed` record from
  `xr webhooks listen` carries it as a `template`: the replay of the event's minute, with `<WEBHOOK_ID>` to fill in.

### NO_COLOR Support

```bash
NO_COLOR=1 xr whoami                           # Disable color (no-color.org)
```

## Stability

`xr` follows [SemVer 2.0.0](https://semver.org/). Its public API is the machine contract a script or an agent depends
on:

- **Invocation:** command names and aliases, flags, positional forms, and every environment variable `xr --help` lists.
- **Exit codes:** the table under [Structured Exit Codes](#structured-exit-codes).
- **Structured output:** every shape `--output json|jsonl|ndjson|yaml|csv|tsv` prints, meaning the success responses
  `xr schema` describes and the error envelope, including its `reason` and `next_step.action` sets.
- **Stored state:** the `~/.xurl/auth.yml` token store format.

Text-mode output, help text, colors, `--verbose` diagnostics, suggestion wording, and raw-mode response bodies are not
contract. The one text-mode exception is the plain `xr --version` line, `xr X.Y.Z`, which scripts parse. A script that
needs any other stable output reads a structured format.

How a change arrives:

- **Patch:** fixes only, including behavior brought back to the documented contract.
- **Minor:** anything new that breaks nothing, such as a command, a flag, an environment variable, or an output field,
  and also a deprecation or a raised minimum Rust version. `reason` and `next_step.action` are closed sets that gain
  members in minors, so treat a value you do not recognize as your default branch.
- **Major:** anything in the contract removed, renamed, or retyped, or its documented meaning changed. A name that has
  to change is added in a minor and deprecated there, then removed in a major that batches removals. Each major's
  breaks are written up with before-and-after invocations under
  [`docs/migrating`](https://github.com/brettdavies/xurl-rs/tree/main/docs/migrating).

[`RELEASES.md` § Versioning](https://github.com/brettdavies/xurl-rs/blob/main/RELEASES.md#versioning) is the full
policy.

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
`~/.xurl/auth.yml` token store this tool writes. See
[crates/xdk/README.md](https://github.com/brettdavies/xurl-rs/blob/main/crates/xdk/README.md) and
[docs.rs](https://docs.rs/xdk-rs).

## Troubleshooting

### X Platform Enrollment

If OAuth succeeds but reads like `xr whoami` fail with an error body containing `client-forbidden` or
`client-not-enrolled`, the app needs X's enrollment step, which is a developer-console setting rather than anything in
`xr`. The recipe is in the repository README under
[X Platform Enrollment](https://github.com/brettdavies/xurl-rs#x-platform-enrollment).

### Token Store Could Not Be Read

`xr` keeps every credential in one YAML file, `~/.xurl/auth.yml` unless `XURL_TOKEN_STORE` names another path. When that
path exists and cannot be loaded, a command that manages the store exits 77 with reason `token-store`, a command that
needs a credential exits 77 with reason `auth-required`, and both carry `next_step.action` `inspect-store`. `xr auth
status` names the path in its message. `xr` writes nothing to a store it could not load, so the file stays as it was
found.

What makes a store unloadable:

- **It is not a store.** The file is not YAML `xr` can read: a hand edit, or a file another tool put there.
- **It cannot be opened.** Its permissions deny the user running `xr`.
- **It is a directory.** `XURL_TOKEN_STORE` names a directory, such as `~/.xurl` itself. Point it at the file inside,
  or unset it.

To recover, repair the file, or move it aside and register again (`xr auth apps add`, then `xr auth oauth2`). To keep
`xr`'s credentials apart from Go `xurl`'s, set `XURL_TOKEN_STORE` to a path of `xr`'s own.

## Relationship to xurl

`xr` is an independent Rust port of [`xdevplatform/xurl`](https://github.com/xdevplatform/xurl), X's own Go CLI. It
keeps that tool's shape: curl-style raw requests, the same auth flows, chunked media upload, and shortcut commands over
the common endpoints.

Where it goes further is the machine-readable side: seven output formats, a typed error envelope with structured exit
codes, and `xr schema` for response types. Its webhook receiver opens no `ngrok` tunnel; the caller brings the public
URL, and `xr` adds the commands that register and subscribe a webhook.

Where behavior diverges on purpose,
[`KNOWN_DIFFERENCES.md`](https://github.com/brettdavies/xurl-rs/blob/main/KNOWN_DIFFERENCES.md) names each case and why.

| Feature                   | Go xurl          | xurl-rs                  |
| ------------------------- | ---------------- | ------------------------ |
| Language                  | Go               | Rust                     |
| Memory safety             | GC               | Compile-time             |
| Binary size               | ~15 MB           | ~10 MB                   |
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
