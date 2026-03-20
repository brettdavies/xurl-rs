# Changelog

All notable changes to this project will be documented in this file.

## [1.0.5] - 2026-03-20

### Added

- `xr completions <shell>` subcommand replacing hidden `--generate-completion` flag by @brettdavies in
  [#6](https://github.com/brettdavies/xurl-rs/pull/6)
- PowerShell and Elvish shell completions
- cargo-deny license and advisory auditing via `deny.toml` by @brettdavies in
  [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- Draft-then-finalize release pattern via `finalize-release.yml`
- Commit provenance guard requiring PRs go through dev before main
- Changelog CI enforcement — PRs to main must include CHANGELOG.md updates
- `cargo binstall` support for pre-built binary installs

### Changed

- Version and completions commands now exit before config/auth initialization by @brettdavies in
  [#6](https://github.com/brettdavies/xurl-rs/pull/6)
- CI/CD migrated to centralized reusable workflows from `brettdavies/.github` by @brettdavies in
  [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- `ci.yml`, `release.yml`, `guard-main-docs.yml` replaced with thin callers
- Homebrew dispatch secret migrated from `HOMEBREW_TAP_TOKEN` to `CI_RELEASE_TOKEN`
- Release archives now include completions, licenses, README, and sha256sums

### Fixed

- `protect-dev.json` aligned with bird (add deletion, non_fast_forward rules, admin bypass) by @brettdavies in
  [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- Update `rustls-webpki` to 0.103.10 to fix [RUSTSEC-2026-0049](https://rustsec.org/advisories/RUSTSEC-2026-0049)

### Documentation

- RELEASING.md rewritten for reusable workflow pipeline and changelog-as-committed-artifact by @brettdavies in
  [#7](https://github.com/brettdavies/xurl-rs/pull/7)

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
