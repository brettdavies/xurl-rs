---
title: Scope Help to the Flags a Command Consumes - Plan
type: refactor
date: 2026-09-20
status: implementation-ready
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
---

# Scope Help to the Flags a Command Consumes - Plan

## Goal Capsule

- **Objective:** A reader of `xr <command> --help`, human or agent, sees the flags that command acts on and none that it
  discards. Today `xr whoami -h` advertises five flags `whoami` ignores.
- **Means:** Split the root globals into two tiers. Presentation and process flags stay global, because they do apply
  everywhere. The five flags that shape an API request move into two `clap::Args` groups, flattened only into the
  commands that read them, following the `CommonFlags` precedent already in the file.
- **Authority:** The parser becomes the single statement of scope. Prose in doc comments describes it; it does not
  define it.
- **Execution profile:** Two visible changes. Help pages lose flags the command never read. `xr whoami --cursor x` moves
  from silently accepted to an `unrecognized argument` usage error at exit 2.
- **Stop conditions:** Stop and ask if `anc audit`'s `p6-must-global-flags` moves off `Pass`, or if the derived
  consumption map disagrees with the map recorded below.

## Product Contract

### Problem Frame

Every `global = true` flag on `Cli` renders on every subcommand's `-h` and `--help`, consumed or not. `xr whoami -h`
lists `--limit`, `--cursor`, `--page`, `--after`, and `--dry-run`. `whoami` is a single-user read: it paginates nothing
and writes nothing, and the dispatcher never passes it any of them.

Help is the contract an agent reads before it builds an invocation. A page listing flags the command discards overstates
the accepted surface, and the cost lands on exactly the reader this CLI is built for. It also buries the two or three
flags that matter under roughly sixteen that do not.

**The prose already drifted.** `--cursor`'s doc comment at `crates/xurl-cli/src/cli/mod.rs:937` names eight commands it
threads through: `search`, `timeline`, `mentions`, `bookmarks`, `likes`, `following`, `followers`, `dms`. The dispatcher
threads `cursor_opt` into ten, adding `muted` and `blocked`, which arrived with #165. The scope exists in two places,
one of them stale. Moving it into the parser leaves one.

### Requirements

- **R1.** A command's `-h` and `--help` list `--limit`, `--cursor`, `--page`, `--after`, or `--dry-run` only when the
  dispatcher passes that command the corresponding value.
- **R2.** Presentation and process flags stay available on every command and in every help page: `--output`, `--json`,
  `--jsonl`, `--raw`, `--no-pager`, `--quiet`, `--color`, `--verbose`, `--no-interactive`, `--timeout`, `--app`.
- **R3.** Every command that reads one of the five today still reads it, with identical behavior. This unit changes
  where a flag is declared, never what it does.
- **R4.** `--page` keeps answering with the `unsupported-pagination` envelope on the commands that accept it, rather
  than becoming an unrecognized argument there.
- **R5.** `anc audit --command xr --principle 6` keeps `p6-must-global-flags` at `Pass`.
- **R6.** Passing a scoped flag to a command that does not declare it is a clap usage error at exit 2, rendered through
  the existing `invalid-args` path in both text and structured output.

### Success Criteria

- `xr whoami --help` lists none of the five. `xr search --help` lists the four paging flags. `xr post --help` lists
  `--dry-run` and no paging flag.
- The derived consumption map and the declared flattens agree, asserted by a test rather than by review.
- Golden help fixtures change only for commands that lost a flag.

### Scope Boundaries

**In scope.** The declaration site of the five request-shaping flags, the `GlobalFlags` plumbing that reads them, the
help pages, completions, and the golden fixtures.

**Out of scope.**

- Which flags a command consumes. This makes help agree with behavior; it does not change behavior.
- The root help page, which legitimately lists everything.
- `--no-pager`, which is a deliberate documented no-op for agents and stays advertised everywhere. It is the one flag
  whose value is accepted rather than read, and that is its purpose.
- `-h` versus `--help` verbosity, which clap already differentiates.
- The unknown-command help-flag defect, which is
  `docs/plans/2026-09-20-0924-fix-unknown-command-under-help-flag-plan.md` and shares no code with this.

## Planning Contract

### Key Technical Decisions

**KTD1. Two tiers, split on what the flag shapes.** A flag that changes how output is rendered or how the process
behaves applies to every command and stays `global = true`. A flag that changes the API request applies only to commands
that make that kind of request and becomes per-command. The line is mechanical, so a future flag lands on one side
without debate.

This is what keeps R5 satisfiable. `p6-must-global-flags` checks that the agentic flags are reachable everywhere; those
are exactly the tier that does not move. De-globalizing all sixteen would have put that audit at risk for no gain, since
`--output` on `whoami` is not a lie.

**KTD2. Two `clap::Args` groups, not five per-command flags.** `PagingFlags` carries `limit`, `cursor`, `page`, and
`after`; `DryRunFlag` carries `dry_run`. Commands flatten one, both, or neither. The four paging flags always travel
together, because `--page` and `--after` are documented aliases of `--cursor` and their `conflicts_with` relationships
only make sense inside one group.

`CommonFlags` at `crates/xurl-cli/src/cli/mod.rs:1587` is the existing precedent: a small `clap::Args` flattened into
the variants that want it. This extends a pattern rather than introducing one.

**KTD3. Derive the consumption map; do not hand-enumerate it.** The map below is today's snapshot, read from the
dispatcher. Re-derive it at implementation time and treat any disagreement as a stop condition, because a hand-copied
list is exactly how `--cursor`'s doc comment went stale. `run_subcommand` destructures `GlobalFlags` and threads each
field into its arms, so the map is recoverable by reading which arms receive `cursor_opt`, `global_limit`, and
`dry_run`.

Snapshot at `50a5f13`, 40 arms in `run_subcommand`:

| Group         | Commands                                                                                                                                                                                              | Count |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| `PagingFlags` | `search`, `timeline`, `mentions`, `bookmarks`, `likes`, `following`, `followers`, `muted`, `blocked`, `dms`                                                                                           | 10    |
| `DryRunFlag`  | `post`, `reply`, `quote`, `delete`, `like`, `unlike`, `repost`, `unrepost`, `bookmark`, `unbookmark`, `follow`, `unfollow`, `mute`, `unmute`, `block`, `unblock`, `dm`, `broadcasts`, `auth`, `media` | 20    |
| Neither       | `whoami`, `user`, `usage`, `schema`, `skill`, `completions`, `version`, `examples`, `validate`, `help`                                                                                                | 10    |

**KTD4. A test asserts the map, so it cannot drift again.** Walk the built `clap::Command` for each subcommand, collect
which of the five it declares, and compare against a committed table. A command that gains a paging call without
flattening `PagingFlags`, or flattens it without using it, fails. This is the permanent fix that makes the prose-drift
class of bug impossible rather than merely corrected once.

**KTD5. `GlobalFlags` loses the three fields it no longer sources from `Cli`.** `commands/mod.rs:228-250` reads
`cli.dry_run`, `cli.limit`, `cli.page`, `cli.cursor`, and `cli.after` before dispatch. Those reads move to the command
arms, which already destructure their own variant fields. The `--page` rejection at `mod.rs:234-243` moves with them, so
R4 holds per-command rather than globally.

### Risks

- **Golden churn is the review surface.** 63 of the 111 fixtures in `crates/xurl-cli/tests/golden/` are help pages. A
  large diff is expected and correct; a diff touching a page for a command in the "Neither" row that should have lost
  nothing is the signal to stop.
- **Completions regenerate.** `./scripts/generate-completions.sh --check` gates them and will fail until regenerated.
- **R6 is a behavior change, not only a help change.** `xr whoami --cursor x` is accepted today and becomes exit 2. That
  belongs in the changelog under `### Changed`.

### Sequencing

Four units, in order. U1 and U2 land together in one PR because the tree does not compile between them.

## Implementation Units

### U1. Declare the two groups and flatten them

**Goal.** R1, R2, R3, KTD1, KTD2.

**Files.** `crates/xurl-cli/src/cli/mod.rs`.

**Approach.** Add `PagingFlags` and `DryRunFlag` beside `CommonFlags`. Move the five `#[arg]` blocks off `Cli`,
preserving every attribute except `global = true`: the env bindings, `value_name`, `conflicts_with`, and the
`FalseyValueParser` on `dry_run`. Flatten each group into the variants the derived map names.

**Test scenarios.** `xr whoami --help` lists none of the five; `xr search --help` lists four; `xr post --help` lists
`--dry-run` only; `xr whoami --cursor x` exits 2.

### U2. Move the plumbing into the arms

**Goal.** R3, R4, KTD5.

**Files.** `crates/xurl-cli/src/cli/commands/mod.rs`, the `auth` and `media` subtrees.

**Approach.** Drop `dry_run`, `global_limit`, and `cursor` from `GlobalFlags`. Each arm reads its own flattened group
and builds `cursor_opt` locally. Move the `--page` rejection into the paging arms. `AuthGlobalFlags` keeps `dry_run`,
sourced from the `auth` variant's own flatten.

**Test scenarios.** Every existing paging and dry-run test passes unchanged. `xr search --page 2` still emits
`unsupported-pagination`.

### U3. The map guard

**Goal.** KTD4.

**Files.** `crates/xurl-cli/tests/`: a new test, beside the existing guards.

**Approach.** Walk `Cli::command()`, and for each subcommand collect which of the five flag names it declares. Compare
against a committed table. Assert both directions: a declared flag the map omits, and a mapped flag the command does not
declare.

**Test scenarios.** Observed failing first by planting a `PagingFlags` flatten on `whoami` and confirming the guard
names it.

### U4. Regenerate the derived artifacts

**Goal.** Keep the gates green.

**Files.** `completions/`, `crates/xurl-cli/tests/golden/`, `crates/xurl-cli/src/cli/mod.rs` doc comments.

**Approach.** Regenerate completions and re-record the golden help pages. Audit the golden diff against the map: every
changed page belongs to a command that lost a flag, and no page for a "Neither" command gains one. Correct `--cursor`'s
doc comment, which now names the group rather than a list of ten command names that would drift again.

**Verification.** `anc audit --command xr --principle 6` reports `p6-must-global-flags` as `Pass`.

## Verification Contract

| Gate                  | Command                                     | Done signal                                      |
| --------------------- | ------------------------------------------- | ------------------------------------------------ |
| Format                | `cargo fmt -- --check`                      | No diff                                          |
| Lint                  | `cargo clippy --all-targets -- -D warnings` | Clean                                            |
| Tests                 | `cargo test`                                | All pass, including the U3 guard                 |
| Golden                | `cargo test --test golden_tests`            | Only pages for commands that lost a flag changed |
| Completions freshness | `./scripts/generate-completions.sh --check` | Fresh after regeneration                         |
| Agent-readiness       | `anc audit --command xr --principle 6`      | `p6-must-global-flags` is `Pass`                 |
| Output discipline     | `bash scripts/lint-stdio.sh`                | Clean                                            |
| Schema freshness      | `cargo test --test schema_tests`            | Drift test passes                                |

## Definition of Done

- `xr whoami --help` lists none of the five flags, and `xr search --help` lists the four paging flags.
- The U3 guard exists and was observed failing against a planted mismatch, with the failure output quoted in the PR
  body.
- The golden diff was audited command by command against the derived map, and the audit is stated in the PR body.
- `p6-must-global-flags` is `Pass`, checked after the change rather than assumed.
- The PR body's `## Changelog (xurl-rs)` names both visible changes under `### Changed`: the narrowed help pages, and
  scoped flags becoming a usage error on commands that ignored them.
