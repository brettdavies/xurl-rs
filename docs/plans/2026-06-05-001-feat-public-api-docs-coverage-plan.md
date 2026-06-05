---
title: "feat: Document public API and enforce missing_docs lint"
type: feat
status: active
date: 2026-06-05
origin: docs/brainstorms/2026-06-05-public-api-docs-coverage-requirements.md
---

# feat: Document public API and enforce missing_docs lint

## Summary

Document every `pub` item exposed under `xurl::*`, fix the four mis-targeted module-doc bugs, add `no_run` usage
examples on a small set of entry-point types, and turn on `#![deny(missing_docs)]` on `src/lib.rs` so the documentation
contract is enforced at compile time. Build-script-included files get a localized `#[allow(missing_docs)]` shim. No live
doctests, no private-item docs.

---

## Problem Frame

`xurl-rs` ships both a binary (`xr`) and a consumable library (`xurl::*`). The library half is published on docs.rs at
`https://docs.rs/xurl-rs/2.0.0/xurl/`. Today the landing page reads as sparse — the module list has no descriptions,
item-level prose is missing across the most public-facing types (`src/api/response/types.rs` at 18.8%,
`src/store/types.rs` at 17.9%, `src/cli/exit_codes.rs` at 0%), and four prominent module roots (`src/api/mod.rs`,
`src/auth/mod.rs`, `src/cli/mod.rs`, `src/store/mod.rs`) have `///` outer-doc comments placed immediately above a `pub
mod foo;` declaration. Rustdoc attaches those docs to the next item, not to the enclosing module, so the four modules
render as undocumented on the landing page despite carrying header prose in the source.

Baseline coverage today is 66.8% (480 of 720 items per `cargo +nightly doc --show-coverage`). No lint currently guards
the contract, so a one-shot pass would start drifting back the moment another `pub` item lands without a doc.

The work is library-consumer-facing — the next downstream Rust crate that reads docs.rs to decide whether to depend on
`xurl` is the audience, and the contract this plan installs is what keeps that audience well-served going forward.

---

## Requirements

Carried forward from the origin brainstorm. R-IDs match the origin document; the plan references them in unit
`Requirements` fields below.

### Documentation completeness on the public surface

- R1. `src/lib.rs` begins with a `//!` crate-level doc orienting a docs.rs reader on the `xurl` library / `xr` binary
  split, the auth-method landscape (OAuth1, OAuth2 PKCE, Bearer), and the entry-point types worth reading first.
- R2. Every public module (`pub mod foo`) carries a `//!` inner-doc header at the top of its `mod.rs` (or single-file
  module) that names what the module is responsible for and its place in the crate.
- R3. Every `pub` item exposed from `xurl::*` — functions, structs, enums, type aliases, consts, traits — carries a
  `///` outer-doc with at least one sentence describing its purpose. `pub` struct fields carry per-field docs; enum
  variants carry per-variant docs.
- R4. The four mis-targeted module docs in `src/api/mod.rs`, `src/auth/mod.rs`, `src/cli/mod.rs`, and `src/store/mod.rs`
  are converted from `///` immediately above `pub mod ...;` to `//!` at the top of the file.

### Lint-enforced floor

- R5. `#![deny(missing_docs)]` is set on `src/lib.rs` and rejects any undocumented `pub` item at compile time.
- R6. `src/main.rs` is not touched by the lint.
- R7. Build-script-emitted files included via `include!()` from `OUT_DIR` (`auth_matrix.rs`, `generated_hosts.rs`) are
  scoped under a localized `#[allow(missing_docs)]` at the include site.
- R8. R5 and R7 land in the same change as R1–R4, or in a tightly-following PR within the same release cycle. The deny
  lint fails the build until every `pub` item is documented and the build-script include sites carry the allow shim, so
  a standalone "turn on the lint" commit is not viable.

### Compile-checked usage examples on entry points

- R9. ~6–8 entry-point `pub` types receive usage examples in their doc comments. The candidate set covers the request
  client, output configuration, token store, the error envelope, and a representative subset of `shortcuts::*`.
- R10. Every example block uses `no_run` — it compiles against the public API during `cargo test --doc` but does not
  execute. No example block executes against a live network endpoint or depends on credentials.

---

## Key Technical Decisions

- **`//!` inner-doc form for module roots, not `///` on the parent declaration.** Standardizing on inner-doc places
  module descriptions inside the file each documents, eliminating the four current mis-targeting bugs and giving a
  single uniform convention across the crate. The alternative — relocating `///` blocks to `src/lib.rs` immediately
  above `pub mod foo;` — would produce equivalent docs.rs output but split conventions across files (see origin:
  `docs/brainstorms/2026-06-05-public-api-docs-coverage-requirements.md`, Key Decisions).

- **`#![deny(missing_docs)]` placed at crate level on `src/lib.rs`, not per-module.** Crate-level placement applies to
  every public item recursively; per-module annotations would fragment the contract and create gaps when new modules
  land. Binary entry point at `src/main.rs` is left untouched per R6.

- **`#[allow(missing_docs)]` shims are narrow.** The allow attribute is placed at each `include!()` call site (or on a
  wrapping inline module scoped to the include), never at crate level. A crate-level allow would defeat the deny lint's
  purpose. Narrow shims keep the human-written surface fully under the deny floor while letting build-script-generated
  code bypass it.

- **Coverage measured as a one-off audit, not a gated CI step.** The deny lint enforces "no undocumented pub item" at
  compile time, which is the authoritative contract. `cargo +nightly doc --show-coverage` runs once during U5
  verification on the implementer's machine to confirm the human-written surface reports near-100% (intentional
  residuals only in the allowed OUT_DIR-included rows). Wiring nightly into CI was rejected for toolchain-friction
  reasons (project pins stable 1.94.1 via `rust-toolchain.toml`).

- **Examples are `no_run` and scoped to entry-point types only.** Per-API examples across every public item would carry
  disproportionate maintenance cost. Examples on the request client, output config, token store, error envelope, and a
  representative shortcut sample give downstream consumers the on-ramps they actually need. `no_run` participates in
  `cargo test --doc` (catches API breakage in docs) without consuming network egress.

- **Doc-prose style mirrors `src/error.rs`.** Short imperative first sentence; optional follow-up paragraph only when
  needed for non-obvious behavior; per-field and per-variant docs at one short line each. No separate style guide is
  written — the existing well-documented modules are the reference (`src/error.rs`, `src/api/shortcuts.rs` at 97.3%,
  `src/cli/mod.rs` at 85.0%).

---

## Implementation Units

### U1. Crate, module, and corrected-header docs

**Goal:** Make the docs.rs landing for `xurl::*` show a complete crate overview and a one-line description for every
public module.

**Requirements:** R1, R2, R4.

**Dependencies:** none.

**Files:**

- `src/lib.rs` — add crate-level `//!` doc (R1). The `#![deny(missing_docs)]` attribute is NOT added here yet — U5 owns
  that flip.
- `src/api/mod.rs` — convert `/// ...` above `pub mod auth_matrix;` to `//!` at the top of the file (R4).
- `src/auth/mod.rs` — same conversion (R4).
- `src/cli/mod.rs` — same conversion (R4).
- `src/store/mod.rs` — same conversion (R4).
- `src/config/mod.rs` — add a `//!` header (currently no module-level doc) (R2).
- `src/api/response/mod.rs` — add a `//!` header (currently 0% documented) (R2).
- Any remaining `pub mod` without a `//!` header at the top of its `mod.rs`. Audit during this unit by walking
  `src/{api,auth,cli,config,store,skill_install}/mod.rs` and any single-file public modules.

**Approach:** At the top of each module's `mod.rs`, add `//!` inner-doc prose summarizing what the module is responsible
for and its place in the crate. The crate-level `//!` on `src/lib.rs` orients a docs.rs reader on the library/binary
split, names the auth landscape (OAuth1 / OAuth2 PKCE / Bearer), and points at the entry-point types covered by examples
in U4. Keep prose tight — module headers are 1–3 short paragraphs, not a full specification of behavior.

**Execution note:** none — documentation-only change.

**Patterns to follow:** `src/skill_install/mod.rs` lines 1–5 (the only existing module root that uses `//!` correctly;
pipeline-summary structure). For prose tone, mirror `src/error.rs` lines 1–14.

**Test scenarios:** Test expectation: none — documentation-only change; verified via the `cargo doc` and coverage runs
in this unit's Verification.

**Verification:**

- `cargo doc --no-deps` succeeds under `RUSTDOCFLAGS="-D warnings"` (the pre-push hook's existing rustdoc invocation) —
  catches broken intra-doc links introduced by the new prose.
- `cargo +nightly doc --no-deps --lib -Z unstable-options --show-coverage` reports nonzero file-level coverage for
  `src/api/mod.rs`, `src/auth/mod.rs`, `src/cli/mod.rs`, `src/store/mod.rs`, `src/config/mod.rs`,
  `src/api/response/mod.rs`, and `src/lib.rs` (each previously at 0%).
- Visual spot-check of `target/doc/xurl/index.html` and each affected module page confirms the `//!` summary renders.

---

### U2. Document public items in the `api/` subtree

**Goal:** Every `pub` item exposed under `xurl::api::*` carries item-level doc prose.

**Requirements:** R3.

**Dependencies:** U1 (module headers establish the structure).

**Files:**

- `src/api/request.rs` — currently 41.2%; the largest gap in this subtree.
- `src/api/response/types.rs` — currently 18.8%; the typed-response surface that downstream consumers pattern-match on.
- `src/api/media.rs` — currently 85.7%; fill remaining gaps.
- `src/api/shortcuts.rs` — currently 97.3%; fill remaining gaps.
- `src/api/endpoints.rs`, `src/api/response/format.rs`, `src/api/auth_matrix.rs` — all currently 100% for human-written
  items; verify no regression and skip if already complete. The `OUT_DIR/auth_matrix.rs` and
  `OUT_DIR/generated_hosts.rs` rows are intentionally addressed in U5 via shims, not here.

**Approach:** Walk each file. For each undocumented `pub` fn/struct/enum/type-alias/const/trait, write a `///` doc
comment with at least one short imperative sentence describing what the item does and, where non-obvious, why a caller
would use it. Add per-field docs to public struct fields and per-variant docs to enum variants. Keep prose focused on
WHAT and WHY at the level a docs.rs reader needs — implementation detail stays in code comments per the project's
code-comments policy.

**Execution note:** none — documentation-only change.

**Patterns to follow:** `src/error.rs` (per-variant, per-field doc style); `src/api/shortcuts.rs` (function-level docs
with short imperative summaries).

**Test scenarios:** Test expectation: none — documentation-only change; coverage verified by U5's nightly coverage audit
and the deny-lint smoke test.

**Verification:** `cargo +nightly doc --no-deps --lib -Z unstable-options --show-coverage` reports 100% for
`src/api/request.rs`, `src/api/response/types.rs`, `src/api/media.rs`, `src/api/shortcuts.rs`. `cargo doc --no-deps`
continues to succeed under `RUSTDOCFLAGS="-D warnings"`.

---

### U3. Document public items in remaining modules

**Goal:** Every `pub` item across the rest of the public surface carries item-level doc prose.

**Requirements:** R3.

**Dependencies:** U1 (module headers established). Independent of U2 — the two units could in principle land in either
order, but typically ship together.

**Files (priority-ordered by current gap size):**

- `src/cli/exit_codes.rs` — currently 0%. Contents are `pub use crate::error::{...}` re-exports only;
  `#![deny(missing_docs)]` does NOT fire on `pub use` re-exports (the lint inherits the source item's doc state), so the
  0% report is a coverage-tool quirk. No action required here beyond verifying the canonical `pub const EXIT_*: i32`
  definitions in `src/error.rs` are documented (see this unit's Approach).
- `src/store/types.rs` — currently 17.9%.
- `src/skill_install/mod.rs` — currently 46.5% (note: the file already uses `//!` correctly; the gap is item-level docs
  on items it exposes).
- `src/auth/pending.rs` — currently 54.5%.
- `src/auth/mod.rs` — currently 65.2% (file-body items beyond the `//!` header, which U1 already converted).
- `src/cli/runner.rs` — currently 75.0%.
- `src/output.rs` — currently 80.6%.
- `src/auth/oauth1.rs` — currently 83.3%.
- `src/cli/mod.rs` — currently 85.0%.
- `src/store/mod.rs` — currently 85.2%.
- `src/auth/oauth2.rs` — currently 85.7%.
- `src/error.rs` — currently 89.1%; fill any remaining gaps.
- `src/envelope.rs` — currently 92.3%; fill any remaining gaps.
- `src/config/mod.rs` — currently 93.8%; fill any remaining gaps.
- `src/auth/callback.rs`, `src/cli/commands/*.rs` — all near 100%; verify no regression.

**Approach:** Same as U2 — walk each file, add docs to undocumented `pub` items, mirror the `src/error.rs` and
`src/api/shortcuts.rs` prose style. Priority order above is by current gap size; ultimately every `pub` item gets
documented before this unit closes.

**Execution note:** none — documentation-only change.

**Patterns to follow:** `src/error.rs`, `src/api/shortcuts.rs`. The canonical `pub const EXIT_*: i32` exit-code
definitions live in `src/error.rs` (already in this unit's Files list); `src/cli/exit_codes.rs` only re-exports them via
`pub use`. Document the constants at the source in `src/error.rs`; mirror the comment style in the prior security-audit
solution (`docs/solutions/security-issues/rust-cli-security-code-quality-audit.md` documents exit codes as public API at
78/77/1).

**Test scenarios:** Test expectation: none — documentation-only change.

**Verification:** `cargo +nightly doc --no-deps --lib -Z unstable-options --show-coverage` reports 100% for every file
in the list above. `cargo doc --no-deps` continues to succeed under `RUSTDOCFLAGS="-D warnings"`.

---

### U4. `no_run` usage examples on entry-point types

**Goal:** Downstream `xurl` consumers can copy a starter example for each entry-point type directly off docs.rs; the
examples compile against the public API at test time, catching shape drift before it ships.

**Requirements:** R9, R10.

**Dependencies:** U2, U3 (item docs land first so examples augment existing prose, not replace it).

**Files:**

- `src/api/request.rs` — example on the request client (build a `Request`, dispatch).
- `src/output.rs` — example on `OutputConfig` (text / json / jsonl selection).
- `src/store/mod.rs` — example on `TokenStore` (open the store at a custom path, read the active app).
- `src/error.rs` — example on `XurlError` (match on a representative variant set).
- `src/api/shortcuts.rs` — examples on `get_me` (read endpoint), `create_post` (write endpoint), and `search_posts`
  (search endpoint), giving downstream consumers a representative cross-section.

**Approach:** For each entry-point type, append a fenced ` ```rust,no_run ` block to its existing `///` doc comment.
Each example: (1) imports the type via fully-qualified path, (2) shows minimal construction, (3) shows one canonical
call. Use rustdoc's `#` line prefix to hide setup boilerplate (env construction, auth wiring) from the rendered docs
while keeping the example compilable. Examples are minimal — they are on-ramps, not specifications.

**Execution note:** none — `no_run` blocks compile against the public API but do not execute, so no special runtime
posture applies.

**Patterns to follow:** Standard rustdoc convention for `no_run` blocks with hidden setup lines (`# use ...; # let
client = ...;`). Any well-documented crate on docs.rs (e.g., `serde`, `reqwest`) demonstrates the pattern.

**Test scenarios:**

- Happy path: `cargo test --doc` compiles every example block in the entry-point set; each `no_run` block parses as
  valid Rust and resolves against the public API.
- Edge: a doc example that references a renamed parameter or removed type causes `cargo test --doc` to fail at compile
  time with an unresolved-import or type-mismatch error, surfacing API drift in docs at test time.
- Integration: the pre-push hook's `cargo test --quiet` already invokes doc tests by default; no hook change required.
  U4 examples participate in every pre-push and CI run going forward.

**Verification:** `cargo test --doc` passes locally with the new examples. The pre-push hook's `cargo test --quiet`
continues to pass. Visual spot-check of `target/doc/xurl/api/struct.*.html` etc. confirms the example blocks render with
the `no_run` banner.

---

### U5. Build-script include shim, lint flip, and verification

**Goal:** `#![deny(missing_docs)]` on `src/lib.rs` rejects any undocumented `pub` item at compile time. The build-script
include sites remain compilable. The public surface is verifiably documented end-to-end.

**Requirements:** R5, R6, R7, R8.

**Dependencies:** U1, U2, U3, U4. The deny flip cannot land before every human-written `pub` item is documented and
every `include!()` site carries the allow shim.

**Files:**

- `src/lib.rs` — add `#![deny(missing_docs)]` near the top, alongside the existing
  `#![allow(clippy::result_large_err)]`.
- `src/api/auth_matrix.rs` — the file already wraps the `include!()` in a `mod generated { use super::AuthScheme;
  include!(...); }` block carrying `#[allow(clippy::all)]`. Add `#[allow(missing_docs)]` to that same `mod generated`
  block (alongside the existing clippy allow). The existing `pub use generated::{AUTH_MATRIX, SHORTCUT_TEMPLATES};`
  re-exports preserve the public paths; no structural change beyond the new attribute.
- `src/skill_install/mod.rs` — the `include!()` currently sits at bare module scope with no wrapper. Introduce a new
  inline `mod generated_hosts { use super::*; include!(concat!(env!("OUT_DIR"), "/generated_hosts.rs")); }` carrying
  `#[allow(missing_docs)]`, then add `pub use generated_hosts::{SkillHost, KNOWN_HOSTS, resolve_host,
  host_envelope_str};` immediately below it to preserve the existing public paths. External consumer to preserve:
  `src/cli/mod.rs:15` (`pub use crate::skill_install::SkillHost;`). In-file references to the four items (callers within
  `src/skill_install/mod.rs`) must continue to resolve unqualified — they will via the `pub use` re-export at module
  scope.
- `src/main.rs` — no change (R6 explicitly leaves it untouched).
- `scripts/hooks/pre-push` — no change. The hook's existing `RUSTFLAGS="-Dwarnings" cargo clippy` and `cargo test
  --quiet` invocations automatically surface any `missing_docs` violation as a build failure once the deny is on.

**Approach:** The `#[allow(missing_docs)]` shim is attribute-scoped to a wrapping inline `mod { include!(...); }` block,
never to the `include!()` call directly. Rustc does not propagate outer attributes through `include!()` macro expansion
onto items synthesized by the include, so `#[allow(missing_docs)]\ninclude!(...)` fails to silence the lint — `cargo
build` would report `error: missing documentation` on items inside `OUT_DIR/auth_matrix.rs` /
`OUT_DIR/generated_hosts.rs`. The only working shape is `#[allow(missing_docs)] mod foo { include!(...); }` paired with
`pub use foo::{...};` re-exports at the parent scope to preserve every public path that was reachable before the
wrapping. `src/api/auth_matrix.rs` already has this layout (the existing `mod generated` wrapper is the model); the work
in `src/skill_install/mod.rs` is to introduce the wrapper and the re-exports per the Files section above. Verify scoping
by running `cargo +nightly doc --show-coverage` and confirming the `OUT_DIR/auth_matrix.rs` and
`OUT_DIR/generated_hosts.rs` rows no longer gate the human-written surface's coverage, and that every existing external
import path (e.g., `xurl::skill_install::SkillHost`) still resolves.

The deny flip and the shims must land together in a single commit (or, at minimum, a single PR). Splitting the shim out
from the deny flip leaves a transient build-broken state; bundling them avoids any window where `cargo build` fails on a
sequenced midpoint.

**Execution note:** Land the lint-rejection smoke test as part of unit verification (introduce a temporary undocumented
`pub fn _smoke_undocumented() {}`, confirm `cargo build` fails, remove the temp fn before commit). This proves the deny
is wired correctly, not just present in source.

**Patterns to follow:** standard rustc attribute scoping. The existing `#![allow(clippy::result_large_err)]` at the top
of `src/lib.rs` shows the crate-level form; the per-variant `#[error("...")]` attributes in `src/error.rs` show the
item-level form the shims will use.

**Test scenarios:**

- Happy path: `cargo build` succeeds across the workspace after the lint flips and the shims land. `cargo test --quiet`
  continues to pass (existing tests + the U4 doc-test compile checks).
- Critical (lint enforcement smoke): introduce a temporary undocumented `pub fn _smoke_undocumented() {}` to any public
  module. `cargo build` fails with `error: missing documentation for a function` pointing at the temp fn. Remove the
  temp fn before commit.
- Critical (shim scoping smoke): temporarily remove the `#[allow(missing_docs)]` from one include site. `cargo build`
  fails with `error: missing documentation` errors on items from `OUT_DIR/auth_matrix.rs` or
  `OUT_DIR/generated_hosts.rs`. Restore the attribute and confirm the build succeeds. Proves the shim is doing
  load-bearing work.
- Cross-platform: pre-push hook's Windows cross-clippy invocation (`RUSTFLAGS="-Dwarnings" cargo clippy --target
  x86_64-pc-windows-gnu --all-targets`) succeeds. Any `pub` item under `cfg(unix)` that lacks docs surfaces here
  cross-platform.
- Integration: full pre-push hook (`cargo fmt --check` → `RUSTFLAGS="-Dwarnings" cargo clippy` → `cargo test --quiet` →
  `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` → Windows cross-clippy) completes green.
- Verification audit: `cargo +nightly doc --no-deps --lib -Z unstable-options --show-coverage` reports ~100% on every
  human-written-source row; intentional residuals only on the `OUT_DIR/...` rows covered by shims.

**Verification:** every test scenario above passes. The plan is shippable when (a) `cargo build` succeeds with
`#![deny(missing_docs)]` active, (b) the lint-rejection smoke test confirms the deny is enforcing, (c) the shim-removal
smoke test confirms the allow is load-bearing, (d) the pre-push hook completes green end-to-end, and (e) the nightly
coverage audit shows full human-written-surface coverage.

---

## Scope Boundaries

Carried forward verbatim from origin (`docs/brainstorms/2026-06-05-public-api-docs-coverage-requirements.md`).

- Item-level documentation of private and crate-internal items (`clippy::missing_docs_in_private_items`) is out. The
  lint targets `pub` only.
- Mock-driven runnable doctests across the API are out. Building and maintaining a public mock surface so every example
  can `assert_eq!` on a call result is disproportionate to the docs.rs reader value.
- Backporting documentation to older published versions (1.x, 2.0.0 itself) is out. The pass lands on the current
  development line and ships on the next release.
- Refreshing `README.md`, `CHANGELOG.md`, or other top-level prose artifacts is out — those are independently maintained
  and not part of this contract.

### Deferred to follow-up work

- Wiring `cargo +nightly doc --show-coverage` as a scheduled or PR-gating CI job. The deny lint covers the regression
  bar; nightly coverage tracking is a separate observability decision that can be taken later without blocking this
  work.
- Documentation conventions written as a standalone style guide (`docs/STYLE.md` or similar). The reference modules
  (`src/error.rs`, `src/api/shortcuts.rs`) carry the implicit pattern; codifying it would be a separate ergonomics
  decision.

---

## Risks & Dependencies

- **Nightly toolchain dependency for the coverage audit.** `cargo +nightly doc --show-coverage` requires a nightly
  install; the project pins stable 1.94.1 via `rust-toolchain.toml`. Mitigation: the audit runs once during U5
  verification on the implementer's machine. CI remains stable-only; the deny lint enforces the contract without
  nightly. If the implementer's environment lacks nightly, install via `rustup toolchain install nightly` for the U5
  audit step only.
- **Build-script include-site attribute placement.** Two `include!()` sites exist today: `src/api/auth_matrix.rs:113`
  (currently wrapped in a context whose surrounding items report 100% file coverage) and `src/skill_install/mod.rs:57`
  (currently at module scope; the file reports 46.5% in part because the included items count against it). Picking the
  wrong attribute scope in U5 either fails to silence the lint (under-scoped) or silences too much human-written code
  (over-scoped). Mitigation: the U5 verification includes a shim-removal smoke test that confirms the allow is doing
  load-bearing work and not silently covering items it shouldn't.
- **Pre-push hook implicitly runs doc tests.** `cargo test --quiet` invokes doc tests by default, which is why the U4
  examples' compile-check works without any hook change. If a future contributor adds `--no-doc` to the hook invocation,
  the U4 compile-check silently disappears. Mitigation: U4's test scenarios document the implicit dependency so it's
  discoverable on the next hook audit.
- **Coverage-tool accounting drift across toolchains.** A future nightly bump may change how `--show-coverage` reports
  re-exports vs originals or how it counts `OUT_DIR`-included items. Mitigation: treat coverage % as a directional
  indicator only. The deny lint is the authoritative contract; the coverage % is for the U5 audit and for human
  inspection on docs.rs, not for any automated decision.

---

## Sources / Research

- **Coverage baseline.** `cargo +nightly doc --no-deps --lib -Z unstable-options --show-coverage` reports 66.8% (480/720
  items). Per- file gaps used to prioritize U2/U3: `src/cli/exit_codes.rs` 0%, `src/store/types.rs` 17.9%,
  `src/api/response/types.rs` 18.8%, `src/api/request.rs` 41.2%, `src/skill_install/mod.rs` 46.5%, `src/auth/pending.rs`
  54.5%.
- **Docs.rs landing.** `https://docs.rs/xurl-rs/2.0.0/xurl/#modules` is the page that motivated the work. Module list at
  the top renders without descriptions today; the U1 + U2 + U3 work directly changes what that page shows.
- **Rustdoc convention.** `///` is outer-doc (binds to the next item); `//!` is inner-doc (binds to the enclosing item).
  The four R4 bug sites place `///` immediately above `pub mod foo;` inside the enclosing `mod.rs`, where it binds to
  `foo` rather than the enclosing module. The fix in U1 converts them to `//!` at the top of each file.
- **Pattern references for prose style.** `src/error.rs` (per-variant, per-field doc style); `src/api/shortcuts.rs`
  lines 1–100 (function- level doc style at 97.3% coverage); `src/skill_install/mod.rs` lines 1–5 (the one existing
  module root using `//!` correctly).
- **Include sites.** `src/api/auth_matrix.rs:113` and `src/skill_install/mod.rs:57` are the two `include!()` invocations
  that need the `#[allow(missing_docs)]` shim in U5.
- **Pre-push hook contract.** `scripts/hooks/pre-push` runs `cargo fmt --check` → `RUSTFLAGS="-Dwarnings" cargo clippy`
  → `cargo test --quiet` (includes doc tests by default) → `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` → Windows
  cross-clippy. Each step picks up the deny lint or the doc-test compile check automatically once U5 lands.
- **Origin brainstorm.** `docs/brainstorms/2026-06-05-public-api-docs-coverage-requirements.md`.
- **Related prior work.** `docs/solutions/security-issues/rust-cli-security-code-quality-audit.md` documents
  `src/cli/exit_codes.rs`'s exit-code values as part of the public CLI contract (78=config, 77=auth, 1=command), which
  U3 should preserve in the new `///` prose for those constants.
