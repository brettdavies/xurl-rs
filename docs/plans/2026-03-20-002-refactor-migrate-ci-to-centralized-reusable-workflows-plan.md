---
title: "refactor: migrate CI to centralized reusable workflows"
type: refactor
status: active
date: 2026-03-20
---

# refactor: migrate CI to centralized reusable workflows

## Overview

Replace xurl-rs's inline GitHub Actions workflows with thin callers that invoke centralized reusable workflows from
`brettdavies/.github`. This aligns xurl-rs with the standard established by the `rust-tool-release` and
`github-repo-setup` skills, adds cargo-deny auditing, package-check validation, and the draft-then-finalize release
pattern.

## Problem Statement / Motivation

The `repo-settings.sh report` audit flags two compliance failures:

- `ci.yml` is inline (should call `rust-ci.yml`)
- `release.yml` is inline (should call `rust-release.yml`)

Additionally:

- `finalize-release.yml` is missing entirely (draft-then-finalize pattern not implemented)
- `deny.toml` is missing (required for cargo-deny audit job in reusable CI)
- `protect-main.json` has stale status check names that won't match the reusable workflow output
- Release workflow uses `HOMEBREW_TAP_TOKEN` (should be `CI_RELEASE_TOKEN`)
- Release creates non-draft releases (should be draft, finalized after Homebrew bottles land)
- Release uploads raw binaries (should be tar.gz/zip archives with completions, licenses, README, sha256sums)

Bird's `main` branch (PR #15, commit `e1925d0`) completed this migration. The reusable workflows are deployed to
`brettdavies/.github` on `main`. xurl-rs is the second consumer.

## Proposed Solution

Replace all three workflow files with thin callers from the skill templates. Add supporting config (`deny.toml`,
Cargo.toml changes). Update the branch protection ruleset after the new CI passes.

## Technical Considerations

### Trusted Publishing with reusable workflows

The reusable `rust-release.yml` runs `cargo publish` inside a `workflow_call` job. The OIDC token's `job_workflow_ref`
claim will reference the callee (`brettdavies/.github/.github/workflows/rust-release.yml@main`), not the caller
(`brettdavies/xurl-rs/.github/workflows/release.yml`).

Bird's latest tag (v0.1.2) predates its CI migration, so **no repo has tested Trusted Publishing through the reusable
workflow yet**. The crates.io Trusted Publishing configuration for xurl-rs currently specifies `workflow=release.yml`
for repo `brettdavies/xurl-rs`.

**Action required:** Before tagging the first release, verify or update the Trusted Publishing configuration on
crates.io. If crates.io validates against `job_workflow_ref`, update the publisher config to reference the callee
workflow. If it validates against the caller repo + workflow, no change needed.

**Fallback:** If Trusted Publishing fails, the release pipeline will halt at `publish-crate`. Fix the config and re-run
manually, or use a one-time `CARGO_REGISTRY_TOKEN` for the release.

### deny.toml license differences from bird

Bird's `deny.toml` allow-list does **not** include `CDLA-Permissive-2.0`. Confirmed locally:

```text
cargo deny check licenses → FAILED
  rejected: CDLA-Permissive-2.0 (webpki-roots v1.0.6, via reqwest[rustls-tls])
```

xurl-rs's `deny.toml` must add `"CDLA-Permissive-2.0"` to the allow-list.

### Homebrew formula uses removed --generate-completion flag

The current formula uses `generate_completions_from_executable(bin/"xr", "--generate-completion")`. Commit `a6a74e0` on
`dev` refactored completions from a `--generate-completion` flag to a `completions` subcommand. If this change ships in
the same release as the CI migration, `brew install` will fail during the `install` phase because `xr
--generate-completion bash` no longer works. The bottle build will fail, bottles never upload, and the
`finalize-release` dispatch never fires — leaving the release stuck as a draft.

**Decision needed:** Either:

- **(A) Update the formula before tagging** — manually update the formula to
  `generate_completions_from_executable(bin/"xr", "completions")`. This temporarily breaks `brew install` for v1.0.4
  until the new release ships (minutes to hours).
- **(B) Split the migration** — ship CI infrastructure changes without the completions refactor first, then ship the
  completions refactor with a formula update in a subsequent release. Safest but requires two releases.
- **(C) Coordinate same-release** — update the formula in the same Homebrew dispatch by extending the dispatch payload
  or manually pushing the formula change right before tagging.

Recommendation: **(A)** — update the formula immediately before tagging. The window of breakage is minimal (between
formula push and release tag), and v1.0.4 bottles already exist so existing users won't re-install from source.

### Chicken-and-egg: ruleset vs CI check names

The thin caller with job key `ci:` produces status checks prefixed `ci /` (e.g., `ci / Fmt, clippy, test`). The current
ruleset requires `Fmt, clippy, test` (no prefix). During the migration PR:

1. The old check name will **not** appear (the inline workflow is gone)
2. The new check names **will** appear but aren't yet required
3. The ruleset will show the PR as failing the `Fmt, clippy, test` requirement

**Solution:** Merge the migration PR using admin bypass. Immediately update the live ruleset via API. The vulnerability
window is minutes.

### Release archive format change

Current releases upload raw binaries (`xr-x86_64-unknown-linux-gnu`). The reusable workflow creates archives
(`xurl-rs-x86_64-unknown-linux-gnu.tar.gz`) containing the binary, completions, licenses, and README. This is a one-time
format change. Existing v1.0.4 assets are unaffected. Document the change in the next CHANGELOG entry.

## System-Wide Impact

- **Interaction graph:** Tag push triggers `release.yml` caller -> reusable `rust-release.yml` -> build + publish +
  draft release -> Homebrew dispatch -> `homebrew-tap` builds bottles -> `finalize-release` dispatch -> publishes draft.
  Chain is longer than current (adds finalize step).
- **Error propagation:** If `cargo publish` fails, no GitHub Release is created and no Homebrew dispatch fires. If
  bottle build fails, the release stays as a draft (visible only to repo admins). Both are safe failure modes.
- **State lifecycle risks:** A stuck draft release is the main risk. Admin can manually publish via `gh api
  repos/.../releases/<id> --method PATCH -f draft=false`.
- **API surface parity:** No user-facing API changes. Binary name (`xr`) unchanged. Only CI/CD infrastructure changes.

## Acceptance Criteria

### Workflow files

- [ ] `ci.yml` is a thin caller to `rust-ci.yml@main` with job key `ci:` and `permissions: contents: read`
- [ ] `release.yml` is a thin caller to `rust-release.yml@main` with `crate: xurl-rs`, `bin: xr`, `CI_RELEASE_TOKEN`
  secret, and `permissions: contents: write, id-token: write`
- [ ] `finalize-release.yml` exists as a thin caller to `rust-finalize-release.yml@main` with `permissions: contents:
  write`
- [ ] `guard-main-docs.yml` unchanged (already correct with status filter)
- [ ] No inline CI logic remains — all jobs delegated to reusable workflows

### Configuration files

- [ ] `deny.toml` exists with license allow-list including `CDLA-Permissive-2.0`
- [ ] `deny.toml` passes `cargo deny check` locally (all four categories)
- [ ] `Cargo.toml` has `codegen-units = 1` and `panic = "abort"` in `[profile.release]`
- [ ] `Cargo.toml` has `[package.metadata.binstall]` section with correct pkg-url and pkg-fmt
- [ ] `Cargo.toml` exclude list includes `deny.toml` and `rustfmt.toml`, does NOT include `completions/`

### Branch protection

- [ ] `protect-main.json` requires: `ci / Fmt, clippy, test`, `ci / Package check`, `ci / Security audit (bans licenses
  sources)`, `check-forbidden-docs`
- [ ] Live ruleset on GitHub matches `protect-main.json`

### Secrets

- [ ] `CI_RELEASE_TOKEN` secret is configured in repo settings
- [ ] `release.yml` references `CI_RELEASE_TOKEN` (not `HOMEBREW_TAP_TOKEN`)
- [ ] `HOMEBREW_TAP_TOKEN` removed after first successful release with new workflows

### Validation

- [ ] `repo-settings.sh report brettdavies/xurl-rs` shows CI workflow compliance passing
- [ ] `pin-actions.sh` shows no unpinned actions (already passing)
- [ ] All three new CI check names appear and pass on the migration PR

## Success Metrics

- `repo-settings.sh report` exits 0 (compliant) for CI workflow compliance
- First release using reusable workflows completes the full pipeline: build -> publish -> draft release -> bottles ->
  finalize
- No manual intervention needed for the release pipeline

## Dependencies & Risks

| Dependency | Status | Risk |
|---|---|---|
| Reusable workflows deployed to `brettdavies/.github` | Done (on main) | None |
| `CI_RELEASE_TOKEN` secret in 1Password | Exists | Must be added to xurl-rs repo |
| Trusted Publishing + reusable workflows | **Untested** | May need crates.io config update |
| Homebrew formula update for completions subcommand | Not started | Must coordinate with release tag |

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Trusted Publishing OIDC rejects callee workflow | Medium | Blocks release | Update crates.io config or use one-time token |
| `deny.toml` fails on new transitive dep | Low | Blocks CI | Add license to allow-list |
| Homebrew bottle build fails (completions flag) | High if not addressed | Stuck draft release | Update formula before tagging |
| Stuck draft release | Low | Cosmetic (admin can fix) | Manual `gh api` publish |

## Implementation Steps

### Phase 1: Preparation (local, no PR)

1. Create `deny.toml` — start from bird's, add `CDLA-Permissive-2.0`, run `cargo deny check` locally
2. Add `CI_RELEASE_TOKEN` secret to repo: `op read "op://secrets-dev/CI_RELEASE_TOKEN/credential" | gh secret set
   CI_RELEASE_TOKEN --repo brettdavies/xurl-rs`
3. Verify Trusted Publishing config on crates.io — check if `workflow` field needs updating for callee path

### Phase 2: Release branch and file changes

1. `git checkout -b release/ci-migration origin/main` — branch from main per release branch pattern
2. Replace `ci.yml` with thin caller template (from `~/.claude/skills/rust-tool-release/templates/ci.yml`)
3. Replace `release.yml` with thin caller template, substituting `crate: xurl-rs`, `bin: xr`
4. Create `finalize-release.yml` from template
5. Add `deny.toml`
6. Update `Cargo.toml`: release profile, binstall metadata, exclude list
7. Update `protect-main.json` with new status check names
8. Cherry-pick completions refactor (`a6a74e0`) if shipping in same release

### Phase 3: PR and merge

1. Push release branch, open PR to main
2. Verify all new CI checks pass (`ci / Fmt, clippy, test`, `ci / Package check`, `ci / Security audit (bans licenses
   sources)`, `check-forbidden-docs`)
3. Merge with admin bypass (old check name won't appear)
4. Update live ruleset via `gh api repos/brettdavies/xurl-rs/rulesets/<id> -X PUT --input
   .github/rulesets/protect-main.json`

### Phase 4: Post-merge

1. Sync dev to main: `git checkout dev && git merge origin/main && git push`
2. Clean up release branch
3. Run `repo-settings.sh report brettdavies/xurl-rs` to confirm compliance

### Phase 5: First release (validates full pipeline)

1. Update Homebrew formula for completions subcommand (if included)
2. Bump version in Cargo.toml, update CHANGELOG.md
3. Tag and push: `git tag vX.Y.Z && git push origin main --tags`
4. Monitor: build -> publish -> draft release -> bottles -> finalize
5. After success: `gh secret delete HOMEBREW_TAP_TOKEN --repo brettdavies/xurl-rs`

## Sources & References

### Skills (authoritative)

- `~/.claude/skills/rust-tool-release/SKILL.md` — CI/CD standard, templates, release process
- `~/.claude/skills/github-repo-setup/SKILL.md` — repo settings, rulesets, required status checks
- `~/.claude/skills/rust-tool-release/templates/` — thin caller templates for ci, release, finalize-release

### Solutions docs (institutional learnings)

- `reusable-ci-workflow-skill-healing-20260320.md` — reusable workflow architecture and naming
- `github-pat-consolidation-across-repos-20260319.md` — CI_RELEASE_TOKEN standard
- `changelog-as-committed-artifact-20260319.md` — why changelog job was removed from CI
- `pre-release-hardening-cargo-deny-ci-System-20260223.md` — deny.toml gotchas (v0.16+ schema)
- `release-branch-pattern-for-guarded-docs-20260317.md` — branch from origin/main, not dev
- `release-pipeline-cross-platform-publish.md` — pipeline architecture (marked stale for inline)

### Reference implementation

- `~/dev/bird` main branch — thin caller workflows, deny.toml, Cargo.toml with binstall metadata
- `brettdavies/.github` main branch — deployed reusable workflows (rust-ci, rust-release, rust-finalize-release)

### Audit output

- `repo-settings.sh report brettdavies/xurl-rs` — 2 CI compliance failures (ci.yml, release.yml)
- `pin-actions.sh /home/brett/dev/xurl-rs` — 12 pinned, 0 issues
