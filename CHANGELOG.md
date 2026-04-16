# Changelog

All notable changes to this project will be documented in this file.

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
