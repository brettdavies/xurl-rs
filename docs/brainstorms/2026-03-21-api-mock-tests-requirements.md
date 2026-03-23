---
date: 2026-03-21
topic: api-mock-tests
---

# Expand Wiremock Integration Tests for All API Shortcuts

## Problem Frame

xurl-rs has 28 shortcut commands that build X API requests and parse responses. Only 8 are covered by wiremock-based
integration tests (`tests/api_tests.rs`). The remaining 20 shortcuts have no automated verification that they format
requests correctly or parse responses without error. Since hitting the real X API costs money and is unreliable for CI,
mock-based tests are the only practical way to maintain confidence in request/response contracts.

## Requirements

- R1. Every shortcut function in `src/api/shortcuts.rs` has at least one wiremock integration test that verifies the
  outbound HTTP method, endpoint path, and request body structure.
- R2. Every shortcut test validates that a typed API response parses correctly and returns the expected data via struct
  field assertions.
- R3. Per-shortcut test data is built using factory functions (`tests/factories.rs`) that produce typed response
  structs. Factories serialize to JSON for wiremock, testing the full round-trip: struct to JSON to HTTP to JSON to
  struct.
- R4. Golden fixture files (`tests/fixtures/golden/`) contain maximal realistic API responses seeded from the OpenAPI
  spec. Dedicated tests prove that typed structs can parse full real-world API JSON shapes.
- R5. Error response scenarios (400, 401, 403, 429) are tested for at least one representative shortcut using shared
  error fixture files (`tests/fixtures/errors/`).
- R6. Wiremock matchers verify query parameters (e.g., `max_results`, `tweet.fields`, `expansions`) where shortcuts
  construct parameterized URLs.
- R7. Existing inline-fixture tests in `tests/api_tests.rs` are migrated to use factories for consistency.

## Success Criteria

- All 28 shortcut functions have passing wiremock tests covering both request shape and response parsing.
- Golden fixture files contain realistic response shapes based on the X API v2 OpenAPI spec.
- Factory functions cover all response type groups: Tweet, User, DmEvent, action confirmations, media upload.
- `cargo test --test api_tests` passes with zero failures.
- No test requires network access or API credentials.

## Scope Boundaries

- Auth correctness is out of scope. Tests assume auth works (use existing `create_mock_auth_with_bearer` helper).
- Streaming endpoint tests are out of scope beyond the existing `test_stream_request_error`.
- Media upload flow (multi-step INIT/APPEND/FINALIZE/STATUS) is out of scope beyond existing tests.
- Output formatting (`src/output.rs`, `src/api/response.rs`) is out of scope; tests validate the typed response structs
  returned by shortcut functions, not terminal output.

## Key Decisions

- **Factories over per-shortcut fixture files**: Factory functions build typed response structs in code. Serde
  serializes them to JSON for wiremock. This tests the full serialize/deserialize round-trip, eliminates 28+ fixture
  files, and gives compile-time safety on test data. Factory functions live in `tests/factories.rs`.
- **Golden fixtures for contract validation**: A small set of maximal `.json` files in `tests/fixtures/golden/` (seeded
  from the X API v2 OpenAPI spec) prove typed structs can parse real-world API responses. These complement the factory
  round-trip tests.
- **Shared error fixtures**: Error responses live in `tests/fixtures/errors/` (e.g., `400_bad_request.json`,
  `429_rate_limit.json`). Error shapes are generic across endpoints, so sharing avoids duplication.
- **Verify both directions**: Tests assert on outbound request shape (method, path, query params, body) AND inbound
  response parsing. This catches both "we sent the wrong thing" and "we can't parse what comes back."
- **Bearer auth only in new tests**: All new shortcut tests use bearer auth via the existing helper. Testing all three
  auth types per shortcut adds no value since auth routing is already tested.

## Dependencies / Assumptions

- Typed response structs (from the `typed-responses` brainstorm) must land first. Factories depend on typed structs with
  `Default` derives.
- Golden fixture response shapes will be seeded from the X API v2 OpenAPI spec. If the spec is unavailable or ambiguous
  for specific endpoints, fall back to X API v2 documentation examples.
- The existing `TestServer` helper and auth helpers in `tests/api_tests.rs` are sufficient and do not need modification.

## Outstanding Questions

### Deferred to Planning

- *Affects R4 / Needs research:* Locate the X API v2 OpenAPI spec and determine the best way to extract maximal example
  responses for golden fixture files. Could be a one-time manual extraction or a script using `jaq`.
- *Affects R7 / Technical:* Should we migrate existing inline-fixture tests in a separate commit or batch with new
  tests? Separate commit is cleaner for review but adds an extra step.
- *Affects R6 / Needs research:* For shortcuts that build complex query strings (e.g., `read_post` with many
  `tweet.fields`), should wiremock match on exact query string or just key parameters? Exact matching is brittle if
  field order changes; key-parameter matching is more resilient.

## Relationship to Other Brainstorms

- **typed-responses** (2026-03-21): Typed response structs should land first. Once shortcuts return typed structs, mock
  tests use factories to build typed test data and assert on struct fields. Both brainstorms share the OpenAPI spec as
  input.

## Next Steps

`/ce:plan` for structured implementation planning (after typed-responses is complete)
