---
title: Conformance JSON Comparison - Plan
type: test
date: 2026-10-08
status: planned
implementation: not started
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
---

# Conformance JSON Comparison - Plan

**Target repo:** brettdavies/xurl-rs. Every path is in it.

---

## Goal Capsule

- **Objective:** the Go parity suite compares what `xr` and Go `xurl` print for every shortcut command both have, not
  only how they exit, without a request to X.
- **Means:** point both binaries at the library's in-process X API, `xdk::testing::MockX`, through `API_BASE_URL`, give
  both the same stored credentials, and add `stdout_json` cases (KTD1 to KTD4).
- **Authority:** the Product Contract's R-IDs win on behavior, KTDs win on mechanism, and a unit overrides neither.
- **Execution profile:** inline, with no subagents, workers, or worktrees. One stack of PRs to `dev`. The X API is
  pay-per-use, so no unit adds a live case.
- **Stop conditions:** stop and ask when a difference in output is neither a defect in `xr` nor something
  `KNOWN_DIFFERENCES.md` already records, since recording a new intentional difference is Brett's call.
- **Who finishes:** the implementer opens each PR and verifies its CI rollup; Brett merges every PR.

---

## Product Contract

### Summary

`crates/xurl-cli/tests/conformance/test_cases.toml` holds 36 cases. Three compare `stdout_json`, and all three need the
live API, so the suite skips them. The other 33 compare exit codes, and a few compare plain stdout. `xr` builds its
output through `Value -> typed struct -> to_value()`, so a field a typed response drops or renames passes every case
that runs.

### Problem Frame

The `Go parity` job is a required check on `dev` and `main`. It installs Go `xurl` at a pinned commit (`18dcb447`,
v1.3.4), sets `XURL_ORIGINAL_BIN`, and runs `cargo test -p xurl-rs --test conformance_runner`. What it proves today is
that both binaries accept the same invocations. It does not prove they print the same data, which is the claim the
suite's name makes.

The harness has no server to answer a request, which is why the JSON cases are live. Both binaries can be pointed at
one: Go `xurl` reads `API_BASE_URL`, `AUTH_URL`, `TOKEN_URL`, and `INFO_URL` in `config/config.go` at the pinned commit,
the same names `xr` reads.

### Requirements

- **R1.** Every shortcut command that exists in both binaries has a case that compares `stdout_json`, or a `skip_reason`
  that names its `KNOWN_DIFFERENCES.md` entry.
- **R2.** No case sends a request to X. A case that would is refused by the harness, not skipped by convention.
- **R3.** A shortcut added to `xr` later fails the suite until it has a case or a recorded reason for having none.
- **R4.** The suite stays green when `XURL_ORIGINAL_BIN` is unset: it skips locally as it does now.
- **R5.** Neither binary reads or writes the developer's real token store, including Go `xurl`'s move of `~/.xurl`.

### Success Criteria

- `XURL_ORIGINAL_BIN=<path to Go xurl> cargo test -p xurl-rs --test conformance_runner` passes with a `stdout_json`
  comparison for each command R1 covers.
- The `Go parity` job passes on the PR.
- Removing a field from a typed response in `crates/xdk/src/api/response/types.rs` fails a case, where today it fails
  none.

### Scope Boundaries

- No change to what either binary prints. A difference the cases find is fixed in `xr`, or recorded, in its own PR.
- No live cases, and no change to the three that exist beyond what R2 requires.
- Raw mode, streaming, and media upload are out: their output is X's body passed through, which the exit-code cases
  cover, or a multi-request flow whose comparison needs its own design.
- Commands only `xr` has are out of the comparison and in the coverage walk (R3), marked as having no counterpart.

---

## Planning Contract

### Key Technical Decisions

- **KTD1. The mock is `xdk::testing::MockX`.** It answers every endpoint the shortcut layer declares with the fixtures
  the response types are validated against, and `crates/xdk/tests/mock_endpoint_coverage.rs` fails when a declared
  endpoint has no route. The CLI crate takes the library's `testing` feature as a dev-dependency, which #323 adds for
  `dry_run_guard.rs`.
- **KTD2. One server per run, started by the runner.** `conformance_runner.rs` is synchronous and spawns both binaries
  with `std::process`. It builds a Tokio runtime, starts `MockX` on it, and keeps both alive for the run. The mock's
  origin reaches each child as `API_BASE_URL`, with `AUTH_URL`, `TOKEN_URL`, and `INFO_URL` set under the same origin,
  so a refresh or a profile lookup cannot leave the machine either.
- **KTD3. Both binaries read the same credentials from their own scratch store.** The store format is the one both read.
  The runner writes one YAML document holding an app with an OAuth2 token whose expiry is far in the future, an OAuth1
  token, and a bearer token, to `xr`'s store path (`XURL_TOKEN_STORE`, through `common::xr_std_with_store_at`) and to
  `.xurl/auth.yml` under the scratch home the original already runs against (`common::original_std_at`).
- **KTD4. A case declares that it needs the mock.** A `mock = true` key on a case gives it the mock's environment and
  the seeded stores. A case tagged `api` without it is refused, which is how R2 is held: the three live cases become
  mock cases or go.
- **KTD5. Coverage is a walk, not a list.** A test reads the shortcut commands from clap (the commands that take
  `--auth`, which is how #323's `dry_run_guard.rs` finds them), and fails for one with neither a `stdout_json` case nor
  a row in a short `NO_COUNTERPART` table naming why. Its failure message names the cause and the file to edit.
- **KTD6. Differences are named where they are ignored.** A field one binary prints and the other does not is a defect
  or a recorded difference. A recorded one is ignored with `json_ignore_fields` on its case and a comment naming the
  `KNOWN_DIFFERENCES.md` entry; nothing is ignored suite-wide.

### Risks & Dependencies

- **Scheme selection differs.** `xr` picks a scheme from what the endpoint accepts; Go `xurl` uses its own order. The
  mock answers any credential, so the outputs still compare. A case that asserts on the `Authorization` header is out of
  scope.
- **Go `xurl` resolves `/2/users/me` differently for some commands.** A command that needs the caller's id may send a
  different number of requests in each binary. Only stdout is compared, so the count does not matter.
- **Output shape.** Go prints the response body as received. `xr` prints its typed response, which names keys as the
  spec does where X answers in its legacy vocabulary. Each such key is one of KTD6's recorded differences, and finding
  them is the point of the work.
- **`#323` (dry-run) adds the `testing` dev-dependency.** If this lands first, U1 adds it.

---

## Implementation Units

### U1. Give the harness a mock and seeded stores

- **Files:** `crates/xurl-cli/tests/conformance/mod.rs`, `crates/xurl-cli/tests/conformance_runner.rs`,
  `crates/xurl-cli/tests/common/mod.rs`, `crates/xurl-cli/Cargo.toml`.
- **Work:** `DifferentialRunner` owns the runtime, the `MockX`, and one seeded store per binary. `TestCase` gains
  `mock`. `run_command` sets the four URL variables and the store for a mock case. A case tagged `api` without `mock` is
  an error at load.
- **Test first:** a mock case for `whoami` that compares `stdout_json`, run with `XURL_ORIGINAL_BIN` set, fails on the
  missing environment before the harness change and passes after. Assert through `MockX::requests` that both binaries
  reached the mock.
- **Verification:** the existing 33 cases pass unchanged; `store_isolation_guard.rs` passes.

### U2. Compare the read commands

- **Files:** `crates/xurl-cli/tests/conformance/test_cases.toml`, `KNOWN_DIFFERENCES.md` when a difference is recorded.
- **Work:** one case each for `read`, `search`, `whoami`, `user`, `timeline`, `mentions`, `bookmarks`, `likes`,
  `following`, `followers`, and `dms`, with `compare = ["exit_code", "stdout_json"]`. The three live cases are replaced
  by their mock forms.
- **Verification:** each new case is seen failing once, by dropping a field from the response type it reads, before it
  is counted.

### U3. Compare the write commands

- **Files:** as U2.
- **Work:** one case each for `post`, `reply`, `quote`, `delete`, `like`, `unlike`, `repost`, `unrepost`, `bookmark`,
  `unbookmark`, `follow`, `unfollow`, `block`, `unblock`, `mute`, `unmute`, and `dm`. `delete` passes `--force` to `xr`
  only if Go `xurl` takes the flag; otherwise the case carries per-binary arguments, which U1's `TestCase` then has to
  allow.
- **Verification:** as U2.

### U4. Hold coverage with a walk

- **Files:** `crates/xurl-cli/tests/conformance_runner.rs`, `AGENTS.md`.
- **Work:** the walk of KTD5, and a line under "Adding a command family" in `AGENTS.md` naming the conformance case as a
  surface. `recipe_guard.rs` requires the walk to be named there.
- **Test first:** remove one case and see the walk name the command and `test_cases.toml`.

---

## Verification Contract

- `cargo test -p xurl-rs --test conformance_runner` with `XURL_ORIGINAL_BIN` unset: every case skips, the walk runs.
- The same with `XURL_ORIGINAL_BIN` set to Go `xurl` at the pinned commit: every mock case compares `stdout_json`.
- `scripts/hooks/pre-push`, and the `Go parity` job on each PR.

## Definition of Done

- R1 to R5 hold, each shown by a case or a test named in its unit.
- `test_cases.toml` has no case that needs X.
- Each recorded difference has a `KNOWN_DIFFERENCES.md` entry its case names.
