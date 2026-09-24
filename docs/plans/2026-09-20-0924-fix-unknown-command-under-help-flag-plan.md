---
title: Unknown Command Under the Help Flag - Plan
type: fix
date: 2026-09-20
status: completed
implementation: U1 merged to dev as #221 (fdeddf5) on 2026-09-23 with the #222-#226 follow-ups in stack #227; unreleased
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
---

# Unknown Command Under the Help Flag - Plan

## Goal Capsule

- **Objective:** A caller who misspells a command and asks for its help learns the command does not exist, instead of
  being handed the root help page and a success exit code. An agent reading the exit code learns the same thing.
- **Means:** On the help-display path, parse argv a second time through a command whose root help flag is inert, and run
  the existing classifier on the result. Only a word that classifies as `UnknownCommand` diverges; every other
  invocation keeps clap's display.
- **Authority:** `classify` stays post-parse and pure. The runner keeps ownership of exit codes and rendering.
- **Execution profile:** One visible change. `xr <unknown> --help` and `xr <unknown> -h` move from root help at exit 0
  to an `unknown-command` error at exit 2, matching what the same word does without the help flag.
- **Stop conditions:** Stop and ask if suppressing the help flag cannot be done without changing the parse surface for
  any invocation that parses today, or if the re-parse cannot be kept off the success path. Neither fired: the second
  parse uses its own command, so the primary parser is untouched, and it runs only on the error path.

## Product Contract

### Problem Frame

An unknown bare command exited 2 with reason `unknown-command` in every shape except one. With `-h` or `--help` present
it printed the root help and exited 0, reporting no error at all.

| Invocation                  | Before U1                                                       | Now                                                      |
| --------------------------- | --------------------------------------------------------------- | -------------------------------------------------------- |
| `xr webhooks`               | exit 2, `unknown command 'webhooks'. Did you mean 'bookmarks'?` | exit 2, same error                                       |
| `xr help webhooks`          | exit 2, same error                                              | exit 2, same error                                       |
| `xr --output json webhooks` | exit 2, `unknown-command` envelope                              | exit 2, `unknown-command` envelope                       |
| `xr auth statsu --help`     | exit 2, `unknown command 'statsu'. Did you mean 'status'?`      | exit 2, same error                                       |
| `xr webhooks --help`        | exit 0, root help                                               | exit 2, `unknown command 'webhooks'` with its suggestion |
| `xr webhooks -h`            | exit 0, root short help                                         | exit 2, the same error as `--help`                       |

**Root cause.** `Cli` carries a root-level `url: Option<String>` positional for raw mode. An unknown word binds to it,
so clap raises no parse error; the help flag then returns `Err(DisplayHelp)`, and `render_parse_error`
(`crates/xurl-cli/src/cli/runner.rs`) wrote that display to stdout at `EXIT_SUCCESS` before `classify` ran. `classify`
takes a parsed `&Cli`, and on this path no `Cli` exists.

Nested families were correct for the opposite reason. `auth` declares no positional, so an unknown word there is a real
clap `InvalidSubcommand` error, which `render_parse_error` routes to `render_unknown_command`. The defect was root-level
only.

`git`, `cargo`, and `gh` all error on `<unknown> --help`.

### Requirements

- **R1.** `xr <word> --help` and `xr <word> -h`, where `<word>` classifies as `UnknownCommand`, emit the same error,
  reason, suggestion, and exit code as `xr <word>`.
- **R2.** The error honors structured output. `xr --output json webhooks --help` emits the `unknown-command` envelope.
  The format comes from `structured_intent`, as on every clap-error path, because no parsed `Cli` exists there; the
  no-help form reads it from the parsed `Cli` through `effective_output()`.
- **R3.** Every invocation that prints help without a mistyped word still prints the same help at exit 0: bare `xr
  --help`, `xr -h`, `xr <known-command> --help`, `xr <known-command> -h`, and every nested `--help` page.
- **R4.** A URL-shaped positional keeps its behavior. `xr /2/users/me --help` prints root help at exit 0, because there
  is no per-URL help page to show.
- **R5.** A raw-only flag keeps its behavior. `xr -X POST webhooks --help` is a raw request, not a typo, and prints root
  help.
- **R6.** No new public `xdk::Error` variant, and no change to the `unknown-command` envelope's shape.

### Success Criteria

- The six rows in the Problem Frame table read `2` in the exit column.
- The cross-mode tables in `crates/xurl-cli/tests/unknown_command_tests.rs` cover the help-flag axis.
- `KNOWN_DIFFERENCES.md` describes the behavior the binary has.

### Scope Boundaries

**In scope.** Root level, both `-h` and `--help`, text and every structured format.

**Out of scope for U1.**

- Nested families, which were already correct.
- Bare `xr` printing root help at exit 0. That is R8 of `docs/plans/2026-09-09-1528-fix-pre-attention-cleanup-plan.md`
  and deliberate.
- `--version`. U1 leaves the `DisplayVersion` short-circuit alone; #226 routes it through the same second parse.
- Changing help content, which belongs to `docs/plans/2026-09-20-0924-refactor-scope-help-to-consumed-flags-plan.md`.

## Planning Contract

### Key Technical Decisions

**KTD1. Parse again with the help flag inert; do not scan argv.** On a help-display error, parse argv a second time
through `Cli::command()` with the help flag replaced by an inert one, and run `classify` on the resulting `Cli`.
Classification stays post-parse, which is the invariant `classify`'s module doc names: clap has already consumed `help`,
`--`, and every value-taking flag, so the classifier never re-implements tokenizing.

The second command is `Cli::command().disable_help_flag(true)` plus an `ArgAction::Count` argument with the same
spellings, `-h` and `--help`, which parses and does nothing. Three clap 4.6 facts fix that shape:

- Disabling the flag alone makes `--help` and `-h` unknown arguments (`ErrorKind::UnknownArgument`), so the second parse
  would never succeed on the invocations it exists for.
- `mut_arg("help", …)` panics, because clap adds the help flag while building the command and the argument does not
  exist beforehand.
- The `help` subcommand stays enabled. `disable_help_subcommand(true)` binds the word `help` to the positional, and
  plain `xr help` then classifies as an unknown command, which R3 forbids.

`disable_help_flag` is a global setting, so a help flag after a subcommand is unknown to the second parse; that parse
fails and clap's own help prints, which is R3's answer for a real command.

Stripping `-h` and `--help` tokens from argv before re-parsing is rejected. It cannot distinguish a help flag from a
positional value that reads as one (`xr post -- -h`), which is the pre-parse pitfall the module was written to avoid.

**KTD2. Diverge only on a successful re-parse that classifies as `UnknownCommand`.** Any other outcome falls through to
clap's display. If the re-parse itself errors, print the original help. The change is strictly additive: the only
invocations that move are ones that printed root help and would otherwise have been an error.

**KTD3. The re-parse runs only on the error path.** `run_with_overrides` parses once on the success path and is
untouched. A second parse costs nothing on any invocation that does real work.

**KTD4. Reuse `render_unknown_command`.** The renderer, the jaro suggestion, the reason string, and the exit code all
exist. U1 adds a route into them, not a second rendering.

**KTD5. `DisplayHelpOnMissingArgumentOrSubcommand` takes the same route.** It reaches the same short-circuit and can
carry the same typo. The re-parse fails for a genuine missing-subcommand case, which KTD2 routes back to help.

### High-Level Technical Design

```text
run_with_overrides
  └─ Cli::try_parse_from
       ├─ Ok(cli) ─────────────────► classify(&cli) ─► dispatch                 (unchanged)
       └─ Err(e) ─► render_parse_error
                     ├─ DisplayHelp | DisplayVersion | DisplayHelpOnMissing…
                     │     └─ parse_without_display_flags(args)            (crates/xurl-cli/src/cli/reparse.rs)
                     │           ├─ Ok(cli) & classify == UnknownCommand(w)
                     │           │     └─ render_unknown_command(w, …)          exit 2
                     │           └─ otherwise ─────────► clap's display, exit 0
                     ├─ InvalidSubcommand ─► render_unknown_command             (a flag spelling: unexpected argument)
                     └─ other ─────────────► the `invalid-args` rendering, exit 2
```

U1 shipped this route for the two help kinds, with the helper in `runner.rs`. On `dev` the helper lives in
`crates/xurl-cli/src/cli/reparse.rs` (#223) and makes the version flag inert as well, which routes `DisplayVersion`
through it (#226).

### Sequencing

One unit, one PR (#221). The test unit and the code change land together because the test is the proof. Five follow-up
PRs from a developer-experience review of #221 stack on it, outside this plan's units; see Reconciliation.

## Implementation Units

### U1. Classify before the help flag renders

**Goal.** R1 through R6.

**Files.**

- `crates/xurl-cli/src/cli/runner.rs`: the hidden-word branch in `render_parse_error`, and the re-parse helper.
- `crates/xurl-cli/tests/unknown_command_tests.rs`: the help-flag axis on both cross-mode tables, the help-flag test,
  and the table of displays that must stay clap's own.
- `crates/xurl-cli/tests/golden_tests.rs` and `crates/xurl-cli/tests/golden/text-unknown-command-help-flag.golden`: one
  text case for `xr webhooks --help`. The golden harness covers the path through `text_cases()`.
- `KNOWN_DIFFERENCES.md`: the help-flag behavior, and root help for a raw request.

`crates/xurl-cli/src/cli/classify.rs` is untouched: the helper builds a command and parses, so it is not pure and stays
out of the classifier.

**Approach.**

1. A helper takes `&[OsString]`, builds the command per KTD1, parses, and returns `Option<Cli>`, `None` on any parse
   error.
2. In `render_parse_error`, before the display short-circuit, the helper runs for the two help-display kinds. On
   `Some(cli)` whose `classify` is `UnknownCommand(word)`, the suggestion comes from `nearest_command(&word)` and the
   word goes to `render_unknown_command`. Otherwise the display prints.
3. The hidden-word branch renders with the provisional `OutputConfig` that `render_parse_error` already builds from
   `structured_intent`, the config the `InvalidSubcommand` branch uses.

**Test scenarios.** Each new one observed failing against the unchanged runner first: 34 failed and 57 passed, every
failure a help-flag case at exit 0 with the root help where exit 2 was expected.

- `xr webhooks --help` and `xr webhooks -h` exit 2 and render exactly as `xr webhooks` does, reason `unknown-command`,
  `command` `webhooks`, suggestion `bookmarks`.
- `xr whoam --help` suggests `whoami`; `xr zzzzzz --help` exits 2 with no suggestion.
- The help-flag axis runs across text, `json`, `jsonl`, `ndjson`, `yaml`, `csv`, and `tsv`, both in process and through
  the spawned binary for `XURL_OUTPUT`.
- `xr --help`, `xr -h`, `xr help`, `xr help whoami`, `xr whoami --help`, `xr whoami -h`, `xr auth apps --help`, and `xr
  auth` exit 0 with clap's own rendering; so do `xr /2/users/me --help` and `xr -X POST webhooks --help`.
- `xr --version` exits 0, unchanged.

**Verification.** `cargo test` green; the golden suite green with one added fixture and no modified one.

## Verification Contract

| Gate                  | Command                                     | Result for U1                    |
| --------------------- | ------------------------------------------- | -------------------------------- |
| Format                | `cargo fmt -- --check`                      | No diff                          |
| Lint                  | `cargo clippy --all-targets -- -D warnings` | Clean                            |
| Tests                 | `cargo test`                                | 1223 passing, 0 failing          |
| Golden                | `cargo test --test golden_tests`            | One added fixture, zero modified |
| Output discipline     | `bash scripts/lint-stdio.sh`                | Clean                            |
| Completions freshness | `./scripts/generate-completions.sh --check` | Fresh; the arg tree is unchanged |
| Schema freshness      | `cargo test --test schema_tests`            | Drift test passes; no new reason |
| Markdown              | `markdownlint-cli2 KNOWN_DIFFERENCES.md`    | Zero issues                      |

## Definition of Done

- Every scenario in U1 passes, and the new ones were observed failing against unchanged code, with the failure output
  quoted in the #221 PR body.
- No golden help fixture changed.
- `KNOWN_DIFFERENCES.md` states that a help flag does not change the unknown-command outcome.
- The #221 PR body's `## Changelog (xurl-rs)` names the visible change under `### Fixed`.

All four hold.

## Reconciliation

(against `xurl-rs` `origin/dev` @ `123d401`, 2026-09-23)

U1 landed as #221. A developer-experience review of #221 found five more parse-error defects, fixed as #222-#226 and
stacked on it; all six landed on `dev` through one atomic `gh stack merge` of stack #227, six squash commits. None is
released.

| Unit / PR | Branch                                           | Commit    | Change                                                                                                           |
| --------- | ------------------------------------------------ | --------- | ---------------------------------------------------------------------------------------------------------------- |
| U1 / #221 | `fix/unknown-command-under-help-flag`            | `fdeddf5` | `xr <typo> --help` and `-h` report the unknown command.                                                          |
| #222      | `fix/parse-errors-02-flag-shaped-words`          | `2eb16d7` | `xr help --help` prints the help command's page; a flag spelling where a command goes is an unexpected argument. |
| #223      | `fix/parse-errors-03-color-flag`                 | `d337673` | `--color` and `XURL_COLOR` reach every parse-error rendering; the second parses move to `cli/reparse.rs`.        |
| #224      | `fix/parse-errors-04-one-error-dialect`          | `312a263` | clap's errors read `Error: … Try '<command> --help'.`; the pointer names the suggested command's help and `xr`.  |
| #225      | `feat/parse-errors-05-unknown-command-next-step` | `0810c41` | The `unknown-command` envelope carries `next_step` `show-help`, the page the text names.                         |
| #226      | `fix/parse-errors-06-version-after-a-typo`       | `123d401` | `xr <typo> --version` and `-V` report the unknown command.                                                       |

The follow-ups change U1's visible text: the pointer after a suggestion names that command's help (`xr webhooks --help`
ends `Try 'xr bookmarks --help'.`), so `text-unknown-command-help-flag.golden` carries that line, and the envelope gains
`next_step`. R1 still holds, since the help-flag form renders exactly as the bare word does.

Code review ran as the `ce-code-review` lite path for U1 (receipt `20260923-163052-bad7058d`, Ready to merge) and
for #222-#226 (receipt `20260923-171023-e4484a69`, Ready to merge); the helper's `hard_block_full` floor was not honored
because the full spine dispatches subagents. Vale and unslop (score 0) ran on every PR body; LanguageTool was
unreachable and skipped, as `RELEASES.md` allows. The pattern is recorded in the solutions corpus at
`design-patterns/unknown-command-needs-two-detectors-one-renderer-and-the-scorer-that-saw-the-word.md`.
