---
date: 2026-06-05
topic: public-api-docs-coverage
---

# Public-API Documentation Coverage

## Summary

A public-API documentation pass that makes the docs.rs landing for `xurl-rs` read complete and adds a hard floor
(`#![deny(missing_docs)]` on `src/lib.rs`) that refuses new undocumented public items at compile time. ~6–8 entry-point
public types get `no_run` usage examples so downstream `xurl` consumers have copyable, compile-checked on-ramps without
the maintenance cost of mock-driven executable doctests.

---

## Problem Frame

`xurl-rs` ships as both a binary (`xr`) and a consumable library (`xurl::*`). The library half is published on docs.rs,
and the v2.0.0 landing page at `https://docs.rs/xurl-rs/2.0.0/xurl/` is the first surface a downstream Rust consumer
reads when deciding whether to depend on the crate.

Today that landing page is sparse: the module list has no descriptions, and many public types lack item-level prose. A
`cargo +nightly doc --show-coverage` run reports **66.8%** documentation coverage (480 of 720 items), but the visible
picture is worse than the number suggests — four of the most prominent module roots (`src/api/mod.rs`,
`src/auth/mod.rs`, `src/cli/mod.rs`, `src/store/mod.rs`) carry `///` outer-doc comments placed immediately above a `pub
mod foo;` declaration. Rustdoc attaches each of those docs to the next item (the inner submodule), not to the enclosing
module, so the four modules render as undocumented on the landing page despite having header prose in the source.

No lint currently guards the coverage. Even a one-shot documentation pass would start drifting back the moment another
`pub` item lands without a doc.

## Key Decisions

- **Public surface only.** The lint and the documentation pass target `pub` items. Private and crate-internal items are
  not required to carry docs. The cost of dragging ~240 internal items into the contract exceeds the docs.rs reader
  value of doing so.
- **Hard floor, not soft warning.** `#![deny(missing_docs)]` rather than `#![warn(missing_docs)]`. The deny variant
  makes new undocumented `pub` items fail `cargo build` rather than emitting warnings that get tuned out.
- **Library only, binary untouched.** The lint applies to `src/lib.rs`; the binary entry point at `src/main.rs` is left
  alone. The binary has no published API surface and does not constrain downstream consumers.
- **Examples on entry points, not everywhere.** A small number of public types (~6–8) are the on-ramps a downstream
  consumer touches first. Those get usage examples. Error variants, response sub-types, and internal helpers get
  intent-level prose but no examples — the maintenance cost would be disproportionate.
- **Examples compile-checked, not executed.** All example blocks use `no_run` — they compile against the public API
  during `cargo test --doc` (catching API breakage in docs) but do not execute. A live wire-smoke variant was considered
  and rejected: docs.rs renders no link, badge, or richer surface for an executed block, so the engineering cost (an
  integration test, a CI gating decision, an X-reachability dependency) exceeds the docs.rs reader value.
- **Mis-targeted module docs are converted to `//!` inner-doc form.** The four bug sites become `//!` at the top of each
  `mod.rs`. The alternative (relocating `///` blocks to `src/lib.rs` immediately above `pub mod foo;`) produces an
  equivalent docs.rs result but splits documentation conventions across files; consolidating on `//!` for module roots
  is simpler.

## Requirements

### Documentation completeness on the public surface

- R1. `src/lib.rs` begins with a `//!` crate-level doc that orients a docs.rs reader on what the `xurl` library is, what
  the `xr` binary is, the relationship between them, and the auth-method landscape (OAuth1, OAuth2 PKCE, Bearer).
  Length: enough to ground the reader before they descend into modules; brevity over completeness.
- R2. Every public module (`pub mod foo`) carries a `//!` inner-doc header at the top of its `mod.rs` (or single-file
  module) that names what the module is responsible for and its place in the crate.
- R3. Every `pub` item exposed from `xurl::*` — functions, structs, enums, type aliases, consts, traits — carries a
  `///` outer-doc with at least one sentence describing its purpose. `pub` struct fields carry per-field docs; enum
  variants carry per-variant docs.
- R4. The four mis-targeted module docs in `src/api/mod.rs`, `src/auth/mod.rs`, `src/cli/mod.rs`, and `src/store/mod.rs`
  are converted from `///` outer-doc immediately above `pub mod ...;` to `//!` inner-doc at the top of the file. Each
  converted module renders with its intended description on its docs.rs module page.

### Lint-enforced floor

- R5. `#![deny(missing_docs)]` is set on `src/lib.rs` and rejects any undocumented `pub` item at compile time.
- R6. `src/main.rs` is not touched by the lint. The binary entry point retains its current annotations.
- R7. Build-script-emitted files included via `include!()` from `OUT_DIR` (`auth_matrix.rs`, `generated_hosts.rs`) are
  scoped under a localized `#[allow(missing_docs)]` at the include site so generated code does not need hand-edited doc
  comments. The allow is narrow — applied at the `include!()` call site, not at crate level.
- R8. R5 and R7 land in the same change as R1–R4, or in a tightly-following PR within the same release cycle. The deny
  lint fails the build until every `pub` item is documented and the build-script include sites carry the allow shim — so
  a standalone "turn on the lint" commit is not viable.

### Compile-checked usage examples on entry points

- R9. ~6–8 entry-point `pub` types receive usage examples in their doc comments. The candidate set (finalized during
  planning) covers the request client, output configuration, token store, the error envelope, and a representative
  subset of `shortcuts::*`. The criterion for inclusion: would a downstream Rust consumer of `xurl` plausibly touch this
  type in their first hour with the crate?
- R10. Every example block uses `no_run` — it compiles against the public API during `cargo test --doc` but does not
  execute. The blocks serve two purposes: catch API-shape breakage in docs (a renamed parameter or moved type fails the
  build), and give docs.rs readers copyable on-ramps. No example block executes against a live network endpoint or
  depends on credentials.

## Scope Boundaries

- Item-level documentation of private and crate-internal items (`clippy::missing_docs_in_private_items`) is out. The
  lint targets `pub` only.
- Mock-driven runnable doctests across the API are out. Building and maintaining a public mock surface so every example
  can `assert_eq!` on a call result is disproportionate to the docs.rs reader value.
- Backporting documentation to older published versions (1.x, 2.0.0 itself) is out. The pass lands on the current
  development line and ships on the next release.
- Refreshing `README.md`, `CHANGELOG.md`, or other top-level prose artifacts is out — those are independently maintained
  and not part of this contract.

## Sources / Research

- `cargo +nightly doc --no-deps --lib -Z unstable-options --show-coverage` — emits the per-file coverage table used to
  establish the 66.8% baseline and to identify the lowest-coverage modules (`src/api/response/types.rs` at 18.8%,
  `src/store/types.rs` at 17.9%, `src/api/request.rs` at 41.2%, `src/skill_install/mod.rs` at 46.5%,
  `src/auth/pending.rs` at 54.5%).
- `https://docs.rs/xurl-rs/2.0.0/xurl/#modules` — the docs.rs landing page that motivated the brainstorm. Module list at
  the top renders with no descriptions today.
- Rustdoc convention: `///` is outer-doc (binds to the next item), `//!` is inner-doc (binds to the enclosing item).
  Module roots may use either form correctly — `///` works when placed in the parent (e.g., above `pub mod api;` in
  `src/lib.rs`); `//!` works when placed at the top of the module's own `mod.rs`. The current bug pattern in
  `src/api/mod.rs`, `src/auth/mod.rs`, `src/cli/mod.rs`, and `src/store/mod.rs` places `///` inside `mod.rs` above a
  `pub mod foo;` declaration, where it binds to `foo`, not to the enclosing module.
- `src/lib.rs` — currently has no `//!` crate-level doc; this is the rendered file at `/xurl/index.html` on docs.rs.
- `src/skill_install/mod.rs` — the one existing module root that uses `//!` correctly. Reference shape for R2
  conversions.
