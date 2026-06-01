# Releasing `xurl-rs`

Every change reaches production via this pipeline. Direct commits to `dev` or `main` are not permitted — every change
has a PR number in its squash commit message, which keeps the history scannable, attributable, and changelog-ready.

```text
feature branch → PR to dev (squash merge)
              → cherry-pick to release/* branch
              → PR to main (squash merge)
              → tag push triggers crates.io publish + GitHub Release + Homebrew dispatch
```

## Branches

| Branch                                 | Role                                    | Lifetime                                    | Protection                           |
| -------------------------------------- | --------------------------------------- | ------------------------------------------- | ------------------------------------ |
| `main`                                 | Production. Only release commits.       | Forever.                                    | `.github/rulesets/protect-main.json` |
| `dev`                                  | Integration. All feature PRs land here. | Forever. Never delete.                      | `.github/rulesets/protect-dev.json`  |
| `feat/*`, `fix/*`, `chore/*`, `docs/*` | Feature work.                           | One PR's worth. Auto-deleted on merge.      | None — squash into dev freely.       |
| `release/*`                            | Head of a dev → main PR.                | One release's worth. Auto-deleted on merge. | None.                                |

`dev` is a **forever branch**. Never delete it locally or remotely, even after a `release/* → main` merge. The next
release cycle reuses the same `dev`. The repo's `deleteBranchOnMerge: true` setting doesn't touch `dev` as long as `dev`
is never the head of a PR — using a short-lived `release/*` head is what keeps the setting compatible with a forever
integration branch.

## Daily development (feature → dev)

```bash
git checkout dev && git pull
git checkout -b feat/short-description
# ... work ...
git push -u origin feat/short-description
gh pr create --base dev --title "feat(scope): what changed"
# CI passes → squash-merge (PR_BODY becomes the dev commit message)
```

- **Commit style**: [Conventional Commits](https://www.conventionalcommits.org/).
- **PR body**: follow `.github/pull_request_template.md`. The `## Changelog` section is the source of truth for
  user-facing release notes — `git-cliff` extracts these bullets verbatim into `CHANGELOG.md` during release prep.
- **No explainer prose anywhere in the body.** User-facing substance only — what is changing for the consumer that was
  not already there. Do NOT recap the workflow (cherry-pick / regenerate / pre-push gate / CI behavior is documented in
  this file and `.github/`).
- **Summary describes the net diff only** — what merged `main` looks like vs the base branch. One short paragraph; not
  commit history, intermediate state, or cherry-pick mechanics.
- **Zero verification artifacts in the body.** No triple-diff stats, leak-check output ("`guard-main-docs` runs clean"),
  patch-id cherry-check counts, pre-push gate results, CI status, prose-scrub findings, or exclusion rationale. Those
  stay local; anomalies get fixed before push, not audit-trailed.
- **PR body prose scrub**: `gh pr create` and `gh pr edit` send body text directly to GitHub; no automated check sees
  it. Save the body to `/tmp/`, run Vale + LanguageTool + unslop, fix findings, then submit via `--body-file`. See
  [§ Prose scrubbing](#prose-scrubbing).

## Releasing dev to main

Engineering docs (`docs/plans/`, `docs/solutions/`, `docs/brainstorms/`, `docs/reviews/`) live on `dev` only.
`guard-main-docs.yml` blocks them from reaching `main`. Use the release-branch cherry-pick pattern:

**Branch naming**: `release/v<version>` or `release/v<version>-<slug>` (e.g. `release/v1.0.5-ci-migration`,
`release/v1.2.0-library-ergonomics`). The `v<version>` prefix is required — `scripts/generate-changelog.sh` extracts the
version from the branch name.

```bash
# 1. Branch from main, NOT dev. Branching from dev causes add/add conflicts
#    when dev and main have divergent histories (the post-squash-merge norm).
git fetch origin
git checkout -b release/v1.2.0-library-ergonomics origin/main

# 2. List the dev commits not yet on main:
git log --oneline dev --not origin/main

# 3. Cherry-pick the ones you want to ship. Docs commits stay on dev.
git cherry-pick <sha1> <sha2> ...

# 4. Triple-diff verification — belt-and-suspenders sweep that catches both
#    directions of drift before the release tag goes out:
#
#    A. main → release  (what users will see; the intended ship surface)
#    B. release → dev   (should be empty for non-doc paths until the
#                        bump/CHANGELOG commits land, and even then should
#                        only list those release-prep files — anything else
#                        is a missed cherry-pick)
#    C. dev → main      (sanity: phantom commits dev "appears ahead" on
#                        because cherry-pick rewrites SHAs post-squash)
git diff origin/main..HEAD --stat                                                # A
git diff HEAD..origin/dev --name-only | grep -v '^docs/' || echo "(none)"        # B
git diff origin/dev..origin/main --stat | tail -5                                # C
#
# Re-confirm no guarded paths leaked (this caught the original miss class):
git diff origin/main..HEAD --name-only \
  | grep -E '^(docs/plans|docs/brainstorms|docs/ideation|docs/reviews|docs/solutions|\.context)' \
  && echo "LEAKED — reset and redo" || echo "(clean — no guarded paths)"
#
# Patch-id cherry check — catches commits on dev that have NO patch-id
# equivalent on release. The file-level diff in B misses this class when
# the same content happens to land via a different commit.
#
# IMPORTANT: in a squash-merge workflow this output is noisy. Every '+'
# line needs human triage — it does NOT auto-block the release. Expected
# sources of '+' lines that are NOT real misses:
#
#   1. Historical commits squash-merged in prior releases. The squash
#      commit on main has a different patch-id than the dev commits it
#      consolidates, so old commits show as '+' forever. Anything older
#      than the previous release tag is almost always this.
#   2. Cherry-picks where conflict resolution stripped guarded paths
#      (docs/plans, docs/brainstorms, etc.) or otherwise altered the
#      tree. Same source-code intent, different patch-id.
#   3. Intentionally skipped commits — docs-only commits, release-prep
#      backports, revert-and-redo prep steps.
#
# A real miss looks like: a recent feat/fix/chore commit on dev whose
# *file content* is not yet on main. To triage a '+' line:
#
#   git show <sha> --stat                       # what did it touch?
#   git diff origin/main..HEAD -- <those-files> # already on release?
#
# If every touched file is guarded (docs/plans/, docs/brainstorms/, etc.)
# OR the content is already on main via a prior squash, it's a false
# positive — no action. Otherwise cherry-pick the commit and re-run the
# triple-diff.
git cherry HEAD origin/dev | grep '^+' || echo "(none — release is patch-equivalent through dev)"
#
# If B lists any non-docs path you didn't expect, fetch dev, identify the
# commit (`git log dev --not origin/main`), cherry-pick it, re-run the
# triple-diff. Missed cherry-picks have shipped to main on this and sibling
# repos before — this step is the cheap way to catch them.

# 5. Bump version in Cargo.toml and commit:
#    sed -i 's/^version = ".*"/version = "1.2.0"/' Cargo.toml
#    cargo update -p xurl-rs   # refresh Cargo.lock
#    git add Cargo.toml Cargo.lock && git commit -m "chore: bump version to 1.2.0"

# 6. Regenerate completions (catches any subcommand/flag changes missed during dev):
~/.claude/skills/rust-tool-release/scripts/generate-completions.sh
git add completions/ && git commit -m "chore: regenerate shell completions" || true

# 7. Generate CHANGELOG.md (auto-detects version from branch name; CI enforces this):
~/.claude/skills/rust-tool-release/scripts/generate-changelog.sh

# 8. Review CHANGELOG.md. See "CHANGELOG is generated, never hand-written" below
#    for the cliff.toml chore-skip footgun and how to recover. Then scrub the
#    generated content through Vale + LanguageTool + unslop — CHANGELOG.md is a
#    generated artifact and inherits whatever prose its upstream PR bodies
#    carry. See "Prose scrubbing" below for the procedure. Fix findings on the
#    upstream PR body and re-run scripts/generate-changelog.sh, not by
#    hand-editing CHANGELOG.md. When clean, commit:
git add CHANGELOG.md && git commit -m "docs: update CHANGELOG.md for v1.2.0"

# 9. Push and open the PR:
git push -u origin release/v1.2.0-library-ergonomics
gh pr create --base main --head release/v1.2.0-library-ergonomics --title "release: v1.2.0"
```

When the PR merges, the deploy / publish workflow picks up the push to `main`. Auto-delete removes the release branch
from the remote on merge. `dev` is untouched.

### Why branch from main, not dev

Branching from `dev` and then `gio trash`-ing the guarded paths seems simpler but produces `add/add` merge conflicts
whenever `dev` and `main` have diverged (which they always do after the first squash merge). The file appears as "added"
on both sides with different content. Always branch from `origin/main` and cherry-pick onto it.

### CHANGELOG is generated, never hand-written

`scripts/generate-changelog.sh` (with `cliff.toml`) is the only sanctioned way to update `CHANGELOG.md`. It requires
`--tag vX.Y.Z` (or extracts the version from the release branch name) and prepends a versioned section while preserving
existing content. The script runs `git-cliff` to prepend a versioned entry for commits since the last tag, then walks
each squash-merged PR's body to extract the `## Changelog` section's `### Added` / `### Changed` / `### Fixed` / `###
Documentation` subsections, replacing the auto-generated bullets with the curated PR-body content (with author and
PR-link attribution).

If a PR's `## Changelog` section is empty, that PR's entry is omitted from the changelog (the convention in
[`.github/pull_request_template.md`](.github/pull_request_template.md): empty section = no user-facing change). To fix a
wrong CHANGELOG entry, fix the input — edit the squash-merged PR body, then re-run the script. Do **not** edit
`CHANGELOG.md` directly.

CI enforces that `CHANGELOG.md` is modified in every PR to main (`ci / Changelog` required status check) and that it
contains a versioned section, not `[Unreleased]`. The release workflow extracts the latest section for the GitHub
Release body.

**`cliff.toml` skips `chore`/`style`/`test`/`ci`/`build` commits regardless of PR-body content.** If a cherry-picked PR
has user-facing `## Changelog` content but its commit subject starts with one of those types, its bullets get silently
dropped. After running the script, cross-check the generated section against `gh pr view <num> --json body` for each
cherry-picked PR; correct mistyped PR titles (e.g. `chore` → `feat`) and re-amend the cherry-pick subject before
re-running. See "Prefer `feat`/`fix` over `chore`" in global CLAUDE.md for prevention.

## Prose scrubbing

Three release-flow artifacts live outside any automated prose check and need a manual scrub before they ship. The rule
packs and orchestrator behavior referenced below are documented at
[`~/dev/agentnative-spec/docs/architecture/voice-enforcement.md`](../agentnative-spec/docs/architecture/voice-enforcement.md):

- **PR bodies.** `gh pr create` and `gh pr edit` send body text directly to GitHub; no in-repo check sees it.
- **`CHANGELOG.md`.** Generated artifact built from upstream PR bodies. Findings inherit whatever prose those PR bodies
  carry.
- **Release-PR bodies.** The `release/v<version> → main` PR gets wrap-up text contributors edit after `CHANGELOG.md` has
  been generated, and the same out-of-repo gap applies.

The scrub procedure:

```bash
# 1. Save the artifact to /tmp/. The auto-format hook skips /tmp paths, so the
#    body keeps its authored shape and no soft-wrapping is injected.
gh pr view <num> --json body --jq .body > /tmp/body.md         # for PR body edits
# cp CHANGELOG.md /tmp/body.md                                 # for changelog scrub

# 2. Vale (against the spec's rule packs — until vendored locally, point at the
#    spec checkout).
vale --no-global --config ~/dev/agentnative-spec/.vale.ini --output=line --minAlertLevel=error /tmp/body.md

# 3. LanguageTool (blocking categories: TYPOS|GRAMMAR|CONFUSED_WORDS, mirrors
#    the orchestrator's whitelist).
curl -sS -X POST "${LANGUAGETOOL_URL:-http://pool.tail42ba87.ts.net:8081}/v2/check" \
  --data-urlencode "language=en-US" --data-urlencode "text@/tmp/body.md" \
  | jaq '.matches[] | select(.rule.category.id | test("^(TYPOS|GRAMMAR|CONFUSED_WORDS)$"))'

# 4. unslop (em-dash density and AI-unique structural patterns Vale + LT do not catch).
~/.claude/skills/unslop/scripts/score.py /tmp/body.md

# 5. Apply fixes per finding. Re-run until 0 blocking and unslop score is 0.

# 6. Apply the cleaned version:
gh pr edit <num> --body-file /tmp/body.md     # for PR body edits
# scripts/generate-changelog.sh                # for CHANGELOG.md (re-runs the
#                                              # PR-body fetch from GitHub)
```

For a `CHANGELOG.md` finding, fix the upstream PR body (which `generate-changelog.sh` re-fetches every run) and
regenerate. Hand-editing `CHANGELOG.md` directly produces drift the next regeneration overwrites.

## Tagging and publishing

After the `release/v<version> → main` PR merges, tag and push:

```bash
git checkout main && git pull
git tag -a -m "Release v1.2.0" v1.2.0
git push origin main --tags
```

> Always use annotated tags (`-a -m`). Bare `git tag <name>` silently fails with
> `fatal: no tag message?` on machines where `tag.gpgsign=true` is set globally
> (a brettdavies dotfile default). See
> [solutions: git tag fails with tag.gpgsign — use annotated tags](https://github.com/brettdavies/solutions-docs/blob/main/best-practices/git-tag-fails-with-tag-gpgsign-use-annotated-tags-2026-04-13.md).

The tag push triggers `.github/workflows/release.yml`, which calls the reusable
`brettdavies/.github/.github/workflows/rust-release.yml@main` and runs:

| Step            | What                                                                                                                                                                                                                                                     |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `check-version` | Verify the tag matches `Cargo.toml` version (gate).                                                                                                                                                                                                      |
| `audit`         | `cargo deny check` (license + advisory + ban).                                                                                                                                                                                                           |
| `build`         | Cross-compile for 5 targets: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`. Each archive includes the `xr` binary, completions, README, and licenses.                 |
| `publish-crate` | `cargo publish` to crates.io via Trusted Publishing (OIDC, no static token).                                                                                                                                                                             |
| `release`       | Create a **non-draft** GitHub Release with `make_latest: false` — visible immediately (so `cargo-binstall` and `/releases/latest` don't 404 during the bottle-build window) but not yet promoted to "Latest". Includes all 5 archives + `sha256sum.txt`. |
| `homebrew`      | Dispatch `update-formula` to `brettdavies/homebrew-tap` (formula `xurl-rs` installs `xr`).                                                                                                                                                               |

After the homebrew-tap workflow uploads bottles to this repo's release assets, it dispatches `finalize-release` back to
this repo, which idempotently flips `make_latest: true`. End result: crate on crates.io, GitHub Release marked latest,
Homebrew formula updated with bottles, all atomically advertised.

`cargo publish` runs BEFORE GitHub Release creation. If publish fails, no release is advertised and no Homebrew update
is triggered.

## crates.io publishing

Publishing uses Trusted Publishing via `rust-lang/crates-io-auth-action` — no static API token. OIDC exchanges a
short-lived GitHub Actions token for a ~30-minute crates.io token.

Trusted Publishing was configured after the v1.0.3 manual publish. To reconfigure:

1. `https://crates.io/settings/tokens/trusted-publishing`
2. Add trusted publisher: owner=`brettdavies`, repo=`xurl-rs`, workflow=`release.yml`
3. Enable "Enforce Trusted Publishing" to disable token-based publishing

## Branch protection

Two rulesets are committed under `.github/rulesets/` and applied to the repo via the GitHub API:

- `protect-main.json` — required signatures, linear history, squash-only merges via PR, required status checks,
  creation/deletion blocked, non-fast-forward blocked.
- `protect-dev.json` — required signatures, deletion blocked, non-fast-forward blocked. The PR-only norm is enforced by
  convention.

### Status-check context pitfall

The `required_status_checks[].context` strings in `protect-main.json` must match exactly what GitHub publishes for each
check:

- **Inline job** (with `name:` field): published as just `<job-name>` (no workflow-name prefix).
- **Reusable-workflow caller** (`uses: .../foo.yml@ref`): published as `<caller-job-id> / <reusable-job-id-or-name>`.

Mixing these produces a stuck-but-green PR: all actual checks report green, but the ruleset waits forever on a context
that will never appear. Confirm the real contexts after a first CI run with:

```bash
gh api repos/brettdavies/xurl-rs/commits/<sha>/check-runs --jq '.check_runs[].name'
```

## Required secrets

| Secret             | Purpose                                                                                                           | Lifecycle                                                      |
| ------------------ | ----------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| `CI_RELEASE_TOKEN` | Fine-grained PAT, Contents R+W, Pull requests R+W. Used by `release.yml` to dispatch the Homebrew formula update. | Rotated annually (max 1 year). 1Password vault: `secrets-dev`. |

`GITHUB_TOKEN` is automatic; CI (`ci.yml`) only needs `contents: read` and uses no extra secrets.

## Distribution channels

| Channel          | How                                                                              |
| ---------------- | -------------------------------------------------------------------------------- |
| Homebrew         | `brew install brettdavies/tap/xurl-rs`                                           |
| Pre-built binary | Download from [GitHub Releases](https://github.com/brettdavies/xurl-rs/releases) |
| Rust crate       | `cargo install xurl-rs` (binary) or `xurl_rs = "..."` in `Cargo.toml` (library)  |
| Fast binary      | `cargo binstall xurl-rs`                                                         |
| From source      | `git clone && cargo build --release`                                             |

## Related docs

- [`.github/pull_request_template.md`](.github/pull_request_template.md) — PR body structure with changelog sections
- [`README.md`](README.md) — install channels, library usage, CLI reference
