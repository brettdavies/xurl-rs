---
title: Webhooks Command Family - Plan
type: feat
date: 2026-10-08
status: planned
implementation: in progress
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
---

# Webhooks Command Family - Plan

**Target repo:** brettdavies/xurl-rs. Every path is in it.

---

## Goal Capsule

- **Objective:** `xr` manages X API webhooks and receives their events: a `webhooks` family over the thirteen webhook
  operations in the vendored spec, and `xr webhooks listen`, a local receiver that answers X's CRC check and prints each
  event as a JSON line.
- **Scope decided by Brett (2026-10-08):** webhooks before chat; the management commands plus `listen`; no built-in
  tunnel, so the caller brings the public URL.
- **Authority:** the Product Contract's R-IDs win on behavior, KTDs win on mechanism, and a unit overrides neither.
- **Execution profile:** inline, with no subagents, workers, or worktrees. One stack of PRs to `dev`. The X API is
  pay-per-use, so no unit adds a live case and no test sends a request to X.
- **Stop conditions:** stop and ask when a webhook endpoint's real behavior cannot be established from the spec, Go
  `xurl`, or X's documentation, and when a choice would add a dependency to `xdk-rs`.
- **Who finishes:** the implementer opens each PR and verifies its CI rollup; Brett merges every PR.

---

## Product Contract

### Summary

Go `xurl` 1.3.4 ships one webhook command, `xurl webhook start`: a local server behind an ngrok tunnel it opens itself,
which answers the CRC check and prints each POST body. It registers nothing with X. `xr` reaches every webhook endpoint
in raw mode and has no command, typed response, or mock route for any of them.

### Problem Frame

An agent that wants events without polling has to register a URL with X, subscribe an account or a stream to it, and run
something that answers X's challenge. With `xr` today that is thirteen hand-built raw requests and a receiver the agent
writes itself. Events are billed per delivery (`post.create` $0.005, `follow.*`, `dm.received`, and `chat.received`
$0.010 on X's pricing page as of 2026-10-07); the management calls have no row on that page.

### Requirements

- **R1.** Every operation under `/2/webhooks`, `/2/account_activity/`, and `/2/tweets/search/webhooks` has a shortcut, a
  typed response, a fixture, a mock route, and one `xr webhooks` verb.
- **R2.** A verb that removes something (`remove` at each level) is destructive: it asks for confirmation, refuses
  without `--force` when it cannot ask, and answers `--dry-run` with `confirmation_required: true`.
- **R3.** Every verb answers `--dry-run` without sending or binding anything, with the credential its request would go
  out under.
- **R4.** `xr webhooks listen` binds a local address, answers `GET <path>?crc_token=` with the HMAC-SHA256 response
  token X documents, keyed by the active app's signing secret (KTD6), and prints each POST body whose signature verifies
  to stdout as one JSON line.
- **R5.** `listen` opens no tunnel and makes no request to X. It prints the local URL and says the caller exposes it.
- **R6.** `listen` is bounded on request: `--max-events <N>` exits 0 after N events, and it exits 0 on SIGINT.
- **R7.** Every new error carries a `reason` from the closed set and a `next_step`.
- **R8.** No test or CI job sends a request to X.

### Success Criteria

- `xr webhooks --help` lists every verb, and each verb's `--dry-run` passes `dry_run_guard.rs` with no new override.
- An in-process test posts a CRC challenge and an event to a bound receiver and reads the right response token and the
  event back.
- The recipe walks in `AGENTS.md` pass with the family added and no walk's test edited.

### Scope Boundaries

- **No `xr activity` family.** `/2/activity/*` stays in raw mode by the standing decision of 2026-10-05.
- **No tunnel.** No `ngrok` crate, no tunnel subcommand. The README names tools that provide a public URL.
- **No event schema.** `listen` prints the body X sent. Typing the event payloads is a later change if one is wanted.
- **No store change.** A registered webhook's id is not cached; `xr webhooks list` is the source.

---

## Planning Contract

### Command grammar

`webhooks` is an API namespace with sibling endpoints, so it is a family (`AGENTS.md` § Command grammar).

| Command                                                   | Endpoint                                                                       | Auth the spec lists    |
| --------------------------------------------------------- | ------------------------------------------------------------------------------ | ---------------------- |
| `xr webhooks list`                                        | `GET /2/webhooks`                                                              | bearer, OAuth2, OAuth1 |
| `xr webhooks add <url>`                                   | `POST /2/webhooks`                                                             | bearer, OAuth2, OAuth1 |
| `xr webhooks validate <webhook_id>`                       | `PUT /2/webhooks/{webhook_id}`                                                 | bearer, OAuth2, OAuth1 |
| `xr webhooks remove <webhook_id>`                         | `DELETE /2/webhooks/{webhook_id}`                                              | bearer, OAuth2, OAuth1 |
| `xr webhooks replay <webhook_id> --from T --to T`         | `POST /2/webhooks/replay`                                                      | bearer                 |
| `xr webhooks subscriptions count`                         | `GET /2/account_activity/subscriptions/count`                                  | bearer                 |
| `xr webhooks subscriptions list <webhook_id>`             | `GET /2/account_activity/webhooks/{webhook_id}/subscriptions/all/list`         | bearer                 |
| `xr webhooks subscriptions add <webhook_id>`              | `POST /2/account_activity/webhooks/{webhook_id}/subscriptions/all`             | OAuth2, OAuth1         |
| `xr webhooks subscriptions check <webhook_id>`            | `GET /2/account_activity/webhooks/{webhook_id}/subscriptions/all`              | OAuth2, OAuth1         |
| `xr webhooks subscriptions remove <webhook_id> <user_id>` | `DELETE /2/account_activity/webhooks/{webhook_id}/subscriptions/{user_id}/all` | bearer                 |
| `xr webhooks stream-links list`                           | `GET /2/tweets/search/webhooks`                                                | bearer                 |
| `xr webhooks stream-links add <webhook_id>`               | `POST /2/tweets/search/webhooks/{webhook_id}`                                  | bearer                 |
| `xr webhooks stream-links remove <webhook_id>`            | `DELETE /2/tweets/search/webhooks/{webhook_id}`                                | bearer                 |
| `xr webhooks listen`                                      | none                                                                           | OAuth1 consumer secret |

### Key Technical Decisions

- **KTD1. The recipe in `AGENTS.md` § "Adding a command family" is the build order.** Each operation gets a
  `SHORTCUT_TEMPLATES` row, a shortcut, a typed response derived from the spec's schema, a fixture with its `spec_*`
  test, a mock route, a clap verb through `family_help.rs`, a dispatch arm in a new `commands/webhooks.rs`, a schema
  registry row, a validate alias, an examples line, and completions. The walks name anything forgotten.
- **KTD2. Verbs are `list`, `add`, `remove`, `validate`, `check`, `count`, and `replay`.** `add` and `remove` follow
  `xr auth apps add` and `remove`. `validate` is the spec's own word for the `PUT` that makes X re-send its CRC check.
- **KTD3. A verb that sends one request goes through `send_or_report`,** so it gets `--dry-run`, the offline credential
  verdict, and the auth matrix's scheme selection with no code of its own. `remove` verbs use the confirmation path
  `delete` and `auth apps remove` use.
- **KTD4. `replay` takes X's own time format.** `--from` and `--to` are twelve digits, `yyyymmddhhmm` in UTC, as the
  spec's pattern requires. A value that does not match is refused offline with reason `validation`.
- **KTD5. The receiver is library code and prints nothing.** `AGENTS.md` puts network I/O in `xdk`. A
  `xdk::webhooks::Receiver` binds the address, answers CRC, and yields each event to its caller; `xr` prints. It is
  built on `tokio::net::TcpListener` the way `crates/xdk/src/auth/callback.rs` is, with `hmac`, `sha2`, and `base64`,
  which `xdk-rs` already depends on. No new dependency.
- **KTD6. The signing secret is the app's OAuth2 client secret, or its OAuth1 consumer secret.** X's webhook
  documentation (`docs.x.com/x-api/webhooks/introduction` and `/quickstart`, read 2026-10-08) says to compute the CRC
  `response_token` "with the OAuth 2.0 client secret when available, or with the OAuth 1.0 consumer secret for existing
  OAuth 1.0-only integrations", and never with a bearer token. `listen` takes the active app's client secret when it has
  one and its consumer secret otherwise; `--secret oauth2|oauth1` names one outright. An app with neither refuses
  `listen` with reason `auth-method-mismatch` and a `next_step`. Go `xurl` keys the CRC with the consumer secret alone,
  which `KNOWN_DIFFERENCES.md` records.
- **KTD7. `listen` binds `127.0.0.1` by default.** `--bind` takes another address. A receiver that listens on every
  interface by default would expose a port on the caller's network.
- **KTD8. Events are verified, as X documents.** Each POST carries `X-Twitter-Webhooks-Signature-OAuth2` (keyed by the
  client secret) or the legacy `X-Twitter-Webhooks-Signature` (keyed by the consumer secret), each
  `sha256=<base64 HMAC-SHA256>` over the raw body. The receiver checks the OAuth2 header when present and the legacy one
  otherwise, compares in constant time, answers 401 to a POST whose signature is missing or wrong, and yields only
  verified events. A refused POST is reported on `tracing` target `xdk::webhooks`. `--allow-unsigned` prints unverified
  POSTs too, for a local test with `curl`; Go `xurl` prints every POST unverified.
- **KTD10. `add` holds the URL rules X documents.** The documentation requires HTTPS and forbids a port in the URL.
  `validate_webhook_url` refuses a URL that breaks either, offline, beside the spec's length bounds.
- **KTD9. `listen` output is JSONL on stdout in every format.** One event per line, the body as X sent it, with a
  non-JSON body carried as a JSON string. The startup line (the local URL, and that the caller exposes it) goes to
  stderr in text output and is a first `{"status":"listening", ...}` line under a structured format.

### Risks & Dependencies

- **The mock cannot prove X accepts a request.** Typed responses and fixtures are held to the spec; whether X's live
  behavior matches the spec for these endpoints is not checked, by R8. The README says so.
- **`validate` and `add` make X call the registered URL.** Neither can succeed live unless a receiver is reachable. The
  examples page shows the order: `listen`, expose, `add`.
- **Two docs must stay in step.** The skill bundle documents the released binary and is updated after the release that
  carries this family.
- **Version.** New public functions and types in `xdk-rs` and new commands in `xr` are `### Added` in both changelogs.
  Both crates are already due a minor.

---

## Implementation Units

### U1. Library: the five `/2/webhooks` operations (#333)

- **Files:** `crates/xdk/build.rs`, `crates/xdk/src/api/shortcuts.rs`, `crates/xdk/src/api/response/types.rs`,
  `crates/xdk/src/testing/mod.rs`, `crates/xdk/tests/fixtures/openapi/example_responses.json`,
  `crates/xdk/tests/spec_validation.rs`.
- **Test first:** `mock_endpoint_coverage.rs` and `spec_validation.rs` fail for the declared endpoints before the routes
  and fixtures exist.

### U2. CLI: `xr webhooks list | add | validate | remove | replay` (#334)

- **Files:** `crates/xurl-cli/src/cli/mod.rs`, `family_help.rs`, a new `commands/webhooks.rs`, `commands/mod.rs`,
  `commands/schema.rs`, `commands/validate.rs`, `commands/examples.rs`, golden fixtures, generated schemas and
  completions.
- **Test first:** CLI tests against `MockX` for each verb, the `remove` confirmation refusal, and the `replay` time
  refusal, each seen failing.

### U3. Account Activity subscriptions (#336)

- **Work:** the five operations through U1's and U2's surfaces, as `xr webhooks subscriptions`.
- **Test first:** as U2, plus a dry run that shows `add` and `check` need a user login and `list`, `count`, and `remove`
  need a bearer.

### U4. Filtered-stream links

- **Work:** the three operations, as `xr webhooks stream-links`.

### U5. The receiver and `xr webhooks listen`

- **Files:** a new `crates/xdk/src/webhooks/` module, `crates/xdk/src/lib.rs`, `crates/xdk/src/error.rs` if a new
  variant is needed, `crates/xurl-cli/src/cli/commands/webhooks.rs`.
- **Work:** KTD5 to KTD10, R4 to R6.
- **Test first:** an in-process test binds the receiver on port 0, sends a CRC `GET` and reads the response token
  against a value computed by hand from a known secret; sends a signed POST and reads the event; sends a POST with a
  wrong signature and one with none and reads 401 with nothing yielded; sends a request to another path and reads a
  404. A CLI test runs `listen --max-events 1` as a child process and posts one signed event.

### U6. Documents

- **Files:** `crates/xurl-cli/README.md`, `crates/xdk/README.md`, `AGENTS.md`, `KNOWN_DIFFERENCES.md`.
- **Work:** the family, the `listen` flow with a caller-supplied public URL, the cost of delivered events, and the
  differences from Go: no tunnel, management verbs Go lacks, and KTD8's outcome.

---

## Verification Contract

- `cargo test --workspace --locked`, and `scripts/hooks/pre-push`.
- Each PR's CI rollup, including `Go parity`, completions freshness, the public-API semver gate, and the agent-native
  audit.
- No live request: `rg -n 'api.x.com' crates/*/tests` shows no new use.

## Definition of Done

- R1 to R8 hold, each shown by a test named in its unit.
- `xr webhooks --help` and every verb's help have golden fixtures, and the examples page invokes the family.
- `KNOWN_DIFFERENCES.md` records each difference from Go's `webhook start`.
