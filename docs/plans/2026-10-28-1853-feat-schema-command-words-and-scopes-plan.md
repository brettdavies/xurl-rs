---
title: Schema Command Words and Scopes - Plan
type: feat
date: 2026-10-28
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
---

# Schema Command Words and Scopes - Plan

**Target repo:** brettdavies/xurl-rs. Every path is in it. **Base:** `dev`. **Issues:** #424, #426.

## Goal Capsule

- **Objective:** An agent or a person can go from any typed `xr` command to the shape of its `data` in one offline call.
  They name the command with the same words they run it with and read its help with, and they get that command's fields,
  about 60 lines, instead of 416 lines of shared wrapper definitions.
- **Means:** a schema registry keyed by command words, scope flags over schemars' own inlined generation, and schema
  pointers in help and dry runs that are derived from the registry (KTD1, KTD3, KTD7, KTD8).
- **Authority:** R-IDs win on behavior, KTDs on mechanism, and a unit overrides neither. Brett's answers to D1–D3 (Open
  Questions) replace the recommended defaults recorded here; all three are answered and recorded. `AGENTS.md`
  § "Where a change goes" places every change in the binary crate.
- **Execution profile:** inline in one session, with no subagents and no worktrees. A stack of three PRs, one per unit;
  the bottom PR targets `dev`.
- **Stop conditions:**
  - A change would send a request to the X API.
  - A raw request would carry a schema pointer.
  - A committed `schema/responses/*.schema.json` would change bytes beyond the renames R15 names.
  - The dry-run `command` value would change (D3).
  - A PR would open while a Brett decision that blocks it is unanswered (Open Questions). Building on the recommended
    default before then is allowed.
- **Who ships:** this session opens the PRs and reports them to the release session. Brett merges.

---

## Product Contract

### Summary

`xr schema` takes a command's own words (`xr schema webhooks subscriptions count`). By default it prints the resolved
schema of that command's `data` with a few metadata keys. `--full`, `--common`, `--envelope`, and `--path` select the
other scopes. Every typed command's help ends with a `Response schema:` footer, and every dry run of a typed command
carries `response_schema`, both naming the `xr schema` call that describes its output.

### Problem Frame

Reproduced on `dev` at `b50850d` (`xr 4.4.0`, a debug build, with `XURL_TOKEN_STORE` at a scratch path and
`API_BASE_URL` at a closed port). Every claim in both issues holds against the binary, with one exception noted below.
Every figure here was measured on that build and on the committed `schema/responses/`.

- **Naming.** `xr schema webhooks subscriptions count` exits 2 with `invalid-args` ("unexpected argument
  'subscriptions'"). `xr schema` takes one hyphen-joined token, while `xr help` takes words. Of the 144 registry names,
  97 are multi-word paths that must be hyphen-joined. Three match no path: `redirect-uri-get` and `redirect-uri-set`
  (whose path is `auth apps redirect-uri get|set`), and `skill-install-all` (the `--all` form of `skill install`). The
  other 44 are single words. A slug cannot be split back into words without the command tree, because 21 command paths
  contain a hyphenated word (`bookmark-folders`, `spaces by-creator`, `dm-events`).
- **Size.** The 144 committed schemas average 416.0 lines (median 405), and `webhooks-subscriptions-count.schema.json`
  is 426. Each of the 138 `ApiResponse` schemas carries the same eight wrapper `$defs` (`Includes`, `Post`,
  `PostPublicMetrics`, `ReferencedPost`, `User`, `UserPublicMetrics`, `ApiError`, `ResponseMeta`), byte-identical across
  all 138 and about 345 lines. The `data` payload resolved on its own averages 62 lines fully inlined, or 71 with its
  own `$defs`; the issue's 67 sits between the two. `properties.data` is a bare `$ref`, so `jq '.properties.data'`
  returns a pointer.
- **Discoverability.** Neither `xr webhooks subscriptions count -h` nor `xr --dry-run webhooks subscriptions count`
  mentions a schema.
- **The exception.** #424's dry-run example shows `"command": "webhooks subscriptions count"`. The binary emits
  `"command": "webhooks-subscriptions-count"`, because each dispatch arm writes its slug by hand (D3).
- **Collisions.** `xr schema envelope` is both a positional and a flag. `--all --list` silently prefers `--all`.
  `--list --output json` prints one `{"message": "<padded text row>"}` document per row rather than one JSON value.

### Requirements

**Naming commands**

- R1. `xr schema [COMMAND]...` takes the space-separated words that `xr <COMMAND>...` and `xr help <COMMAND>...` take,
  for every command with a registry row.
- R2. `--list` and `--all` name each command by its words. `--list` lists commands only, with no `envelope` row.
- R3. A word that names no command is refused the way `xr help` refuses it: `unknown-command`, exit 2, the word in
  `command`, the nearest sibling in `suggestion`, and a `show-help` step.
- R4. A real command with no typed response keeps today's refusal (`validation`, exit 1, "no typed response"), now
  naming the command by its words. When commands beneath that path have rows, the message names them, so
  `xr schema webhooks` points at `webhooks list`, `webhooks add`, and the rest.
- R5. A name from before this change, meaning a hyphen-joined slug or the `envelope` positional, is refused with
  `invalid-args`, exit 2, and a `fix-input` step carrying the corrected command (D1).
- R6. A bare word that names one of `xr schema`'s scope flags (today `common`, `envelope`, `full`, `list`, `all`,
  `path`) is refused with `invalid-args`, exit 2. For a scope that takes no value, the step is `fix-input` carrying
  `xr schema --<word>`; otherwise it is `show-help`. `envelope` is also a name from before this change, so R5 governs
  it, and this same refusal applies.

**Scopes**

- R7. With command words and no scope flag, `xr schema` prints the command's `data` schema with every `$ref` resolved in
  place, plus `command`, `data_cardinality` (`object` or `array`), and `data_type`.
  `common_schema: "xr schema --common"` is added when the command answers with the API wrapper, and
  `data_nullable: true` when `data` may be null.
- R8. `--full` prints the command's complete standalone schema, byte-equal to its committed file.
- R9. `--common` prints the shared API response wrapper (`data`, `includes`, `meta`, `errors`) and the shared
  definitions it reaches.
- R10. `--envelope` prints the CLI output envelope schema, unchanged.
- R11. `--path <DOT_PATH>` prints one resolved node. With command words the path starts inside `data`, and a leading
  `data.` is accepted; with `--common` it starts at the wrapper. Each step unwraps a nullable `anyOf` and array `items`.
  A segment that does not exist is refused with `invalid-args`, exit 2, the given path in `path`, and the names
  available at the last resolved node in `known_properties`. Its `next_step` is `fix-input` with a `template` that keeps
  the resolved prefix and leaves the last segment as `<property>`, such as
  `xr schema post --path public_metrics.<property>`.
- R12. At most one of command words, `--common`, `--envelope`, `--list`, or `--all` is given; with none, `xr schema`
  keeps today's usage refusal (`validation`, exit 1). `--full` needs command words and excludes `--path`. `--path` needs
  command words or `--common`.

**Pointers**

- R13. The `-h` and `--help` output of every command with a registry row ends with a `Response schema:` block that names
  `xr schema <words>`. Commands without a row print no such block.
- R14. Every shared dry-run envelope of a command with a registry row carries `response_schema: "xr schema <words>"` as
  an additive key, and text output prints it as a `response schema:` line. Raw requests and commands without a row carry
  neither.

**Neighboring surfaces**

- R15. `schema/responses/` holds one file per registry row, named by the row's words joined with `-`, with the content
  `--full` prints. The renames are `redirect-uri-get` → `auth-apps-redirect-uri-get` and `redirect-uri-set` →
  `auth-apps-redirect-uri-set`. The two `skill install` files are removed (D2).
- R16. The `xr validate --schema` aliases and the `typed_alias` coverage of every registry response type stay unchanged.
- R17. `AGENTS.md`, `crates/xurl-cli/README.md`, the examples page, and `xr schema --help` describe the new grammar, and
  each PR leaves them true for the code in that PR.

### Key Decisions

- **Version call and old names: a minor, xr 4.5.0 (D1, Brett chose C).** The whole reshape ships in 4.5.0 alongside
  #421's renames, on blast-radius grounds. Old slugs and the `envelope` positional refuse at exit 2 with a `fix-input`
  step carrying the corrected command. Governs R5.
- **`skill install` is schema-less (D2, Brett: "Make it schema-less").** It prints the installer's own report, as
  `skill update` does, and joins it on the schema-less list (KTD10). Governs R1, R15.
- **A registry row with no schema is `internal`.** A command the resolver returns always has a row, so a lookup that
  finds none means the registry is broken: reason `internal`, with the `report-issue` step.
- **Free-text `validation` refusals carry a `rule` (the merge chain's validation shape).** `no-typed-response` and
  `missing-command` name the two `xr schema` refusals that have no validator, once the OQ1 PR defines the key.
- **The dry-run `command` value stays as it is (D3, Brett: "Keep the slug").** `response_schema` is the only
  new key. Governs R14.
- **For an array, the default body describes one element.** `data_cardinality` says it is an array, so `.properties`
  gives the fields whatever the cardinality. #426 defines `data_cardinality` as a metadata key, which would carry no
  information if the body were the array schema itself. Governs R7.
- **`--all` prints the default scope for every command.** The full copies are the committed files, and #426 puts
  `--full` and `--all` in conflict. Governs R2, R12.
- **An unknown word gets the `xr help` refusal (`unknown-command`, exit 2) rather than today's `validation`, exit
  1.** #424 asks for one command form across `xr`, `xr help`, and `xr schema`, and one refusal follows from that.
  Governs R3.

### Success Criteria

- `xr schema webhooks subscriptions count` prints at most 35 lines. The mean default output across the 138 API-response
  commands is at most 75 lines, against 416 today, measured over the built binary.
- Every command path resolves by its words, and no hyphen-joined slug appears in `--list` or
  `--all`.
- Following the `next_step` of any refusal R3, R5, R6, or R11 defines reaches a successful `xr schema` call in one more
  invocation.

### Scope Boundaries

- The dry-run `command` value is untouched (D3).
- `xr validate --schema` names are untouched (R16).
- No committed copy of the `--common` schema: it is generated from the types on every call, so it cannot drift.
- Considered and not built: accepting old slugs silently and permanently. #424 asks for no hyphenated CLI slugs, and an
  unlisted alias would keep two names alive with nothing pointing at the right one. Evidence that would change this: a
  first-party consumer that cannot move.
- Considered and not built: a footer string written into each command's `after_help`. It would copy each registry name a
  second time, and the registry already owns them (KTD7).
- Considered and not built: a hand-written `$ref` inliner. schemars' `inline_subschemas` setting does the same job
  (KTD3). Evidence that would change this: a view that schemars cannot produce, such as a recursive type that needs
  partial inlining.
- Considered and not built: a test that bounds the mean view length. A large type added later would fail it with no
  defect present. The structural walk in U2 catches the real regression, which is the wrapper leaking into the view.

#### Deferred to Follow-Up Work

- `schema/responses/auth-status.schema.json` describes a bare array of apps, but `xr --output json auth status` prints
  `{"apps": [...], "status": "ok"}`. A separate issue.
- `xr validate --schema nope` reads stdin before it checks the name, so a wrong name waits on input. A separate issue.
- The `xurl-rs-skill` references use `xr schema post --output json | jaq '.properties.data'`,
  `xr schema --list | awk '{print $1}'`, and `xr schema broadcasts-moderators-list`. They need an edit in that repo when
  this releases.

---

## Planning Contract

### Key Technical Decisions

- KTD1. **Registry rows are keyed by command words, and the file name is derived.** `SCHEMA_ENTRIES` and
  `SCHEMA_LESS_COMMANDS` in `crates/xurl-cli/src/cli/commands/schema.rs` hold `"webhooks subscriptions count"`. The file
  stem is the words joined with `-`, computed and never stored, and a walk test holds the derived stems unique.
  `SCHEMA_NAME_OVERRIDES` and `schema_name_for_path` go, because the clap path joined with a space is now the name.
- KTD2. **Words resolve against clap's command tree, the same tree `xr help` walks.** The resolver matches names and
  aliases depth by depth. At the first word that fails, it scores that word against the siblings at that depth with the
  existing Jaro scorer in `crates/xurl-cli/src/cli/classify.rs`, and renders through `render_unknown_command` in
  `crates/xurl-cli/src/cli/runner.rs`, made crate-visible. This follows
  `docs/solutions/design-patterns/unknown-command-needs-two-detectors-one-renderer-and-the-scorer-that-saw-the-word.md`:
  one renderer, and the suggestion comes from whoever scored the word. Once #421 adds
  `crates/xurl-cli/src/cli/retired_verbs.rs`, the resolver consults it the way the unknown-command path does.
- KTD3. **The default view and `--path` read a second generation of the row's type with schemars' `inline_subschemas`
  setting.** schemars 1.2.3 inlines every subschema under that setting and keeps a `$ref` only for a recursive type
  (`SchemaSettings::inline_subschemas`, `schemars-1.2.3/src/generate.rs`), so `xr` writes no inliner. Each registry row
  builds its schema from a `SchemaSettings` it is handed rather than calling `schema_for!`, so one row serves both
  generations: default settings for `--full`, which stays byte-equal to the committed files, and inlined settings for
  the view. The view takes `properties.data` for an API-response row or the root for a CLI-built row, and unwraps array
  `items` and a nullable `anyOf` to the element. `data_cardinality` and `data_nullable` come from that unwrap.
  `data_type` is the element's `$ref` name in the default generation, because an inlined body carries no type name; a
  CLI-built row whose root is an object has no `$ref`, and uses the root `title` (`InstallEnvelope`,
  `RedirectUriGetResponse`). A recursive type would leave `$defs` in the inlined root; the view then keeps those
  definitions, and a walk test shows none exists today. The metadata keys sit at the document root, and the document
  stays a valid schema because JSON Schema 2020-12 Core § 4.3.1 says unknown keywords "SHOULD be treated as
  annotations". The view only renders output: it computes no API response, touches no store, and classifies no failure,
  so it belongs in `crates/xurl-cli` (`AGENTS.md` § "Where a change goes").
- KTD4. **`--common` is the schema of the wrapper over an untyped payload.** It is generated from `ApiResponse`
  instantiated with `serde_json::Value`, so `data` reads as any value and the shared definitions appear once.
- KTD5. **`--path` walks the KTD3 inlined schema, and its refusal adds two error keys.** `path` holds the given dot path
  and `known_properties` the names at the last resolved node. They are named after `schema`/`known_schemas` and
  `host`/`known_hosts` on `ErrorBody` in `crates/xurl-cli/src/cli/envelope.rs`. `schema/output.schema.json` is
  regenerated. The error variant has `additionalProperties: false`, so it must declare both keys.
- KTD6. **Scope rules are clap declarations.** An `ArgGroup` that allows at most one member, and is not required, holds
  command words and the four no-command modes, with `requires` and `conflicts_with` for `--full` and `--path`. A
  required group would turn a bare `xr schema` into a clap error at exit 2, which R12 rules out. A violation then takes
  the existing parse-error path (`invalid-args`, exit 2, a `show-help` step) with no new code. The R6 scope-word set is
  read from the `schema` subcommand's long flags in the clap tree, not written out, so a new scope flag is covered when
  it is declared.
- KTD7. **The help footer is added to the command tree once, from the registry, in the one constructor every parse and
  help render uses.** Each registered path gets `Response schema:` and the indented `xr schema <words>` appended to its
  `after_help`, and to its `after_long_help` when one is set. The decorating function and `pointer_for` live beside the
  registry in `crates/xurl-cli/src/cli/commands/schema.rs`, which owns the names, so no new module is needed. Try a
  `clap` raw-method attribute on `Cli` first: it puts the footer on every `Cli::command()` and `Cli::try_parse_from`
  call, including completions and the tests' `common::command_paths()`, with no call site to remember. If the derive
  applies struct-level methods before the subcommands exist, fall back to one constructor function that every
  help-rendering call site adopts. The help walk (U3) proves whichever one ships.
- KTD8. **The dry-run pointer is set in one place.** The runner parses with `try_get_matches_from` on the decorated
  command, then builds `Cli` with `from_arg_matches`, formatting a failure the way `Cli::try_parse_from` does (clap
  formats the error against the command), so the golden suite pins any change in error text. It keeps the matches so it
  can read the matched subcommand path, asks the registry for the pointer, and hands it to `OutputConfig`. One
  `OutputConfig` helper inserts `response_schema` into the context. `print_dry_run` and `print_dry_run_refusal` in
  `crates/xurl-cli/src/cli/output/mod.rs` both call it first. That matters because the text branch of
  `print_dry_run_refusal` calls `dry_run::summary` directly and never reaches `print_dry_run`.
  `crates/xurl-cli/src/cli/output/dry_run.rs` adds the key to `WORDED` and prints `response schema: <pointer>` after the
  credential line. Raw mode matches no subcommand, so it carries no pointer.
- KTD9. **Structured `--list` output is an array of `{command, type, file}` records.** `file` is the derived stem plus
  `.schema.json`, and text output keeps aligned rows. `scripts/generate-response-schemas.sh` and the drift tests read
  the structured form, because command names now contain spaces and splitting a text row on whitespace cannot recover
  them. The generator calls `xr schema --output json <words>` in U1, and adds `--full` in U2 once the flag exists.
- KTD10. **`skill install` is schema-less (D2).** Both of its rows and both files go, with the flag-form table and the
  `--` handling in `invocation()` that served only them, the two `UNVALIDATED_TYPES` entries, and the report types'
  `JsonSchema` derive. Every row is then named by words alone, and a file's stem is the words joined with `-`.
- KTD11. **Old names are recognized in one function, and refused (D1).** A single token matches a row when it
  equals that row's derived stem, one of the two irregular names (`redirect-uri-get`, `redirect-uri-set`), or the stem
  of a command its family retired (#421's `retired_verbs.rs`), which names its replacement's row. The caller refuses
  with `invalid-args`, exit 2, and `fix-input` carrying `xr schema <words>`. The precedent is `render_misplaced_flags`
  in `crates/xurl-cli/src/cli/runner.rs`: one line that names the problem and ends `Run: <command>`.

### High-Level Technical Design

How `xr schema` resolves its arguments once clap has enforced R12:

```mermaid
flowchart TD
    A[xr schema args] --> B{clap: scope group valid?}
    B -- no --> E1[invalid-args, exit 2, show-help]
    B -- yes --> C{which mode}
    C -- --envelope --> O1[envelope schema]
    C -- --list --> O2[records / aligned rows]
    C -- --all --> O3[words -> default view, every row]
    C -- --common --> P{--path?}
    C -- words --> W{single token?}
    W -- scope word --> E2[invalid-args, fix-input xr schema --word]
    W -- old slug --> E3[invalid-args, fix-input xr schema words]
    W -- otherwise --> R[walk clap tree by words]
    R -- word fails --> E4[unknown-command, exit 2, suggestion]
    R -- path, no row --> E5[validation, exit 1, no typed response]
    R -- row --> S{--full?}
    S -- yes --> O4[full schema = committed file]
    S -- no --> P
    P -- no --> O5[resolved data view + metadata]
    P -- yes --> Q{segment exists?}
    Q -- no --> E6[invalid-args, exit 2, path + known_properties]
    Q -- yes --> O6[resolved node]
```

Flag combinations (R12, enforced by KTD6):

| Given                           | `--full` | `--path`                   | Other modes                                 |
| ------------------------------- | -------- | -------------------------- | ------------------------------------------- |
| words                           | allowed  | allowed, not with `--full` | refused                                     |
| `--common`                      | refused  | allowed                    | refused                                     |
| `--envelope`, `--list`, `--all` | refused  | refused                    | refused                                     |
| nothing                         | refused  | refused                    | today's usage refusal, `validation`, exit 1 |

### Assumptions

These are bets made without a person to confirm them. Each can be revised in review.

- `data_nullable` is added beyond #426's metadata set, because `stream rules add|remove` can answer with `data` null,
  and a filter written from the default view would fail on null.
- Every shared dry-run envelope of a registered command carries `response_schema`, including those that would fail,
  since the schema answers the same question either way.
- `--list` records carry `file` so the generator and the drift tests need no copy of the stem rule.
- The scope-word set for R6 is the flag names, and a walk test holds that no top-level command takes one of those names.

### Sequencing and Overlap with Other Work

The stack is U1 → U2 → U3. U2 and U3 both need U1's words. U3 comes last because it re-blesses the help and dry-run
fixtures that other slots move, so its re-bless runs on the newest `dev`. Each PR starts about 25 CI jobs against the
account's 20-job concurrency cap, so a push of the whole stack queues about 75 jobs. Push the stack once per round and
land it with `gh stack merge` on the top PR, which skips the per-merge restack runs.

| Other work                                                 | Overlap                                                                                                                                                                     | Handling                                                                                                                                                                                               |
| ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| #421, `issues-verb-alignment`                              | Renames `broadcasts scheduled remove`/`live` and `broadcasts sources rename`, three registry rows and their files; adds `retired_verbs.rs`; re-blesses `schema-list.golden` | Its plan merges first. U1 keys rows by the names it finds on `dev` when it rebases; the walk in R1's tests names any row clap no longer parses. U1 consults the retired-verb lookup if present (KTD2). |
| #414, #415, #416, #417, #418, #425, `issues-help-accuracy` | Every `help-*.golden`; #415 reorders options                                                                                                                                | U3's footer is appended after `Examples:`, in a separate hunk from option order. Re-bless after the last rebase.                                                                                       |
| #419, #420, #422, `issues-dry-run-envelopes`               | `dry-run-*.golden`; #420 adds `message` to failing dry runs                                                                                                                 | U3 changes only `print_dry_run`, `WORDED`, and the summary line; #420 works in the validation path. Re-bless after rebase.                                                                             |
| #409, spec refresh                                         | May add typed commands                                                                                                                                                      | A row added after U1 must use words; the R1 walk fails with a `Cause:` line naming the file.                                                                                                           |
| #423, `issues-go-differences-docs`                         | `crates/xurl-cli/README.md`                                                                                                                                                 | Textual conflicts only; resolve by hunk.                                                                                                                                                               |
| `xr` 4.4.1 cut                                             | `dev` moves                                                                                                                                                                 | Rebase before each push.                                                                                                                                                                               |

---

## Implementation Units

### U1. Command words for `xr schema`

- **Goal:** every registry row is reached by its command words, and `--list`, `--all`, the committed files, and the
  generator all name commands that way.
- **Requirements:** R1, R2, R3, R4, R5, R6, R15, R16, R17.
- **Dependencies:** layer 4 of the merge chain. U1's base is the head of the OQ1 PR (issues-dry-run-envelopes), which
  sits on #447; U2 and U3 chain above it, and verb-alignment's second PR sits on U3.
- **Files:**
  - `crates/xurl-cli/src/cli/commands/schema.rs`
  - `crates/xurl-cli/src/cli/mod.rs` (the `Schema` variant: `command: Vec<String>`, its help, `SCHEMA_HELP`)
  - `crates/xurl-cli/src/cli/runner.rs`
  - `crates/xurl-cli/src/cli/classify.rs`
  - `crates/xurl-cli/src/cli/commands/examples.rs`
  - `schema/responses/redirect-uri-get.schema.json` → `schema/responses/auth-apps-redirect-uri-get.schema.json`, and the
    same for `-set`
  - `scripts/generate-response-schemas.sh`
  - `completions/` (regenerated)
  - `crates/xurl-cli/README.md` (§ schema, and the "Which Errors Carry a Step" rows for `invalid-args` and
    `unknown-command`)
  - `AGENTS.md` (§ "Adding a command family", the schema registry bullet)
  - Tests: `crates/xurl-cli/tests/schema_tests.rs`, `crates/xurl-cli/tests/cli_run_tests.rs`,
    `crates/xurl-cli/tests/golden_tests.rs` and `crates/xurl-cli/tests/golden/` (`help-schema`, `schema-list`,
    `examples`, new `reason-*` fixtures), `crates/xurl-cli/tests/next_step_tests.rs`,
    `crates/xurl-cli/tests/agentic_tests.rs`
- **Approach:**
  1. Rewrite the registry keys and the schema-less set as words (KTD1). Drop the override table and
     `schema_name_for_path`. Move `skill install` to the schema-less list (KTD10).
  2. Add the word resolver (KTD2) and the old-name recognizer (KTD11), both pure functions over the registry and
     `Cli::command()`.
  3. Route refusals: unknown word to `render_unknown_command`, scope word and old name to an `invalid-args` envelope
     with its step, and a known path without a row to the existing `validation` refusal. The `envelope` positional
     branch in `run_schema` goes here, replaced by R5 and R6; `--envelope` keeps working.
  4. Make `--list` print records in structured formats and aligned rows in text, with no `envelope` row (KTD9). Make
     `--all` key its values by words.
  5. Rename the two files, rewrite the generator to read `--list --output tsv`, then regenerate the completions.
  6. Update the docs and examples that name slugs. `SCHEMA_HELP` and the examples page show a multi-word command and
     one paired example: `xr schema search` and then
     `xr search "from:jack" --output json | jaq -r '.data[].text'`, so the step from schema to filter is shown in
     context.
- **Execution note:** rewrite the walks in `schema_tests.rs` first, so that every clap path joined with a space is in
  exactly one set, and watch them fail against the slug registry before changing it.
- **Patterns to follow:** `render_misplaced_flags` for an `invalid-args` refusal with a `fix-input` invocation;
  `render_unknown_command` for the unknown word; the `Cause:` walk style already in `schema_tests.rs`.
- **Test scenarios:**
  - `xr schema webhooks subscriptions count --output json` exits 0 and prints the same document
    `webhooks-subscriptions-count.schema.json` holds (until U2 changes the default).
  - `xr schema auth apps redirect-uri get` exits 0. Its document equals `auth-apps-redirect-uri-get.schema.json`, and no
    `redirect-uri-get.schema.json` remains.
  - `xr schema post`, `xr schema search-users`, and `xr schema spaces by-creator` resolve: single words, a hyphenated
    top-level word, and a hyphenated subcommand word.
  - `xr --output json schema webhooks subscriptoins count` exits 2 with `reason: unknown-command`,
    `command: "subscriptoins"`, `suggestion: "subscriptions"`, and a `show-help` step for `xr webhooks --help`.
  - `xr --output json schema post extra` exits 2 with `unknown-command` and `command: "extra"`.
  - `xr --output json schema webhooks` exits 1 with `validation`, and a message naming `webhooks` as having no typed
    response that lists the `webhooks` commands that do.
  - `xr --output json schema webhooks-subscriptions-count` exits 2 with `invalid-args` and
    `next_step: {action: fix-input, command: "xr schema webhooks subscriptions count"}`, and `redirect-uri-get` yields
    `xr schema auth apps redirect-uri get`. The test then runs that `command` and asserts exit 0 and the same document
    as the words form, which proves Success Criteria item 3 for this refusal.
  - `xr --output json schema envelope` exits 2 with `invalid-args` and `fix-input` carrying `xr schema --envelope`;
    `xr schema path` gets a `show-help` step.
  - `xr schema skill install` and `xr schema skill update` exit 1 with the no-typed-response refusal (D2).
  - `xr schema broadcasts scheduled remove` refuses with `suggestion: cancel` from `retired_verbs.rs`, and the old name
    `broadcasts-scheduled-remove` with `fix-input` carrying `xr schema broadcasts scheduled cancel`.
  - A row the registry resolves without a schema refuses with `internal` and `report-issue` (unit tests in `run.rs`
    and `runner.rs`).
  - `xr schema --list --output json` parses as one JSON array of `{command, type, file}`, one per row, with no
    `envelope` entry. `--output tsv` has a header and one row per command.
  - `xr schema --all --output json` keys are exactly the registry's words.
  - Walk: every clap path joined with a space is in exactly one of the registry or the schema-less set, and every
    registry key is a clap path.
  - Walk: no two rows derive the same file stem, every row has its committed file, and no committed file is orphaned.
  - Walk: no top-level command is named after a scope word, with the words read from the `schema` subcommand's flags.
  - Tests this unit and U2 retire or rewrite on purpose: `envelope_schema_via_flag_equals_positional` (U1's R5 and R6
    refusal) and `schema_all_takes_precedence_over_list` (U2's scope group),
    `schema_list_advertises_every_registered_command_plus_envelope` (no `envelope` row), and the slug-keyed walks
    `every_clap_command_is_in_exactly_one_schema_set` and `declared_schema_sets_name_only_commands_clap_parses` (rekeyed
    by words).
  - Walk: every registry `type_name` keeps its validate alias or exemption (R16, the existing test, unchanged).
- **Verification:** `scripts/generate-response-schemas.sh` leaves only the two renames as changes under
  `schema/responses/`. `schema_tests`, `golden_tests`, `next_step_tests`, `cli_run_tests`, `agentic_tests`, and
  `recipe_guard` pass, and the completions freshness check is clean.

### U2. Scopes: the resolved default, `--full`, `--common`, `--path`

- **Goal:** `xr schema <words>` prints the resolved `data` view by default, and the scope flags select every other view.
- **Requirements:** R7, R8, R9, R10, R11, R12, R15, R17.
- **Dependencies:** U1. One PR: D1 puts the default flip in the same minor as the flags.
- **Files:**
  - `crates/xurl-cli/src/cli/commands/schema_resolve.rs` (new: select `data`, unwrap to the element, read the metadata,
    walk a dot path)
  - `crates/xurl-cli/src/cli/commands/schema.rs` (rows build from a `SchemaSettings`; dispatch per scope; `--common`
    source per KTD4)
  - `crates/xurl-cli/src/cli/commands/mod.rs` (module declaration)
  - `crates/xurl-cli/src/cli/mod.rs` (flags `--full`, `--common`, `--path`; the scope `ArgGroup`)
  - `crates/xurl-cli/src/cli/runner.rs` (dispatch arguments)
  - `crates/xurl-cli/src/cli/envelope.rs` (`path`, `known_properties` on `ErrorBody`; the module doc's regeneration
    command uses `--envelope`)
  - `schema/output.schema.json` (regenerated)
  - `scripts/generate-response-schemas.sh` (calls `--full`)
  - `crates/xurl-cli/src/cli/commands/examples.rs`, `AGENTS.md` (§ Output formats, if it names the schema scopes)
  - `crates/xurl-cli/README.md`: the schema section gains the scopes. Its code-generation examples
    (`json-schema-to-typescript`, `datamodel-codegen`) pass `--full`, so the generated types keep the wrapper. Its "All
    schemas for MCP tool definitions" line for `--all` points at `schema/responses/` for the wire schemas, because
    `--all` now prints the default view.
  - `docs/migrating/v4.5.0.md` (new; shaped like `docs/migrating/v3.0.0.md`). It is an old → new table:
    - slug → words;
    - `xr schema envelope` → `--envelope`;
    - `jq '.properties.data'` → `.properties` on the default view, or `--full` to keep `.properties.data`;
    - `xr schema --list | awk '{print $1}'` → `--list --output tsv`;
    - `--all` keys → words.

    The page also warns that `jq '.properties.data'` on the new default prints `null` and exits 0, so a pipeline using
    it fails silently.
  - Tests: `crates/xurl-cli/tests/schema_tests.rs`, unit tests in `schema_resolve.rs`, `crates/xurl-cli/tests/golden/`
    (`help-schema`, `examples`, a new `reason-invalid-args-schema-path` fixture),
    `crates/xurl-cli/tests/agentic_tests.rs` (its `schema envelope` calls become `--envelope`),
    `crates/xurl-cli/tests/next_step_tests.rs`
- **Approach:**
  1. Change each row to build from a `SchemaSettings` it is handed (KTD3), and build the resolver as pure functions over
     a `serde_json::Value` schema, with unit tests against small literal schemas before the registry uses it.
  2. Wire the scopes: the default view, `--full` (today's document), `--common` (KTD4), `--envelope`, and `--path`
     (KTD5).
  3. Declare the scope rules in clap (KTD6).
  4. Point the drift tests and the generator at `--full`. `--all` emits the default view per row.
  5. Add the two `ErrorBody` keys and regenerate `schema/output.schema.json`. Then add the `--path` case to the
     `invalid-args` row of "Which Errors Carry a Step" in `crates/xurl-cli/README.md`: `fix-input` with a template.
     `next_step_tests.rs` holds that table to the golden fixture.
- **Patterns to follow:** `typed_alias` and `SCHEMA_ALIASES` in `crates/xurl-cli/src/cli/commands/validate.rs` for the
  `known_*` list shape; `print_response` for every format.
- **Test scenarios:**
  - `xr schema webhooks subscriptions count --output json` prints `command: "webhooks subscriptions count"`,
    `data_cardinality: "object"`, `data_type: "AccountActivitySubscriptionCount"`,
    `common_schema: "xr schema --common"`, and the four `properties` with no `$ref` and no `$defs`, in at most 35 lines.
  - `xr schema search` prints `data_cardinality: "array"`, `data_type: "Post"`, and Post's properties at the root, with
    `public_metrics` inlined.
  - `xr schema stream rules add` carries `data_nullable: true` and `data_cardinality: "array"`.
  - `xr schema auth status` (a CLI-built row) carries no `common_schema`, and its root resolves `AppStatusEntry`.
  - `xr schema <words> --full --output json` equals the committed file byte for byte, for every row (the drift walk).
  - `xr schema --common` has `properties` `data`, `includes`, `meta`, `errors` and `$defs` holding the eight shared
    definitions.
  - `xr schema --envelope --output json` equals `schema/output.schema.json`.
  - `xr schema post --path public_metrics` and `xr schema post --path data.public_metrics` print the same resolved
    `PostPublicMetrics` node.
  - `xr schema search --path entities` unwraps the array of posts and resolves `entities`.
  - `xr schema --common --path includes.users` prints the resolved `User`.
  - `xr --output json schema post --path public_metrics.nope` exits 2 with `invalid-args`,
    `path: "public_metrics.nope"`, a `known_properties` list containing `like_count`, and
    `next_step: {action: fix-input, template: "xr schema post --path public_metrics.<property>"}`. Filling the template
    with `like_count` exits 0.
  - `xr schema post --path text.length` (through a string) exits 2 with an empty `known_properties`.
  - Clap refusals: `xr schema post --common`, `xr schema --full`, `xr schema --list --all`, and
    `xr schema --envelope --path x` each exit 2 with `invalid-args`.
  - Walk: no row's default view contains a `$ref` or `$defs`, or `includes`, `meta`, or `errors` at its root. This
    catches the wrapper leaking back into the view, and a future recursive type fails with a `Cause:` line naming the
    row. The walk lives in `schema_tests.rs`, a file the recipe already names, so `recipe_guard` stays green.
  - Resolver unit tests:
    - A nullable `anyOf` unwraps to its single non-null branch and sets `data_nullable`.
    - `items` unwraps on an array step.
    - A root with leftover `$defs` keeps them in the view.
    - `data_type` reads the element's `$ref` name from the default generation, and the root `title` for a CLI-built
      object row (`xr schema auth apps redirect-uri get` gives `RedirectUriGetResponse`).
- **Verification:** the drift walk passes against `--full`, and `scripts/generate-response-schemas.sh` changes no file.
  `schema/output.schema.json` differs only by the two keys. Measured on the built binary,
  `xr schema webhooks subscriptions count` prints at most 35 lines and the mean view over the API-response rows is at
  most 75 lines. That is a check made at build time, not a test, because a large type added later would move the mean
  without any defect.

### U3. Schema pointers in help and dry runs

- **Goal:** a caller looking at a typed command's help or dry run finds the `xr schema` call that describes its output.
- **Requirements:** R13, R14, R17.
- **Dependencies:** U1. U2 is not needed, but the stack order puts U3 last.
- **Files:**
  - `crates/xurl-cli/src/cli/commands/schema.rs` (`pointer_for(path)` and the tree decoration, beside the registry)
  - `crates/xurl-cli/src/cli/mod.rs` (the raw-method attribute on `Cli`, KTD7)
  - `crates/xurl-cli/src/cli/runner.rs` (parse keeping the matches; the matched path reaches `OutputConfig`, KTD8)
  - Only under KTD7's fallback, the other `Cli::command()` call sites that render help:
    `crates/xurl-cli/src/cli/reparse.rs`, `crates/xurl-cli/src/cli/classify.rs`,
    `crates/xurl-cli/src/cli/misplaced_flags.rs`
  - `crates/xurl-cli/src/cli/output/mod.rs` (`OutputConfig` field, the pointer helper, `print_dry_run`,
    `print_dry_run_refusal`)
  - `crates/xurl-cli/src/cli/output/dry_run.rs` (`WORDED`, the summary line, unit tests)
  - `crates/xurl-cli/src/cli/envelope.rs` or wherever the dry-run variant of the envelope schema is declared (document
    `response_schema`), and `schema/output.schema.json`
  - `crates/xurl-cli/README.md` (§ Dry Runs), `AGENTS.md` (§ Output formats, the dry-run paragraph)
  - Tests: `crates/xurl-cli/tests/dry_run_guard.rs`, `crates/xurl-cli/tests/golden_tests.rs` and the re-blessed
    `help-*.golden` and `dry-run-*.golden`
- **Approach:**
  1. Add `pointer_for` and decorate the tree once (KTD7).
  2. Parse keeping the matches, compute the pointer from the matched path, and hand it to `OutputConfig` (KTD8).
  3. Insert the key through the one helper both dry-run printers call, and render the text line.
  4. Re-bless the help and dry-run fixtures only after rebasing on the newest `dev`.
- **Execution note:** add the two walks first (help footer iff registered; dry-run pointer iff registered) and watch
  them fail before decorating anything.
- **Patterns to follow:** the clap-driven walk in `crates/xurl-cli/tests/dry_run_guard.rs`, which already runs every
  command under `--dry-run` from synthesized arguments; the `family_help.rs` rendering style for the footer's
  indentation.
- **Test scenarios:**
  - `xr webhooks subscriptions count -h` and `--help` both end with `Response schema:` and
    `xr schema webhooks subscriptions count`.
  - `xr help webhooks subscriptions count` prints the same footer.
  - `xr webhooks -h` (a family with no row) and `xr auth oauth2 --help` (schema-less) print no footer.
  - `xr skill install --help` prints no footer (D2).
  - With a bearer token set, `xr --dry-run --output json webhooks subscriptions count` carries
    `response_schema: "xr schema webhooks subscriptions count"` next to `auth` and `command`. In text output a
    `response schema: xr schema webhooks subscriptions count` line follows the credential line.
  - With an empty store, the same dry run (a credential refusal, exit 77 in the envelope) also carries
    `response_schema`. In text output the refusal's lines include `response schema:`, which proves the text branch of
    `print_dry_run_refusal` goes through the helper.
  - `xr --dry-run --output json /2/users/me` carries no `response_schema`.
  - A parse failure that the golden suite pins (an unknown flag, a missing value) renders byte-identical to `dev` after
    the runner moves to `try_get_matches_from`.
  - Walk (help), in `golden_tests.rs`: for every clap path, the help ends with the footer iff the path has a registry
    row.
  - Walk (dry run), in `dry_run_guard.rs`: for every command it runs that answers with the shared envelope,
    `response_schema` is present iff the path has a row, and equals `xr schema <path>`. The installer's own report
    from `skill install` and `skill update` carries none, which is what a command with no row carries.
  - Unit (`dry_run.rs`): `response_schema` is never printed as an input line, and the `response schema:` line comes
    after `credential:`.
- **Verification:** the help and dry-run walks pass. The re-blessed fixtures differ from `dev` only by the footer and
  the `response_schema` key or line. `schema/output.schema.json` documents the key.

---

## Verification Contract

| Gate                    | Command                                                                                                                             | Applies to              |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| Schema walks and drift  | `cargo test -p xurl-rs --test schema_tests`                                                                                         | U1, U2                  |
| Golden fixtures         | `cargo test -p xurl-rs --test golden_tests`; re-bless only moved fixtures with `XURL_GOLDEN_BLESS=1`                                | U1, U2, U3              |
| Dry-run guard           | `cargo test -p xurl-rs --test dry_run_guard`                                                                                        | U3                      |
| Step table and reasons  | `cargo test -p xurl-rs --test next_step_tests`                                                                                      | U1, U2                  |
| Recipe and agentic docs | `cargo test -p xurl-rs --test recipe_guard --test agentic_tests --test cli_run_tests`                                               | every unit              |
| Full suite              | `cargo test --workspace --no-fail-fast`                                                                                             | every unit, before push |
| Lint and format         | `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --check`                                                        | every unit              |
| Generated artifacts     | `scripts/generate-response-schemas.sh`, `scripts/generate-completions.sh`; then `git status` shows only intended changes            | U1, U2                  |
| Local CI mirror         | `scripts/hooks/pre-push`, run in the background                                                                                     | every unit, before push |
| Changelog bump          | `scripts/check-surface-bump.sh check <base> <head> <pr-body>` passes with the `### Added` entries                                   | every PR                |
| Measured size           | by hand on the built binary: `xr schema webhooks subscriptions count \| wc -l`, and the mean view length over the API-response rows | U2                      |

No gate sends a request to X. Tests use the in-crate mock and wiremock, and every by-hand run sets `XURL_TOKEN_STORE` to
a scratch path and `API_BASE_URL` to a closed port.

---

## Definition of Done

- Every unit's test scenarios exist, and each walk was observed failing before its fix.
- Every gate in the Verification Contract passes for the PR's head, and every CI check on each PR reads `SUCCESS` in the
  rollup.
- `schema/responses/` changes only as R15 names, and `schema/output.schema.json` only by the keys KTD5 and KTD8 add.
- Each PR's `## Changelog (xurl-rs)` files its entries under `### Added`, the section `check-surface-bump.sh` accepts
  for a minor's surface growth (the #315 precedent), and states plainly what an old invocation gets. Each PR's body
  states the minor call and its reason, and its Related Issues names #424, #426, or both.
- Re-blessed fixtures are only those this work moves, re-blessed after the last rebase on `dev`.
- No code from abandoned approaches is left in the diff, and no new comment breaks the repo's comment policy.

---

## Open Questions

Sent to Brett through the release session. Each one has a recommended default that the build uses for any unit the
question does not block.

- **D1, version call and old names: answered, C.** Everything ships in the minor xr 4.5.0 by Brett's decision, on
  blast-radius grounds, with #421's renames in the same release; #421 merges first, and U1 rekeys the rows for whatever
  names it leaves. Old slugs and the `envelope` positional refuse through `fix-input`. RELEASES.md § Versioning lists
  positional forms and `xr schema` output as contract, so each PR body states the call and its reason.
- **D2, the `skill install --all` row: answered, schema-less.** Brett: "Make it schema-less". `skill install` joins
  `skill update` on the schema-less list (KTD10), and `xr schema skill-install` from 4.4 refuses as `unknown-command`,
  which fits the D1 minor. The JSON reports are unchanged and stay pinned by their golden fixtures.
- **D3, the dry-run `command` value: answered, unchanged.** Brett: "Keep the slug". `response_schema` is the only new
  key.

---

## Sources

- Issues #424 and #426, and the reproduction above.
- `RELEASES.md` § Versioning (contract: invocation, positional forms, `xr schema` output) and `RELEASES-PREFLIGHT.md`
  (the `xr schema` shape diff against the last tag).
- `docs/solutions/design-patterns/unknown-command-needs-two-detectors-one-renderer-and-the-scorer-that-saw-the-word.md`
  (KTD2).
- `docs/solutions/workflow-issues/partition-a-shared-file-by-disjoint-symbols-for-parallel-development.md` (the U3
  and #420 split in `output/`).
- `docs/plans/2026-10-28-1849-feat-sibling-verb-alignment-plan.md` (#421's renames and `retired_verbs.rs`).
- `docs/migrating/v3.0.0.md` (the shape of a migration page for a major).
- JSON Schema 2020-12 Core § 4.3.1 (unknown keywords are annotations), and `schemars-1.2.3/src/generate.rs`
  (`inline_subschemas`).

---

## Review Record

Every review ran inline in this session. Brett's standing rule for this repo forbids subagents, so no reviewer agent and
no outside model was dispatched. Nobody watched this session, so each choice below was settled from the evidence named
beside it, and only the decisions that are Brett's (D1–D3) stayed open. All three are answered (Open Questions).

### Engineering review (2026-10-28, `/plan-eng-review`, plan at `6b5fa41`)

Target: this plan. Mode: FULL_REVIEW, with scope accepted as-is. Prior learning applied:
`unit-ordering-breaks-zero-broken-tests` (confidence 10/10, 2026-04-02). Each unit already carries its own test changes.

**Scope Challenge.** The plan proposes 2 new modules and about 25 changed source, script, and doc files, plus about 160
re-blessed fixtures, so the complexity gate tripped. No feature was cut, because both issues are in scope by the brief.
The smaller arrangement drops a separate pointer module: decoration and `pointer_for` live beside the registry, which
owns the names (KTD7).

- S1. [INFORMATIONAL] (9/10) `schemars-1.2.3/src/generate.rs:46-51`, "Inline all subschemas instead of using references.
  Some references may still be generated in schemas for recursive types." The draft hand-wrote an inliner. Folded into
  KTD3 [Layer 1].
- S2. [INFORMATIONAL] (8/10) Structure: no `schema_pointer.rs`. Folded into KTD7 and U3's files.

**1. Architecture.**

- A1. [CRITICAL] (9/10) `crates/xurl-cli/src/cli/output/mod.rs:533-541`: the text branch of `print_dry_run_refusal`
  calls `dry_run::summary(false, exit_code, ctx, Some(&refusal))` and returns, so it never reaches the
  `self.print_dry_run(...)` at :554. The draft's KTD8 claimed the refusal path "reaches it through that call", which is
  true only for structured output, so a text credential refusal would drop the `response schema:` line. Folded: one
  helper inserts the key, both printers call it first, and U3 adds the text-refusal scenario.
- A2. [INFORMATIONAL] (8/10) `crates/xurl-cli/src/cli/runner.rs:177`, `Cli::try_parse_from(args_vec.iter())`, discards
  the matches that KTD8 needs. Folded: parse with `try_get_matches_from` plus `from_arg_matches`, keep clap's error
  formatting, and U3 adds a golden parity scenario.
- A3. [INFORMATIONAL] (8/10) R6's scope words would be a hand-synced copy of the flag names (red flag 8). Folded: read
  them from the `schema` subcommand's long flags (KTD6).
- Design twice, for `schema_resolve.rs`. Shape A is free functions over a schema `Value`. Shape B is scope methods on a
  registry row, which hide the two generations and the metadata behind one call. Pick: B's interface (the row answers a
  scope), with A's pure helpers inside it, because the caller then never chooses generation settings. Red-flag screen:
  flag 8 (hand-synced list) is fixed by A3, and no other flag applies.
- Production failure per new path:
  - #421 renames a word → the resolver consults `retired_verbs.rs` (KTD2).
  - A recursive type arrives → the U2 walk fails, naming the row.
  - A help path misses the decoration → the U3 help walk fails.
  - A command prints its own dry-run report → the U3 walk names it exempt.

**2. Code quality.**

- Q1. [INFORMATIONAL] (8/10) The draft's mean-line-count walk would fail when a large type is added, with no defect
  present. Folded: a structural walk (no `$ref`, no `$defs`, no wrapper keys), and the 75-line mean becomes a check at
  build time.
- Q2. [INFORMATIONAL] (9/10) `crates/xurl-cli/tests/recipe_guard.rs:65` treats any test file containing `Cause:` as a
  walk that `AGENTS.md` must name. Folded: the new walks live in `schema_tests.rs`, `dry_run_guard.rs`, and
  `golden_tests.rs`, which the recipe already names.
- Q3. [INFORMATIONAL] (7/10) Under D2-a, `xr schema skill install --all` without `--` is a clap conflict. Folded: U1
  adds that scenario, and `xr schema --help` documents `--`.

**3. Tests.** Framework: Rust (`cargo test`, `rstest`, `assert_cmd`, golden fixtures, wiremock).

```text
CODE PATHS                                         USER FLOWS
[+] commands/schema.rs (U1, U2)                    [+] agent: command -> its data shape
  ├── word resolver                                  ├── [★★★ PLANNED] words -> view (U2 scenarios)
  │   ├── [★★★ PLANNED] hit, alias, hyphen word      ├── [★★★ PLANNED] help footer -> xr schema (U3 walk)
  │   ├── [★★★ PLANNED] unknown word + suggestion    └── [★★★ PLANNED] dry run -> response_schema (U3 walk)
  │   └── [★★★ PLANNED] path without a row         [+] refusals an agent recovers from
  ├── old-name recognizer (D1)                       ├── [★★★ PLANNED] old slug -> fix-input (U1)
  │   └── [★★★ PLANNED] stem, 3 irregulars           ├── [★★★ PLANNED] scope word -> fix-input (U1)
  ├── scopes                                         └── [★★★ PLANNED] bad --path -> known_properties (U2)
  │   ├── [★★★ PLANNED] view, nullable, CLI-built
  │   ├── [★★★ PLANNED] --full = file (drift walk)
  │   ├── [★★  PLANNED] --common, --envelope
  │   └── [★★★ PLANNED] --path hit, miss, scalar
  └── --list / --all records [★★ PLANNED]
[+] output/ (U3)
  ├── [★★★ PLANNED] pointer in success + refusal, json and text
  └── [★★★ PLANNED] raw mode carries none
[+] runner.rs parse change
  └── [★★★ PLANNED] golden parity of parse errors (existing suite)
COVERAGE: 17/17 planned paths have a scenario | GAPS closed in review: 3 (A1 text refusal, A2 parse parity, Q3 flag form)
```

Regression: single-word names (44) and every `--full` document stay byte-identical, held by the drift walk. U1 lists the
intentional changes: the retired and rewritten tests. Value card for the one critical path: `Value: protects=a text
credential refusal names the schema; fails_when=print_dry_run_refusal's text branch skips the helper; why_new=no dry-run
test checks text refusals for a context key; seam=none; no_claim=X accepting the credential`.

**4. Performance.** No issues found. `xr schema` makes two schema generations per call, and `--all` makes 288
[estimated: microseconds each, the same order as today's 144]. Decorating the clap tree adds about 144 string appends to
each `Cli::command()` build [estimated: under a millisecond], on a path that already builds the whole tree.

**Outside voice:** disabled (`codex_reviews` is `disabled`), so it was skipped, and no native replacement ran, under the
no-subagents rule.

**Reality rows.**

| Premise                                          | Receipt                                                               | Verdict   |
| ------------------------------------------------ | --------------------------------------------------------------------- | --------- |
| `xr schema <words>` exits 2 today                | repro run on `b50850d`                                                | true      |
| 144 names: 97 multi-word, 44 single, 3 irregular | `crates/xurl-cli/src/cli/commands/schema.rs:40-487` and the clap walk | true      |
| Mean 416.0 lines, 8 identical wrapper defs       | `schema/responses/*.schema.json`, measured                            | true      |
| Dry-run `command` is hand-written per arm        | `crates/xurl-cli/src/cli/commands/lists.rs:74` and siblings           | true      |
| `--all` beats `--list`                           | `crates/xurl-cli/src/cli/commands/schema.rs:614-623`                  | true      |
| The text refusal skips `print_dry_run`           | `crates/xurl-cli/src/cli/output/mod.rs:533-541`                       | true (A1) |
| The error variant is closed                      | `schema/output.schema.json`, `additionalProperties: false`            | true      |
| schemars inlines all but recursive types         | `schemars-1.2.3/src/generate.rs:46-51`                                | true      |
| Unknown keywords are annotations                 | JSON Schema 2020-12 Core § 4.3.1                                      | true      |

```text
REALITY: premise-table pass 9 premises, unverified=0 crates/xurl-cli/src/cli/output/mod.rs:533
REALITY: already-done pass no commit on origin/dev touches schema words or response_schema crates/xurl-cli/src/cli/commands/schema.rs:606
REALITY: surface-check pass every named command, script, test file, and fixture exists; new flags are proposals crates/xurl-cli/src/cli/mod.rs:1922
REALITY: binding-decisions pass no branch decision conflicts; library-first check passes (KTD3) AGENTS.md:1
REALITY: numbers pass figures measured on b50850d; 35- and 75-line bounds are estimates checked at build schema/responses/webhooks-subscriptions-count.schema.json:1
REALITY: deferred-asks finding #424's dry-run example implies words in command; held as D3 crates/xurl-cli/src/cli/commands/lists.rs:74
REALITY: pr-count pass 3 PRs as the brief directs; about 25 jobs per PR against a 20-job cap .github/workflows/ci.yml:1
```

**Decision ledger.** Every remedy above was approved by an evidence-settled auto-decision, under the brief's instruction
to answer review questions from evidence: S1, S2, A1, A2, A3, Q1, Q2, Q3. D1, D2, and D3 are Brett's and remain open
(Open Questions). Approval readiness: PASS for S1–S2, A1–A3, Q1–Q3. D1–D3 are pending with the owner.

**NOT in scope:** the `auth status` schema mismatch and validate's stdin ordering (separate issues), the `xurl-rs-skill`
edits (another repo, at release), and the dry-run `command` value (D3).

**What already exists:** `render_unknown_command` and `render_misplaced_flags` (`runner.rs`), the Jaro scorer
(`classify.rs`), the clap-driven walk (`dry_run_guard.rs`), `print_response` and `write_flattened` for every format, and
schemars' inlining. The plan reuses each one and builds none of them again.

**Failure modes:** 0 critical gaps. Every new path has a planned test and a visible error.

**Parallelization:** sequential implementation, no parallelization opportunity. U2 and U3 both start from U1, and the
repo forbids worktrees.

### Document review (2026-10-28, `/compound-engineering:ce-doc-review`, non-interactive, plan at `ca56249`)

The lenses ran inline in this session, because Brett's no-subagent rule rules out dispatch. That means no lens had an
independent context, and agreement between lenses promoted nothing. The lenses were coherence and feasibility (always
on), scope-guardian (every plan), adversarial (a bootstrapped plan with no validated upstream contract, touching a
public contract), and product-lens (the plan takes output-shape positions). security-lens and design-lens had no
signal. The cross-model pass did not run: the no-outside-model rule, and `codex_reviews` is disabled.

Applied, each needed to carry out a decision the plan already made, under the caller's grant to correct implementation
and verification sections:

- R12 and KTD6: "exactly one" scope contradicted the table and the kept no-argument refusal. Now at most one, with a
  group that is not required, so a bare `xr schema` stays `validation` at exit 1. (coherence, feasibility)
- U1's dependencies and the stop condition: one said build on defaults and the other said stop. Now D1 and D2 block
  opening a PR, not building, and the dependent code stays in KTD10's row and KTD11's function. (coherence)
- R5, R6, U1, and U2: U1's tests expected the `envelope` positional refusal while U2 removed the branch. U1 now owns it,
  and R6 defers to R5 for `envelope` under D1-B. (coherence)
- KTD9: the generator named `--full` in U1, before U2 adds the flag. (coherence, feasibility)
- KTD3: `data_type` had no source for a CLI-built object row, which has no `$ref`. It uses the root `title`, verified
  on `skill-install.schema.json` and `redirect-uri-get.schema.json`, with a unit test. (feasibility)
- U2's README edits: the code-generation examples pass `--full`, and the `--all` "MCP tool definitions" line points at
  `schema/responses/`, since the default view drops the wrapper those uses need. (product-lens)
- The flowchart: a single ordinary word had no edge; the `no` label is now `otherwise`. (coherence)
- U2's migration page: the silent-`null` warning had merged into the last table row. (coherence)

Considered and kept, with reasons: `data_nullable` (without it the element view would hide that `data` can be null);
old-name handling (the brief asked how old names behave); R4's list of commands (the agent-ready contract wants a fix
the agent can act on). Proposed fixes awaiting confirmation: none. Decisions for the user: D1–D3 only, unchanged.
Dropped: 3, below the evidence bar (a stop-condition reading during rebase, TSV column order, and an array-view premise
already recorded as a Key Decision).

## Implementation Tasks

- [ ] **T1 (P1, human: ~2h / CC: ~10min)**: output: a pointer helper called by both dry-run printers
  - Surfaced by: Architecture, A1, `output/mod.rs:533-541`
  - Files: `crates/xurl-cli/src/cli/output/mod.rs`, `crates/xurl-cli/src/cli/output/dry_run.rs`
  - Verify: the U3 text-refusal scenario
- [ ] **T2 (P1, human: ~3h / CC: ~15min)**: runner: parse keeping the matches, with clap's error formatting
  - Surfaced by: Architecture, A2, `runner.rs:177`
  - Files: `crates/xurl-cli/src/cli/runner.rs`
  - Verify: `cargo test -p xurl-rs --test golden_tests` unchanged before any re-bless
- [ ] **T3 (P1, human: ~4h / CC: ~15min)**: schema: rows build from `SchemaSettings`; the view uses `inline_subschemas`
  - Surfaced by: Scope Challenge, S1
  - Files: `crates/xurl-cli/src/cli/commands/schema.rs`, `crates/xurl-cli/src/cli/commands/schema_resolve.rs`
  - Verify: the U2 resolver unit tests and the structural walk
- [ ] **T4 (P2, human: ~1h / CC: ~5min)**: schema: derive the R6 scope words from the clap flags
  - Surfaced by: Architecture, A3
  - Files: `crates/xurl-cli/src/cli/commands/schema.rs`
  - Verify: the U1 scope-word walk
- [ ] **T5 (P2, human: ~1h / CC: ~5min)**: tests: a structural walk instead of a line-count walk, kept in files the
  recipe names
  - Surfaced by: Code quality, Q1 and Q2
  - Files: `crates/xurl-cli/tests/schema_tests.rs`
  - Verify: `cargo test -p xurl-rs --test schema_tests --test recipe_guard`
- [x] **T6 (P2, human: ~30min / CC: ~5min)**: schema: document `--` for the flag form; dropped, as D2 leaves none
  - Surfaced by: Code quality, Q3
  - Files: `crates/xurl-cli/src/cli/mod.rs`, `crates/xurl-cli/tests/schema_tests.rs`
  - Verify: the U1 flag-form scenarios

## Developer Experience Review

`/plan-devex-review`, 2026-10-28, plan at `2300d0f`, mode DX POLISH (an improvement to a product already in use). Run
inline, with no subagent and no outside model. Each gate was settled from evidence because nobody watched the session.
Product type: CLI tool (`xr`, with the `xdk` library downstream).

### Developer Persona Card

```text
TARGET DEVELOPER PERSONA
========================
Who:       an AI coding agent driving xr from a shell; second, a human X API developer at a terminal
Context:   about to write a jq or jaq filter over a typed command's --output json, or to check what a command returns
Tolerance: one failed call; it reads errors literally, cannot ask, and pays per token for every line it reads
Expects:   the words it runs the command with also name the schema; refusals carry the corrected command
```

### Developer Empathy Narrative

Observed on `b50850d` unless marked predicted. I have just run `xr webhooks subscriptions count -h` and want the shape
of `.data` before I write a filter. Nothing in the help mentions a schema. I know `xr schema` exists from `xr --help`,
so I try `xr schema webhooks subscriptions count`. It exits 2 with "unexpected argument 'subscriptions'". I run
`xr schema --list` and scan 145 rows until I find `webhooks-subscriptions-count`. That works and prints 426 lines: the
full `Post` and `User` schemas, which this endpoint never returns. `.properties.data` is a `$ref`, so I chase it into
`$defs` to find four string fields. Predicted: on `xr schema search` I would have had to notice that `data` is an array
by reading `type` beside the `$ref`. That is three calls and more than 400 lines to learn four field names, and the dry
run I ran earlier gave no hint that any of this existed.

### Competitive DX Benchmark

Clock: from "I want the shape of this command's output" to "I know the field names and whether `data` is an array", for
the agent persona, with `xr` installed. The time column is an estimate from the call and line counts.

| Tool                | Start → result                     | Calls and evidence type           | DX choice                                                       | Source                                         |
| ------------------- | ---------------------------------- | --------------------------------- | --------------------------------------------------------------- | ---------------------------------------------- |
| `kubectl explain`   | resource and dot path → field docs | 1 call, reported                  | dot path into a resource (`pod.spec.containers`), `--recursive` | in-distribution knowledge, not re-checked here |
| AWS CLI             | operation → output skeleton        | 1 call, reported                  | `--generate-cli-skeleton output` on the operation itself        | in-distribution knowledge, not re-checked here |
| `gh`                | command → JSON fields              | 1 call, reported                  | `--json` with no value lists the field names                    | in-distribution knowledge, not re-checked here |
| `xr` 4.4.0          | words → fields                     | 3 calls, 400+ lines, observed     | slug positional, full wrapper, `$ref` chasing                   | repro on `b50850d`                             |
| `xr` with this plan | words → fields                     | 1 call, about 30 lines, estimated | words positional, resolved `data` view, `--path`                | this plan                                      |

Target: Champion (one call, well under 2 minutes). The plan reaches that: the help footer or the dry-run pointer names
the call, and that call answers. Measuring the line count is a build-time check under U2.

### Magical Moment Specification

The moment is when the agent reads `Response schema: xr schema webhooks subscriptions count` under a command's help or
in its dry run, runs exactly that, and sees the four fields `jq '.data.<field>'` will meet, with no `$ref` to chase.
Vehicle: the U3 footer and pointer, plus the U2 default view. Both are already planned. The review added the paired
example in `SCHEMA_HELP` and on the examples page, so the step from schema to filter is shown, not inferred.

### Developer Journey Map

```text
STAGE           | DEVELOPER DOES                               | FRICTION POINTS                         | STATUS
----------------|----------------------------------------------|-----------------------------------------|---------
1. Discover     | reads -h or a dry run of a typed command     | no pointer today                        | fixed (U3)
2. Install      | already has xr                               | none                                    | ok
3. Hello World  | xr schema <the words it ran>                 | slug translation, 426 lines             | fixed (U1, U2)
4. Real Usage   | xr schema <words> --path a.b, writes a filter| array cardinality must be noticed       | ok (data_cardinality; paired example added)
5. Debug        | follows a refusal's next_step                | --path miss had no step; R4 named none  | fixed (R11 template, R4 lists commands)
6. Upgrade      | moves scripts from 4.4 forms                 | jq '.properties.data' turns null, exit 0| fixed in docs (migration table); D1 open
```

### First-Time Developer Confusion Report

```text
FIRST-TIME DEVELOPER REPORT
============================
Persona: AI coding agent
Attempting: learn the output shape of `xr search` after this plan ships

T+0:00  Runs `xr search -h`; the last lines read "Response schema:  xr schema search".          [addressed: U3]
T+0:05  Runs `xr schema search`; sees data_cardinality "array", data_type "Post", Post's fields. [addressed: U2]
T+0:10  Tries `xr schema search --path public_metrics.likes`; refused with known_properties and
        a template ending `public_metrics.<property>`; fills it with like_count; succeeds.    [addressed: R11 step]
T+0:20  Tries `xr schema webhooks` (a family); the message lists the webhooks commands with a
        schema.                                                                              [addressed: R4]
T+0:30  A script from 4.4 runs `xr schema post | jq '.properties.data'` and gets null.         [addressed in docs: migration table]
```

### Pass Summary

| Pass               | Before | After | Finding and disposition                                                                                                    |
| ------------------ | ------ | ----- | -------------------------------------------------------------------------------------------------------------------------- |
| 1. Getting Started | 7      | 8     | X3: paired schema-to-filter example in `SCHEMA_HELP` and the examples page. Folded into U1.                                |
| 2. API/CLI design  | 8      | 8     | No issues. Words match `xr help`, the default is the useful view, and every escape hatch is a flag.                        |
| 3. Error messages  | 6      | 8     | X1 (CAP6 cleared): `--path` miss had no `fix`; now `fix-input` with a template (R11). X2: R4 names the commands with rows. |
| 4. Documentation   | 6      | 8     | X4: the migration page is an old → new table with the silent-`null` warning. Folded into U2.                               |
| 5. Upgrade path    | 5      | 7     | X5: under D1-B the deprecation note reaches stderr in every format (KTD11). The residual is D1 itself, Brett's.            |
| 6. Dev environment | 7      | 7     | X7: Windows PowerShell 5.1 may strip a bare `--`; noted on D2 (medium confidence, unverified).                             |
| 7. Community       | 7      | 7     | No issues. `xurl-rs-skill` edits are a deferred follow-up.                                                                 |
| 8. DX measurement  | 6      | 7     | X6: U1 runs the fix-input command an old slug returns and asserts it succeeds. Line counts are measured at build.          |

Error path matrix (agent-ready-v1; CLI channel = stderr, distinct exit, JSON envelope):

```text
Path                          | Class      | what | fix  | recovery | code | context | trace | redaction | channel | Cap
unknown word (R3)             | usage      | pass | pass | implied  | pass | N/A     | N/A   | pass      | pass    | none
old slug, D1-A (R5)           | usage      | pass | pass | implied  | pass | N/A     | N/A   | pass      | pass    | none
scope word (R6)               | usage      | pass | pass | implied  | pass | N/A     | N/A   | pass      | pass    | none
--path miss (R11), as drafted | usage      | pass | fail | implied  | pass | N/A     | N/A   | pass      | pass    | CAP6:--path-miss:fix
--path miss (R11), amended    | usage      | pass | pass | implied  | pass | N/A     | N/A   | pass      | pass    | none
schema generation failure     | unexpected | pass | pass | pass     | pass | pass    | pass  | pass      | pass    | none (existing report-issue path)
```

N/A reasons: context and trace do not apply to a usage error rejected before any work begins. The corrected `--path`
line in text output is `no property 'nope' under 'public_metrics'; it has like_count, reply_count, ... Run: xr schema
post --path public_metrics.<property>`.

### NOT in scope (DX)

- Shell completion of command words after `xr schema` (`[COMMAND]...` is free text to clap). A DX expansion, not polish.
- A `data_path` hint such as `.data[]` in the view: `data_cardinality` already says it, and the paired example shows it.
- `data_type` and `data_cardinality` in `--list` records: #426 asks for the response type there, and the view carries
  both.

### What already exists (DX)

The unknown-command renderer and its suggestion scorer, `render_misplaced_flags`'s `Run: <command>` shape, the
`next_step` contract with `template` for placeholders (`register-app`), `print_response` for every format, the dry-run
text summary, and `family_help.rs` for example layout.

### DX Implementation Checklist

```text
DX IMPLEMENTATION CHECKLIST
============================
[ ] One call from a command's words to its fields (U1, U2)
[ ] Every typed command's help and dry run names that call (U3)
[ ] Every new refusal carries what, fix, code and channel; --path miss carries a template (R11)
[ ] xr schema --help and the examples page show the schema-to-filter pair (U1)
[ ] Migration table covers slug, envelope, .properties.data, --list parsing, --all keys (U2)
[ ] The old-slug refusal's command runs green in a test (U1)
[ ] Works with no terminal: no prompt anywhere in xr schema (existing)
[ ] Changelog entries filed under ### Added, with the minor call stated in the body (every PR)
```

### DX Implementation Tasks

- [ ] **T7 (P1, human: ~1h / CC: ~5min)**: schema: `--path` refusal carries `fix-input` with a template
  - Surfaced by: Pass 3, X1 (CAP6:--path-miss:fix)
  - Files: `crates/xurl-cli/src/cli/commands/schema.rs`, `crates/xurl-cli/README.md`,
    `crates/xurl-cli/tests/next_step_tests.rs`
  - Verify: the U2 `--path` miss scenario, including the filled template exiting 0
- [ ] **T8 (P2, human: ~30min / CC: ~5min)**: schema: the R4 refusal names the commands beneath the path that have rows
  - Surfaced by: Pass 3, X2
  - Files: `crates/xurl-cli/src/cli/commands/schema.rs`
  - Verify: the U1 `xr schema webhooks` scenario
- [ ] **T9 (P2, human: ~1h / CC: ~10min)**: docs: paired example in `SCHEMA_HELP` and the examples page; migration table
  - Surfaced by: Passes 1 and 4, X3 and X4
  - Files: `crates/xurl-cli/src/cli/mod.rs`, `crates/xurl-cli/src/cli/commands/examples.rs`, `docs/migrating/v4.5.0.md`
  - Verify: `help-schema.golden` and `examples.golden` re-blessed with the new lines; `agentic_tests` green
- [ ] **T10 (P2, human: ~30min / CC: ~5min)**: tests: run the old-slug refusal's `command` and assert it succeeds
  - Surfaced by: Pass 8, X6
  - Files: `crates/xurl-cli/tests/schema_tests.rs`
  - Verify: `cargo test -p xurl-rs --test schema_tests`

**Decision record (DX).** Each gate was settled from evidence: product type CLI (AGENTS.md § Running xr), persona agent
plus human (the brief), target Champion (the plan's one-call path), vehicle footer plus pointer (in plan), mode POLISH
(enhancement). X1–X7 were all within the accepted scope, as routine work or follow-through of R3–R11, and were folded
in. No DX choice is Brett's beyond D1–D3.

## DX Review Scorecard

```text
+====================================================================+
|              DX PLAN REVIEW — SCORECARD                             |
+====================================================================+
| Dimension            | Score  | Prior  | Trend  |
|----------------------|--------|--------|--------|
| Getting Started      |  8/10  |  7/10  |   ↑    |
| API/CLI/SDK          |  8/10  |  8/10  |   =    |
| Error Messages       |  8/10  |  6/10  |   ↑    |
| Documentation        |  8/10  |  6/10  |   ↑    |
| Upgrade Path         |  7/10  |  5/10  |   ↑    |
| Dev Environment      |  7/10  |  7/10  |   =    |
| Community            |  7/10  |  7/10  |   =    |
| DX Measurement       |  7/10  |  6/10  |   ↑    |
+--------------------------------------------------------------------+
| TTHW                 | 1 call | 3 calls| ↑ (estimated) |
| Competitive Rank     | Champion                                    |
| Magical Moment       | designed via help footer + dry-run pointer  |
| Product Type         | CLI tool                                    |
| Mode                 | POLISH                                      |
| Overall DX           |  8/10  |  6/10  |   ↑    |
+====================================================================+
| DX PRINCIPLE COVERAGE                                               |
| Zero Friction      | covered                                        |
| Learn by Doing     | covered (paired example)                       |
| Fight Uncertainty  | covered (every refusal carries a step)         |
| Opinionated + Escape Hatches | covered (default view; --full, --path) |
| Code in Context    | covered                                        |
| Magical Moments    | covered                                        |
| Agents Are Users   | covered                                        |
+====================================================================+
```

"Prior" is this plan as drafted before the review, not an earlier release. The Upgrade Path residual is D1, which is
Brett's to decide.

## GSTACK REVIEW REPORT

| Review         | Trigger               | Why                             | Runs | Status      | Findings                                                      |
| -------------- | --------------------- | ------------------------------- | ---- | ----------- | ------------------------------------------------------------- |
| CEO Review     | `/plan-ceo-review`    | Scope & strategy                | 0    | —           | —                                                             |
| Outside Review | codex, plan review    | Independent 2nd opinion         | 2    | disabled    | not run in either review: `codex_reviews` disabled            |
| Eng Review     | `/plan-eng-review`    | Architecture & tests (required) | 1    | ISSUES OPEN | 9 issues, 0 critical gaps; all folded, 3 owner decisions open |
| Design Review  | `/plan-design-review` | UI/UX gaps                      | 0    | —           | —                                                             |
| DX Review      | `/plan-devex-review`  | Developer experience gaps       | 1    | ISSUES OPEN | score: 6/10 → 8/10, TTHW: 3 calls → 1 call                    |

- **OUTSIDE COVERAGE:** codex, plan-review phase, disabled for both the eng and DX reviews; no findings. The other
  entries in this log belong to other plans on `dev`.
- **VERDICT:** no review is CLEAR. Every finding from the eng and DX reviews is folded into the plan. What stays open is
  three owner decisions, so eng review is required again only if an answer to D1–D3 changes a unit.

**UNRESOLVED DECISIONS:**

- D1: answered, C (a minor, xr 4.5.0; old names refused through `fix-input`)
- D2: answered, schema-less (`skill install` joins the schema-less list)
- D3: answered, unchanged (`response_schema` is the only new key)
