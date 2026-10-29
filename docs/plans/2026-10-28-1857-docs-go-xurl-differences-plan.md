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

- The doc's list of `xr` commands with no Go counterpart omits fifteen runnable commands under eleven names: `usage`,
  `usage credits`, `muted`, `blocked`, `media alt-text`, `media subtitles`, `broadcasts moderators`, `schema`,
  `validate`, `examples`, and `skill`. The issue names eight of the eleven. A hand list goes stale with each new family.
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
- R18. The command-names section opens with what carries over: `xr` reads the `~/.xurl/auth.yml` Go `xurl` 1.3.4 writes,
  so the apps and credentials stored with Go are there on the first `xr` run.

**Behavior**

- R6. Go sends its first positional word as the request path and drops every word after it (`xurl lists list` sends
  `GET /lists`; `xurl likers <id>` sends `GET /likers`).
- R7. Go writes an API error's body to stdout at exit 1, and a flag error to both streams; `xr` keeps stdout for the
  command's answer and writes every failure to stderr, as text or as an envelope carrying `reason`, `exit_code`, and
  `next_step` where a step exists, with `retry_after_secs` and `retry_at` on a 429 that names its reset.
- R8. The exit-code section states that Go exits 1 on every failure, with its table unchanged.
- R9. Go refuses `--dry-run` as an unknown flag and sends nothing; `xr` takes `--dry-run` on every command, raw requests
  included, sends and stores nothing, and answers a dry run that reaches its checks with `would_succeed` at exit 0. The
  section shows a migrated script run whole under `XURL_DRY_RUN=1`, which turns each of its `xr` calls into a dry run.
- R10. Go runs `delete`, `auth clear`, and `auth apps remove` at once and has no `--force`; `xr` asks before each of
  them when stdin and stderr are terminals, refuses with `confirmation-required` (exit 1) under `--no-interactive`,
  `--quiet`, or no terminal, and proceeds under `--force`. A dry run reports `confirmation_required: true` for every
  command that asks.
- R11. Go picks a credential by kind (OAuth2, then OAuth1, then bearer) whatever the endpoint accepts; `xr` checks the
  endpoint's accepted schemes before sending and refuses a mismatch at exit 2, reason `auth-method-mismatch`.
- R12. Go's list commands take `-n/--max-results` for one page and no page token; `xr`'s take the same `-n`, and on a
  list command whose endpoint pages, `--cursor` (alias `--after`) carries X's `pagination_token`; `--page` refuses with
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
  script (R2, R7, R9, R10) and linking `KNOWN_DIFFERENCES.md` and the CLI README's table; R9's clause names
  `XURL_DRY_RUN=1`.
- R17. All three files describe present behavior with no history narration, and every test that reads them keeps
  passing.

### Success Criteria

- Each claim in the three files reproduces offline against Go `xurl` at `18dcb447` and `xr` at the PR's base, by the
  method in the Verification Contract.
- A reader holding a Go `xurl` script can find each command it calls in R3's map or among Go's matching names without
  reading `xr --help`: each of Go 1.3.4's 60 commands from KTD5's walk is in the map, has the same name in `xr`, or is
  named among R4's.

### Key Decisions

- KD1. **The binaries win over the issue.** Where the issue and an observation disagree, the docs say what the binary
  does, and the Appendix records the disagreement. Governs R2 to R14.
- KD2. **A rule replaces the `xr`-only list.** Go 1.3.4's command set is fixed at the pin, so mapping it to `xr` names
  is a list that changes only when the pin does; `xr`'s set grows with each family, and the hand list has already fallen
  fifteen commands behind. `crates/xurl-cli/tests/recipe_guard.rs` holds the same rule for shortcut function lists ("a
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
- `xr schema --dry-run` and `xr auth apps update 123 --dry-run` answer `status: error`, `reason: validation`, exit 1,
  where `AGENTS.md` § "Output formats" says `--dry-run` answers at exit 0 on any command.
- A guard holding the README's `xr` command count to `common::command_paths()`, if Brett widens the scope past docs.

---

## Planning Contract

### Key Technical Decisions

- KTD1. **One owner per difference.** Each difference is stated in full once, in a `KNOWN_DIFFERENCES.md` section; a
  README row is a short projection of it. The README table links the file, not each section, so a heading edit cannot
  break a dozen anchors.
- KTD2. **New sections follow the existing shape, and show the script change.** A heading
  `## <claim> (intentional improvement)` or `(intentional difference)`; a paragraph on Go citing its source file at the
  pin (`cli/root.go`, `api/client.go`); a paragraph on `xr`; a Go-versus-`xr` table when three or more invocations
  differ. A section whose difference makes a Go-era script change (R7, R10, R12, R14) shows the `xr` invocation the
  script needs, as `docs/migrating` does for each major. Observations live in this plan, not the doc.
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
  non-dash character). Go's argv form of a secret is described in prose, never shown as an invocation, in all three
  files.
- KTD9. **The diff carries only the edited text.** None of the three files is in the edit hook's wrapped shape at `HEAD`
  (`git show HEAD:<file> | md-wrap.py -w 120` differs by 18, 20, and 75 lines), so an edit that lets the hook reflow the
  whole file rewraps paragraphs no unit touches. Each unit wraps its own new text at 120 by hand and leaves every other
  line byte-identical to `HEAD`.

### High-Level Technical Design

`KNOWN_DIFFERENCES.md` after U1, in reading order. Existing headings are unchanged; new ones are marked.

```text
# Known Differences from Go xurl
  intro: compared against Go xurl 1.3.4, the release `Go parity` runs                      (R1)
## Command names that differ, and commands only one tool has           [new]                (R2-R5, R18)
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
## `delete` and the store commands ask first                             [new]                (R10)
## A credential the endpoint does not accept is refused before sending   [new]                (R11)
## `--cursor` asks for the next page                                     [new]                (R12)
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
- **Requirements:** R1 to R14, R17, R18.
- **Dependencies:** none.
- **Files:** `KNOWN_DIFFERENCES.md`.
- **Approach:**
  1. Add the intro sentence naming Go `xurl` 1.3.4 and the `Go parity` job (R1).
  2. Add the command-names section: a table of Go name, `xr` name, and what `xr` does with the Go name (R2, R3); the
     three Go-only commands (R4); the rule for `xr`-only commands, pointing at `xr --help` (R5, KD2); its opening line
     on the shared store (R18).
  3. In the webhooks section, delete the `xr`-only bullet list and keep its sentence that a Go sign-in does not request
     `calls.read` and `calls.write`.
  4. Correct the exit-code section's opening paragraph (R8); keep its table.
  5. Add the error-streams section after it (R7), and extend the bare-word section's table and text (R6, KTD4).
  6. Append the dry-run, confirmation, auth-preflight, pagination, post-vocabulary, and secret-input sections (R9 to
     R14).
  7. Land the corrections (steps 3 and 4) and the additions as two commits, so a reviewer sees each kind apart.
- **Execution note:** KTD9 applies; after each edit, read the diff hunk by hunk and restore any line the edit did not
  mean to change.
- **Patterns to follow:** the existing sections' Go-then-`xr`-then-table shape and their Go source citations
  (`cli/root.go`, `api/media.go`); `crates/xurl-cli/README.md` § "Machine-Readable Output" for the post-vocabulary key
  list R13 cites rather than restates.
- **Test scenarios:**
  - Every Go claim the unit adds reproduces against Go `xurl` at `18dcb447` under a scratch `HOME`, at a closed port or
    a local mock, with the output the Appendix records.
  - Every `xr` claim reproduces against `xr` built at the PR's base with `XURL_TOKEN_STORE` at a scratch path and
    `API_BASE_URL` at a closed port or the local mock.
  - A store Go `xurl` 1.3.4 wrote under a scratch `HOME` (a bearer token, and fake OAuth1 values through
    `xurl auth oauth1`) lists the same app and credential kinds in `xr --output json auth status`.
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
- **Execution note:** KTD9 applies, and this file differs most from the hook's shape.
- **Technical design:** the rows R15 adds, directionally (cell wording is the implementer's within each claim):
  - Binary size, Linux x86_64 release: `~26 MB` (bundles `chat`) and `~13 MB`.
  - Commands in `--help`: `60, 29 of them shortcuts` and `160`.
  - `xurl posts USER`: `xr user-posts USER`; `xr posts` reads posts by id.
  - `--dry-run`: `No` and `Every command, offline`.
  - Mistyped command: `Sent as a path, later words dropped` and `Exit 2, names the nearest command`.
  - `delete`, `auth clear`, `auth apps remove`: `Run at once` and `Ask first; --force skips`.
  - Credential the endpoint does not accept: `Sent anyway` and `Exit 2 before sending`.
  - Caller's user id: `Asked of /2/users/me every run` and `Stored per login; whoami --cached`.
  - API errors: `Body on stdout, exit 1` and `Envelope on stderr, exit by status`.
  - Next page of a list: `Raw request with pagination_token` and `--cursor`.
  - Secrets from a file or stdin: `Bearer token only` and `Every secret, --<name>-file`.
  - Post vocabulary in shortcut output: `As X sends it` and `The spec's names`.
  - `chat`, `mcp`, `token`: `Yes` and `No`.
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
  naming R2, R7, R9, and R10 in a clause each, linking `KNOWN_DIFFERENCES.md` and the CLI README's table. R10's clause
  names `xr delete`, not every destructive command, since `media subtitles remove` does not ask.
- **Execution note:** KTD9 applies.
- **Patterns to follow:** the surrounding section's link style (relative links into `crates/xurl-cli/README.md`).
- **Test scenarios:**
  - `crates/xurl-cli/tests/recipe_guard.rs` passes, so the paragraph names fewer than three shortcut function names.
  - `crates/xdk/tests/readme_landing_tests.rs` passes, so the landing program is untouched.
- **Verification:** the paragraph reads in one pass and both links resolve.

---

## Verification Contract

| Gate              | How                                                                                                                                                                                                                                         | Proves                                                       |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| Go claims         | Go `xurl` from `go install github.com/xdevplatform/xurl@18dcb447f090667171ff23666a05d6d387e2aa73` into a scratch `GOPATH`, run under `env -i` with a scratch `HOME`, `API_BASE_URL` at `127.0.0.1:9` or a local mock that logs each request | R2 to R14's Go side, offline                                 |
| `xr` claims       | `cargo build -p xurl-rs` at the PR base; the same mock and closed port; `XURL_TOKEN_STORE` at a scratch path                                                                                                                                | R2 to R14's `xr` side                                        |
| Counts            | KTD5's help-tree walk on both binaries                                                                                                                                                                                                      | R15's count row                                              |
| Go command map    | walk Go 1.3.4's 60 commands from KTD5 against the command-names section                                                                                                                                                                     | each is mapped, shares its name in `xr`, or is named Go-only |
| Doc-reading tests | `cargo test -p xurl-rs --test recipe_guard --test agentic_tests --test next_step_tests` and `cargo test -p xdk-rs --test readme_stability_tests --test readme_landing_tests`                                                                | R17                                                          |
| Markdown          | `markdownlint-cli2` on the three files, through `scripts/hooks/pre-commit`                                                                                                                                                                  | lint clean                                                   |
| Local CI mirror   | `scripts/hooks/pre-push`, in the background                                                                                                                                                                                                 | the gates CI runs                                            |
| CI                | every check on each PR reads `SUCCESS` in the rollup, `Go parity` included                                                                                                                                                                  | the stack is green                                           |

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

| Claim                  | Issue says                                                                                | Observed                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| ---------------------- | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `posts`                | Go `posts USERNAME` is `xr user-posts`                                                    | Holds. Go `posts jack` sent `GET /2/users/by/username/jack`; `xr posts jack` sent `GET /2/tweets?ids=jack`; `xr posts --help`: "Read 1 to 100 posts by id or URL in one request"                                                                                                                                                                                                                                                                                                                                    |
| Stray words            | Go drops words after an unknown command                                                   | Holds. Go `lists list` sent `GET /lists`, `likers 1585341984679469056` sent `GET /likers`, `zzzzzz extra words` sent `GET /zzzzzz`; each exit 1                                                                                                                                                                                                                                                                                                                                                                     |
| `--dry-run`            | Go: `unknown flag: --dry-run`                                                             | Holds. Go exit 1 on `--dry-run /2/users/me`, `read --dry-run`, `delete --dry-run`; mock saw no request. Every `xr` leaf answered `--dry-run` without an unknown-flag refusal                                                                                                                                                                                                                                                                                                                                        |
| Confirmation           | `xr` gates with `--force --no-interactive`                                                | Go `delete 1585341984679469056` sent `DELETE /2/tweets/1585341984679469056` at once; `auth apps remove scratch` and `auth clear --all` acted at once. `xr auth apps remove default` with stdin not a terminal: exit 1, `confirmation-required`, `next_step.action: confirm`, `template: xr auth apps remove default --output json --force`. `--force` alone proceeds. 16 commands report `confirmation_required: true` under `--dry-run`, matching `CONFIRMED_COMMANDS` in `crates/xurl-cli/tests/agentic_tests.rs` |
| Auth preflight         | `auth-method-mismatch` at exit 2 before sending                                           | Holds. Bearer-only store: `xr like 1585341984679469056` and `xr delete ... --force` exit 2, `auth-method-mismatch`, mock saw nothing. Go `like` sent `GET /2/users/me` with the bearer; `api/client.go` `getAuthHeader` falls back OAuth2, OAuth1, bearer                                                                                                                                                                                                                                                           |
| Caller id              | Go asks `/2/users/me` every run                                                           | Holds. Go `like` sent `/2/users/me` first; `cli/shortcuts.go` `resolveMyUserID`                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Pagination             | Go has no `--cursor`; `xr` has `--cursor` / `--limit` / `-n`                              | Partly. Go `search x --cursor T`: `unknown flag: --cursor`; no `pagination_token` anywhere in Go's CLI. Go has `-n/--max-results` on list commands. `xr` sent `pagination_token=TOKEN1` on `search`, `quotes`, `lists posts`, not on `spaces search`. `xr search x --page 2`: exit 1, `unsupported-pagination`                                                                                                                                                                                                      |
| Error streams          | Go writes 401/404/429 bodies to stdout; flag errors twice                                 | Holds. Go `read` against 404, 401, 429: body on stdout, stderr empty, exit 1 each; raw `/2/users/me`: body on stdout plus `Error: request failed` on stderr. `--bogus`: `unknown flag: --bogus` on stdout, `Error: unknown flag: --bogus` and usage on stderr (`main.go` prints the error a second time). `xr read`: stdout empty, envelope on stderr, exit 4 `not-found`, 77 `auth-required`, 3 `rate-limited` with `retry_after_secs` and `retry_at`                                                              |
| Exit codes             | (doc) Go string-matches the body                                                          | Not backed. Every `os.Exit` in the Go source passes 1; 404, 401, 429 each exit 1                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| Secrets                | Go takes secrets only on argv                                                             | Not backed. Go `auth app-only -` reads the token from stdin, and Go reads `CLIENT_SECRET` from the environment. `--client-secret`, `--consumer-secret`, `--access-token`, `--token-secret` are flag values only. `xr` has `--client-secret-file`, `--consumer-secret-file`, `--access-token-file`, `--token-secret-file`, `--bearer-token-file`; `printf 'S\n' \| xr auth apps add scratch --client-id CID --client-secret-file -` stored it                                                                        |
| Post vocabulary        | `xr` normalizes `post_count`, `repost_count`, `referenced_posts`, `edit_history_post_ids` | Holds for shortcuts. Same body to both: Go `read` printed `retweet_count`, `referenced_tweets`, `edit_history_tweet_ids`; `xr read` printed `repost_count`, `referenced_posts`, `edit_history_post_ids`; raw `xr /2/tweets/<id>` printed X's keys. Go requested `tweet.fields=...&expansions=author_id,referenced_tweets.id`; `xr` requested `post.fields=...&expansions=...referenced_posts`                                                                                                                       |
| Go-only commands       | `chat`, `mcp`                                                                             | Plus `token`. `xr token`, `xr mcp`: exit 2 `unknown-command`, no suggestion. `xr webhook start` suggests `webhooks`; `xr completion` suggests `completions`                                                                                                                                                                                                                                                                                                                                                         |
| Counts                 | "29 endpoints"; "101 commands across 26 families"                                         | Not backed. Go: 60 commands (KTD5), 29 shortcuts over 27 distinct method and path pairs in `api/shortcuts.go`. `xr`: 160 commands, 66 top-level words, 18 families, 100 declared endpoints (`SHORTCUT_TEMPLATES` rows in `crates/xdk/build.rs`)                                                                                                                                                                                                                                                                     |
| Binary size            | (README) Go ~15 MB, `xr` ~13 MB                                                           | Go 1.3.4 Linux x86_64 25.8 MB, arm64 12.5 MB, macOS arm64 21.3 MB; `xr` 4.4.0 Linux x86_64 12.6 MB, aarch64 9.7 MB, macOS arm64 9.2 MB                                                                                                                                                                                                                                                                                                                                                                              |
| `NO_COLOR`             | (README) Go: no                                                                           | Holds. `NO_COLOR=1` Go `auth app` still printed `\e[32m`; error lines still `\e[31m`                                                                                                                                                                                                                                                                                                                                                                                                                                |
| Dry-run exits (review) | (`AGENTS.md`) answers at exit 0 on any command                                            | Not on every one. `xr schema --dry-run` and `xr auth apps update 123 --dry-run`: `status: error`, `reason: validation`, exit 1                                                                                                                                                                                                                                                                                                                                                                                      |
| Page token (review)    | n/a                                                                                       | `/2/spaces/search` declares `query`, `state`, `max_results`, and field parameters, no `pagination_token`; `xr spaces search x --cursor TOKEN1` sent none                                                                                                                                                                                                                                                                                                                                                            |
| Gate coverage (review) | n/a                                                                                       | `xr media subtitles remove 123 --language en --dry-run` reports no `confirmation_required`; it is absent from `CONFIRMED_COMMANDS`                                                                                                                                                                                                                                                                                                                                                                                  |

---

## Engineering Review Record

`/plan-eng-review` of this plan on 2026-10-28, run inline in one session with no subagent and no outside model (Brett's
standing rule for this repo; `codex_reviews` is also disabled). Nobody watches the session, so each choice below was
settled from the cited evidence, and the one choice that widens the brief went to Brett instead.

**Scope Challenge.** Three files change and nothing new is built, so the complexity gate did not trip. Scope accepted
as-is.

**Reality rows.**

- REALITY: premise-table pass. Each claim carries an Appendix row or a Go source receipt at `18dcb447`
  (`cli/shortcuts.go` `printResult` and `resolveMyUserID`, `api/client.go` `getAuthHeader`, `main.go`,
  `cli/chat_unsupported.go:1` build tag). unverified=0; KTD5's count and the 4.4.1 assumption are tagged [re-measure
  before gate].
- REALITY: already-done pass. `git log origin/dev --since=2026-10-27` over the three files shows #427 and #428, both in
  the base and both extending the bare-word section KTD4 builds on.
- REALITY: surface-check pass. Every command and flag the plan names exists in the walked `--help` of the binary it
  names; the Go words `xr` lacks were observed refused.
- REALITY: binding-decisions pass. The 2026-09-22 wire-vocabulary decision (normalize in `decode()`, raw mode untouched)
  is what R13 describes.
- REALITY: numbers finding, fixed. The Problem Frame named twelve omitted commands including `lists members`, which the
  list covers as `members`; recomputed from the tree walk, fifteen runnable commands under eleven names [measured: tree
  walk against the list]. Every other count is [measured]: 60 and 160 commands, 29 shortcuts, 27 endpoints, 100 declared
  endpoints, 16 gated commands, binary bytes.
- REALITY: deferred-asks pass. The issue's completed `xr`-only list becomes KD2's rule (named in the PR body for Brett
  to reverse); its `-H` redaction mention stays out (Scope Boundaries: no Go contrast); its "101 commands" figure
  becomes KTD5's count.
- REALITY: pr-count finding, kept. Two PRs, one more than the default. `scripts/cargo-inputs.sh` classes
  `KNOWN_DIFFERENCES.md` as inert, so PR1 skips the Rust jobs; both READMEs are cargo inputs, so PR2 runs the full
  battery once. The split costs one short run, and `gh stack merge` lands both without a restack.

**Section 1, Architecture.**

- [INFORMATIONAL] (confidence: 8/10) `crates/xurl-cli/tests/agentic_tests.rs:822`,
  `const CONFIRMED_COMMANDS: &[&str] = &[` lists sixteen commands and not `media subtitles remove`, which sends
  `DELETE /2/media/subtitles` unasked. R10's heading claimed every destructive command asks. Accepted: R10 names Go's
  three counterparts plus the `confirmation_required` discovery rule, the heading narrows, and U3's clause names
  `xr delete`. R10 also states the refusal conditions from `crates/xurl-cli/src/cli/commands/mod.rs:76`,
  `!no_interactive && !quiet && std::io::stdin().is_terminal() && std::io::stderr().is_terminal()`.
- [INFORMATIONAL] (confidence: 8/10) R9 claimed exit 0 on every dry run; `xr schema --dry-run` exits 1 with `validation`
  (Appendix). Accepted: R9 promises exit 0 only for a dry run that reaches its checks, and the mismatch with `AGENTS.md`
  went to follow-ups.
- [INFORMATIONAL] (confidence: 8/10) `crates/xdk/vendor/x-api-openapi.json`, `/2/spaces/search` declares no
  `pagination_token`. R12 said every list command takes `--cursor`. Accepted: R12 covers list commands whose endpoint
  pages, and the heading narrows.
- Design-twice: no new module or boundary. Red flag 8 (hand-synced list): the README rows project `KNOWN_DIFFERENCES.md`
  sections (accepted under KTD1, one short line per row), and R15's `xr` count tracks the clap tree by hand (the guard
  is Brett's call, KTD7).

**Section 2, Code quality.**

- [INFORMATIONAL] (confidence: 9/10) `git show HEAD:<file> | python3 ~/.claude/md-wrap.py -w 120` differs from `HEAD` by
  18 lines for `KNOWN_DIFFERENCES.md`, 20 for `README.md`, 75 for `crates/xurl-cli/README.md`. An edit the hook reflows
  would rewrap untouched paragraphs. Accepted: KTD9 and an execution note on each unit.
- [INFORMATIONAL] (confidence: 7/10) R15's "a row for each difference that has a two-cell shape" left the row set to the
  implementer. Accepted: U2 lists the rows directionally.
- No shared-code extraction proposed.

**Section 3, Tests.**

```text
DOC CLAIMS                                       READER FLOWS
[+] KNOWN_DIFFERENCES.md (U1)                    [+] Migrating a Go xurl script
  ├── [MANUAL] Go claims: scratch HOME, mock       ├── [MANUAL] each Go command found in R3's map
  ├── [MANUAL] xr claims: scratch store, mock      └── [MANUAL] posts -> user-posts reached first (KTD3)
  └── [GAP] no test reads this file (inert)      [+] Reading the README table (U2)
[+] crates/xurl-cli/README.md (U2)                 ├── [★★ TESTED] secret-on-argv guard  agentic_tests.rs:1033
  ├── [MANUAL] KTD5 count re-walk at the base      ├── [★★ TESTED] .id filter guard      agentic_tests.rs:1058
  └── [GAP] count drifts after the next family     └── [★★ TESTED] steps table           next_step_tests.rs:342
[+] README.md (U3)                               [+] Root README summary
  ├── [★★ TESTED] function-list guard  recipe_guard.rs:95
  └── [★★ TESTED] landing program      readme_landing_tests.rs:20

COVERAGE: 5/10 paths by existing tests | 5 by the Verification Contract's manual reproduction
GAPS: 2 (both accepted: no test reads KNOWN_DIFFERENCES.md; the count guard is Brett's call)
```

- Test review: no new test proposed. The brief keeps the change to docs, and `Go parity` already runs the differential
  suite on behavior. Test Plan artifact written to the gstack project directory.

**Section 4, Performance.** No issues found. The only cost is CI time, counted under pr-count.

**Failure modes.** One critical gap: after the next family lands, R15's `xr` count goes stale with no test, no error,
and no reader noticing. KTD5's re-walk catches it only at authoring time. Disposition: a guard over
`common::command_paths()` is Brett's call (KTD7).

**Outside voice.** Disabled (`codex_reviews`); no reviewer dispatched.

**NOT in scope.** The defects under Deferred to Follow-Up Work; a committed evidence harness; documenting `xr`-only
features with no Go contrast.

**What already exists.** `Go parity` (`.github/workflows/ci.yml`, `GO_XURL_REV`) and
`crates/xurl-cli/tests/conformance_runner.rs` test behavior differentially; `CONFIRMED_COMMANDS` and `dry_run_guard.rs`
pin the gate and the dry-run rule; `crates/xurl-cli/README.md` § "Machine-Readable Output" already lists the renamed
post keys, which R13 cites.

**Parallelization.** Sequential implementation, no parallelization opportunity.

## Implementation Tasks

- [ ] **T1 (P1, human: ~3h / CC: ~30min)** `KNOWN_DIFFERENCES.md`: write U1 with R9, R10, and R12 as amended
  - Surfaced by: Section 1 findings on R9, R10, R12
  - Files: `KNOWN_DIFFERENCES.md`
  - Verify: each claim re-run against both binaries; doc-reading tests; markdownlint
- [ ] **T2 (P1, human: ~1h / CC: ~15min)** CLI README: write U2's rows from its technical design
  - Surfaced by: Section 2, R15 row set
  - Files: `crates/xurl-cli/README.md`
  - Verify: KTD5 re-walk; `agentic_tests`, `next_step_tests`
- [ ] **T3 (P2, human: ~30min / CC: ~5min)** all three files: keep each diff to the edited text (KTD9)
  - Surfaced by: Section 2, edit-hook shape
  - Files: `KNOWN_DIFFERENCES.md`, `README.md`, `crates/xurl-cli/README.md`
  - Verify: `git diff` shows no hunk outside the edited paragraphs

---

## Developer Experience Review Record

`/plan-devex-review` of this plan on 2026-10-28, run inline in one session with no subagent and no outside model. Nobody
watches the session, so every Step 0 question and pass decision below was settled from the cited evidence; none needed
Brett.

**Product type.** Documentation for a CLI tool: the plan changes three Markdown files and no command.

**Persona** (from #423 § "Agent-native considerations": "Agents and developers ... migrating existing `xurl` skills and
scripts").

```text
TARGET DEVELOPER PERSONA
========================
Who:       a script or skill author with working Go xurl 1.3.4 invocations, or the agent rewriting them
Context:   swapping `xurl` for `xr` in an existing script, CI job, or agent skill
Tolerance: one screen of README, then a few minutes in KNOWN_DIFFERENCES.md; every X call is billed
Expects:   a Go-to-xr command map first, then what changes in output, errors, and prompts
```

**Developer perspective.** I have a skill that calls `xurl posts`, `xurl delete`, and `xurl read`, and parses whatever
lands on stdout. The README's CLI section links "Relationship to xurl" and `KNOWN_DIFFERENCES.md`. The doc opens on
webhooks and ngrok, which my skill never touches. Further down, a list says `xr posts` has no Go counterpart, so I take
`posts` to be a new command and keep my `xr posts jack`. It sends `GET /2/tweets?ids=jack`, and its dry run had said
`would_succeed: true`. My error branch reads stdout and finds it empty, because `xr` writes the envelope to stderr. My
cleanup step's `xr delete` stops with `confirmation-required`. Each lesson costs a run, and against the live API each
run is billed. (Observed: the doc's order, the list's wording, the `posts` request, the empty stdout, the refusal.
Predicted: the reader's conclusions and the elapsed time.)

**Competitive benchmark.** Clock: from landing on `README.md` with a Go `xurl` script to knowing every change the script
needs.

| Tool                 | Start to result                     | Time and evidence                        | DX choice                                                       | Source                        |
| -------------------- | ----------------------------------- | ---------------------------------------- | --------------------------------------------------------------- | ----------------------------- |
| `gh` replacing `hub` | README to knowing what changes      | unknown                                  | Prose comparison, no command map                                | `cli/cli` `docs/gh-vs-hub.md` |
| `xr` today           | README to the script's four changes | ~10 min, several failed runs [estimated] | Divergence list, no map, `posts` misread                        | this plan's Problem Frame     |
| `xr` after this plan | README to the script's four changes | under 2 min to read them [estimated]     | Four hazards on the README, map first in `KNOWN_DIFFERENCES.md` | R16, KTD3                     |

Target: Champion (under 2 minutes) to see the four hazards, which the README paragraph (R16) reaches in one screen. The
full map is a few minutes' read. Measuring it needs a reader, so the target stays estimated.

**Magical moment.** The migrated script run whole under `XURL_DRY_RUN=1`: every `xr` call answers offline with
`would_succeed`, the credential it would use, and `confirmation_required`, and nothing reaches X. Observed: `whoami`,
`user-posts jack`, `delete <id>`, and a raw `/2/users/me` each answered `status: dry_run`, and the logging mock saw 0
requests. Vehicle: R9's section and R16's clause, both existing behavior.

**Mode.** DX POLISH: an enhancement whose scope #423 sets.

**Journey.**

```text
STAGE           | READER DOES                                 | FRICTION POINT                           | STATUS
----------------|---------------------------------------------|------------------------------------------|--------
1. Discover     | reads README "The CLI: xr"                  | hazards one link away                    | fixed (R16)
2. Install      | brew or cargo install                       | none for this doc                        | ok
3. First result | finds `xurl posts` in the map               | map absent; `posts` reads as new         | fixed (R2, R3, KTD3)
4. Real usage   | rewrites each call                          | no invocation for the changed calls      | fixed (KTD2)
5. Debug        | runs the script                             | each failure costs a paid run            | fixed (R9's XURL_DRY_RUN=1)
6. Upgrade      | keeps the Go store and logins               | unstated whether the store carries over  | fixed (R18)
```

**First-time developer report.**

```text
T+0:00  README "The CLI: xr": a bullet links Relationship to xurl and KNOWN_DIFFERENCES.md.
T+0:30  KNOWN_DIFFERENCES.md opens on webhooks and ngrok; the script has no webhook.
T+1:30  The list says `xr posts` has no Go counterpart; the reader keeps `posts`.
T+2:30  `xr posts jack` sends ids=jack; the dry run had said would_succeed: true.
T+4:00  The error branch reads stdout and finds nothing; the envelope is on stderr.
T+6:00  `xr delete` refuses with confirmation-required; the reader adds --force after reading the template.
```

Addressed: T+0:30 and T+1:30 by KTD3 and R2; T+4:00 by R7 and KTD2; T+6:00 by R10 and KTD2; T+2:30's dry-run answer is a
defect under Deferred to Follow-Up Work.

**Decisions** (each settled from evidence; none crosses the docs-only boundary).

- DX-1. Show the script change. A section whose difference forces a change shows the `xr` invocation the script needs
  (KTD2). Evidence: `README.md` § "Stability" says each major's breaks are written up "with before-and-after invocations
  under `docs/migrating`", and the existing sections already tabulate invocations.
- DX-2. State what carries over (R18). Evidence: `xr --output json auth status`, run on a store Go 1.3.4 wrote, listed
  `{"name":"default","bearer":true,"bearer_source":"store"}`; U1 extends the check to an OAuth1 entry.
- DX-3. Name `XURL_DRY_RUN=1` as the way to check a migrated script (R9, R16). Evidence: the magical-moment observation
  above.
- DX-4. Measure success criterion 2 by walking Go's 60 commands against the map (Verification Contract).
- DX-5. Describe Go's argv form of a secret in prose (KTD8), so no doc shows a secret as an argument.

**Passes.**

| Pass               | Before | After | Finding                                                                                                       |
| ------------------ | ------ | ----- | ------------------------------------------------------------------------------------------------------------- |
| 1. Getting started | 6      | 8     | The four hazards sit one link deep; R16 lifts them onto the README                                            |
| 2. CLI design      | 7      | 8     | No Go-to-`xr` map; R3 adds it. Command names are fixed contract, out of scope                                 |
| 3. Error messages  | 7      | 7     | Docs explain the envelope; `xr token` exits `unknown-command` with only `xr --help`, a code gap outside scope |
| 4. Documentation   | 6      | 8     | No invocation for a changed call; KTD2 adds them                                                              |
| 5. Upgrade path    | 6      | 8     | Comparison version unstated (R1) and store carry-over unstated (R18)                                          |
| 6. Dev environment | 7      | 8     | `XURL_DRY_RUN=1` exists but goes unmentioned for migration (R9)                                               |
| 7. Community       | 7      | 7     | `.github/ISSUE_TEMPLATE/config.yml` already routes Go `xurl` bugs upstream; no gap in scope                   |
| 8. DX measurement  | 5      | 6     | Success criterion 2 is now walked (DX-4); reading time stays an estimate                                      |

**Error paths the docs describe** (agent-ready-v1, existing `xr` behavior).

```text
Path                              | Class   | what | fix  | recovery | code | context | trace | redaction | channel | Cap
xr token (a Go word)              | usage   | pass | fail | N/A      | pass | N/A     | N/A   | pass      | pass    | CAP6:fix
xr delete <id>, no terminal       | usage   | pass | pass | pass     | pass | N/A     | N/A   | pass      | pass    | none
xr read <id> -> 404               | api     | pass | pass | pass     | pass | pass    | N/A   | pass      | pass    | none
xr posts jack (a Go habit)        | usage   | fail | fail | N/A      | N/A  | N/A     | N/A   | pass      | fail    | CAP6:fix
```

`xr token` today: `unknown-command`, `next_step: show-help xr --help`, with nothing saying the command is Go-only. A
corrected line would name `KNOWN_DIFFERENCES.md`. `xr posts jack` dry-runs as `would_succeed: true`, where a corrected
answer would refuse a non-numeric id and point at `user-posts`. Both are code, outside this plan; they reach release
with the other follow-ups. There is no CAP4 finding: no secret leaks and no prompt blocks, since a refused `delete`
names `--force`.

**NOT in scope.** A `docs` link on `unknown-command` for Go-only words and Go spellings; id validation on `xr posts`; a
measured reading-time study.

**What already exists.** `docs/migrating` (the before-and-after convention), `XURL_DRY_RUN`, the `confirm` step's
`--force` template, `xr auth status`, and `.github/ISSUE_TEMPLATE/config.yml`'s link to Go `xurl`'s tracker.

### Implementation Tasks (DX review)

- [ ] **T4 (P1, human: ~1h / CC: ~10min)** `KNOWN_DIFFERENCES.md`: write R18 and the `XURL_DRY_RUN=1` paragraph, and the
  invocation lines KTD2 asks for
  - Surfaced by: DX-1 to DX-3
  - Files: `KNOWN_DIFFERENCES.md`, `README.md`
  - Verify: the shared-store and dry-run checks re-run at the PR base; the Go command map gate

---

## DX Review Scorecard

```text
+====================================================================+
|              DX PLAN REVIEW — SCORECARD                             |
+====================================================================+
| Dimension            | Score  | Prior  | Trend  |
|----------------------|--------|--------|--------|
| Getting Started      |  8/10  |  6/10  |  +2 ↑  |
| API/CLI/SDK          |  8/10  |  7/10  |  +1 ↑  |
| Error Messages       |  7/10  |  7/10  |   0    |
| Documentation        |  8/10  |  6/10  |  +2 ↑  |
| Upgrade Path         |  8/10  |  6/10  |  +2 ↑  |
| Dev Environment      |  8/10  |  7/10  |  +1 ↑  |
| Community            |  7/10  |  7/10  |   0    |
| DX Measurement       |  6/10  |  5/10  |  +1 ↑  |
+--------------------------------------------------------------------+
| TTHW                 | <2 min | ~10 min| est.   |
| Competitive Rank     | Champion (estimated)                        |
| Magical Moment       | designed via XURL_DRY_RUN=1 over the script |
| Product Type         | Documentation (CLI tool)                    |
| Mode                 | POLISH                                      |
| Overall DX           |  8/10  |  6/10  |  +2 ↑  |
+====================================================================+
| DX PRINCIPLE COVERAGE                                               |
| Zero Friction      | covered (R16)                                  |
| Learn by Doing     | covered (KTD2 invocations)                     |
| Fight Uncertainty  | covered (R7, R10, R18)                         |
| Opinionated + Escape Hatches | covered (--force, --dry-run)         |
| Code in Context    | covered (the script change per section)        |
| Magical Moments    | covered (XURL_DRY_RUN=1)                       |
| Agents Are Users   | gap: unknown-command for Go words, code only   |
+====================================================================+
```

No dimension is below 6, and TTHW is under 10 minutes. Unresolved decisions in this review: none.

---

## GSTACK REVIEW REPORT

| Review         | Trigger               | Why                             | Runs | Status      | Findings                                        |
| -------------- | --------------------- | ------------------------------- | ---- | ----------- | ----------------------------------------------- |
| CEO Review     | `/plan-ceo-review`    | Scope & strategy                | 0    | not run     | n/a                                             |
| Outside Review | codex (plan review)   | Independent 2nd opinion         | 0    | disabled    | `codex_reviews` disabled                        |
| Eng Review     | `/plan-eng-review`    | Architecture & tests (required) | 1    | ISSUES OPEN | 7 issues, 1 critical gap, all folded            |
| Design Review  | `/plan-design-review` | UI/UX gaps                      | 0    | not run     | n/a                                             |
| DX Review      | `/plan-devex-review`  | Developer experience gaps       | 1    | ISSUES OPEN | score: 6/10 → 8/10, TTHW: ~10 min → under 2 min |

- **OUTSIDE COVERAGE:** codex, plan-review phase, disabled by config for both reviews; no findings.
- **VERDICT:** Eng and DX reviews done; every finding folded into the plan. Build does not wait on the open item below.

**UNRESOLVED DECISIONS:**

- The count guard (Brett): widen the change past docs to hold R15's `xr` count to the clap tree, or accept the hand
  count.
