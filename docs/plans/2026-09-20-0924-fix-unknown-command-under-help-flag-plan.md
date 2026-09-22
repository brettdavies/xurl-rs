---
title: Unknown Command Under the Help Flag - Plan
type: fix
date: 2026-09-20
status: implementation-ready
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
---

# Unknown Command Under the Help Flag - Plan

## Goal Capsule

- **Objective:** A caller who misspells a command and asks for its help learns the command does not exist, instead of
  being handed the root help page and a success exit code. An agent reading the exit code learns the same thing.
- **Means:** On the help-display path, re-parse with clap's help flag disabled and run the existing classifier on the
  result. Only a word that classifies as `UnknownCommand` diverges; every other invocation keeps today's behavior.
- **Authority:** `classify` stays post-parse and pure. The runner keeps ownership of exit codes and rendering.
- **Execution profile:** One visible change. `xr <unknown> --help` and `xr <unknown> -h` move from root help at exit 0
  to an `unknown-command` error at exit 2, matching what the same word already does without the help flag.
- **Stop conditions:** Stop and ask if suppressing the help flag cannot be done without changing the parse surface for
  any invocation that parses today, or if the re-parse cannot be kept off the success path.

## Product Contract

### Problem Frame

An unknown bare command exits 2 with reason `unknown-command` in every shape except one. With `-h` or `--help` present
it prints the root help and exits 0, reporting no error at all.

| Invocation                  | Exit | Behavior                                                |
| --------------------------- | ---- | ------------------------------------------------------- |
| `xr webhooks`               | 2    | `unknown command 'webhooks'. Did you mean 'bookmarks'?` |
| `xr help webhooks`          | 2    | same                                                    |
| `xr --output json webhooks` | 2    | `unknown-command` envelope                              |
| `xr auth statsu --help`     | 2    | `unknown command 'statsu'. Did you mean 'status'?`      |
| `xr webhooks --help`        | 0    | root help                                               |
| `xr webhooks -h`            | 0    | root short help                                         |

**Root cause.** `Cli` carries a root-level `url: Option<String>` positional for raw mode. An unknown word binds to it,
so clap raises no parse error; the help flag then returns `Err(DisplayHelp)`, and `render_parse_error`
(`crates/xurl-cli/src/cli/runner.rs:349`) short-circuits to stdout at `EXIT_SUCCESS` before `classify` ever runs.
`classify` takes a parsed `&Cli`, and on this path no `Cli` exists.

Nested families are correct for the opposite reason. `auth` declares no positional, so an unknown word there is a real
clap `InvalidSubcommand` error, which `render_parse_error` already routes to `render_unknown_command`
(`runner.rs:371-376`). The defect is root-level only.

`git`, `cargo`, and `gh` all error on `<unknown> --help`.

### Requirements

- **R1.** `xr <word> --help` and `xr <word> -h`, where `<word>` classifies as `UnknownCommand`, emit the same error,
  reason, suggestion, and exit code as `xr <word>` does today.
- **R2.** The error honors structured output. `xr --output json webhooks --help` emits the `unknown-command` envelope,
  through the same `structured_intent` path the no-help form uses.
- **R3.** Every invocation that prints help today still prints the same help at exit 0: bare `xr --help`, `xr -h`, `xr
  <known-command> --help`, `xr <known-command> -h`, and every nested `--help` page.
- **R4.** A URL-shaped positional keeps its current behavior. `xr /2/users/me --help` prints root help at exit 0,
  because there is no per-URL help page to show.
- **R5.** A raw-only flag keeps its current behavior. `xr -X POST webhooks --help` is a raw request, not a typo, and
  prints root help.
- **R6.** No new public `xdk::Error` variant, and no change to the `unknown-command` envelope's shape.

### Success Criteria

- The six rows in the Problem Frame table read `2` in the exit column, with the last two changed and the rest unmoved.
- The cross-mode table in `crates/xurl-cli/tests/unknown_command_tests.rs` covers the help-flag axis.
- `KNOWN_DIFFERENCES.md` describes the behavior the binary actually has.

### Scope Boundaries

**In scope.** Root level, both `-h` and `--help`, text and every structured format.

**Out of scope.**

- Nested families, which are already correct.
- Bare `xr` printing root help at exit 0. That is R8 of `docs/plans/2026-09-09-1528-fix-pre-attention-cleanup-plan.md`
  and deliberate.
- `--version`, which stays on the `DisplayVersion` short-circuit untouched.
- Changing help content, which belongs to `docs/plans/2026-09-20-0924-refactor-scope-help-to-consumed-flags-plan.md`.

## Planning Contract

### Key Technical Decisions

**KTD1. Re-parse with the help flag disabled; do not scan argv.** On a help-display error, rebuild the command with
`Cli::command().disable_help_flag(true).disable_help_subcommand(true)`, parse argv through it, and run `classify` on the
resulting `Cli`. Classification stays post-parse, which is the invariant `classify`'s module doc names: clap has already
consumed `help`, `--`, and every value-taking flag, so the classifier never re-implements tokenizing.

The alternative, stripping `-h` and `--help` tokens from argv before re-parsing, is rejected. It cannot distinguish a
help flag from a positional value that happens to read as one (`xr post -- -h`), which is exactly the pre-parse pitfall
the module was written to avoid.

**KTD2. Diverge only on a successful re-parse that classifies as `UnknownCommand`.** Any other outcome falls through to
today's behavior. If the re-parse itself errors, print the original help. This makes the change strictly additive: the
only invocations that can move are ones that today print root help and would otherwise have been an error.

**KTD3. The re-parse runs only on the error path.** `run_with_overrides` parses once on the success path and is
untouched. A second parse costs nothing on any invocation that does real work.

**KTD4. Reuse `render_unknown_command`.** The renderer, the jaro suggestion, the reason string, and the exit code all
exist. This unit adds a route into them, not a second rendering.

**KTD5. `DisplayHelpOnMissingArgumentOrSubcommand` takes the same route.** It reaches the same short-circuit and can
carry the same typo. The re-parse will fail for a genuine missing-subcommand case, which KTD2 already routes back to
help.

### High-Level Technical Design

```text
run_with_overrides
  └─ Cli::try_parse_from
       ├─ Ok(cli) ─────────────────► classify(&cli) ─► dispatch          (unchanged)
       └─ Err(e) ─► render_parse_error
                     ├─ DisplayVersion ───────────────► stdout, exit 0   (unchanged)
                     ├─ DisplayHelp | DisplayHelpOnMissing…
                     │     └─ NEW: reparse_without_help(args)
                     │           ├─ Ok(cli) & classify == UnknownCommand(w)
                     │           │     └─ render_unknown_command(w, …)   exit 2
                     │           └─ otherwise ─────────► stdout, exit 0  (unchanged)
                     ├─ InvalidSubcommand ─► render_unknown_command      (unchanged)
                     └─ other ─────────────► envelope or clap text, 2    (unchanged)
```

### Sequencing

One unit, one PR. The test unit and the code change land together because the test is the proof.

## Implementation Units

### U1. Classify before the help flag renders

**Goal.** R1 through R6.

**Files.**

- `crates/xurl-cli/src/cli/runner.rs`: the new branch inside `render_parse_error`, plus the re-parse helper.
- `crates/xurl-cli/src/cli/classify.rs`: only if the helper belongs beside `classify`; prefer the runner, since the
  helper builds a command and is not pure.
- `crates/xurl-cli/tests/unknown_command_tests.rs`: the cross-mode table gains the help-flag axis.
- `KNOWN_DIFFERENCES.md`: correct the overstated sentence.
- `crates/xurl-cli/tests/golden/`: a fixture for the new output, if the golden harness covers this path.

**Approach.**

1. Add a helper that takes `&[OsString]`, builds the command with the help flag and help subcommand disabled, parses,
   and returns `Option<Cli>`. `None` on any parse error.
2. In `render_parse_error`, before the existing help short-circuit, run the helper for the two help-display kinds. On
   `Some(cli)` whose `classify` is `UnknownCommand(word)`, take the suggestion from `nearest_command(&word)` and return
   `render_unknown_command`. Otherwise fall through unchanged.
3. The provisional `OutputConfig` built at `runner.rs:362` already precedes the `InvalidSubcommand` branch; order the
   new branch so it uses the same config and the same `structured_intent`.

**Test scenarios.** Each observed failing against unchanged code first.

- `xr webhooks --help` exits 2, reason `unknown-command`, `command` is `webhooks`, suggestion `bookmarks`.
- `xr webhooks -h` matches the long-flag case exactly.
- `xr whoam --help` suggests `whoami`.
- `xr zzzzzz --help` exits 2 with no suggestion.
- `xr --output json webhooks --help` emits the envelope; the same for `jsonl`, `yaml`, `csv`, `tsv`.
- `xr --help`, `xr -h`, `xr whoami --help`, `xr whoami -h`, `xr auth apps --help` all still exit 0 with unchanged bytes.
  The golden fixtures are the assertion.
- `xr /2/users/me --help` exits 0 with root help.
- `xr -X POST webhooks --help` exits 0 with root help.
- `xr --version` exits 0, unchanged.

**Verification.** `cargo test`, plus the golden suite green with exactly one added fixture and no modified ones.

## Verification Contract

| Gate                  | Command                                     | Done signal                      |
| --------------------- | ------------------------------------------- | -------------------------------- |
| Format                | `cargo fmt -- --check`                      | No diff                          |
| Lint                  | `cargo clippy --all-targets -- -D warnings` | Clean                            |
| Tests                 | `cargo test`                                | All pass, new scenarios included |
| Golden                | `cargo test --test golden_tests`            | One added fixture, zero modified |
| Output discipline     | `bash scripts/lint-stdio.sh`                | Clean                            |
| Completions freshness | `./scripts/generate-completions.sh --check` | Fresh; the arg tree is unchanged |
| Schema freshness      | `cargo test --test schema_tests`            | Drift test passes; no new reason |
| Markdown              | `markdownlint-cli2 KNOWN_DIFFERENCES.md`    | Zero issues                      |

## Definition of Done

- Every scenario in U1 passes, and the two new ones were observed failing against unchanged code, with the failure
  output quoted in the PR body.
- No golden help fixture changed. A modified one means R3 regressed.
- `KNOWN_DIFFERENCES.md` no longer claims the positional is classified before anything is sent without qualification.
- The PR body's `## Changelog (xurl-rs)` names the visible change under `### Fixed`.
