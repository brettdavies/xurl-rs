# Releases rationale

Companion to [`RELEASES.md`](./RELEASES.md). RELEASES.md is the runbook (commands, paths, decision tables). This file
holds the WHY behind those rules: branching model, PR conventions, release pipeline, CHANGELOG generation, prose-check
pipeline, branch-protection pitfalls.

Read this when:

- A rule in RELEASES.md doesn't make sense and you're tempted to change it.
- A new contributor asks "why do we do X this way".
- You're adding a new release-flow rule and need to know where it fits the existing model.

## Branching model

### Forever `dev`, ephemeral release branches

`dev` is never deleted, even after a release. The next release cycle reuses the same `dev`. The repo's
`deleteBranchOnMerge: true` setting doesn't touch `dev` as long as `dev` is never the head of a PR. Using a short-lived
`release/*` head is what keeps the setting compatible with a forever integration branch.

Engineering docs (`docs/plans/`, `docs/solutions/`, `docs/brainstorms/`, `docs/reviews/`) live on `dev` only. They never
reach `main`. `guard-main-docs.yml` blocks them from PRs targeting `main`, and `guard-release-branch.yml` rejects any PR
to main whose head isn't `release/*`.

### Why the release branch is cut from `main`, never from `dev`

Every release squash-merges into `main`, so `dev` and `main` diverge in history even as their content converges: after
the first release they share only an ancient merge-base. Cutting the release branch from `dev` (or merging `dev` into
`main`) forces a 3-way merge across that divergence: `add/add` collisions on files both sides changed, plus
rename/delete pairs git cannot auto-resolve. The conflict pile is an artifact of the lineage, not of the content
shipping.

Always cut the release branch from `origin/main` and bring `dev`'s content onto it as a forward diff, never by
reconciling histories. The default is the whole-tree overlay (`git checkout origin/dev -- .`, then strip the guarded
set): `main` ships `dev`'s tree minus a small, known exclusion set, so asserting that end-state directly is simpler and
safer than hand-resolving a merge. The overlay commit carries no per-PR history, so the changelog is built from the PRs
merged into `dev` since the previous release (`generate-changelog.py --from-dev-prs`) rather than from the branch's
commits; the result is the same per-PR section a cherry-picked branch would yield. Cherry-picking the dev squash-commits
is kept only as an exception for a repo with a stated reason it cannot overlay, at the cost of guarded-path conflict
handling.

Either way, the release must start from a `main` that `dev` fully contains. Security PRs, hotfixes, and config edits
land on `main` first, and both constructions take `dev`'s content for the files they touch, so anything `main` holds
that `dev` never received is reverted by the release. `scripts/release/drift.sh` lists that set and the cut waits until
it is empty.

### Version branch naming

Branch naming `release/v<version>` or `release/v<version>-<slug>` (e.g. `release/v1.0.5-ci-migration`,
`release/v1.2.0-library-ergonomics`) makes release branches sortable and unambiguous when multiple cuts are in flight.
The `v<version>` prefix is required: `generate-changelog.py` extracts the version from the branch name. Slug is
kebab-case, short, descriptive.

## PR body conventions

### No explainer prose in the body

Every section of a PR body is user-facing substance only: the **net diff**, what is changing for the consumer that was
not already there, not the commit history or intermediate state that produced it. Workflow mechanics (cherry-pick,
regenerate, pre-push gate, CI behavior) are documented in RELEASES.md and `.github/`, NOT in the PR body. Triple-diff
output ("A: 12 files, B: none, C: clean"), leak-check narration ("`guard-main-docs` runs clean", "no guarded paths
leaked"), patch-id cherry-check counts, pre-push gate results, CI check status, exclusion rationale, and other
verification artifacts stay local; anomalies get fixed before push, not audit-trailed in the body.

The PR body is read by humans reviewing what shipped. Workflow mechanics and tool-fix provenance are noise from that
perspective; they belong in this file, the script outputs, and the commit history respectively.

### Why `feat`/`fix` are preferred over `chore`

`cliff.toml` drops commits whose subject starts with `chore`, `style`, `test`, `ci`, or `build`, unless the subject
carries `!` or the body carries `BREAKING CHANGE:`, which route to the Breaking group first. Mistyping a user-facing
change as `chore` silently strips it from release notes. Prefer `feat` / `fix` when the change has any user-observable
effect (config defaults, env vars, default behaviors, new shortcut commands, OAuth flow changes).

Security advisory bumps in particular use `fix(deps):`, never `chore(deps):`, so they appear in the changelog. A bumped
dependency that closes a CVE is user-visible value, not internal tooling.

### Why required-when-empty sub-headers

`Related Issues/Stories` has four labels (`Story:` / `Issue:` / `Architecture:` / `Related PRs:`). `Files Modified` has
four sub-headers (`Modified` / `Created` / `Renamed` / `Deleted`). All four must appear in every PR, even when empty:
write `- None.` or `n/a` rather than deleting the label. Reason: scanners and humans both rely on a known section shape.
Conditionally-absent sections force every reader to mentally check "did the author skip this or does it not apply?"

### Why no AI attribution

`Co-Authored-By: Claude …`, robot emoji / "Generated with Claude Code" trailers, or any similar AI-attribution trailer
is banned from commit messages and PR bodies. Commits and PRs stand on their own technical content. Attribution trailers
are noise and they age poorly as tools shift.

### Why no hard line wraps

Author each paragraph and each bullet as one logical line, however long. GitHub soft-wraps for display. Hard wraps
within prose produce visible mid-sentence breaks in some renderers and interfere with the prose-check pipeline: Vale's
line-anchored output reports findings against split lines, and LanguageTool's input handling can choke on certain
control-char interactions. The auto-format hook skips `/tmp/` paths so the body keeps its authored shape; don't undo
that with manual wrapping during composition. Same rule applies to commit messages composed via heredoc.

### Why release-PR bodies repeat changelog entries from upstream PRs

The release PR carries the same `### Added` / `### Changed` / `### Fixed` / `### Documentation` bullets as the feature
PRs it cherry-picks. The repetition is intentional and harmless: `cliff.toml` already skips its own changelog-update
commit, so the release-PR squash commit can't be double-counted in any future regeneration.

### Why internal-tooling commits don't appear in `## Changelog`

`chore(cliff): ...`, `chore(ci): ...`, and similar internal-tooling commits don't appear in the PR body's `##
Changelog`. They are not user-facing. They belong in commit history and in the Files Modified section of the PR body,
not in the source-of-truth release notes.

## Versioning

### Why the contract is the machine surface, not the source

SemVer's first rule is that software using it "MUST declare a public API". A CLI's public API is what a caller types and
what a program reads back, not its source. [clig.dev](https://clig.dev/#future-proofing) lists "Subcommands, arguments,
flags, configuration files, environment variables" as interfaces a CLI commits to, and says "Changing output for humans
is usually OK" as long as scripts are steered to `--plain` or `--json`. That is this repository's split between text and
structured output: text is for a person and may improve release to release, while the JSON envelope's `reason`,
`exit_code`, and `next_step` are what an agent branches on.

### Why a patch carries nothing new

SemVer reserves a patch for "only backward compatible bug fixes", and states: "A bug fix is defined as an internal
change that fixes incorrect behavior." It makes the minor mandatory for "new, backward compatible functionality" and for
anything "marked as deprecated". A new command, field, or `reason` is new functionality whatever its size, so it cannot
ride in a patch.

### Why restoring the documented contract is a patch

A fix can change an exit code or a JSON key and still be a patch, because the value it replaces was the incorrect
behavior: exit `0` for a mistyped command, or a legacy key name the published schema does not declare. HashiCorp's
[provider versioning guidance](https://developer.hashicorp.com/terraform/plugin/best-practices/versioning), written for
tools that wrap a remote API, lists "Fixing attributes to match behavior with the remote API" as a patch. SemVer's FAQ
leaves the harder case to judgment: when a large audience depends on the incorrect behavior, a major can be the kinder
release even though the fix is strictly a patch.

### Why X's churn lands as minors

X changes its API often, mostly additively, and without version numbers. HashiCorp draws the same line for providers:
adding a new resource or data source, aliasing an existing one, and marking an attribute as deprecated are minors, while
renaming or removing an attribute is a major. Reading both spellings of a renamed field keeps an upstream rename from
becoming a break for `xr`'s callers, so majors stay reserved for this project's own deliberate breaks. As clig.dev puts
it, "if you're putting out a major version bump every month, it's meaningless."

### Why a closed set can grow in a minor

`reason` and `next_step.action` are closed so an agent can branch on them without parsing prose. Growth stays backward
compatible only when a consumer treats an unrecognized value as its default branch, which is the same contract
`#[non_exhaustive]` gives a Rust enum. Removing or renaming a member breaks a consumer that matched on it, so that
remains a major.

### Why `xdk-rs` breaks in the middle position before 1.0

Cargo's [SemVer guide](https://doc.rust-lang.org/cargo/reference/semver.html) treats `0.y.z` changes to `y` as major and
changes to `z` as minor, and its default requirements treat versions as compatible when "their left-most non-zero
major/minor/patch component is the same". A dependent on `^0.1.0` therefore takes `0.1.1` automatically and never
`0.2.0`, so a break must move `y` and anything additive or a fix moves `z`.

### Why the changelog section decides the version

The person who wrote the change classifies it once, when the PR is reviewed, and the release reads that classification
instead of re-deriving it from commit subjects. Two gates catch a mis-filed break: `cargo semver-checks` for the
library's API, and the preflight's command-surface and `xr schema` diffs against the last tag for the CLI. An addition
filed under a patch section is the mistake review misses most easily, because nothing in the diff reads as wrong. So the
`Changelog bump` check compares each PR's generated surface against its base and fails the PR while the section is still
cheap to fix, instead of leaving the question to whoever cuts the release.

## Triple-diff verification

The overlay recipe screens the staged release tree twice before the commit (A: release→dev for paths outside the guarded
set and the version files, B: no guarded path in release→main) and enumerates what the release adds (D). The cherry-pick
exception runs three diffs (A: main→release, B: release→dev for paths outside the guarded set, C: dev→main) plus a
patch-id cherry check. This is belt-and-suspenders because missed cherry-picks have shipped to `main` on this and
sibling repos before, and the file-level diff in B alone doesn't catch the patch-id false-negative class.

B excludes only the guarded set, not all of `docs/`. `docs/migrating/` ships to `main`, so a wholesale `docs/` exclusion
hides a missed migration-guide pick.

### Why the guarded set resolves from the workflow

`guard-main-docs` is what CI enforces on a PR to `main`: the reusable workflow's hardcoded base list plus this repo's
`extra_paths`. Every hand-kept copy of that union (runbook, checklist, preflight script) drifted from it, and a copy
that omits a guarded path reports a real leak as clean while CI turns red after the push.
`scripts/release/guarded-paths.sh` reads `extra_paths` out of the caller workflow and adds the base list, so registering
a path in the workflow is the only edit a new guarded path needs. The base list is the one copy that still needs a
manual edit when the reusable changes, because it lives in another repo. Entries are globs with one rule set shared by
the reusable and the script (`**/` any depth, `*` and `?` within a segment, trailing slash guards the subtree), so
`**/.agent/` guards that directory wherever it appears and the two never disagree about what is guarded.

### Why the release enumerates what it adds

The leak check screens the diff against the registered set, so it says nothing about a category nobody registered. A new
engineering directory or a stray note under `docs/` passes the local check and `guard-main-docs` alike. Step D lists
every `docs/` file and every markdown file the release adds to `main` outside the guarded set and puts them in front of
a human; each one needs a reason to ship, or it gets registered in `extra_paths` and dropped from the branch. Root-level
markdown is in scope because an agent-facing glossary at the repo root is exactly the kind of addition a `docs/`-only
listing misses.

### Why patch-id cherry-check output is noisy

In a squash-merge workflow, `git cherry HEAD origin/dev` produces many `+` lines that need human triage. They do NOT
auto-block the release. Expected sources of false positives:

1. **Historical commits squash-merged in prior releases.** The squash commit on main has a different patch-id than the
   dev commits it consolidates, so old commits show as `+` forever. Anything older than the previous release tag is
   almost always this.
2. **Cherry-picks where conflict resolution stripped guarded paths** (`docs/plans/`, `docs/brainstorms/`, etc.) or
   otherwise altered the tree. Same source-code intent, different patch-id.
3. **Intentionally skipped commits** (docs-only commits, release-prep backports, revert-and-redo prep steps).

A real miss looks like: a recent feat/fix/chore commit on dev whose *file content* is not yet on main. To triage a `+`
line:

```bash
git show <sha> --stat                       # what did it touch?
git diff origin/main..HEAD -- <those-files> # already on release?
```

If every touched file is guarded (`docs/plans/`, `docs/brainstorms/`, etc.) OR the content is already on main via a
prior squash, it's a false positive (no action). Otherwise cherry-pick the commit and re-run the triple-diff.

## CHANGELOG generation

### Generated, never hand-written

`scripts/generate-changelog.py` (vendored from the `github-repo-setup` skill, with the repo-local `cliff.toml`) is the
only sanctioned way to update `CHANGELOG.md`. On an overlay-built release branch it runs as `--from-dev-prs`: the PRs
merged into `dev` since the previous release are the entries, and each PR's body supplies its `## Changelog → ###
Breaking changes / Added / Changed / Fixed / Documentation` subsections (with author and PR-link attribution). On a
cherry-picked branch it runs `git-cliff` first to prepend a versioned entry from the branch's commits, then expands the
same way.

If a PR's body offers no changelog section at all, its title becomes a `Changed` bullet, except for `chore`, `ci`,
`build`, `style`, and `test` PRs, which stay out unless they carry a `## Changelog` of their own. A body that carries
the `## Changelog` heading and leaves it empty is the template's way of saying the PR ships nothing user-facing, and the
title fallback respects that: internal work also lands as `fix(ci)`, `fix(hooks)`, and `fix(release)`, which the type
prefixes above do not cover. To fix a wrong CHANGELOG entry, fix the input: edit the squash-merged PR body, then re-run
the script. Do **not** edit `CHANGELOG.md` directly.

CI enforces that `CHANGELOG.md` is modified in every PR to main (`ci / Changelog` required status check) and that it
contains a versioned section, not `[Unreleased]`. The release workflow extracts the latest section for the GitHub
Release body.

### Why `cliff.toml` skips chore/style/test/ci/build

These commit types do not produce user-facing content. If a cherry-picked PR has user-facing `## Changelog` content but
its commit subject starts with one of those types, its bullets get silently dropped. After running the script,
cross-check the generated section against `gh pr view <num> --json body` for each cherry-picked PR; correct mistyped PR
titles (e.g. `chore` → `feat`) and re-amend the cherry-pick subject before re-running. See § Why `feat`/`fix` are
preferred over `chore` above for prevention.

## Release pipeline

### Annotated tags + Trusted Publishing

Always use annotated tags (`-a -m`). Bare `git tag <name>` silently fails with `fatal: no tag message?` on machines
where `tag.gpgsign=true` is set globally (a brettdavies dotfile default). See
[solutions: git tag fails with tag.gpgsign, use annotated tags](https://github.com/brettdavies/solutions-docs/blob/main/best-practices/git-tag-fails-with-tag-gpgsign-use-annotated-tags-2026-04-13.md).

Subsequent releases use the OIDC Trusted Publishing flow built into `release.yml`: no static token in CI. The initial
publish (`v1.0.3` for this repo) required a regular crates.io API token because Trusted Publishing needs the crate to
exist first; that token has since been removed.

### Why `make_latest: false` then `finalize-release`

The GitHub Release is created visible-but-not-latest (`make_latest: false`) so `cargo-binstall` and `/releases/latest`
don't 404 during the bottle-build window, but the release isn't yet promoted to "Latest" while bottles upload. After the
homebrew-tap workflow uploads bottles to this repo's release assets, it dispatches `finalize-release` back to this repo,
which idempotently flips `make_latest: true`. End result: crate on crates.io, GitHub Release marked latest, Homebrew
formula updated with bottles, all atomically advertised.

### Why backport `main` → `dev` after publish

Once `finalize-release.yml` has flipped the GitHub Release to `published`, the release-bookkeeping files on `main`
(`Cargo.toml` version, `Cargo.lock`, `CHANGELOG.md`) need to reach `dev` so future builds from `dev` report the released
version and so the next dev work starts from the released baseline.

The backport is a PR opened by `scripts/sync-dev-after-release.sh`, never a merge of `main` into `dev` and never a
direct push. The squash-merged branches share no recent history, so a merge conflicts on every file both sides touched,
and a direct push to `dev` bypasses its required status checks. The script writes the released version into the binary
crate's `Cargo.toml`, copies each crate's changelog from `main`, adopts the other paths the release branch edited,
refreshes every workspace member's `Cargo.lock` entry, and opens the PR. The lock is refreshed rather than copied from
`main`: a release can move the library beside the binary, so more than one member's entry changes, and copying `main`'s
lock would revert dependency updates `dev` merged after the release. The script refuses to commit a lock that `cargo
build --locked` rejects. The postflight backport gate treats that merged PR as the durable signal that the backport ran.
Dev-only content (`CONCEPTS.md`, the engineering docs) is never part of the copy, so the backport cannot remove it.

The other paths are discovered rather than listed. A release branch is edited for reasons nobody predicts (a doc fix, a
reverted payload, a deleted config), and a fixed list misses each such edit silently until the next release's overlay
restores `dev`'s copy over it. Discovery is bounded by the previous `v*` tag, the last point the branches agreed: a path
`dev` left alone since then is release-prep and adopted, and a path `dev` also changed is contested and withheld unless
the operator names it, so widening the copy cannot revert `dev`'s unreleased work.

### Rollback

Rollback happens at the surface users consume (crates.io, the GitHub Release, the Homebrew formula), not in git. Yanking
a version, re-pointing `releases/latest`, or reverting a formula bump is fast and reversible; rewriting `main` is
neither, and the release flow exists so that `main` only ever moves forward through a PR. After the rollback, the fix or
revert lands through `dev`, a release branch, and `main` like any other change, so the branch reconverges with what is
live. Recording the last-good identifiers before the release is what makes the rollback a single command under incident
pressure.

### Cross-compile target matrix

Seven targets: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`,
`aarch64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`. xurl-rs is a
network CLI against a hosted API (X / Twitter); the binaries are consumed by interactive shell users on the three
desktop platforms and on Linux CI runners, including Alpine and other glibc-free hosts, which the musl rows serve.
`release.yml` passes `linux_musl_required: true`, so a musl build failure blocks the release, and
`release-matrix-check.yml` builds the same seven rows on every push to a `release/*` branch so a broken row surfaces
before the tag.

The Windows row uses `x86_64-pc-windows-msvc` (MSVC ABI) rather than GNU because the X API uses TLS with a vendored
rustls and rustls-platform-verifier; the MSVC ABI is the path of least resistance for that stack on Windows. CI also
runs a separate `ci / Windows check` job on every PR (not just at release) so MSVC build failures surface immediately,
not at tag time.

## Prose scrubbing scope

Three release-flow artifacts live outside any automated prose check and need a manual scrub before they ship:

- **PR bodies.** `gh pr create` and `gh pr edit` send body text directly to GitHub; no automated prose check has reach
  there.
- **`CHANGELOG.md`.** A generated artifact built from upstream PR bodies; it inherits whatever prose those PR bodies
  carry, so scrubbing happens at generation time on the release branch.
- **Release-PR bodies.** The `release/v<version>` PR to `main` carries contributor-authored wrap-up text composed after
  `CHANGELOG.md` has been generated, and the same out-of-repo gap applies.

The canonical Vale + LanguageTool rule packs and orchestrator behavior live in the agentnative-spec repo at
`docs/architecture/voice-enforcement.md` (local checkout at `~/dev/agentnative-spec`). This repo does not vendor a local
copy of those packs; point Vale at the spec checkout via `--config`.

Scrub-before-submit (author in `/tmp/`, scrub there, submit via `--body-file`) avoids the round-trip of "submit, scrub,
edit, scrub again". Every fix lands locally and the public PR sees only clean text. The auto-format hook skips `/tmp/`
paths so the body keeps its authored shape and no soft-wrapping is injected.

For a `CHANGELOG.md` finding, fix the upstream PR body (which `generate-changelog.py` re-fetches every run) and
regenerate. Hand-editing `CHANGELOG.md` directly produces drift the next regeneration overwrites.

## Branch protection

### Status-check context strings

The `required_status_checks[].context` strings in `protect-main.json` MUST match exactly what GitHub publishes for each
check:

- **Inline job** (with `name:` field): published as just `<job-name>` (no workflow-name prefix).
- **Reusable-workflow caller** (`uses: .../foo.yml@ref`): published as `<caller-job-id> / <reusable-job-id-or-name>`.

Mixing these produces a stuck-but-green PR: all actual checks report green, but the ruleset waits forever on a context
that will never appear. Confirm the real contexts after a first CI run with:

```bash
gh api repos/brettdavies/xurl-rs/commits/<sha>/check-runs --jq '.check_runs[].name'
```

### Why rulesets live in-repo

Committing the JSON alongside code means ruleset changes land via the same review process as workflow changes. A
`chore(ci): tighten protect-main` change goes through dev → release/* → main like anything else.

## Related docs

- [`RELEASES.md`](./RELEASES.md) (operational runbook: commands, paths, decision tables)
- [`RELEASES-PREFLIGHT.md`](./RELEASES-PREFLIGHT.md) (pre-cut checklist gating the release-branch cut)
- [`README.md`](README.md) (the router between the library and the CLI)
- [`.github/pull_request_template.md`](.github/pull_request_template.md) (PR body structure with changelog sections)
