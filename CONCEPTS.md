# Concepts

Shared domain vocabulary for this project — entities, named processes, and status concepts with project-specific
meaning. Seeded with core domain vocabulary, then accretes as ce-compound and ce-compound-refresh process learnings;
direct edits are fine. Glossary only, not a spec or catch-all.

## Spec automation

### Vendored spec

The checked-in snapshot of X's OpenAPI document that every spec consumer in the repo reads — build-time code generation,
validation gates, and drift comparison alike. It is stored in canonical form (stable key and set ordering) so that byte
equality means semantic equality; the refresh process is the only place the live upstream document is fetched.

### Spec drift

Divergence between the vendored spec and the live upstream document. Detected by comparing the canonical forms of both
sides; raw-byte comparison is meaningless because upstream serializes set-valued arrays in nondeterministic per-request
order. Drift is informational, never build-blocking: it triggers the refresh process rather than failing anything.

### Refresh PR

The automation-owned draft pull request that carries a new vendored spec revision toward the integration branch. Its
head branch belongs to the workflow: the branch is regenerated rather than manually maintained, and at most one refresh
PR is open at a time — a human-owned refresh PR takes precedence and the automation stands down.

The body starts workflow-owned (an agent runbook, rewritten with fresh drift detail whenever the vendored content
changes) and becomes agent-owned once the reconciling agent rewrites it to the repository's PR template; a no-change
pass updates only the run reference so agent-authored sections survive.

### Agent runbook

The generated body of a fresh refresh PR: hardcoded reconciliation instructions addressed to whichever agent works the
PR. It assigns the role explicitly (executing the reconciliation is the deliverable, not a review), orders the steps,
and splits decision authority — spec-mandated changes are implemented and logged as breaking changes, while judgment
calls are proposed with a recommendation and precedent-based reasoning for a human to resolve.

### Reconciling agent

The agent invoked against an open refresh PR to bring the codebase in line with the new vendored spec: renames the spec
mandates, retirements the spec forces, derived-artifact regeneration, and the final rewrite of the PR body to the
repository template. Invocation phrasing matters: a review-shaped prompt routes agents into report-only behavior before
the runbook is read.

### Auth-method matrix

The mapping from API endpoint to the authentication methods it accepts, derived from the vendored spec's security
declarations at build time. It exists to reject a request client-side when the chosen auth method cannot succeed, and it
fails the build loudly when a shortcut's endpoint disappears from the spec — the earliest tripwire that a spec revision
broke the command surface.

### Spec↔types validation gate

The test-suite gate that compares each typed response's declared fields against the corresponding schema in the vendored
spec, failing when the spec renamed or removed a field the hand-written types still declare. Direction is struct→spec
only: new upstream fields never fail the gate, since forward compatibility is handled by the types' unknown-field
capture. Documented divergences from a buggy upstream spec are carried on an explicit allowlist with reasons.
