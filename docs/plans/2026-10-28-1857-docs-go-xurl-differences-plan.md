---
title: Documented Differences from Go xurl 1.3.4 - Plan
type: docs
date: 2026-10-28
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
---

# Documented Differences from Go xurl 1.3.4 - Plan

**Target repo:** brettdavies/xurl-rs. Every path is in it. **Base:** `dev`. **Issue:** #423.

## Goal Capsule

- **Objective:** Someone moving a script, a skill, or an agent between Go `xurl` 1.3.4 and `xr` can read, before running
  either tool, every difference that changes what the script sends, what it reads back, or how it fails, and every such
  claim holds when both binaries are run.
- **Means:** `KNOWN_DIFFERENCES.md` owns each difference; the comparison table in `crates/xurl-cli/README.md` §
  "Relationship to xurl" and a short paragraph in `README.md` summarize and link to it (KTD1).
- **Authority:** R-IDs win on what the docs say, KTDs on how they say it, and a unit overrides neither. The binaries win
  over the issue text: Go `xurl` at `18dcb447f090667171ff23666a05d6d387e2aa73` (tag `v1.3.4`) and `xr` built at the PR's
  base (KD1).
- **Execution profile:** inline in one session, no subagents or worktrees. Two stacked PRs, the bottom one to `dev`.
  Docs only: no code, test, fixture, help text, or golden file changes.
- **Stop conditions:** a claim can be neither observed offline nor cited from the Go source at the pinned commit; a
  wording would need a command or flag to change name or behavior; the `Go parity` job's `GO_XURL_REV` moves off
  `18dcb447` before the PRs merge (re-verify every Go claim against the new pin first); a sibling slot's merged change
  alters a behavior a section describes (re-run that section's check).
- **Ships:** the stack opens with every check green; Brett merges.

---

## Product Contract

### Summary

`KNOWN_DIFFERENCES.md` names Go `xurl` 1.3.4 as its comparison point and gains a map from Go's command names to `xr`'s,
the Go commands `xr` lacks, and a section for each behavior difference #423 names that holds against 1.3.4. Two existing
passages get corrected: the exit-code section's account of how Go picks an exit code, and the hand-kept list of
`xr`-only commands, which becomes a rule. The CLI README's comparison table gains a row per difference, with binary
sizes and command counts measured from both tools. The root README gains a short paragraph naming the differences most
likely to break a migrated script.

### Problem Frame

`KNOWN_DIFFERENCES.md` describes eight deliberate divergences but omits several that a migrating caller hits first:
`xurl posts USER` names a different command in `xr`, Go reports API errors on stdout, and Go deletes without asking. The
issue was drafted from a side-by-side sandbox run, and reproduction against both binaries shows the docs and the issue
each carry claims the binaries do not back:

- The doc's list of `xr` commands with no Go counterpart omits at least eleven: `usage`, `usage credits`, `muted`,
  `blocked`, `media alt-text`, `media subtitles`, `broadcasts moderators`, `lists members`, `schema`, `validate`,
  `examples`, and `skill`. The issue noticed seven of them. A hand list goes stale with each new family.
- The exit-code section says Go "maps HTTP errors to exit codes by string-matching the error message body". At 1.3.4
  every `os.Exit` in the Go source passes `1`, and `main.go` exits 1 on any Cobra error. The section's table already
  shows 1; its narrative does not match.
- The CLI README gives Go's binary as ~15 MB. The 1.3.4 Linux x86_64 release binary is 25.8 MB.
- The issue's counts ("29 endpoints", "101 commands across 26 families"), its claim that Go takes secrets "only on
  argv", and its pairing of `--force` with `--no-interactive` do not match either binary. The Appendix holds each
  observation.

### Requirements

**Comparison point**

- R1. `KNOWN_DIFFERENCES.md` names Go `xurl` 1.3.4 as the version every claim holds against, and says it is the release
  the `Go parity` CI job runs.

**Command surface**

- R2. A reader running `xurl posts USER` learns that `xr user-posts USER` is the equivalent, and that `xr posts` reads 1
  to 100 posts by id or URL, so `xr posts jack` asks X for a post whose id is `jack`.
- R3. Every Go 1.3.4 command whose `xr` name differs is mapped to its `xr` name: `posts` to `user-posts`,
  `auth app-only` and its alias `auth bearer` to `auth app`, `completion` to `completions`, `webhook start` to
  `webhooks listen`. Typing the Go name in `xr` exits 2 with reason `unknown-command`.
- R4. The Go commands with no `xr` counterpart are named: `chat` (end-to-end encrypted XChat), `mcp` (a stdio bridge to
  X's MCP server), and `token` (prints the active OAuth2 access token).
- R5. The doc tells a reader which `xr` commands have no Go counterpart by a rule that stays true as families are added:
  every `xr` command outside R3's map and Go's matching names (KD2).

**Behavior**

- R6. Go sends its first positional word as the request path and drops every word after it (`xurl lists list` sends
  `GET /lists`; `xurl likers <id>` sends `GET /likers`).
- R7. Go writes an API error's body to stdout at exit 1, and a flag error to both streams; `xr` keeps stdout for the
  command's answer and writes every failure to stderr, as text or as an envelope carrying `reason`, `exit_code`, and
  `next_step` where a step exists, with `retry_after_secs` and `retry_at` on a 429 that names its reset.
- R8. The exit-code section states that Go exits 1 on every failure, with its table unchanged.
- R9. Go refuses `--dry-run` as an unknown flag and sends nothing; `xr` answers `--dry-run` on every command, raw
  requests included, offline and at exit 0.
- R10. Go runs `delete`, `auth clear`, and `auth apps remove` at once and has no `--force`; `xr` asks first on a
  terminal, refuses with `confirmation-required` (exit 1) where it cannot ask, and proceeds under `--force`.
- R11. Go picks a credential by kind (OAuth2, then OAuth1, then bearer) whatever the endpoint accepts; `xr` checks the
  endpoint's accepted schemes before sending and refuses a mismatch at exit 2, reason `auth-method-mismatch`.
- R12. Go's list commands take `-n/--max-results` for one page and no page token; `xr`'s take the same `-n` plus
  `--cursor` (alias `--after`), which carries X's `pagination_token`, and `--page` refuses with
  `unsupported-pagination`.
- R13. Go requests `tweet.fields` and prints X's body as sent; `xr` shortcuts request the spec's `post.fields` and print
  legacy post-vocabulary keys under the spec's names, while a raw `xr` request prints X's keys as sent.
- R14. Go takes the bearer token as an argument, through `--bearer-token`, or from stdin as `-`, and takes the client
  secret and the OAuth1 secrets only as flag values or, for the client secret, `CLIENT_SECRET`; `xr` gives each of those
  secrets a `--<name>-file` flag that reads a path or stdin (`-`).

**Summaries**

- R15. The CLI README's comparison table carries a row for each difference in R2 to R4 and R6 to R14 that has a two-cell
  shape, and for the stored caller id, binary size, and command count; each cell is short and the detail lives in
  `KNOWN_DIFFERENCES.md`.
- R16. The root README carries a paragraph of at most five lines naming the differences most likely to break a migrated
  script (R2, R7, R9, R10) and linking `KNOWN_DIFFERENCES.md` and the CLI README's table.
- R17. All three files describe present behavior with no history narration, and every test that reads them keeps
  passing.

### Success Criteria

- Each claim in the three files reproduces offline against Go `xurl` at `18dcb447` and `xr` at the PR's base, by the
  method in the Verification Contract.
- A reader holding a Go `xurl` script can find each command it calls in R3's map or among Go's matching names without
  reading `xr --help`.

### Key Decisions

- KD1. **The binaries win over the issue.** Where the issue and an observation disagree, the docs say what the binary
  does, and the Appendix records the disagreement. Governs R2 to R14.
- KD2. **A rule replaces the `xr`-only list.** Go 1.3.4's command set is fixed at the pin, so mapping it to `xr` names
  is a list that changes only when the pin does; `xr`'s set grows with each family, and the hand list has already fallen
  eleven commands behind. `crates/xurl-cli/tests/recipe_guard.rs` holds the same rule for shortcut function lists ("a
  hand-maintained registry that goes stale the day a shortcut is added"). It also removes the subcommand names #421
  renames (`broadcasts scheduled live`, `broadcasts sources rename`, `calls scheduled cancel`) from this file. Governs
  R5.
- KD3. **Counts appear once, in the CLI README table, counted the same way on both sides.** Both are runnable commands
  in the `--help` tree, `help` excluded. Governs R15.
- KD4. **The root README stays a pointer.** Five lines at most, no table, no command list. Governs R16.

### Scope Boundaries

- No change to code, tests, fixtures, help text, completions, or golden files.
- `KNOWN_DIFFERENCES.md` keeps every existing section heading, since `crates/xurl-cli/tests/conformance/test_cases.toml`
  and the README link to them by text or anchor.
- `xr`-only features with no Go behavior to contrast stay out of `KNOWN_DIFFERENCES.md`, such as
  `webhooks listen --dry-run` listing a `-H` header's name without its value.
- The docs name no commit hash; the plan records the commits the evidence came from.
- Considered and not built: a test holding the README's `xr` command count to the clap tree. The brief makes this change
  docs only. A stale count after the next family lands is the evidence that would change the call; it is listed below
  for Brett.
- Considered and not built: committing the evidence harness. It runs Go `xurl` from a scratch `GOPATH` and a Python
  mock, and `Go parity` already runs the differential suite in CI.

#### Deferred to Follow-Up Work

Each item below is a defect or gap reproduction surfaced outside this change's scope, sent to release for triage.

- A word before a real command word is dropped: `xr zzz whoami` runs `whoami`, and `xr chat conversations` prints the
  `conversations` help at exit 0 in text output. `classify` in `crates/xurl-cli/src/cli/classify.rs` returns `Raw`
  whenever a subcommand matched, without reading the root positional.
- `xr auth bearer` suggests `clear` and `xr auth app-only` suggests `apps`, where Go's word maps to `xr auth app`.
- `xr posts jack --dry-run` answers `would_succeed: true`; the real run sends `GET /2/tweets?ids=jack`.
- A shortcut prints `0` for a metric X did not send (`bookmark_count` and `impression_count` on a `read` whose body
  carried neither).
- A 401 from the API answers `reason: auth-required` with no `next_step`.
- `crates/xurl-cli/tests/conformance/test_cases.toml` line 300's `skip_reason` repeats the string-matching account R8
  corrects.
- A guard holding the README's `xr` command count to `common::command_paths()`, if Brett widens the scope past docs.

---

## Planning Contract

### Key Technical Decisions

- KTD1. **One owner per difference.** Each difference is stated in full once, in a `KNOWN_DIFFERENCES.md` section; a
  README row is a short projection of it. The README table links the file, not each section, so a heading edit cannot
  break a dozen anchors.
- KTD2. **New sections follow the existing shape.** A heading `## <claim> (intentional improvement)` or
  `(intentional difference)`; a paragraph on Go citing its source file at the pin (`cli/root.go`, `api/client.go`); a
  paragraph on `xr`; a Go-versus-`xr` table when three or more invocations differ. Observations live in this plan, not
  the doc.
- KTD3. **Migration order.** The new command-names section goes first, since a migrated script fails there before
  anything else. New behavior sections follow the existing ones they extend: error streams after "Exit code mapping for
  HTTP errors"; the others after "The caller's id is asked for once per login", in R order.
- KTD4. **The unknown-word drop extends the existing section.** R6 adds two rows to the table under "A bare word is a
  command, not an endpoint" and one sentence; no new section. The new wording never claims `xr` refuses every stray
  word, since the leading-word defect above exists.
- KTD5. **Counting method.** Walk each binary's `--help` tree recursively, count every leaf plus a group that runs on
  its own (`xr usage`), count Cobra's `completion` once, and exclude `help`. At `18dcb447` Go has 60, 29 of them
  shortcuts; at `b50850d` `xr` has 160. Re-run the walk on the PR's base before opening U2, since `dev` moves.
- KTD6. **Binary sizes come from the release archives.** Unpacked Linux x86_64 binaries: Go 1.3.4 is 25,828,104 bytes
  (the build that bundles `chat`; its arm64 build, without `chat`, is 12,517,560), and `xr` 4.4.0 is 12,566,016. The
  table says "~26 MB" and "~13 MB" and names the platform. The release profile has not changed since `v4.4.0`.
- KTD7. **The brief's docs-only directive stands after one challenge.** A count guard would keep R15's `xr` count true,
  but adding a test widens the change past the brief; the guard goes to Brett as a follow-up item.
- KTD8. **The guards on the READMEs shape the wording.** Root `README.md` names no three of the shortcut function names
  `recipe_guard` counts (`repost`, `unrepost`, `bookmark`, `unbookmark` collide with command words). No README line puts
  a secret flag before a value (`agentic_tests::secret_on_argv` matches `--client-secret` followed by a space and a
  non-dash character).

### High-Level Technical Design

`KNOWN_DIFFERENCES.md` after U1, in reading order. Existing headings are unchanged; new ones are marked.

```text
# Known Differences from Go xurl
  intro: compared against Go xurl 1.3.4, the release `Go parity` runs                      (R1)
## Command names that differ, and commands only one tool has           [new]                (R2-R5)
## Webhooks: no built-in tunnel, ...                                     [xr-only list removed] (R5)
## Exit code mapping for HTTP errors                                     [narrative corrected] (R8)
## API errors go to stderr, never stdout                                 [new]                (R7)
## A bare word is a command, not an endpoint                             [two rows added]     (R6)
## A raw-only flag before a command word is refused
## Every stream the spec declares is streamed
## OAuth1 signatures use RFC 5849 percent-encoding
## A wait on media processing has a deadline
## A waited `media upload` answers one document
## The caller's id is asked for once per login
## `--dry-run` checks any command offline                                [new]                (R9)
## A command that destroys something asks first                          [new]                (R10)
## A credential the endpoint does not accept is refused before sending   [new]                (R11)
## List commands take a page token                                       [new]                (R12)
## Shortcut output uses the spec's post vocabulary                       [new]                (R13)
## Every secret can come from a file or stdin                            [new]                (R14)
```

New heading wording is directional; the implementer may tighten it while keeping each claim.

### Assumptions

- The CLI README table can grow to about twenty rows without a restructure; markdownlint does not check table line
  length (`.markdownlint-cli2.yaml`, `MD013.tables: false`).
- `xr` 4.4.1, cut from `dev` while this work is open, changes none of the behaviors R6 to R14 describe; the PR-base
  re-check confirms it.
- Brett accepts KD2's rule in place of completing the list. The issue asks for the list; the rule tells the same reader
  the same thing, and the PR body names the choice so he can reverse it in review.

### Cross-Slot Dependencies

| Slot (issues)                                 | Shared surface                                                | Effect on this plan                                                                                              |
| --------------------------------------------- | ------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `issues-verb-alignment` (#421)                | subcommand names under `broadcasts`, `calls`, `conversations` | KD2 drops those names from `KNOWN_DIFFERENCES.md`; no collision left                                             |
| `issues-help-accuracy` (#425)                 | `--cursor` and `--limit` help text                            | R12 names no command list and points at each command's `--help`                                                  |
| `issues-dry-run-envelopes` (#419, #420, #422) | dry-run envelope keys                                         | R9 names only `would_succeed` and exit 0, which no sibling plan changes                                          |
| `issues-schema-ergonomics` (#424, #426)       | `crates/xurl-cli/README.md`                                   | Different sections; textual conflicts resolve by hunk                                                            |
| spec refresh (#409)                           | vendored spec                                                 | Adds no command, so KTD5's count holds; R11's scheme check reads the spec, so re-run R11's check after it merges |

---

## Implementation Units

### U1. Compare against Go xurl 1.3.4 in KNOWN_DIFFERENCES.md

- **Goal:** `KNOWN_DIFFERENCES.md` carries R1 to R14 in the order KTD3 sets.
- **Requirements:** R1 to R14, R17.
- **Dependencies:** none.
- **Files:** `KNOWN_DIFFERENCES.md`.
- **Approach:**
  1. Add the intro sentence naming Go `xurl` 1.3.4 and the `Go parity` job (R1).
  2. Add the command-names section: a table of Go name, `xr` name, and what `xr` does with the Go name (R2, R3); the
     three Go-only commands (R4); the rule for `xr`-only commands, pointing at `xr --help` (R5, KD2).
  3. In the webhooks section, delete the `xr`-only bullet list and keep its sentence that a Go sign-in does not request
     `calls.read` and `calls.write`.
  4. Correct the exit-code section's opening paragraph (R8); keep its table.
  5. Add the error-streams section after it (R7), and extend the bare-word section's table and text (R6, KTD4).
  6. Append the dry-run, confirmation, auth-preflight, pagination, post-vocabulary, and secret-input sections (R9 to
     R14).
  7. Land the corrections (steps 3 and 4) and the additions as two commits, so a reviewer sees each kind apart.
- **Patterns to follow:** the existing sections' Go-then-`xr`-then-table shape and their Go source citations
  (`cli/root.go`, `api/media.go`); `crates/xurl-cli/README.md` § "Machine-Readable Output" for the post-vocabulary key
  list R13 cites rather than restates.
- **Test scenarios:**
  - Every Go claim the unit adds reproduces against Go `xurl` at `18dcb447` under a scratch `HOME`, at a closed port or
    a local mock, with the output the Appendix records.
  - Every `xr` claim reproduces against `xr` built at the PR's base with `XURL_TOKEN_STORE` at a scratch path and
    `API_BASE_URL` at a closed port or the local mock.
  - The section headings that existed before U1 are byte-identical after it.
  - No line describes a past state ("previously", "now", "used to").
- **Verification:** the doc-reading tests and markdownlint pass; each claim has its Appendix row or a Go source
  citation.

### U2. Expand the comparison table in the CLI README

- **Goal:** `crates/xurl-cli/README.md` § "Relationship to xurl" compares the tools row by row, with sizes and counts
  measured from both.
- **Requirements:** R15, R17.
- **Dependencies:** U1 (the rows summarize its sections).
- **Files:** `crates/xurl-cli/README.md`.
- **Approach:**
  1. Rename the table's Go column to `Go xurl 1.3.4`.
  2. Correct the binary-size row (KTD6) and add the command-count row (KD3, KTD5).
  3. Add a row per difference R15 lists, in KTD3's order; keep the existing rows.
  4. Point the paragraph above the table at `KNOWN_DIFFERENCES.md` for the detail behind each row.
- **Patterns to follow:** the existing table's short cells; KTD8's wording limits.
- **Test scenarios:**
  - Re-running KTD5's walk on the PR's base gives the count the row states.
  - Each row's two cells match the `KNOWN_DIFFERENCES.md` section it summarizes.
  - `crates/xurl-cli/tests/agentic_tests.rs` and `crates/xurl-cli/tests/next_step_tests.rs` pass, and
    `crates/xdk/tests/readme_stability_tests.rs` and `readme_landing_tests.rs` pass.
- **Verification:** the table renders on GitHub with no broken cell; every row traces to a section.

### U3. Summarize the differences in the root README

- **Goal:** A reader of `README.md` sees the four migration hazards KD4 names and where the detail lives.
- **Requirements:** R16, R17.
- **Dependencies:** U1, U2.
- **Files:** `README.md`.
- **Approach:** replace the "Relationship to xurl" bullet under "The CLI: xr" with a paragraph of at most five lines
  naming R2, R7, R9, and R10 in a clause each, linking `KNOWN_DIFFERENCES.md` and the CLI README's table.
- **Patterns to follow:** the surrounding section's link style (relative links into `crates/xurl-cli/README.md`).
- **Test scenarios:**
  - `crates/xurl-cli/tests/recipe_guard.rs` passes, so the paragraph names fewer than three shortcut function names.
  - `crates/xdk/tests/readme_landing_tests.rs` passes, so the landing program is untouched.
- **Verification:** the paragraph reads in one pass and both links resolve.

---

## Verification Contract

| Gate              | How                                                                                                                                                                                                                                         | Proves                       |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------- |
| Go claims         | Go `xurl` from `go install github.com/xdevplatform/xurl@18dcb447f090667171ff23666a05d6d387e2aa73` into a scratch `GOPATH`, run under `env -i` with a scratch `HOME`, `API_BASE_URL` at `127.0.0.1:9` or a local mock that logs each request | R2 to R14's Go side, offline |
| `xr` claims       | `cargo build -p xurl-rs` at the PR base; the same mock and closed port; `XURL_TOKEN_STORE` at a scratch path                                                                                                                                | R2 to R14's `xr` side        |
| Counts            | KTD5's help-tree walk on both binaries                                                                                                                                                                                                      | R15's count row              |
| Doc-reading tests | `cargo test -p xurl-rs --test recipe_guard --test agentic_tests --test next_step_tests` and `cargo test -p xdk-rs --test readme_stability_tests --test readme_landing_tests`                                                                | R17                          |
| Markdown          | `markdownlint-cli2` on the three files, through `scripts/hooks/pre-commit`                                                                                                                                                                  | lint clean                   |
| Local CI mirror   | `scripts/hooks/pre-push`, in the background                                                                                                                                                                                                 | the gates CI runs            |
| CI                | every check on each PR reads `SUCCESS` in the rollup, `Go parity` included                                                                                                                                                                  | the stack is green           |

No test runs against X. No golden fixture is re-blessed.

## Definition of Done

- R1 to R17 hold in the merged text, each traceable to an Appendix row or a Go source citation.
- `git diff` across the stack touches only `KNOWN_DIFFERENCES.md`, `crates/xurl-cli/README.md`, and `README.md`.
- Each PR names #423 under Related Issues and files its user-visible doc changes under `### Documentation` in
  `## Changelog (xurl-rs)`.
- The follow-up items above have reached release.
- No draft text from abandoned wordings is left in the diff.

---

## Appendix

### Evidence record

Go `xurl` is `v1.3.4` at `18dcb447f090667171ff23666a05d6d387e2aa73`, confirmed with `git describe --tags` in a clone and
with `go version -m` on the installed binary; it is the commit `.github/workflows/ci.yml` pins as `GO_XURL_REV`. `xr` is
`dev` at `b50850d`, version `4.4.0`. Go ran under `env -i` with a scratch `HOME` holding a fake bearer token; `xr` ran
with `XURL_TOKEN_STORE` at a scratch path holding the same. Both pointed `API_BASE_URL` at `127.0.0.1:9` or at a local
Python server answering 200, 401, 404, or 429 and logging each request's method, path, and auth scheme. Nothing reached
X.

| Claim            | Issue says                                                                                | Observed                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| ---------------- | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `posts`          | Go `posts USERNAME` is `xr user-posts`                                                    | Holds. Go `posts jack` sent `GET /2/users/by/username/jack`; `xr posts jack` sent `GET /2/tweets?ids=jack`; `xr posts --help`: "Read 1 to 100 posts by id or URL in one request"                                                                                                                                                                                                                                                                                                                                    |
| Stray words      | Go drops words after an unknown command                                                   | Holds. Go `lists list` sent `GET /lists`, `likers 1585341984679469056` sent `GET /likers`, `zzzzzz extra words` sent `GET /zzzzzz`; each exit 1                                                                                                                                                                                                                                                                                                                                                                     |
| `--dry-run`      | Go: `unknown flag: --dry-run`                                                             | Holds. Go exit 1 on `--dry-run /2/users/me`, `read --dry-run`, `delete --dry-run`; mock saw no request. Every `xr` leaf answered `--dry-run` without an unknown-flag refusal                                                                                                                                                                                                                                                                                                                                        |
| Confirmation     | `xr` gates with `--force --no-interactive`                                                | Go `delete 1585341984679469056` sent `DELETE /2/tweets/1585341984679469056` at once; `auth apps remove scratch` and `auth clear --all` acted at once. `xr auth apps remove default` with stdin not a terminal: exit 1, `confirmation-required`, `next_step.action: confirm`, `template: xr auth apps remove default --output json --force`. `--force` alone proceeds. 16 commands report `confirmation_required: true` under `--dry-run`, matching `CONFIRMED_COMMANDS` in `crates/xurl-cli/tests/agentic_tests.rs` |
| Auth preflight   | `auth-method-mismatch` at exit 2 before sending                                           | Holds. Bearer-only store: `xr like 1585341984679469056` and `xr delete ... --force` exit 2, `auth-method-mismatch`, mock saw nothing. Go `like` sent `GET /2/users/me` with the bearer; `api/client.go` `getAuthHeader` falls back OAuth2, OAuth1, bearer                                                                                                                                                                                                                                                           |
| Caller id        | Go asks `/2/users/me` every run                                                           | Holds. Go `like` sent `/2/users/me` first; `cli/shortcuts.go` `resolveMyUserID`                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Pagination       | Go has no `--cursor`; `xr` has `--cursor` / `--limit` / `-n`                              | Partly. Go `search x --cursor T`: `unknown flag: --cursor`; no `pagination_token` anywhere in Go's CLI. Go has `-n/--max-results` on list commands. `xr` sent `pagination_token=TOKEN1` on `search`, `quotes`, `lists posts`, not on `spaces search`. `xr search x --page 2`: exit 1, `unsupported-pagination`                                                                                                                                                                                                      |
| Error streams    | Go writes 401/404/429 bodies to stdout; flag errors twice                                 | Holds. Go `read` against 404, 401, 429: body on stdout, stderr empty, exit 1 each; raw `/2/users/me`: body on stdout plus `Error: request failed` on stderr. `--bogus`: `unknown flag: --bogus` on stdout, `Error: unknown flag: --bogus` and usage on stderr (`main.go` prints the error a second time). `xr read`: stdout empty, envelope on stderr, exit 4 `not-found`, 77 `auth-required`, 3 `rate-limited` with `retry_after_secs` and `retry_at`                                                              |
| Exit codes       | (doc) Go string-matches the body                                                          | Not backed. Every `os.Exit` in the Go source passes 1; 404, 401, 429 each exit 1                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| Secrets          | Go takes secrets only on argv                                                             | Not backed. Go `auth app-only -` reads the token from stdin, and Go reads `CLIENT_SECRET` from the environment. `--client-secret`, `--consumer-secret`, `--access-token`, `--token-secret` are flag values only. `xr` has `--client-secret-file`, `--consumer-secret-file`, `--access-token-file`, `--token-secret-file`, `--bearer-token-file`; `printf 'S\n' \| xr auth apps add scratch --client-id CID --client-secret-file -` stored it                                                                        |
| Post vocabulary  | `xr` normalizes `post_count`, `repost_count`, `referenced_posts`, `edit_history_post_ids` | Holds for shortcuts. Same body to both: Go `read` printed `retweet_count`, `referenced_tweets`, `edit_history_tweet_ids`; `xr read` printed `repost_count`, `referenced_posts`, `edit_history_post_ids`; raw `xr /2/tweets/<id>` printed X's keys. Go requested `tweet.fields=...&expansions=author_id,referenced_tweets.id`; `xr` requested `post.fields=...&expansions=...referenced_posts`                                                                                                                       |
| Go-only commands | `chat`, `mcp`                                                                             | Plus `token`. `xr token`, `xr mcp`: exit 2 `unknown-command`, no suggestion. `xr webhook start` suggests `webhooks`; `xr completion` suggests `completions`                                                                                                                                                                                                                                                                                                                                                         |
| Counts           | "29 endpoints"; "101 commands across 26 families"                                         | Not backed. Go: 60 commands (KTD5), 29 shortcuts over 27 distinct method and path pairs in `api/shortcuts.go`. `xr`: 160 commands, 66 top-level words, 18 families, 100 declared endpoints (`SHORTCUT_TEMPLATES` rows in `crates/xdk/build.rs`)                                                                                                                                                                                                                                                                     |
| Binary size      | (README) Go ~15 MB, `xr` ~13 MB                                                           | Go 1.3.4 Linux x86_64 25.8 MB, arm64 12.5 MB, macOS arm64 21.3 MB; `xr` 4.4.0 Linux x86_64 12.6 MB, aarch64 9.7 MB, macOS arm64 9.2 MB                                                                                                                                                                                                                                                                                                                                                                              |
| `NO_COLOR`       | (README) Go: no                                                                           | Holds. `NO_COLOR=1` Go `auth app` still printed `\e[32m`; error lines still `\e[31m`                                                                                                                                                                                                                                                                                                                                                                                                                                |
