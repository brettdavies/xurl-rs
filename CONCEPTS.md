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
capture. Documented divergences from a buggy upstream spec are carried on an explicit allowlist with reasons. It cannot
see the spec itself diverging from what the API sends on the wire; that is the live smoke gate's job.

## Credential store and test isolation

### Token store

The per-machine credential file the CLI reads and writes: a set of registered apps, each holding its client credentials
and any OAuth1, per-user OAuth2, or bearer tokens, plus the name of the default app. One store backs a run. The binary
resolves it under the home directory unless the operator names another file through the supported variable; library
callers pass the path explicitly.

Loading an up-to-date store never rewrites it: credentials supplied through the environment are backfilled in memory and
reach disk only on the next explicit save. Only converting a legacy-format file writes during load. Every sibling file
the auth flow needs, such as the headless OAuth2 pending state, derives its path from the store's path.

### Spawn seam

The single door through which the test suite runs the built binary. It strips every configuration variable of its own
that the binary reads from the inherited environment, leaving the home directory alone, and points the token store at a
path the child cannot write, unless the test supplies its own temporary store, so a spawn that never asked for a store
fails loudly rather than reaching a shared or real file.

### Store isolation guard

The test-suite gate that fails when any test resolves the real token store or spawns the binary outside the spawn seam.
It scans test sources and inline test modules for the idioms that reach the home directory or spawn raw, attributes each
hit to its test, and permits only tests named on an allowlist with a stated reason; a companion check fails when an
allowlisted test no longer exists. The idiom list is a denylist that grows one idiom at a time, each addition proven red
before it counts.

### Live smoke gate

The release-preflight check that reads one post and one user from the live API with the operator's real login and fails
when a typed field reads back empty, a legacy field name appears, or a value lands in the unknown-field bucket. It is
the only check that can see the vendored spec naming a field the API does not send. It never runs under the default
suite, refuses without an explicit opt-in because each run spends paid reads, and is the named exception the store
isolation guard permits.
