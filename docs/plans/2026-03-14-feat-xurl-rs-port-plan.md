---
title: "xurl-rs — Rust Port of xurl (X/Twitter API CLI)"
type: feat
status: completed
date: 2026-03-14
origin: docs/brainstorms/2026-03-14-rust-porting-skill-brainstorm.md
---

# xurl-rs — Rust Port of xurl

## Overview

Port `xurl` (Go CLI for authenticated X/Twitter API requests) to Rust as `xurl-rs`. This serves as the validation case
study for both the rust-porting-skill and the go2rust transpiler.

The port must achieve **Absolute Parity** — identical outputs for identical inputs — before any enhancements are
considered.

## Problem Statement

xurl is a useful Go CLI tool but:

- Go binaries are larger than Rust equivalents
- Startup time can be improved with Rust's zero-cost abstractions
- Static linking in Rust is simpler (single binary, no runtime)
- This port validates our porting framework end-to-end

More importantly: this is the **proving ground** for the rust-porting-skill. If the skill can't port xurl successfully,
  the skill needs fixing.

## Source Analysis (from research)

- **Repository:** github.com/xdevplatform/xurl
- **Language:** Go 1.24
- **Source LOC:** 4,733
- **Test LOC:** 2,175
- **Total:** ~6,900 LOC
- **Packages:** 8 (cli, api, auth, store, config, errors, utils, version)
- **Commands:** 28 shortcut commands + raw curl-style mode
- **Auth:** OAuth2 PKCE, OAuth1 HMAC-SHA1, Bearer token
- **Token Store:** YAML at ~/.xurl with legacy JSON + .twurlrc migration
- **Complexity:** Small-medium (no real concurrency, sequential API calls)
- **Strategy:** parallel-validation (2,175 LOC tests → red team has material)

## Proposed Solution

Use the rust-porting-skill workflow end-to-end:

1. Run go2rust on xurl source for mechanical scaffold
2. Follow the 7-phase skill workflow
3. Red team / green team parallel strategy
4. Differential conformance testing against live xurl
5. Dual benchmarking (vs xurl, vs curl)

### Target Architecture (Rust)

```text
xurl-rs/
  Cargo.toml
  src/
    main.rs                 # Entry point
    cli/
      mod.rs                # CLI definition (clap derive)
      commands/             # 28 shortcut commands
        post.rs
        reply.rs
        like.rs
        search.rs
        dm.rs
        ...
      raw.rs                # Raw curl-style mode
    api/
      mod.rs                # X API client
      request.rs            # Request building
      response.rs           # Response handling
    auth/
      mod.rs                # Auth orchestration
      oauth2.rs             # OAuth2 PKCE flow
      oauth1.rs             # OAuth1 HMAC-SHA1
      bearer.rs             # Bearer token auth
      callback.rs           # OAuth callback server
    store/
      mod.rs                # Token store
      yaml_store.rs         # YAML persistence
      migration.rs          # Legacy JSON + .twurlrc migration
    config/
      mod.rs                # Configuration handling
    error.rs                # Error types (thiserror)
    utils.rs                # Utility functions
  tests/
    cli_tests.rs            # CLI integration tests (assert_cmd)
    api_tests.rs            # API client tests
    auth_tests.rs           # Auth flow tests
    store_tests.rs          # Token store tests
    conformance/
      mod.rs                # Differential conformance tests
      fixtures/             # Recorded API responses
```

### Crate Selections (justified)

| Crate | Purpose | Rationale |
|-------|---------|-----------|
| `clap` (derive) | CLI parsing | Standard, maps from cobra |
| `reqwest` | HTTP client | Standard, maps from net/http |
| `serde` + `serde_yaml` | Token store | YAML persistence parity |
| `serde_json` | JSON handling | API responses |
| `anyhow` | Error handling | Application-level errors |
| `thiserror` | Error types | Typed errors for auth, API |
| `tokio` | Async runtime | OAuth callback server only |
| `oauth2` | OAuth2 PKCE | Standard crate, proven |
| `hmac` + `sha1` | OAuth1 signing | HMAC-SHA1 for OAuth1 |
| `base64` | OAuth1 encoding | Signature encoding |
| `url` | URL handling | Query parameter construction |
| `dirs` | Home directory | ~/.xurl path resolution |
| `assert_cmd` | CLI testing | Integration test standard |
| `predicates` | Test assertions | Output matching |
| `wiremock` | Mock HTTP | API response mocking for tests |

## Implementation Phases

### Phase 1: Skill-Driven Analysis (30 min)

- [ ] Run rust-porting-skill Phase 1 (ANALYZE) on xurl
- [ ] Generate EXISTING_STRUCTURE.md
- [ ] Run go2rust on xurl source for scaffold
- [ ] Review scaffold output, note translation gaps

### Phase 2: Architecture + Spec (1 hour)

- [ ] Run skill Phase 2 (ARCHITECT) — generate PROPOSED_ARCHITECTURE.md
- [ ] Run skill Phase 3 (SPECIFY) — generate BEHAVIORAL_SPEC.md
- [ ] Catalog all 28 commands with their flags and expected outputs
- [ ] Run skill Phase 4 (PLAN) — generate PORT_PLAN.md

### Phase 3: Red Team — Test Porting (1.5 hours, parallel with Phase 4)

- [ ] Port xurl's 2,175 LOC of Go tests to Rust
- [ ] Build differential conformance harness
- [ ] Create fixture recordings for API-dependent tests
- [ ] Write edge case tests not in original
- [ ] Set up benchmark suite (hyperfine config)

### Phase 4: Green Team — Implementation (2 hours, parallel with Phase 3)

- [ ] Start from go2rust scaffold + BEHAVIORAL_SPEC.md
- [ ] Implement token store (YAML read/write, migration)
- [ ] Implement auth flows (OAuth2 PKCE, OAuth1, Bearer)
- [ ] Implement API client (request building, response handling)
- [ ] Implement CLI layer (clap derive, 28 commands)
- [ ] Implement raw curl-style mode

### Phase 5: Merge + Validation (1 hour)

- [ ] Run red team's test suite against green team's code
- [ ] Fix failures (determine: test bug or implementation bug)
- [ ] Run differential conformance tests against live xurl
- [ ] All tests must pass before proceeding

### Phase 6: Benchmarking (30 min)

- [ ] Benchmark vs xurl: startup time, request throughput
- [ ] Benchmark vs curl: raw request performance
- [ ] Generate benchmark report with tables
- [ ] Document binary size comparison

### Phase 7: Polish (30 min)

- [ ] README with usage, installation, benchmarks
- [ ] --help text matches xurl's help text
- [ ] CI configuration (GitHub Actions)
- [ ] clippy clean, rustfmt'd, edition 2024
- [ ] Human review package ready

## Acceptance Criteria

### Parity (hard gates)

- [ ] All 28 shortcut commands produce identical output to xurl
- [ ] Raw curl-style mode produces identical output
- [ ] Exit codes match for all commands (success and error cases)
- [ ] Token store format is compatible (read xurl's YAML, write same format)
- [ ] Legacy migration works identically (JSON + .twurlrc)
- [ ] --help output matches xurl's help output

### Quality

- [ ] All original Go tests pass when ported to Rust
- [ ] Differential conformance tests pass
- [ ] Clippy clean (deny warnings)
- [ ] Edition 2024
- [ ] No unwrap() in production code
- [ ] Documentation on all public items

### Performance

- [ ] Startup time faster than xurl (expected: significantly)
- [ ] Request throughput equal or better
- [ ] Binary size documented (expected: smaller with static linking)
- [ ] Benchmarks reproducible via `scripts/benchmark.sh`

## Dependencies

- **rust-porting-skill** — must be complete enough to drive Phases 1-4
- **go2rust** — must handle xurl's patterns for scaffold generation
- xurl source at `~/github-stars/xdevplatform/xurl/`
- xurl binary installed for differential testing
- X API credentials for live testing (Brett has these configured)
- Go toolchain for running original xurl

## Risk Analysis

| Risk | Impact | Mitigation |
|------|--------|------------|
| OAuth2 PKCE flow complex in Rust | Medium | oauth2 crate handles most complexity |
| OAuth1 signing exact-byte-parity | High | Extensive test fixtures, byte-level comparison |
| API rate limits during conformance testing | Medium | Use recorded fixtures for core tests, live as bonus |
| Legacy .twurlrc format undocumented | Low | Reverse-engineer from xurl source |
| go2rust scaffold too incomplete | Medium | Fall back to spec-only implementation |

## Sources

- **xurl source analysis:** `~/obsidian-vault/OpenClaw/research/rust-porting/xurl-case-study/source-analysis.md`
- **Porting assessment:** `~/obsidian-vault/OpenClaw/research/rust-porting/xurl-case-study/porting-assessment.md`
- **Crate mapping:** `~/obsidian-vault/OpenClaw/research/rust-porting/xurl-case-study/crate-mapping.md`
- **Test inventory:** `~/obsidian-vault/OpenClaw/research/rust-porting/xurl-case-study/test-inventory.md`
- **Brainstorm:** `~/obsidian-vault/OpenClaw/docs/brainstorms/2026-03-14-rust-porting-skill-brainstorm.md`
