---
date: 2026-03-21
topic: typed-responses
---

# Typed API Response Structs

## Problem Frame

xurl-rs uses raw `serde_json::Value` for all API responses. This works for a CLI that pipes JSON to the terminal, but
becomes a liability when xurl-rs is consumed as a library crate. The concrete use case: `bird` currently shells out to
the `xurl` CLI and parses JSON output to extract fields like tweet IDs, usernames, and metrics. Replacing that with a
crate dependency requires typed responses so `bird` gets compile-time safety, autocomplete, and documentation instead of
fragile runtime JSON indexing.

Today, only 6 places in production code access fields on `serde_json::Value` (4 in `media.rs`, 2 in
`cli/commands/mod.rs`). All 28 shortcut functions and the output layer treat the Value as an opaque pass-through. This
means the migration surface is well-bounded.

## Requirements

- R1. API shortcut functions return typed response structs instead of `serde_json::Value`. For example, `create_post()`
  returns `Result<ApiResponse<Tweet>>` instead of `Result<serde_json::Value>`.
- R2. A shared set of response types covers the X API v2 surface used by the 28 shortcuts: `Tweet`, `User`, `DmEvent`,
  action confirmations, and media upload responses.
- R3. A generic `ApiResponse<T>` wrapper provides the common response shape: `data: T` (or `Vec<T>` for list endpoints),
  optional `includes`, and optional pagination `meta`.
- R4. All types derive `Serialize` + `Deserialize` so the CLI can serialize typed responses back to JSON for terminal
  output with no behavior change.
- R5. Types are initially generated from the X API v2 OpenAPI spec using `cargo-typify` or equivalent, then hand-curated
  into a clean, idiomatic public API.
- R6. The `xurl` library crate exposes the typed response structs as part of its public API so consumers like `bird` can
  import them directly.
- R7. Fields that are conditionally present (due to `fields` and `expansions` query parameters) are `Option<T>`.
- R8. Response structs are permissive — unknown JSON fields are silently skipped (serde default behavior). This ensures
  forward compatibility when the X API adds new fields.
- R9. All response structs derive `Default` so test factory functions can use struct update syntax (`Tweet { id:
  "123".into(), ..Default::default() }`) to build minimal test data.

## Success Criteria

- All 28 shortcut functions return typed responses. No public function returns raw `serde_json::Value`.
- `cargo test` passes. Existing tests are updated to assert on struct fields.
- The CLI binary (`xr`) produces identical JSON output before and after the migration.
- A downstream crate can `use xurl::api::response::{Tweet, User, ApiResponse}` and access fields with compile-time
  safety.

## Scope Boundaries

- Auth types (`OAuth1Token`, `OAuth2Token`, etc.) are already typed and out of scope.
- Request body types (`PostBody`, `PostReply`, etc.) are already typed and out of scope.
- The CLI output layer (`src/output.rs`, `src/api/response.rs`) will serialize typed structs back to JSON. Its
  syntax-highlighting behavior does not change.
- Streaming endpoint responses are out of scope for typed responses in this phase (they are line-by-line JSON).
- The OpenAPI spec is used as a generation starting point, not as a runtime dependency.

## Key Decisions

- **Approach C (generate then curate)**: Generate types from the X API v2 OpenAPI spec, then hand-curate into a clean
  public API. Balances correctness with ergonomics.
- **Replace, not dual-maintain**: Shortcut functions replace `Value` returns with typed returns. No parallel `_typed()`
  variants. Clean API surface.
- **CLI output unchanged**: Typed structs serialize back to JSON via serde. The terminal experience is identical.
- **Sequencing**: This work precedes the mock testing expansion (from the `api-mock-tests` brainstorm). Once typed
  responses exist, mock tests assert on struct fields rather than JSON paths.

## Dependencies / Assumptions

- The X API v2 OpenAPI spec is publicly available and covers the endpoints used by the 28 shortcuts.
- `cargo-typify` (already installed) can generate usable Rust types from the spec's JSON Schema components.
- The curated type set is approximately 10-15 structs. The X API v2 surface that xurl-rs touches is bounded.
- `bird` is the immediate downstream consumer and drives the "what fields matter" prioritization.

## Outstanding Questions

### Deferred to Planning

- *Affects R3 / Needs research:* How should `ApiResponse<T>` handle endpoints that return a single item (`data: T`) vs.
  a list (`data: Vec<T>`)? Options: two generic variants, a trait, or an enum.
- *Affects R5 / Needs research:* Does `cargo-typify` produce usable output from the X API v2 OpenAPI spec, or will
  manual extraction be needed? Run a spike to evaluate.
- *Affects R2 / Technical:* What should the action confirmation types look like? The API returns varied shapes like
  `{"data": {"liked": true}}`, `{"data": {"following": true}}`, `{"data": {"deleted": true}}`. Options: one generic
  struct with optional fields, separate small structs per action, or a shared enum.
- *Affects R1 / Technical:* The `send_request()` method on `ApiClient` currently returns `Value`. Should it become
  generic (`send_request<T: DeserializeOwned>`) or should deserialization happen in each shortcut function?
- *Affects R4 / Technical:* How should the output layer handle serialization back to JSON? Direct
  `serde_json::to_value()` then existing pretty-print, or refactor to serialize directly?

## Relationship to Other Brainstorms

- **api-mock-tests** (2026-03-21): Typed responses should land first. The mock testing expansion then benefits from
  struct-level assertions. The fixture files from that brainstorm still apply — they define the JSON shapes that the
  typed structs must deserialize from.

## Next Steps

`/ce:plan` for structured implementation planning
