# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Changed

- Migrate CI/CD to centralized reusable workflows from `brettdavies/.github`
- Replace inline `ci.yml`, `release.yml` with thin callers
- Convert `guard-main-docs.yml` to reusable workflow caller
- Add draft-then-finalize release pattern via `finalize-release.yml`

### Added

- `deny.toml` for cargo-deny license and advisory auditing
- `cargo binstall` metadata for pre-built binary installs
- Changelog CI enforcement — PRs to main must include CHANGELOG.md updates
- Release profile optimizations (`codegen-units = 1`, `panic = "abort"`)

### Fixed

- Align `protect-dev.json` with bird (add deletion, non_fast_forward rules, admin bypass)
- Add `docs/reviews/` to guard-main-docs forbidden paths

### Documentation

- Rewrite RELEASING.md for reusable workflow pipeline and changelog-as-committed-artifact

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
