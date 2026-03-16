---
title: "xurl-rs: Production-grade Rust Port of the xurl X/Twitter API CLI"
problem_type: rust-port
component: xurl-rs
symptoms:
  - Go xurl lacked agentic-friendly flags (--output json, --quiet, --no-interactive, structured exit codes)
  - No shell autocomplete in the Go original
  - Type safety and Rust idioms preferred for long-term maintainability of API client code
  - Single-binary distribution with no runtime dependencies desired
tags:
  - rust
  - cli
  - twitter
  - x-api
  - oauth
  - clap
  - port
  - api-client
  - testing
  - shell-completion
  - agentic-flags
difficulty: expert
time_spent_hours: 14
date: 2026-03-16
---

# xurl-rs: Production-grade Rust Port of the xurl X/Twitter API CLI

## Problem Summary

The existing Go `xurl` CLI provided full X/Twitter API access (OAuth1/2, Bearer auth, media upload, streaming, DMs, social graph) but lacked structured output modes needed for agent-driven automation (`--output json`, `--quiet`, `--no-interactive`, UNIX exit codes). A full Rust port (`xurl-rs`, binary `xr`) was built with 100% feature parity across all 60+ commands, adding RFC 5849–verified OAuth1 signatures, chunked media upload, shell autocomplete for 5 shells, and publishing artifacts (crates.io metadata, Homebrew formula, release guide). The port followed a red-team/green-team methodology with 5 test phases including differential conformance testing against the Go original.

---

## Motivation

- Go `xurl` had no `--output json`, no structured exit codes, no `--no-interactive` — unusable in agent pipelines without scraping human-readable output
- Rust gives memory safety, a single static binary, type-safe API contracts, and `cargo install` distribution
- The Go binary's test coverage was good but its architecture made agentic extension awkward

---

## Key Architectural Decisions

### Output System (OutputConfig)

Carries output format preferences through the call stack as a first-class struct. Avoids global state; makes output behavior testable.

```rust
pub struct OutputConfig {
    pub format: OutputFormat,  // Text | Json | Jsonl
    pub quiet: bool,
    pub no_color: bool,        // respects NO_COLOR env var
}

impl OutputConfig {
    pub fn print_response(&self, value: &serde_json::Value) { ... }
    pub fn print_error(&self, e: &XurlError) { ... }  // JSON to stderr if --output json
    pub fn info(&self, msg: &str) { ... }              // suppressed by --quiet
}
```

Every command handler receives `&OutputConfig` and calls `out.print_response()` — no naked `println!` in command code.

### Structured Exit Codes

```rust
// src/cli/exit_codes.rs
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_ERROR: i32 = 1;
pub const EXIT_AUTH_REQUIRED: i32 = 2;
pub const EXIT_RATE_LIMITED: i32 = 3;
pub const EXIT_NOT_FOUND: i32 = 4;
pub const EXIT_NETWORK_ERROR: i32 = 5;
```

Top-level `main` maps error variants to codes:

```rust
fn exit_code_for_error(e: &XurlError) -> i32 {
    match e {
        XurlError::Auth(_) => EXIT_AUTH_REQUIRED,
        XurlError::Http(msg) if msg.contains("429") => EXIT_RATE_LIMITED,
        XurlError::Http(msg) if msg.contains("404") => EXIT_NOT_FOUND,
        XurlError::Io(_) => EXIT_NETWORK_ERROR,
        _ => EXIT_ERROR,
    }
}
```

### Agentic Flags

Four flags every agent-targeted CLI should have:

```rust
#[arg(long, global = true, default_value = "text", value_enum)]
pub output: OutputFormat,   // text | json | jsonl

#[arg(long, short = 'q', global = true)]
pub quiet: bool,            // suppress progress/info output

#[arg(long, global = true)]
pub no_interactive: bool,   // error instead of prompting

#[arg(long, global = true, default_value = "30")]
pub timeout: u64,           // request timeout in seconds
```

Also: `NO_COLOR` env var support, `XURL_OUTPUT` env var for default format.

### OAuth1 Signature with Injectable Nonce/Timestamp

To enable deterministic testing without live API calls:

```rust
pub fn build_oauth1_header(method, url, token, params) -> Result<String> {
    build_oauth1_header_with_nonce_ts(method, url, token, params, None, None)
}

// Testable variant — inject fixed nonce + timestamp
pub fn build_oauth1_header_with_nonce_ts(
    ..., fixed_nonce: Option<&str>, fixed_timestamp: Option<&str>
) -> Result<String> { ... }
```

Verified against RFC 5849 Section 3.4 test vectors.

### Wiremock for API Tests

All API layer tests use wiremock mock servers — no live network, no flakiness:

```rust
#[tokio::test]
async fn test_create_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/tweets"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "data": {"id": "123", "text": "Hello world!"}
        })))
        .mount(&server).await;

    let mut client = ApiClient::new_with_base_url(&server.uri(), auth);
    let result = create_post(&mut client, "Hello world!", &[], &opts).unwrap();
    assert_eq!(result["data"]["id"], "123");
}
```

---

## Build Phases

| Phase | What It Did |
|---|---|
| 1 | go2rust transpiler output — 14 errors from 983 |
| 2 | Red team: ported test stubs from Go tests |
| 3 | Green team: implementation to pass tests |
| 4 | Phase 7 polish (ran out of order — see lessons) |
| 5 | Phase 6 differential conformance against Go binary |
| 6 | Mock-server API tests (23 tests ported from Go httptest) |
| 7 | OAuth1 RFC 5849 vector verification |
| 8 | Shell autocomplete (bash/zsh/fish/powershell/elvish) |
| 9 | Agentic flags fully wired (--output/--quiet/--no-interactive/exit codes) |
| 10 | Publishing prep (crates.io metadata, Homebrew formula, README, RELEASING.md) |

---

## Metrics

| Metric | Before | After |
|---|---|---|
| Clippy warnings | 179 | 0 |
| Tests | 137 | 253 |
| Compile errors | 0 | 0 |
| `cargo publish --dry-run` | ❌ missing metadata | ✅ clean |
| Shell autocomplete | ❌ | ✅ 5 shells |
| Agentic flags | ❌ | ✅ fully wired |
| Differential conformance | ❌ not run | ✅ 34/37 pass (3 need live creds) |

---

## Lessons Learned (What Went Wrong)

### 1. Phase ordering was violated — polish ran before differential testing

**What happened:** Phase 7 (clippy polish) ran before Phase 6 (differential validation). Code was polished before being proven correct.

**Fix:** Phase ordering is a hard gate. Differential testing is a **correctness gate**. Polish is an **aesthetics gate**. Correctness always comes first. Add `<!-- GATE: Phase N must be ✅ before starting N+1 -->` to plan docs.

---

### 2. Commented-out tests were accepted

**What happened:** `tests/api_tests.rs` had all API tests commented out with TODO comments. This looked like coverage but was silence.

**Fix:** Zero tolerance. Commented-out tests are an automatic PR reject. Use `todo!()` macro if implementation is deferred — it compiles but panics visibly. Never hide gaps behind comments.

---

### 3. Naming was decided late

**What happened:** Binary renamed `xurl` → `xr`, crate renamed `xurl` → `xurl-rs` mid-project. Broke all integration test `use xurl::` imports.

**Fix:** Naming is a Day 0 decision. Checklist before `cargo new`:
- [ ] Crate name available on crates.io?
- [ ] Binary name conflicts with existing system tools?
- [ ] Lib name (snake_case, no hyphens) distinct from binary?
- [ ] All integration test imports written against the final lib name?

---

### 4. `Option<&TokenStore>` that always returned `Some`

**What happened:** `token_store()` returned `Option<&TokenStore>` but could never return `None` — API was dishonest, callers handled an impossible case.

**Fix:** If a function cannot return `None`/`Err` in practice, don't make callers handle it. Honesty rule: return type must reflect real failure modes only.

---

### 5. Subagents used for coding work instead of Claude Code

**What happened:** Implementation phases used `sessions_spawn` subagents instead of `claude --permission-mode bypassPermissions --print`. Subagents lack file-editing context that Claude Code has natively.

**Fix:** 
- **Subagents** → research, analysis, reading, synthesis
- **Claude Code (`ce:work`)** → any file creation, editing, building

---

## Reusable Patterns for Future Rust CLIs

Copy these into any new Rust CLI project:

1. **`OutputConfig` struct** — thread output preferences through the stack, never use naked `println!` in command handlers
2. **`exit_codes.rs` module** — consistent numeric exit codes with match-based mapping from error types
3. **Agentic flags** (`--output`, `--quiet`, `--no-interactive`, `--timeout`, `NO_COLOR`, `XURL_OUTPUT`) — every agent-targeted CLI needs these four
4. **Injectable nonce/timestamp** on OAuth1 functions — enables deterministic testing without live API
5. **Wiremock fixture pattern** — `MockServer::start().await` + `Mock::given()` for hermetic API tests

---

## Prevention Checklist for Future Rust Ports

### Day 0 (Before Writing Code)
- [ ] Lock crate name, binary name, lib name — add "DO NOT CHANGE" comment to Cargo.toml
- [ ] Check crates.io: `cargo search <name>`
- [ ] Check PATH conflicts: `which <binary-name>`
- [ ] Configure clippy to deny warnings from commit 1: `[lints.clippy] all = "deny"`
- [ ] Plan phases with explicit GATE markers — differential testing before polish

### Every PR
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test count did not decrease without explanation
- [ ] No commented-out `#[test]` blocks
- [ ] All `Option`/`Result` return types reflect real failure modes

### Phase 6 (Differential Testing) — Must Complete Before Phase 7
- [ ] Golden outputs captured from reference implementation
- [ ] Differential harness running against real reference binary
- [ ] All commands produce equivalent output
- [ ] Phase 6 marked ✅ in plan doc before Phase 7 begins

---

## Related

- [`FEATURE_PARITY.md`](../../FEATURE_PARITY.md) — full Go vs Rust feature matrix
- [`KNOWN_DIFFERENCES.md`](../../KNOWN_DIFFERENCES.md) — intentional divergences from Go original
- [`RELEASING.md`](../../RELEASING.md) — crates.io + Homebrew release procedure
