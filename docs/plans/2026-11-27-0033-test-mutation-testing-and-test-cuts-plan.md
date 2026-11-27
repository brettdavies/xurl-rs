---
title: Mutation Testing and Test Cuts - Plan
type: test
date: 2026-11-27
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
---

# Mutation Testing and Test Cuts - Plan

## Goal Capsule

- **Objective:** cut the xurl-rs test count a great deal without losing coverage, and find the code that no test checks.
- **Means:** cargo-mutants makes small deliberate bugs in `src` and runs the whole workspace suite against each one with
  nextest, never stopping at the first failure. A log-to-matrix script turns each mutant's log into the tests that
  killed it. The matrix gives four lists: missed mutants, tests that catch no mutant, redundant tests, and load-bearing
  tests. Missed mutants become gap tests at the public boundary. Redundant and inert tests become cuts, and each cut PR
  is confirmed by a rerun that loses no caught mutant.
- **Authority:** KTD1 to KTD8 are Brett's decisions and are not reopened. KTD9, KTD10, and KTD22 are decisions of
  xurl-plan-prime, the coordinating session, which owns the code calls. KTD21 waits on Brett. The rest are this plan's
  own, each with its evidence.
- **Execution profile:** a run of pull requests to `dev`, each from a `feat/` or `fix/` branch cut from `dev`, under the
  branch, commit, and review rules in `AGENTS.md` and the contributor's global instructions. Mutation runs happen on the
  16-CPU Linux server, never in CI.
- **Stop conditions:** stop and report when a baseline fails in the copied tree, when a cut PR's rerun ends with a
  mutant missed that its baseline caught, when a gap test needs a change under `src` other than an `exclude_re` entry
  (that change is its own `fix` or `refactor` PR), when free disk space falls below the floor, or when a cut would touch
  a golden fixture, a wire-format test, or a source-reading guard.
- **Who finishes:** each implementing slot opens its pull request and runs its reviews. The prime merges to `dev`.
  Release PRs are outside this plan.

---

## Product Contract

### Summary

A committed `.cargo/mutants.toml` makes a plain `cargo mutants` from the workspace root run the right way: nextest, the
whole workspace suite, no fail-fast, a `mutants` cargo profile without debug info, the `.git` directory copied, the
unviable constructor guesses and `Debug` impls left out, and the output outside the checkout. A pilot on
`crates/xdk/src/webhooks/` measures the cost and yields the first matrix, from which the log-to-matrix script is
written. The areas then run one at a time behind the changes that move their lines. Each area gets a PR of gap tests and
a PR of cuts.

### Problem Frame

The suite is large, and nothing measures which of its tests matter. At `dev` 492e187:

| Measure                | Count                                                                                                           |
| ---------------------- | --------------------------------------------------------------------------------------------------------------- |
| Test functions         | 2,335 (1,928 in `tests/`, 407 unit)                                                                             |
| Integration-test files | 141                                                                                                             |
| Golden fixtures        | 349 (197 help, 85 dry-run)                                                                                      |
| Test code              | 70.7k lines, against 55.5k lines of `src`                                                                       |
| Largest test files     | `cli_tests.rs` 208, `store_tests.rs` 100, `api_tests.rs` 94, `oauth2_flow_tests.rs` 81, `spec_validation.rs` 80 |

No existing tool finds deletion candidates in bulk. The gstack `test-audit` skill optimizes for confidence over deletion
count, defaults to 10 candidates per run, and its near-duplicate pre-filter works on whole files, so it can't see two
duplicate tests inside one 208-test file. `ce-simplify-code`'s test-value reviewer names the right class (the same
contract asserted at several layers) but prefers a few well-evidenced findings. Both are good at gating one deletion
with proof. Neither finds candidates in bulk.

Earlier hand audits have already landed: the 15 process-boundary copies of golden fixtures (#482), the 6
environment-mutating tests whose cover runs without one (#484), and the `.twurlrc` import tests moved onto the shipped
constructor (#485). What remains is too large for a hand audit.

Where a large cut would come from, read from the suite's shape and not yet measured:

1. **Hand-written tests that a walk already covers.** The walks run every command over every store shape and assert one
   property each: `dry_run_guard`, `credential_step_walk`, `rerun_walk`, `validation_value_guard`, `post_id_guard`, and
   `help_claims_guard`. A per-command test of the same property duplicates them by construction. 90 test functions in
   `crates/*/tests/*.rs` outside the walks and guards have `dry_run` in their name, and 27 test files there outside the
   walks and guards mention `next_step` (`rg -l next_step crates/*/tests/*.rs`, less the files named for a guard or
   walk). Some of them pin a value no walk checks.
2. **The same contract at several layers:** the library (`crates/xdk/tests`), the CLI driven in-process
   (`run_with_store_path`), the spawned binary (`common::xr()`), and a golden fixture. A dry-run envelope pinned by its
   golden and by field assertions in `cli_tests.rs` is the typical case.

Mutation testing turns "break the behavior and see whether a test fails" into a command, and its matrix of mutants
against killing tests is the evidence each cut and each gap test rests on.

### Requirements

**Configuration**

- R1. A plain `cargo mutants` from the workspace root, in any clone, runs the whole workspace suite under nextest
  against each mutant, with `--no-fail-fast`, the `mutants` cargo profile, the `.git` directory copied, and the two
  exclusions, and writes nothing inside the checkout.
- R2. The `mutants` cargo profile builds without debug info and keeps the dependency opt-levels in `.cargo/config.toml`.

**Runs**

- R3. Every run holds the cost guardrails: a disk-space floor, a hard `timeout -k` bound, `-j2` or `-j3`, an overnight
  batch size, its scratch directories cleaned when it ends, and no start while another slot builds.
- R4. A run's results are used only against the `src` they ran on. A file whose content has changed since its run is run
  again before a cut relies on it.

**The matrix**

- R5. A log-to-matrix script reads the output of one or more runs and writes the mutant-by-test matrix and the four
  lists. It checks itself against cargo-mutants' own outcome files and fails, naming the mismatch, when they disagree.

**Pilot and areas**

- R6. The pilot runs on `crates/xdk/src/webhooks/` after #645 to #647 merge and after R's lever 8 and merged test layout
  land. It measures the per-mutant cost, the unviable and timeout rates, the cost of `--no-fail-fast`, and a build
  directory's disk use.
- R7. Whether to write a project-local skill is decided after the pilot, from what the pilot needed.
- R8. After the pilot, the areas run one at a time: `store` and `auth` after F's fix PRs, then the rest of `xdk`, then
  the CLI after R's source splits. Each area run starts only after every change that moves its `src` lines has landed.

**Gap tests and cuts**

- R9. Each area gets one PR of gap tests. A missed mutant gets a test at the public boundary that fails on it, or an
  `exclude_re` entry naming its file and function with the reason when it changes no behavior. Every missed mutant is
  checked against the doctests first. `xdk`'s gap PR for the rest of `xdk` carries the `url_authority.rs` property test.
- R10. Each area gets one PR of cuts. Its candidates come from the four lists and from discovery classes 1 and 2. Each
  deleted test carries a retirement card (`detects`, `stronger_proof`, `history`, `validation`), and a rerun shows that
  no caught mutant is lost.
- R11. Golden fixtures, wire-format tests, and source-reading guards are never cut, and the CLI's cuts land before R's
  `cli_tests.rs` split.

### Acceptance Examples

- AE1. In a fresh clone at M1's merge commit, `cargo mutants --workspace --list | wc -l` prints the count M1's PR body
  records (4,473 at 492e187), and `git status --porcelain` prints nothing afterwards.
- AE2. In that clone, `cargo mutants --workspace -f crates/xdk/src/webhooks/signing.rs -F '<one mutant name>'` passes
  its baseline in the copied tree, tests the one mutant, and writes `/tmp/xurl-rs-mutants/mutants.out`, with nothing new
  inside the checkout. `signing.rs` holds no struct-field mutant, which `-F` would not filter out (KTD10), so the run
  tests exactly one.
- AE3. `cargo build -p anstyle-parse --profile mutants -v` shows `-C opt-level=3` and `-C strip=debuginfo` on the
  `anstyle_parse` compile line, and no `-C debuginfo`.
- AE4. On the pilot's output, the script's missed list equals `missed.txt` line for line, and every mutant in
  `caught.txt` has at least one killing test in the matrix.
- AE5. A cut PR's rerun ends with every rerun mutant caught, and its body quotes the summary line.
- AE6. A gap test is seen failing on its mutant's diff before the gap PR is pushed, and the PR body quotes that failure.

### Scope Boundaries

- **Out of scope, as decided (KTD7):** golden fixtures, wire formats, and source-reading guards. Mutants model some bug
  kinds only. Exact golden bytes, wire formats, ordering, and the structural guards that read source catch no mutants by
  design, so the matrix says nothing about their value. They stay, and they seed the keep set (KTD19).
- **Not in any plan, each needing Brett's say-so:** a CI `--in-diff` mutation job; splitting the other large test files
  (`oauth2_flow_tests.rs`, `api_tests.rs`, `golden_tests.rs`, `store_tests.rs`); a doc-comment accuracy audit; the media
  upload chart.
- **`crates/xdk-consumer-check`** (25 mutants) is in no area. It exists so CI's `Consumer check` compiles an embedder
  against the documented surface, and its mutants change what that example prints.
- **The skill's design.** M4 holds the decision point only.
- **The attribute form of a skip.** `#[mutants::skip]` needs the `mutants` crate as a dependency, which takes Brett's
  say-so (KTD22).
- **A run wrapper script.** The run procedure (KTD16) is a list of commands. If the pilot shows the steps repeat, M4's
  skill decision is where they would be captured.
- **CI.** No workflow runs cargo-mutants, and nothing here changes CI.

### Sources

- Brett's decisions for this plan, relayed in the coordinating session's brief of 2026-11-27 (KTD1 to KTD8), and
  xurl-plan-prime's calls on KTD9, KTD10, and KTD22.
- The cleanup order-of-events note of 2026-11-26: Phase 5 is this plan, and its phases 0 to 4 and its order rules set
  this plan's gates. Its plan R is the refactor plan,
  `docs/plans/2026-11-27-0026-refactor-file-splits-and-test-layout-plan.md` on `dev`: lever 8 is its U1, the merged test
  layout its U2, the source splits its U5 to U8 (`response/types.rs` is U8), the `hints.rs` test move its U9, and the
  `cli_tests.rs` split its U10. Its plan F is the formal-verification plan,
  `docs/plans/2026-11-27-0031-test-formal-verification-plan.md`, whose fix PRs for `auth` and `store` are its U5. This
  plan cites the order note's phases and steps beside those units.
- The pstack skills audit of 2026-11-26, § "Tests" through § "Do we want our own skill for it?": suite size, discovery
  classes, the gate, the four lists, the three wrong-answer traps, and the skill question.
- The test audit of 2026-11-03 (CI plan U2): its decision list (batches A to C, landed as #482, #484, #485), its
  retention list, and its timings.
- The tests wall-clock spike of 2026-11-25: lever 8 (the hermetic skill clone) and the merged test layout's measured
  gains and guard inventory.
- cargo-mutants 27.1.0, the installed binary (`cargo mutants --help`, `--emit-schema=config`), its book (`book/src`),
  and its `src/`, read at tag `v27.1.0` (`f2820cd`). The book pages this plan relies on are the same at `6c720ae` except
  for a link in `output.md`.
- The repo at 492e187: `AGENTS.md`, `.config/nextest.toml`, `.cargo/config.toml`, the root `Cargo.toml`,
  `crates/xurl-cli/tests/nextest_config_guard.rs`, `crates/xdk/build.rs`, `crates/xdk/tests/build_info.rs`,
  `crates/xurl-cli/tests/cargo_inputs_tests.rs`, `scripts/cargo-inputs.sh`, `crates/xdk/src/api/request/call.rs`,
  `crates/xdk/src/api/url_authority.rs`, and `docs/plans/2026-10-27-0058-perf-ci-scoping-and-speed-plan.md` (U6).
- `qmd query` over the solutions corpus for mutation testing, test pruning, and nextest. The documents it surfaced that
  bear on this plan: "Triage the test suite when you tombstone a subject" (incidental coverage lost to a deletion), "A
  nextest log's (n/N) counter is finish order" (a CPU-bound test's time depends on what shares its machine), and "Batch
  expensive verification at unit boundaries".

---

## Planning Contract

### Key Technical Decisions

KTD1 to KTD8 are Brett's decisions.

- KTD1. **`.cargo/mutants.toml` is committed by a later PR.** It sets `test_tool = "nextest"` and
  `test_workspace = true`; an `exclude_re` for the unviable `Call::` constructor guesses in
  `crates/xdk/src/api/shortcuts.rs` and for `fmt` impls; `profile = "mutants"`, for a cargo profile `mutants` with
  `debug = "none"`; and an output location outside the tree. (session-settled: user-directed.) Governs R1, R2. M1.
- KTD2. **Every run passes `--no-fail-fast` through to nextest.** (session-settled: user-directed, chosen over
  cargo-mutants' fail-fast default: the matrix needs every failing test, not the first.) Governs R1, R5.
- KTD3. **A log-to-matrix script turns each mutant's log into the tests that killed it**, written during the pilot from
  the pilot's real logs, and yields four lists: missed mutants, tests that catch no mutant, redundant tests, and
  load-bearing tests. (session-settled: user-directed.) Governs R5. M3.
- KTD4. **The pilot runs on `crates/xdk/src/webhooks/` (151 mutants after the exclusions)**, after PRs #645 to #647
  merge and after R's lever 8 and merged test layout land. (session-settled: user-directed, chosen over `store`: neither
  R nor F touches `webhooks`, so the pilot's lists stay valid.) Governs R6. M2.
- KTD5. **Whether to write a project-local skill is decided only after the pilot.** (session-settled: user-directed.)
  Governs R7. M4.
- KTD6. **The areas run one at a time: `store` and `auth` after F's fixes, then the rest of `xdk`, then the CLI after
  R's source splits. Each area gets a PR of cuts confirmed by a rerun that loses no caught mutant, and a PR of gap tests
  at the public boundary. The `url_authority.rs` property test goes in `xdk`'s gap PR.** (session-settled:
  user-directed.) Governs R8 to R10. M5 to M9.
- KTD7. **Out of scope: golden fixtures, wire formats, and source-reading guards**, and the order note's "Not in any
  plan" items. (session-settled: user-directed.) Governs R11.
- KTD8. **Cost guardrails: a disk-space floor, `timeout -k`, `-j2` or `-j3`, overnight batches, scratch directories
  cleaned after each run, and no runs while other slots build.** (session-settled: user-directed.) Governs R3. KTD16
  turns each one into a step.

The decisions below settle what the sources leave open.

- KTD9. **`copy_vcs = true`.** (xurl-plan-prime.) cargo-mutants copies the tree into a temp directory without `.git` by
  default (book, `build-dirs.md`: "files or directories matching these patterns are not copied ... `.git` ... If your
  tree's build or tests require the VCS directory then it can be copied with `--copy-vcs=true` or by setting
  `copy_vcs = true` in `.cargo/mutants.toml`"; `src/copy_tree.rs` skips `VCS_DIRS` unless `copy_vcs`). Four tests fail
  outside a git checkout, so without it the baseline fails and the run tests no mutant (exit 4):
  - `build_info::crate_git_sha_is_set_in_git_checkout` expects `CRATE_GIT_SHA`, which `crates/xdk/build.rs` reads with
    `git rev-parse HEAD` (`run_git`, which returns `None` outside a checkout);
  - the three `cargo_inputs_tests`, whose `scripts/cargo-inputs.sh` exits with "not inside a git work tree" (line 80)
    and lists tracked Rust files with `git ls-files`;
  - the spike saw exactly these four fail in a scratch copy with no `.git`. The slot clone's `.git` is 8.4 MB, copied
    once per build directory.
- KTD10. **The exclusions are `with Call::` and `<impl (std::)?fmt::Debug for`.** (xurl-plan-prime, on the count to use
  and on keeping user-visible `fmt` impls.) § "Counts" gives the commands and totals: 7,196 mutants at 492e187, 4,473
  after both exclusions.
  - `with Call::` matches 2,703 mutants: 2,686 in `shortcuts.rs` (1,004 `Call::new(...)`, 841 `Call::from(...)`, 841
    `Call::from_iter(...)`) and 17 in `crates/xurl-cli/src/cli/commands/mod.rs`. None can compile: `Call`'s only
    constructor is `pub(crate) fn new(client: &Client, request: RequestOptions)`
    (`crates/xdk/src/api/request/call.rs:245`), and `Call` implements no `Default`, `From`, or `FromIterator`. The regex
    names the replacement, not the file, so it keeps matching after R splits `commands/mod.rs`.
  - `<impl (std::)?fmt::Debug for` matches the 20 hand-written `Debug` impls (`Client`, `ClientBuilder`, `Call`,
    `Config`, `EnvOverrides`, `Auth`, `TokenStore`, the store's `App`, `Token`, `OAuth1Token`, and `OAuth2Token`,
    `OAuth1Credential`, `OAuth2Credential`, `SigningSecrets`, `UrlTarget`, `StreamLines`, `StreamSourceCreated`,
    `StreamSourceKey`, `MediaUpload`, `MockX`). They are developer diagnostics. Several redact secrets, and the only
    mutant cargo-mutants makes of them prints nothing, which cannot leak one.
  - The four `Display` impls stay in the run, because each renders text a user reads: `AuthMismatch`
    (`crates/xdk/src/error.rs:465`, the auth-mismatch message), `NoTempFile` (`crates/xdk/src/store/atomic.rs:158`),
    `ForwardFailure` (`crates/xdk/src/webhooks/forward.rs:36`, why a webhook forward failed), and `ProgramFailure`
    (`crates/xurl-cli/src/cli/commands/webhooks_program.rs:42`, why a `webhooks listen` program failed).
  - Genres of the 4,473: 2,608 function-value, 980 binary-operator, 322 match-arm, 295 unary-operator, 144 match-guard,
    124 struct-field. `--list` builds nothing (23 s, 83 MB) and leaves `git status` clean.
  - No name regex reaches a struct-field mutant (`delete field <f> from struct <S> expression in <fn>`). cargo-mutants
    27.1.0 pushes that genre without the `allows_mutant` check every other genre passes (`src/visit.rs:747` against
    `src/visit.rs:335`, read at `v27.1.0`), so neither `-F`/`examine_re` nor `-E`/`exclude_re` filters it. Probe at
    492e187: `cargo mutants --workspace --list -f 'crates/xdk/src/webhooks/**' -F '^no such mutant$'` exits 0 and lists
    `crates/xdk/src/webhooks/budget.rs:102:16: delete field first_wait from struct Self expression in
    RetryBudget::with_first_wait`, and `-E 'delete field'` leaves it listed. All 124 struct-field mutants are in the
    4,473: 95 in the CLI, 26 in the rest of `xdk`, 2 in `store`, 1 in `webhooks`, none in `auth`. A file filter (`-f`,
    `-e`) does reach them. KTD20 and KTD22 work around the gap.
- KTD11. **`[profile.mutants]` lives in the root `Cargo.toml`, beside `[profile.release]`, with `inherits = "test"` and
  `debug = "none"`.** The cargo-mutants book suggests that table there (`performance.md`). `.cargo/config.toml` already
  holds the ten `[profile.dev.package.*]` opt-levels from #641, and its comment says CI's cache key hashes that file and
  not the virtual root manifest, so a profile in the root manifest leaves CI's cache untouched.
  `crates/xurl-cli/tests/release_profile_guard.rs` compares `[profile.release]` only, so a new root profile doesn't trip
  it. Cargo merges the config file's `dev` profile before a custom profile inherits from `test` and, through it, `dev`,
  so the opt-levels carry over. Measured with the profile given on the command line, which cargo merges as it would a
  config file (`cargo build -p anstyle-parse --profile mutants --config 'profile.mutants.inherits="test"' --config
  'profile.mutants.debug="none"' -v`): `anstyle_parse` compiles with `-C opt-level=3`, `utf8parse` with none, both with
  `-C strip=debuginfo` and no `-C debuginfo`, into `target/mutants/`. M1 runs the same check against the committed root
  placement (AE3). `debug = "none"` leaves `debug-assertions` on, which `debug_assert!` in `rerun.rs` needs, and no test
  asserts on backtrace content.
- KTD12. **`--no-fail-fast` goes in `additional_cargo_test_args`, not after `--` on each command line.** A key in the
  committed config holds for every run, a plain `cargo mutants` included, which "every run" (KTD2) needs. cargo-mutants
  appends that key only to the test phase (`src/cargo.rs`, `cargo_argv`:
  `if phase == Phase::Test { cargo_args.extend(options.additional_cargo_test_args...) }`), so each mutant's test command
  is `cargo nextest run --cargo-profile=mutants --verbose --workspace --no-fail-fast`. The book warns about mixing that
  key with a second `--` for test-binary arguments. No run here passes any.
- KTD13. **Runs use nextest's `default` profile, and no nextest profile is added.** The `ci` profile sets `retries = 1`
  and `fail-fast = false`. A retry costs time and changes no outcome here, because `flaky-result = "fail"` holds in
  every profile. `default` stops at the first failure, which `--no-fail-fast` overrides, and ends a hung test after 5
  minutes (`slow-timeout = { period = "60s", terminate-after = 5 }`), which nextest reports as that test failing.
  `nextest_config_guard.rs` holds every profile to both rules and is untouched.
- KTD14. **Output stays outside the checkout two ways.** The committed config sets `output = "/tmp/xurl-rs-mutants"` for
  a plain run, and every run in this plan passes `--output <run>/out` (KTD16), which beats the config value (book,
  `config-file.md`: for scalar options, the command line takes precedence). The `output` value is used as written: a
  relative path is relative to the current directory (`src/main.rs`,
  `output_parent_dir = options.output_in_dir ... unwrap_or(workspace.root())`), and nothing expands `~` or a variable.
  So the only value that is outside every clone from any directory is an absolute path, and `/tmp` exists on Linux and
  macOS. cargo-mutants creates the last path component itself (`create_dir`, not `create_dir_all`, in `src/output.rs`).
  The default it replaces is `mutants.out` in the workspace root, which would need a `.gitignore` entry to hide. The
  shared `/tmp/xurl-rs-mutants` holds one plain run at a time: a second plain run, from any clone, moves the first to
  `mutants.out.old` and a third deletes that, and two at once wait on `lock.json` (`OutputDir::new`,
  `src/output.rs:116-135`). No run this plan relies on is a plain run, and each run's own `<run>/out` is never reused,
  so a rerun never rotates a run's output away.
- KTD15. **A run never shares a target directory between jobs.** cargo-mutants 27.1.0 neither sets nor clears
  `CARGO_TARGET_DIR`: no file in its `src/` names it, and it only adjusts `CARGO_ENCODED_RUSTFLAGS`. With the variable
  exported, or `build.target-dir` set in a cargo config, every job would build into one directory, and one job's tests
  could run another job's mutated binaries. Each run unsets `CARGO_TARGET_DIR` and `CARGO_BUILD_TARGET_DIR` in its own
  subshell and, before it starts, checks that `cargo metadata --no-deps --format-version 1` reports `target_directory`
  as the checkout's own `target`, which catches a config file setting as well.
- KTD16. **The run procedure.** Each guardrail of KTD8 is a step. A run is one cargo-mutants invocation over one area or
  one shard of an area (a batch).
  1. **Clean start.** `git status --porcelain` prints nothing, and the run records `git rev-parse HEAD`. cargo-mutants
     copies untracked files as well (`gitignore` filtering is off by default since 25.0.2), so a dirty tree would be
     mutated as found.
  2. **Idle machine.** No `cargo`, `rustc`, or `cargo-nextest` process is running outside the run. The run holds
     `flock -n ~/dev/xurl-rs/.context/perf-measure.lock` for its whole length, the lock the spike's measurements held,
     so two runs started at once can't both pass the check. The load average and the count of other build processes are
     recorded at the start and the end. An agent waiting for an idle machine backgrounds a check every 10 seconds that
     exits when the machine is idle, bounded at 12 hours, then reports.
  3. **Disk floor.** The volume holding the run directory has at least 60 GB free, plus one build directory's peak size
     per job. The first job reuses the baseline's directory and each other job copies its own (`src/lab.rs`). Until the
     pilot measures a build directory, take 20 GB each, so `-j2` needs 100 GB. A background check every 10 seconds sends
     the run `SIGINT` if free space falls below 60 GB.
  4. **No shared target directory** (KTD15).
  5. **The run itself, in the background, under a bound.** The run directory is
     `~/.cache/xurl-rs-mutants/<area>-<short sha>-<YYYYMMDD-HHMM>/`. The command runs from the workspace root, with
     `CARGO_TARGET_DIR` and `CARGO_BUILD_TARGET_DIR` unset, as
     `TMPDIR=<run>/tmp timeout -s INT -k 300 <bound> cargo mutants --workspace -j2 -f '<area glob>' --output <run>/out`,
     with its output going to `<run>/cargo-mutants.log`. `<bound>` is 1.5 times the batch's estimate. `SIGINT` lets
     cargo-mutants stop cleanly and delete its build directories (it handles Ctrl-C, `src/interrupt.rs`). `-k 300` kills
     the group if it hasn't stopped after 5 minutes. `TMPDIR` points cargo-mutants' build directories, which it makes
     with `tempfile` in the system temp directory, at the run's own directory, so cleanup touches nothing another
     session owns.
  6. **Overnight batch size.** A batch's estimate (mutants times the measured per-mutant time, divided by the measured
     `-j` gain) is at most 6 hours, so its bound fits a night. The CLI area splits into shards with `--shard k/n` on one
     commit (KTD23).
  7. **Read the exit code.** 0, 2 (missed mutants), and 3 (timeouts) are results. 4 is a failed baseline: stop and
     diagnose. 1 and 70 are usage or internal errors: stop.
  8. **Cleanup, every time.** After a hard kill, any `<run>/tmp/cargo-mutants-*` directory left behind is emptied with
     `cargo clean --manifest-path <dir>/Cargo.toml`, because `trash` keeps the space on the same volume, then trashed.
     Then `<run>/tmp` is trashed, and no `cargo`, `rustc`, `cargo-nextest`, or test process of the run is left.
     `<run>/out` holds only logs, diffs, and JSON, and stays until every PR that reads it has merged.
  9. **Jobs.** `-j2` is the pilot's setting. Later batches use `-j3` only when the pilot's load and memory show room: 16
     CPUs and 93 GB of memory, with nextest running up to 16 test threads in each job.
- KTD17. **Every missed mutant is checked against the doctests before it counts as a gap.** nextest doesn't run
  doctests, so a bug that only a doctest catches shows as missed. Only a doctest that runs can catch a mutant that
  compiles: a `no_run` block is compiled and never run, a `compile_fail` block passes on code that fails to build, and a
  `text` block is not a test. At 492e187 four doctests run, all over `xdk::webhooks`:
  `crates/xdk/src/webhooks/budget.rs:52`, `crates/xdk/src/webhooks/checkpoint.rs:156`,
  `crates/xdk/src/webhooks/signing.rs:125`, and `crates/xdk/src/testing/webhooks.rs:21`. The other Rust blocks are 18
  `rust,no_run` (`shortcuts.rs` 4, one each in ten other `src` files, and the 4 in `crates/xdk/README.md`, which
  `crates/xdk/src/lib.rs:1` includes) and one `compile_fail` (`call.rs`); `credential_step.rs`'s and `hints.rs`'s blocks
  are `text`. Each run recounts the code fences in `crates/*/src` doc comments and in `crates/xdk/README.md` with `rg`,
  reading each fence's info string. For each missed mutant in a function a running doctest reaches, apply its diff from
  `mutants.out/diff/` in a scratch clone at the run's commit and run `cargo test --doc -p <package>`. A doctest that
  fails catches it. The mutant is then recorded as caught by that doctest, with no gap test. An area that no running
  doctest reaches skips the check and says so in its gap PR. Doctests are never cut.
- KTD18. **The log-to-matrix script is `scripts/mutants-matrix.py`**, a stdlib-only `uv run --script` with the shebang
  `scripts/generate-changelog.py` uses (`#!/usr/bin/env -S PYTHONDONTWRITEBYTECODE=1 uv run --script`), so it leaves no
  virtual environment, lock file, or bytecode in the tree. Python rather than shell, because a set cover and JSON joins
  over thousands of mutants are unreadable in `jq`.
  - **Input:** one or more `mutants.out` directories, each holding `mutants.json` (each mutant's name, file, function,
    and genre), `outcomes.json` (each scenario's summary, `log_path`, `diff_path`, and per-phase durations),
    `caught.txt`, `missed.txt`, `timeout.txt`, `unviable.txt`, and `log/`. The log directory is `log/`
    (`src/output.rs:139`), not the `logs/` the book names. The baseline log supplies the full test list. Each run's
    recorded commit comes with it. An optional keep list names tests that are never cut, each marked `credit` or
    `no-credit` (KTD19). An optional accepted list names missed struct-field mutants a gap PR accepted (KTD22), which
    list 1 reports apart. Both lists are plain text in `~/.cache/xurl-rs-mutants/lists/`, beside the runs they serve,
    and each PR that relies on them quotes the entries it adds.
  - **Parsing:** in each mutant's log, every nextest result line that ends a test unsuccessfully (`FAIL`, `TIMEOUT`, a
    signal, or whatever else the pilot's logs show) counts as a kill by that test, keyed by nextest's
    `<binary-id> <test-name>`. The pilot's logs fix the exact line shapes. The script is written from them, not from
    this list.
  - **Output**, written into the run directory: the matrix as JSON (each caught mutant mapped to its killing tests, and
    each test mapped to the mutants it kills, with its home area per KTD19), and the four lists as text:
    1. missed mutants, by file and function, with their diff paths, and apart from them the mutants that only
       `no-credit` tests catch (KTD19), which are gap candidates too;
    2. tests that catch no mutant;
    3. redundant tests: those outside the keep set KTD19 chooses, each with the kept tests that cover its catches;
    4. load-bearing tests: each test that is the only catcher of a mutant, with those mutants.
  - **Timeouts and unviable mutants:** timeout mutants are listed apart, with any tests their logs show failing before
    the kill. Unviable mutants are counted and dropped.
  - **Self-checks, which fail the script and name the mutant:** the missed list must equal `missed.txt`; every mutant in
    `caught.txt` must show at least one killing test; no missed mutant's log may show one; the baseline log must show no
    failure; every mutant log that ran tests must list as many tests as the baseline, timeouts excepted; and the runs it
    merges must not disagree on a file's content (R4).
  - **How it is checked:** against the pilot's real output, with the self-checks, and by hand on ten mutants' logs. It
    gets no test file of its own. It proposes and doesn't decide: the cut PRs' rerun (KTD20) catches a wrong redundancy
    call, and the self-checks catch a wrong missed list.
- KTD19. **The keep set is a greedy set cover, crate by crate.** A library behavior is proved at the library's public
  boundary: AGENTS.md makes `xdk` "consumable from downstream Rust crates" with `xr` as one consumer, and discovery
  class 2 keeps the strongest boundary.
  1. Seed the keep set with the keep list (golden fixture tests, wire-format tests, source-reading guards, chosen at
     triage by reading), every load-bearing test, and every gap test. A source-reading guard is kept with `no-credit`:
     its catches cover no mutant in steps 2 and 3 and make no test load-bearing in list 4. Such a guard reads the
     mutated copy's `src` as text, so it fails on a mutant that changes the text it scans whatever the behavior does:
     `crates/xurl-cli/tests/dispatch_guard.rs` reads `crates/xurl-cli/src/cli/commands/*.rs` (`source`, lines 13-16) and
     matches `Commands::<Variant>` in each group module, so a mutant that replaces a group's `run` body or deletes one
     of its arms fails it. With credit, that catch would mark the behavioral test of the same command redundant, and
     KTD20's rerun would still find the mutant caught, by the guard. `parse_path_guard.rs`
     (`crates/xurl-cli/src/cli/parse.rs`), `http_client_guard.rs` (`crates/xdk/src`), the `spec_validation.rs` tests
     that read `response/types.rs` by path, and every other test that reads a `src` file are `no-credit` too, chosen at
     triage by reading. Golden fixture and wire-format tests keep `credit`, because they fail on what the code outputs.
  2. Cover the mutants in `crates/xdk/src` with `xdk`'s own tests (its `tests/` and its unit tests), greedily: at each
     step keep the test that catches the most mutants not yet covered, with ties going to a walk (a test whose failure
     message carries a `Cause:` line), then by name for a stable order. A CLI test's catches of `xdk` mutants don't
     count toward this cover. So an `xdk` test is never cut because a CLI test also catches its mutants.
  3. Cover every remaining caught mutant, CLI and `xdk`, with any test, by the same rule.
  4. Every test outside the keep set is a redundant-test candidate (list 3). A test's home area is the area holding most
     of its catches for an `xdk` test, and the CLI area for every CLI test. A test that catches nothing takes the area
     its file tests, read at triage. Cuts are grouped into PRs by home area.
  5. An `xdk` mutant that only CLI tests catch is not a gap, since it is caught. A library test for it would add to the
     count, against the goal, so it is listed and not filled.
- KTD20. **A cut PR's rerun covers exactly the mutants a deleted test catches.** On the cut branch, run every mutant, in
  every area run so far, whose catchers include a deleted test, plus the area's timeouts, by name
  (`-F '^<escaped name>$'` for each), with `-f <file>` for each file that holds one. Every named mutant must end caught.
  Two cargo-mutants behaviors shape the check. `-F` reaches no struct-field mutant (KTD10), so the file filter keeps the
  run to the named files and their struct-field mutants come along: each of those is judged against its recorded
  outcome, and only a named mutant must end caught. A `-F` that matches nothing exits 0 having tested nothing
  (`--list -F '^no such mutant$'` exits 0), so a name that drifted (a helper deleted from `crates/xdk/src/testing/`, a
  rebase) would pass the rerun silently: before the rerun counts, the sorted names in its `outcomes.json` must include
  every expected name (`comm -13` of the two lists prints nothing), and a missing name sends the cut back to the
  per-file refresh below. Deleting a unit test moves no mutant today: in each of the 51 `src` files with `#[cfg(test)]`,
  the test module sits after every mutated line or in its own file. Deleting a test can't change the outcome of a mutant
  whose catchers were all kept: nextest runs each test in its own process, so a kept test that failed on a mutant fails
  on it again. A full rerun of the area would add only mutants that can't change. Before choosing cuts, every file whose
  content changed since its run (`git diff --quiet <run sha> <base> -- <file>`) is run again at the base, so every name
  in the matrix exists at the base (R4). A rerun mutant that ends missed means a deleted test was needed: restore one
  that caught it, rerun that mutant, and record it on the card. After a cut PR merges, its deleted tests leave the
  matrix.
- KTD21. **When a cut PR may open. Pending Brett. B is written below; A and C change only M9's gate.** A cut in one area
  can delete the only test that catches a mutant in an area not yet run, and that cut's rerun can't see it, because a
  run records catches only in the files it mutates. `xdk`'s tests reach no CLI code (`crates/xdk/Cargo.toml` has no
  dependency on `xurl-rs`). The CLI's tests reach every `xdk` area.
  - **B, as written:** gap PRs follow each area's run, and every cut PR waits for the last area, the CLI. The cuts come
    from one tree-wide matrix, list 2 means "catches no mutant anywhere", and one set cover serves all 4,448 area
    mutants.
  - **C, xurl-plan-prime's recommendation:** `xdk`'s cut PRs wait until all three `xdk` areas have run, and the CLI's
    cut PR waits for the CLI run. Under KTD19 an `xdk` test's cut depends only on `xdk`'s tests and `xdk`'s mutants, all
    of which are known once `xdk` has run, so C is as safe as B, and `xdk`'s cuts don't wait on R's source splits.
  - **A:** each area's cut PR follows its own run. Each later area's run adds a step: its missed mutants are rerun in a
    scratch clone with every earlier cut reverted, and a reverted test that catches one comes back in that area's gap
    PR.
- KTD22. **A missed mutant that changes no behavior gets an `exclude_re` entry, not an attribute.** (xurl-plan-prime.)
  The entry names the file and the function with no line number
  (`'crates/xdk/src/<file>\.rs:[0-9]+:[0-9]+: replace <function> '` and narrower when one mutant of the function is
  meant), with a TOML comment giving the reason. It adds no dependency. The attribute form
  (`#[cfg_attr(test, mutants::skip)]` with `mutants` as a dev-dependency, `attrs.md`) keeps the reason beside the code
  but adds a crate to both manifests, and switching to it needs Brett's say-so. A renamed function stops matching its
  entry, and its mutants show up as missed again on the next run, so the drift shows. A struct-field mutant cannot take
  an entry, because `exclude_re` never reaches that genre (KTD10). One that changes no behavior goes in the accepted
  list (KTD18) by file, function, and field, with its reason, and the gap PR body quotes it. The script reports it apart
  from list 1 on every later run, and an accepted entry that matches no missed mutant is reported too, so it is dropped
  once a test catches it or the field goes.
- KTD23. **The areas and their sizes,** with `-E` as in KTD10, at 492e187. "Serial" estimates 46 s per viable mutant: a
  build of about 6 s once R's merged test layout lands (the spike measured the rebuild of every test target after an
  `xdk` edit falling from 66.7 s and 56.5 s to 5.9 s and 6.0 s) plus a full suite of about 40 s (39.3 s locally after
  spike levers 1 to 3, and 1 s more with the merged layout). The pilot replaces the estimate. § "Counts" gives each
  area's glob, count, and estimate.

  - `store` and `auth` run as one area because F changes both and their tests overlap.
  - `-j2` or `-j3` cuts the wall time by somewhat less than 2 or 3 times, so the whole tree is about four to six
    overnight batches.
  - The CLI area runs as shards on one commit, each kept as its own `out` directory, and the script reads them together.
  - The counts move with R's splits and the fix PRs, and each run records its own.

- KTD24. **The pilot is the first area.** Its gap PR (M5) follows the pilot. Its cuts go through M9 like every area's.
  The pilot also measures:
  - per-mutant build and test time, from `outcomes.json`'s phase durations;
  - the unviable rate after the exclusions, and the timeout rate;
  - the peak size of one build directory, which sets the disk floor for later batches;
  - the size of the `out` directory per mutant, since every area's `out` stays until its cut PR merges (under KTD21 B,
    until the CLI's) and a caught mutant's log holds the output of every test it fails under `--no-fail-fast`;
  - load and memory at `-j2`, which decide `-j3`;
  - **the cost of `--no-fail-fast`**, outside cargo-mutants so that every cargo-mutants run still passes it: for ten
    caught mutants spread over the five `webhooks` files, apply each diff in a scratch clone at the pilot's commit and
    time `cargo nextest run --workspace --cargo-profile mutants` with and without `--no-fail-fast`, alternating (ABBA);
  - **spurious catches:** each list-4 mutant is rerun once at `-j1`. A sole catch that doesn't repeat is a flake, and it
    hides a gap. Later areas repeat this only if the pilot finds one;
  - **spurious timeouts:** each timeout is rerun alone at `-j1`. One that passes there means `-j2` slowed the suite past
    the bound, and `timeout_multiplier` goes into the config for later runs.
- KTD25. **The comment sweep and the pilot's lists.** The order note's Phase 4 sweep deletes plain `//` comment lines,
  which leaves the mutant set as it was and shifts line numbers, so names change. A sweep landing after the pilot leaves
  its gap PR unaffected, since gap tests target functions, not lines. Before `webhooks`' cut PR, KTD20's refresh reruns
  each `webhooks` file the sweep changed, at most 151 mutants, about two hours. The pilot's timing doesn't move. Every
  later area runs after the sweep has landed in its files, under the order note's first rule.
- KTD26. **PR types.** M1 is `build(mutants): ...`, M3 is `build(scripts): ...`, the gap PRs are `test(<crate>): ...`,
  and the cut PRs are `test(<crate>): retire ...`. Branches are `feat/` or `fix/`, as #482 was
  (`fix/retire-golden-fixture-copies`). None of them is user-facing, so their `## Changelog` sections stay empty, and
  the `cliff.toml` filter drops these types anyway. A missed mutant that exposes a defect is fixed in its own `fix(...)`
  PR with a changelog entry, and the area's runs wait for it (R8).

### Counts

Measured at `dev` 492e187 with cargo-mutants 27.1.0. Each command is run from the workspace root.

| Command                                                                               | Mutants |
| ------------------------------------------------------------------------------------- | ------: |
| `cargo mutants --workspace --list`                                                    |   7,196 |
| `cargo mutants --workspace --list -E 'with Call::' -E '<impl (std::)?fmt::Debug for'` |   4,473 |
| the same with `-f 'crates/xdk/src/webhooks/**'`                                       |     151 |

The areas, each with the same two `-E` flags. "Serial" is KTD23's estimate of 46 s per viable mutant.

| Area               | `-f` glob (with `-e` for the rest of `xdk`)                                                                                                                         |   Mutants | Serial, estimated |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------: | ----------------: |
| Pilot              | `crates/xdk/src/webhooks/**`                                                                                                                                        |       151 |             1.9 h |
| `store` and `auth` | `crates/xdk/src/store/**` (381), `crates/xdk/src/auth/**` (322)                                                                                                     |       703 |             9.0 h |
| Rest of `xdk`      | `crates/xdk/src/**`, `-e` store, auth, webhooks: `api` 910 (`shortcuts.rs` 422, `media.rs` 90), `error.rs` 89, `time.rs` 82, `testing` 21, `config` 16, `yaml.rs` 6 |     1,124 |            14.4 h |
| CLI                | `crates/xurl-cli/src/**`: `commands` 1,000, `output` 419, `hints.rs` 188, `skill_install` 159, `runner.rs` 113, `classify.rs` 113, the rest 478                     |     2,470 |            31.6 h |
| **Total**          |                                                                                                                                                                     | **4,448** |        **56.9 h** |

`crates/xdk-consumer-check` holds the other 25 of the 4,473 and is in no area.

### Assumptions

- R's merged test layout (R's U2, U6 of the CI speed plan) gives each crate two integration-test binaries, `tests/all/`
  and `tests/isolated/` (the environment-mutating tests in `xdk-rs` and `parse_stack_tests` in `xurl-rs`), so nextest
  names tests as `xurl-rs::all <module>::<test>` and `xurl-rs::isolated <module>::<test>`, or the layout's real binary
  names. The script keys tests by what the pilot's logs show.
- R's lever 8 removes every network access from the suite, so no mutant's run depends on anything outside the machine.
- Tests share no fixed port or path across processes: the `localhost:8080` strings in the tests are URLs that nothing
  binds, and every bind found in the tests uses port 0. A clash between two jobs would show as a flake (KTD24).
- The 16-CPU server stays the place runs happen. cargo-mutants on macOS or Windows is not planned.

### High-Level Technical Design

```text
M1 config PR ──► M2 pilot run (webhooks, 151) ──► M3 script PR (from M2's logs)
                       │                                   │
                       └────────► M4 skill decision ◄──────┘
                                         │
M5 webhooks gap PR ◄─────────────────────┘
M6 store+auth run ──► gap PR      (after F's fix PRs and the sweep)
M7 rest-of-xdk run ──► gap PR     (after #648, R's types.rs split, and the sweep)
M8 CLI run (shards) ──► gap PR    (after R's source splits and the sweep)
M9 cut PRs, one per area          (gate: KTD21) ──► R's cli_tests.rs split
```

Each run's `out` directory feeds the script. The script's matrix feeds the gap PR (list 1, after the doctest check) and,
at M9, the cut PRs (lists 2 to 4 and the set cover). Each PR's own verification goes through cargo-mutants again: the
gap PR reruns its mutants and finds them caught, and the cut PR's rerun finds none lost.

### Alternative Approaches Considered

- **`cargo test` as the test tool.** It skips no doctests, but under cargo-mutants it runs each test binary's tests in
  one process and stops at the first failing binary unless given `--no-fail-fast`. Today's 141 binaries, or R's merged
  layout, would make a log of one binary's failures in each mutant. The suite is built for nextest, which CI runs.
- **A `mutants` nextest profile.** It would only restate `default` and would have to keep `nextest_config_guard.rs`'s
  two rules. `--no-fail-fast` in the config does the one thing a profile would add.
- **`--in-place`.** It mutates the checkout itself and can't run `-j2`. The copy is the safer default, and KTD9 makes it
  pass.
- **`exclude_globs` for `shortcuts.rs`.** That would also drop its 422 viable mutants: the validators, the query
  builders, and the field lists.
- **A full area rerun for each cut PR.** It costs as much as the area's run, and the mutants it adds can't change
  outcome (KTD20).
- **Per-test coverage instead of reading** to split list 2 into "asserts too little" and "never reaches the code."
  Reading is cheap at triage. Coverage tooling would be a new harness.

### Risks & Dependencies

| Risk                                                                                                                             | Mitigation                                                                                        |
| -------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| The baseline fails in the copied tree, on a file or a state the copy lacks                                                       | M1's one-mutant run (AE2) proves the baseline before any overnight batch                          |
| Jobs share a target directory and run each other's binaries                                                                      | KTD15's unset and `cargo metadata` check                                                          |
| The disk fills: one build directory per job plus the baseline                                                                    | KTD16's floor, the 10-second watcher, `cargo clean` before `trash`, and the pilot's measured peak |
| Other slots build during a run and skew its timing or cause spurious timeouts                                                    | KTD16's idle check and lock, the recorded load, and KTD24's `-j1` rerun of timeouts               |
| A flaky test fails under a mutant and hides a real gap                                                                           | KTD24's `-j1` rerun of list-4 mutants                                                             |
| The script misreads a log                                                                                                        | KTD18's self-checks against cargo-mutants' own lists, and KTD20's rerun                           |
| `src` changes between a run and the cut that relies on it                                                                        | KTD20's per-file refresh                                                                          |
| A cut removes the only catcher of a mutant in an area not yet run                                                                | KTD21, pending Brett                                                                              |
| A library behavior ends up tested only through the CLI                                                                           | KTD19's crate-by-crate cover                                                                      |
| A test guards something no mutant models                                                                                         | the keep list (KTD7, KTD19) and triage by reading                                                 |
| A behavior only a doctest checks shows as missed                                                                                 | KTD17                                                                                             |
| Spec-refresh PRs change `shortcuts.rs` and `response/types.rs` between the rest of `xdk`'s run and its cut                       | KTD20's refresh reruns only the changed files                                                     |
| A source-reading guard catches a mutant by its text, and the cover cuts the behavioral test of the same code                     | KTD19's `no-credit` mark on every test that reads `src`                                           |
| A by-name rerun tests nothing, because a name drifted and `-F` matching nothing exits 0                                          | KTD20's check that the rerun's `outcomes.json` holds every expected name                          |
| `-F` and `-E` pass every struct-field mutant (124 at 492e187), so a rerun pulls in extra mutants and `exclude_re` can't drop one | KTD20's per-file `-f` and judging only the named set; KTD22's accepted list                       |

Dependencies: #645, #646, and #647 (`auth`, `store`, `webhooks`), and #648 (`api/media.rs`), stacked on them; R's lever
8 (R's U1) and merged test layout (R's U2) (Phase 1), its source splits (R's U5 to U8, Phase 2, `response/types.rs`
being U8) and the `hints.rs` test move (R's U9), and its `cli_tests.rs` split (R's U10, Phase 6, after M9); F's fix PRs
for `auth` and `store` (F's U5, Phase 3, step 13); the Phase 4 comment sweep.

### System-Wide Impact

- Two config additions: `.cargo/mutants.toml` and a `[profile.mutants]` table in the root `Cargo.toml`. A plain
  `cargo build` or `cargo test` never uses the profile, and CI's cache key doesn't hash the root manifest.
- One script, `scripts/mutants-matrix.py`, which nothing in CI runs.
- The test count falls by whatever M9's cuts remove. The gap PRs add tests and `exclude_re` entries.
- `AGENTS.md` and `crates/xurl-cli/README.md` don't change. Neither crate's public surface changes, and no golden
  fixture moves.
- The suite's wall time falls with the cuts, which R's closing measurement records.

### Sequencing

M1 can land as soon as it's written: it changes no test and no `src`. M2 waits for #645 to #647, R's lever 8 and merged
test layout, and M1. M3 follows M2, from its logs. M4 follows M2 and M3. M5 follows M3. M6, M7, and M8 each wait for
their gates, in that order. Each gap PR follows its run. M9's PRs wait for KTD21's gate, and the CLI's lands before R's
`cli_tests.rs` split.

---

## Implementation Units

### M1. The mutants config and cargo profile

- **Goal:** a plain `cargo mutants` from the root runs the way R1 and R2 say.
- **Requirements:** R1, R2. KTD1, KTD2, KTD9 to KTD15.
- **Dependencies:** none.
- **Files:**
  - create `.cargo/mutants.toml` with the keys in the table below and nothing else. Each key carries a short comment
    naming the trap it closes: the package-only default test scope, the fail-fast default, the missing `.git`, the
    unviable guesses, and the in-tree `mutants.out`.
  - modify the root `Cargo.toml`: a `[profile.mutants]` table with `inherits = "test"` and `debug = "none"`, and a
    comment saying cargo-mutants selects it and that it keeps `.cargo/config.toml`'s dependency opt-levels.

The keys of `.cargo/mutants.toml`:

| Key                          | Value                                             | Why         |
| ---------------------------- | ------------------------------------------------- | ----------- |
| `test_tool`                  | `"nextest"`                                       | KTD1        |
| `test_workspace`             | `true`                                            | KTD1        |
| `additional_cargo_test_args` | `["--no-fail-fast"]`                              | KTD2, KTD12 |
| `profile`                    | `"mutants"`                                       | KTD1, KTD11 |
| `copy_vcs`                   | `true`                                            | KTD9        |
| `exclude_re`                 | `["with Call::", "<impl (std::)?fmt::Debug for"]` | KTD10       |
| `output`                     | `"/tmp/xurl-rs-mutants"`                          | KTD14       |

- **Test scenarios:**
  - `cargo mutants --workspace --list | wc -l` equals the count the base gives with the same `-E` flags on the command
    line (4,473 at 492e187), and `git status --porcelain` prints nothing.
  - `cargo mutants --workspace --list -f 'crates/xdk/src/webhooks/**'` lists 151 at 492e187, or the base's count, which
    the PR body records.
  - AE3's compile-line check, against the committed table.
  - AE2's one-mutant run, under KTD16's procedure, with the output landing in `/tmp/xurl-rs-mutants/mutants.out`: the
    baseline passes in the copy, which proves KTD9, and `git status --porcelain` stays empty.
  - `nextest_config_guard` and `release_profile_guard` pass.
- **Verification:** the scenarios, quoted in the PR body. No golden fixture or test moves.

### M2. The pilot run

- **Goal:** the first matrix, on `crates/xdk/src/webhooks/`, and the costs every later batch is sized by.
- **Requirements:** R3, R6. KTD4, KTD16, KTD24.
- **Dependencies:** #645 to #647 merged; R's lever 8 and merged test layout merged; M1 merged.
- **Files:** none in the repo. Output goes in `~/.cache/xurl-rs-mutants/webhooks-<sha>-<stamp>/`.
- **Approach:**
  1. Record the base commit and `--list -f 'crates/xdk/src/webhooks/**'`'s count.
  2. Run under KTD16 at `-j2`, sized as one batch (about two hours by KTD23's estimate, so a 3-hour bound).
  3. Take KTD24's measurements: the fail-fast sample, the `-j1` reruns of list-4 mutants and of timeouts, a build
     directory's peak size, and load and memory.
  4. Report the results to the coordinating session: the outcome counts, per-mutant time, the `-j` and disk numbers, and
     what the logs' result lines look like.
- **Verification:** the run's exit code is 0, 2, or 3. Its `outcomes.json` has a baseline that passed. Cleanup left no
  build directory and no process behind.

### M3. The log-to-matrix script

- **Goal:** `scripts/mutants-matrix.py`, written from M2's logs, producing the matrix and the four lists.
- **Requirements:** R5. KTD3, KTD18, KTD19.
- **Dependencies:** M2's output.
- **Files:** create `scripts/mutants-matrix.py`, with a usage block naming its input, its output files, and its exit
  codes (0 when the lists are written, non-zero when a self-check fails, with the mutant and the check named).
- **Test scenarios:**
  - On M2's output, every self-check in KTD18 passes (AE4).
  - Ten mutants chosen across the five files and the outcomes are checked by hand: the killing tests the matrix lists
    equal the failing tests in each log.
  - A copy of M2's output with one test removed from a caught mutant's log fails the "at least one killing test" or
    "test count" check, naming that mutant.
  - Two `out` directories of the same files at different commits are refused.
  - With a keep list that marks a test `no-credit`, a mutant only that test catches leaves list 4 and shows in list 1's
    guard-only part, and a test whose only catches are shared with that test is not called redundant on its strength.
  - An accepted-list entry shows apart from list 1 while its mutant is missed, and is reported as stale once it matches
    no missed mutant.
- **Verification:** the four lists for `webhooks` go in the PR body as counts, with list 1 in full.

### M4. The skill decision

- **Goal:** decide whether a project-local skill should hold the deletion workflow.
- **Requirements:** R7. KTD5.
- **Dependencies:** M2 and M3.
- **Approach:** the pilot's slot reports which steps took judgment that would repeat across the areas: choosing
  deletions from the lists and classes, the rerun and the reading of a newly missed mutant, keeping contract tests out,
  and the guardrails. It recommends yes or no. The coordinating session takes it to Brett. A skill, if wanted, is its
  own task with its own brief, written from those steps.
- **Verification:** the decision is recorded in this plan's Review log.

### M5. `webhooks` gap PR

- **Goal:** every missed `webhooks` mutant is caught by a test at the public boundary, or excluded with its reason.
- **Requirements:** R9. KTD17, KTD22.
- **Dependencies:** M3.
- **Approach:**
  1. Run KTD17's doctest check on list 1.
  2. For each remaining missed mutant, read the function and decide:
     - **a test:** it asserts the behavior the mutant breaks, through `xdk`'s public API (`crates/xdk/tests`, or a unit
       test of a `pub` item where the integration path can't reach it). For a behavior only `xr` shows, the test goes
       through `xr`.
     - **an `exclude_re` entry:** the mutant changes nothing a caller can observe, such as a capacity hint or a check
       the next line repeats.
     - **a `fix` PR:** the mutant shows the code is wrong.
  3. Each new test is seen failing on its mutant's diff (AE6): apply the diff in a scratch clone, run the test, and
     quote the failure.
  4. Rerun the list-1 mutants on the branch by name, with `-f` for their files, under KTD20's two checks: the rerun's
     `outcomes.json` names every expected mutant, and only the named ones are judged. Each ends caught, excluded, or,
     for a struct-field mutant, in the accepted list (KTD22).
- **Verification:** the rerun's summary line, the quoted failures, and the test count before and after
  (`cargo nextest list --workspace`), in the PR body.

### M6. `store` and `auth`: run and gap PR

- **Requirements:** R4, R8, R9. KTD6, KTD16, KTD23.
- **Dependencies:** F's fix PRs for `auth` and `store` (F's U5, Phase 3, step 13) merged; #645 and #646 merged; the
  Phase 4 sweep landed in `crates/xdk/src/store/` and `crates/xdk/src/auth/`; M3.
- **Approach:** run under KTD16, with `-f 'crates/xdk/src/store/**' -f 'crates/xdk/src/auth/**'`, in one or two batches,
  then the gap PR as M5 does.
- **Verification:** as M2 and M5.

### M7. The rest of `xdk`: run and gap PR

- **Requirements:** R4, R8, R9. KTD6, KTD16, KTD23.
- **Dependencies:** #648 merged (`api/media.rs`); R's `response/types.rs` split merged (R's U8, Phase 2, step 4); the
  sweep landed in the area's files; M3.
- **Approach:** run under KTD16 with
  `-f 'crates/xdk/src/**' -e 'crates/xdk/src/store/**' -e 'crates/xdk/src/auth/**' -e 'crates/xdk/src/webhooks/**'`, in
  two or three batches, then the gap PR as M5 does. The gap PR also carries the property test for
  `crates/xdk/src/api/url_authority.rs`, which has none. `split_special_url` is the WHATWG authority split that keeps a
  URL's userinfo out of `value` and the dry-run `url` fields, so a failure here leaks a credential. With proptest,
  already a dev-dependency of both crates:
  - for any input, `head + authority + tail` equals the input after the split's preprocessing: C0 control and space
    trimmed from both ends, every tab and newline removed, and, for a special scheme, `\` read as `/` where the split
    reads it so;
  - for a special scheme, no `@` from a userinfo ends up in `head` or `tail`, so a caller that drops the userinfo from
    `authority` drops every part it could be.

  The test is seen failing on a mutant of `split_special_url` before it's pushed.
- **Verification:** as M2 and M5.

### M8. The CLI: run and gap PR

- **Requirements:** R4, R8, R9. KTD6, KTD16, KTD23.
- **Dependencies:** R's source splits merged (R's U5 to U7, Phase 2, steps 1 to 3, and the `hints.rs` test move, R's
  U9); the sweep landed in `crates/xurl-cli/src/`; M3.
- **Approach:** run under KTD16 with `-f 'crates/xurl-cli/src/**'`, as `--shard k/n` batches on one commit, each sized
  to KTD16's 6 hours. The gap PR follows, as M5 does, with tests through `xr` (the spawned binary or the in-process
  run).
- **Verification:** as M2 and M5. Every shard's `out` directory has the same commit and the same `--list` count.

### M9. Cut PRs, one per area

- **Goal:** delete every test the matrix shows to be redundant or inert, without losing a caught mutant.
- **Requirements:** R4, R10, R11. KTD7, KTD19 to KTD21.
- **Dependencies (the gate, pending Brett, KTD21):**
  - as written (B): M8's run, and every gap PR merged;
  - under C: `xdk`'s cut PRs after M7's run, and the CLI's after M8's;
  - under A: each area's cut PR after its own run, and M6 to M8 each add the reverted-cuts rerun of their missed
    mutants.

  Under every option, the CLI's cut PR lands before R's `cli_tests.rs` split.
- **Approach:**
  1. Refresh the matrix for every file changed since its run (KTD20).
  2. Run the script with the keep list. Take list 3 and the deletable part of list 2 as candidates, grouped by home area
     (KTD19), and add discovery classes 1 and 2 by reading: a hand-written test a walk covers, and the same contract at
     several layers. Read each candidate against its covering tests: a test that pins a value no kept test checks, or
     that guards something no mutant models, joins the keep list.
  3. Fill each candidate's retirement card:
     - `detects`: the mutants it catches (count and names) or "none";
     - `stronger_proof`: the kept tests that catch each of them, or, for an inert test, why it catches nothing (it never
       reaches the mutated code, or it asserts nothing about what it runs);
     - `history`: `git log --follow --format='%h %s' -- <file>` for its file, and why it was written;
     - `validation`: the rerun's summary and the suite's summary line.
  4. Delete the tests, and any helper only they used, in one commit per test file.
  5. Run KTD20's rerun on the branch. Its `outcomes.json` names every expected mutant, and every named mutant ends
     caught (AE5). On a miss, restore a test that caught it, rerun, and record it on the card.
- **Verification:** the cards, the rerun's summary and its name check, the test count before and after,
  `cargo nextest run --workspace` and `cargo test --doc --workspace` green, and no golden fixture, wire-format test, or
  source-reading guard touched.

---

## Verification Contract

| Gate                  | Command                                                                                                         | Applies to                |
| --------------------- | --------------------------------------------------------------------------------------------------------------- | ------------------------- |
| Mutant list           | `cargo mutants --workspace --list` (with `-f` per area), counted, and `git status --porcelain` empty afterwards | M1, every run             |
| Profile               | `cargo build -p anstyle-parse --profile mutants -v`, read for `-C opt-level=3` and `-C strip=debuginfo`         | M1                        |
| Baseline in the copy  | AE2's one-mutant run                                                                                            | M1                        |
| Run procedure         | KTD16's steps, each logged with its result                                                                      | M2, M6 to M8, every rerun |
| Script self-checks    | `scripts/mutants-matrix.py` over the run's `out` directories exits 0                                            | M3 onwards                |
| Doctests              | KTD17's check on every list-1 mutant                                                                            | M5 to M8                  |
| Red first             | each gap test seen failing on its mutant's diff, the failure quoted                                             | M5 to M8                  |
| No caught mutant lost | KTD20's rerun ends with every rerun mutant caught                                                               | M9                        |
| Tests                 | `cargo nextest run --workspace`, read from its summary line, and `cargo test --doc --workspace`                 | M1, M3, M5 to M9          |
| Lint and format       | `cargo clippy -- -D warnings`, `cargo fmt --check`                                                              | M5 to M9                  |
| Local CI mirror       | `scripts/hooks/pre-push`                                                                                        | before each push          |
| Test count            | `cargo nextest list --workspace`, before and after                                                              | M5 to M9                  |

No run sends a request to X. Every run happens on the Linux server, never in CI.

---

## Definition of Done

- `.cargo/mutants.toml` and `[profile.mutants]` are on `dev`, and AE1 to AE3 hold on a fresh clone.
- Every area has run, and its `out` directories are kept until its cut PR merges.
- `scripts/mutants-matrix.py` is on `dev`, and its self-checks pass on every run.
- M4's decision is recorded.
- Each area's gap PR has merged. Every missed mutant is caught by a test seen failing on it, caught by a doctest,
  excluded with its reason, or fixed in a `fix` PR.
- The `url_authority.rs` property test is on `dev`.
- Each area's cut PR has merged with complete retirement cards and a rerun that lost no caught mutant, before R's
  `cli_tests.rs` split.
- No golden fixture, wire-format test, or source-reading guard was cut.
- The test count before M5 and after M9 is recorded, with the number of tests each PR removed and added.
- Every run directory's `tmp` is gone, every `out` directory has been trashed once no open PR needs it, and no process
  of any run is left.

---

## Engineering Review

`/plan-eng-review`, one round, run unattended at ae60d46 (the plan's `src` equals 492e187's): no question went to
anyone, and each choice below is the review's recommended option, recorded in the section it changes. Evidence is of
three kinds and labeled as such: a static read of this clone, `cargo mutants --list` probes with cargo-mutants 27.1.0
(nothing built, `git status --porcelain` empty afterwards), and cargo-mutants' `src/` at tag `v27.1.0` read from GitHub.

**Scope.** Scope accepted as-is. The work spans more than eight files across its PRs, so the structure question ran: the
original arrangement, one PR per unit and per area, is kept, since a smaller one would merge PRs the area gates keep
apart. No feature is cut or deferred.

**Findings.**

| ID  | Severity | Section                                              | Finding                                                                                                                                                                                                       | Disposition                                                                                                                                                                           |
| --- | -------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| E1  | P1       | KTD10, KTD20, KTD22, AE2, M5                         | `-F` and `-E` never reach a struct-field mutant (`src/visit.rs:747`), so a by-name rerun pulls extra mutants and fails on any that were missed, `exclude_re` can't drop one, and AE2 would test two mutants   | Fixed: KTD10 records the gap and its probe; KTD20 and M5 rerun with `-f` per file and judge only the named set; KTD22 adds the accepted list; AE2 uses `signing.rs`, which holds none |
| E2  | P1       | KTD20, M9                                            | A `-F` that matches nothing exits 0 having tested nothing, so a rerun whose names drifted passes silently                                                                                                     | Fixed: the rerun's `outcomes.json` must hold every expected name before it counts                                                                                                     |
| E3  | P1       | KTD19, KTD18, M3                                     | A source-reading guard fails on a mutant's text: `dispatch_guard.rs` reads `commands/*.rs`. Kept with credit, it would let the cover cut the behavioral test of the same code, and the rerun would still pass | Fixed: such tests are kept `no-credit`; mutants only they catch are listed as gap candidates; two M3 scenarios cover it                                                               |
| E4  | P2       | KTD17                                                | Four doctests run; the others are `no_run`, `compile_fail`, or `text` and can't catch a viable mutant, so a per-file check over "about 25" spends runs on nothing                                             | Fixed: the check covers running doctests only, recounted at each run                                                                                                                  |
| E5  | P2       | Sources, Assumptions, Risks & Dependencies, M6 to M8 | Plans R and F are on `dev`, and R's layout has two binaries per crate (`all`, `isolated`)                                                                                                                     | Fixed: citations name R's U1, U2, U5 to U10 and F's U5; the assumption names both binaries                                                                                            |
| E6  | P3       | Problem Frame                                        | "142 test files ... assert `next_step`" and "107 tests" don't re-derive: 27 files and 90 functions in `crates/*/tests/*.rs` outside the walks and guards                                                      | Fixed, with the counting command                                                                                                                                                      |
| E7  | P3       | KTD24                                                | `out` directories stay until their cut PRs merge, and their size is never measured                                                                                                                            | Fixed: the pilot measures it                                                                                                                                                          |
| E8  | P3       | KTD14                                                | Plain runs share `/tmp/xurl-rs-mutants`, and a third plain run deletes the first's output                                                                                                                     | Fixed: KTD14 says so and why no run of the plan is affected                                                                                                                           |
| E9  | P3       | KTD10                                                | "`--list` builds nothing (23 s, 83 MB)": a warm `gtime cargo mutants --workspace --list` took 0.72 s and 25 MB here, so 23 s is likely a cold figure                                                          | Left as written; the figure gates nothing                                                                                                                                             |

**Architecture.** No new module, service, or public surface: the plan adds two config tables and a leaf script nothing
calls, so the design-twice check doesn't apply. Boundaries hold: nothing changes either crate's surface, CI, or a golden
fixture. E3 is the one coupling defect: the keep list mixed tests kept for reasons mutants can't model with tests whose
catches prove behavior.

**Code quality.** E1, E2, and E8 are defects in how the plan drives cargo-mutants, all from behavior its book doesn't
state. The script stays one file with one job, and the new lists are inputs it already had the shape for.

**Tests.** The plan's own proof is the self-checks, the hand check of ten logs, the rerun, and red-first gap tests.
Coverage of the plan's paths:

```text
CODE PATHS                                              STATUS
.cargo/mutants.toml + [profile.mutants] (M1)
  ├── list count equals the -E count                    [★★★ TESTED] M1 scenario 1, AE1
  ├── profile keeps dependency opt-levels               [★★★ TESTED] AE3
  └── baseline passes in the copy (copy_vcs)            [★★★ TESTED] AE2, one mutant
run procedure (KTD16)
  ├── exit 0/2/3 results, 4 stop, 1/70 stop             [★★ TESTED] M2 verification
  └── cleanup after a hard kill                         [★★ TESTED] M2 verification
scripts/mutants-matrix.py (M3)
  ├── log parse → killing tests                         [★★★ TESTED] self-checks + 10 by hand
  ├── a dropped kill or a short log                     [★★★ TESTED] corrupted copy scenario
  ├── mixed commits refused                             [★★★ TESTED] M3 scenario
  ├── no-credit keep entries (E3)                       [★★★ TESTED] M3 scenario
  └── accepted list, stale entry (E1)                   [★★★ TESTED] M3 scenario
gap PR (M5 to M8)
  ├── gap test red on its mutant                        [★★★ TESTED] AE6
  ├── doctest catch                                     [★★ TESTED] KTD17 check
  └── rerun: every named mutant caught                  [★★★ TESTED] M5 step 4 + name check (E2)
cut PR (M9)
  ├── rerun of mutants a deleted test caught            [★★★ TESTED] AE5 + name check (E2)
  └── a guard's textual catch hiding a lost test        [★★★ TESTED] no-credit (E3)

COVERAGE: 14/14 paths with a ★★ or ★★★ check after the fixes; 2 were gaps before (E2, E3)
```

The script keeps KTD18's decision to have no test file: its proof is its self-checks over real output and the rerun
downstream, and the two new scenarios run against a copy of M2's output like the existing ones. No test is made obsolete
by this plan's own changes; the cuts are M9's.

**Performance.** One finding, E7. The cost model (46 s per viable mutant, 56.9 h serial) is an estimate the pilot
replaces. A by-name rerun without `-f` would add every struct-field mutant in the workspace, 124 × about 46 s, about 1.6
h; E1's per-file `-f` keeps that to the named files'.

**Failure modes.**

| Path          | Realistic failure                                                   | Covered by                              | Seen or silent                                    |
| ------------- | ------------------------------------------------------------------- | --------------------------------------- | ------------------------------------------------- |
| Rerun by name | Names drift and `-F` tests nothing                                  | E2's name check                         | loud after the fix; silent before                 |
| Set cover     | A guard's text catch covers a mutant, so the behavioral test is cut | E3's `no-credit`                        | loud after the fix (gap candidate); silent before |
| Rerun by name | A struct-field mutant rides along and was missed in the matrix      | E1: judged against its recorded outcome | loud                                              |
| Plain run     | A third plain run deletes the first's output                        | KTD14; no plan run is plain             | loud (missing directory)                          |
| Run           | Disk fills mid-run                                                  | KTD16's watcher and floor               | loud                                              |
| Matrix        | A flaky sole catcher hides a gap                                    | KTD24's `-j1` rerun                     | loud                                              |

Two critical gaps flagged (E2, E3), both closed in the plan.

**What already exists.** cargo-mutants' own outcome files are the script's cross-check, nextest's per-test process
isolation is what makes KTD20's partial rerun sound, and the walks (`dry_run_guard`, `credential_step_walk`,
`rerun_walk`, `validation_value_guard`, `post_id_guard`, `help_claims_guard`) are what discovery class 1 reads against.
The gstack `test-audit` retirement card is the card M9 fills. Nothing here is rebuilt.

**Not in scope.** Reporting the struct-field filter gap to cargo-mutants upstream: it is outward-facing, and the
workaround needs no upstream change. The `#[mutants::skip]` attribute, which would reach struct-field mutants: KTD22
keeps it Brett's call. A test file for the script: KTD18 decides against one.

**Parallelization.** Sequential implementation, no parallelization opportunity: every run needs the idle machine (KTD16
step 2), and each later unit reads the one before it. Gap and cut PRs are written while no run holds the machine.

**Reality rows.**

- REALITY: premise-table pass 31 claims checked, 3 corrected, unverified=2 (the list cost in KTD10 and the per-mutant
  estimate in KTD23) docs/plans/2026-11-27-0033-test-mutation-testing-and-test-cuts-plan.md:72
- REALITY: already-done pass `origin/dev` is three docs-only commits past ae60d46, and #645 to #648 are open
  docs/plans/2026-11-27-0033-test-mutation-testing-and-test-cuts-plan.md:557
- REALITY: surface-check finding every flag and key exists in `cargo mutants --help` and `--emit-schema=config`, and
  `-F`/`-E` skip struct-field mutants docs/plans/2026-11-27-0033-test-mutation-testing-and-test-cuts-plan.md:249
- REALITY: binding-decisions pass KTD1 to KTD8, KTD9, KTD10, KTD21, and KTD22 stand unedited
  docs/plans/2026-11-27-0033-test-mutation-testing-and-test-cuts-plan.md:186
- REALITY: numbers finding the mutant counts re-derive exactly, and the Problem Frame's file and test counts are
  corrected docs/plans/2026-11-27-0033-test-mutation-testing-and-test-cuts-plan.md:481
- REALITY: deferred-asks pass no linked issue docs/plans/2026-11-27-0033-test-mutation-testing-and-test-cuts-plan.md:156
- REALITY: pr-count pass one PR per unit and area as Sequencing states, and this review adds none
  docs/plans/2026-11-27-0033-test-mutation-testing-and-test-cuts-plan.md:583

**Outside voice.** Not run: the run's instructions forbid agents and subagents, and the Codex CLI and the native
fallback are both. No outside findings.

**Decisions for Brett.** KTD21 stays pending as written. This review's view, not a decision: C holds, because KTD19's
crate-by-crate cover, with E3's `no-credit` rule, makes an `xdk` test's cut depend only on `xdk`'s tests and mutants.

---

## Review log

- 2026-11-27: plan written. Pending: KTD21 (Brett); `/plan-eng-review`, `/plan-devex-review`,
  `/compound-engineering:ce-doc-review`.
- 2026-11-27: `/plan-eng-review` round 1 started, on Opus, as its own process from a clone of the commit that adds this
  line.
- 2026-11-27: `/plan-eng-review` round 1 ended (`claude-opus-5-5` only, 971 s): 3 P1, 2 P2, 4 P3 (E1 to E9 in §
  Engineering Review), each folded into the plan but E9, which gates nothing. E1's struct-field gap was confirmed
  against cargo-mutants' `src/visit.rs` and by `--list -E 'delete field'`, E4's doctest count by the fence info strings,
  and E5's unit IDs against plans R and F on `dev`. Pending Brett: KTD21, and whether to report the struct-field filter
  gap to cargo-mutants upstream. Next: `/plan-devex-review`.
- 2026-11-27: `/plan-devex-review` round 1 started, on Opus, as its own process from a clone of the commit that adds
  this line, with the persona of a developer or unattended agent who runs the config, the run procedure, and the script.

## GSTACK REVIEW REPORT

| Review         | Trigger               | Why                             | Runs | Status      | Findings                                                    |
| -------------- | --------------------- | ------------------------------- | ---- | ----------- | ----------------------------------------------------------- |
| CEO Review     | `/plan-ceo-review`    | Scope & strategy                | 0    | —           | —                                                           |
| Outside Review | codex (skipped)       | Independent 2nd opinion         | 0    | skipped     | not run: the run's instructions forbid agents and subagents |
| Eng Review     | `/plan-eng-review`    | Architecture & tests (required) | 1    | ISSUES OPEN | 7 issues, 2 critical gaps; all folded into the plan         |
| Design Review  | `/plan-design-review` | UI/UX gaps                      | 0    | —           | no UI                                                       |
| DX Review      | `/plan-devex-review`  | Developer experience gaps       | 0    | —           | —                                                           |

The Eng row counts the four review sections (E1 to E5, E7, E8). E6 and E9 are reality-row findings outside that count.
Both critical gaps, E2 and E3, are closed in the plan.

**OUTSIDE COVERAGE:** codex, plan-review phase, skipped: the run allowed no agent or subagent. No outside findings.

**VERDICT:** Eng review done, with every finding folded into the plan. Not CLEAR by the dashboard's rule, which clears
only a review that found nothing: eng review required.

**UNRESOLVED DECISIONS:**

- KTD21 (Brett): when a cut PR may open, A, B (as written), or C (xurl-plan-prime's recommendation). It changes only
  M9's gate.
