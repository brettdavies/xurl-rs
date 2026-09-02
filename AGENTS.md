---
name: xurl-rs
binary: xr
description: Fast, ergonomic CLI for the X (Twitter) API. Rust port of the Go xurl, with OAuth1 / OAuth2-PKCE / Bearer auth, 30 high-level shortcut commands, chunked media upload, and streaming.
homepage: https://github.com/brettdavies/xurl-rs
repository: https://github.com/brettdavies/xurl-rs
---

# AGENTS.md

## Running xr

The crate is `xurl-rs`. The installed binary is `xr`. The library re-exports under `xurl`.

```bash
# Generic request — full control over verb, path, body, headers
xr request GET /2/users/me
xr request POST /2/tweets --data '{"text":"hello"}'
xr request GET '/2/tweets/search/recent?query=from:jack'

# Shortcut command — one of the high-level wrappers over common endpoints
xr me
xr post "hello"
xr search "from:jack"
xr like 1234567890
xr follow @jack

# JSON output for parsing
xr me --output json

# JSONL — one line per record, ideal for streaming + jaq pipelines
xr search "from:jack" --output jsonl | jaq '.id'

# Media upload — chunked INIT/APPEND/FINALIZE/STATUS state machine
xr media-upload ./image.png

# Schema introspection — emit the JSON shape of every typed response
xr schema
xr schema --command me

# Auth — see § Auth paths below
xr auth                          # interactive OAuth2-PKCE in a browser
xr auth --no-browser             # headless OAuth2 (copy-paste URL flow)
xr auth status                   # list configured apps and token freshness
```

Bare `xr` (no arguments) prints help and exits with the usage exit code.

## Auth paths

Four auth modes, selected by what's available in the token store and the environment.

| Path               | When used                                                                        | Surface                           |
| ------------------ | -------------------------------------------------------------------------------- | --------------------------------- |
| OAuth1 (HMAC-SHA1) | App tokens stored in `~/.xurl` with `consumer_key` + `consumer_secret`           | Legacy v1.1 + some v2 write paths |
| OAuth2 PKCE        | User-scoped flow via `xr auth` (browser-driven) or `xr auth --no-browser`        | All v2 user-scoped endpoints      |
| OAuth2 headless    | `xr auth --no-browser`: copy-paste the URL flow on a graphical-display-less host | Same as OAuth2 PKCE               |
| Bearer (app-only)  | `XURL_BEARER_TOKEN=…` in env                                                     | v2 read-only endpoints + search   |

The CLI picks per request: if a Bearer is set and the endpoint accepts app-auth, it's used; otherwise the stored
user-scoped tokens for the active app drive the call. Multi-app is supported in the token store; `xr auth status`
enumerates them.

`src/auth/` holds the four implementations. OAuth1 signing follows RFC 5849 (HMAC-SHA1, percent-encoded base string,
sorted parameter list). PKCE is the standard `code_verifier`/`code_challenge` flow with refresh-token rotation.

## Token store

YAML at `~/.xurl`. Schema is documented in `src/store/types.rs`. Migration logic lives in `src/store/migration.rs` and
runs on every load: older formats upgrade transparently and the upgraded file is written back. Multiple apps are stored
under the same file with a per-app block.

`xr auth status` is the operator-facing surface. Programmatic access uses `xurl::store::TokenStore`.

## Output formats

`OutputConfig` (`src/output.rs`) drives three formats:

- `--output text` (default): human-readable tables / formatted responses
- `--output json`: pretty-printed JSON envelope
- `--output jsonl`: one JSON record per line; ideal for streaming + pipeline composition with `jaq`

Streaming endpoints emit a continuous JSONL stream when `--output jsonl` is set; non-streaming endpoints emit one record
then close.

## Shortcut commands

`src/api/shortcuts.rs` ships `pub fn` wrappers over the X API endpoints documented in `vendor/x-api-openapi.json`
(`create_post`, `delete_post`, `like_post`, `repost`, `bookmark`, `follow_user`, `mute_user`, `send_dm`, `lookup_user`,
`get_timeline`, `get_mentions`, `search_posts`, `read_post`, `get_me`, `get_followers`, `get_following`,
`get_liked_posts`, `get_bookmarks`, `get_dm_events`, `get_usage`, `get_usage_credits`, and their `un*` inverses where the spec documents
them). Each maps to one CLI command via `src/cli/` and returns a typed response via `src/api/response/types.rs`. The
build-time auth matrix at `src/api/auth_matrix.rs` panics if a shortcut targets an endpoint absent from the vendored
spec — block/unblock are absent from the spec and have no shortcut surface.

Adding a shortcut means: implement the function in `shortcuts.rs`, add a typed response in `response/types.rs` (or
reuse), register in `src/cli/commands/mod.rs`, and update `xr schema` coverage by ensuring the response type derives
`schemars::JsonSchema`.

### Command grammar: flags vs subcommands

- Flags never select endpoints. A flag tunes one endpoint's request (auth method, output shape, pagination, fields);
  it never retargets the path.
- A standalone action or read in the core domain gets its own top-level command word: `dm`/`dms`,
  `bookmark`/`bookmarks`, and `timeline`/`mentions` are separate commands because they are separate endpoints.
- A subcommand family groups a noun that owns several operations: tooling nouns (`auth`, `skill`, `schema`,
  `completions`) and API namespaces with sibling endpoints (`media` over `/2/media/*`, `usage` over `/2/usage/*`).

Routing a new endpoint: when an existing family noun owns it (an API-namespace sibling), add a subcommand there; when
it stands alone in the core domain, add a top-level command; never add an endpoint-selecting flag to an existing
command.

## Architecture

- `src/api/`: HTTP client (`request.rs`), endpoints (`endpoints.rs`), shortcuts (`shortcuts.rs`), media upload
  (`media.rs`), and typed responses (`response/`).
- `src/auth/`: OAuth1 (HMAC-SHA1 per RFC 5849), OAuth2 PKCE (interactive + headless via callback handler), Bearer token.
  PKCE pending-state is in `pending.rs`; the callback HTTP server is `callback.rs`.
- `src/cli/`: clap-based CLI. `commands/mod.rs` is the handler layer; subdir files split media, schema, auth, streaming.
  `exit_codes.rs` encodes the CLI's exit-code contract.
- `src/config/`: env-var-based configuration.
- `src/store/`: YAML token store at `~/.xurl`; multi-app, with `migration.rs` for transparent upgrades.
- `src/output.rs`: `OutputConfig` for text/json/jsonl formatting.
- `src/error.rs`: `XurlError` via `thiserror`.
- `src/lib.rs`: public library surface. The `xurl` library is consumable from downstream Rust crates; the binary `xr` is
  one consumer among potentially several.

## Quality bar

- Clippy clean, edition 2024 (`cargo clippy -- -D warnings`)
- Formatted with rustfmt (`cargo fmt --check`); style edition pinned in `rustfmt.toml`
- No `unwrap()` in production code
- Comprehensive tests (`cargo test`): unit, integration, and differential conformance
- Zero broken tests policy: a failing test on `dev` blocks new work until it's fixed or reverted, not until "later"
- `cargo deny check` passes (advisories, licenses, bans, sources)
- Cross-platform: Linux, macOS, Windows (MSVC). Pre-push and CI both run a Windows compatibility check.

The pinned toolchain (`rust-toolchain.toml`) is the supply-chain anchor. Rustup verifies component SHA256s from the
distribution manifest; the pin is effectively a SHA pin. Toolchain bumps land via reviewed PR after ≥7-day quarantine.

## Testing

```bash
cargo test                    # unit + integration
cargo test -- --ignored       # slower / network-dependent tests
scripts/hooks/pre-push        # full local CI mirror (fmt, clippy, test, deny, shellcheck, Windows cross-clippy)
```

The pre-push hook mirrors CI 1:1. Run it before pushing if `core.hooksPath = scripts/hooks` is not set locally.

## Releasing

See [`RELEASES.md`](RELEASES.md) for the operational runbook, [`RELEASES-PREFLIGHT.md`](RELEASES-PREFLIGHT.md) for the
pre-cut go/no-go checklist, and [`RELEASES-RATIONALE.md`](RELEASES-RATIONALE.md) for the why behind every rule. The
short version: feature branch → PR to `dev` (squash) → cherry-pick to `release/v<version>` cut from `main` → PR to
`main` (squash) → annotated tag push triggers `release.yml`.

### Spec-refresh PRs

The spec-drift workflow opens draft PRs to `dev` (head `spec-refresh`) whose body is an agent runbook. Invoke the
reconciling agent as "Triage and fix PR #N".

## Known differences from the Go original

See [`KNOWN_DIFFERENCES.md`](KNOWN_DIFFERENCES.md) for intentional deviations from
[`xdevplatform/xurl`](https://github.com/xdevplatform/xurl).

## Documented solutions

`docs/solutions/` is a symlink to `~/dev/solutions-docs/`, a shared, searchable archive of past solutions and best
practices organized by category with YAML frontmatter (`module`, `tags`, `problem_type`). Search with `qmd query
"<topic>" --collection solutions` before implementing or debugging in a documented area; the corpus crosses repos and
already captures known pitfalls.
