# Releasing xurl-rs

## Automated (preferred)

Tag a version and push — CI handles everything:

```bash
# 1. Bump version in Cargo.toml
# 2. Commit and tag
git add Cargo.toml
git commit -m "chore: bump version to 1.0.4"
git tag v1.0.4
git push origin main --tags
```

This triggers `.github/workflows/release.yml` which:

- Builds binaries for 5 targets (linux x86_64/aarch64, macos x86_64/aarch64, windows x86_64)
- Creates a GitHub Release with all binaries attached
- Publishes to crates.io via Trusted Publishing (OIDC, no static token)
- Dispatches a `repository_dispatch` event to `brettdavies/homebrew-tap`, which automatically updates the formula's version and SHA256

Changelog is auto-generated on every push to main via git-cliff.

## Required GitHub Secrets

| Secret | Purpose |
|--------|---------|
| `HOMEBREW_TAP_TOKEN` | Fine-grained PAT with `contents:write` on `brettdavies/homebrew-tap` |

`GITHUB_TOKEN` is provided automatically by GitHub Actions.

crates.io publishing uses Trusted Publishing (OIDC) — no static token needed.

## Manual Steps (post-release)

### Regenerate Completions (if CLI flags changed)

```bash
cargo build --release
./target/release/xr --generate-completion bash > completions/xr.bash
./target/release/xr --generate-completion zsh > completions/_xr
./target/release/xr --generate-completion fish > completions/xr.fish
```

## Distribution Channels

| Channel | How |
|---------|-----|
| Homebrew | `brew tap brettdavies/tap && brew install xurl-rs` |
| Pre-built binary | Download from [GitHub Releases](https://github.com/brettdavies/xurl-rs/releases) |
| Rust crate | `cargo install xurl-rs` |
| From source | `git clone && cargo build --release` |
