---
date: 2026-04-03
topic: library-ergonomics
---

# Library Ergonomics for Crate Consumers

## Problem Frame

xurl-rs v1.1.0 ships a library API with typed responses and 29 shortcut functions, but two API design choices optimized
for CLI-internal use create friction for crate consumers. The primary downstream consumer (bird) is migrating from
subprocess to crate import and has identified specific ergonomic gaps:

1. `ApiClient<'a>` borrows `&'a mut Auth`, preventing simple ownership in consumer structs (self-referential borrow).
   Consumers must use closure-scoping workarounds (`with_api()`) to manage the lifetime.
2. `XurlError::Api(String)` contains raw JSON but doesn't expose the HTTP status code structurally, forcing both
   consumers and xurl's own CLI (`exit_code_for_error` in `src/main.rs` string-matches on `msg.contains("401")`) to
   parse strings to classify errors.

A third gap — private resolve helpers (`resolve_my_user_id`, `resolve_user_id`) — was evaluated and excluded as Layer 3
composition that belongs in consumers (see Scope Boundaries).

These are non-blocking (bird can work around both), but fixing them in xurl makes crate embedding a first-class pattern
alongside subprocess use.

### API Layer Architecture

xurl's library API has a clear layering that informs scope boundaries:

```text
Layer 3: Composition + caching    Consumer responsibility (bird, future consumers)
Layer 2: Typed shortcuts           29 functions — get_me(), like_post(), etc. (xurl library API)
Layer 1: Raw request               send_request() — untyped JSON passthrough (xurl library API)
```

xurl owns Layers 1-2. Consumers own Layer 3, which includes composing shortcuts (e.g., calling `get_me()` and extracting
`.data.id`), caching results, and applying business logic. This boundary keeps xurl's API surface focused on typed
access to X API endpoints while giving consumers full control over composition and performance.

## Requirements

**Owned Client Type**

- R1. Replace `ApiClient<'a>` with an owned `ApiClient` that holds `Auth` and `Config` by value, eliminating the
  lifetime parameter. Consumers create one instance and use it for the session.
- R2. Provide a `from_env()` constructor that reads env vars, creates `Config` and `Auth`, and returns a ready-to-use
  client. One-liner setup for consumers.
- R3. The owned client must support all 29 shortcut functions and `send_request()` — same API surface as the previous
  `ApiClient<'a>`.

**Structured Error**

- R4. Change `XurlError::Api(String)` to `Api { status: u16, body: String }` for HTTP error responses (status >= 400).
  Consumers and xurl's own CLI can pattern-match on status codes directly.
- R5. Introduce `XurlError::Validation(String)` for non-HTTP uses of `Api` — at least 10 call sites currently construct
  `XurlError::Api(String)` for validation errors, user-not-found, schema errors, media processing failures, and
  error-only 200 responses where no HTTP status code exists. These must not be forced into `Api { status, body }` with a
  fabricated status code. The 10th site is `deserialize_response()` in `types.rs:325` (errors-only 200 responses with no
  `data` field). Display: `#[error("{0}")]` (body-only, same as current behavior).

**CallOptions Type**

- R6. Introduce `CallOptions` to replace `RequestOptions` in shortcut method signatures. `CallOptions` exposes only
  consumer-relevant fields (`auth_type`, `username`, `no_auth`, `verbose`). Internal request fields (`method`,
  `endpoint`, `data`, `query_params`) are set by the shortcut itself. `RequestOptions` remains for `send_request()` (raw
  power path). This stops leaking internal request structure to crate consumers.

**Shortcuts as Methods**

- R7. Convert all 29 shortcut free functions to methods on `ApiClient`. Call pattern changes from `api::create_post(&mut
  client, text, &[], &opts)` to `client.create_post(text, &[], &call_opts)`. The `shortcuts.rs` file becomes an `impl
  ApiClient { ... }` block (Rust allows split impl blocks across modules). This is the Rust-idiomatic pattern used by
  reqwest, octocrab, and every other Rust HTTP client library.

**exit_code_for_error**

- R8. Move `exit_code_for_error()` from private `src/main.rs` to public library (`src/cli/exit_codes.rs`). The function
  pattern-matches on `Api { status, .. }` directly instead of string-scanning the body for "401"/"429"/"404". Enables
  unit testing and optional reuse by consumers.

**Display Format**

- R9. The `Display` impl for `Api { status, body }` must preserve the current behavior of emitting the body string only
  (`#[error("{body}")]`), not `"HTTP 401: {body}"`. This maintains CLI output compatibility and subprocess parsing for
  consumers using xurl via `xr`.

## Success Criteria

- bird can hold `ApiClient` as a struct field without lifetime gymnastics
- bird can pattern-match `XurlError::Api { status: 401, .. }` for auth error classification
- bird calls `client.create_post(text, &[], &call_opts)` (method + CallOptions)
- xurl CLI (`xr`) continues to work identically — internal CLI code updated to use the new API
- `exit_code_for_error` pattern-matches on `XurlError::Api { status, .. }` instead of string-matching on the body
- `from_env()` validates Config (non-empty client_id) and returns `Result<ApiClient>`
- `output.rs` `error_kind()` handles `Validation` variant (returns `"validation"`)
- All existing tests pass; new tests cover the additions (13 new tests, 13 mechanical updates, 1 regression fix)

## Scope Boundaries

- No changes to CLI behavior or output
- No changes to typed response structs (`ApiResponse<T>`, `Tweet`, `User`, etc.)
- No changes to shortcut return types (parameter changes: `&mut ApiClient` → `&mut self`, `&RequestOptions` →
  `&CallOptions`)
- No new API endpoints or shortcut functions
- No async API (blocking only). Async + concurrent `ApiClient` is a future requirement (P2 TODO: interior mutability via
  `RwLock<TokenStore>` in Auth)
- No resolve helpers (`resolve_my_user_id`, `resolve_user_id`) — Layer 3 composition that belongs in consumers
- No schemars/JsonSchema changes (orthogonal to this work)

## Key Decisions

| Decision | Rationale |
|---|---|
| Breaking changes in v1.2.0 | bird is the only crate consumer, both repos have the same owner, and the subprocess CLI is unaffected. No need for a deprecation cycle. |
| Replace `ApiClient<'a>` (not add alongside) | Clean break. No dual-type confusion. The old borrowed pattern was an internal implementation detail, not a deliberate library design. |
| Structured error enum variant | `Api { status, body }` is more useful than `Api(String)` + helper method. Since we're already making breaking changes, do the clean version. |
| Non-HTTP errors → `Validation(String)` | ~10 call sites construct `XurlError::Api` for non-HTTP errors. Named `Validation` because most are input validation; covers CLI validation, errors-only 200s, media failures, user-not-found. Single variant keeps the enum small. |
| Errors-only 200 responses → Validation | X API v2 returns HTTP 200 with errors-only body. Routing to `Api { status: 200, body }` confuses consumers matching `status >= 400`. Validation communicates "can't classify by HTTP status alone." Cross-model tension: outside voice argued for Api, user confirmed Validation. |
| Shortcuts → methods on ApiClient | Rust-idiomatic (`client.create_post(...)` not `api::create_post(&mut client, ...)`). Matches reqwest, octocrab, every Rust HTTP client. `shortcuts.rs` becomes `impl ApiClient { ... }` block. |
| `CallOptions` replaces `RequestOptions` in shortcuts | Shortcuts internally set method, endpoint, data, query_params. Consumers only control auth_type, username, no_auth, verbose. `RequestOptions` stays for `send_request()` raw path. Stops leaking internal request structure. |
| `from_env()` convenience alongside `new(config, auth)` | `from_env()` is sugar for the common case. `new()` is the power path for consumers needing customization (bird uses `new()` with app_name override). `from_env()` returns `Result<ApiClient>` to validate Config. |
| `exit_code_for_error()` moves to library | Enables unit testing. Pattern-matches `Api { status, .. }` directly instead of string-scanning body. Cross-model tension: outside voice argued bird won't use it; user confirmed move. |
| Config not stored by ApiClient | ApiClient only uses `config.api_base_url` during construction. Stores `base_url: String`, not full Config. Config is consumed, not owned. |
| Display format preserves body-only output | `#[error("{body}")]` keeps CLI stderr output unchanged. Subprocess consumers parsing error output are unaffected. |
| Resolve helpers stay in consumers | Layer 3 composition. xurl owns Layers 1-2. |

## Dependencies / Assumptions

- bird is the primary (and currently only) crate consumer driving these requirements
- Both subprocess and crate embedding remain first-class supported patterns
- Breaking changes are acceptable in v1.x since bird coordinates releases with xurl

## Outstanding Questions

### Resolve Before Planning

(none)

### Deferred to Planning

(all resolved during eng review — see Key Decisions)

- ~~[Affects R1] reqwest::blocking::Client per-request or held?~~ **RESOLVED:** Held by ApiClient (already does this).
  Connection pooling preserved.
- ~~[Affects R7] Methods or free functions?~~ **RESOLVED:** Methods on ApiClient. See R7.
- ~~[Affects R4] Parsed serde_json::Value in Api variant?~~ **RESOLVED:** Keep body as raw String. Consumers parse if
  needed. Simpler.
- ~~[Affects R5] Non-HTTP variant naming?~~ **RESOLVED:** `Validation(String)`. See Key Decisions.

## Next Steps

-> `/ce:plan` for structured implementation planning (can be planned independently from bird migration)
