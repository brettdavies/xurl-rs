# Releasing xurl-rs

## Development workflow

All changes MUST go through feature PRs to `dev`. Never commit directly to `dev` or `main`. This ensures every change
has a PR number in its squash commit message, which git-cliff uses to generate changelog entries with PR links and
author attribution.

```text
feature branch → PR to dev (squash merge) → cherry-pick to release branch → PR to main (squash merge)
```

## Merging dev to main

Engineering docs (`docs/plans/`, `docs/solutions/`, `docs/brainstorms/`) live on `dev` only. `guard-main-docs.yml`
blocks them from `main`. You MUST use the release branch pattern:

```bash
# 1. Branch from main, NOT dev
git checkout -b release/v1.1.0 origin/main

# 2. Cherry-pick only non-docs commits from dev
git cherry-pick <commit1> <commit2> ...

# 3. Verify no docs paths leaked through
git diff origin/main --stat

# 4. Generate changelog (REQUIRED — CI enforces this)
~/.claude/skills/rust-tool-release/scripts/generate-changelog.sh
git add CHANGELOG.md
git commit -m "docs: update CHANGELOG.md"

# 5. Push and open a PR to main
git push -u origin release/v1.1.0
gh pr create --base main
```

**CRITICAL:** Always branch from `origin/main`. Branching from `dev` causes `add/add` merge conflicts when dev and main
have divergent histories (e.g., after squash merges).

## Changelog

CHANGELOG.md is a committed artifact managed during release prep — not auto-generated in CI. The `generate-changelog.sh`
script prepends new entries from unreleased commits while preserving existing content. It automatically fetches PR
metadata from GitHub for author attribution and PR links.

CI enforces that CHANGELOG.md is modified in every PR to main (`ci / Changelog` required status check). The release
workflow extracts the latest section from CHANGELOG.md for the GitHub Release body.

## Tagging and releasing

After the PR merges to main, tag and push:

```bash
git checkout main && git pull
git tag v1.1.0
git push origin main --tags
```

This triggers `.github/workflows/release.yml` which:

- Verifies the tag matches `Cargo.toml` version
- Runs `cargo deny` (license + advisory + ban checking)
- Builds binaries for 5 targets (linux x86_64/aarch64, macos x86_64/aarch64, windows x86_64)
- Ad-hoc codesigns macOS binaries
- Creates `.tar.gz` archives with binary + LICENSE + README + shell completions
- Publishes to crates.io via Trusted Publishing (OIDC, no static token)
- Creates a **draft** GitHub Release with archives and sha256sums
- Dispatches `repository_dispatch` to `brettdavies/homebrew-tap`, which auto-updates the formula version and SHA256
- After Homebrew bottles are built, `finalize-release.yml` publishes the draft

### Pipeline order

```text
check-version + audit -> build (5 targets) -> publish-crate -> release (draft) -> homebrew -> finalize
```

`cargo publish` runs BEFORE GitHub Release creation. If publish fails, no release is advertised and no Homebrew update
is triggered.

## Required GitHub Secrets

| Secret | Purpose | Rotation |
|--------|---------|----------|
| `CI_RELEASE_TOKEN` | Fine-grained PAT with `contents:write` for CI release automation (Homebrew dispatch, changelog, rulesets) | Max 1 year; renew before expiry |

`GITHUB_TOKEN` is provided automatically by GitHub Actions.

Secrets are stored in 1Password (`secrets-dev` vault).

## crates.io Publishing

Publishing uses
[Trusted Publishing](https://doc.rust-lang.org/cargo/reference/registry-authentication.html#trusted-publishing) via
`rust-lang/crates-io-auth-action`. No static API token is needed — OIDC exchanges a short-lived GitHub Actions token for
a ~30-minute crates.io token.

Trusted Publishing was configured after the v1.0.3 manual publish. If it ever needs reconfiguration:

1. Go to `https://crates.io/settings/tokens/trusted-publishing`
2. Add trusted publisher: owner=`brettdavies`, repo=`xurl-rs`, workflow=`release.yml`
3. Enable "Enforce Trusted Publishing" to disable token-based publishing

## Shell Completions

Pre-build completions locally and commit to `completions/`. Regenerate whenever subcommands or flags change:

Use the skill script: `~/.claude/skills/rust-tool-release/scripts/generate-completions.sh`

## Distribution Channels

| Channel | How |
|---------|-----|
| Homebrew | `brew install brettdavies/tap/xurl-rs` |
| Pre-built binary | Download from [GitHub Releases](https://github.com/brettdavies/xurl-rs/releases) |
| Rust crate | `cargo install xurl-rs` |
| Fast binary | `cargo binstall xurl-rs` |
| From source | `git clone && cargo build --release` |
