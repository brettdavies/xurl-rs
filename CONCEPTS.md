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

### Endpoint decision

An entry in `endpoint-decisions.json` recording what was decided for spec operations that have no shortcut command:
`planned` (a command family to build, retired when it ships), `raw-only`, or `not-supported` (no command until the
entry's `revisit_when` trigger fires). Each carries a reason, who decided, and when. Every spec operation has a shortcut
or a decision, and only a person changes one.

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

A missing or blank file is a fresh store with no apps; no placeholder app stands in for them, and an app named `default`
appears only when a sign-in or token save needs one. A file that exists but cannot be read or parsed also loads with no
apps, and every write to it is refused, so a damaged store is reported rather than overwritten.

### Spawn seam

The single door through which the test suite runs the built binary. It strips every variable the binary reads from the
inherited environment, including other tools' config-directory variables, leaving the home directory and the XDG base
directories alone, and points the token store at a path the child cannot write, unless the test supplies its own
temporary store, so a spawn that never asked for a store fails loudly rather than reaching a shared or real file.

### Store isolation guard

The test-suite gate that fails when any test resolves the real token store or spawns the binary outside the spawn seam.
It scans test sources and inline test modules for the idioms that reach the home directory or spawn raw, attributes each
hit to its test, and permits only tests named on an allowlist with a stated reason; a companion check fails when an
allowlisted test no longer exists. The idiom list is a denylist that grows one idiom at a time, each addition proven red
before it counts.

### Live smoke gate

The release-preflight check that reads one post and one user from the live API with the operator's real login and fails
when a typed metric reads back zero, a post metric lands in the unknown-field bucket, or the library reports having read
a key the API sent in its legacy post vocabulary under its current name. It is the only check that can see the vendored
spec naming a field the API does not send. It never runs under the default suite, refuses without an explicit opt-in
because each run spends paid reads, and is the named exception the store isolation guard permits.

### Go parity gate

The CI check that runs the differential conformance suite: each case runs against `xr` and against the Go original,
built at a pinned commit, and their results are compared. Without the original named, the suite skips rather than
passing, so this gate is the only run that compares the two; cases that need the live API stay skipped there as well.
The pinned commit is also the version every known difference is stated against.

## Command surface

### Raw mode

The curl-style path `xr [OPTIONS] <URL>`: the root command's positional is an absolute `http(s)://` URL or a
`/`-prefixed path that is prefixed with the API base URL. Raw mode bypasses the auth-method matrix and sends the request
as given. Any other bare positional is either a mistyped shortcut, reported as an unknown command, or a URL validation
error.

### Command family

A noun that owns several operations and groups them as subcommands, such as `xr bookmark-folders`, `xr calls`, or
`xr broadcasts`: a tooling noun, or an API namespace with sibling endpoints. A standalone read or action in the core
domain gets a top-level word instead. Adding one touches the surfaces AGENTS.md § "Adding a command family" lists, each
guarded by a walk.

### Dry-run envelope

The answer any command gives under `--dry-run`, on stdout at exit 0 with nothing sent to X or stored:
`status: dry_run`, whether the real run would succeed, the exit code it would end on, and the command's inputs. When
the real run would fail, `reason` names why; a command that would stop to ask for confirmation says so rather than
asking.

### Error envelope

The structured error object every machine-readable output mode emits on stderr: `status`, a typed kebab-case `reason`
from a closed vocabulary, `exit_code`, and an optional `message`, plus per-error fields such as `next_step`, `command`,
and `suggestion`. Agents branch on `reason` and `exit_code`; the text-mode rendering of the same error may add hints
that never appear in the envelope.

### Walk

A test that reads a live declaration, such as the command tree, a registry, or the declared endpoints, and fails naming
each entry a required surface lacks, so that adding to the declaration without the matching surface fails a test
rather than a review. A walk checks against the declaration itself, never against a list written into the test.

### Known difference

A deliberate divergence of `xr` from the Go original it ports, stated against the version the Go parity gate pins and
verified by running both binaries rather than by reading either one's documentation. Each is written up with what the
original does, what `xr` does, and why. A differential case that would fail only because of one is skipped with a reason
that names the write-up, never left failing. A command or flag the original spells differently is a known difference
too, so a reader migrating a Go invocation finds its `xr` form among them.

## Skill install

### Skill host

A coding-agent tool whose user skills directory `xr` can clone its agent-facing skill bundle into. Where a host
documents a variable that relocates its configuration directory, `xr` honors it; a host that documents none resolves its
skill destination from the home directory or the stand-in the operator names for it.

### Skill destination

The directory the skill bundle lands in for one skill host. It resolves, highest first, from the host's own
configuration-directory variable, the skill home (a directory the operator names to stand in for the home directory in
skill destinations only), a standard base-directory variable the host follows, and the home directory; an empty value
counts as unset.

The skill home outranks base-directory variables because it replaces the whole home while they describe parts of the
real one, which keeps a sandboxed install inside its sandbox on machines that set those variables globally. A host can
also have a legacy location it still reads but `xr` no longer installs to: an update removes a copy found there and
installs at the destination, and an install reports the copy and leaves it in place.

## Published surface

### Public API gate

The release check that compares the crate's public Rust surface against the last released tag and fails when the version
bump a change requires exceeds the one being claimed.

It reasons over the language's own compatibility model rather than a textual diff, so an added field, a changed variant
shape, or a new attribute are all visible to it regardless of which module declares them. Distinct from the command
surface check, which covers what the binary exposes rather than what downstream crates compile against. The baseline is
the last released tag rather than the integration branch, because a type introduced and reshaped between releases was
never something a consumer could depend on.

### Accepted break

A breaking change to the published library surface that the project chooses to ship below a major version, recorded so
the Public API gate reclassifies it instead of refusing the release.

The record names the item it covers and is scoped to one release: once that release moves the baseline past the change,
the record matches nothing and has to be removed, or it understates the next break of the same kind. Recording a break
is not silencing it. The gate still names the item and still refuses any release below the recorded level.

## Release flow

### Backport

The pull request that carries a release's bookkeeping from `main` back to the integration branch once the release has
published: version numbers, the lockfile's workspace-member versions, each crate's new changelog section, and any other
file the release branch edited. It lets the next release start from the released baseline instead of reverting what the
last one shipped.

A backport is always a pull request, never a merge of `main` into the integration branch or a direct push. It adopts a
file from `main` on its own only when the integration branch has not touched it since the previous release (a
release-prep path); a file both branches changed (a contested path) is reported and taken only when named. The lockfile
is refreshed from the synced manifests rather than copied from `main`, because `main`'s copy would undo dependency
updates the integration branch merged after the release.
