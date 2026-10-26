---
title: Dry-Run Text Summary - Plan
type: feat
date: 2026-10-26
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
---

# Dry-Run Text Summary - Plan

**Target repo:** brettdavies/xurl-rs. Every path is in it. **Base:** `dev`.

## Goal Capsule

- **Objective:** A person who runs `xr --dry-run <command>` in text output can tell from the answer alone that nothing
  was sent, whether the real run would succeed, and, when it would not, why and with what exit code.
- **Means:** a text renderer for the dry-run answer in the binary's output layer, built from the envelope's own keys
  (KTD1, KTD2).
- **Authority:** R-IDs win on behavior, KTDs on mechanism, and a unit overrides neither. `AGENTS.md` § "Where a change
  goes" places all wording in the binary.
- **Execution profile:** inline in one session, no subagents or worktrees. One PR to `dev`.
- **Stop conditions:** a change would move a byte of the json, jsonl, yaml, csv, or tsv answer (R4); a change would make
  a dry run exit non-zero or send a request; a change would need the library to return prose.

## Product Contract

### Summary

In text output, `--dry-run` prints a short summary instead of the command's context as raw JSON: a line saying nothing
was sent, whether the real run would succeed, the exit code and reason when it would not, the credential the request
would go out under, and the command's inputs. The structured formats keep #323's envelope unchanged.

### Problem Frame

`OutputConfig::print_dry_run` in `crates/xurl-cli/src/cli/output/mod.rs` passes the command's context straight to
`print_response` when the format is not structured. A person reading the default text output therefore sees the context
keys (`command`, the inputs, `auth`) as pretty-printed JSON and never sees `status`, `would_succeed`, or `exit_code`;
`crates/xurl-cli/tests/golden/dry-run-text.golden` pins exactly that for `xr --dry-run whoami`, and
`crates/xurl-cli/README.md` § "Dry Runs" documents it ("Text output prints the context without `status`,
`would_succeed`, and `exit_code`"). A refused dry run is worse: its answer looks like a successful one, because the
refusal's `reason` is just one more JSON key. Text output is the human surface, so the one question a dry run exists to
answer is the one the human cannot read off it.

### Requirements

**What the summary says**

- R1. In text output, every dry-run answer states that nothing was sent and whether the real run would succeed.
- R2. When the real run would not succeed, the summary states the exit code it would end on and the reason: the
  validator's code for an input refusal, or the error's message and the recovery step its text error would print for a
  credential refusal. Under `--quiet` the recovery step is left out, as it is from a text error.
- R3. The summary shows, each on its own line when the envelope carries it: the command and its inputs, the credential
  (scheme, app, login, and whether the token has expired), and that the real run would stop to ask for confirmation.

**What stays the same**

- R4. json, jsonl, yaml, csv, and tsv answers stay byte-identical to today's envelope, and every dry run still exits 0
  and writes its answer to stdout.

### Key Decisions

- **Text output gets a human summary.** (session-settled: user-directed — chosen over keeping #323's text output, which
  prints the context as JSON: a person could not see whether the real run would succeed.) Governs R1, R2, R3.
- **#323's dry-run contract holds for structured output.** The envelope, its keys, and exit 0 are what agents branch on.
  Governs R4.

### Scope Boundaries

- No change to which commands answer a dry run, what they check offline, or what the envelope carries.
- No color or styling beyond what `--no-color` and `NO_COLOR` already govern; the summary is plain lines.
- `--quiet` keeps the summary: it is the command's answer, not advice (R2 owns the recovery step).
- Considered and not built: a per-command sentence ("would post 'Hello' as @alice"). It would need a template per
  command and drift from the envelope; evidence that the generic lines read badly for a specific command would change
  the call.

## Planning Contract

### Key Technical Decisions

- KTD1. **The renderer lives in the binary's output layer, in a file of its own.**
  `crates/xurl-cli/src/cli/output/mod.rs` is 974 lines, so the text summary goes in a new
  `crates/xurl-cli/src/cli/output/dry_run.rs` that `print_dry_run` calls from its non-structured branch. The library is
  untouched: `AGENTS.md` keeps every sentence in the binary.
- KTD2. **The summary is built from the envelope's keys, not per-command templates.** `print_dry_run` already receives
  `would_succeed`, `exit_code`, and the context that `DryRun::answer` and `print_dry_run_refusal` build. Known keys
  (`command`, `reason`, `message`, `auth`, `confirmation_required`) get their own lines. Every other context key prints
  as an input line: a scalar as it is, an array joined with commas (an empty one skipped, such as `post`'s `media_ids`),
  and any other object as compact JSON. A new command's dry run therefore needs no text work.
- KTD3. **A credential refusal's recovery step comes from the same hint chain the real run uses.** The runner's error
  path in `crates/xurl-cli/src/cli/runner.rs` picks a text error's hint from a chain that reads the store snapshot and
  the invocation (`choose_hint`, `mismatch_hint`, and the rest, ending in `general_hint`). That chain moves into one
  function in `crates/xurl-cli/src/cli/hints.rs`, called by both the runner and the dry-run refusal, so the two cannot
  name different next commands. The structured branch keeps dropping `next_step`, as the README documents.
- KTD4. **The answer stays on stdout at exit 0.** The would-be exit code is printed, not returned, as #323 settled for
  every format.

### High-Level Technical Design

Directional sketch of the text answer, not exact wording:

```text
Dry run of `bookmark-folders add`: nothing was sent.
The real run would fail with exit code 1: folder-name-too-long
  name: A name that runs past the limit
```

```text
Dry run of `delete`: nothing was sent.
The real run would succeed.
  post_id: 1234567890
  credential: oauth2, app myapp, login alice (token current)
  asks for confirmation first; --force skips it
```

```text
Dry run of `whoami`: nothing was sent.
The real run would fail with exit code 77: <the error's message>
<the recovery line the real run's text error prints, such as the register-app step on an empty store>
```

## Implementation Units

### U1. Text renderer for the dry-run answer

- **Goal:** text-mode dry runs print the summary of R1 to R3 instead of the bare context.
- **Requirements:** R1, R2, R3, R4; KTD1, KTD2, KTD3, KTD4.
- **Dependencies:** none.
- **Files:**
  - create `crates/xurl-cli/src/cli/output/dry_run.rs` (renderer and its unit tests)
  - modify `crates/xurl-cli/src/cli/output/mod.rs` (`print_dry_run` and `print_dry_run_refusal` call it in text mode)
  - modify `crates/xurl-cli/src/cli/hints.rs` and `crates/xurl-cli/src/cli/runner.rs` (the shared hint chain, KTD3)
  - modify `crates/xurl-cli/src/cli/commands/mod.rs` (`DryRun::answer` reaches the store snapshot and invocation the
    chain reads)
  - modify `crates/xurl-cli/tests/golden_tests.rs` (text-mode cases)
  - modify `crates/xurl-cli/tests/golden/dry-run-text.golden`; create the new text-mode fixtures
- **Approach:**
  1. Add the renderer over `would_succeed`, `exit_code`, and the context; keep the structured branch of `print_dry_run`
     as it is.
  2. Move the runner's hint chain into one `hints` function (KTD3) and switch the runner's error path to it, with no
     change to any error's output.
  3. Give the dry-run refusal what that function reads; in text mode pass the hint's lines to the renderer. How the
     snapshot and invocation reach `DryRun::answer` is the implementer's choice.
  4. Re-bless `dry-run-text.golden`, the one fixture whose content R1 moves.
- **Patterns to follow:** `print_error_with_hint` for hint lines and `--quiet`;
  `crates/xurl-cli/tests/golden/text-*.golden` for text-mode fixtures.
- **Test scenarios:**
  - `xr --dry-run whoami` against a signed-in OAuth2 store prints "nothing was sent", "would succeed", and the
    credential line with app, login, and token state.
  - `xr --dry-run bookmark-folders add ""` prints that the real run would fail with exit code 1 and `empty-folder-name`,
    and the `name` input line.
  - `xr --dry-run whoami` against an empty store prints the would-be exit code 77, the refusal message, and the same
    recovery line the real run's text error prints.
  - `xr --quiet --dry-run whoami` against an empty store prints the refusal without the recovery line.
  - `xr --dry-run delete 1234567890` prints the confirmation line; with `--force` it does not, because the real run
    would not ask.
  - `xr --dry-run post Hello` prints a `body` input line and no line for the empty `media_ids`.
  - `xr --dry-run calls scheduled add --start 2026-11-02T16:00:00Z --invite 783214 --invite 2244994945` prints the
    invitee ids on one line.
  - `xr --dry-run webhooks replay 1146654567674912769 --from 202601150000 --to 202601151200` prints the window as input
    lines.
  - Every existing `text-*.golden` error fixture holds after the hint chain moves.
  - `xr --output json --dry-run post Hello` is byte-identical to its current fixture (every existing `dry-run-*` json
    fixture holds).
- **Verification:** the only golden fixture that moves is `dry-run-text.golden`; the new text fixtures read as the
  sketches above; `dry_run_guard` passes unchanged.

### U2. Document the text summary

- **Goal:** the README and AGENTS.md describe what a text-mode dry run prints.
- **Requirements:** R1, R4.
- **Dependencies:** U1.
- **Files:** modify `crates/xurl-cli/README.md` (§ "Dry Runs") and `AGENTS.md` (§ "Output formats").
- **Approach:** replace the README's "Text output prints the context without ..." sentence with a description and one
  text example; add one sentence to AGENTS.md's `--dry-run` paragraph saying text output summarizes the same facts.
- **Test expectation:** none -- documentation only; `markdownlint` and the recipe guard cover it.
- **Verification:** both documents describe the behavior U1's fixtures pin.

## Verification Contract

| Gate | Command | Proves |
| ---- | ------- | ------ |
| Text fixtures | `cargo test -p xurl-rs --test golden_tests` | R1 to R3 read as pinned; R4's json fixtures hold |
| Dry-run rule | `cargo test -p xurl-rs --test dry_run_guard` | nothing is sent or stored under the flag |
| Full suite | `cargo test --workspace --no-fail-fast` | no other output moved |
| Lint | `cargo clippy --workspace --all-targets --features xdk-rs/testing -- -D warnings` | clean |
| Local CI mirror | `scripts/hooks/pre-push` | the gates CI runs |

Re-bless only `dry-run-text.golden` and the new text fixtures, with `XURL_GOLDEN_BLESS=1`.

## Definition of Done

- R1 to R4 hold, each pinned by a golden fixture or a unit test in `crates/xurl-cli/src/cli/output/dry_run.rs`.
- `git diff` shows no structured-format fixture moving.
- The PR's `## Changelog (xurl-rs)` files the change under Changed: text output only, no contract key moves.
- No code from abandoned approaches is left in the diff.
