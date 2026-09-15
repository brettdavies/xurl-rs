---
title: Concurrency Group Ownership in Workflow Callers - Plan
type: fix
date: 2026-09-14
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
artifact_readiness: implementation-ready
execution: code
---

# Concurrency Group Ownership in Workflow Callers - Plan

## Goal Capsule

- **Objective:** A change pushed to this repository gets a CI verdict that reflects the change. A run never reports
  failure while every job it shows is green, and a release never loses its publish to a cancellation nothing asked for.
- **Means:** Every concurrency group in this repository is keyed on a literal token naming who owns it, so no two
  declarations can land in one group by deriving the same key from caller context (KTD1).
- **Authority:** Requirements win on behavior. Key Technical Decisions win on mechanism. Units override neither.
- **Execution profile:** Configuration work with no unit-testable surface. The proof is a real run on a PR, not a test
  suite. Verify by observing the job list of an actual run, because the failure this fixes is invisible to every local
  check and to `actionlint`.
- **Stop conditions:** Stop and ask if the reusable's side has not landed first (see Dependencies), or if a run still
  loses a job after the change, which would mean the cause is not the group key.
- **Tail ownership:** Brett reviews and merges. The implementer opens the PR and stops.

---

## Product Contract

### Summary

Three of this repository's workflows declare a concurrency group. One of them, `release-matrix-check.yml`, keys it on a
literal token (`release-matrix-`). The other two key it on `${{ github.workflow }}`, which is the caller's own workflow
name and therefore the same value any other declaration would produce for the same run. `ci.yml` carries a `-caller-`
suffix added as a workaround when that collision took the whole `ci` reusable call out of a run.

This unit makes all three use the shape the correct one already uses, and adds a guard so the shape cannot regress
unnoticed.

### Problem Frame

On 2026-09-14, PR #147 added a workflow-level concurrency group to `ci.yml` using the shape the repo standard documents.
The run that followed reported **failure with six green jobs and no `ci / *` job at all**. No job failed; `gh run view
--log-failed` returned nothing. The seven jobs the `rust-ci.yml` reusable contributes never appeared.

The cause is that `rust-ci.yml` declares its own group as `${{ github.workflow }}-${{ github.ref }}`, and inside a
called workflow `github.workflow` resolves to the **caller's** workflow name. Both declarations therefore resolved to
the same string. GitHub's documented behavior is that a queued job entering a group already held in progress cancels
that in-progress work when `cancel-in-progress: true` is set. The caller's own inline jobs took the group first, so the
reusable's queue cancelled the run that invoked it.

The `-caller-` suffix currently in `ci.yml` separates the two keys and restored the run to thirteen jobs. It works, and
it is a local patch on one file for a rule that governs every workflow here.

What makes this worth fixing rather than leaving: the failure presents as a green-jobs-but-failed run with no error text
anywhere, which costs an hour to diagnose the first time and will cost it again. And the same collision on the release
path would cancel a publish rather than a re-run.

### Requirements

- **R1.** Every `concurrency.group` in this repository begins with a literal token that names the workflow owning it, so
  two declarations cannot resolve to one group by deriving the same key from caller context.
- **R2.** `ci.yml` keeps a cancelling group covering the jobs it declares itself, and drops the `-caller-` workaround.
- **R3.** A workflow whose group violates R1 fails a check before it reaches a run, locally and in CI.
- **R4.** No change to which jobs run, in what order, or with what permissions. The only behavioral change is which runs
  cancel which.

### Scope Boundaries

In scope: the three workflows here that declare a group, and the guard that keeps them honest.

Out of scope: `release.yml`, `finalize-release.yml`, the three guard callers, and `dependabot-preflight.yml`, none of
which declares a group today. R1 governs them if one is ever added; this plan does not add one. Also out of scope: the
`bird` and `agentnative-cli` callers, and the repo-standard skill text, which documents the caller-context behavior
correctly but shows an example that invites the collision.

### Dependencies

The companion plan in `brettdavies/.github` namespaces `rust-ci.yml`'s group. **That PR merges first.** Until it does,
the reusable's key is still `CI-<ref>` for this caller, and this repository's new `ci-<ref>` differs from it only by
case. GitHub does not document whether group names are compared case-insensitively, and this plan does not rely on the
answer: landing the reusable first removes the question.

---

## Planning Contract

### Key Technical Decisions

- **KTD1. A concurrency group key begins with a literal owner token.** `release-matrix-check.yml` already does this
  (`release-matrix-${{ github.ref }}`), as does `rust-release.yml` in the reusable repo (`release-${{ github.repository
  }}`). Three of the five relevant declarations across both repositories already follow it; this makes the other two
  consistent rather than inventing a convention. A literal token is what a caller and a callee cannot both produce by
  accident, because neither can derive it from context.

  Rejected: keeping `${{ github.workflow }}` and relying on the reusable's namespace alone. That leaves this
  repository's safety dependent on a file in another repository, and the failure mode returns silently if that file
  changes.

  Rejected: dropping the caller group entirely and letting the reusable's group cover everything. The reusable's group
  covers only the jobs the reusable defines. `ci.yml` declares five jobs of its own, which would then have no
  cancellation at all.

- **KTD2. The guard is a grep over the workflow files, not a schema.** The rule is one line: the value after `group:`
  starts with a character other than `$`. `actionlint` does not model concurrency semantics and will not learn to. A
  four-line script run by the existing hook pair and by CI costs nothing and fails loudly on the one shape that matters.

- **KTD3. `spec-drift.yml` is fixed even though it cannot collide today.** It calls no reusable, so its caller-context
  key is currently harmless. It is fixed because R1 is a property of the repository rather than of the workflows that
  happen to be at risk, and because a future edit that adds a reusable call to it would reintroduce the failure with no
  signal.

### Assumptions

- Concurrency groups are scoped per repository, so a literal token only needs to be unique within this repo.
- The `ci-` token is unused elsewhere here; `rg 'group:' .github/workflows` confirms it at plan time.

---

## Implementation Units

### U1. Literal owner tokens on all three groups

- **Goal:** No group key in this repository is derived solely from caller context.
- **Requirements:** R1, R2, R4.
- **Dependencies:** The `brettdavies/.github` PR merges first.
- **Files:** `.github/workflows/ci.yml`, `.github/workflows/spec-drift.yml`.
- **Approach:**
  1. `ci.yml`: `group: ci-${{ github.ref }}`, replacing `${{ github.workflow }}-caller-${{ github.ref }}`. Keep
     `cancel-in-progress: true`. Replace the comment explaining the suffix with one stating the rule: the token names
     the owner, and the reusable this workflow calls carries its own.
  2. `spec-drift.yml`: `group: spec-drift-${{ github.ref }}`, keeping `cancel-in-progress: true`.
  3. `release-matrix-check.yml` is already conformant and is not touched.
- **Patterns to follow:** `release-matrix-check.yml`'s existing declaration.
- **Test scenarios:**
  - Test expectation: none. Configuration only; the proof is the run, below.
- **Verification:** `actionlint` clean. The PR's own run lists all thirteen checks, including all seven `ci / *` jobs.
  Push a second commit while the first run is in flight and confirm the superseded run cancels and the new one
  completes, which is the behavior the group exists for.

### U2. A guard that fails a context-derived group key

- **Goal:** R1 is enforced rather than remembered.
- **Requirements:** R3.
- **Dependencies:** U1 lands first, so the guard starts green.
- **Files:** `scripts/lint-workflow-concurrency.sh` (create), `scripts/hooks/_lib.sh`, `scripts/hooks/pre-commit`,
  `scripts/hooks/pre-push`, `.github/workflows/ci.yml`.
- **Approach:**
  1. The script reads every `.github/workflows/*.yml`, finds each `group:` line under a `concurrency:` block, and fails
     when the value begins with `$`. The failure message names the file and states the rule and the fix, since the
     person who hits it will not know why the rule exists.
  2. A `check_workflow_concurrency` wrapper in `_lib.sh`, following the existing `check_*` shape: takes a file list,
     skips cleanly on an empty list, returns the script's status.
  3. `pre-commit` calls it with the staged workflow files; `pre-push` calls it when the push carries a workflow, beside
     the existing `actionlint` step.
  4. A step in the existing `Output discipline` job runs it in CI, which is where a change that bypasses the hooks is
     caught.
- **Patterns to follow:** `scripts/lint-stdio.sh` for the script shape and failure message; `check_actionlint` in
  `scripts/hooks/_lib.sh` for the wrapper.
- **Test scenarios:**
  - A workflow whose group is `${{ github.workflow }}-${{ github.ref }}`: the script exits non-zero and names the file.
  - A workflow whose group is `ci-${{ github.ref }}`: the script exits zero.
  - A workflow with no `concurrency:` block: the script exits zero.
  - `pre-commit` with a conforming workflow staged: passes. With a violating one staged: fails and names the file.
  - The script observed failing against the pre-U1 shape before it is wired into CI, so the guard is known to bite.
- **Verification:** `shellcheck --severity=warning` clean on the new script and the changed hooks. The `Output
  discipline` job passes on the PR.

---

## Verification Contract

| Gate              | Command                                                                              | Expected                                        |
| ----------------- | ------------------------------------------------------------------------------------ | ----------------------------------------------- |
| Workflow lint     | `actionlint`                                                                         | Clean                                           |
| Shell lint        | `shellcheck --severity=warning scripts/lint-workflow-concurrency.sh scripts/hooks/*` | Clean                                           |
| Concurrency guard | `bash scripts/lint-workflow-concurrency.sh`                                          | Exits zero, after U1                            |
| Guard bites       | The same script against the pre-U1 `ci.yml` shape                                    | Exits non-zero, naming the file                 |
| Real run          | `gh pr view <n> --json statusCheckRollup`                                            | Thirteen checks, all seven `ci / *` present     |
| Cancellation      | Push twice in quick succession                                                       | The superseded run cancels; the newer completes |

The run check is the one that matters. Every local gate passed while the collision was live, and the only signal was a
job that failed to appear.

---

## Definition of Done

- All three group keys begin with a literal owner token; none contains `${{ github.workflow }}` as its leading element.
- `ci.yml` carries no `-caller-` suffix and no comment referring to one.
- The guard fails on a context-derived key and passes on the repository as it stands, in the hooks and in CI.
- A real PR run lists all thirteen checks with all seven `ci / *` jobs present.
- The PR body states the merge-order dependency on the `brettdavies/.github` PR.
- No experimental or abandoned code remains in the diff.

---

## Sources

- The failed run: repository Actions run `34910294817`, six green jobs, no `ci / *` job, conclusion `failure`.
- The fixed run: `34910560339`, thirteen jobs, conclusion `success`.
- GitHub concurrency semantics: a queued job entering a group already held in progress is cancelled or cancels, per
  `cancel-in-progress`.
- In-repo precedent: `.github/workflows/release-matrix-check.yml`.
- Cross-repo precedent: `rust-release.yml` in `brettdavies/.github`.
