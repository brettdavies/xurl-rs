---
name: xurl-rs
binary: xr
description: Fast, ergonomic CLI for the X (Twitter) API. Rust port of the Go xurl, with OAuth1 / OAuth2-PKCE / Bearer auth, 27 high-level shortcut commands, chunked media upload, and streaming.
homepage: https://github.com/brettdavies/xurl-rs
repository: https://github.com/brettdavies/xurl-rs
---

# AGENTS.md

## Running xr

The CLI package is `xurl-rs` and the installed binary is `xr`. The library is the `xdk-rs` package, imported as `xdk`;
the `xurl` library target inside the CLI crate exists for its own tests and is not an API.

```bash
# Raw request — full control over verb, path, body, headers
xr /2/users/me
xr -X POST /2/tweets -d '{"text":"hello"}'
xr '/2/tweets/search/recent?query=from:jack'

# Shortcut command — one of the high-level wrappers over common endpoints
xr whoami
xr post "hello"
xr search "from:jack"
xr like 1234567890
xr follow @jack

# JSON output for parsing
xr whoami --output json
xr search "from:jack" --output json | jaq -r '.data[].id'

# JSONL — one record per line on streaming endpoints
xr -s /2/tweets/search/stream --output jsonl | jaq -c '.data.id'

# Media upload — chunked INIT/APPEND/FINALIZE/STATUS state machine
xr media upload ./image.png

# Schema introspection — emit the JSON shape of a typed response
xr schema --list
xr schema whoami

# Auth — see § Auth paths below
xr auth oauth2                   # interactive OAuth2-PKCE in a browser
xr auth oauth2 --no-browser      # headless OAuth2 (copy-paste URL flow)
xr auth status                   # list configured apps and token freshness
```

Bare `xr` (no arguments) prints the root help on stdout at exit 0. A word that names no command exits 2 with reason
`unknown-command`, echoing the word in `command` and naming the nearest real command in `suggestion` when one is close
enough. Read those rather than parsing the message.

## Auth paths

Four auth modes, selected by what's available in the token store and the environment.

| Path               | When used                                                                                           | Surface                                                 |
| ------------------ | --------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| OAuth1 (HMAC-SHA1) | User tokens stored in `~/.xurl` via `xr auth oauth1` (consumer key + secret, access token + secret) | v2 user-context endpoints whose spec entry lists OAuth1 |
| OAuth2 PKCE        | User-scoped flow via `xr auth oauth2` (browser-driven) or `xr auth oauth2 --no-browser`             | All v2 user-scoped endpoints                            |
| OAuth2 headless    | `xr auth oauth2 --no-browser`: copy-paste the URL flow on a graphical-display-less host             | Same as OAuth2 PKCE                                     |
| Bearer (app-only)  | `XURL_BEARER_TOKEN=…` in env, or stored via `xr auth app --bearer-token`                            | v2 read-only endpoints + search                         |

The CLI picks per request: if a Bearer is set and the endpoint accepts app-auth, it's used; otherwise the stored
user-scoped tokens for the active app drive the call. Multi-app is supported in the token store; `xr auth status`
enumerates them.

On exit 77 the error names what to do next. Read it from the envelope rather than guessing:

```bash
xr --output json auth status            # {"status":"ok","apps":[...]}; each entry carries client_id_hint and bearer
xr --output json whoami 2>&1 >/dev/null # the failure itself, carrying next_step
```

Branch on `next_step.action`: `register-app` means nothing is registered, so run its `template` with real values;
`sign-in` and `select-app` carry a `command` to run verbatim; `inspect-store` means the store file could not be read and
names it in the message.

`crates/xdk/src/auth/` holds the four implementations. OAuth1 signing follows RFC 5849 (HMAC-SHA1, percent-encoded base
string, sorted parameter list). PKCE is the standard `code_verifier`/`code_challenge` flow with refresh-token rotation.

## Token store

YAML at `~/.xurl`. Schema is documented in `crates/xdk/src/store/types.rs`. Migration logic lives in
`crates/xdk/src/store/migration.rs` and runs on every load: older formats upgrade transparently and the upgraded file is
written back. Multiple apps are stored under the same file with a per-app block.

`xr auth status` is the operator-facing surface. Programmatic access uses `xdk::store::TokenStore`.

## Output formats

`OutputConfig` (`crates/xurl-cli/src/cli/output/mod.rs`) drives seven formats, selected with `--output`:

- `text` (default): human-readable tables / formatted responses
- `json`: pretty-printed JSON envelope
- `jsonl` (alias `ndjson`): one JSON record per line; ideal for streaming + pipeline composition with `jaq`
- `yaml` (alias `yml`), `csv`, `tsv`: tabular and config-friendly renderings of the same envelope

Streaming endpoints emit a continuous JSONL stream when `--output jsonl` is set; non-streaming endpoints emit one record
then close.

Text output is written for humans and the structured formats for agents, and the two need not match word for word: text
carries prose and a help pointer, structured output carries stable fields to branch on. Every structured error carries a
kebab-case `reason` from a closed set, an `exit_code`, a human `message`, the offending value when there is one, and a
`next_step` object `{action, command | template, docs}`. `action` is a closed set; `command` is runnable verbatim by a
non-TTY caller, while `template` carries angle-bracket placeholders only the caller can fill. Prefer additive envelope
changes: add keys rather than renaming or retyping existing ones.

## Shortcut commands

`crates/xdk/src/api/shortcuts.rs` ships `pub fn` wrappers over the X API endpoints documented in
`crates/xdk/vendor/x-api-openapi.json`; the file is the list. Each shortcut names its endpoint through the constants
`crates/xdk/build.rs` generates from its `SHORTCUT_TEMPLATES` rows (`auth_matrix::endpoints`), maps to one CLI command
via `crates/xurl-cli/src/cli/`, and returns a typed response via `crates/xdk/src/api/response/types.rs`. The build
script also generates the auth matrix from the vendored spec and fails the build if a declared endpoint is absent from
it; `crates/xdk/src/api/auth_matrix.rs` wraps the generated table for runtime lookup.

### Adding a command family

A family touches the surfaces below, about nine files across the two crates. Every surface a contributor could forget is
covered by a walk that reads the live declaration (clap's command tree, the schema registry, the alias table, or the
declared endpoint list) and names what is missing, the likely cause, and the file to edit. Forget a surface, and a test
says which one. No walk checks against a list written into the test, so adding a family means editing the surfaces,
never the tests.

- **Endpoint declaration.** A `SHORTCUT_TEMPLATES` row in `crates/xdk/build.rs`: the constant name, the method, and the
  spec path. The build panics when the path is not in the vendored spec, and `crates/xdk/tests/auth_matrix_coverage.rs`
  fails when the generated auth matrix has no entry for it.
- **Shortcut.** A `pub fn` in `crates/xdk/src/api/shortcuts.rs` that reads `endpoints::<NAME>` for its method and path.
  `crates/xdk/tests/path_literal_guard.rs` fails on a `/2/` literal in the request layer or the mock.
- **Typed response.** A struct in `crates/xdk/src/api/response/types.rs` deriving `JsonSchema`, or an existing one.
  `crates/xdk/tests/spec_types_validation.rs` fails when a struct field is not in the spec's schema.
- **Response fixture.** A key in `crates/xdk/tests/fixtures/openapi/example_responses.json`, with a `spec_*` test and a
  `FIXTURE_ENDPOINTS` row in `crates/xdk/tests/spec_validation.rs`. That file fails for a fixture with no test, no
  endpoint mapping, or a shape the spec does not give the endpoint.
- **Mock route.** A `Route` in `crates/xdk/src/testing/mod.rs` naming the endpoint constant and the fixture that answers
  it. `crates/xdk/tests/mock_endpoint_coverage.rs` fails for a declared endpoint the running mock does not answer.
- **Clap command.** A variant in `crates/xurl-cli/src/cli/mod.rs` with its `after_help`, through a
  `crates/xurl-cli/src/cli/family_help.rs` declaration when the family has several verbs.
  `crates/xurl-cli/tests/golden_tests.rs` demands a `help-<command>.golden` fixture for every command.
- **Dispatch arm.** An arm in `crates/xurl-cli/src/cli/commands/mod.rs`; a verb that resolves a handle or acts on a post
  from the caller's account goes through `act_from_me_on_user`, `act_on_user`, or `act_from_me_on_post`. The dry-run
  golden fixtures pin the envelope.
- **Schema registry.** A `SCHEMA_ENTRIES` row, or a `SCHEMA_LESS_COMMANDS` name for a command with no typed response, in
  `crates/xurl-cli/src/cli/commands/schema.rs`, then `scripts/generate-response-schemas.sh`.
  `crates/xurl-cli/tests/schema_tests.rs` fails for a command in neither set or in both, and for a committed schema that
  drifted.
- **Validate alias.** A `typed_alias` row, or an `UNVALIDATED_TYPES` entry for a shape the CLI builds itself, in
  `crates/xurl-cli/src/cli/commands/validate.rs`. The walk in `crates/xurl-cli/tests/schema_tests.rs` fails for a
  registry response type with neither, and `crates/xurl-cli/tests/validate_tests.rs` fails when the `--schema` help and
  the `unknown-schema` envelope disagree.
- **Examples page.** An `xr <family>` line under the use-case section it belongs to in
  `crates/xurl-cli/src/cli/commands/examples.rs`. `crates/xurl-cli/tests/agentic_tests.rs` fails for a family the page
  never invokes.
- **Completions.** `scripts/generate-completions.sh`; CI's completions freshness gate fails when they are stale.

Re-bless a golden fixture only where a surface above says the content moved, with `XURL_GOLDEN_BLESS=1 cargo test -p
xurl-rs --test golden_tests`; a fixture that moves for any other reason is a defect.

`crates/xurl-cli/tests/recipe_guard.rs` checks this section: every file it names exists, every walk (a test whose
failure message carries a `Cause:` line) is named here, and no document lists the shortcut functions by hand.

### Command grammar: flags vs subcommands

- Flags never select endpoints. A flag tunes one endpoint's request (auth method, output shape, pagination, fields); it
  never retargets the path.
- A standalone action or read in the core domain gets its own top-level command word: `dm`/`dms`,
  `bookmark`/`bookmarks`, and `timeline`/`mentions` are separate commands because they are separate endpoints.
- A subcommand family groups a noun that owns several operations: tooling nouns (`auth`, `skill`, `schema`,
  `completions`) and API namespaces with sibling endpoints (`media` over `/2/media/*`, `usage` over `/2/usage/*`).

Routing a new endpoint: when an existing family noun owns it (an API-namespace sibling), add a subcommand there; when it
stands alone in the core domain, add a top-level command; never add an endpoint-selecting flag to an existing command.

## Architecture

The repository is a Cargo workspace with two members: `crates/xdk` (package `xdk-rs`, library target `xdk`) and
`crates/xurl-cli` (package `xurl-rs`, binary `xr`). Paths below are relative to the member's `src/`.

- `crates/xdk/src/api/`: HTTP client (`request/`: client and option types in `mod.rs`, URL rendering in `url.rs`,
  auth-scheme selection in `auth_header.rs`, transport in `transport.rs`), endpoints (`endpoints.rs`), shortcuts
  (`shortcuts.rs`), media upload (`media.rs`), and typed responses (`response/`).
- `crates/xdk/src/auth/`: OAuth1 (HMAC-SHA1 per RFC 5849), OAuth2 PKCE (interactive + headless via callback handler),
  Bearer token. PKCE pending-state is in `pending.rs`; the callback HTTP server is `callback.rs`.
- `crates/xurl-cli/src/cli/`: clap-based CLI. `commands/mod.rs` is the handler layer; subdir files split media, schema,
  streaming, and `commands/auth/`, where `mod.rs` routes to `signin.rs`, `session.rs`, and `apps.rs` and owns
  `AppStatusEntry`, while `types.rs` holds the bearer-source enum and the redirect-URI shapes. `exit_codes.rs` encodes
  the exit-code contract.
- `crates/xdk/src/config/`: env-var-based configuration.
- `crates/xdk/src/store/`: YAML token store at `~/.xurl`; multi-app, with `migration.rs` for transparent upgrades.
- `crates/xurl-cli/src/cli/output/`: `OutputConfig` for text/json/jsonl formatting; `delimited.rs` holds the csv/tsv
  serializer.
- `crates/xdk/src/error.rs`: `Error` (re-exported as `xdk::Error`) via `thiserror`.
- `crates/xdk/src/lib.rs`: public library surface. The `xdk` library is consumable from downstream Rust crates; the
  binary `xr` is one consumer among potentially several. `crates/xurl-cli/src/lib.rs` exposes the CLI module only so the
  binary's own tests can drive it in-process; it is not an embedder API.

### Where a change goes

The line between the crates holds on four rules; a change that crosses one belongs on the other side.

1. **Dependencies.** The library's graph carries no `clap`, `clap_complete`, `colored`, or `open` on any feature
   combination: `cargo tree -p xdk-rs --all-features -i <crate>` is empty for each, and CI's `Consumer check` compiles
   an out-of-package embedder against the documented surface.
2. **I/O.** The library returns data, URLs, and `tracing` events (targets `xdk::wire`, `xdk::media`, `xdk::vocabulary`,
   `xdk::auth`, `xdk::store`, `xdk::config`); it never writes to stdout or stderr and never opens a browser. It reads
   the process environment in one place, `EnvOverrides::from_env`, which takes the client variables (`CLIENT_ID`,
   `CLIENT_SECRET`, `REDIRECT_URI`, `AUTH_URL`, `TOKEN_URL`, `API_BASE_URL`, `INFO_URL`, `XURL_BEARER_TOKEN`); `HOME`,
   `XURL_OUTPUT`, `XURL_TOKEN_STORE`, and `NO_COLOR` are read once, in `crates/xurl-cli/src/cli/env.rs`. Binding the
   loopback OAuth2 callback listener and reading or writing `~/.xurl` are network and file I/O, and belong to the
   library.
3. **Surface.** Every published module is one an embedder calls (`api`, `auth`, `config`, `error`, `store`, and
   `testing` behind its feature). Machinery only `xr` reaches is `pub` and `#[doc(hidden)]`, with a comment naming the
   `xr` path that uses it. The CLI crate's `xurl` library target is doc-hidden and exists so its integration tests can
   drive the dispatcher in-process.
4. **Errors.** The library states the fact: an `Error` variant plus `kind()`, `exit_code()`, `next_action()`, and
   `docs_url()`, each matched exhaustively in-crate. The binary composes the sentence: the `Auth Error:` prefixes, the
   recovery wording, `next_step.command` and `template`, and the envelope live in `crates/xurl-cli/src/cli/output/`.
   Exit codes sit on the library side because they are part of the closed classification an embedder branches on; the
   wording is not.

Deciding where a new feature lands:

- It talks to X (an endpoint, an auth scheme, a media phase, a stream), reads or writes the credential store, or
  classifies a failure: `crates/xdk`. The shortcut, its typed response, and its `Error` arm land there; the `xr` command
  that exposes it is a separate change in `crates/xurl-cli`.
- It parses argv, renders output in any format, reads terminal-shaped environment, prompts, opens a browser, installs a
  skill bundle, or turns an error into prose: `crates/xurl-cli`.
- `xr` needs a library internal no embedder would call: `pub` plus `#[doc(hidden)]` in the library with a comment naming
  the `xr` path; never `pub(crate)` (it does not cross crates) and never a copy in the binary.
- A new dependency goes in the library only when an embedder's build needs it; test-only helpers sit behind the
  `testing` feature; anything terminal-shaped goes to the binary.

Three placements are fixed: `schemars::JsonSchema` derives on the response types are library (an embedder can emit the
same schemas an MCP tool definition needs; `xr schema` is the binary); the token store is library and shared by both
crates, while `XURL_TOKEN_STORE` is the binary's alone; sign-in logic (PKCE, the callback listener, the token exchange)
is library, and opening the browser and printing the paste-the-URL advice is binary.

## Quality bar

- Clippy clean, edition 2024 (`cargo clippy -- -D warnings`)
- Formatted with rustfmt (`cargo fmt --check`); style edition pinned in `rustfmt.toml`
- No `unwrap()` in production code
- Comprehensive tests (`cargo test`): unit, integration, and differential conformance
- Zero broken tests policy: a failing test on `dev` blocks new work until it's fixed or reverted, not until "later"
- `cargo deny check` passes (advisories, licenses, bans, sources)
- Cross-platform: Linux, macOS, Windows (MSVC). Pre-push and CI both run a Windows compatibility check.

The pinned toolchain (`rust-toolchain.toml`) is the supply-chain anchor. Rustup verifies component SHA256s from the
distribution manifest; the pin is effectively a SHA pin. Toolchain bumps land via reviewed PR after ≥7-day quarantine.

## Testing

```bash
cargo test                    # unit + integration
cargo test -- --ignored       # slower / network-dependent tests
scripts/hooks/pre-push        # local CI mirror (fmt, clippy, test, msrv, doc, examples, the offline example
                              # run, TLS feature cells, deny, shellcheck, Windows cross-clippy, markdownlint,
                              # actionlint; docs.rs nightly build and cargo-hack powerset when installed)
```

Tests never resolve the real home directory. Build stores and auth on an explicit path under a `tempfile::TempDir`
(`TokenStore::new_with_path`, `Auth::new_with_store_path`, `run_with_store_path`).
`crates/xurl-cli/tests/store_isolation_guard.rs` fails the suite when a test file names `Auth::new(`,
`TokenStore::new()`, `TokenStore::with_credentials(`, `default_store_path()`, `default_pending_path()`,
`dirs::home_dir()`, sets `HOME` on a child process, or spawns the `xr` binary outside `common::xr()` and
`common::xr_with_store` (which point `XURL_TOKEN_STORE` at an unwritable scratch path or the test's own temp store); a
test that must touch the real path goes on its allowlist with the reason. A companion guard in
`crates/xurl-cli/tests/agentic_tests.rs` derives every environment variable either crate reads and fails when `xr
--help` does not advertise one.

`scripts/hooks/` holds a pair, activated together by `git config core.hooksPath scripts/hooks`: `pre-commit` runs
format, workflow, and markdown checks over the staged files only, and `pre-push` runs the CI mirror over the repo. Run
`scripts/hooks/pre-push` by hand when `core.hooksPath` is unset; invoked that way it sweeps everything, where the hook
path scopes each step to what the push changes.

Four CI gates have no hook counterpart and fail only on the PR: completions freshness, the package check, the public-API
semver gate, and the agent-native audit with the release binary's size ceiling. Run them yourself when a change touches
the CLI surface, the library API, or the release profile.

## Releasing

See [`RELEASES.md`](RELEASES.md) for the operational runbook, [`RELEASES-PREFLIGHT.md`](RELEASES-PREFLIGHT.md) for the
pre-cut go/no-go checklist, and [`RELEASES-RATIONALE.md`](RELEASES-RATIONALE.md) for the why behind every rule. The
short version: feature branch → PR to `dev` (squash) → `dev`'s tree overlaid onto `release/v<version>` cut from `main` →
PR to `main` (squash) → annotated tag push triggers `release.yml`.

### Spec-refresh PRs

The spec-drift workflow opens PRs to `dev` (head `spec-refresh`) whose body is an agent runbook. Invoke the reconciling
agent as "Triage and fix PR #N".

## Known differences from the Go original

See [`KNOWN_DIFFERENCES.md`](KNOWN_DIFFERENCES.md) for intentional deviations from
[`xdevplatform/xurl`](https://github.com/xdevplatform/xurl).

## Documented solutions

On `dev`, `docs/solutions/` is a symlink to `~/dev/solutions-docs/`, a shared, searchable archive of past solutions and
best practices organized by category with YAML frontmatter (`module`, `tags`, `problem_type`). Search with `qmd query
"<topic>" --collection solutions` before implementing or debugging in a documented area; the corpus crosses repos and
already captures known pitfalls.

`CONCEPTS.md`, on `dev` alongside the engineering docs, holds the shared domain vocabulary (entities, named processes,
and status concepts with project-specific meaning), relevant when orienting to the codebase or discussing domain
concepts. Neither the symlink nor `CONCEPTS.md` ships to `main`; `guard-main-docs` blocks both.
