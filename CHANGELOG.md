# Changelog

All notable changes to this project will be documented in this file.

## [2.1.0] - 2026-06-05

### Added

- Document the entire `xurl` library public surface (crate-level overview, per-module headers, per-item prose, per-field and per-variant docs) so docs.rs renders complete reference material for downstream Rust consumers. by @brettdavies in [#70](https://github.com/brettdavies/xurl-rs/pull/70)
- Enforce `#![deny(missing_docs)]` on the library surface so new undocumented `pub` items fail `cargo build` at compile time.
- Add `no_run` usage examples on the seven entry-point types (`ApiClient`, `OutputConfig`, `TokenStore`, `XurlError`, `get_me`, `create_post`, `search_posts`) so downstream consumers have copyable on-ramps that compile against the public API.
- Five top-level `pub const`s on the `xurl` library surface: `CRATE_VERSION` (this crate's version), `CRATE_GIT_SHA` (`Option<&str>`, the git HEAD SHA at build time when one is available), `API_SPEC_VERSION` (`info.version` of the vendored X API spec), `API_SPEC_SHA256` (content hash of the vendored spec file), and `API_SPEC_DATE` (UTC date of the last refresh). Downstream library consumers can read linked-crate identity and X API spec identity without invoking `xr --version` as a subprocess. by @brettdavies in [#71](https://github.com/brettdavies/xurl-rs/pull/71)
- `vendor/spec-metadata.json`: a checked-in sidecar carrying the vendored spec's identity (info_version, content_sha256, refreshed_at, source_url). `scripts/refresh-x-openapi.sh` writes this atomically alongside `vendor/x-api-openapi.json` so build-time metadata always describes the actual bytes that ship.
- `scripts/diff-x-openapi-spec.sh`: standalone diff/render tool for X OpenAPI snapshots. Surfaces path, schema, and categorical-value (enum + discriminator-mapping) additions and removals as a markdown body, used by the spec-drift workflow and callable manually.

### Changed

- Bump `hmac` 0.12.1 → 0.13.0 and `sha1` 0.10.6 → 0.11.0 so OAuth1 HMAC-SHA1 signing now runs on the `digest` 0.11 line alongside `sha2`. No user-observable behavior change in `xr`; downstream Rust consumers of the `xurl` library that link these crates directly will see the major version bumps. by @dependabot[bot] in [#61](https://github.com/brettdavies/xurl-rs/pull/61)
- Vendored X API spec refreshed to upstream as of 2026-06-05. `info.version` still 2.165; path count still 139. New schema `PostDeleteActivityResponsePayload`; new `post.create`/`post.delete` values in the `ActivityStreamingResponsePayload.discriminator.mapping` and `ActivitySubscriptionCreateRequest.properties.event_type.enum`; new `includes: Expansions` field on the streaming payload. by @brettdavies in [#71](https://github.com/brettdavies/xurl-rs/pull/71)
- Spec-drift workflow report now distinguishes "silent content bump" (`info.version` matches on both sides) from "real version bump," lists path / schema / categorical-value diffs with `+`/`-` value markers, and renders the refresh-locally footer only when no structural diff surfaced.

### Documentation

- Fix four mis-targeted `///` blocks in `src/api/mod.rs`, `src/auth/mod.rs`, `src/cli/mod.rs`, `src/store/mod.rs` where the doc bound to the next item (a `pub mod foo;` or `use ...;`) rather than the enclosing module. Same pattern fixed across additional files surfaced during implementation. by @brettdavies in [#70](https://github.com/brettdavies/xurl-rs/pull/70)

**Full Changelog**: [v2.0.0...v2.1.0](https://github.com/brettdavies/xurl-rs/compare/v2.0.0...v2.1.0)

## [2.0.0] - 2026-06-05

### Added

- Endpoint-aware auth-method validator (`xurl::api::auth_matrix`) refuses `--auth X` invocations against endpoints whose spec entry does not list `X`, with a typed `XurlError::AuthMethodMismatch` and `auth-method-mismatch` envelope by @brettdavies in [#56](https://github.com/brettdavies/xurl-rs/pull/56)
- Auto-detect intersects the active app's stored credentials with the endpoint's accepted schemes and prefers OAuth2 → OAuth1 → Bearer; three envelope shapes surface the outcome: - **Explicit mismatch**: user passed `--auth X` against an endpoint that doesn't accept `X` - **Empty intersection**: active app has creds but none overlap with the endpoint's supported set - **Wrong app**: active app holds nothing, but other apps in the store do; carries `other_apps_with_creds` so the recovery hint can name them
- AuthMethodMismatch envelope carries `endpoint` (spec template, for agent pattern-matching), `rendered_url` (substituted path, for user-facing messages), `app` (active app name), `requested`, `supported`, `available_in_app`, and `other_apps_with_creds`
- `EXIT_AUTH_MISMATCH = 2` (`EX_USAGE`) for the new error variant; distinct from `EXIT_AUTH_REQUIRED = 77`
- CI drift-check workflow (`.github/workflows/spec-drift.yml`): weekly cron + PR-touches-vendor trigger; opens or updates a `[spec-drift]` issue on divergence
- Build-time shortcut coverage check fails `cargo test` when any shortcut targets a path absent from the spec
- `MIGRATING.md` at the repo root documenting the v1 → v2 migration for library consumers
- `vendor/x-api-openapi.json` + `scripts/refresh-x-openapi.sh` + `vendor/README.md`
- `scripts/generate-changelog.py --dry-run` idempotency check that runs the regen pipeline against `CHANGELOG.md`, compares against the current file, restores the original on exit, and exits 1 with a unified diff if regeneration would drift. by @brettdavies in [#57](https://github.com/brettdavies/xurl-rs/pull/57)

### Changed

- Release flow documentation (`RELEASES.md`, `RELEASES-RATIONALE.md`) names `generate-changelog.py` as the entry point. by @brettdavies in [#55](https://github.com/brettdavies/xurl-rs/pull/55)
- BREAKING: `xurl::api::RequestOptions.endpoint: String` replaced by `target: RequestTarget` (`Template { path, path_params, query } | RawUrl(String)`); shortcut callers and direct library consumers MUST construct the new shape by @brettdavies in [#56](https://github.com/brettdavies/xurl-rs/pull/56)
- BREAKING: `ApiClient::get_auth_header_public` takes `&RequestOptions` instead of the four stringly-typed parameters
- `RawUrl` targets now enforce an `http(s)://` scheme allowlist; non-HTTP(S) values return `XurlError::InvalidUrl`
- `cliff.toml` routes `feat!:`/`fix!:` and `BREAKING CHANGE:` footers into a `[Breaking]` changelog section

### Fixed

- `scripts/generate-changelog.py` now ships alongside the docs that reference it; the previous `generate-changelog.sh` delegated to a missing Python helper and could not be run as documented. by @brettdavies in [#55](https://github.com/brettdavies/xurl-rs/pull/55)
- Auto-detect no longer attempts OAuth2 for endpoints the spec lists as Bearer-only (or vice versa); the intersection refuses incompatible combinations before the HTTP round-trip rather than letting X reject with a generic 403 by @brettdavies in [#56](https://github.com/brettdavies/xurl-rs/pull/56)
- When the active app holds no credentials but other apps in the store hold some, the resolver surfaces the wrong-app envelope (exit 2) instead of the misleading generic `auth-required` (exit 77)
- `scripts/generate-changelog.py` no longer prepends a duplicate `## [X.Y.Z]` section when one already exists for the requested tag. by @brettdavies in [#57](https://github.com/brettdavies/xurl-rs/pull/57)

### Removed

- BREAKING: `block_user` and `unblock_user` shortcuts (target paths absent from the current OpenAPI spec); use raw mode if you need the behavior, see MIGRATING.md by @brettdavies in [#56](https://github.com/brettdavies/xurl-rs/pull/56)

**Full Changelog**: [v1.3.0...v2.0.0](https://github.com/brettdavies/xurl-rs/compare/v1.3.0...v2.0.0)

## [1.3.0] - 2026-06-04

### Added

- Add `RELEASES-PREFLIGHT.md`, a pre-cut go/no-go checklist that gates release-branch creation and covers API-contract surface, live-API smoke tests, distribution probes, and release-mechanics sanity. by @brettdavies in [#27](https://github.com/brettdavies/xurl-rs/pull/27)
- Add `RELEASES-RATIONALE.md`, holding the rationale behind every release rule: branching model, PR conventions, triple-diff verification, CHANGELOG generation, release pipeline, prose scrubbing, branch-protection pitfalls.
- Add `.github/pull_request_template.md` so new PRs in this repo get the canonical Summary, Changelog, Testing, and Files Modified prompts.
- Add `.markdownlint-cli2.yaml` so local markdownlint runs use the same line-wrap and style rules as other brettdavies repos.
- Expose `xurl::cli` as a public module with three layered entrypoints: `run_argv() -> i32` for the binary, `run(args, stdout, stderr) -> i32` for library callers, and `run_with_store_path(args, stdout, stderr, store_path) -> i32` for tests. by @brettdavies in [#29](https://github.com/brettdavies/xurl-rs/pull/29)
- Add `Auth::new_with_store_path(cfg, path)` as the canonical constructor and `Config::default_store_path()` exposing the legacy `~/.xurl` resolution to library consumers.
- Add a compile-time `Send + Sync` assertion on `Auth` so regressions break the build instead of leaking into async callers.
- New CLI surface: `auth apps add --redirect-uri`, `auth apps update --redirect-uri`, `auth apps redirect-uri get [NAME]`, `auth apps redirect-uri set NAME URI`. by @brettdavies in [#30](https://github.com/brettdavies/xurl-rs/pull/30)
- Structured JSON output for `auth status`, `auth apps list`, and `auth apps redirect-uri get` under `--output json`.
- `.anc.toml` declaring the X (Twitter) platform's canonical command verbs (post, reply, like, follow, mute, dm, ...) for anc's `p6-may-standard-names` audit. by @brettdavies in [#33](https://github.com/brettdavies/xurl-rs/pull/33)
- New `--color <auto|always|never>` global flag (`XURL_COLOR`); `auto` honors stderr's TTY-ness, and `NO_COLOR` is absolute per https://no-color.org/. by @brettdavies in [#34](https://github.com/brettdavies/xurl-rs/pull/34)
- Environment-variable backing for every agentic flag: `XURL_VERBOSE`, `XURL_QUIET`, `XURL_NO_INTERACTIVE`, `XURL_TIMEOUT`, `XURL_APP` (in addition to the pre-existing `XURL_OUTPUT`). Boolean flags use `FalseyValueParser` so `XURL_QUIET=0` correctly disables quiet.
- Add `xr examples` subcommand that prints a curated, grouped gallery of common invocations covering auth, posting, social graph, account inspection, DMs, media upload, raw mode, schemas, and multi-app workflows. by @brettdavies in [#35](https://github.com/brettdavies/xurl-rs/pull/35)
- Add `Examples:` block with 3 to 5 curated invocations to every `xr` subcommand's `--help`, including paired text and `--output json` forms for scripting.
- Add `ENVIRONMENT VARIABLES:` section to `xr --help` listing `XURL_OUTPUT`, `XURL_QUIET`, `XURL_NO_INTERACTIVE`, `XURL_TIMEOUT`, `XURL_COLOR`, `XURL_VERBOSE`, `XURL_APP`, and `REDIRECT_URI` with their flag equivalents.
- Add `EXIT CODES:` matrix to `xr --help` documenting codes 0 through 5 (success, general error, invalid args or auth required, rate-limited, not found, network error).
- Add TTY-behavior note to `xr --help` explaining that piping to a non-TTY strips color and suppresses human-only banners without extra flags.
- `xr skill install [<host>] [--all] [--dry-run] [--output json]` to install the `AGENTS.md` bundle into per-host skills directories. Supports `claude_code`, `codex`, `cursor`, `factory`, `kiro`, and `opencode`. by @brettdavies in [#36](https://github.com/brettdavies/xurl-rs/pull/36)
- Wire `--timeout` / `XURL_TIMEOUT` through `ApiClient`, OAuth2 token exchange + refresh, and the `/2/users/me` lookup so the flag bounds every HTTP request. Default stays 30 seconds. by @brettdavies in [#37](https://github.com/brettdavies/xurl-rs/pull/37)
- SIGTERM and SIGINT now cancel the OAuth2 callback listener and any active streaming request. Streaming emits a `{"status":"cancelled","reason":"sigterm"}` envelope under `--output json` / `--jsonl` and returns exit code 0.
- Add global `--json` and `--jsonl` flags on the root command as shorthands for `--output json` and `--output jsonl`; both conflict with `--output` and with each other, resolved by `Cli::effective_output()`. by @brettdavies in [#38](https://github.com/brettdavies/xurl-rs/pull/38)
- Add global `--raw` flag (env `XURL_RAW`, falsey-value parser) that emits compact JSON in `json`/`jsonl` modes and strips ANSI styling in text mode; threaded through `OutputConfig::new_with_raw`.
- Add `xurl::envelope::Envelope`, a `#[derive(JsonSchema)]` enum modeling the three response shapes (`ok`, `dry_run`, `error`) every JSON output path serializes into.
- Add `xr schema envelope` and `xr schema --envelope` subcommands that emit the canonical agent-native envelope JSON Schema (Draft 2020-12) for downstream pinning.
- Add `schema/output.schema.json` at the repo root as the pinned envelope schema snapshot, with a drift guard in `tests/schema_tests.rs` that asserts byte-equality against the runtime emitter.
- Add `OutputConfig::print_success(out, payload)` and `OutputConfig::print_dry_run(out, would_succeed, exit_code, ctx)` as the canonical envelope writers alongside `print_error`; object payloads flatten into the envelope for verb context fields.
- Add an `envelope` row to `xr schema --list` so consumers can discover the envelope schema alongside the 29 typed response commands.
- Global `--force` flag (with confirmation gate) on `xr delete`, `xr auth clear`, and `xr auth apps remove`. Under `--no-interactive`, the flag is mandatory; calling the op without it returns `{"status":"error","reason":"confirmation-required","exit_code":1}` on stderr and never touches the API. Under a TTY without `--force`, dialoguer prompts the operator. by @brettdavies in [#39](https://github.com/brettdavies/xurl-rs/pull/39)
- Global `--dry-run` flag (env-backed via `XURL_DRY_RUN`) on every write op: post, reply, quote, delete, like, unlike, repost, unrepost, bookmark, unbookmark, follow, unfollow, block, unblock, mute, unmute, dm, media upload, app add/update/remove, redirect-uri set, and every `xr auth` subcommand. The envelope shape is `{"status":"dry_run","would_succeed":bool,"reason":"<kebab>","exit_code":int,"command":"<verb>",…}`. Pre-flight validators catch `empty-body`, `body-too-long`, `too-many-attachments`, `empty-username`, and `empty-post-id` so the dry-run path predicts failure before any HTTP call.
- Global `--limit <n>` flag (env-backed via `XURL_LIMIT`), clamped to `1..=100`. Applies to search, timeline, mentions, bookmarks, likes, following, followers, and dms. Per-command `-n/--max-results` keeps precedence when both are set.
- Add `OutputConfig::verbose`, `OutputConfig::warning`, and `OutputConfig::progress` helpers so library consumers can emit diagnostics that respect `--verbose`, `--quiet`, `--format json`, and TTY detection. by @brettdavies in [#40](https://github.com/brettdavies/xurl-rs/pull/40)
- Add `scripts/lint-stdio.sh` to fail the build when any `println!`, `eprintln!`, `print!`, or `eprint!` macro appears outside `src/output.rs`.
- Add a `lint-stdio` job to `.github/workflows/ci.yml` that runs the guard on every push and pull request.
- Back `--json` and `--jsonl` with the `XURL_JSON` and `XURL_JSONL` environment variables, parsed via `FalseyValueParser` so `0`, `false`, and empty values disable the flag. by @brettdavies in [#41](https://github.com/brettdavies/xurl-rs/pull/41)
- Introduce the `xr skill update [<host>] [--all] [--dry-run] [--output json]` subcommand, which removes the existing destination and re-runs the install pipeline; the JSON envelope `action` field is `skill-update` so consumers can tell update apart from first-time install.
- Advertise `XURL_JSON`, `XURL_JSONL`, and `XURL_NO_BROWSER` in the `ENVIRONMENT VARIABLES` block of `xr --help`.
- Add global `--no-pager` flag (env `XURL_NO_PAGER`) accepted as a documented no-op so agents and wrappers that reflexively pass `--no-pager` no longer get rejected. by @brettdavies in [#42](https://github.com/brettdavies/xurl-rs/pull/42)
- `XURL_NO_BROWSER` env var for `xr auth oauth2 --no-browser` so headless and CI runners can set the headless flow by default. by @brettdavies in [#43](https://github.com/brettdavies/xurl-rs/pull/43)
- `xr auth oauth2` auto-engages the headless (remote-step-1) flow when stdout is not a TTY and `--no-browser` / `XURL_NO_BROWSER` is unset, emitting `{"status":"awaiting_callback","url":"..."}` on stdout instead of attempting to open a browser.
- Extend `--output` to accept `csv`, `tsv`, `yaml`, and `ndjson` (a strict alias of `jsonl`) so agents trained on delimited or YAML conventions can pipe `xr` output through existing toolchains. by @brettdavies in [#46](https://github.com/brettdavies/xurl-rs/pull/46)
- Emit a `_warning` flattening column on `csv` and `tsv` rows whose nested object or array cells got JSON-stringified, so consumers detect lossy flattening without parsing every cell.
- Add a global `--cursor <token>` flag (env `XURL_CURSOR`) plus an `--after` synonym (env `XURL_AFTER`) that threads `pagination_token` into search, timeline, mentions, bookmarks, likes, following, followers, dms, and raw-mode list URLs.
- URL-encode the cursor token via `form_urlencoded::byte_serialize` on every list shortcut so exotic upstream tokens round-trip back to the X API unchanged.
- Reject `--page <n>` (env `XURL_PAGE`) with the canonical `{status:"error",reason:"unsupported-pagination",exit_code:1,...}` envelope on stderr and exit non-zero before any API call, turning a silent no-op into a structured signal for agents trained on offset pagination.
- Add the `xr validate` subcommand that reads JSON from stdin (no file arg or `-`) or a file path and validates it against bundled response schemas (`tweet`, `tweets`, `user`, `users`, `dm`, `dms`, `usage`, `envelope`, plus the small fixed-shape result types).
- Auto-detect the validation schema from the document's top-level shape when `--schema` is omitted, or pin a target type explicitly via `--schema <name>` for stricter agent pipelines.
- Emit `{status:"ok",schema:...,valid:true}` on validation success and `{status:"error",reason:"validation-failed",...}` on failure, with exit code 1 on validation failure and exit code 2 on argument errors.
- Document the stdin convention in a new `INPUT FROM STDIN` block on the root `--help` so agents discover the no-file-arg input pattern from the top-level help text.
- Document `toml` and `xml` as unsupported `--output` values in the help text so substring searches over `xr --help` surface the complete token catalog.
- `schema/responses/<command>.schema.json` for every entry in `SCHEMA_ENTRIES` (35 files at v1.3.0): the 14 existing typed shortcut commands plus `auth-status`, `auth-apps-list`, `redirect-uri-get`, `redirect-uri-set`, `skill-install`, and `skill-install-all`. Downstream agents and CI consumers can pin against the on-disk JSON Schemas without invoking `xr`. by @brettdavies in [#48](https://github.com/brettdavies/xurl-rs/pull/48)
- `scripts/generate-response-schemas.sh` regenerates the per-command files by enumerating commands via `xr schema --list` and dumping each via `xr schema <cmd> --output json`.
- Typed `RedirectUriGetResponse` and `RedirectUriSetResponse` for the `auth apps redirect-uri get` / `set` JSON output paths, replacing the prior ad-hoc `serde_json::json!()` shapes so the schema is derivable.
- `XURL_BEARER_TOKEN` env var now feeds `Auth::get_bearer_token_header`, so one-shot agent flows can pipe a bearer through the environment without first running `xr auth app --bearer-token`. Resolution order is env-supplied bearer first, then the resolved app's stored bearer; empty env values fall through. Matches the precedence shape of every other agentic flag (`XURL_VERBOSE`, `XURL_OUTPUT`, `XURL_NO_BROWSER`). by @brettdavies in [#51](https://github.com/brettdavies/xurl-rs/pull/51)
- `xurl::auth::resolve_bearer_token` public free function factored out of `get_bearer_token_header` so library consumers and unit tests can exercise every precedence branch without touching the process environment.
- First successful authentication against a named app now promotes that app to default when the previous default holds no credentials of any kind. Applies to `xr auth oauth2 --app NAME`, `xr auth oauth1 --app NAME`, and `xr auth app --bearer-token … --app NAME`. Subsequent invocations resolve NAME without an explicit `--app`, matching the "first signed-in app becomes default" UX. Users who want a different default later can still run `xr auth default <name>` explicitly; this auto-promote helper only fires when the existing default's `oauth2_tokens`, `oauth1_token`, `bearer_token`, and `unnamed_oauth2_token` are all empty. by @brettdavies in [#52](https://github.com/brettdavies/xurl-rs/pull/52)
- `TokenStore::promote_to_default_if_first_credentialed(candidate)` public helper that encapsulates the promotion contract. Idempotent no-op on already-credentialed defaults, unknown candidates, empty candidate names, and self-referential calls.
- `TokenStore::default_app_is_uninitialized` predicate underpinning the promotion check.

### Changed

- Bumped declared `rust-version` from `1.85` to `1.94` to match the pinned toolchain, closing the MSRV gap between the promise to library consumers and the code that actually ships. by @brettdavies in [#28](https://github.com/brettdavies/xurl-rs/pull/28)
- Reshape `OutputConfig` print methods (`info`, `status`, `print_response`, `print_stream_line`, `print_error`, `print_message`) to accept `&mut dyn Write` at the call site, leaving `OutputConfig` as a pure `Send + Sync + Clone` configuration struct with no I/O state. by @brettdavies in [#29](https://github.com/brettdavies/xurl-rs/pull/29)
- Rename leaf formatters in `src/api/response/format.rs` to `format_response`, `colorize_json`, `write_colorized_value`, and `write_colorized_value_ln`; the public re-export in `src/api/response/mod.rs` updates accordingly.
- `auth status` text output now includes a `redirect_uri:` line per app showing the effective URI and source label. When the env var overrides a stored value, a `stored_redirect_uri:` line surfaces the stored value. by @brettdavies in [#30](https://github.com/brettdavies/xurl-rs/pull/30)
- `auth apps list` text output includes the same inline redirect URI hint per app row.
- The OAuth2 callback listener now binds the host, port, and path from the resolved redirect URI rather than the hardcoded `127.0.0.1:8080/callback`.
- Path matching on the callback listener tightens from `starts_with` to exact-or-querystring, so a custom redirect URI like `/oauth/return` no longer matches unrelated prefixes.
- Test failures inside `src/api/response/types.rs`, `src/config/mod.rs`, and `src/auth/callback.rs` now panic with named subjects (e.g., "Tweet response must deserialize") instead of generic `unwrap()` messages. by @brettdavies in [#33](https://github.com/brettdavies/xurl-rs/pull/33)
- `-v/--verbose` is now a single root-level global flag and applies to every subcommand. The duplicate `-v` definitions on `CommonFlags`, `media upload`, and `media status` are removed. by @brettdavies in [#34](https://github.com/brettdavies/xurl-rs/pull/34)
- BREAKING: change `EXIT_AUTH_REQUIRED` from `2` to `77` (`EX_NOPERM` from sysexits); the previous value collided with `EX_USAGE`, so clap parse errors keep `2` and auth-required failures now exit unambiguously. Downstream scripts branching on the auth exit code must update. by @brettdavies in [#38](https://github.com/brettdavies/xurl-rs/pull/38)
- Reshape error JSON under `--output json|jsonl` from the prior `{"error","kind","code"}` triple to the canonical envelope `{"status":"error","reason":<kebab>,"exit_code":<int>,"message":<str>}` on every output path.
- Promote `exit_code_for_error(&XurlError)` and the private `error_kind(&XurlError)` to `XurlError::exit_code()` and `XurlError::kind()` methods, re-exported via `xurl::error`; the free function is preserved as a thin shim so existing call sites keep compiling.
- Narrow `XurlError::kind()` to a closed kebab-case set (`auth-required`, `rate-limited`, `not-found`, `network-error`, `invalid-args`, `invalid-method`, `validation`, `serialization`, `io`, `token-store`) so agents pattern-match on a stable identifier.
- Switch the runner from `Cli::parse_from` to `Cli::try_parse_from` so `DisplayHelp`, `DisplayVersion`, and `DisplayHelpOnMissingArgumentOrSubcommand` exit `0` to stdout, and other parse errors route through envelope emission on stderr when JSON intent is present.
- Emit the `invalid-args` envelope on clap parse failures when JSON intent is detected in the unparsed argv (`--json`, `--jsonl`, `--output json|jsonl`, `--output=json|jsonl`) or in `XURL_OUTPUT`; otherwise preserve clap's text rendering.
- Bump crate version from `1.2.0` to `1.3.0` in `Cargo.toml` and `Cargo.lock`.
- `OutputConfig` gains `print_dry_run` and `print_confirmation_required` helpers so any handler can emit the canonical envelope in one call. by @brettdavies in [#39](https://github.com/brettdavies/xurl-rs/pull/39)
- `XurlError::EnvelopeAlreadyEmitted { exit_code }` is the new sentinel that lets the runner suppress its trailing `print_error` when a typed envelope was already written; agents see exactly one JSON object on stderr.
- `src/api/shortcuts` is now `pub` so the new validators are importable from the command handlers without re-exports.
- Suppress verbose request/response logs, streaming connection banners, OAuth salvage warnings, `.twurlrc` import warnings, and `REDIRECT_URI` rejection warnings when `--format json` or `--quiet` is set, so machine-readable output stays clean. by @brettdavies in [#40](https://github.com/brettdavies/xurl-rs/pull/40)
- Colorize verbose request and response lines only when `OutputConfig::use_color` is enabled, matching the existing color-detection rules for other output.
- Tighten `ROOT_HELP` wording to reference the new `--no-pager` flag and reaffirm that `xr` writes directly to stdout and stderr and never invokes `$PAGER`. by @brettdavies in [#42](https://github.com/brettdavies/xurl-rs/pull/42)
- `xr auth default` without an app argument now gates the dialoguer picker on `--no-interactive` AND stdin/stderr TTY-ness. Non-TTY sessions exit non-zero with a `{"status":"error","reason":"no-tty","exit_code":1,...}` envelope on stderr instead of stranding the dialoguer state machine on `/dev/null`. by @brettdavies in [#43](https://github.com/brettdavies/xurl-rs/pull/43)
- `xr auth oauth2 --no-browser` (no `--step`) now auto-promotes to step 1 and emits the canonical `{"status":"awaiting_callback","url":"..."}` envelope, instead of rejecting the invocation as "requires --step 1 or --step 2". The explicit `--no-browser --step 1` path keeps its prior envelope shape.
- Drop the `dialoguer` dependency from `xurl-rs`, shrinking the dep tree by removing `console`, `shell-words`, `encode_unicode`, and `unicode-width` transitives. by @brettdavies in [#45](https://github.com/brettdavies/xurl-rs/pull/45)
- Replace the `Select` prompts in `xr auth default` with a stdin-based numbered picker that writes choices to stderr and reads one line from stdin.
- Replace the `Confirm` prompt in `confirm_destructive` with a stdin `[y/N]` reader that accepts `y`/`yes` (case-insensitive) and defaults to `false` on EOF or any other input.
- Route every printer in `src/output.rs` (including `print_response`, `print_error`, `print_error_envelope`, `print_success`, `print_dry_run`, `print_confirmation_required`, `print_message`, `info`, `status`, `warning`, `verbose`, `progress`, `print_stream_line`) through `OutputFormat::is_structured()` so the new formats inherit the machine-readable suppression rules and stderr discipline that `json` and `jsonl` already enforce. by @brettdavies in [#46](https://github.com/brettdavies/xurl-rs/pull/46)
- Extend `RequestOptions` and `CallOptions` with a `pagination_token` field defaulting to an empty string so library consumers can thread cursors without breaking non-paginated callers.
- `AppStatusEntry`, `InstallEnvelope`, `InstallMultiEnvelope`, and `ResolveSource` now derive `schemars::JsonSchema`. `AppStatusEntry` is `pub(crate)` so the `xr schema` module can reference it. by @brettdavies in [#48](https://github.com/brettdavies/xurl-rs/pull/48)
- `xr skill install` and `xr skill update` now clone from `brettdavies/xurl-rs-skill` instead of `brettdavies/xurl-rs`. The skill bundle (`AGENTS.md`, fixtures, templates, evals, scripts) lives in its own repo so the install path no longer pulls the entire `xurl-rs` source tree. Install directories (`~/.claude/skills/xurl-rs/`, `~/.codex/skills/xurl-rs/`, …) stay the same; only the source URL moves. by @brettdavies in [#49](https://github.com/brettdavies/xurl-rs/pull/49)
- `.anc.toml` `domain_verbs` declares the three xr-specific surface verbs (`media`, `examples`, `validate`) alongside the X platform vocabulary, so the `p6-may-standard-names` audit recognizes the full xr command surface once upstream anc-cli honors per-CLI domain vocab. by @brettdavies in [#50](https://github.com/brettdavies/xurl-rs/pull/50)
- `xurl::auth::resolve_bearer_token` now accepts an `app_name: &str` parameter so library consumers can scope the resolution to a specific app rather than the default. Library callers that were using the previous two-argument signature pass `""` for the legacy default-app behavior. by @brettdavies in [#52](https://github.com/brettdavies/xurl-rs/pull/52)

### Fixed

- Bump `rand` to 0.9.4 to clear RUSTSEC-2026-0097 (`ThreadRng` unsoundness when accessed from a custom `log::Logger`). by @brettdavies in [#25](https://github.com/brettdavies/xurl-rs/pull/25)
- Bump `rustls-webpki` to 0.103.13 to clear RUSTSEC-2026-0104 (reachable panic in CRL `IssuingDistributionPoint` parsing, triggerable before signature verification).
- Resolved a private-intra-doc-link in `src/auth/pending.rs` that broke `cargo doc -D warnings` and would render badly on docs.rs. by @brettdavies in [#28](https://github.com/brettdavies/xurl-rs/pull/28)
- Stop wrapping JSON schema bodies in `{"message": "..."}` under `--output json` for `xr schema <command>` and `xr schema --all`, so the output stays valid JSON for downstream consumers. by @brettdavies in [#29](https://github.com/brettdavies/xurl-rs/pull/29)
- Route OAuth2 browser-launch-failure messages through `OutputConfig::print_message` so they respect the configured stderr writer rather than going straight to process stdout.
- Browsers that resolve `localhost` to `[::1]` no longer time out during the OAuth2 flow. The listener dual-binds both loopback addresses when the URI host is `localhost`. by @brettdavies in [#30](https://github.com/brettdavies/xurl-rs/pull/30)
- The browser cannot be opened before the callback listener is actively draining the accept queue, eliminating a race on fast machines.
- Invalid redirect URIs are rejected at write time. `http` is allowed only on loopback hosts; non-loopback `http` and other schemes return a validation error.
- `auth status` and `auth apps list` previously read `~/.xurl` directly through a fresh `TokenStore::new()`, bypassing the runner's configured store path. Both now use `&auth.token_store`, which makes tempdir isolation reliable for library-level tests.
- Stop leaking streaming status messages such as `Connecting to streaming endpoint` and `--- Streaming response started ---` to stderr under JSON mode. by @brettdavies in [#40](https://github.com/brettdavies/xurl-rs/pull/40)
- Clear `XURL_DRY_RUN` before spawning `xr` in U9 subprocess tests so parallel test scheduling cannot leak the var and flake the awaiting_callback assertions. by @brettdavies in [#44](https://github.com/brettdavies/xurl-rs/pull/44)
- `tests/schema_tests.rs::committed_response_schemas_match_runtime` drift guard asserts byte-equality between every `schema/responses/*.schema.json` and the runtime-emitted shape; CI fails on drift with the regen command in the error message. by @brettdavies in [#48](https://github.com/brettdavies/xurl-rs/pull/48)
- `xr <subcommand> --app NAME` now correctly resolves `client_id` and `client_secret` to NAME's stored values across `Auth::with_app_name` and `Auth::with_token_store` switches. The previous "preserve if non-empty" check could not distinguish env-supplied values from values copied off the previous app's store entry during `Auth::new_with_store_path`, so `--app NAME` switches silently re-used the previous app's stored `client_id`. The user-facing symptom was an OAuth2 step-1 URL carrying the wrong `client_id`, yielding a 401 from X. `Auth` now tracks origin via two new `client_id_from_env` and `client_secret_from_env` fields, and `with_token_store` drops the older `self.client_id == old_app.client_id` equality heuristic. by @brettdavies in [#51](https://github.com/brettdavies/xurl-rs/pull/51)
- `xr <subcommand> --app NAME --auth oauth2` could send the request unauthenticated and surface upstream 401s instead of the freshly minted token. `oauth2::refresh_oauth2_token` looked up the cached token via `get_first_oauth2_token()` (no arg), which resolved to the empty-string app name and so `resolve_app` fell back to the default app. A token saved under NAME via `xr auth oauth2 --app NAME` was invisible to the refresh path, the token lookup returned None, the request layer silently skipped the Authorization header (see below), and X returned 401 that masqueraded as a token rejection. Both lookups now use `app_name_lookup` consistently and the named-username branch routes through `get_oauth2_token_for_app`. by @brettdavies in [#52](https://github.com/brettdavies/xurl-rs/pull/52)
- `xr <subcommand> --app NAME --auth oauth1` ignored `--app NAME` and read the default app's OAuth1 credentials. `Auth::get_oauth1_header` now calls `get_oauth1_tokens_for_app(&self.app_name)`. Without this, a token saved under NAME by `xr auth oauth1 --app NAME` was unreachable for subsequent invocations targeting NAME.
- `xr <subcommand> --app NAME --auth app` ignored `--app NAME` and read the default app's bearer. `resolve_bearer_token` now takes an `app_name` parameter and uses `get_bearer_token_for_app`; the internal `Auth::get_bearer_token_header` threads `self.app_name`.
- `xr <subcommand> --app NAME` without an explicit `--auth` flag (auto-detect) probed default-app credentials first via `get_first_oauth2_token`, `get_oauth1_tokens`, and `has_bearer_token` (all no-arg). The auto-detect path now scopes every probe to the active app via `*_for_app(&self.auth.app_name())`.
- `xr auth oauth1 --app NAME …` saved the token to the default app instead of NAME because the CLI handler called `save_oauth1_tokens` (no arg). Now routes through `save_oauth1_tokens_for_app(&candidate)` using the active app name.
- `xr auth app --bearer-token … --app NAME` saved the bearer to the default app instead of NAME. Now routes through `save_bearer_token_for_app(&candidate)`.
- `ApiClient::send_request` (and the multipart and streaming siblings) now propagate auth-resolution errors instead of silently sending the request without an Authorization header. The older `if let Ok(auth) = ...` form let any `XurlError::Auth` from `get_auth_header` disappear into a request that went to X without credentials, which X then rejected with 401. The 401 looked identical to "X rejected our token" even though we never sent one. The new `?` propagation surfaces the real `auth-required` envelope with the token-not-found `message`.

### Documentation

- Document the `docs/solutions` archive and `qmd query` retrieval in `AGENTS.md` so agent consumers can find it from the onboarding doc. by @brettdavies in [#26](https://github.com/brettdavies/xurl-rs/pull/26)
- Harden `RELEASES.md` with triple-diff verification, prose scrubbing, the CHANGELOG generation contract and `cliff.toml` chore-skip footgun, non-draft release behavior, and branch-protection rulesets.
- Rewrite `RELEASES.md` as a pure operational runbook with the branch table, daily-dev flow, PR-body rules, dev to main cherry-pick procedure, tagging table, and prose-scrub procedure; rationale moves out via arrow-links. by @brettdavies in [#27](https://github.com/brettdavies/xurl-rs/pull/27)
- Document new CLI surface (per-app redirect URI, `-u USERNAME` fallback, structured output) in README by @brettdavies in [#32](https://github.com/brettdavies/xurl-rs/pull/32)
- Document X platform "Pay-per-use Production" enrollment workaround in README
- Document the headless OAuth2 flow on `xr auth oauth2 --help` with `--no-browser --step 1` and `--step 2 --auth-url '<paste>'` invocations for servers, containers, and SSH sessions. by @brettdavies in [#35](https://github.com/brettdavies/xurl-rs/pull/35)
- Document env-var and flag precedence on `xr search --help` via the `XURL_OUTPUT=json xr search "rustlang" -n 25` example.
- Document the envelope variants, the closed `reason` vocabulary, and the regeneration command for `schema/output.schema.json` in the `xurl::envelope` module rustdoc. by @brettdavies in [#38](https://github.com/brettdavies/xurl-rs/pull/38)
- Update `xr schema --list` rustdoc and the `xr schema` usage string to advertise the new `envelope` row and `xr schema envelope` invocation.
- Note in root `xr --help` that `xr` writes directly to stdout and stderr and never invokes `$PAGER`, so output is pipe-safe by default and no `--no-pager` flag is required. by @brettdavies in [#41](https://github.com/brettdavies/xurl-rs/pull/41)
- Add 11 output-writer tests, 8 agentic subprocess tests, and 2 wiremock pagination tests covering each new format, the cursor query-parameter threading and URL encoding, the unsupported-pagination envelope, and ok and fail paths for `xr validate`. by @brettdavies in [#46](https://github.com/brettdavies/xurl-rs/pull/46)
- Split the bearer preflight gate in `RELEASES-PREFLIGHT.md` into two subgates so the env-var one-shot path (`XURL_BEARER_TOKEN=… xr <shortcut> --auth app`) and the stored two-step path (`xr auth app --bearer-token` then `--auth app`) are both explicitly verified before tagging. The single combined line previously documented only the env-var inline form, which the bearer resolver did not honor until this PR. by @brettdavies in [#51](https://github.com/brettdavies/xurl-rs/pull/51)
- Add a Multi-app credential routing section to RELEASES-PREFLIGHT.md with seven gates covering OAuth1, OAuth2, and bearer save/read isolation across at least two registered apps, auto-detect with `--app NAME`, the first-signed-in-app auto-default behavior on every sign-in handler, promotion idempotence, and the auth-error envelope vs upstream 401 distinction. Replaces the implicit assumption that one default app sufficed for live smoke. by @brettdavies in [#52](https://github.com/brettdavies/xurl-rs/pull/52)

### Tests

- 11 red team tests in `tests/auth_tests.rs`: 7 cover bearer resolution (env-only, env-overrides-store, env-empty falls through, env-unset falls through, neither-set errors, env-empty + store-empty errors, real-env integration via `std::env::set_var` behind an `unsafe` guard mirroring `test_env_redirect_uri_wins_over_app_stored`); 4 cover app-switch precedence (env-supplied `client_id` survives a switch, store-derived `client_id` re-resolves to new app's stored value, round-trip default to bird-dev to default re-resolves correctly, `client_secret` follows the same contract). by @brettdavies in [#51](https://github.com/brettdavies/xurl-rs/pull/51)
- `tests/auth_tests.rs`: 3 red team tests for OAuth2 refresh-context (named app with named username, named app with empty username uses default_user precedence, salvage-state unnamed slot resolution). by @brettdavies in [#52](https://github.com/brettdavies/xurl-rs/pull/52)
- `tests/auth_tests.rs`: 6 multi-app tests covering OAuth1 + bearer + OAuth2 across two apps and runtime `--app NAME` switching, plus env-var precedence over per-app stores.
- `tests/api_tests.rs`: 2 tests asserting `XurlError::Auth` propagates from `ApiClient::get_me` when no token is in the store for the requested auth path, instead of falling through to a wiremock-absent network error.
- `tests/store_tests.rs`: 7 tests covering the promotion contract: promotes on uninitialized default, no-op when default already has bearer (or any other credential), no-op on unknown candidate, on empty candidate, on candidate==default, treats unnamed salvage tokens as "credentialed", and the underlying `default_app_is_uninitialized` signal.

**Full Changelog**: [v1.2.0...v1.3.0](https://github.com/brettdavies/xurl-rs/compare/v1.2.0...v1.3.0)

## [1.2.0] - 2026-04-16

### Added

- Add `ApiClient::from_env()` one-liner constructor that reads `CLIENT_ID`/`CLIENT_SECRET` from environment variables by @brettdavies in [#21](https://github.com/brettdavies/xurl-rs/pull/21)
- Add `CallOptions` consumer-facing struct for shortcut methods, exposing only `auth_type`, `username`, `no_auth`, `verbose`, `trace`
- Add `no_auth` field to skip authentication entirely on a per-request basis
- Add `XurlError::Validation(String)` variant for non-HTTP validation errors (e.g., errors-only 200 responses)

### Changed

- Change `ApiClient` from borrowed `&mut Auth` to owned `Auth` — no lifetime parameter, storable in structs by @brettdavies in [#21](https://github.com/brettdavies/xurl-rs/pull/21)
- Change 29 shortcut functions from free functions (`api::create_post(&mut client, ...)`) to methods (`client.create_post(...)`)
- Change `XurlError::Api(String)` to `Api { status: u16, body: String }` for structured HTTP error matching
- Change `exit_code_for_error()` to pattern-match on `Api { status, .. }` directly instead of string matching

### Fixed

- Bump `rustls-webpki` to 0.103.12 to clear [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098) and [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099) (name constraint validation). by @brettdavies in [#23](https://github.com/brettdavies/xurl-rs/pull/23)

### Documentation

- Document exit code mapping improvement as known difference from Go version in KNOWN_DIFFERENCES.md by @brettdavies in [#21](https://github.com/brettdavies/xurl-rs/pull/21)

**Full Changelog**: [v1.1.0...v1.2.0](https://github.com/brettdavies/xurl-rs/compare/v1.1.0...v1.2.0)

## [1.1.0] - 2026-04-02

### Added

- Add `xr usage` shortcut command that returns full API usage data (tweet caps, daily project breakdown, per-app breakdown) by @brettdavies in [#13](https://github.com/brettdavies/xurl-rs/pull/13)
- Add `--remote` flag for headless OAuth2 authentication on machines without a browser by @brettdavies in [#14](https://github.com/brettdavies/xurl-rs/pull/14)
- Add `--step` (1 or 2) and `--auth-url` (with `-` for stdin) companion flags
- Add JSON output support for step 1 (`--output json` emits `{"auth_url": "...", "instructions": "..."}`)
- Add typed response structs: `Tweet`, `User`, `DmEvent`, `UsageData`, 7 action confirmations, 3 wrapper/meta types, 3 nested types by @brettdavies in [#17](https://github.com/brettdavies/xurl-rs/pull/17)
- Add `deserialize_response<T>()` helper with guards for empty and errors-only 200 responses
- Add `ApiResponse<T>` generic wrapper with `data`, `includes`, `meta`, `errors`, and forward-compatible `extra` fields
- Add `xr schema <command>` to output JSON Schema for any command's response type by @brettdavies in [#18](https://github.com/brettdavies/xurl-rs/pull/18)
- Add `xr schema --list` to show all 29 commands with their response types
- Add `xr schema --all` to output all schemas as a single JSON document
- Add `schemars` dependency for compile-time JSON Schema generation via `#[derive(JsonSchema)]`

### Changed

- Rename `--remote` to `--no-browser` for the headless OAuth2 authentication flow by @brettdavies in [#15](https://github.com/brettdavies/xurl-rs/pull/15)
- Change all 29 shortcut functions from `Value` returns to typed `ApiResponse<T>` returns (**breaking** for library consumers) by @brettdavies in [#17](https://github.com/brettdavies/xurl-rs/pull/17)

### Fixed

- Rename completion files to standard convention (`xr.zsh`, `xr.elvish`, `xr.powershell`) by @brettdavies in [#11](https://github.com/brettdavies/xurl-rs/pull/11)
- Regenerate bash and fish completions for completions subcommand
- Fix test isolation in `Auth::with_token_store()` where real `~/.xurl` credentials leaked into test assertions by @brettdavies in [#12](https://github.com/brettdavies/xurl-rs/pull/12)

### Documentation

- Add shell completions regeneration step to the release process as a safety net for missed completions during development by @brettdavies in [#13](https://github.com/brettdavies/xurl-rs/pull/13)

**Full Changelog**: [v1.0.5...v1.1.0](https://github.com/brettdavies/xurl-rs/compare/v1.0.5...v1.1.0)

## [1.0.5] - 2026-03-21

### Added

- `xr completions <shell>` subcommand replacing hidden `--generate-completion` flag by @brettdavies in [#6](https://github.com/brettdavies/xurl-rs/pull/6)
- PowerShell and Elvish shell completions by @brettdavies in [#6](https://github.com/brettdavies/xurl-rs/pull/6)
- cargo-deny license and advisory auditing via `deny.toml` by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- Draft-then-finalize release pattern via `finalize-release.yml`
- Commit provenance guard requiring PRs go through dev before main
- Changelog CI enforcement — PRs to main must include CHANGELOG.md updates
- `cargo binstall` support for pre-built binary installs

### Changed

- Version and completions commands now exit before config/auth initialization by @brettdavies in [#6](https://github.com/brettdavies/xurl-rs/pull/6)
- CI/CD migrated to centralized reusable workflows from `brettdavies/.github` by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- `ci.yml`, `release.yml`, `guard-main-docs.yml` replaced with thin callers
- Homebrew dispatch secret migrated from `HOMEBREW_TAP_TOKEN` to `CI_RELEASE_TOKEN`
- Release archives now include completions, licenses, README, and sha256sums

### Fixed

- `protect-dev.json` aligned with bird (add deletion, non_fast_forward rules, admin bypass) by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- Update `rustls-webpki` to 0.103.10 to fix [RUSTSEC-2026-0049](https://rustsec.org/advisories/RUSTSEC-2026-0049)
- Move `thiserror`, `anyhow`, `dirs`, `percent-encoding` to platform-independent `[dependencies]` — fixes Windows build by @brettdavies in [#9](https://github.com/brettdavies/xurl-rs/pull/9)

### Documentation

- RELEASING.md rewritten for reusable workflow pipeline and changelog-as-committed-artifact by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)

**Full Changelog**: [v1.0.4...v1.0.5](https://github.com/brettdavies/xurl-rs/compare/v1.0.4...v1.0.5)

## [1.0.4] - 2026-03-16

### Changed

- Switch to Trusted Publishing (OIDC) for crates.io authentication — no static secrets by @brettdavies in
  [#4](https://github.com/brettdavies/xurl-rs/pull/4)
- Pin all GitHub Actions by commit SHA for supply-chain security by @brettdavies in
  [#4](https://github.com/brettdavies/xurl-rs/pull/4)
- Switch reqwest from native-tls to rustls-tls for cross-compilation compatibility by @brettdavies in
  [#3](https://github.com/brettdavies/xurl-rs/pull/3)
- Update macOS CI runner from deprecated `macos-13` to `macos-latest` by @brettdavies in
  [#3](https://github.com/brettdavies/xurl-rs/pull/3)
- Opt into Node.js 24 for GitHub Actions (`FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`) by @brettdavies in
  [#4](https://github.com/brettdavies/xurl-rs/pull/4)

**Full Changelog**: [v1.0.3...v1.0.4](https://github.com/brettdavies/xurl-rs/compare/v1.0.3...v1.0.4)

## [1.0.3] - 2026-03-16

### Added

- Full xurl-rs implementation — Rust port of Go [xurl](https://github.com/xdevplatform/xurl) CLI by @brettdavies in
  [#2](https://github.com/brettdavies/xurl-rs/pull/2)
- 28 shortcut commands: post, reply, quote, delete, read, search, like, repost, bookmark, follow, block, mute, dm,
  timeline, mentions, whoami, and more
- Raw API mode: `xr /2/users/me`, `xr -X POST /2/tweets -d '{...}'`
- OAuth2 PKCE, OAuth1 HMAC-SHA1, and Bearer token authentication
- YAML token store with multi-app management at `~/.xurl`
- Media upload (chunked) with status polling
- Shell completions for bash, zsh, fish, powershell, elvish
- Agent-native features: `--output json/jsonl`, `--quiet`, `--no-interactive`, structured exit codes (0-5)
- `NO_COLOR` and `XURL_OUTPUT` environment variable support
- Release infrastructure: CI, cross-platform builds (5 targets), crates.io Trusted Publishing, Homebrew tap dispatch

### Fixed

- Switch reqwest from native-tls to rustls-tls for cross-compilation by @brettdavies in
  [#3](https://github.com/brettdavies/xurl-rs/pull/3)

### New Contributors

- @brettdavies made their first contribution in [#2](https://github.com/brettdavies/xurl-rs/pull/2)
