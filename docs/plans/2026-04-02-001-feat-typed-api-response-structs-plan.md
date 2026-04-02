---
title: "feat: Add typed API response structs for library consumers"
type: feat
status: active
date: 2026-04-02
origin: docs/brainstorms/2026-03-21-typed-responses-requirements.md
deepened: 2026-04-02
---

# feat: Add typed API response structs for library consumers

## Overview

Replace raw `serde_json::Value` returns across all 29 API shortcut functions with typed response structs. This gives
downstream library consumers (primarily `bird`) compile-time safety, autocomplete, and documentation instead of fragile
runtime JSON indexing. The CLI binary produces semantically equivalent JSON output — typed structs serialize back to
JSON via serde (field ordering may differ from pre-migration output).

## Problem Frame

xurl-rs uses `serde_json::Value` as a universal return type for all API responses. This works for a CLI that pipes JSON
to the terminal, but becomes a liability when consumed as a library crate. `bird` currently shells out to the `xr` CLI
and parses JSON output — replacing that with a crate dependency requires typed responses. Today, only 6 places in
production code access fields on `Value` (4 in `media.rs`, 2 in `cli/commands/mod.rs`). All 29 shortcuts and the output
layer treat the Value as an opaque pass-through, making the migration surface well-bounded. (see origin:
`docs/brainstorms/2026-03-21-typed-responses-requirements.md`)

## Requirements Trace

- R1. Shortcut functions return typed response structs instead of `serde_json::Value`
- R2. Shared response types cover the X API v2 surface: `Tweet`, `User`, `DmEvent`, action confirmations, media uploads
- R3. Generic `ApiResponse<T>` wrapper: `data: T`, optional `includes`, optional `meta`
- R4. All types derive `Serialize` + `Deserialize` for CLI JSON output with no behavior change
- R5. ~~Types optionally generated from X API v2 OpenAPI spec via `cargo-typify`, then hand-curated~~ Demoted to
  implementation note — optional tooling, not a requirement. Plan does not depend on it.
- R6. Library crate exposes types as public API: `use xurl::api::response::{Tweet, User, ApiResponse}`
- R7. Conditionally present fields (due to `fields`/`expansions` params) are `Option<T>`
- R8. Forward-compatible deserialization — unknown fields preserved via `#[serde(flatten)] pub extra: BTreeMap<String,
  Value>` on every response struct. Unknown fields round-trip through serialize/deserialize, CLI output preserves all
  API fields, and library consumers access new fields immediately via `extra["field_name"]` before types are updated.
- R9. All types derive `Default` for test factory functions with struct update syntax
- R10. Spec-as-test validation — a CI test validates hand-written types can deserialize the X API v2 OpenAPI spec's
  example responses. `scripts/update-types.sh` fetches the latest spec, runs cargo-typify, and diffs against
  hand-written types to show what changed.

## Scope Boundaries

- **Library API: breaking change** — Shortcut function return types change from `Result<serde_json::Value>` to
  `Result<ApiResponse<T>>`. No parallel `_typed()` variants. Downstream consumers (`bird`) must update to struct field
  access. Requires semver major bump (2.0.0).
- Auth types (`OAuth1Token`, `OAuth2Token`) already typed — out of scope
- Request body types (`PostBody`, `PostReply`) already typed — out of scope
- Streaming endpoint responses — out of scope (line-by-line JSON)
- OpenAPI spec used as generation starting point, not runtime dependency
- Output layer formatting/syntax-highlighting behavior unchanged
- `send_request()` continues returning `Value` — deserialization moves to shortcut boundary
- **Raw mode unchanged** — `xr raw` subcommand and direct `send_request()` calls remain on the `Value` path. Library
  consumers who call `send_request()` directly still get `Value`. The typed API is via shortcut functions only.
- **Media upload** (`src/api/media.rs`): calls `send_request()` directly (not through shortcuts) and prints intermediate
  responses. In scope for typed response structs but follows a distinct deserialization pattern — `from_value()` at each
  call site plus `to_value()` before each `print_response()` call. This is a second deserialization pattern alongside
  the shortcut boundary pattern.
- **CLI output equivalence**: JSON output is *semantically equivalent* (same data, same structure) but field ordering
  may differ from pre-migration output because serde serializes struct fields in declaration order, not original API
  response order. The conformance runner's `stdout_json` mode uses structural JSON comparison (order-independent).

## Context & Research

### Relevant Code and Patterns

- `src/api/request.rs:90` — `ApiClient::send_request()` returns `Result<serde_json::Value>`, single deserialization
  point
- `src/api/shortcuts.rs` (672 lines) — 29 shortcut functions, all return `Result<serde_json::Value>`, pure pass-through
- `src/api/response.rs` (100 lines) — `format_and_print_response()` and three private colorization helpers
- `src/api/media.rs` — 4 Value field accesses for media ID, processing state, check_after_secs, progress_percent
- `src/cli/commands/mod.rs` — 2 Value field accesses in `resolve_my_user_id()` and `resolve_user_id()`
- `src/output.rs` — `OutputConfig::print_response(&serde_json::Value)` formats for terminal
- `src/store/types.rs` — established serde patterns: `#[serde(rename)]`, `#[serde(default)]`,
  `#[serde(skip_serializing_if)]`
- `src/api/shortcuts.rs` — existing request body types (`PostBody`, `PostReply`, `PostMedia`) as pattern reference
- `src/error.rs` — `XurlError` with `From<serde_json::Error>` impl already handles deserialization errors

### Institutional Learnings

- **Parse JSON once at transport layer** (from bird code review): deserialization from Value to typed struct should
  happen at one canonical place — the shortcut function boundary
- **Return type honesty** (from xurl-rs port methodology): if a field is always present, don't make it `Option<T>`.
  Model the actual API behavior, not defensive worst-case
- **Keep shortcuts.rs cohesive** (from SRP-not-LOC learning): 29 uniform functions should stay in one file even as
  signatures change. Place new types in a separate module
- **Forward compatibility** (from Expensify integration learning): never use `#[serde(deny_unknown_fields)]` on response
  structs. APIs add fields regularly. Use `#[serde(default)]` for optional fields
- **Avoid cloning Value before deserializing** (from transport layer review): use `serde_json::from_value()` which takes
  ownership

## Key Technical Decisions

- **One generic `ApiResponse<T>`** (resolves origin Q1): Single items use `ApiResponse<Tweet>`, lists use
  `ApiResponse<Vec<Tweet>>`. Serde handles both shapes transparently. No need for separate `ApiListResponse<T>` — the
  caller always knows which shape based on the endpoint. (see origin: deferred Q1)
- **Keep `send_request()` returning `Value`** (resolves origin Q4): Each shortcut deserializes via
  `serde_json::from_value::<ApiResponse<T>>(value)?`. Minimizes blast radius, keeps media.rs and streaming unchanged.
  (see origin: deferred Q4)
- **Separate action confirmation structs** (resolves origin Q3): 7 types — each with a single bool field matching the
  API's response field name. Distribution: `LikedResult` (like_post, unlike_post), `FollowingResult` (follow_user,
  unfollow_user), `DeletedResult` (delete_post), `RetweetedResult` (repost, unrepost), `BookmarkedResult` (bookmark,
  unbookmark), `BlockingResult` (block_user, unblock_user), `MutingResult` (mute_user, unmute_user). 7 types for 13
  shortcuts. Separate types chosen over a generic because each API field name is distinct (`liked`, `following`,
  `deleted`, etc.) and downstream consumers get field-specific compile-time safety. (see origin: deferred Q3)
- **`serde_json::to_value()` at CLI dispatch** (resolves origin Q5): Output layer (`print_response`,
  `format_and_print_response`) remains unchanged. CLI handlers convert typed responses back to Value before printing.
  (see origin: deferred Q5)
- **cargo-typify spike is optional** (resolves origin Q2): ~10-15 structs is small enough for manual writing. If the
  spike produces usable output, use as starting point. Plan does not depend on it. (see origin: deferred Q2)
- **`#[serde(flatten)]` on every response struct** (resolves unknown field preservation): Every response struct
  (`ApiResponse<T>`, `Tweet`, `User`, `DmEvent`, action types, media types, `UsageData`, `Includes`, `ResponseMeta`)
  includes `#[serde(flatten)] pub extra: BTreeMap<String, serde_json::Value>`. Unknown API fields are captured during
  deserialization and re-emitted during serialization in deterministic alphabetical order. CLI output preserves ALL API
  fields including ones not yet in the struct. Library consumers access new fields via `extra["field_name"]` before
  promoting to named fields. `BTreeMap` chosen over `HashMap` for deterministic output ordering. Eliminates the need for
  a raw/untyped output mode. Chosen over: raw Value field on ApiResponse (doubles memory), accept data loss (CLI
  regression), or separate raw codepath (dual maintenance).
- **Spec-as-test + update script for type maintenance** (resolves field update workflow): Hand-write ergonomic types.
  `scripts/update-types.sh` fetches the latest X API v2 OpenAPI spec, runs cargo-typify, and diffs against hand-written
  types to show what changed. A CI test in `tests/spec_validation.rs` validates types can deserialize the spec's example
  responses. When the spec adds fields, CI fails with a clear message. Developer runs the script, reviews the diff,
  promotes fields from `extra` to named struct fields. Chosen over: build.rs codegen (unergonomic generated names, slow
  builds, painful debugging) or pure manual (no drift detection).
- **Convert response module to directory**: `src/api/response.rs` → `src/api/response/` directory with `types.rs` and
  `format.rs`. Follows the `src/store/` split pattern. Keeps types separate from formatting per SRP.
- **Replace, not dual-maintain** (carried from origin): Shortcut functions replace `Value` returns with typed returns.
  No parallel `_typed()` variants. This is an intentional breaking change to the library API.

## Open Questions

### Resolved During Planning

- **ApiResponse\<T\> single vs list** (origin Q1): One generic wrapper. `data: T` where T is either a single struct or
  `Vec<Struct>`. Each shortcut function specifies the concrete T at compile time, and serde deserializes into that
  specific type.
- **Action confirmation types** (origin Q3): 7 separate small structs, one per distinct API field name. Shared across 13
  action shortcuts.
- **send_request() generic or not** (origin Q4): Keep returning Value. Deserialization at shortcut boundary via
  `from_value()`.
- **Output layer approach** (origin Q5): `to_value()` conversion at CLI dispatch. Output layer unchanged.
- **cargo-typify viability** (origin Q2): Optional spike, manual writing as viable fallback.
- **Empty body from `send_request()`**: `send_request()` returns `json!({})` for empty or non-JSON 2xx response bodies
  (request.rs lines 157-165). `from_value::<ApiResponse<T>>(json!({}))` would fail because there is no `data` field.
  Resolution: the X API v2 consistently returns JSON bodies for all 29 endpoints used by xurl-rs — the empty-body
  fallback is a defensive path unlikely to trigger. Keep `data: T` (not `Option<T>`) for clean typed API. Each shortcut
  should guard against the empty Value before `from_value()` — if Value is an empty object `{}`, return a descriptive
  `XurlError` rather than a cryptic serde deserialization error.
- **R7 vs return type honesty** (tension between requirement and institutional learning): R7 says conditionally present
  fields are `Option<T>`. The "return type honesty" learning says don't use `Option<T>` for always-present fields.
  Resolution: these are compatible. Fields always present in the API response (e.g., `Tweet.id`, `Tweet.text`) are
  non-Optional. Fields that depend on which `tweet.fields`/`user.fields` the caller requests (e.g., `created_at`,
  `public_metrics`) are `Option<T>` because the API omits them when not requested. "Conditionally present" means omitted
  by the server based on request params, not "might be null."
- **CLI output field ordering**: `serde_json::to_value()` round-trip serializes struct fields in declaration order, not
  original API response key order. CLI output is semantically equivalent JSON but not byte-for-byte identical.
  Conformance runner's `stdout_json` mode uses structural comparison (order-independent). Wiremock tests are the primary
  safety net; conformance runner covers only 3 of 37 test cases with `stdout_json`.
- **`Default` and required fields**: `Default` derive on types with non-Optional fields (e.g., `Tweet.id: String`)
  produces semantically invalid instances (`id: ""`). This is intentional per origin R9 — the trade-off is accepted for
  test ergonomics. Tests using `..Default::default()` must always override required fields like `id` and `text`.
- **`ApiResponse.errors` field**: The X API v2 can return partial errors alongside valid `data` in 200 responses. The
  `errors: Option<Vec<ApiError>>` field captures these for consumers that want to inspect warnings. No behavioral change
  in xurl-rs — the field is deserialized and re-serialized as pass-through.

### Deferred to Implementation

- **Exact field coverage per struct**: Which optional Tweet/User fields to include depends on what xurl-rs shortcut
  functions actually request via `tweet.fields`/`user.fields` params. Determine by reading each shortcut's query
  parameters during implementation.
- **Includes struct initial shape**: Define concrete `Includes` struct in Unit 1 with fields for the expansions xurl-rs
  currently uses: `users: Option<Vec<User>>`, `tweets: Option<Vec<Tweet>>`. Additional fields (media, polls, places)
  deferred. These fields are currently pass-through only — no shortcut or CLI code accesses `resp.includes` — but `bird`
  will need them when consuming xurl as a library.
- **Media upload response shape**: The multi-step upload flow (init → append → finalize → poll status) may have
  variations per step. Define types from actual API responses during implementation.
- **UsageResponse structure**: The usage endpoint wraps data in the standard `{"data": {...}}` envelope (confirmed via
  test fixtures), so `ApiResponse<UsageData>` works. The inner `UsageData` struct has a unique shape (project_cap,
  daily_project_usage, daily_client_app_usage). Define exact fields from test fixtures during implementation.

## High-Level Technical Design

> *This illustrates the intended approach and is directional guidance for review, not implementation specification. The
> implementing agent should treat it as context, not code to reproduce.*

### Data Flow (Before → After)

```text
BEFORE:
  HTTP response → send_request() → Value → shortcut (pass-through) → Value → CLI dispatch → print_response(Value)

AFTER:
  HTTP response → send_request() → Value → shortcut (from_value::<ApiResponse<T>>) → ApiResponse<T>
    ├── Library consumer: accesses typed fields directly (resp.data.id, resp.data.text)
    └── CLI binary: to_value() → print_response(Value)  [output unchanged]
```

### Type Hierarchy

```text
ApiResponse<T>
├── data: T
├── includes: Option<Includes>
├── meta: Option<ResponseMeta>
└── errors: Option<Vec<ApiError>>

T is one of:
├── Tweet                          (create_post, read_post, reply_to_post, quote_post)
├── Vec<Tweet>                     (search_posts, get_timeline, get_mentions, get_bookmarks, get_liked_posts)
├── User                           (get_me, lookup_user)
├── Vec<User>                      (get_following, get_followers)
├── DmEvent                        (send_dm)
├── Vec<DmEvent>                   (get_dm_events)
├── LikedResult                    (like_post, unlike_post)
├── FollowingResult                (follow_user, unfollow_user)
├── DeletedResult                  (delete_post)
├── RetweetedResult                (repost, unrepost)
├── BookmarkedResult               (bookmark, unbookmark)
├── BlockingResult                 (block_user, unblock_user)
├── MutingResult                   (mute_user, unmute_user)
├── MediaUploadResponse            (media upload init/finalize)
└── UsageData                      (get_usage)
```

### Shortcut Response Categories

| Category | Shortcuts | Return Type | Count |
|---|---|---|---|
| Tweet (single) | create_post, reply_to_post, quote_post, read_post | `ApiResponse<Tweet>` | 4 |
| Tweet (list) | search_posts, get_timeline, get_mentions, get_bookmarks, get_liked_posts | `ApiResponse<Vec<Tweet>>` | 5 |
| User (single) | get_me, lookup_user | `ApiResponse<User>` | 2 |
| User (list) | get_following, get_followers | `ApiResponse<Vec<User>>` | 2 |
| Action confirm | like/unlike, follow/unfollow, delete, repost/unrepost, bookmark/unbookmark, block/unblock, mute/unmute | `ApiResponse<ActionType>` | 13 |
| DM | send_dm, get_dm_events | `ApiResponse<DmEvent>` / `ApiResponse<Vec<DmEvent>>` | 2 |
| Usage | get_usage | `ApiResponse<UsageData>` | 1 |

## Implementation Units

- [ ] **Unit 1: Create response types module with typed structs**

  **Goal:** Define all typed response structs and restructure the response module for SRP compliance.

  **Requirements:** R2, R3, R4, R6, R7, R8, R9

  **Dependencies:** None

  **Files:**
- Create: `src/api/response/mod.rs`
- Create: `src/api/response/types.rs`
- Create: `src/api/response/format.rs`
- Modify: `src/api/mod.rs` (response module declaration unchanged — Rust handles file→dir transparently)
- Delete: `src/api/response.rs` (replaced by directory module)
- Test: `src/api/response/types.rs` (inline `#[cfg(test)]` module)

  **Approach:**
- Convert `src/api/response.rs` to directory module following the `src/store/` split pattern
- Move `format_and_print_response()` to `format.rs`
- Define all types in `types.rs`: `ApiResponse<T>`, `Includes`, `ResponseMeta`, `ApiError`, `Tweet`,
  `TweetPublicMetrics`, `User`, `UserPublicMetrics`, `DmEvent`, 7 action result types, `MediaUploadResponse`,
  `MediaProcessingInfo`, `UsageData`
- Re-export types from `mod.rs` so `use xurl::api::response::{Tweet, User, ApiResponse}` works. Consider also
  re-exporting the most common types from `src/api/mod.rs` for ergonomics — library consumers currently import via `use
  xurl::api::{create_post, ApiClient}` and adding response types to the same path avoids double imports
- All types derive `Debug, Clone, Serialize, Deserialize, Default`
- Every response struct includes `#[serde(flatten)] pub extra: BTreeMap<String, serde_json::Value>` for forward
  compatibility (R8). Unknown fields captured during deserialization and re-emitted during serialization. This applies
  to `ApiResponse<T>`, `Tweet`, `User`, `DmEvent`, all action result types, `MediaUploadResponse`,
  `MediaProcessingInfo`, `UsageData`, `Includes`, `ResponseMeta`, `TweetPublicMetrics`, `UserPublicMetrics`
- Use `#[serde(default)]` on struct-level for optional field groups
- Use `Option<T>` for conditionally present fields per R7
- Optional: run cargo-typify spike first against X API v2 OpenAPI spec. If output is usable, use as starting point for
  curating types. If not, write manually (bounded at ~15 structs)

  **Patterns to follow:**
- `src/store/types.rs` — serde derive patterns, field attributes
- `src/api/shortcuts.rs` `PostBody`/`PostReply` — request body struct patterns (response types mirror these with
  `Deserialize` added)
- `src/store/` directory module — the split pattern (types.rs extracted from mod.rs)

  **Test scenarios:**
- Happy path: Deserialize valid single-tweet API response JSON into `ApiResponse<Tweet>` — all fields populated
  correctly
- Happy path: Deserialize valid list response JSON into `ApiResponse<Vec<Tweet>>` — correct Vec length and field values
- Happy path: Deserialize action confirmation `{"data": {"liked": true}}` into `ApiResponse<LikedResult>` — bool field
  correct
- Happy path: Deserialize response with `includes` and `meta` fields — optional wrapper fields populated
- Edge case: Deserialize response with unknown/extra fields — succeeds, unknown fields captured in `extra` BTreeMap (R8)
- Edge case: Unknown fields round-trip — deserialize JSON with unknown field `"new_field": 42`, serialize back to JSON,
  verify `"new_field": 42` is present in the output (flatten preservation)
- Edge case: Nested unknown fields — `Tweet` with unknown field AND `TweetPublicMetrics` with unknown field — both
  captured in their respective `extra` BTreeMaps
- Edge case: `extra` is empty HashMap when no unknown fields present — serialization produces no extra keys
- Edge case: Deserialize response with missing optional fields — fields are `None`, no error (R7)
- Edge case: `Default::default()` produces a valid struct for each type (R9) — verify with struct update syntax
- Error path: Deserialize completely invalid JSON (wrong types, missing required `id`) — returns serde error
- Round-trip: Serialize typed struct to JSON via `serde_json::to_value()`, then deserialize back — identical struct
- Adversarial: Type confusion — API returns array `[]` where object `{}` expected for `data` field → serde error, not
  panic
- Adversarial: Type confusion — API returns `{"data": "string"}` where `data` should be Tweet object → serde error
- Adversarial: `data` field is `null` (`{"data": null}`) → serde error for non-Option `data: T`
- Adversarial: Numeric overflow — `{"data": {"id": "123", "public_metrics": {"like_count": 99999999999999}}}` →
  deserializes correctly (u64) or returns serde error, no panic
- Adversarial: Deeply nested unknown fields — `{"data": {"id": "123"}, "extra": {"a": {"b": {"c": [1,2,3]}}}}` → unknown
  fields silently skipped per R8, no stack overflow
- Adversarial: Empty string for required String fields — `{"data": {"id": "", "text": ""}}` → deserializes (empty string
  is valid String), consumer must validate semantically

  **Verification:**
- `cargo test` passes for all new deserialization tests
- `cargo doc` shows the new types in the public API surface under `xurl::api::response`
- Module compiles with no warnings

- [ ] **Unit 2: Migrate shortcut functions to typed returns + update tests**

  **Goal:** Change all 29 shortcut function return types from `Result<Value>` to `Result<ApiResponse<T>>`, add
  deserialization at the shortcut boundary, and update all test assertions atomically. Return type changes and test
  updates land together so tests never break (zero broken tests policy).

  **Requirements:** R1, R4, R9

  **Dependencies:** Unit 1

  **Files:**
- Modify: `src/api/shortcuts.rs`
- Modify: `tests/api_tests.rs`
- Modify: `tests/wiring_tests.rs` (if any assertions on Value fields)

  **Approach:**
- Change each shortcut's return type from `Result<serde_json::Value>` to `Result<ApiResponse<T>>` where T matches the
  response category (see decision matrix in High-Level Technical Design)
- Extract a `deserialize_response<T: DeserializeOwned>(value: Value) -> Result<ApiResponse<T>>` helper that: (1) guards
  against empty `{}` Value with a descriptive `XurlError`, (2) calls `serde_json::from_value()` taking ownership (no
  clone needed, per transport layer learning). All 29 shortcuts call this helper instead of raw `from_value()`. DRY:
  guard logic lives in one place.
- `send_request()` remains unchanged — still returns `Result<Value>`
- The `From<serde_json::Error>` impl on `XurlError` already handles deserialization errors
- All 29 functions follow the same mechanical pattern — call `deserialize_response::<T>(value)?` after `send_request()`
- Update existing wiremock-based tests in `api_tests.rs` to assert on typed struct fields instead of
  `resp["data"]["id"]` indexing. Wiremock mock responses stay the same — only the assertion side changes.
- Add `Default` trait usage tests showing struct update syntax: `Tweet { id: "123".into(), ..Default::default() }`
- Run conformance tests — note: only 3 of 37 test cases use `stdout_json` structural comparison; most compare only exit
  codes. Wiremock tests are the primary safety net for output equivalence.

  **Execution note:** Migrate in groups by response category (tweet shortcuts, user shortcuts, action shortcuts, DM
  shortcuts, usage). For each group: change return types AND update corresponding test assertions in the same commit.
  Each group can be verified independently with `cargo test`.

  **Patterns to follow:**
- Existing shortcut function structure in `src/api/shortcuts.rs`
- `serde_json::from_value()` ownership semantics (consumes the Value, no clone)
- Existing `TestServer` and mock patterns in `tests/api_tests.rs`
- `assert_cmd` patterns in `tests/wiring_tests.rs`

  **Test scenarios:**
- Happy path: `create_post()` via wiremock returns `ApiResponse<Tweet>` with correct `data.id` and `data.text`
- Happy path: `search_posts()` via wiremock returns `ApiResponse<Vec<Tweet>>` with correct list length
- Happy path: `get_me()` via wiremock returns `ApiResponse<User>` with correct `data.username`
- Happy path: `like_post()` via wiremock returns `ApiResponse<LikedResult>` with `data.liked == true`
- Happy path: `send_dm()` via wiremock returns `ApiResponse<DmEvent>` with correct event fields
- Happy path: Existing `test_create_post` asserts `response.data.id == "99999"` instead of `response["data"]["id"] ==
  "99999"`
- Happy path: Existing list endpoint tests assert `response.data.len()` and `response.data[0].id`
- Happy path: Action confirmation tests assert `response.data.liked == true` (typed bool, not Value)
- Happy path: Factory pattern works — `Tweet { id: "test".into(), text: "hello".into(), ..Default::default() }` compiles
  and produces valid struct
- Edge case: API returns response with extra fields not in struct — deserialization succeeds
- Edge case: `send_request()` returns empty `{}` (empty body fallback) — shortcut returns descriptive `XurlError`, not
  cryptic serde error
- Error path: API returns malformed JSON that doesn't match expected type — returns `XurlError::Json`
- Adversarial: Wiremock returns `{"data": [{"id": "1"}]}` (array) for a single-item shortcut like `create_post()` →
  `XurlError::Json`, not panic or silent wrong type
- Adversarial: Wiremock returns `{"errors": [{"message": "forbidden"}]}` with no `data` field for a 200 response →
  `XurlError::Json` (data field is required, not Optional)
- Adversarial: Wiremock returns `{"data": {"liked": true}, "extra_top_level": "ignored"}` for `like_post()` → succeeds,
  extra top-level fields captured in `extra`
- Integration: End-to-end shortcut call via wiremock → typed response with all fields accessible by name
- Integration: Conformance tests pass — `stdout_json` tests use structural comparison (field-order independent)
- Integration: `assert_cmd` tests for representative commands show unchanged stdout JSON

  **Verification:**
- `cargo test` passes with zero failures at every commit (no broken-tests window)
- Shortcut functions compile with new typed return signatures
- `cargo clippy` clean
- No remaining `resp["data"]` or `resp["data"]["id"]` style indexing in test assertions
- Conformance runner shows no regressions

- [ ] **Unit 3: Update CLI dispatch and media Value access sites**

  **Goal:** Update the 6 `serde_json::Value` field access sites to use typed struct fields, and add `to_value()`
  conversion at the CLI dispatch layer for output formatting.

  **Requirements:** R1, R4

  **Dependencies:** Unit 2

  **Files:**
- Modify: `src/cli/commands/mod.rs`
- Modify: `src/api/media.rs`

  **Approach:**
- **CLI dispatch** (`cli/commands/mod.rs`): Each match arm in `run_subcommand` receives a typed response from the
  shortcut function. Convert to Value via `serde_json::to_value(&response)?` before calling
  `out.print_response(&value)`. The output layer remains unchanged.
- **resolve_my_user_id** (line ~402): Change from `resp["data"]["id"].as_str()` to `resp.data.id` (or
  `resp.data.id.as_str()` depending on field type). Returns `Result<String>`.
- **resolve_user_id** (line ~417): Same pattern as above.
- **media.rs** (4 Value accesses + 3 print_response calls): Define `MediaUploadResponse` and `MediaProcessingInfo` types
  in Unit 1's types.rs. Replace `response["data"]["id"].as_str()` with `response.data.id`, and
  `response["data"]["processing_info"]["state"].as_str()` with `response.data.processing_info.as_ref().map(|p|
  p.state.as_str())` (or similar typed access). The media upload flow calls `send_request()` directly (not through
  shortcuts), so add `from_value()` deserialization at each call site in media.rs. Additionally, media.rs calls
  `out.print_response()` on intermediate responses (init, finalize, processing status) — these need `to_value()`
  conversion before printing, same as CLI dispatch. Note: `handle_media_append_request` and `send_multipart_request`
  remain on the Value path (they use multipart, not JSON responses).

  **Patterns to follow:**
- Existing `run_subcommand` dispatch pattern in `src/cli/commands/mod.rs`
- `serde_json::to_value()` for re-serialization

  **Test scenarios:**
- Happy path: CLI handler converts `ApiResponse<Tweet>` to Value and prints — output matches raw Value format
- Happy path: `resolve_my_user_id` extracts user ID from typed `ApiResponse<User>` response
- Happy path: `resolve_user_id` extracts target user ID from typed `ApiResponse<User>` response
- Happy path: Media upload init extracts media ID from typed `MediaUploadResponse`
- Happy path: Media status poll reads processing state and check_after_secs from typed `MediaProcessingInfo`
- Edge case: Media `processing_info` absent (completed upload) — `Option` is `None`, no panic
- Edge case: Media print_response calls use `to_value()` on typed response before printing
- Integration: CLI output for a tweet create command is semantically equivalent JSON (field ordering may differ;
  validate with structural comparison, not byte-for-byte string comparison)
- Integration: Unknown API fields survive CLI round-trip — wiremock returns tweet with unknown field `"brand_new_field":
  "surprise"`, CLI handler deserializes to `ApiResponse<Tweet>`, converts via `to_value()`, verify `brand_new_field`
  appears in the serialized output. This is the end-to-end proof that `serde(flatten)` preserves unknown fields through
  the full CLI dispatch path.

  **Verification:**
- All 6 Value field access sites eliminated — `grep` for `["data"]` indexing in production code returns zero matches
- Media upload flow works end-to-end in tests (including intermediate print_response calls)
- CLI output semantically equivalent for all commands (same data, structure; field ordering may differ)

- [ ] **Unit 4: Add spec-as-test validation and update script**

  **Goal:** Create infrastructure for keeping typed response structs in sync with the X API v2 OpenAPI spec. A CI test
  catches type drift. A developer script shows what changed and helps promote new fields.

  **Requirements:** R10

  **Dependencies:** Unit 1

  **Files:**
- Create: `scripts/update-types.sh`
- Create: `tests/spec_validation.rs`
- Create: `tests/fixtures/openapi/` (directory for cached spec file)
- Test: `tests/spec_validation.rs`

  **Approach:**
- `scripts/update-types.sh`: Fetches the latest X API v2 OpenAPI spec JSON, runs `cargo typify` on it to generate
  reference types, and diffs against the hand-written `src/api/response/types.rs` to show what fields are new, changed,
  or removed. Output is a readable diff, not an automated edit. Developer reviews and promotes fields manually.
- `tests/spec_validation.rs`: Loads a cached copy of the OpenAPI spec from `tests/fixtures/openapi/`, extracts example
  response payloads for each endpoint used by the 29 shortcuts, and attempts to deserialize each into the corresponding
  `ApiResponse<T>`. If deserialization fails, the test names the specific type and field that drifted. Unknown fields
  landing in `extra` are acceptable (flatten handles them). The test fails only when a required field is missing or a
  type mismatch occurs.
- The cached spec file is checked into the repo. `scripts/update-types.sh` also refreshes this cached copy. CI tests run
  against the cached copy for reproducibility.

  **Patterns to follow:**
- Existing test patterns in `tests/api_tests.rs` (inline JSON fixtures)
- `scripts/hooks/pre-push` pattern for developer-facing scripts

  **Test scenarios:**
- Happy path: All current types deserialize from spec example responses without error
- Happy path: Spec has new unknown fields not in structs — deserialization succeeds, fields land in `extra`
- Edge case: Spec removes a field that our struct has as `Option<T>` — deserialization still succeeds (field is None)
- Error path: Spec changes a field type (e.g., `id` from string to integer) — test fails with clear message naming the
  type and field
- Error path: Spec adds a new required field our struct doesn't have — test fails if the field has no default and isn't
  in `extra`

  **Verification:**
- `cargo test --test spec_validation` passes
- `scripts/update-types.sh` runs without error and produces a readable diff
- Script is executable and documented in README or AGENTS.md

## System-Wide Impact

- **Interaction graph:** `send_request()` → shortcut functions → CLI dispatch → `print_response()`. The change happens
  at the shortcut→dispatch boundary. `send_request()` and `print_response()` are unchanged.
- **Error propagation:** Deserialization errors from `serde_json::from_value()` propagate as `XurlError::Json` via the
  existing `From` impl. No new error variants needed.
- **State lifecycle risks:** None. Response types are ephemeral (created, used, dropped). No persistence or caching.
- **API surface parity:** This is an intentional breaking change to the library API. The CLI binary behavior is
  unchanged. Downstream consumers (`bird`) must update imports and switch from Value indexing to struct field access.
- **Integration coverage:** Wiremock tests are the primary safety net for typed response correctness. The conformance
  runner provides limited end-to-end coverage (3 of 37 tests compare `stdout_json` structurally; most compare only exit
  codes). Together with `assert_cmd` tests, they cover both library and CLI interfaces.
- **Unchanged invariants:** `send_request()` return type, output layer formatting, auth flow, streaming endpoints,
  request body types — none of these change.

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| X API response shape assumptions wrong | Permissive deserialization (no `deny_unknown_fields`), liberal `Option<T>`, test against real API shapes via wiremock fixtures |
| CLI output field ordering differs after round-trip | JSON output is semantically equivalent but field order follows struct declaration, not API response order. Conformance runner uses structural JSON comparison. Use `#[serde(skip_serializing_if = "Option::is_none")]` to prevent null field injection |
| Breaking library API change | Intentional per origin doc. Requires semver major bump to 2.0.0. `bird` is the only known consumer and drives the change |
| cargo-typify spike fails | Manual struct writing is the fallback. ~15 structs is manageable. Plan does not depend on typify |
| Media upload flow has undocumented response variations | Define media types conservatively with `Option<T>` fields. Test each upload step independently. `handle_media_append_request` and `send_multipart_request` remain on Value path |
| Media upload may not use `{"data": {...}}` envelope | The v2 media endpoint wraps v1.1 internally. `check_media_status` uses v1.1-style query params. **Validate against live API before implementing media types.** If envelope differs, media deserializes directly to `MediaUploadResponse` (not wrapped in `ApiResponse<T>`). Test fixtures are synthetic mocks, not validated against real API. |
| Empty response body from `send_request()` | Guard in `deserialize_response` helper: check for empty `{}` Value before `from_value()`, return descriptive error. X API v2 consistently returns JSON bodies for covered endpoints — defensive edge case |
| X API OpenAPI spec may lack example responses | Unit 4 spec-as-test depends on example payloads in the spec. If examples are missing for some endpoints, fall back to hand-written test fixtures derived from wiremock mocks in `api_tests.rs`. The spec-as-test validates what it can; missing endpoints are documented as coverage gaps, not test failures. |

## Sources & References

- **Origin document:**
  [docs/brainstorms/2026-03-21-typed-responses-requirements.md](docs/brainstorms/2026-03-21-typed-responses-requirements.md)
- Related brainstorm:
  [docs/brainstorms/2026-03-21-api-mock-tests-requirements.md](docs/brainstorms/2026-03-21-api-mock-tests-requirements.md)
  (typed responses land first, mock tests benefit from struct assertions)
- Institutional learning: `docs/solutions/architecture-patterns/code-review-round2-quality-improvements.md` (parse JSON
  once at transport)
- Institutional learning: `docs/solutions/rust-patterns/rust-cli-port-methodology.md` (return type honesty)
- Institutional learning: `docs/solutions/best-practices/rust-module-splitting-srp-not-loc-20260327.md` (keep
  shortcuts.rs cohesive)
- Institutional learning: `docs/solutions/architecture-patterns/xurl-subprocess-transport-layer.md` (avoid Value
  cloning)
- Institutional learning:
  `docs/solutions/integration-issues/expensify-integration-server-api-undocumented-behavior-20260324.md`
  (forward-compatible deserialization)

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 0 | — | — |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | — |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | CLEAR (PLAN) | 5 issues, 0 critical gaps |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | — |
| Outside Voice | (Claude subagent) | Independent plan challenge | 1 | ISSUES | 13 findings, 3 accepted |

- **OUTSIDE VOICE:** 13 findings from Claude subagent. 3 accepted (merge Units 2+4, BTreeMap for deterministic ordering,
  media live validation risk). 10 dismissed (performance overrated for CLI, print_response needs Value for colorization,
  minor doc/testing points).
- **UNRESOLVED:** 0
- **VERDICT:** ENG CLEARED — ready to implement. Run `/ce-work` when ready.
