---
title: SRP Module Boundaries - Plan
type: refactor
status: completed
date: 2026-09-14
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
---

# SRP Module Boundaries - Plan

## Goal Capsule

- **Objective:** Each of the eight largest source files in the crate carries a recorded verdict. Four hold more than one
  responsibility and get a named seam, the exact items that cross it, and the visibility each move needs. Four hold one
  responsibility and stay whole, with the reason on the record so the question does not get reopened on line count
  alone.
- **Means:** Apply the checklist in `docs/solutions/best-practices/rust-module-splitting-srp-not-loc-20260327.md` file
  by file (KTD1). Where a split is warranted, promote the file to a directory module and move concerns into siblings,
  with the parent owning every type more than one child reads (KTD2, KTD3). Re-exports in the parent keep every path a
  caller already imports (KTD4).
- **Authority:** The prior ruling wins on whether to split. Key Technical Decisions win on mechanism inside a split.
  Units override neither.
- **Execution profile:** Behavior-preserving refactor. No unit changes observable output, a public API item, a wire
  shape, or a test expectation.
- **Stop conditions:** Stop and ask if a move needs a `pub(crate)` field, if a unit cannot preserve an import path used
  outside the module tree, or if the compiler reports a cycle between siblings.
- **Tail ownership:** U4 owns the output-discipline guard update. The guard's allow-list names a file path that a
  promotion changes, so the glob and the module move land together.

---

## Product Contract

### Summary

Eight files carry between 909 and 1705 lines each. The question each one answers is whether it holds one responsibility
or several. Four of them mix concerns that have their own vocabulary, their own tests, and no shared state with each
other: HTTP transport beside auth-scheme selection beside URL rendering; three auth command families beside interactive
prompting beside schema types; git process hardening beside path inspection beside envelope rendering; a stdio owner
beside a delimited-table serializer. Each gets a directory module and named siblings.

Four hold one responsibility at any length: a file of serde declarations, a 35-arm dispatch router, a clap type
reference, and a uniform API surface. They stay whole.

### Problem Frame

The crate's refactor trigger fires on line count, which is a prompt to review rather than a verdict. The prior ruling
settled how to answer the prompt: split on mixed concerns, on types defined far from their consumer, or on `pub(crate)`
hacks working around a bad boundary, and keep whole on uniform function collections, pure declarations, and match-based
routers whose arms delegate. Without a recorded verdict per file, the same eight files get re-argued on size every time
the trigger fires, and the argument can land on either side.

The four files that do mix concerns pay for it in review surface. A diff in `src/api/request.rs` does not say whether
auth selection, URL rendering, or transport changed. A diff in `src/skill_install/mod.rs` does not say whether the git
hardening surface moved. `run_auth_command` carries `#[allow(clippy::too_many_lines)]` over 462 lines whose arms hold
flow logic, which is the suppression the ruling names as a split signal rather than the routing-match case it exempts.

### Requirements

**Verdict coverage**

- R1. Every one of the eight reviewed files carries either a named seam or an explicit keep-whole judgment with its
  reason.
- R2. The two module homes a prior decision fixed are recorded as settled and are not proposed for a move.

**Split discipline**

- R3. A split preserves every import path used outside the module tree. The only files outside a promoted directory that
  a unit's diff touches are the output-discipline guard and the architecture list, both of which name a moved path as a
  literal string.
- R4. A split preserves blame on every moved line.
- R5. A split changes no observable behavior, no public API item, no emitted envelope, and no test expectation. Test
  count before and after a unit is identical.
- R6. Tests move to the sibling that owns the code they exercise.
- R7. A visibility widening is the minimum that compiles: `pub(super)` for an item an ancestor reads, and no field
  visibility change where a descendant relationship already grants access.

### Key Decisions

- **The ruling's checklist decides, not the line count** (recorded: KTD10 of the pre-attention cleanup plan). Governs
  R1.
- **Directory promotion over a flat sibling file** (chosen over adding `src/api/request_url.rs` beside
  `src/api/request.rs`: a sibling at the same level cannot read the parent's private items, which turns every shared
  helper into a `pub(crate)` widening, the exact hack the ruling names as a split signal). Governs R7.
- **Parent owns every type or helper more than one child reads** (chosen over sibling-to-sibling imports: a directory
  that keeps growing accumulates cycles between siblings, and a child importing an ancestor is acyclic by construction).
  Governs R3.
- **One PR per unit** (chosen over one refactor PR: each promotion is a rename plus a move, and a reviewer reading four
  at once cannot tell a move from an edit).

### Success Criteria

- A reader looking for the percent-encoding rules, the auth-scheme intersection, the git hardening surface, or the RFC
  4180 quoting rules opens the file whose name says so.
- `cargo clippy --all-targets -- -D warnings` reports no `too_many_lines` suppression on `run_auth_command`.
- `git blame -C -C -C` on each promoted file and its siblings attributes the moved lines to the commits that wrote
  them rather than to the split.
- The public API surface after all four units is identical to the surface before them.

### Scope Boundaries

**In scope**

- The four files with a named seam, their sibling files, the re-exports that hold their import paths, and the tests that
  move with them.
- `scripts/lint-stdio.sh`, whose allow-list glob names `src/output.rs` literally.
- `AGENTS.md`, whose architecture list names `request.rs` and `src/output.rs` and describes the `src/cli/` subdir
  layout.

**Outside this work**

- Any behavior change, envelope key, flag, or error message.
- The envelope construction and emission boundary inside `src/output.rs`. KTD17 of the pre-attention cleanup plan
  settles it by making `emit_error_envelope` the one emitter that `print_error_envelope`, `print_confirmation_required`,
  and `write_envelope_or_text_error` call into. This plan holds that boundary and does not re-cut it.
- The homes `src/store/snapshot.rs` and `src/cli/hints.rs` hold.
- Collapsing the duplicate format dispatch between `skill_install`'s `render_structured` and
  `OutputConfig::write_structured`. U3 names the file that holds the duplicate; removing it is separate work.

### Sources

- `docs/solutions/best-practices/rust-module-splitting-srp-not-loc-20260327.md`: the decision framework and the
  checklist every verdict below applies.
- `docs/solutions/best-practices/module-directory-promotion-pattern-2026-04-22.md`: the `git mv` recipe, the
  tests-move-with-code rule, and the parent-owns-shared-types rule.
- `docs/solutions/best-practices/rust-pub-crate-fields-for-cross-module-impl-pattern-2026-04-20.md`: the cross-module
  `impl` mechanics and the case where a field widening is genuinely required.
- `src/api/response/mod.rs`: the re-export shape a promotion uses, a private `mod` beside a `pub mod` with a `pub use`
  list that holds the crate-level paths.
- `docs/plans/2026-09-09-1528-fix-pre-attention-cleanup-plan.md`: KTD10 commissions this plan, KTD16 fixes two module
  homes, KTD17 settles the emitter boundary.

---

## Planning Contract

### Key Technical Decisions

- KTD1. **The checklist is the test, applied to what the file holds.** Each verdict below names the concerns present and
  maps them to a checklist row: mixed concerns, types far from their consumer, private helpers shared across unrelated
  functions, or a `too_many_lines` suppression over genuine complexity. A file whose functions are uniform, whose
  content is declarative, or whose match arms delegate is a keep-whole verdict, and that is a result rather than a
  deferral.
- KTD2. **A child module reads its ancestor's private items, so three of the four splits need no field widening.** Rust
  resolves a private item in the module that declares it and in every descendant. `ApiClient`'s five private fields stay
  private when the struct is declared in `request/mod.rs` and the `impl` blocks live in `request/transport.rs` and
  `request/auth_header.rs`. The `pub(crate)` field pattern applies to the other shape, where the struct is declared in
  one sibling and impl'd from another; no unit here takes that shape.
- KTD3. **A child that needs a sibling's item reads it through a parent import.** `request/mod.rs` carries `use
  url::{build_url_for_target, render_template_template};`, and `request/auth_header.rs` reaches it with `use
  super::render_template_template;`. A private `use` in the parent is an item visible to descendants, so the parent
  import costs nothing and keeps the dependency graph a tree.
- KTD4. **Inherent methods carry no module path, so moving an `impl` block is invisible to callers.**
  `client.send_request(...)` resolves against the type, not the file. Only free functions, constants, and types need a
  re-export, and in every split below the parent keeps or re-exports each one, so `src/api/mod.rs`,
  `src/cli/commands/schema.rs`, `src/cli/commands/skill.rs`, `src/lib.rs`, and `tests/cli_tests.rs` are untouched.
- KTD5. **The promotion is `git mv`, then build, then move code.** Renaming the file and declaring the empty siblings
  first proves the import tree still resolves before a single line moves. `git blame -C -C -C` is the probe that
  reads the result, not `git log --follow`. Git stores no rename and recomputes one from content similarity at diff
  time, so a promotion whose parent keeps well under half the original file pairs nothing at the default threshold,
  and at any threshold low enough to pair it pairs the largest sibling instead of the parent. Squash-merge collapses
  the branch to one diff, so no commit arrangement changes that. Copy detection still attributes the moved lines.
- KTD6. **The output-discipline guard's allow-list is part of the move.** `scripts/lint-stdio.sh` excludes
  `src/output.rs` by literal glob, and `warn_stderr` holds the crate's only `eprintln!`. Promoting the file without
  widening the glob to the directory turns the guard red on code that did not change.
- KTD7. **Three of the four units land behind the pre-attention cleanup plan's Phase B.** That plan edits
  `src/api/request.rs` (its U5), `src/cli/commands/auth.rs` (its U10, U12a, U12b), and `src/output.rs` (its U5 and U10).
  A directory promotion against a file with open work in flight produces a rename conflict that a reviewer cannot read.
  `src/skill_install/mod.rs` appears in none of its file lists, so U3 is free of the constraint.

### Sequencing

1. U3 first, with no dependency on other work.
2. U1, U2, and U4 after the pre-attention cleanup plan's Phase B merges to `dev`, in any order. They share no file.

---

## Implementation Units

### U1. `src/api/request.rs` splits into a directory with URL, auth-header, and transport siblings

- **Goal:** A diff in the request layer says which of the four concerns changed.
- **Requirements:** R3, R4, R5, R6, R7. Implements KTD2, KTD3, KTD4, KTD5.
- **Dependencies:** The pre-attention cleanup plan's U5, which edits this file (KTD7).
- **Concerns present:** Four, none sharing state with another. URL rendering and percent-encoding (one `AsciiSet`
  constant and four free functions with their own 170-line test cluster). Auth-scheme selection and the
  mismatch-envelope construction that follows a failed intersection (four methods, 241 lines, no HTTP). Transport for
  three response shapes (three methods, 350 lines, plus header inspection and verbose logging). Type declarations for
  the client and its four option records (190 lines). Checklist row: mixed concerns, business logic beside transport
  I/O.
- **Seam:** Directory promotion. `git mv src/api/request.rs src/api/request/mod.rs`, then `url.rs`, `auth_header.rs`,
  and `transport.rs` beside it.
- **Files:** `src/api/request.rs` (renamed to `src/api/request/mod.rs`), `src/api/request/url.rs` (create),
  `src/api/request/auth_header.rs` (create), `src/api/request/transport.rs` (create), `AGENTS.md` (the architecture list
  names `request.rs`).
- **What moves:**
  - To `url.rs`: `URL_VALUE_ENCODE_SET`, `build_url_for_target`, `render_template_template`, `validate_raw_url_scheme`,
    `write_encoded`, and the `tmpl` fixture with the thirteen `build_url_*` tests.
  - To `auth_header.rs`: a second `impl ApiClient` holding `get_auth_header`, `get_auth_header_public`,
    `available_auth_in_app`, and `other_apps_with_credentials`.
  - To `transport.rs`: a third `impl ApiClient` holding `send_request`, `send_multipart_request`, and `stream_request`;
    the free functions `user_supplied_header`, `log_header_overrides`, and `log_response_headers`; the eight
    `user_supplied_header_*` tests.
  - Stays in `mod.rs`: `RequestTarget` and its `Default`, `RequestOptions`, `CallOptions` with its `Default` and
    `to_request_options`, `MultipartOptions`, `DEFAULT_TIMEOUT_SECS`, the `ApiClient` struct, its constructors and
    accessors (`new`, `with_timeout`, `timeout_secs`, `set_output`, `from_env`, `auth_app_name`, `build_url`,
    `build_url_public`), and the two `call_options_*` tests.
- **Visibility changes:** `get_auth_header` goes from private to `pub(super)` because `transport.rs` calls it across a
  sibling boundary through the type. `build_url_for_target` and `render_template_template` are `pub(super)` in `url.rs`,
  imported once by `mod.rs` per KTD3. `build_url` stays private: it is declared in `mod.rs`, and both children are
  descendants. No field changes: `base_url`, `client`, `auth`, `timeout_secs`, and `out` stay private per KTD2.
- **Re-export contract:** `src/api/mod.rs` keeps `pub use request::{ApiClient, CallOptions, DEFAULT_TIMEOUT_SECS,
  MultipartOptions, RequestOptions, RequestTarget};` verbatim, because every named item stays declared in
  `request/mod.rs`. Every method call site is unaffected per KTD4.
- **Test scenarios:**
  - Test expectation: the existing suite, unchanged in count and in assertion text, with the two moved clusters passing
    from their new files.
- **Verification:** `cargo test` reports the same test count; `git diff --stat` lists only `src/api/request/**` and
  `AGENTS.md`; `git blame -C -C -C src/api/request/mod.rs` attributes its moved lines to the commits that wrote them.

### U2. `src/cli/commands/auth.rs` splits into a directory with sign-in, session, apps, and types siblings

- **Goal:** `run_auth_command` is a router whose arms delegate, and each auth command family has a file.
- **Requirements:** R3, R4, R5, R6, R7. Implements KTD2, KTD3, KTD4, KTD5.
- **Dependencies:** The pre-attention cleanup plan's U10, U12a, and U12b, all of which edit this file (KTD7).
- **Concerns present:** Three command families in one file (the auth verbs, the app registry under `run_app_command`,
  the redirect-URI pair under `run_redirect_uri_command`). Interactive stdin prompting (`prompt_select`) beside
  credential business logic. Three `Serialize` plus `JsonSchema` response shapes whose second consumer is
  `src/cli/commands/schema.rs`. `#[allow(clippy::too_many_lines, clippy::too_many_arguments)]` over a 462-line
  `run_auth_command` whose arms carry flow logic rather than delegation: the OAuth2 arm alone runs headless auto-engage
  detection, a two-step remote flow, a stdin read of the redirect URL, and a choice between two JSON envelope shapes.
  Checklist row: mixed concerns, plus a `too_many_lines` suppression over genuine complexity rather than a routing
  match.
- **Seam:** Directory promotion. `git mv src/cli/commands/auth.rs src/cli/commands/auth/mod.rs`, then `signin.rs`,
  `session.rs`, `apps.rs`, and `types.rs` beside it.
- **Files:** `src/cli/commands/auth.rs` (renamed to `src/cli/commands/auth/mod.rs`), `src/cli/commands/auth/signin.rs`
  (create), `src/cli/commands/auth/session.rs` (create), `src/cli/commands/auth/apps.rs` (create),
  `src/cli/commands/auth/types.rs` (create), `AGENTS.md` (the `src/cli/` entry describes the subdir layout).
- **What moves:**
  - To `types.rs`: `AppStatusEntry`, `RedirectUriGetResponse`, `RedirectUriSetResponse`, and the `is_false` serde
    predicate.
  - To `signin.rs`: the `AuthCommands::Oauth2`, `Oauth1`, and `App` arm bodies, as `pub(super) fn oauth2`, `oauth1`, and
    `bearer`.
  - To `session.rs`: the `AuthCommands::Status`, `Clear`, and `Default` arm bodies as `pub(super) fn status`, `clear`,
    and `set_default`, and `prompt_select`, whose only caller is the default-selection flow.
  - To `apps.rs`: `AppGlobalFlags`, `run_app_command`, and `run_redirect_uri_command`, which `run_app_command` is the
    sole caller of.
  - Stays in `mod.rs`: `AuthGlobalFlags`, the `run_auth_command` router, and the three helpers more than one child
    reads: `truncate`, `build_app_status_entries` (read by the status flow and by the app-list arm), and
    `credential_less_default_warning` (read by the OAuth2 sign-in arm).
- **Visibility changes:** `run_app_command` and the six extracted arm functions become `pub(super)`. The three
  parent-held helpers stay private and descendants read them per KTD2. No type visibility changes: the three response
  shapes are already `pub(crate)`.
- **Re-export contract:** `auth/mod.rs` carries `pub(crate) use types::{AppStatusEntry, RedirectUriGetResponse,
  RedirectUriSetResponse};`, which keeps `use crate::cli::commands::auth::{AppStatusEntry, RedirectUriGetResponse,
  RedirectUriSetResponse};` in `src/cli/commands/schema.rs` byte-identical. `src/cli/commands/mod.rs` keeps
  `auth::run_auth_command` and `auth::AuthGlobalFlags`, both still declared in `auth/mod.rs`.
- **Interaction with the fixed homes:** KTD16 of the pre-attention cleanup plan puts the credentialed-apps query that
  `credential_less_default_warning` performs into `src/store/snapshot.rs`, leaving the CLI wording behind. This unit
  moves the wording with its parent and leaves the query where that decision put it.
- **Test scenarios:**
  - Test expectation: `tests/cli_tests.rs` secret-exclusion coverage over the rendered `AppStatusEntry` JSON passes
    unchanged, proving the type's derives and field set survived the move.
- **Verification:** `cargo clippy --all-targets -- -D warnings` passes with the `too_many_lines` allow removed from
  `run_auth_command`; `git diff --stat` lists only `src/cli/commands/auth/**` and `AGENTS.md`.

### U3. `src/skill_install/mod.rs` gains git, destination, render, and update siblings

- **Goal:** The git hardening surface, the destination rules, and the envelope rendering each have a file, and the
  install and update verbs stop sharing one.
- **Requirements:** R3, R5, R6, R7. Implements KTD3, KTD4.
- **Dependencies:** None. This file appears in no in-flight unit (KTD7).
- **Concerns present:** Four. Git process hardening and spawn (three constant tables, two command builders, one spawn
  reducer). `$HOME` expansion and destination inspection (one status enum, three path functions). Envelope rendering,
  which re-implements the format dispatch `OutputConfig` owns (five private render functions and a writer). Install and
  update orchestration for one host and for all hosts (seven functions across two verbs that duplicate each other's
  shape). Checklist row: mixed concerns, process execution beside filesystem inspection beside output rendering.
- **Seam:** Sibling files inside the directory that already exists. No promotion step: `src/skill_install/` is already a
  directory holding `mod.rs` and `skill.json`.
- **Files:** `src/skill_install/mod.rs`, `src/skill_install/git.rs` (create), `src/skill_install/destination.rs`
  (create), `src/skill_install/render.rs` (create), `src/skill_install/update.rs` (create).
- **What moves:**
  - To `git.rs`: `GIT_HARDEN_FLAGS`, `GIT_HARDEN_ENV_REMOVE`, `GIT_HARDEN_ENV_SET`, `build_clone_command`,
    `format_clone_command`, `spawn_git_clone`, and the three hardening tests
    (`build_clone_command_applies_hardening_surface`, `git_harden_env_set_disables_user_config`,
    `format_clone_command_matches_canonical_shape`).
  - To `destination.rs`: `DestinationStatus` with `as_envelope_str`, `expand_tilde`, `expand_tilde_with`,
    `check_destination`, and the seven tilde and destination tests.
  - To `render.rs`: `render_envelope`, `render_multi`, `render_structured`, and `emit_envelope`.
  - To `update.rs`: `run_update`, `run_update_multi`, `render_update_dry_run`, and `render_update_error`.
  - Stays in `mod.rs`: the pipeline doc comment, the `generated_hosts` include and its re-export, `InstallError` with
    `reason` (both `git.rs` and `destination.rs` return it, so the parent owns it), `InstallEnvelope`,
    `InstallMultiEnvelope`, the five action and status constants, `compute_install_envelope`, `run_install`,
    `run_install_multi`, `run_for_all_hosts`, and `emit_missing_host_envelope` (the install and update paths both call
    it).
- **Visibility changes:** `spawn_git_clone` and the four render functions go from private to `pub(super)`, because
  `mod.rs` is an ancestor rather than a descendant and cannot read a child's private item. Every other moved item is
  already `pub`.
- **Re-export contract:** `mod.rs` adds `pub use git::{GIT_HARDEN_ENV_REMOVE, GIT_HARDEN_ENV_SET, GIT_HARDEN_FLAGS,
  build_clone_command, format_clone_command};`, `pub use destination::{DestinationStatus, check_destination,
  expand_tilde, expand_tilde_with};`, and `pub use update::{run_update, run_update_multi};`. That keeps
  `xurl::skill_install::KNOWN_HOSTS` in `tests/cli_tests.rs`, `crate::skill_install::SkillHost` in `src/cli/mod.rs`,
  `crate::skill_install::{InstallEnvelope, InstallMultiEnvelope}` in `src/cli/commands/schema.rs`, and the two
  `run_*_multi` calls in `src/cli/commands/skill.rs` unchanged.
- **Recorded, not fixed here:** `render_structured` dispatches on `OutputFormat` over the same seven variants that
  `OutputConfig::write_structured` dispatches on. Naming `render.rs` makes the duplication visible in one place;
  collapsing it is out of scope per the Scope Boundaries.
- **Test scenarios:**
  - Test expectation: the ten moved unit tests pass from their new files, and `tests/cli_tests.rs` host-enumeration
    coverage passes unchanged, proving the generated host map re-export survived.
- **Verification:** `cargo test` reports the same test count; `git diff --stat` lists only `src/skill_install/**`.

### U4. `src/output.rs` splits the delimited-table serializer into a sibling

- **Goal:** The RFC 4180 quoting rules live in a file that says so, and the stdio owner holds only stdio decisions.
- **Requirements:** R3, R4, R5, R6, R7. Implements KTD5, KTD6.
- **Dependencies:** The pre-attention cleanup plan's U5 and U10, both of which edit this file (KTD7).
- **Concerns present:** Two. The module's declared responsibility is ownership of every `println!` and `eprintln!` in
  the crate: a config carrier, a format enum, and a print family that decides whether a message is emitted, to which
  writer, and in which format. Separate from that, `write_flattened`, `scalar_cell`, and `delimited_escape` answer how a
  JSON value becomes header and value rows with escaped cells, hold no reference to `OutputConfig`, and carry quoting
  rules of their own (double-quote wrapping for CSV, control-character replacement for TSV). Checklist row: mixed
  concerns, a serializer inside an emitter. This is the narrowest of the four seams, roughly 110 lines, and it is
  sequenced last.
- **Seam:** Directory promotion. `git mv src/output.rs src/output/mod.rs`, then `delimited.rs` beside it.
- **Files:** `src/output.rs` (renamed to `src/output/mod.rs`), `src/output/delimited.rs` (create),
  `scripts/lint-stdio.sh`, `AGENTS.md` (two references to `src/output.rs`).
- **What moves:** `write_flattened`, `scalar_cell`, and `delimited_escape`, with their doc comments.
- **What stays:** `OutputFormat` with `is_structured`, `OutputConfig` with every constructor and print method,
  `write_structured`, `write_envelope_or_text_error`, the `Default` impl, `warn_stderr`, `strip_ansi`, and the existing
  test module.
- **Visibility changes:** `write_flattened` goes from private to `pub(super)` because the two `print_response` arms call
  it from the parent. `scalar_cell` and `delimited_escape` stay private inside `delimited.rs`.
- **Re-export contract:** Nothing to re-export. All three moved functions are private, and every public item stays
  declared in `mod.rs`, so `crate::output::{OutputConfig, OutputFormat, warn_stderr}` and the `xurl::output` surface in
  `src/lib.rs` are unchanged.
- **Guard update:** `scripts/lint-stdio.sh` excludes `--glob '!src/output.rs'`. The glob widens to the directory, and
  the two message strings that name the file follow. `warn_stderr` holds the only `eprintln!` in the crate, so a stale
  glob turns the guard red on unmoved code (KTD6).
- **Boundary held:** `print_error`, `print_error_envelope`, `print_success`, `print_dry_run`,
  `print_confirmation_required`, and `write_envelope_or_text_error` stay where KTD17 of the pre-attention cleanup plan
  puts them, behind one `emit_error_envelope`. This unit does not touch that group.
- **Test scenarios:**
  - Test expectation: `bash scripts/lint-stdio.sh` passes after the glob update and fails before it, which is the proof
    the guard change is load-bearing rather than cosmetic.
- **Verification:** `bash scripts/lint-stdio.sh` reports clean; `cargo test` reports the same test count; `git diff
  --stat` lists only `src/output/**`, `scripts/lint-stdio.sh`, and `AGENTS.md`.

---

## Files Kept Whole

Each verdict below is a result, not a deferral. The file was read for its module doc, its type declarations, its `impl`
blocks, and where its concerns actually sit.

### `src/api/response/types.rs` (1053 lines)

Keep whole. 447 lines of `Serialize` plus `Deserialize` plus `JsonSchema` struct declarations, one generic
`deserialize_response`, and 606 lines of round-trip tests. This is the ruling's pure-type-definition case verbatim.
Splitting by endpoint family would put `ApiResponse<T>`, `Includes`, `ResponseMeta`, and `ApiError` in one file and
their payload types in another, so a reader chasing one response shape opens two files instead of one.

### `src/cli/commands/mod.rs` (925 lines)

Keep whole. One responsibility, dispatch. `run` resolves the global flags and hands off; `run_subcommand` is a 35-arm
match whose arms follow one shape (build a dry-run context, validate, construct the client, call a shortcut, print the
typed response); `run_raw_mode` is the second dispatch path from the same entrypoint; the remaining functions are
dispatch support. The `#[allow(clippy::too_many_lines)]` here sits on exactly the routing match the ruling exempts, and
the file's size tracks the command surface rather than a second concern. The destructive-confirmation gate (`Gate`,
`gate_destructive`, `confirm_destructive`, `is_interactive`) is parent-owned state the `auth` child imports through
`super`, which is the direction the promotion pattern prescribes, so it is a correct boundary rather than a leak.

### `src/cli/mod.rs` (1705 lines)

Keep whole, exempt. Recorded as exempt by KTD10 of the pre-attention cleanup plan and matching the ruling's own
`cli/mod.rs` precedent. 52 help constants and the clap derive types (`Cli`, `Commands`, `UsageCommands`, `SkillCmd`,
`CommonFlags`, `AuthCommands`, `AppCommands`, `RedirectUriCommands`, `MediaCommands`, `ColorChoice`). This is a type
reference; splitting it means opening several files to read one CLI surface.

### `src/api/shortcuts.rs` (1006 lines)

Keep whole, exempt. Recorded as exempt by KTD10 and matching the ruling's own `shortcuts.rs` precedent at a smaller
size. One `impl ApiClient` block with 30 methods of uniform shape: build a `RequestTarget`, attach the query from
`CallOptions`, call `send_request`, deserialize into a typed response. The six `validate_*` guards, the two `resolve_*`
normalizers, and the three private body structs are the only non-uniform items; each is a short pure function co-located
with the endpoint it guards, with one consumer group in `commands::run_subcommand`, so they stay where the calls are.

---

## Homes Already Fixed

KTD16 of the pre-attention cleanup plan settles two module homes. This plan records them and proposes no move.

- `src/store/snapshot.rs` holds `StoreSnapshot`, `LoadState`, and the credentialed-apps query. Library code, no CLI
  strings.
- `src/cli/hints.rs` holds the `Hint` type, the five-state chooser, the `NextStep` builder with its quote helper, and
  the enrollment matcher. CLI code.

---

## Verification Contract

| Gate               | Command                                                                               | Applies to     | Done signal                                                             |
| ------------------ | ------------------------------------------------------------------------------------- | -------------- | ----------------------------------------------------------------------- |
| Format             | `cargo fmt -- --check`                                                                | U1, U2, U3, U4 | No diff                                                                 |
| Lint               | `cargo clippy --all-targets -- -D warnings`                                           | U1, U2, U3, U4 | Clean; for U2, with no `too_many_lines` allow on `run_auth_command`     |
| Tests              | `cargo test`                                                                          | U1, U2, U3, U4 | Same test count as the unit's base commit, all passing                  |
| Output discipline  | `bash scripts/lint-stdio.sh`                                                          | U4             | Clean after the glob update, red against the unchanged glob             |
| Schema freshness   | `cargo test --test schema_tests`                                                      | U2, U3, U4     | Drift test passes with no schema regeneration                           |
| Blast radius       | `git diff --stat` against the unit's base commit                                      | U1, U2, U3, U4 | Only the module tree, plus `scripts/lint-stdio.sh` (U4) and `AGENTS.md` |
| Blame continuity   | `git blame -C -C -C` on each promoted file and its siblings                           | U1, U2, U4     | Moved lines attribute to the commits that wrote them, not to the split  |
| Visibility minimum | `rg 'pub(\(crate\))? [a-z_]+:' <moved files>`                                         | U1, U2, U3, U4 | No field widening in any moved struct                                   |
| Markdown           | `markdownlint-cli2 docs/plans/2026-09-14-1200-refactor-srp-module-boundaries-plan.md` | This plan      | Zero issues                                                             |

---

## Definition of Done

- All eight reviewed files carry a recorded verdict in this document: four seams with their moves, four keep-whole
  judgments with their reasons.
- The two homes KTD16 fixed are recorded here and untouched by every unit.
- Each unit lands as its own PR, in the sequenced order, with every gate in the Verification Contract green.
- No unit's diff changes an emitted envelope, a flag, an error message, a public API item, or a test assertion.
- Every promotion used `git mv`, and `git blame -C -C -C` on the promoted file and its siblings attributes the moved
  lines to the commits that wrote them.
- The stdio guard's allow-list and the `AGENTS.md` architecture list name the paths that exist after the moves.
