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

### Drift report

The comparison of the vendored spec against the live upstream document that a refresh PR's agent runbook carries, as do
the drift workflow's PR comment and the local pre-push warning. It gives each kind of change its own section: the path
and schema inventories, categorical values, auth methods, and every other field added, removed, or changed in place,
grouped by location. Prose edits are counted rather than listed, so each change appears in exactly one place. No
section's silence stands in for another's: the section for changes in place is built from those changes alone, so it
appears beside an auth change as readily as by itself.

### Refresh PR

The automation-owned pull request that carries a new vendored spec revision toward the integration branch. Its
head branch belongs to the workflow: the branch is regenerated rather than manually maintained, and at most one refresh
PR is open at a time — a human-owned refresh PR takes precedence and the automation stands down.

The body starts workflow-owned (an agent runbook, rewritten with a fresh drift report whenever the vendored content
changes) and becomes agent-owned once the reconciling agent rewrites it to the repository's PR template; a no-change
pass updates only the run reference so agent-authored sections survive.

The workflow recognizes its open refresh PR by a fixed phrase in the title, so a reconciling agent that retitles the PR
for what it adds keeps that phrase. A refresh PR without it is invisible to the next run, which regenerates the head
branch from the integration branch, and that closes the PR.

### Agent runbook

The generated body of a fresh refresh PR: hardcoded reconciliation instructions addressed to whichever agent works the
PR. It assigns the role explicitly (executing the reconciliation is the deliverable, not a review), orders the steps,
and splits decision authority — spec-mandated changes are implemented and logged as breaking changes, while judgment
calls are proposed with a recommendation and precedent-based reasoning for a human to resolve.

### Reconciling agent

The agent invoked against an open refresh PR to bring the codebase in line with the new vendored spec: renames the spec
mandates, retirements the spec forces, commands for the added endpoints that fit the tool with an endpoint decision
proposed for each one that does not, derived-artifact regeneration, and the final rewrite of the PR body to the
repository template. Invocation phrasing matters: a review-shaped prompt routes agents into report-only behavior before
the runbook is read.

### Auth-method matrix

The mapping from each endpoint a shortcut command calls to the authentication methods it accepts, derived from the
vendored spec's security declarations at build time. It exists to reject a request client-side when the chosen auth
method cannot succeed, and it fails the build loudly when a shortcut's endpoint disappears from the spec, which makes it
the earliest tripwire that a spec revision broke the command surface. An endpoint outside it, and every raw-mode
request, is permissive: the request goes out under the preferred stored credential, or the one named, and X judges it.

### Endpoint decision

An entry in `endpoint-decisions.json` recording what was decided for spec operations that have no shortcut command:
`planned` (a command family to build, retired when it ships), `raw-only`, or `not-supported` (no command until the
entry's `revisit_when` trigger fires). `raw-only` means `xr` already serves the need, so a command would add nothing:
raw mode or another command covers it, as with a second path to a resource a command reads, a batch of one, a niche
read, or a stream. `not-supported` means the capability needs something `xr` does not hold or drive, such as XChat's
encryption keys, a live call's audio socket, or the encoder that takes a broadcast live; raw mode still reaches the
endpoint. Each entry carries a reason, who decided, and when. Every spec operation has a shortcut or a decision, and
only a person changes one.

### Spec↔types validation gate

The test-suite gate that compares each typed response's declared fields against the corresponding schema in the vendored
spec, failing when the spec renamed or removed a field the hand-written types still declare. Direction is struct→spec
only: new upstream fields never fail the gate, since forward compatibility is handled by the types' unknown-field
capture. Documented divergences from a buggy upstream spec are carried on an explicit allowlist with reasons. It cannot
see the spec itself diverging from what the API sends on the wire; that is the live smoke gate's job.

### Spec bounds gate

The test-suite gate that holds each limit a shortcut validator checks before sending (an item count, a text length, a
maximum value) to the bound the vendored spec states for the request field it guards, so a refresh that moves a bound
fails it in either direction. A bound the spec lowered is a change in behavior, because a call that passed before is now
refused. The gate is also a walk over the library's declared limits: each one records where the spec states it or
carries the reason it mirrors none, so a limit added later cannot sit outside the gate.

## Credential store and test isolation

### Token store

The per-machine credential file the CLI reads and writes: a set of registered apps, each holding its client credentials
and any OAuth1, per-user OAuth2, or bearer tokens, plus the name of the default app. One store backs a run. The binary
resolves it under the home directory unless the operator names another file through the supported variable; library
callers pass the path explicitly.

Loading an up-to-date store never rewrites it: credentials supplied through the environment are backfilled in memory and
reach disk only on the next explicit save. Only converting a legacy-format file writes during load. Every sibling file
the auth flow needs, such as the pending state of a headless sign-in, derives its path from the store's path.

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

## Sign-in

### Interactive sign-in

The OAuth2 sign-in that opens the provider's authorize page in a browser and receives the redirect on a loopback
callback listener the CLI binds at the app's registered redirect address. It runs only when output goes to a terminal
and the caller has not asked for the headless form.

The listener opens the browser only once it is bound, and serves one redirect: a request for another path is answered
not-found and the wait goes on, while a request on the callback path ends the sign-in, completing it when the state
matches and a code is present and refusing it otherwise. The refusal is shown in the browser.

### Headless sign-in

The two-step OAuth2 sign-in for a host where no browser can reach a local listener: the first step prints the authorize
URL and records the flow's pending state beside the token store, and the second takes the redirect URL the operator
pastes back, checks it against that pending state, and exchanges the code. No listener is bound. The CLI switches to it
on its own when output is not a terminal.

*Avoid:* remote flow, copy-paste flow

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

### Live session

A broadcast or call while it is running. No `xr` command acts inside one: going live, ending it, and taking part in its
chat or audio belong to the app that drives the session, while commands manage the resources around it and read what it
leaves behind. The library keeps its calls for those endpoints for a program that drives a session, and raw mode
reaches them.

### Dry-run envelope

The answer any command gives under `--dry-run`, on stdout at exit 0 with nothing sent to X and no stored credential
changed: `status: dry_run`, whether the real run would succeed, the exit code it would end on, and the command's inputs.
A command that would send a request also names the credential it would go out under, read offline: an expired token is
reported rather than refreshed, and whether X still accepts the credential is not checked. When the real run would fail,
`reason` names why; a command that would stop to ask for confirmation says so rather than asking. The first command to
open an old-format store upgrades it once, a dry run included.

### Error envelope

The structured error object every machine-readable output mode emits on stderr: `status`, a typed kebab-case `reason`
from a closed vocabulary, `exit_code`, and an optional `message`, plus per-error fields such as `next_step`, `command`,
and `suggestion`, and `warnings` when the library warned before the run failed. Agents branch on `reason` and
`exit_code`; the text-mode rendering of the same error may add hints that never appear in the envelope.

### Errors-only answer

A 200 from X that carries `errors` and no `data`: X refusing the request itself rather than failing it with an error
status. `xr` reads it by the problem type of its first error, as not found, forbidden, or an invalid request, never as a
success and never as its own fault. A lookup by handle that finds no account for any handle it named reads as not found
naming the first such handle, the same answer a lookup that finds only some of its handles gives.

### Retired word

A command word a command family no longer takes because it was renamed: typed anyway, it refuses as an unknown command
whose suggestion names the word that replaced it, never the sibling nearest by spelling, and it never runs as an alias.
A command dropped with no replacement is not a retired word; it refuses as any unknown command does.

### Walk

A test that reads a live declaration, such as the command tree, a registry, or the declared endpoints, and fails naming
each entry a required surface lacks, so that adding to the declaration without the matching surface fails a test
rather than a review. A walk checks against the declaration itself, never against a list written into the test.

### Known difference

A deliberate divergence of `xr` from the Go original it ports, stated against Go `xurl` 1.3.4 and verified by running
both binaries rather than by reading either one's documentation. Each is written up with what the original does, what
`xr` does, and why. A command or flag the original spells differently is a known difference too, so a reader migrating a
Go invocation finds its `xr` form among them.

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

The record is keyed by the kind of break, and a note beside it names the item it was taken for. It is scoped to one
release: once that release moves the baseline past the change, the break it was taken for no longer registers, yet the
record stands and would reclassify the next break of the same kind, so the release's backport removes it. Recording a
break is not silencing it. The gate still names the item and still refuses any release below the recorded level.

## Release flow

### Backport

The pull request that carries a release's bookkeeping from `main` back to the integration branch once the release has
published: version numbers, the lockfile's workspace-member versions, each crate's new changelog section, and any other
file the release branch edited. When the release moved the library, it also removes each accepted break the new tag
made stale. It lets the next release start from the released baseline instead of reverting what the last one shipped.

A backport is always a pull request, never a merge of `main` into the integration branch or a direct push. It adopts a
file from `main` on its own only when the integration branch has not touched it since the previous release (a
release-prep path); a file both branches changed (a contested path) is reported and taken only when named. The lockfile
is refreshed from the synced manifests rather than copied from `main`, because `main`'s copy would undo dependency
updates the integration branch merged after the release.
