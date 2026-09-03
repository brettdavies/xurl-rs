---
name: xurl-rs
binary: xr
description: Fast, ergonomic CLI for the X (Twitter) API. Rust port of the Go xurl, with OAuth1 / OAuth2-PKCE / Bearer auth, 27 high-level shortcut commands, chunked media upload, and streaming.
homepage: https://github.com/brettdavies/xurl-rs
repository: https://github.com/brettdavies/xurl-rs
---

# AGENTS.md

## Running xr

The crate is `xurl-rs`. The installed binary is `xr`. The library re-exports under `xurl`.

```bash
# Raw request — full control over verb, path, body, headers
xr /2/users/me
xr -X POST /2/tweets -d '{"text":"hello"}'
xr '/2/tweets/search/recent?query=from:jack'

# Shortcut command — one of the high-level wrappers over common endpoints
xr whoami
xr post "hello"
xr search "from:jack"
xr like 1234567890
xr follow @jack

# JSON output for parsing
xr whoami --output json
xr search "from:jack" --output json | jaq -r '.data[].id'

# JSONL — one record per line on streaming endpoints
xr -s /2/tweets/search/stream --output jsonl | jaq -c '.data.id'

# Media upload — chunked INIT/APPEND/FINALIZE/STATUS state machine
xr media upload ./image.png

# Schema introspection — emit the JSON shape of a typed response
xr schema --list
xr schema whoami

# Auth — see § Auth paths below
xr auth oauth2                   # interactive OAuth2-PKCE in a browser
xr auth oauth2 --no-browser      # headless OAuth2 (copy-paste URL flow)
xr auth status                   # list configured apps and token freshness
```

Bare `xr` (no arguments) fails with `No URL provided`, exit code 1, and points at `--help`.

## Auth paths

Four auth modes, selected by what's available in the token store and the environment.

| Path               | When used                                                                                           | Surface                                                 |
| ------------------ | --------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| OAuth1 (HMAC-SHA1) | User tokens stored in `~/.xurl` via `xr auth oauth1` (consumer key + secret, access token + secret) | v2 user-context endpoints whose spec entry lists OAuth1 |
| OAuth2 PKCE        | User-scoped flow via `xr auth oauth2` (browser-driven) or `xr auth oauth2 --no-browser`             | All v2 user-scoped endpoints                            |
| OAuth2 headless    | `xr auth oauth2 --no-browser`: copy-paste the URL flow on a graphical-display-less host             | Same as OAuth2 PKCE                                     |
| Bearer (app-only)  | `XURL_BEARER_TOKEN=…` in env, or stored via `xr auth app --bearer-token`                            | v2 read-only endpoints + search                         |

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

`OutputConfig` (`src/output.rs`) drives seven formats, selected with `--output`:

- `text` (default): human-readable tables / formatted responses
- `json`: pretty-printed JSON envelope
- `jsonl` (alias `ndjson`): one JSON record per line; ideal for streaming + pipeline composition with `jaq`
- `yaml` (alias `yml`), `csv`, `tsv`: tabular and config-friendly renderings of the same envelope

Streaming endpoints emit a continuous JSONL stream when `--output jsonl` is set; non-streaming endpoints emit one record
then close.

## Shortcut commands

`src/api/shortcuts.rs` ships `pub fn` wrappers over the X API endpoints documented in `vendor/x-api-openapi.json`
(`create_post`, `delete_post`, `like_post`, `repost`, `bookmark`, `follow_user`, `mute_user`, `send_dm`, `lookup_user`,
`get_timeline`, `get_mentions`, `search_posts`, `read_post`, `get_me`, `get_followers`, `get_following`,
`get_liked_posts`, `get_bookmarks`, `get_dm_events`, `get_usage`, `get_usage_credits`, and their `un*` inverses where
the spec documents them). Each maps to one CLI command via `src/cli/` and returns a typed response via
`src/api/response/types.rs`. `build.rs` generates the auth matrix from the vendored spec and fails the build if a
shortcut targets an endpoint absent from it; `src/api/auth_matrix.rs` wraps the generated table for runtime lookup.
Block/unblock are absent from the spec and have no shortcut surface.

Adding a shortcut means: implement the function in `shortcuts.rs`, add a typed response in `response/types.rs` (or
reuse), register in `src/cli/commands/mod.rs`, and update `xr schema` coverage by ensuring the response type derives
`schemars::JsonSchema`.

### Command grammar: flags vs subcommands

- Flags never select endpoints. A flag tunes one endpoint's request (auth method, output shape, pagination, fields); it
  never retargets the path.
- A standalone action or read in the core domain gets its own top-level command word: `dm`/`dms`,
  `bookmark`/`bookmarks`, and `timeline`/`mentions` are separate commands because they are separate endpoints.
- A subcommand family groups a noun that owns several operations: tooling nouns (`auth`, `skill`, `schema`,
  `completions`) and API namespaces with sibling endpoints (`media` over `/2/media/*`, `usage` over `/2/usage/*`).

Routing a new endpoint: when an existing family noun owns it (an API-namespace sibling), add a subcommand there; when it
stands alone in the core domain, add a top-level command; never add an endpoint-selecting flag to an existing command.

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

Tests never resolve the real home directory. Build stores and auth on an explicit path under a `tempfile::TempDir`
(`TokenStore::new_with_path`, `Auth::new_with_store_path`, `run_with_store_path`). `tests/store_isolation_guard.rs`
fails the suite when a test file names `Auth::new(`, `TokenStore::new()`, `TokenStore::with_credentials(`,
`default_store_path()`, `default_pending_path()`, `dirs::home_dir()`, sets `HOME` on a child process, or spawns the `xr`
binary outside `common::xr()` and `common::xr_with_store` (which point `XURL_TOKEN_STORE` at an unwritable scratch path
or the test's own temp store); a test that must touch the real path goes on its allowlist with the reason.

The pre-push hook mirrors CI 1:1. Run it before pushing if `core.hooksPath = scripts/hooks` is not set locally.

## Releasing

See [`RELEASES.md`](RELEASES.md) for the operational runbook, [`RELEASES-PREFLIGHT.md`](RELEASES-PREFLIGHT.md) for the
pre-cut go/no-go checklist, and [`RELEASES-RATIONALE.md`](RELEASES-RATIONALE.md) for the why behind every rule. The
short version: feature branch → PR to `dev` (squash) → cherry-pick to `release/v<version>` cut from `main` → PR to
`main` (squash) → annotated tag push triggers `release.yml`.

### Spec-refresh PRs

The spec-drift workflow opens PRs to `dev` (head `spec-refresh`) whose body is an agent runbook. Invoke the reconciling
agent as "Triage and fix PR #N".

## Known differences from the Go original

See [`KNOWN_DIFFERENCES.md`](KNOWN_DIFFERENCES.md) for intentional deviations from
[`xdevplatform/xurl`](https://github.com/xdevplatform/xurl).

## Documented solutions

On `dev`, `docs/solutions/` is a symlink to `~/dev/solutions-docs/`, a shared, searchable archive of past solutions and
best practices organized by category with YAML frontmatter (`module`, `tags`, `problem_type`). Search with `qmd query
"<topic>" --collection solutions` before implementing or debugging in a documented area; the corpus crosses repos and
already captures known pitfalls.

`CONCEPTS.md`, on `dev` alongside the engineering docs, holds the shared domain vocabulary (entities, named processes,
and status concepts with project-specific meaning), relevant when orienting to the codebase or discussing domain
concepts. Neither the symlink nor `CONCEPTS.md` ships to `main`; `guard-main-docs` blocks both.
