<!-- Canonical version: 2026.11.03 (generic) -->
<!-- canonical: version-scheme semver -->
# Releasing `xurl-rs`

Operational runbook. Rationale lives in [`RELEASES-RATIONALE.md`](./RELEASES-RATIONALE.md). Pre-cut go/no-go checklist
lives in [`RELEASES-PREFLIGHT.md`](./RELEASES-PREFLIGHT.md).

```text
feature branch → PR to dev (squash merge)
              → dev's tree overlaid onto a release/* branch cut from main
              → PR to main (squash merge)
              → tag-release.sh pushes the tags; the release runs on main:
                crates.io publish + GitHub Release + Homebrew dispatch
```

Direct commits to `dev` or `main` are not permitted: every change has a PR number in its squash commit message.

## Branches

| Branch                                 | Role                                    | Lifetime                                    | Protection                           |
| -------------------------------------- | --------------------------------------- | ------------------------------------------- | ------------------------------------ |
| `main`                                 | Production. Only release commits.       | Forever.                                    | `.github/rulesets/protect-main.json` |
| `dev`                                  | Integration. All feature PRs land here. | Forever. Never delete.                      | `.github/rulesets/protect-dev.json`  |
| `feat/*`, `fix/*`, `chore/*`, `docs/*` | Feature work.                           | One PR's worth. Auto-deleted on merge.      | None. Squash into dev freely.        |
| `release/*`                            | Head of a dev → main PR.                | One release's worth. Auto-deleted on merge. | None.                                |

→ Rationale: [`RELEASES-RATIONALE.md` § Branching model](./RELEASES-RATIONALE.md#branching-model).

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
- **PR body**: follow `.github/pull_request_template.md`. See [§ PR body](#pr-body).
- **PR body prose scrub**: see [§ Prose scrubbing](#prose-scrubbing).

### Dev-direct exception

Paths that live only on `dev` and never ship to `main` can be committed directly to `dev` without a feature branch or
PR. The `guard-main-docs` workflow blocks them from `main` PRs regardless. The exception applies to:

- Engineering docs: `docs/architecture/`, `docs/brainstorms/`, `docs/ideation/`, `docs/plans/`, `docs/research/`,
  `docs/reviews/`, `docs/solutions/`, and anything under `.context/`.
- Agent-facing glossary: `CONCEPTS.md`.

`scripts/release/guarded-paths.sh` prints the authoritative set, resolved from the workflow. Run it rather than trusting
this list, which is a reading aid. A path that stays off `main` but is not in the reusable workflow's base list is
registered through the caller's `extra_paths` input in `.github/workflows/guard-main-docs.yml`; entries are globs (`**/`
any depth, `*` and `?` within a segment, a trailing `/` guards the subtree).

The standard feature → PR → squash-merge flow remains required for everything else, including consumer-facing markdown
(README, AGENTS, CONTRIBUTING, CHANGELOG, in-repo runbooks). `docs/migrating/` ships to `main`.

## PR body

Every PR (feature, fix, docs, release) uses `.github/pull_request_template.md` verbatim. Six required sections, in
template order, no inventions: `## Summary`, `## Changelog`, `## Type of Change`, `## Related Issues/Stories`, `##
Testing`, `## Files Modified`. The template's other sections (`## Key Features`, `## Benefits`, `## Breaking Changes`,
`## Deployment Notes`, `## Screenshots/Recordings`, `## Checklist`, `## Additional Context`) are optional: fill `##
Breaking Changes` on a major, delete the rest when they do not apply.

- **No explainer prose anywhere in the body.** User-facing substance only.
- **Summary describes the net diff only**: what merged `main` looks like vs the base branch. Not commit history,
  intermediate state, or cherry-pick mechanics.
- **Zero verification artifacts in the body.** No triple-diff stats, leak-check output ("`guard-main-docs` runs clean"),
  patch-id cherry-check counts, pre-push gate results, CI status, or prose-scrub findings. Anomalies get fixed before
  push, not audit-trailed.
- **Changelog** subsections (`### Breaking changes` / `### Added` / `### Changed` / `### Deprecated` / `### Fixed` /
  `### Documentation`, the six headings of [§ Versioning](#versioning)): 1-5 bullets each, delete empty subsections,
  each bullet starts with a verb. The subsection an
  entry sits under sets the release's version (§ Versioning). A crate's `## Changelog (<crate>)` heading left standing
  with no bullets under it says the PR ships nothing user-facing for that crate, and `generate-changelog.py` adds
  nothing for it, whatever the PR title says.
- **Type of Change**: one checkbox. Prefer `feat`/`fix` over `chore` for any user-observable change.
- **Related Issues/Stories**: four labels (`Story:` / `Issue:` / `Architecture:` / `Related PRs:`). All four required
  even when empty (`- None.` / `n/a`).
- **Files Modified**: four sub-headers (`Modified` / `Created` / `Renamed` / `Deleted`). All four required even when
  empty.
- **No AI attribution** in commits or PR bodies.
- **No hard line wraps**: one logical line per paragraph or bullet.

→ Rationale: [`RELEASES-RATIONALE.md` § PR body conventions](./RELEASES-RATIONALE.md#pr-body-conventions).

## Versioning

The marked lines below are github-repo-setup's SemVer section of `references/versioning.md`, which the drift check
compares, and line 2 declares the scheme. Both crates follow it, each on its own tag line: every PR files each crate's
changes in that crate's `## Changelog (<crate>)` block, and each crate's release takes the highest bump its own PRs
call for.

<!-- canonical:begin versioning-semver -->

This repo follows [SemVer 2.0.0](https://semver.org/). Its version comes from its changelog: every PR files each
user-facing change under one of the six headings below, and a release takes the highest bump any of its PRs calls for.
The rules cover only the surfaces § What the contract is names.

### What the contract is

<!-- canonical:repo-begin contract -->

The repository releases two crates, each with its own contract and its own tag line.

#### `xr`

`xr` (package `xurl-rs`, tags `vX.Y.Z`) has its machine contract as its public API:

- **Invocation:** command names and aliases, flags, positional forms, and every environment variable `xr --help` lists.
- **Exit codes:** the table in `crates/xurl-cli/README.md`.
- **Structured output:** every shape `--output json|jsonl|ndjson|yaml|csv|tsv` prints, meaning the success responses in
  `schema/responses/` and `xr schema`, and the error envelope in `schema/output.schema.json`, including its `reason` and
  `next_step.action` sets.
- **Stored state:** the `~/.xurl/auth.yml` token store format.

Text-mode output, help text, colors, `--verbose` diagnostics, suggestion wording, and raw-mode response bodies (X's
bytes, passed through) are not contract. The one text-mode exception is the plain `xr --version` line, `xr X.Y.Z`, which
scripts parse. A script that needs any other stable output reads a structured format.

#### `xdk-rs`

`xdk-rs` (tags `xdk-rs-vX.Y.Z`) has its public Rust API as its contract, which `cargo semver-checks` reads, and the
behavior its rustdoc documents. Items marked `#[doc(hidden)]` are outside it.

<!-- canonical:repo-end -->

### The six headings

| Heading            | Holds                                                                   | Bump  |
| ------------------ | ----------------------------------------------------------------------- | ----- |
| `Breaking changes` | a contract surface removed, renamed, or retyped, or its meaning changed | major |
| `Added`            | a new contract surface                                                  | minor |
| `Changed`          | a visible change outside the contract                                   | patch |
| `Deprecated`       | a contract surface marked for removal                                   | minor |
| `Fixed`            | behavior brought back to the documented contract                        | patch |
| `Documentation`    | documentation only                                                      | patch |

`generate-changelog.py` writes them in this order and any other heading after them. A contract removal is a
`Breaking changes` bullet that starts with "Remove"; a removal outside the contract goes under `Changed`, or is left
out. Removals ship when they are ready, with no deprecation schedule. A `type!:` commit is a prompt to check the PR's
sections, never the bump itself.

### The bump

`scripts/generate-changelog.py --bump` prints the next version from the PRs merged since the last release, and
`cut-release-branch.sh` prints it beside the version the cut is given:

- The highest heading among their sections sets the bump.
- A `rust-version` raised since the last tag is a minor-level change, so it lifts a patch to a minor. A Rust repo's
  `rust-version` equals the toolchain `rust-toolchain.toml` pins, and CI's MSRV job builds on exactly that version.
- Before 1.0, a major or a minor moves the middle number and a patch moves the last: `0.4.2` becomes `0.5.0` or `0.4.3`.
  This is stricter than Cargo's convention on purpose: an addition never ships as a last-number bump, so a `^0.y`
  dependent receives only fixes until it opts in.
- From 1.0, when the sections call for a major, the release agent asks the maintainer: a major, or a minor under the
  small-break exception. A minor ships `docs/migrating/v<version>.md`, whose first line names the exception, and
  preflight fails a 1.0+ release that keeps its major and carries `### Breaking changes` without that guide.

### Overrides

<!-- canonical:repo-begin overrides -->

None. A repo that departs from the bump, the pre-1.0 rule, or the MSRV rule states the rule it follows here.

<!-- canonical:repo-end -->

<!-- canonical:end versioning-semver -->

### What each bump holds for `xr`

- **Patch:** only fixes. Behavior brought back to the documented contract or to how the X API behaves, even when an exit
  code or a JSON key changes as a result. Documentation, text-mode changes, and dependency or security updates that
  leave the contract alone. Nothing new.
- **Minor:** anything new that breaks nothing. A command, flag, environment variable, output field, output format,
  `reason`, or `action`; endpoint coverage a spec refresh adds; a deprecation; an MSRV raise.
- **Major:** anything in the contract removed, renamed, or retyped, or its documented meaning changed.

A minor can carry fixes, and a major can carry both. A break taken into a minor under the small-break exception
(§ The bump) ships a guide under `docs/migrating/` that maps each old invocation to its replacement, and `MIGRATING.md`
lists it.

### X API changes

X ships additive API changes often and does not version them.

- X adds an endpoint or a field: `xr` adds a command or a field, a minor.
- X renames or drops something the contract names: read both spellings and keep emitting the contract's name, the way
  typed responses read X's legacy post vocabulary. When the contract's own name has to change, add the new name in a
  minor, deprecate the old one, and remove it in a major.
- X changes behavior in a way that makes `xr` wrong: the fix is a patch.

### Closed sets

`reason` and `next_step.action` are closed sets an agent branches on. A new member ships in a minor, and a consumer
treats a value it does not recognize as its default branch. Removing or renaming a member is a major.

### Two tag lines

The two tag lines stay independent. A library release moves the CLI's version only through an MSRV raise, which both
crates share because `rust-version` is declared once in the root `Cargo.toml`, or when the CLI adopts new library API
and the `xdk-rs` requirement in the root `Cargo.toml` moves. An `xdk-rs` break is filed under `### Breaking changes`
with its before/after snippet (the policy in `crates/xdk/README.md`).

### The changelog bump check

The `Changelog bump` workflow backs the review for `xr`. It calls the shared `rust-changelog-bump.yml` reusable, which
runs `scripts/check-surface-bump.sh` to compare the commands, flags, and flag values in `completions/xr.bash` and every
schema under `schema/` against the PR's base. It fails when that surface grows while the `## Changelog (xurl-rs)` block
has no bullet under `### Added`, `### Deprecated`, or `### Breaking changes`, and when the surface shrinks while the
block has none under `### Breaking changes` or `### Fixed`. It reads the block with the changelog generator's own
parser, re-runs when the PR body is edited, and is a required check on `dev`
(`changelog-bump / Surface growth needs a minor section`). Env vars, exit codes, and the store format have no generated
artifact to compare, so review alone covers them. The `xdk-rs` block is not read: the library's surface is its Rust
API, which `cargo semver-checks` compares in CI's public-API semver job and in preflight's api-contract gate.

At release time, `scripts/generate-changelog.py --bump --crate <crate>` prints each crate's next version from the PRs
merged since its last tag, and `scripts/generate-changelog.py --crate <crate> --from-dev-prs --tag <tag> --dry-run`
lists the entries behind it.

→ Rationale: [`RELEASES-RATIONALE.md` § Versioning](./RELEASES-RATIONALE.md#versioning).

## Releasing dev to main

Before cutting a release branch, walk [`RELEASES-PREFLIGHT.md`](./RELEASES-PREFLIGHT.md) end-to-end. Any unchecked item
holds the release.

Engineering docs (`docs/plans/`, `docs/solutions/`, `docs/brainstorms/`, `docs/reviews/`) live on `dev` only.
`guard-main-docs.yml` blocks them from reaching `main`, and `guard-release-branch.yml` rejects any PR to main whose head
isn't `release/*`.

**Branch naming**: `release/v<version>` or `release/v<version>-<slug>` (e.g. `release/v1.0.5-ci-migration`,
`release/v1.2.0-library-ergonomics`). The `v<version>` prefix is required: `generate-changelog.py` extracts the version
from the branch name.

`main` and `dev` share only an ancient merge-base: every release squash-merges into `main`, so the two branches diverge
in history even as their content converges. Reconciling that with a merge, or a branch cut from `dev`, produces a pile
of rename/delete and lockfile conflicts that are artifacts of the lineage, not of the content shipping. The release
branch is therefore built as a **clean descendant of `main`** with `dev`'s tree overlaid on top, asserting the desired
end-state directly. `scripts/release/cut-release-branch.sh` builds it:

```bash
# 1. Build the branch: drift gate, branch from main, overlay dev's tree, strip the
#    guarded paths, generate the CLI's changelog, and run checks A, B, and D. Stops
#    before committing; --dry-run prints the plan and touches nothing. The version
#    comes from § Versioning: the highest bump the merged PRs' sections call for.
scripts/release/cut-release-branch.sh 1.3.0

# 2. Bump the version carriers, refresh the lockfile and the completions, and generate
#    the library's changelog, in the order § Project specifics lists.

# 3. Commit the overlay as one commit sitting directly on top of main, then run the
#    preflight gates against it.
git add -A
git commit
cargo build --release --bin xr
scripts/release/preflight.sh all

# 4. Push and open the PR. Scrub the body in /tmp/ first.
git push -u origin release/v1.3.0
gh pr create --base main --head release/v1.3.0 --title "release: v1.3.0" --body-file /tmp/body.md
```

The script asserts `dev`'s tree onto the `main` base with `git read-tree -u --reset`, one operation that carries the
deletions too, so a file `main` carries and `dev` deleted or moved cannot ship. It strips the paths `guard-main-docs`
forbids, resolved from the workflow by `scripts/release/guarded-paths.sh` rather than from any restated copy. It writes
`crates/xurl-cli/CHANGELOG.md` with `scripts/generate-changelog.py --crate xurl-rs --from-dev-prs --tag v<version>`,
naming the crate because `RELEASE_MANIFEST` in `scripts/release/release.env` is the binary's manifest, not the root's.
Then it runs three checks:

| Check                                    | Asserts                                                                                                                                                                           | On failure                                                                                                     |
| ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| A: staged tree vs `origin/dev`           | nothing differs but the version carriers (a `Cargo.toml`, `Cargo.lock`, or `CHANGELOG.md` at any depth, so each member's counts) and the stripped guarded paths                   | fails, naming each unexpected path                                                                             |
| B: guarded paths vs `origin/main`        | no guarded path is added or modified (`--diff-filter=ACMR`, so removing a guarded doc `main` still carries reads as cleanup, not a leak)                                          | fails, naming each leak                                                                                        |
| D: unguarded docs added to `origin/main` | every added `docs/` entry or markdown file is meant to ship (`--no-renames`, so a doc moved from one `main` carries lists as added rather than as a rename the filter would drop) | reports, never fails: each needs a reason to ship, or registering in the workflow's `extra_paths` and removing |

Only exit 0 leads to step 2:

| Exit | Meaning                                                                                    | Next                                                                                          |
| ---- | ------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------- |
| 0    | The branch is staged and every check passed; the script prints the commit steps.           | Step 2.                                                                                       |
| 1    | The drift gate, the changelog step, or a check failed.                                     | Never commit the branch. A drift failure stops before branching; any other recovers as below. |
| 2    | Setup error: a dirty worktree, an unknown ref, a missing tool, or no guarded-path pattern. | Fix what it names. Only the guarded-path error stops after branching; it recovers as below.   |

A cut that stopped after branching, at a failed check or at any failing step, leaves the overlay staged on the release
branch and prints the way back instead of the commit steps. The worktree was clean when the cut started, so everything
staged is the script's own output, and discarding it loses nothing:

```bash
git checkout -f dev
git branch -D release/v1.3.0
```

Then fix the cause and re-run. The script refuses a dirty worktree, so a failed cut left in place blocks the re-run.

The result is a single commit whose diff against `main` is the release, with `main` as an ancestor, so the PR merges
with zero conflicts. The merge publishes nothing: no workflow here runs on a push to `main`, and every artifact comes
from the annotated tags ([§ Tagging and publishing](#tagging-and-publishing)), so merge and tag in one sitting and
`main` never documents a version with no release to install. Auto-delete removes `release/v1.3.0` from the remote on
merge. `dev` is untouched.

→ Rationale (why overlay, not merge; why cut from `main`):
[`RELEASES-RATIONALE.md` § Branching model](./RELEASES-RATIONALE.md#branching-model). CHANGELOG mechanics:
[`RELEASES-RATIONALE.md` § CHANGELOG generation](./RELEASES-RATIONALE.md#changelog-generation).

### Exception: cherry-pick

The overlay is the release construction for this repo. Cherry-picking the dev squash-commits onto the `origin/main` base
is the exception, kept for a repo that has a stated reason it cannot overlay (record it under Project specifics); the
per-PR changelog is not such a reason, since `--from-dev-prs` builds it from `dev` either way. When cherry-picking, run
the triple-diff verification:

```bash
# 1. Nothing on main that dev never received, then branch from main, NOT dev.
scripts/release/drift.sh
git fetch origin
git checkout -B release/v1.3.0 origin/main

# 2. List the dev commits not yet on main.
git log --oneline dev --not origin/main

# 3. Cherry-pick the ones to ship. Docs commits stay on dev.
git cherry-pick <sha1> <sha2> ...

# 4. Triple-diff verification.
GUARDED="$(scripts/release/guarded-paths.sh)"

git diff origin/main..HEAD --stat                                              # A: ship surface
git diff HEAD..origin/dev --name-only | grep -Ev "$GUARDED" || echo "(none)"   # B: no missed picks
git diff origin/dev..origin/main --stat | tail -5                              # C: phantom-commits sanity

# Re-confirm no guarded paths leaked. --diff-filter=ACMR for the same reason as the
# overlay's check B: a deletion of a guarded path main still carries is cleanup, not a
# leak, and an unfiltered grep aborts a correct release over it.
git diff origin/main..HEAD --diff-filter=ACMR --name-only \
  | grep -E "$GUARDED" \
  && echo "LEAKED: reset and redo" || echo "(clean)"

# D: what this release ADDS to main (see the overlay's check D for why).
# `--no-renames` lists a doc moved from one main carries as added; rename detection
# would report it as R, and the A filter would drop it.
git diff --no-renames origin/main..HEAD --diff-filter=A --name-only | grep -E '(^docs/|\.md$)' | grep -Ev "$GUARDED" || echo "(none unguarded)"

# Patch-id cherry check (noisy in squash-merge workflow; triage per-line).
git cherry HEAD origin/dev | grep '^+' || echo "(none)"
```

Cherry-picks of PRs that touched guarded paths hit modify/delete or rename/delete conflicts, since those paths live on
`dev` but are blocked from `main`; resolve them per the next section. Then generate the CLI's changelog the way the cut
script does (`scripts/generate-changelog.py --crate xurl-rs --from-dev-prs --tag v1.3.0`), work through § Project
specifics, and finish with steps 3 and 4 of the overlay procedure.

→ Triple-diff false-positive triage:
[`RELEASES-RATIONALE.md` § Triple-diff verification](./RELEASES-RATIONALE.md#triple-diff-verification).

### Cherry-pick conflicts on guarded paths

Cherry-picks of feature PRs that touched `docs/plans/` / `docs/brainstorms/` / `docs/ideation/` / `docs/reviews/` /
`docs/solutions/` / `.context/` files will hit modify/delete conflicts on the release branch. Those paths exist on `dev`
but are blocked from `main` by `guard-main-docs.yml`, so the cherry-pick sees them as "deleted in HEAD, modified in
`<commit>`". A PR that renames such a file also produces rename/delete conflicts on the same paths.

Resolution (the standard `git rm` is denied by repo policy; use the plumbing form):

```bash
# 1. Mark every unmerged guarded path as deleted in the index.
git update-index --remove $(git diff --name-only --diff-filter=U)

# 2. Trash the orphan worktree files left by the rename target side.
gio trash docs/plans/<leftover-paths>.md

# 3. Continue the cherry-pick.
git cherry-pick --continue --no-edit
```

Repeat per conflicting commit. After all picks land, run `git ls-files docs/plans/ docs/brainstorms/`. If anything
remains, drop it with the same two-step pattern and commit as `chore(release): drop stray plan spikes from cherry-pick
rename detection` before the leak check. Rename detection occasionally re-adds a path under the rename target's new
name; the post-pick `ls-files` check catches that.

## Tagging and publishing

After the `release/v<version> → main` PR merges, tag from `main`:

```bash
git checkout main && git pull --ff-only
scripts/release/tag-release.sh --dry-run   # every check, and the tags it would push; creates nothing
scripts/release/tag-release.sh
```

The script tags `main`'s head with `v<version>`, the version in `crates/xurl-cli/Cargo.toml`, and with
`xdk-rs-v<version>` when the library's version has no tag yet, then pushes them in one atomic push: the remote receives
every tag or none. The tags are annotated, which a machine with `tag.gpgsign` set requires. Before it creates a tag, it
lists the queued and in-progress workflow runs in every repository the account owns and has not archived. Those runs
share the account's concurrent-job limit with the release's builds, so a listed run holds the cut until it finishes, or
until `--force`. On success it prints a `Next:` block naming the runs to watch and the postflight command for each tag.

| Exit | Meaning                                                                                                                                                                              | Next                                                    |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------- |
| 0    | Every tag is pushed. Under `--dry-run`, every check passed and nothing was created.                                                                                                  | Watch the runs the `Next:` block names.                 |
| 1    | `runners-busy`, or `push-failed`, after which the tags the run created are deleted.                                                                                                  | The code's heading below, then run the script again.    |
| 2    | A refusal before any tag exists: `dirty-tree`, `main-not-current`, `remote-unreachable`, `workspace-unreadable`, `tag-exists`, or `runner-check-failed`; or a missing tool, by name. | The code's heading below, or install the tool it names. |

### When the cut stops

Each refusal prints `error[<code>]:` on stderr with the values behind it, then a `next:` line and a `see:` line naming
the heading here.

#### `dirty-tree`

The working tree has uncommitted changes, which the tagged commit would not carry. Commit or stash them, then run the
script again.

#### `main-not-current`

The checkout is not on `main`, or its `main` is not `origin/main`. Off `main`, or behind it,
`git checkout main && git pull --ff-only` fixes it. A `main` that carries commits `origin/main` lacks would tag a commit
the remote's `main` never received, so a person decides: reset `main` to `origin/main`, or land those commits through a
pull request first.

#### `remote-unreachable`

`git fetch` or `git ls-remote --tags` against the remote failed. Check `git remote -v` and the network, then run the
script again.

#### `workspace-unreadable`

`cargo metadata` failed, and the error quotes what cargo printed, so the script cannot tell which workspace members to
tag. Without that check it would tag `v<version>` alone and leave `xdk-rs-v<version>` behind. The usual cause is a
toolchain that `rust-toolchain.toml` pins and rustup could not install. Fix what cargo names, then run the script again.

#### `tag-exists`

`v<version>` is already on the remote, so this version was cut. A new release bumps the version first; a release already
running is followed with `scripts/release/postflight.sh --tag v<version> all`. A tag in the checkout that the remote
lacks is left from an earlier attempt: `git tag -d <tag>`, then run the script again. A library tag already on the
remote is no refusal: the library did not move, and only `v<version>` is pushed.

#### `runner-check-failed`

`gh` could not list the account's repositories, or one repository's runs, usually because its sign-in expired. Run
`gh auth status`, then the script again; `--force` pushes without the check.

#### `runners-busy`

The listed runs, each with its repository, workflow, status, and URL, are queued or in progress in a repository the
account owns. A release started beside them waits for runners. Run the script again once they finish, or pass `--force`
to push now.

#### `push-failed`

`git push --atomic` failed, and git's output, which the error quotes, says why. The script deleted the tags it created,
so the remote received none and a second run starts clean. Fix what git names, then run the script again.

### What the tags start

The library's tag starts `.github/workflows/release-lib.yml` on the tag's ref, which publishes `xdk-rs` ([§ Releasing
the library](#releasing-the-library)). The binary's tag starts `.github/workflows/release.yml`, and that run, titled
`Dispatch v<version>`, builds nothing. Its one job runs `gh workflow run release.yml --ref main -f tag=v<version>` and
links the dispatched runs in its summary. The dispatched run, titled `Release v<version>`, runs on `main`, where it
restores the caches earlier releases saved; a run on a tag's ref saves into the tag's scope, which no later release
reads. It calls the reusable `brettdavies/.github/.github/workflows/rust-release.yml@main` with the tag:

| Step                  | What                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `validate-inputs`     | Refuse a tag that is not `v<major>.<minor>.<patch>`, one that does not exist, and one whose commit is not the commit the run checked out, `main`'s head, before anything builds (gate).                                                                                                                                                                                                                                                         |
| `check-version`       | Verify the tag matches the `xurl-rs` version in `crates/xurl-cli/Cargo.toml` (gate).                                                                                                                                                                                                                                                                                                                                                            |
| `audit`               | `cargo deny check` (license + advisory + ban).                                                                                                                                                                                                                                                                                                                                                                                                  |
| `build`               | Cross-compile binaries for 7 targets beside `check-version`, `audit`, and `sbom`: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`. The musl rows are required (`linux_musl_required: true`), and a failed required row cancels the others. Each archive includes binary, completions, and licenses. |
| `sbom`                | Generate a CycloneDX SBOM of `xr` from the tagged lockfile, with a read-only token.                                                                                                                                                                                                                                                                                                                                                             |
| `attest`              | Once `build`, `sbom`, `check-version`, and `audit` pass, sign what `build` uploaded, before anything is published: build provenance for all 7 archives and `sha256sum.txt`, and the SBOM against the archives. A failure here publishes nothing (gate).                                                                                                                                                                                         |
| `publish-crate`       | Wait up to ten minutes for `xdk-rs` to reach crates.io at the bound the CLI declares, so the library tag pushed beside this one publishes first, then `cargo publish -p xurl-rs --no-verify` via Trusted Publishing (OIDC, no static token). The verify compile is skipped because `ci / Package check` ran `cargo publish --dry-run --workspace` on the release PR, whose tree is the tagged tree.                                             |
| `release`             | Create a **non-draft** GitHub Release with `make_latest: false`. Includes all 7 archives + `sha256sum.txt`.                                                                                                                                                                                                                                                                                                                                     |
| `verify-attestations` | Download every published file and verify each against its attestation with `gh attestation verify --signer-workflow`, all at once. A failure withholds the Homebrew dispatch (gate).                                                                                                                                                                                                                                                            |
| `homebrew`            | Dispatch `update-formula` to `brettdavies/homebrew-tap` (formula name: `xurl-rs`, installs `xr`).                                                                                                                                                                                                                                                                                                                                               |

Each coded stop in the reusable is an error annotation titled with its code, which links the heading in
[`brettdavies/.github`'s README](https://github.com/brettdavies/.github/blob/main/README.md) that gives the recovery.
Two of them follow from how this release runs. `release-tag-moved` means `main` moved past the tag before the dispatched
run checked it out; the annotation gives the dispatch onto the tag's own ref, which releases the tagged commit and saves
no cache a later release restores. `library-wait-timeout` means `xdk-rs` did not reach crates.io within the wait; once
`release-lib.yml` is green, `gh run rerun <id> --failed` continues the release with its builds and attestations kept.

The tap's formula installs this release's archives. Its `update-formula` workflow downloads the four it names (the two
`apple-darwin` and the two `linux-musl` archives), verifies each against the attestation `attest` made, and pins its
checksum; an archive with no attestation stops the bump, so `attest: true` in `release.yml` is what lets a release reach
Homebrew. The tap then installs and tests the formula from those archives on four platforms and lands the bump on its
`main` in the same run. It builds no bottle: the binaries are compiled once, by `release.yml`, and `brew install`
downloads the archive.

After the tap lands the formula bump, it dispatches `finalize-release` back to this repo, which idempotently flips
`make_latest: true`. Its `warm-main` job then runs `gh workflow run ci.yml --ref main`: the release merge starts no CI
run, so the release has the account's runners to itself, and `main`'s CI caches fill once the release is out. That run
saves one entry shared by CI's Linux jobs, one for the Windows job, and one for each of the four reusable CI jobs.

#### When the dispatch job fails

A failed `Dispatch the release onto main` job built nothing. Its error annotation, titled `release-dispatch-failed`,
links here, and `scripts/release/postflight.sh release` fails on it. GitHub can accept a dispatch and still fail the job,
so look for the release run before you start one:

```bash
gh run list --workflow release.yml --branch main --event workflow_dispatch --limit 3
```

When no run titled `Release v<version>` is listed, dispatch it by hand:

```bash
gh workflow run release.yml --ref main -f tag=v<version>
```

→ Rationale (the dispatched run, `make_latest` flow, target matrix, annotated-tag gotcha):
[`RELEASES-RATIONALE.md` § Release pipeline](./RELEASES-RATIONALE.md#release-pipeline).

### After publish: sync `dev` with the release

Once `finalize-release.yml` has flipped the GitHub Release to `published`, bring the release bookkeeping
(`crates/xurl-cli/Cargo.toml`, `Cargo.lock`, the crate changelogs, and whatever else the release branch edited that
never round-tripped to `dev`) back to `dev` so the integration branch starts from the released baseline:

```bash
scripts/sync-dev-after-release.sh v3.0.0
```

The script cuts a `chore/sync-dev-after-v3.0.0` branch, writes the released version into `crates/xurl-cli/Cargo.toml`,
copies each crate's changelog from `main` verbatim (`crates/xurl-cli/CHANGELOG.md`, `crates/xdk/CHANGELOG.md`), adopts
the release-prep paths it discovers, refreshes every workspace member's `Cargo.lock` entry from the synced manifests
(refusing a lock that `cargo metadata --locked` rejects), and opens a PR against `dev` with the version in its title;
merge it once CI is green. `scripts/release/postflight.sh backport` gates on that merged PR. Never merge `main` into
`dev` or push to `dev` directly: the squash-merged histories share no recent ancestry, so the merge conflicts on every
file both sides touched, and a direct push bypasses `dev`'s required checks. Dev-only content (`CONCEPTS.md`, the
engineering docs) is never part of the copy, so the sync cannot remove it.

When the release moved `xdk-rs`, the same PR deletes each `required-update` entry under
`[package.metadata.cargo-semver-checks.lints]` in `crates/xdk/Cargo.toml` that the new `xdk-rs-v<version>` tag moves the
baseline past.
`cargo semver-checks check-release --package xdk-rs --baseline-rev xdk-rs-v<version> --release-type patch` names every
lint that still fires, and an entry whose lint it does not name goes: left standing, it would reclassify the next break
of the same kind. `scripts/release/preflight.sh api-contract` runs the same probe and fails while such an entry stands.

Discovery compares `origin/dev` with `origin/main` and classifies every differing path against the previous `v*` tag,
the last point the two branches agreed. A path `dev` has not touched since that tag is release-prep and is adopted. A
path `dev` has also changed since that tag is contested: the script lists it and leaves it out. `--only PATH`
(repeatable) narrows the adopted set to the paths you name, release-prep or contested, and `--include-contested` adopts
every contested path. Guarded paths (`scripts/release/guarded-paths.sh`) never enter either list. `--dry-run` prints the
contested paths and the paths it would sync, then ends on `dev` with no branch created and the tree clean. The lock
refresh runs offline against the local registry cache, so a crate missing from it stops the run with exit 70 before
anything is committed (`cargo fetch` fills the cache), and a missing `cargo` stops it with exit 69. A `dev` already in
sync exits 0 with nothing committed. After the commit, when the sync carried `crates/xurl-cli/CHANGELOG.md`, the script
re-runs `scripts/generate-changelog.py --dry-run --tag v3.0.0 --crate xurl-rs`, the arguments the cut script uses, and,
if that does not pass, warns with the generator's own reason line.

→ Rationale: [`RELEASES-RATIONALE.md` § Release pipeline](./RELEASES-RATIONALE.md#release-pipeline).

## Releasing the library

`xdk-rs` versions and tags independently of the CLI: its tags are `xdk-rs-vX.Y.Z`, its changelog is
`crates/xdk/CHANGELOG.md`, and `.github/workflows/release-lib.yml` runs the reusable
`brettdavies/.github/.github/workflows/rust-lib-release.yml`, which publishes `cargo publish -p xdk-rs` and creates a
GitHub Release (`make_latest: false`, so the binary's release stays the repository's latest) from that changelog. No
binaries, no Homebrew.

**Order is a rule.** `cargo publish -p xurl-rs` resolves `xdk-rs` from the registry, so the library reaches crates.io
before the CLI publishes. `scripts/release/tag-release.sh` pushes both tags in one push, the CLI's builds run while the
library publishes, and the CLI's `publish-crate` job waits up to ten minutes for the bound `crates/xurl-cli/Cargo.toml`
declares to reach the index. A bound still missing after the wait stops as `library-wait-timeout`, with nothing
published and the builds kept.

The repository root's `CHANGELOG.md` is prose routing to the two crate changelogs and carries no release history.
`generate-changelog.py` refuses to write a file carrying its `changelog-router` marker and names `--crate` instead, so a
run that forgets the flag stops rather than burying the router under a version section. It is the one changelog that is
hand-written, and the only one markdownlint checks.

Both crate changelogs are cut on the release branch, each from its own crate's section in the PRs merged into `dev`
since the previous release. `--from-dev-prs` is what makes that possible: the release branch is one overlay commit on
top of `main`, so it carries no per-PR history for git-cliff to read. `cut-release-branch.sh` writes the CLI's; the
library's bump and changelog are operator steps, listed in order in § Project specifics, because its next version is
read from the PRs' `## Changelog (xdk-rs)` blocks rather than passed to the script.

```bash
# 1. Every breaking entry carries a before/after snippet (the policy in crates/xdk/README.md).
#    The snippet lives in the PR body, inside the `## Changelog (xdk-rs)` block, as a
#    fenced block indented under its bullet:
#
#      ## Changelog (xdk-rs)
#
#      ### Breaking changes
#
#      - Change `Client::send_dm` to return ...
#
#        ```rust
#        // Before
#        ...
#        // After
#        ...
#        ```
#
#    Edit the PR body and regenerate; never hand-edit the changelog. A PR with no such
#    block keeps its commit subject as its bullet.

# 2. After the release PR merges, tag both crates on the merge commit in one push
#    (§ Tagging and publishing), then confirm the library's release and the pair.
git checkout main && git pull --ff-only
scripts/release/tag-release.sh
scripts/release/postflight.sh --tag xdk-rs-v0.2.0 all
scripts/release/postflight.sh --tag v1.3.0 tags
```

The library's next changelog window starts at the backport of the CLI tag on its commit: the backport's subject names
only the CLI tag, and the library tag sits on `main`, which shares no recent history with `dev`. The `tags` gate fails
when a release moves `xdk-rs` without `xdk-rs-v<version>` on the CLI tag's commit, and prints the `git tag` command.

A rehearsal runs the same pipeline without a tag: `gh workflow run release-lib.yml --ref <branch>` skips the tag check,
runs `cargo publish -p xdk-rs --dry-run`, and creates no release. A rehearsal can never publish. Locally, `cargo publish
-p xdk-rs --dry-run` runs the same packaging and verification step the pipeline's publish job runs, and `cargo package
-p xdk-rs --list` confirms the file set, which is what the `exclude` negation in `crates/xdk/Cargo.toml` exists to
control.

`scripts/release/*` and `scripts/sync-dev-after-release.sh` read the CLI's version and tag line (`RELEASE_MANIFEST`,
`crates/xurl-cli/Cargo.toml`; `v[0-9]*` tags); the library's bookkeeping is the two files above.

## Rollback

A bad release is rolled back at the registry and distribution surfaces first, then repaired in git. Rollback re-points
what users get; it does not revert history. After rolling back, land a `fix/*` or `revert` through the normal `dev` to
`release/*` to `main` flow so `main` matches what is live. Knowing the last-good identifier before the release goes out
is a [`RELEASES-POSTFLIGHT.md`](./RELEASES-POSTFLIGHT.md) gate.

```bash
PREV=v3.0.0   # the last-good tag, recorded before the release
BAD=v3.1.0    # the release being rolled back

# crates.io: yank the bad version. Existing lockfiles keep resolving it; new resolutions
# (cargo install, and cargo binstall, which follows the index) skip it.
cargo yank --version "${BAD#v}" xurl-rs

# GitHub Release: re-point /releases/latest (and cargo-binstall) at the last-good tag.
gh release edit "$PREV" --latest

# Homebrew: revert the tap's commit for the bad release (`chore(xurl-rs): bump to
# vX.Y.Z`) so `brew install` resolves the previous version, whose assets are still
# attached to the previous GitHub Release. A release the tap published with a bottle
# (4.3.0 and earlier) has a second commit, `xurl-rs: add X.Y.Z bottle.`; revert it first.
git -C ~/dev/homebrew-tap revert <bump-sha>
```

`cargo yank --undo --version <version> xurl-rs` reverses a wrong yank. A yanked version cannot be published again, so
the fix ships as the next patch version through the normal flow. A bad library release yanks `xdk-rs` the same way; its
GitHub Release is never latest and nothing installs it through Homebrew, so the yank is the whole rollback.

→ Rationale: [`RELEASES-RATIONALE.md` § Rollback](./RELEASES-RATIONALE.md#rollback).

## Prose scrubbing

Three release-flow artifacts live outside any automated prose check and need a manual scrub before they ship:

- PR bodies (`gh pr create` / `gh pr edit` send body text directly to GitHub).
- `CHANGELOG.md` (a generated artifact built from upstream PR bodies).
- Release-PR bodies (composed after `CHANGELOG.md` has been generated).

The canonical Vale + LanguageTool rule packs live in the agentnative-spec repo at
`docs/architecture/voice-enforcement.md` (local checkout at `~/dev/agentnative-spec`). This repo does not ship a local
copy; point Vale at the spec checkout via `--config`.

```bash
# 1. Save the artifact to /tmp/.
gh pr view <num> --json body --jq .body > /tmp/body.md         # for PR body edits
# cp CHANGELOG.md /tmp/body.md                                 # for changelog scrub

# 2. Vale (against the spec's rule packs).
vale --no-global --config ~/dev/agentnative-spec/.vale.ini --output=line --minAlertLevel=error /tmp/body.md

# 3. LanguageTool grammar check via lt_check (~/dotfiles/config/shell/languagetool.sh).
#    Skips cleanly if LT is unreachable. Inspect: `lt_rules`, `lt_info`.
lt_check /tmp/body.md

# 4. unslop (em-dash density and AI-unique structural patterns).
~/.claude/skills/unslop/scripts/score.py /tmp/body.md

# 5. Apply fixes per finding. Re-run until 0 blocking and unslop score is 0.

# 6. Apply the cleaned version.
gh pr edit <num> --body-file /tmp/body.md     # for PR body edits
# ./scripts/generate-changelog.py   # for CHANGELOG.md
```

For a `CHANGELOG.md` finding, fix the upstream PR body and regenerate. Hand-editing `CHANGELOG.md` directly produces
drift the next regeneration overwrites.

→ Rationale + which artifacts need this:
[`RELEASES-RATIONALE.md` § Prose scrubbing scope](./RELEASES-RATIONALE.md#prose-scrubbing-scope).

## Branch protection

Two rulesets are committed under `.github/rulesets/` and applied to the repo via the GitHub API:

- `protect-main.json` (required signatures, linear history, squash-only merges via PR with one approving review,
  creation/deletion blocked, non-fast-forward blocked, and the required status checks the file lists: every check `dev`
  requires but one, plus `ci / Changelog` and the three `guard-*` checks). The one it leaves out is `Surface growth
  needs a minor section`, because the `Changelog bump` workflow does not run on release PRs. A release tree that `dev`
  never checked as a whole, from a cherry-pick release or a fix made on the release branch, meets no weaker gate on
  `main`.
- `protect-dev.json` (required signatures, linear history, squash-only merges via PR with no approval required,
  deletion blocked, non-fast-forward blocked, and the required status checks the file lists: every `ci / ...` job of
  the reusable workflow, the repository's own CI jobs, `Agent-native audit`, `Go parity`, and `MSRV` among them, and
  `Surface growth needs a minor section` from the `Changelog bump` workflow). The file is the list; apply it after
  changing it.

The repository admin role bypasses both rulesets. That is how the owner merges a release PR without a second reviewer,
and how planning documents are committed to `dev` directly.

A job its `if:` skips on a documentation-only PR still reports under its own name, as skipped, and that satisfies the
requirement. A matrix job skipped whole never expands and reports nothing, which is why `Features (...)` runs on every
PR and skips its steps instead.

### Applying changes

```bash
# First apply (creating a ruleset):
gh api -X POST repos/brettdavies/xurl-rs/rulesets --input .github/rulesets/protect-dev.json

# Subsequent updates (replace by ID — find via `gh api repos/brettdavies/xurl-rs/rulesets`):
gh api -X PUT repos/brettdavies/xurl-rs/rulesets/<id> --input .github/rulesets/protect-main.json

# Read back what a ruleset requires; the output equals the file's list:
gh api repos/brettdavies/xurl-rs/rulesets/<id> \
  --jq '.rules[] | select(.type == "required_status_checks") | .parameters.required_status_checks[].context'
```

→ Status-check context strings (inline vs reusable):
[`RELEASES-RATIONALE.md` § Status-check context strings](./RELEASES-RATIONALE.md#status-check-context-strings).

## Project specifics

### Version carriers

`cut-release-branch.sh` leaves every version carrier alone and writes only the CLI's changelog. On the staged branch,
before the commit, in this order:

```bash
# 1. Bump the CLI. The workspace root is a virtual manifest; the version the `vX.Y.Z`
#    tag names lives in the binary crate. Use the version the cut was given.
sed -i 's/^version = ".*"/version = "1.3.0"/' crates/xurl-cli/Cargo.toml

# 2. When the release moves the library, bump it together with the CLI's declared
#    bound. The bound lives in the workspace manifest and must move with the library,
#    or the workspace stops resolving; the tag order in § Releasing the library
#    protects it. The number comes from § Versioning, read from the PRs'
#    `## Changelog (xdk-rs)` blocks.
sed -i 's/^version = ".*"/version = "0.2.0"/' crates/xdk/Cargo.toml
sed -i 's/^xdk-rs = { version = "[^"]*"/xdk-rs = { version = "0.2.0"/' Cargo.toml

# 3. Refresh the lockfile entry of each crate bumped above.
cargo update -p xurl-rs
cargo update -p xdk-rs

# 4. Regenerate the completions, which catches any subcommand or flag change missed
#    during dev.
./scripts/generate-completions.sh

# 5. When the release moves the library, generate its changelog. The member's
#    [package.metadata.changelog] table names its tag line and its paths. Each PR
#    contributes what its `## Changelog (xdk-rs)` block says, so a PR that touched
#    both crates contributes only what it addressed to the library. The dry run is
#    idempotent and answers whether the file still matches its inputs.
scripts/generate-changelog.py --crate xdk-rs --from-dev-prs --tag xdk-rs-v0.2.0
scripts/generate-changelog.py --crate xdk-rs --from-dev-prs --tag xdk-rs-v0.2.0 --dry-run

# 6. Scrub both changelogs per § Prose scrubbing. Fix findings on the PR bodies and
#    regenerate, never by hand-editing a changelog; the CLI's regenerates with the
#    command the cut ran.
scripts/generate-changelog.py --crate xurl-rs --from-dev-prs --tag v1.3.0
```

`scripts/release/preflight.sh mechanics` checks the result: the CLI's version against the top section of its changelog,
and, while a library release is pending (no `xdk-rs-v<version>` tag at the library's manifest version), the library's
version against the top section of its own.

### First-time publish (one-time)

The initial publish of a crate requires a regular crates.io API token, because Trusted Publishing is configured on a
crate's own settings page and that page does not exist until the name is claimed. Both crates have completed it:
`xurl-rs` with `v1.0.3`, and `xdk-rs` through a `0.0.0` placeholder before `xdk-rs-v0.1.0`. A crate this workspace
publishes for the first time repeats it.

**Claim the name with a placeholder version, not the version you intend to release.** The release pipeline runs `cargo
publish -p <crate>` against whatever the manifest says, and crates.io refuses a version it already holds, so a bootstrap
that publishes the real version leaves its tag with nothing to publish and fails the run. `0.0.0` reserves the name,
keeps every real version on the OIDC path, and is yanked once the first real version is up. As `xdk-rs` ran it:

1. Verify your email on crates.io (`https://crates.io/settings/profile`).
2. Publish the placeholder with `CARGO_REGISTRY_TOKEN` set, then put the tree back. The CLI's declared bound moves with
   the member version or the workspace stops resolving, and `--allow-dirty` is required because both manifests are
   modified:

   ```bash
   sed -i 's/^version = "0.1.0"/version = "0.0.0"/' crates/xdk/Cargo.toml
   sed -i 's|^xdk-rs = { version = "0.1.0"|xdk-rs = { version = "0.0.0"|' Cargo.toml
   cargo publish -p xdk-rs --allow-dirty
   git checkout crates/xdk/Cargo.toml Cargo.toml Cargo.lock
   ```

3. Configure Trusted Publishing on crates.io: `https://crates.io/settings/tokens/trusted-publishing` → add
   `brettdavies/xurl-rs`, workflow `release-lib.yml` (the CLI's entry names `release.yml`).
4. Enable "Enforce Trusted Publishing" to block token-based publishes.
5. Remove the `CARGO_REGISTRY_TOKEN` repository secret.
6. Release `0.1.0` through the pipeline as in § Releasing the library, then yank the placeholder: `cargo yank --version
   0.0.0 xdk-rs`.

Subsequent releases use the OIDC flow built into `release.yml` and `release-lib.yml`: no static token in CI.

### Required secrets

| Secret                 | Purpose                                                                                                           | Lifecycle                                         |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------------------------- |
| `CI_RELEASE_TOKEN`     | Fine-grained PAT, Contents R+W, Pull requests R+W. Used by `release.yml` to dispatch the Homebrew formula update. | Rotated annually. 1Password vault: `secrets-dev`. |
| `CARGO_REGISTRY_TOKEN` | crates.io API token. Required only for the first publish.                                                         | Removed after Trusted Publishing was enforced.    |

`GITHUB_TOKEN` is automatic; CI (`ci.yml`) needs `contents: read` and `pull-requests: read` and uses no extra secrets.

### Distribution channels

| Channel          | How                                                                              |
| ---------------- | -------------------------------------------------------------------------------- |
| Homebrew         | `brew install brettdavies/tap/xurl-rs`                                           |
| Pre-built binary | Download from [GitHub Releases](https://github.com/brettdavies/xurl-rs/releases) |
| Rust crate       | `cargo install --locked xurl-rs` (binary) or `cargo add xdk-rs` (library)        |
| Fast binary      | `cargo binstall xurl-rs`                                                         |
| From source      | `git clone && cargo build --release`                                             |

## Related docs

- [`RELEASES-PREFLIGHT.md`](./RELEASES-PREFLIGHT.md) (pre-cut go/no-go checklist gating release-branch creation)
- [`RELEASES-POSTFLIGHT.md`](./RELEASES-POSTFLIGHT.md) (post-tag verification of both publish pipelines)
- [`RELEASES-RATIONALE.md`](./RELEASES-RATIONALE.md) (release-flow rationale: branching, PR body, pipeline, prose-check)
- [`.github/pull_request_template.md`](.github/pull_request_template.md) (PR body structure with changelog sections)
- [`AGENTS.md`](AGENTS.md) (project structure, daily development)
- [`README.md`](README.md) (the router between the library and the CLI)
