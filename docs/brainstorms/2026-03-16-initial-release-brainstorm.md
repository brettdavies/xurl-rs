---
title: "xurl-rs v1.0.3 Initial Release"
status: completed
date: 2026-03-16
---

# Brainstorm: xurl-rs v1.0.3 Initial Release

## What We're Building

A professional initial release of xurl-rs v1.0.3 — the Rust port of Go xurl. This covers code fixes, repo metadata, and
a release checklist to ensure nothing amateur ships publicly.

## Why This Approach

The codebase is fundamentally ready (137 tests passing, clean formatting, crate packages successfully). The remaining
work is polish: fix 3 code issues found in review, configure GitHub repo metadata, and establish the release sequence.

## Key Decisions

### 1. Repo Visibility: Keep Private For Now

- **Decision:** Dry-run the full release process with the repo private.
- **Rationale:** Ensures everything is perfect before public exposure. crates.io publish and homebrew install will be
  tested after going public.
- **Sequence:** Fix code > set metadata > merge dev to main > go public > tag > release.

### 2. Code Fixes: Fix All 3 Issues Before Merge

- **Decision:** Ship pristine — fix all issues found in the review.
- **Items:**
- **Clippy warnings (2):** `conformance_runner` test code — redundant field name, unused `tags` field. CI runs
    `-Dwarnings` on main, so these would fail.
- **println/exit bypass:** `src/cli/commands/mod.rs:36-38` uses `println!()` + `process::exit(1)` instead of returning
    an error through OutputConfig. Breaks `--output json` and `--quiet` for this one error path.
- **British spelling:** `src/api/shortcuts.rs:54` says "Normalises" — should be "Normalizes" for consistency with
    American English used throughout the codebase.

### 3. GitHub Repo Metadata: Set Up Programmatically

- **Decision:** Use `gh repo edit` to set description, homepage, topics.
- **Fields:**
- Description: "Fast, ergonomic CLI for the X (Twitter) API. Rust port of xurl."
- Homepage: `https://github.com/brettdavies/xurl-rs`
- Topics: rust, twitter, x-api, cli, oauth, api-client

### 4. Homebrew SHA256: Automated via Repository Dispatch

- **Decision:** Fully automate the Homebrew formula update using a cross-repo `repository_dispatch` pattern.
- **How it works:**
- xurl-rs `release.yml` fires a `repository_dispatch` event to `brettdavies/homebrew-tap` after the GitHub Release is
    created.
- homebrew-tap has its own workflow that receives the event, fetches the release tarball, computes sha256, updates the
    formula, commits, and pushes.
- **Why dispatch over inline job:** Clean separation of concerns. Each repo owns its own automation. Scales if more
  formulas are added later.
- **Requires:** Fine-grained PAT with `contents: write` on `brettdavies/homebrew-tap`, stored as `HOMEBREW_TAP_TOKEN`
  secret in xurl-rs repo.

## Release Sequence (Ordered)

1. Fix 3 code issues on `dev` branch
2. Add `repository_dispatch` step to xurl-rs `release.yml`
3. Create update workflow in `homebrew-tap` repo
4. Create fine-grained PAT, add as `HOMEBREW_TAP_TOKEN` secret
5. Set GitHub repo metadata
6. Merge `dev` to `main` via PR
7. Make repo public
8. Tag `v1.0.3` on main
9. Push tag — CI builds 5 binaries, publishes to crates.io, creates GitHub Release, dispatches Homebrew update
10. Verify all 4 distribution channels work

## Open Questions

None — all decisions resolved.
