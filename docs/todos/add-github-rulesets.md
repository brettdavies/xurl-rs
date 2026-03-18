---
status: completed
priority: p2
issue_id: ""
tags: [github-rulesets, branch-protection, ci]
dependencies: []
---

# Add GitHub Rulesets for Branch Protection

## Problem Statement

The xurl-rs repo has ruleset JSON files checked in at
`.github/rulesets/` but they are NOT applied to GitHub. The
`gh api repos/brettdavies/xurl-rs/rulesets` endpoint returns an
empty array. The JSON files exist as documentation but have never
been activated. The homebrew-tap repo has working rulesets that
should be used as the reference for a complete configuration.

## Findings

- `.github/rulesets/protect-main.json` exists locally but only has
  `pull_request` and `required_signatures` rules — missing several
  protections that homebrew-tap includes
- `.github/rulesets/protect-dev.json` exists locally but only has
  `required_signatures` — minimal protection
- `gh api repos/brettdavies/xurl-rs/rulesets` returns empty —
  neither ruleset is applied to GitHub
- The existing JSON files are incomplete compared to the
  homebrew-tap reference (missing `creation`, `deletion`,
  `non_fast_forward`, `required_status_checks`,
  `required_linear_history`)
- The `update` rule should NOT be included — it causes a false
  `mergeStateStatus: BLOCKED` in the GitHub UI for bypass actors.
  See `~/dev/homebrew-tap/docs/solutions/workflow-issues/github-ruleset-merge-state-blocked-bypass-actors-20260318.md`

## Proposed Solutions

### Option 1: Update and apply rulesets (recommended)

**Approach:** Update the existing JSON files to match the
homebrew-tap reference (with appropriate status check names for
xurl-rs CI), then apply both via the GitHub API.

**Reference files:**

- `~/dev/homebrew-tap/.github/rulesets/protect-main.json` — main
  branch ruleset (squash-only, required status checks, signed
  commits, linear history)
- `~/dev/homebrew-tap/.github/rulesets/protect-dev.json` — dev
  branch ruleset (deletion protection, force-push protection,
  signed commits)

**Current xurl-rs `protect-main.json` has:**

- `pull_request` (squash-only, 0 required reviews)
- `required_signatures`

**Missing rules to add for `protect-main`:**

- `creation` — prevent branch recreation
- `deletion` — prevent branch deletion
- `non_fast_forward` — block force pushes
- `required_status_checks` — require CI to pass (determine correct
  context names from xurl-rs CI workflow)
- `required_linear_history` — enforce linear commit history

**Missing rules to add for `protect-dev`:**

- `deletion` — prevent branch deletion
- `non_fast_forward` — block force pushes

**Rules to NOT include:**

- `update` — causes false BLOCKED status in GitHub UI/API for
  bypass actors; redundant when `pull_request` and
  `non_fast_forward` are present

**Bypass actors (already correct in existing files):**

```json
"bypass_actors": [
  {
    "actor_id": 5,
    "actor_type": "RepositoryRole",
    "bypass_mode": "always"
  }
]
```

Note: `Integration` actor bypass is NOT supported on personal
repos (org-only). Use `RepositoryRole` with `actor_id: 5` (admin).

**Pros:**

- Builds on existing JSON files
- Brings xurl-rs to parity with homebrew-tap
- Consistent branch protection across all repos

**Cons:**

- Need to determine xurl-rs CI status check context names first

**Effort:** 30 minutes

**Risk:** Low

## Recommended Action

Update both JSON files to match homebrew-tap's rule set (minus the
`update` rule), determine correct CI status check context names,
then apply via `gh api repos/brettdavies/xurl-rs/rulesets -X POST --input <file>`.

## Technical Details

**Affected files:**

- `.github/rulesets/protect-main.json` (update existing)
- `.github/rulesets/protect-dev.json` (update existing)

**Reference files (homebrew-tap):**

- `~/dev/homebrew-tap/.github/rulesets/protect-main.json`
- `~/dev/homebrew-tap/.github/rulesets/protect-dev.json`

**Key considerations:**

- The existing JSON files have the correct bypass actor config
  (`actor_id: 5, RepositoryRole`) — preserve this
- Status check context names must match the `name:` field of
  GitHub Actions jobs in xurl-rs CI workflow
- The `strict_required_status_checks_policy: true` setting
  requires the branch to be up to date before merging
- Apply via POST (creating new rulesets), not PUT — no existing
  rulesets to update on GitHub
- Rulesets are applied via the GitHub API; JSON files serve as
  source-of-truth documentation

## Resources

- **Reference implementation:** `~/dev/homebrew-tap/.github/rulesets/`
- **Known issue (update rule):**
  `~/dev/homebrew-tap/docs/solutions/workflow-issues/github-ruleset-merge-state-blocked-bypass-actors-20260318.md`
- **Release branch pattern:**
  `~/dev/homebrew-tap/docs/solutions/workflow-issues/release-branch-pattern-for-guarded-docs-20260317.md`

## Acceptance Criteria

- [x] `.github/rulesets/protect-main.json` updated with full rule
      set (no `update` rule)
- [x] `.github/rulesets/protect-dev.json` updated with full rule set
- [x] Both rulesets applied to GitHub via API (POST)
- [x] `gh api repos/brettdavies/xurl-rs/rulesets` returns both
      rulesets
- [ ] Verified: PR to main shows green merge button when checks pass
      (will confirm on next PR)
- [x] Verified: force push to main/dev is rejected
      (`non_fast_forward` rule active; admin bypass is intentional,
      matching homebrew-tap)

## Work Log

### 2026-03-18 - Initial Discovery

**By:** Claude Code

**Actions:**

- Found existing JSON files are not applied to GitHub (API returns
  empty)
- Compared xurl-rs rulesets against homebrew-tap reference —
  identified missing rules
- Documented the `update` rule exclusion based on today's finding

**Learnings:**

- JSON files in `.github/rulesets/` are NOT auto-applied — they
  must be pushed via the GitHub API
- The `update` rule causes `mergeStateStatus: BLOCKED` false alarm
- `Integration` bypass is org-only; use `RepositoryRole` for
  personal repos
- xurl-rs protect-dev.json has no bypass actors (empty array) —
  evaluate whether this is intentional

## Notes

- Do NOT include the `update` rule — it's redundant and causes
  a misleading BLOCKED status in the GitHub UI
- The existing `protect-dev.json` has an empty `bypass_actors`
  array — decide whether the admin should have bypass on dev
- Coordinate with bird to apply consistent rulesets across all
  three repos
