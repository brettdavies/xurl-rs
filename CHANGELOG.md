# Changelog

All notable changes to this project will be documented in this file.

## [1.3.0] - 2026-06-03

### Added

- New CLI surface: `auth apps add --redirect-uri`, `auth apps update --redirect-uri`, `auth apps redirect-uri get [NAME]`, `auth apps redirect-uri set NAME URI`. by @brettdavies in [#30](https://github.com/brettdavies/xurl-rs/pull/30)
- Structured JSON output for `auth status`, `auth apps list`, and `auth apps redirect-uri get` under `--output json`.
- New `--color <auto|always|never>` global flag (`XURL_COLOR`); `auto` honors stderr's TTY-ness, and `NO_COLOR` is absolute per https://no-color.org/. by @brettdavies in [#34](https://github.com/brettdavies/xurl-rs/pull/34)
- Environment-variable backing for every agentic flag: `XURL_VERBOSE`, `XURL_QUIET`, `XURL_NO_INTERACTIVE`, `XURL_TIMEOUT`, `XURL_APP` (in addition to the pre-existing `XURL_OUTPUT`). Boolean flags use `FalseyValueParser` so `XURL_QUIET=0` correctly disables quiet.
- `xr skill install [<host>] [--all] [--dry-run] [--output json]` to install the `AGENTS.md` bundle into per-host skills directories. Supports `claude_code`, `codex`, `cursor`, `factory`, `kiro`, and `opencode`. by @brettdavies in [#36](https://github.com/brettdavies/xurl-rs/pull/36)
- Wire `--timeout` / `XURL_TIMEOUT` through `ApiClient`, OAuth2 token exchange + refresh, and the `/2/users/me` lookup so the flag bounds every HTTP request. Default stays 30 seconds. by @brettdavies in [#37](https://github.com/brettdavies/xurl-rs/pull/37)
- SIGTERM and SIGINT now cancel the OAuth2 callback listener and any active streaming request. Streaming emits a `{"status":"cancelled","reason":"sigterm"}` envelope under `--output json` / `--jsonl` and returns exit code 0.
- Global `--force` flag (with confirmation gate) on `xr delete`, `xr auth clear`, and `xr auth apps remove`. Under `--no-interactive`, the flag is mandatory; calling the op without it returns `{"status":"error","reason":"confirmation-required","exit_code":1}` on stderr and never touches the API. Under a TTY without `--force`, dialoguer prompts the operator. by @brettdavies in [#39](https://github.com/brettdavies/xurl-rs/pull/39)
- Global `--dry-run` flag (env-backed via `XURL_DRY_RUN`) on every write op: post, reply, quote, delete, like, unlike, repost, unrepost, bookmark, unbookmark, follow, unfollow, block, unblock, mute, unmute, dm, media upload, app add/update/remove, redirect-uri set, and every `xr auth` subcommand. The envelope shape is `{"status":"dry_run","would_succeed":bool,"reason":"<kebab>","exit_code":int,"command":"<verb>",…}`. Pre-flight validators catch `empty-body`, `body-too-long`, `too-many-attachments`, `empty-username`, and `empty-post-id` so the dry-run path predicts failure before any HTTP call.
- Global `--limit <n>` flag (env-backed via `XURL_LIMIT`), clamped to `1..=100`. Applies to search, timeline, mentions, bookmarks, likes, following, followers, and dms. Per-command `-n/--max-results` keeps precedence when both are set.
- `XURL_NO_BROWSER` env var for `xr auth oauth2 --no-browser` so headless and CI runners can set the headless flow by default. by @brettdavies in [#43](https://github.com/brettdavies/xurl-rs/pull/43)
- `xr auth oauth2` auto-engages the headless (remote-step-1) flow when stdout is not a TTY and `--no-browser` / `XURL_NO_BROWSER` is unset, emitting `{"status":"awaiting_callback","url":"..."}` on stdout instead of attempting to open a browser.

### Changed

- Bumped declared `rust-version` from `1.85` to `1.94` to match the pinned toolchain, closing the MSRV gap between the promise to library consumers and the code that actually ships. by @brettdavies in [#28](https://github.com/brettdavies/xurl-rs/pull/28)
- `auth status` text output now includes a `redirect_uri:` line per app showing the effective URI and source label. When the env var overrides a stored value, a `stored_redirect_uri:` line surfaces the stored value. by @brettdavies in [#30](https://github.com/brettdavies/xurl-rs/pull/30)
- `auth apps list` text output includes the same inline redirect URI hint per app row.
- The OAuth2 callback listener now binds the host, port, and path from the resolved redirect URI rather than the hardcoded `127.0.0.1:8080/callback`.
- Path matching on the callback listener tightens from `starts_with` to exact-or-querystring, so a custom redirect URI like `/oauth/return` no longer matches unrelated prefixes.
- `-v/--verbose` is now a single root-level global flag and applies to every subcommand. The duplicate `-v` definitions on `CommonFlags`, `media upload`, and `media status` are removed. by @brettdavies in [#34](https://github.com/brettdavies/xurl-rs/pull/34)
- `OutputConfig` gains `print_dry_run` and `print_confirmation_required` helpers so any handler can emit the canonical envelope in one call. by @brettdavies in [#39](https://github.com/brettdavies/xurl-rs/pull/39)
- `XurlError::EnvelopeAlreadyEmitted { exit_code }` is the new sentinel that lets the runner suppress its trailing `print_error` when a typed envelope was already written; agents see exactly one JSON object on stderr.
- `src/api/shortcuts` is now `pub` so the new validators are importable from the command handlers without re-exports.
- `xr auth default` without an app argument now gates the dialoguer picker on `--no-interactive` AND stdin/stderr TTY-ness. Non-TTY sessions exit non-zero with a `{"status":"error","reason":"no-tty","exit_code":1,...}` envelope on stderr instead of stranding the dialoguer state machine on `/dev/null`. by @brettdavies in [#43](https://github.com/brettdavies/xurl-rs/pull/43)
- `xr auth oauth2 --no-browser` (no `--step`) now auto-promotes to step 1 and emits the canonical `{"status":"awaiting_callback","url":"..."}` envelope, instead of rejecting the invocation as "requires --step 1 or --step 2". The explicit `--no-browser --step 1` path keeps its prior envelope shape.

### Fixed

- Bump `rand` to 0.9.4 to clear RUSTSEC-2026-0097 (`ThreadRng` unsoundness when accessed from a custom `log::Logger`). by @brettdavies in [#25](https://github.com/brettdavies/xurl-rs/pull/25)
- Bump `rustls-webpki` to 0.103.13 to clear RUSTSEC-2026-0104 (reachable panic in CRL `IssuingDistributionPoint` parsing, triggerable before signature verification).
- Resolved a private-intra-doc-link in `src/auth/pending.rs` that broke `cargo doc -D warnings` and would render badly on docs.rs. by @brettdavies in [#28](https://github.com/brettdavies/xurl-rs/pull/28)
- Browsers that resolve `localhost` to `[::1]` no longer time out during the OAuth2 flow. The listener dual-binds both loopback addresses when the URI host is `localhost`. by @brettdavies in [#30](https://github.com/brettdavies/xurl-rs/pull/30)
- The browser cannot be opened before the callback listener is actively draining the accept queue, eliminating a race on fast machines.
- Invalid redirect URIs are rejected at write time. `http` is allowed only on loopback hosts; non-loopback `http` and other schemes return a validation error.
- `auth status` and `auth apps list` previously read `~/.xurl` directly through a fresh `TokenStore::new()`, bypassing the runner's configured store path. Both now use `&auth.token_store`, which makes tempdir isolation reliable for library-level tests.

### Documentation

- Document the `docs/solutions` archive and `qmd query` retrieval in `AGENTS.md` so agent consumers can find it from the onboarding doc. by @brettdavies in [#26](https://github.com/brettdavies/xurl-rs/pull/26)
- Harden `RELEASES.md` with triple-diff verification, prose scrubbing, the CHANGELOG generation contract and `cliff.toml` chore-skip footgun, non-draft release behavior, and branch-protection rulesets.
- Document new CLI surface (per-app redirect URI, `-u USERNAME` fallback, structured output) in README by @brettdavies in [#32](https://github.com/brettdavies/xurl-rs/pull/32)
- Document X platform "Pay-per-use Production" enrollment workaround in README

**Full Changelog**: [v1.2.0...v1.3.0](https://github.com/brettdavies/xurl-rs/compare/v1.2.0...v1.3.0)

## [1.2.0] - 2026-04-16

### Added

- Add `ApiClient::from_env()` one-liner constructor that reads `CLIENT_ID`/`CLIENT_SECRET` from environment variables by @brettdavies in [#21](https://github.com/brettdavies/xurl-rs/pull/21)
- Add `CallOptions` consumer-facing struct for shortcut methods, exposing only `auth_type`, `username`, `no_auth`, `verbose`, `trace`
- Add `no_auth` field to skip authentication entirely on a per-request basis
- Add `XurlError::Validation(String)` variant for non-HTTP validation errors (e.g., errors-only 200 responses)

### Changed

- Change `ApiClient` from borrowed `&mut Auth` to owned `Auth` — no lifetime parameter, storable in structs by @brettdavies in [#21](https://github.com/brettdavies/xurl-rs/pull/21)
- Change 29 shortcut functions from free functions (`api::create_post(&mut client, ...)`) to methods (`client.create_post(...)`)
- Change `XurlError::Api(String)` to `Api { status: u16, body: String }` for structured HTTP error matching
- Change `exit_code_for_error()` to pattern-match on `Api { status, .. }` directly instead of string matching

### Fixed

- Bump `rustls-webpki` to 0.103.12 to clear [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098) and [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099) (name constraint validation). by @brettdavies in [#23](https://github.com/brettdavies/xurl-rs/pull/23)

### Documentation

- Document exit code mapping improvement as known difference from Go version in KNOWN_DIFFERENCES.md by @brettdavies in [#21](https://github.com/brettdavies/xurl-rs/pull/21)

**Full Changelog**: [v1.1.0...v1.2.0](https://github.com/brettdavies/xurl-rs/compare/v1.1.0...v1.2.0)

## [1.1.0] - 2026-04-02

### Added

- Add `xr usage` shortcut command that returns full API usage data (tweet caps, daily project breakdown, per-app breakdown) by @brettdavies in [#13](https://github.com/brettdavies/xurl-rs/pull/13)
- Add `--remote` flag for headless OAuth2 authentication on machines without a browser by @brettdavies in [#14](https://github.com/brettdavies/xurl-rs/pull/14)
- Add `--step` (1 or 2) and `--auth-url` (with `-` for stdin) companion flags
- Add JSON output support for step 1 (`--output json` emits `{"auth_url": "...", "instructions": "..."}`)
- Add typed response structs: `Tweet`, `User`, `DmEvent`, `UsageData`, 7 action confirmations, 3 wrapper/meta types, 3 nested types by @brettdavies in [#17](https://github.com/brettdavies/xurl-rs/pull/17)
- Add `deserialize_response<T>()` helper with guards for empty and errors-only 200 responses
- Add `ApiResponse<T>` generic wrapper with `data`, `includes`, `meta`, `errors`, and forward-compatible `extra` fields
- Add `xr schema <command>` to output JSON Schema for any command's response type by @brettdavies in [#18](https://github.com/brettdavies/xurl-rs/pull/18)
- Add `xr schema --list` to show all 29 commands with their response types
- Add `xr schema --all` to output all schemas as a single JSON document
- Add `schemars` dependency for compile-time JSON Schema generation via `#[derive(JsonSchema)]`

### Changed

- Rename `--remote` to `--no-browser` for the headless OAuth2 authentication flow by @brettdavies in [#15](https://github.com/brettdavies/xurl-rs/pull/15)
- Change all 29 shortcut functions from `Value` returns to typed `ApiResponse<T>` returns (**breaking** for library consumers) by @brettdavies in [#17](https://github.com/brettdavies/xurl-rs/pull/17)

### Fixed

- Rename completion files to standard convention (`xr.zsh`, `xr.elvish`, `xr.powershell`) by @brettdavies in [#11](https://github.com/brettdavies/xurl-rs/pull/11)
- Regenerate bash and fish completions for completions subcommand
- Fix test isolation in `Auth::with_token_store()` where real `~/.xurl` credentials leaked into test assertions by @brettdavies in [#12](https://github.com/brettdavies/xurl-rs/pull/12)

### Documentation

- Add shell completions regeneration step to the release process as a safety net for missed completions during development by @brettdavies in [#13](https://github.com/brettdavies/xurl-rs/pull/13)

**Full Changelog**: [v1.0.5...v1.1.0](https://github.com/brettdavies/xurl-rs/compare/v1.0.5...v1.1.0)

## [1.0.5] - 2026-03-21

### Added

- `xr completions <shell>` subcommand replacing hidden `--generate-completion` flag by @brettdavies in [#6](https://github.com/brettdavies/xurl-rs/pull/6)
- PowerShell and Elvish shell completions by @brettdavies in [#6](https://github.com/brettdavies/xurl-rs/pull/6)
- cargo-deny license and advisory auditing via `deny.toml` by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- Draft-then-finalize release pattern via `finalize-release.yml`
- Commit provenance guard requiring PRs go through dev before main
- Changelog CI enforcement — PRs to main must include CHANGELOG.md updates
- `cargo binstall` support for pre-built binary installs

### Changed

- Version and completions commands now exit before config/auth initialization by @brettdavies in [#6](https://github.com/brettdavies/xurl-rs/pull/6)
- CI/CD migrated to centralized reusable workflows from `brettdavies/.github` by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- `ci.yml`, `release.yml`, `guard-main-docs.yml` replaced with thin callers
- Homebrew dispatch secret migrated from `HOMEBREW_TAP_TOKEN` to `CI_RELEASE_TOKEN`
- Release archives now include completions, licenses, README, and sha256sums

### Fixed

- `protect-dev.json` aligned with bird (add deletion, non_fast_forward rules, admin bypass) by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- Update `rustls-webpki` to 0.103.10 to fix [RUSTSEC-2026-0049](https://rustsec.org/advisories/RUSTSEC-2026-0049)
- Move `thiserror`, `anyhow`, `dirs`, `percent-encoding` to platform-independent `[dependencies]` — fixes Windows build by @brettdavies in [#9](https://github.com/brettdavies/xurl-rs/pull/9)

### Documentation

- RELEASING.md rewritten for reusable workflow pipeline and changelog-as-committed-artifact by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)

**Full Changelog**: [v1.0.4...v1.0.5](https://github.com/brettdavies/xurl-rs/compare/v1.0.4...v1.0.5)

## [1.0.4] - 2026-03-16

### Changed

- Switch to Trusted Publishing (OIDC) for crates.io authentication — no static secrets by @brettdavies in
  [#4](https://github.com/brettdavies/xurl-rs/pull/4)
- Pin all GitHub Actions by commit SHA for supply-chain security by @brettdavies in
  [#4](https://github.com/brettdavies/xurl-rs/pull/4)
- Switch reqwest from native-tls to rustls-tls for cross-compilation compatibility by @brettdavies in
  [#3](https://github.com/brettdavies/xurl-rs/pull/3)
- Update macOS CI runner from deprecated `macos-13` to `macos-latest` by @brettdavies in
  [#3](https://github.com/brettdavies/xurl-rs/pull/3)
- Opt into Node.js 24 for GitHub Actions (`FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`) by @brettdavies in
  [#4](https://github.com/brettdavies/xurl-rs/pull/4)

**Full Changelog**: [v1.0.3...v1.0.4](https://github.com/brettdavies/xurl-rs/compare/v1.0.3...v1.0.4)

## [1.0.3] - 2026-03-16

### Added

- Full xurl-rs implementation — Rust port of Go [xurl](https://github.com/xdevplatform/xurl) CLI by @brettdavies in
  [#2](https://github.com/brettdavies/xurl-rs/pull/2)
- 28 shortcut commands: post, reply, quote, delete, read, search, like, repost, bookmark, follow, block, mute, dm,
  timeline, mentions, whoami, and more
- Raw API mode: `xr /2/users/me`, `xr -X POST /2/tweets -d '{...}'`
- OAuth2 PKCE, OAuth1 HMAC-SHA1, and Bearer token authentication
- YAML token store with multi-app management at `~/.xurl`
- Media upload (chunked) with status polling
- Shell completions for bash, zsh, fish, powershell, elvish
- Agent-native features: `--output json/jsonl`, `--quiet`, `--no-interactive`, structured exit codes (0-5)
- `NO_COLOR` and `XURL_OUTPUT` environment variable support
- Release infrastructure: CI, cross-platform builds (5 targets), crates.io Trusted Publishing, Homebrew tap dispatch

### Fixed

- Switch reqwest from native-tls to rustls-tls for cross-compilation by @brettdavies in
  [#3](https://github.com/brettdavies/xurl-rs/pull/3)

### New Contributors

- @brettdavies made their first contribution in [#2](https://github.com/brettdavies/xurl-rs/pull/2)
