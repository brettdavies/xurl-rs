---
title: "feat: Add --remote flag for headless OAuth2 authentication"
type: feat
status: completed
date: 2026-03-26
deepened: 2026-03-26
---

# feat: Add --remote flag for headless OAuth2 authentication

## Overview

Add a two-step `--remote` flag to `xr auth oauth2` that enables OAuth2 PKCE authentication on headless machines (SSH,
containers, CI) where no browser or localhost callback server is available. Step 1 generates the auth URL and persists
PKCE state to disk. Step 2 accepts the redirect URL pasted from a browser on another machine and completes the token
exchange.

## Problem Frame

On headless machines, `xr auth oauth2` cannot complete because it requires a local browser to authorize and a localhost
callback server to receive the redirect. Users must auth on a machine with a browser and manually copy `~/.xurl` back.
This is a well-known gap — `gogcli` solves it with an identical two-step `--remote` pattern.

## Requirements Trace

- R1. `xr auth oauth2 --remote --step 1` prints an auth URL to stdout and persists PKCE state to `~/.xurl.pending`
- R2. `xr auth oauth2 --remote --step 2 --auth-url <url>` extracts the code from the redirect URL, loads PKCE state,
  exchanges for a token, and saves to `~/.xurl`
- R3. Tokens saved identically to the interactive flow (same `save_oauth2_token` path)
- R4. Works on headless Linux without a browser or display
- R5. Clear error messages guide the user through the two-step process
- R6. `--remote` without `--step` is an error (must be explicit)
- R7. `--remote --step 2` without a prior step 1 (no pending file) is an error
- R8. Pending state file uses 0o600 permissions and is deleted after successful step 2
- R9. `--auth-url -` reads the redirect URL from stdin to avoid procfs exposure on shared machines
- R10. Step 2 validates that the pending file's `client_id` matches the runtime auth context and hard-fails on mismatch
- R11. Pending state older than 15 minutes is rejected at step 2

## Scope Boundaries

- No changes to the interactive (non-remote) OAuth2 flow
- No new dependencies — use existing `serde_yaml`, `rand`, `sha2`, `base64`, `url`, `dirs` crates
- No changes to the token store format — remote flow saves tokens via the same `save_oauth2_token` path
- No `--redirect-uri` override flag in this iteration (can be added later if needed)

## Context & Research

### Relevant Code and Patterns

- `src/auth/oauth2.rs` — `run_oauth2_flow()` holds the full PKCE flow; `generate_code_verifier_and_challenge()` already
  extracted as a reusable function
- `src/auth/mod.rs` — `Auth` struct with `oauth2_flow()` facade method; accessors for `client_id()`, `auth_url()`,
  `token_url()`, `redirect_uri()`
- `src/auth/callback.rs` — localhost callback server (bypassed entirely in remote mode)
- `src/cli/mod.rs:443` — `AuthCommands::Oauth2` currently a unit variant; needs fields for `--remote`, `--step`,
  `--auth-url`
- `src/cli/commands/mod.rs:508` — `AuthCommands::Oauth2` handler calls `auth.oauth2_flow("")` with no args
- `src/store/mod.rs` — `TokenStore::save_oauth2_token()`, file path from `dirs::home_dir()`, 0o600 permissions pattern
- `src/error.rs` — `XurlError::auth(msg)` and `XurlError::auth_with_cause(prefix, cause)` patterns

### Institutional Learnings

- **Token file permissions** (security audit): Always 0o600 via `OpenOptionsExt::mode()`. Apply to `~/.xurl.pending`.
- **Headless OAuth pattern** (rclone/Box solution): Authorize on a browser machine, paste token/URL to headless machine.
  Proven UX pattern — this feature implements the same concept natively.
- **PKCE testing gap** (security audit): Missing unit tests for `generate_code_verifier_and_challenge()`. Address in
  unit 4 as incremental improvement.
- **`--no-interactive` flag**: Already exists in CLI for headless/agent pipelines. Remote flow should work correctly
  alongside it.

## Key Technical Decisions

- **Separate pending state file (`~/.xurl.pending`)**: PKCE verifier and state are temporary flow state, not
  credentials. Keeping them in a separate file avoids polluting the token store format, simplifies cleanup, and isolates
  failure modes. File is YAML with 0o600 permissions, deleted after step 2.
- **Refactor `run_oauth2_flow` into composable parts**: Extract URL generation and code-for-token exchange into separate
  functions so both the interactive and remote flows reuse the same logic without duplication.
- **`--step` as a required companion to `--remote`**: Explicit step selection prevents ambiguity. `--remote` alone is an
  error. `--step` without `--remote` is also an error (validated via clap `requires`).
- **`--auth-url` flag for step 2 with stdin fallback**: The user pastes the full redirect URL (including
  `?code=...&state=...`). The CLI parses it — no need for the user to manually extract the code parameter. Passing
  `--auth-url -` reads from stdin instead, which avoids exposing the auth code in `/proc/*/cmdline` and `ps` output on
  shared machines. Stdin is the recommended path in help text; the direct flag value is a convenience fallback.
- **Client ID validation between steps**: Step 2 loads the pending file's `client_id` and compares it against the
  runtime `auth.client_id()`. If they differ (from `--app` change, env var drift, or store mutation between steps), step
  2 hard-fails with a diagnostic error naming the expected app. This prevents confusing Twitter API errors from
  credential mismatches and catches a confused-deputy condition where a token could be stored under the wrong app.
- **Pending file hardening**: Write to a temp file in the same directory (e.g., `~/.xurl.pending.tmp`) with `create_new`
  (O_CREAT | O_EXCL) and 0o600 permissions, then atomically rename to `~/.xurl.pending`. This avoids the TOCTOU symlink
  race that exists in a delete-then-create sequence (between the delete and the create, an attacker could place a
  symlink). If the temp file already exists, delete it first (temp files are not security-sensitive). If the final
  pending file already exists, warn to stderr that the previous pending flow is being abandoned — the rename overwrites
  it atomically. At step 2, re-check file permissions are still 0o600 and owner is the current uid before reading
  (`#[cfg(unix)]` guarded — see below). Enforce a 15-minute TTL using the `created_at` field — reject stale pending
  files to bound the PKCE verifier exposure window. On expired file rejection, delete the stale file to avoid leaving
  verifier material on disk.
- **Cross-platform permission guards**: Permission verification (0o600 check) and owner validation (uid check) require
  Unix-specific APIs (`PermissionsExt`, `libc::getuid()`). Gate these checks behind `#[cfg(unix)]`, matching the
  existing pattern in `store/mod.rs`. On non-Unix platforms, the checks are skipped — this is acceptable because R4
  targets headless Linux and the primary threat model is multi-user Unix machines.
- **State parameter entropy**: Both the PKCE verifier and the state parameter use 256 bits (32 bytes) from a CSPRNG
  (`rand::random`), matching the existing implementation in `oauth2.rs`.

## Open Questions

### Resolved During Planning

- **Q: Should `--step` accept strings ("begin"/"complete") or numbers (1/2)?** Numbers — matches gogcli precedent,
  shorter to type, unambiguous.
- **Q: Should step 1 also print instructions, or just the URL?** Print both: a brief instruction line and the URL on its
  own line for easy copy-paste. JSON/JSONL output mode emits structured JSON with the URL field.
- **Q: Should pending state expire?** Yes — enforce a 15-minute TTL at step 2 using the `created_at` field. This bounds
  the PKCE verifier exposure window on disk and is slightly longer than Twitter's ~10-minute auth code lifetime, giving
  the user reasonable time to copy the redirect URL.
- **Q: Where does pending state file logic live?** In a new `src/auth/pending.rs` module — keeps `oauth2.rs` focused on
  the OAuth2 protocol and `store/mod.rs` focused on credentials.

### Deferred to Implementation

- **Exact output format for step 1 instructions**: Will be refined during implementation based on what reads well in a
  terminal. The structured JSON output should include `auth_url` and `instructions` fields.
- **Whether `generate_code_verifier_and_challenge` needs additional unit tests**: The security audit flagged this as a
  gap. Check current test coverage and add if missing.

## High-Level Technical Design

> *This illustrates the intended approach and is directional guidance for review, not implementation specification. The
> implementing agent should treat it as context, not code to reproduce.*

```text
Step 1 (headless machine):
  $ xr auth oauth2 --remote --step 1
  → if ~/.xurl.pending exists, warn "Overwriting previous pending auth flow" to stderr
  → generate PKCE verifier + challenge (256-bit CSPRNG)
  → generate state parameter (256-bit CSPRNG)
  → build auth URL with client_id, redirect_uri, scopes, state, challenge
  → write {verifier, state, client_id, redirect_uri, app_name, created_at} to ~/.xurl.pending.tmp
    (create_new + 0o600), then atomic rename to ~/.xurl.pending
  → print auth URL + instructions to stdout

  User copies URL → opens in browser on another machine → authorizes → browser redirects to callback URL
  User copies the redirect URL from browser address bar (it will show an error page since no callback server)

Step 2 (headless machine):
  $ xr auth oauth2 --remote --step 2 --auth-url -    # reads URL from stdin (recommended)
  $ xr auth oauth2 --remote --step 2 --auth-url "http://localhost:8080/callback?code=abc&state=xyz"
  → read ~/.xurl.pending
  → verify file permissions are still 0o600 and owner is current uid
  → verify created_at is within 15-minute TTL
  → validate pending.client_id matches runtime auth.client_id() (catch app/env mismatch)
  → parse --auth-url to extract code and state
  → validate state matches pending state
  → exchange code for token (POST to token_url with verifier from pending file)
  → resolve username via /2/users/me
  → save token via token_store.save_oauth2_token()
  → delete ~/.xurl.pending
  → print success message
```

## Implementation Units

- [x] **Unit 1: Add pending state persistence module**

  **Goal:** Create `src/auth/pending.rs` to save and load PKCE pending state to/from `~/.xurl.pending`.

  **Requirements:** R1, R2, R7, R8

  **Dependencies:** None

  **Files:**
- Create: `src/auth/pending.rs`
- Modify: `src/auth/mod.rs` (add `mod pending;`)
- Test: `tests/auth_pending.rs`

  **Approach:**
- Define a `PendingOAuth2State` struct with serde Serialize/Deserialize: `code_verifier`, `state`, `client_id`,
  `redirect_uri`, `app_name` (all String), `created_at` (u64, Unix seconds)
- All functions accept an explicit `path: &Path` parameter for test isolation (matching the `TokenStore::new_with_path`
  pattern). Add `default_pending_path() -> PathBuf` convenience function using `dirs::home_dir().join(".xurl.pending")`
- `save(state: &PendingOAuth2State, path: &Path)` — serialize to YAML, write to a temp file (`{path}.tmp`) with
  `create_new` (O_CREAT | O_EXCL) and 0o600 permissions via `OpenOptionsExt::mode()`, then atomically rename to `path`.
  If the temp file already exists, delete it first. The rename atomically overwrites any existing pending file
- `load(path: &Path)` — on Unix (`#[cfg(unix)]`), verify permissions are 0o600 and owner is current uid before reading;
  verify `created_at` is within 15-minute TTL; if expired, delete the stale file and return error; deserialize YAML into
  `PendingOAuth2State`
- `delete(path: &Path)` — remove the file
- `exists(path: &Path) -> bool` — check if pending file exists (for overwrite warning in step 1)

  **Patterns to follow:**
- `store/mod.rs` — YAML serialization with serde, 0o600 permissions pattern, `dirs::home_dir()` usage
- `error.rs` — `XurlError::auth()` for missing/corrupt/stale/permission-mismatch pending file errors

  **Test scenarios:**
- Save and load round-trip preserves all fields
- Load from nonexistent file returns descriptive error
- Delete removes the file
- File permissions are 0o600 on Unix
- Load rejects file older than 15 minutes with TTL error
- Load rejects file with wrong permissions (if testable in the environment)
- `exists` returns false when no file, true after save

  **Verification:**
- `cargo test` passes for the new module
- `cargo clippy` clean

- [x] **Unit 2: Refactor `run_oauth2_flow` into composable parts**

  **Goal:** Extract URL generation and code-for-token exchange from `run_oauth2_flow` into reusable functions that both
  the interactive and remote flows can call.

  **Requirements:** R1, R2, R3

  **Dependencies:** None (pure refactor of existing code; independent of Unit 1)

  **Files:**
- Modify: `src/auth/oauth2.rs`

  **Approach:**
- Extract `build_auth_url(auth: &Auth, state: &str, challenge: &str) -> Result<String>` — builds the authorization URL
  with all query parameters
- Extract `exchange_code_for_token(auth: &mut Auth, code: &str, verifier: &str, username: &str) -> Result<String>` —
  handles the full post-exchange pipeline: POSTs to token endpoint, parses response, resolves username (if `username` is
  empty, calls `auth.fetch_username()` using the new access token — matching the existing convention in
  `run_oauth2_flow`), computes expiration, saves to store via `save_oauth2_token`, returns access_token. The name
  reflects the entry point (code exchange) though the function completes the full token lifecycle including persistence
- Rewrite `run_oauth2_flow` to call these two functions plus the existing callback server flow
- Make both extracted functions `pub(crate)` — they are internal helpers consumed by `run_oauth2_flow` and the remote
  flow functions. Integration tests should exercise the public `Auth` facade methods (`remote_oauth2_step1`,
  `remote_oauth2_step2`) rather than testing internal helpers directly

  **Patterns to follow:**
- Existing function signatures in `oauth2.rs` — `&Auth` for read-only, `&mut Auth` for store mutations
- Error handling via `XurlError::auth_with_cause()`

  **Test scenarios:**
- Existing tests still pass (refactor preserves behavior)
- `build_auth_url` produces a URL with all required OAuth2 query parameters (response_type, client_id, redirect_uri,
  scope, state, code_challenge, code_challenge_method)
- `exchange_code_for_token` handles missing `access_token` in response

  **Verification:**
- `cargo test` passes with no regressions
- `run_oauth2_flow` still works identically (same integration behavior)

- [x] **Unit 3: Add remote OAuth2 flow functions and integration tests**

  **Goal:** Implement `run_remote_step1` and `run_remote_step2` functions that compose the extracted helpers with pending
  state persistence, plus integration tests covering the round-trip flow.

  **Requirements:** R1, R2, R3, R4, R5, R7, R8

  **Dependencies:** Unit 1, Unit 2

  **Files:**
- Modify: `src/auth/oauth2.rs` (add `run_remote_step1`, `run_remote_step2`)
- Modify: `src/auth/mod.rs` (add facade methods on `Auth`)
- Create: `tests/auth_remote.rs` (integration tests for the remote flow)

  **Approach:**
- `run_remote_step1(auth: &Auth, pending_path: &Path) -> Result<String>`:
- If `pending::exists(pending_path)`, warn to stderr "Overwriting previous pending auth flow"
- Generate state + PKCE verifier/challenge
- Build auth URL via `build_auth_url`
- Save `PendingOAuth2State` via `pending::save()` (temp file + atomic rename — overwrites any existing file safely)
- Return the auth URL string
- `run_remote_step2(auth: &mut Auth, auth_url: &str, username: &str, pending_path: &Path) -> Result<String>`:
- Load pending state via `pending::load()` (enforces TTL, permission checks)
- Validate `pending.client_id` matches `auth.client_id()` — hard-fail with diagnostic error naming the expected app if
  they differ
- Parse `auth_url` to extract `code` and `state` query parameters
- Validate `state` matches pending state
- Call `exchange_code_for_token` with loaded verifier
- Only on success: delete pending file via `pending::delete()`. On any error (state mismatch, missing code, HTTP 400,
  username resolution failure), return error WITHOUT deleting the pending file — this allows the user to retry step 2
- Return access_token
- Add `Auth::remote_oauth2_step1(&self, pending_path: &Path) -> Result<String>` and `Auth::remote_oauth2_step2(&mut
  self, auth_url: &str, username: &str, pending_path: &Path) -> Result<String>` facade methods
- Add `Auth::app_name(&self) -> &str` accessor (needed to populate `PendingOAuth2State.app_name` and for diagnostic
  error messages on app mismatch; `app_name` field is currently private with no getter)
- Integration tests use `wiremock` to mock the token exchange and `/2/users/me` endpoints; test helper must override
  `Config::token_url` (not just `api_base_url`) to point at the wiremock server; use `tempfile` for both store and
  pending file isolation

  **Patterns to follow:**
- `run_oauth2_flow` function structure — state generation, URL building, token exchange, save
- `Auth` facade pattern — thin methods that delegate to module functions
- `tests/api_tests.rs` — wiremock setup, `create_test_config` pattern (extended with `token_url` override)

  **Test scenarios:**
- Happy path: step 1 → step 2 with valid mock responses → token saved, pending file deleted
- Step 1 creates pending file and returns a valid auth URL
- Step 1 when pending file already exists → warns and overwrites
- Step 2 with valid redirect URL completes token exchange (mock HTTP)
- Step 2 with state mismatch returns descriptive error
- Step 2 with missing code parameter returns descriptive error
- Step 2 without prior step 1 (no pending file) returns descriptive error
- Step 2 with mismatched client_id returns diagnostic error naming the expected app
- Step 2 with expired pending file (created_at older than 15 min) returns TTL error
- Step 2 with expired/revoked code (mock 400 response) → error, pending file preserved
- Pending file is NOT deleted after failed step 2 (allows retry)
- Token saved matches expected format (same as interactive flow)

  **Verification:**
- `cargo test auth_remote` passes
- No test isolation leaks (no writes to real `~/.xurl` or `~/.xurl.pending`)

- [x] **Unit 4: Add CLI flags and command dispatch**

  **Goal:** Add `--remote`, `--step`, and `--auth-url` flags to `AuthCommands::Oauth2` and wire up the command dispatch
  to call the remote flow functions.

  **Requirements:** R1, R2, R4, R5, R6

  **Dependencies:** Unit 3

  **Files:**
- Modify: `src/cli/mod.rs` (`AuthCommands::Oauth2` variant)
- Modify: `src/cli/commands/mod.rs` (`run_auth_command` match arm)

  **Approach:**
- Change `AuthCommands::Oauth2` from unit variant to struct variant with fields:
- `--remote` (bool) — enables manual two-step flow
- `--step` (Option<u8>) — `1` or `2`, requires `--remote`
- `--auth-url` (Option<String>) — redirect URL for step 2, requires `--step 2`. Accepts `-` to read from stdin
  (recommended for shared machines to avoid exposing the auth code in `/proc/*/cmdline`)
- Use clap `requires` attribute: `--step` requires `--remote`, `--auth-url` requires `--step`. Use clap `value_parser`
  with allowed values `1` and `2` to reject invalid step numbers at parse time. Validate `--step 2` requires
  `--auth-url` as a runtime check in `run_auth_command` (clap's `requires_if` is complex with derive — simpler to check
  in the match arm and return a descriptive error)
- In `run_auth_command`, match on the field combination:
- `remote: false` → existing `auth.oauth2_flow("")` path
- `remote: true, step: Some(1)` → call `auth.remote_oauth2_step1(pending_path)`, print URL and instructions
- `remote: true, step: Some(2)` → if `auth_url == "-"`, read one line from stdin; call
  `auth.remote_oauth2_step2(auth_url, "", pending_path)`, print success
- `remote: true, step: None` → error: `--remote requires --step 1 or --step 2`
- Invalid step values (e.g., `--step 3`) are rejected at parse time by clap `value_parser`
- Step 1 text output: instruction line + auth URL on its own line. JSON output: `{"auth_url": "...", "instructions":
  "..."}`
- Step 2 text output: success message matching existing `"OAuth2 authentication successful!"` pattern

  **Patterns to follow:**
- `AuthCommands::Oauth1` — struct variant with named fields and `#[arg(long)]` attributes
- `AuthCommands::Clear` — conditional logic based on flag combinations
- `out.print_message()` for text output, `out.print_response()` for structured JSON output

  **Test scenarios:**
- `xr auth oauth2` (no flags) still works as before
- `xr auth oauth2 --remote --step 1` prints auth URL
- `xr auth oauth2 --remote --step 2 --auth-url <url>` completes exchange
- `xr auth oauth2 --remote` without `--step` shows error
- `xr auth oauth2 --step 1` without `--remote` shows error (clap validation)
- `xr auth oauth2 --remote --step 3` shows error (clap value_parser rejection)
- `xr auth oauth2 --remote --step 2` without `--auth-url` shows descriptive runtime error
- `xr auth oauth2 --remote --step 2 --auth-url -` reads URL from stdin
- `xr auth oauth2 --help` includes `--remote`, `--step`, `--auth-url` with descriptions and stdin recommendation
- CLI flag validation tests use `assert_cmd` and `predicates` in `tests/auth_remote.rs` alongside Unit 3's library-level
  tests

  **Verification:**
- `cargo test` and `cargo clippy` pass
- `xr auth oauth2 --help` shows new flags with clear descriptions
- `xr auth oauth2` without new flags behaves identically to current behavior

## System-Wide Impact

- **Interaction graph:** The remote flow bypasses `callback.rs` entirely. No changes to the callback server, token
  refresh, or any other auth path. The `Auth` struct gains two new facade methods but existing methods are unchanged.
- **Error propagation:** New errors follow existing `XurlError::auth(msg)` convention. Exit code mapping is unchanged —
  auth errors already map to `EXIT_AUTH_REQUIRED` (2). New string-prefixed error messages follow the existing
  `XurlError::auth(msg)` convention: `PendingStateNotFound`, `PendingStateExpired`, `PendingStatePermissions`,
  `AppMismatch`, `StateMismatch`, `MissingCode`.
- **State lifecycle risks:**
- The `~/.xurl.pending` file is created in step 1 and deleted in step 2. If step 2 fails, the file persists for retry.
  If the user abandons the flow, the file is orphaned — the 15-minute TTL ensures stale files are rejected rather than
  silently consumed.
- Step 2 validates `pending.client_id` against the runtime `auth.client_id()` to catch mismatches from `--app` changes,
  env var drift, or store mutations between steps. This prevents confusing Twitter API errors and a potential
  confused-deputy condition where a token is stored under the wrong app.
- If step 1 is run again before step 2 completes, the previous pending state is overwritten with a warning to stderr.
  This matches the single-flow-at-a-time design of the reference implementation (gogcli). No file locking needed.
- **Procfs exposure:** The `--auth-url` flag value (containing the auth code) is visible in `/proc/*/cmdline` on Linux.
  Mitigated by offering `--auth-url -` (stdin) as the recommended input method and documenting it in help text. The auth
  code is single-use and short-lived (~10 minutes), limiting the exposure window.
- **API surface parity:** The library API (`Auth` struct) gains `remote_oauth2_step1` and `remote_oauth2_step2` methods.
  These are additive — no breaking changes.
- **Integration coverage:** Unit 3 covers the library-level round-trip flow from pending state through HTTP exchange to
  token store. Unit 4 includes CLI flag validation and error message tests.

## Risks & Dependencies

- **Twitter auth code expiration:** The authorization code from Twitter expires in ~10 minutes. If the user takes too
  long between step 1 and step 2, the exchange will fail with a clear error from Twitter's API. The 15-minute pending
  file TTL is intentionally slightly longer to avoid a race where the file expires before the code does.
- **PKCE verifier on disk:** Persisting the PKCE verifier to a file is an inherent security downgrade from holding it
  only in stack memory. Mitigated by: 0o600 permissions at creation via temp file + atomic rename (no TOCTOU window),
  permission and owner re-check before reading at step 2 (`#[cfg(unix)]`), 15-minute TTL to bound exposure, and deletion
  after use. Exploitation requires combining file read access with auth code interception — a two-factor attack.
- **Auth code in procfs:** The `--auth-url` flag value is visible in `/proc/*/cmdline` on shared machines. Mitigated by
  offering `--auth-url -` (stdin) as the recommended input method. The auth code is single-use and short-lived (~10
  minutes), and the exposure window is only the duration of the token exchange network call.
- **`open` crate on headless:** `open::that()` will fail on headless machines, which is fine — the current code already
  prints the URL to stdout as fallback. Remote mode skips `open::that()` entirely.
- **App/credential mismatch between steps:** If the user changes `--app`, env vars, or store credentials between step 1
  and step 2, the token exchange would fail with an opaque Twitter API error. Mitigated by validating
  `pending.client_id` against the runtime `auth.client_id()` at step 2 with a diagnostic error message.

## Sources & References

- Origin:
  [.context/compound-engineering/todos/001-ready-p2-remote-oauth-flow.md](../../.context/compound-engineering/todos/001-ready-p2-remote-oauth-flow.md)
- Reference implementation: `gogcli` `auth_add.go` (`gog auth add --remote`)
- Related: `docs/solutions/deployment-issues/headless-box-sync-rclone-bisync-systemd.md` (headless OAuth pattern)
- Related: `docs/solutions/security-issues/rust-cli-security-code-quality-audit.md` (token file permissions, PKCE
  testing gap)
