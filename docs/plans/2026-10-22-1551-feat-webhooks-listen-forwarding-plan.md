---
title: Webhooks Listen Forwarding and Restart Replay - Plan
type: feat
date: 2026-10-22
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
deepened: 2026-10-22
---

# Webhooks Listen Forwarding and Restart Replay - Plan

**Target repo:** brettdavies/xurl-rs. Every path is in it. **Base:** `dev`, which carries the webhooks family (#333
to #339, `docs/plans/2026-10-08-1745-feat-webhooks-family-plan.md`).

---

## Goal Capsule

- **Objective:** An agent running `xr webhooks listen` gets every verified X event into its own app, an HTTP endpoint or
  a local program or both, and after a restart recovers the events it missed, seeing before it pays what the recovery
  will request.
- **Means:** `listen` forwards from one ordered queue per target after it acknowledges X (KTD3), and records a
  checkpoint that `xr webhooks replay --from-checkpoint` turns into a replay window (KTD7, KTD8).
- **Authority:** R-IDs win on behavior, KTDs on mechanism, and a unit overrides neither. `AGENTS.md` § "Where a change
  goes" decides which crate a piece lands in (KTD1).
- **Execution profile:** inline in one session, no subagents or worktrees, test-first with each red observed. One PR per
  unit, stacked on `dev`.
- **Stop conditions:** a test would need a request to X (R15); a design would make `listen` send a request to X (R9); a
  change would rename or retype an existing envelope key.
- **Who finishes:** Brett reviews and merges every PR.

---

## Product Contract

### Summary

`xr webhooks listen` gains `--forward <URL>` with repeatable `-H` headers and a program after `--`, each fed every
verified event in delivery order. It gains `--checkpoint <FILE>`, which
`xr webhooks replay <ID> --from-checkpoint <FILE>` reads to request the gap after a restart. A taken port answers with a
step that reruns `listen` on a free port. The library gains each piece an embedder needs to do the same.

### Problem Frame

`listen` answers X's CRC check, verifies each event's signature, and prints it as a JSON line. Anything past that is the
caller's to build: an agent that wants events in its own app writes a relay from stdout, and that relay owns retries,
ordering, and what happens when its target is down. X documents no redelivery (its webhook and Account Activity pages
name the 10-second answer window and an hourly CRC, and no retry), so every event that arrives while `listen` is down is
lost unless someone notices and calls `/2/webhooks/replay` by hand with the right window. That call is bearer-only and
replayed deliveries are billed, so an automatic recovery that guesses the window costs money the caller did not decide
to spend.

A taken port shows the same gap at a smaller scale: `listen` answers reason `io` with no command, so an agent that hits
it has nothing to run.

The request came from an agentic user of `xr` (2026-10-09) and was approved by Brett then; the flag shapes were left
open and are settled in this plan.

### Requirements

**Forwarding**

- R1. `listen --forward <URL>` sends each verified event to URL as a `POST` whose body is the event's bytes as X sent
  them, carrying the signature header X sent and each `-H 'Name: Value'` the caller gave.
- R2. Everything after `--` is a program and its arguments; `listen` runs it once per verified event with the event body
  on its stdin, never through a shell.
- R3. `--forward` and a program may be given together; each receives every event, in the order X delivered them to
  `listen`.
- R4. X's acknowledgment never waits for a forward to finish. An event is acknowledged once it verifies and is queued,
  as today; when a target falls far enough behind that its queue is full, deliveries slow until it catches up.
- R5. A forward that fails is retried within a bounded budget, then reported, and `listen` keeps running. A forward
  failure never changes `listen`'s exit status; a panic in `xr` is not a forward failure and exits 1 (KTD13).
- R6. On shutdown, by signal or `--max-events`, `listen` finishes the forwards already queued before it exits, within
  one bounded drain deadline, and reports how many it could not deliver. A second signal exits at once.
- R7. Under a structured format, stdout carries the `listening` line and then one line per event, as today. Forward
  outcomes never reach stdout.

**Restart recovery**

- R8. `listen --checkpoint <FILE>` records, to the minute, when the oldest verified event not yet delivered or reported
  failed at every forward target arrived, or the most recent event when none is pending, and a crash mid-write leaves
  the previous record readable (R7, second-pass D1).
- R9. `listen` sends no request to X under any combination of flags.
- R10. `xr webhooks replay <WEBHOOK_ID> --from-checkpoint <FILE>` requests the window from the checkpoint's minute to
  the current minute, limited to the 24 hours X can replay, and says so when the limit cut part of the gap.
- R11. `--dry-run` on that command shows the window it would request and sends nothing. Events a replay delivers reach
  the forward targets like any other, including events the targets already received.

**Errors and contract**

- R12. A port another process holds answers with its own reason and a `next_step` that reruns `listen` with `--port 0`.
- R13. Every new error carries a reason from the closed set and a `next_step`, per `AGENTS.md` § Output formats.
- R14. `--dry-run` on `listen` reports its forward targets and checkpoint, and binds nothing, runs nothing, and writes
  nothing.
- R15. No test sends a request to X. Forward tests target local servers and local programs.

**Library**

- R16. An embedder can forward an event to a URL under the same budget, read an event's headers and arrival time, read
  and write the checkpoint, compute the replay window, and recognize the taken-port error, all from `xdk` without the
  binary.

### Key Decisions

- **Forwarding is built into `listen`.** (session-settled: user-approved — chosen over documenting a stdout pipe or
  leaving it out: the agentic user asked for it and Brett approved it on 2026-10-09.) Governs R1, R3.
- **A local app or script is a forward target too.** (session-settled: user-directed — chosen over URL-only forwarding:
  Brett added it to the approved forwarding on 2026-10-09.) Governs R2.
- **The program comes after `--`.** (session-settled: user-approved — chosen over `--exec` with one command string and
  over `--exec` with repeated `--exec-arg`: no quoting layer, so an agent passes words verbatim, and it reads the same
  on Windows.) Governs R2.
- **One process per event, with the event on stdin.** (session-settled: user-approved — chosen over one long-running
  program fed a line per event: piping stdout already serves that.) Governs R2.
- **X is acknowledged before forwarding; a failed forward is retried within a bound and then reported.**
  (session-settled: user-approved — chosen over holding X's answer until the forward succeeds: X waits 10 seconds and
  documents no redelivery.) Governs R4, R5.
- **Restart coverage is built into `xr`.** (session-settled: user-directed — chosen over documenting a supervisor
  recipe: Brett, 2026-10-09.) Governs R8, R10.
- **The replay is its own invocation that reads `listen`'s checkpoint.** (session-settled: user-approved — chosen over a
  `listen` flag that requests the replay on start: the billed request always comes from a command the caller runs on
  purpose, `AGENTS.md` says flags never select endpoints, and `listen` keeps sending nothing to X.) Governs R9, R10,
  R11.
- **Repeated events pass through after a replay.** (session-settled: user-approved — chosen over tracking event ids
  across restarts: a target must tolerate repeats, and dedupe would need a store of ids.) Governs R11.
- **The library and the binary are both held to best in class, and the binary builds on the library.** (session-settled:
  user-directed — chosen over a review of the CLI alone: Brett, 2026-10-09.) Governs R16.

### Success Criteria

- An in-process test runs `listen --max-events 1 --forward <local URL> -- <program>`, posts one signed event, and reads
  it byte for byte at both targets before `listen` exits 0.
- After `listen` stops with a checkpoint written, `xr --dry-run webhooks replay <ID> --from-checkpoint <FILE>` names a
  window starting at the checkpoint's minute.
- The `AGENTS.md` recipe walks and `crates/xurl-cli/tests/dry_run_guard.rs` pass with no walk's test edited.

### Scope Boundaries

- **No tunnel**, as in the webhooks plan. The caller exposes the address.
- **No typed event payloads.** Forwarding carries X's bytes.
- **No Account Activity to X Activity migration.** X's docs mark the Account Activity API deprecated; Brett deferred it
  on 2026-10-22.
- **Considered and not built:**
  - *Dedupe across restarts.* It needs a durable store of event ids, and a target must tolerate repeats anyway. Evidence
    that would change this: a target that cannot be made idempotent.
  - *A dead-letter file for failed forwards.* The failure record names the event's arrival time, and a replay within 24
    hours recovers it. Evidence: a gap longer than 24 hours that matters.
  - *Re-signing forwarded events with a key of `xr`'s own,* as Stripe CLI does. X's signature header already lets the
    target verify with the app's secret.
  - *More than one URL, more than one program, or concurrent forwards to one target.* One of each keeps order simple.
    Evidence: a fan-out need.
  - *An environment variable naming the signature scheme for the program.* It matters only under `--allow-unsigned`, a
    local-test flag.
  - *A flag to skip TLS verification on the forward.* Local targets are plain HTTP; nothing asks for it.

#### Deferred to Follow-Up Work

- The skill bundle documents the new flags after the release that carries them.
- X documents an Account Activity replay endpoint
  (`POST /2/account_activity/replay/webhooks/{webhook_id}/subscriptions/all`) that the vendored spec does not have, so
  `xr` cannot declare it. `/2/webhooks/replay` covers every event delivered or attempted to the webhook, Account
  Activity events included. Revisit when a spec refresh adds it.

### Sources

- X: `docs.x.com/x-api/webhooks/introduction` (answer within 10 seconds, a 200 acknowledges, CRC on registration and
  hourly); `docs.x.com/x-api/webhooks/create-replay-job-for-webhook` and the vendored spec's
  `CreateWebhookReplayJobRequest` (`from_date` and `to_date` inclusive, `yyyymmddhhmm` UTC, 24 hours, bearer only);
  `docs.x.com/x-api/account-activity/introduction` (the Account Activity replay endpoint). Read 2026-10-22.
- Stripe CLI `stripe listen`: `-f, --forward-to <url>`, `-H, --headers "Key1:Value1, Key2:Value2"`, a line per event,
  forwarded events re-signed with a CLI secret. smee-client: a source and a target URL. adnanh/webhook: a command per
  hook, with request values passed as arguments, environment, or a file. The `gh webhook` extension's flag documentation
  could not be retrieved.
- `docs/solutions/conventions/non-interactive-orchestrator-detach-stdin-and-re-gate.md` (a child's stdin must never be
  the operator's terminal), `docs/solutions/best-practices/rust-bounded-http-reads-with-take-2026-04-20.md` (cap a
  response read), `docs/solutions/best-practices/rust-url-validation-https-only-with-localhost-exception-2026-04-20.md`
  (validate a URL once, at load).

---

## Planning Contract

### Command grammar

Directional; clap's generated help is the authority once built.

```text
xr webhooks listen [--port N] [--bind ADDR] [--path P] [--secret oauth2|oauth1]
                   [--allow-unsigned] [--max-events N]
                   [--forward URL] [-H 'Name: Value']... [--forward-timeout SECS]
                   [--checkpoint FILE]
                   [-- PROGRAM [ARG]...]

xr webhooks replay WEBHOOK_ID (--from YYYYMMDDHHMM --to YYYYMMDDHHMM | --from-checkpoint FILE)
```

`-H` and `--forward-timeout` require a forward target. `--from-checkpoint` conflicts with `--from` and `--to`, which
stay required together.

### Key Technical Decisions

- KTD1. **Placement follows `AGENTS.md` § "Where a change goes".** The library owns the URL forward (network I/O), the
  event's headers and arrival time, the checkpoint file (file I/O), the replay window, and the taken-port
  classification. The binary owns argv, spawning the program (as `crates/xurl-cli/src/cli/skill_install/git.rs` spawns
  `git`), the per-target queues and their output, and composing each `next_step`. Governs R16.
- KTD2. **`Event` gains the headers a forward needs and its arrival time.** The signature headers and `Content-Type` as
  received, and a UTC arrival time. `Event` is `#[non_exhaustive]`, so the fields are additive.
- KTD3. **Acknowledge, then queue, then forward.** The consume loop prints the event line, then puts the event on each
  target's bounded queue (capacity 256), and the checkpoint follows delivery (KTD7). A full queue blocks the loop, which
  fills the receiver's 64-slot channel and then delays X's 200. Memory stays bounded and a stalled target slows
  deliveries instead of dropping events. Holding X's answer for the forward was rejected: one slow target would make X
  count every delivery failed. Governs R3, R4.
- KTD4. **One retry budget for both target kinds.** Up to four attempts, waiting 1, 2, then 4 seconds between them. Each
  attempt is cut off by `--forward-timeout` (default 30 seconds; a program target may run an agent). A URL attempt is
  delivered on a 2xx from the forward URL itself; the client follows no redirect, so a 3xx is a final failure, not
  retried, whose record names the status and never the `Location` (R11, second-pass D3: a followed 301 or 302 turns the
  `POST` into a body-less `GET` that can answer 200). It is retried on a connection failure, a timeout, 408, 429, or
  5xx, and not on another 4xx. A program attempt is retried on a non-zero exit or a timeout. The budget is a library
  type so an embedder's own target runs under the same rule. The waits add up to 7 seconds, which covers a local server
  restarting, the common transient failure; a target down longer is reported and recovered by a replay, so a longer
  budget would only hold up every event queued behind it. Governs R5, R6.
- KTD5. **The URL request carries X's bytes and X's signature.** `POST`, the body unchanged, `Content-Type` as X sent
  it, the signature header as received, then `Idempotency-Key: "<hex>"`, the SHA-256 of the event's exact bytes as 64
  lowercase hex characters, quoted as the Structured Field String the IETF httpapi draft
  (draft-ietf-httpapi-idempotency-key-header-07) specifies (DX D9: a key a target deduplicates on, the same for a retry
  and for a replay of the same bytes; R9, second-pass D5: idempotency middleware already keys on this name, and RFC 6648
  advises against a new `X-` name), then the caller's `-H` headers, which may not set `Host`, `Content-Length`, or
  `Idempotency-Key`; a `-H` with no colon, an empty name, or a name or value HTTP does not allow is refused before
  `listen` binds, identified by its position among the `-H` flags and never by its value (R5, D7). The target verifies
  with the app's own secret, the same way `listen` does. The response body is read up to a cap and discarded.
  `--forward` takes `http` or `https` and is refused offline otherwise, with reason `validation`; a URL that carries a
  username or password is refused the same way, with a message naming `-H 'Authorization: ...'`, and a failure's cause
  drops the URL with `reqwest`'s `without_url` so no record repeats it (R4, D6). Governs R1.
- KTD6. **The program target runs with no shell and no terminal.** Argv after `--` goes straight to the OS; stdin is
  written from its own task and then closed, and the attempt timeout covers the write and the wait together, so a
  program that never reads its stdin cannot hold an attempt open past the timeout; the child's stdout and stderr go to
  `listen`'s stderr, never its stdout (R7). An attempt past its timeout, the KTD12 drain deadline, or a second signal
  kills the program and everything it started: on Unix its own process group through `killpg` and the linked `libc`, on
  Windows its kill-on-close Job Object, which the program is assigned to while suspended (R3, D5). `listen` checks
  before binding that the program resolves, by searching `PATH` itself (and `PATHEXT` on Windows) with no new dependency
  (both read once in `crates/xurl-cli/src/cli/env.rs`, per `AGENTS.md` rule 2, and exempt beside `HOME` from the guard
  that every variable `xr` reads is advertised in `--help`, since they are OS search paths, not `xr` settings, C2), or
  by checking a name with a path separator directly, and refuses with reason `validation` when it does not. The
  program's environment gains `XR_EVENT_SHA256`, the same key the URL target gets in `Idempotency-Key`, as 64 lowercase
  hex characters, unquoted (DX D9, R9). Bytes from X reach the program only on stdin, never in its arguments, its
  environment, or a shell, so an event cannot become a command. Governs R2.
- KTD7. **The checkpoint is a one-field JSON file written atomically, once a minute at most.**
  `{"last_event_at": <Unix seconds>}` (R2, D3: a number needs no parser). The time recorded is the arrival of the oldest
  event not yet finished at every forward target, where finished means delivered or reported `forward-failed`, and the
  arrival of the newest event when nothing is pending, so with no target it is the newest event (R7, second-pass D1: a
  consumer commits its position after it processes a record). Each worker reports every finished event to the consume
  loop in queue order; the loop keeps each target's oldest pending arrival and writes the record when the minute of that
  time differs from the minute last written, through the store's `write_atomically` in `crates/xdk/src/store/atomic.rs`
  (temp file, fsync, rename, fsync of the directory). A replay window is minute-granular and its start is inclusive, so
  a record from earlier in the same minute recovers exactly what a per-event record would, and a burst of deliveries
  costs one fsync a minute instead of one per event. `listen` only writes it; a missing file is created on the first
  event. A hard stop with a backlog (a kill, an OOM kill, an abort) leaves the record at the oldest undelivered minute,
  so `replay --from-checkpoint` covers the backlog and the faster target sees repeats. The field keeps its approved name
  (R2): it names the newest event only when nothing is pending. Governs R8.
- KTD8. **The replay window is computed in the library from the checkpoint and the clock,** through the one UTC date
  formatter the workspace has, moved from `crates/xurl-cli/src/cli/output/rfc3339.rs` into `xdk` and shared with
  `retry_at` (R2, D3). `from` is the checkpoint's minute, inclusive, so events from that minute come again; `to` is the
  current minute. A `from` older than 24 hours is moved up to the earliest minute X can replay, and the result says the
  window was clamped and from when events are unrecoverable. A missing or unparseable checkpoint is refused with reason
  `io` (exit 5, a local file that could not be read, step `fix-input`), and a checkpoint later than the clock with
  reason `validation`. Governs R10, R11.
- KTD9. **A taken port is its own error.** `Receiver::bind` maps `io::ErrorKind::AddrInUse` to a new
  `Error::AddressInUse` carrying the address and the OS error as its source (Pass 3 context), with kind
  `address-in-use`, next action `fix-input`, and exit code 5, the code the `io` error it replaces has today, so only the
  reason and the step change. `Error` is `#[non_exhaustive]`, so the variant is additive. The binary composes the step
  the way `resume_wait_hint` in `crates/xurl-cli/src/cli/hints.rs` does. It is the same invocation with `--port 0`, as a
  `command` when the caller gave no `-H`, and as a `template` with each header value replaced by `<value>` when they
  did, since a header often carries a credential and its value is never echoed. Governs R12, R13.
- KTD10. **A forward outcome is a stderr record.** A failed forward writes one stderr line: prose in text output, and
  under a structured format a JSON object with `status: "forward-failed"`, the target kind, the attempt count, the
  cause, the event's arrival time, its `event_sha256` (DX D9; lowercase hex, R9),
  `side_effects: "printed, not delivered"`, and a `next_step` of
  `{action: "run-command", template: "xr webhooks replay <WEBHOOK_ID> --from <minute> --to <minute>"}` with `--from` set
  to the event's arrival minute and `--to` to the minute after it, so the agent fills in only the webhook id (DX D10;
  R13, second-pass D7: a two-minute window does not depend on X accepting equal bounds, at the cost of replaying one
  more minute of events, which X may bill). Shutdown with undelivered events writes one `status: "forward-incomplete"`
  record with the count. A successful forward writes one `forwarded` record the same way, with the target kind, the
  attempts, the HTTP status or exit code, the time taken, and the event's arrival time (DX D7), so an agent can count
  deliveries. Stderr, because stdout's contract is one line per event (R7) and `xr` already writes its structured error
  envelopes there; a consumer reading events from stdout never has to tell a status record from an event. Governs R5,
  R6, R7.
- KTD11. **Dry runs show the plan of action and nothing else.** `listen --dry-run` adds the forward URL, the header
  names (never values), the program and its arguments, the timeout, and the checkpoint path to its context.
  `replay --dry-run --from-checkpoint` adds `from`, `to`, and `clamped`. Governs R11, R14.
- KTD12. **Shutdown drains under one deadline, not one budget per event.** From the stop signal, the workers have one
  attempt timeout plus the budget's 7 seconds of waits to deliver what is queued; whatever is undelivered then is
  counted in the `forward-incomplete` record, and the checkpoint stays at the first undelivered event's minute (R7). A
  second signal exits at once. Each queue is closed and drained to its end; at the deadline the workers are aborted and
  the events still queued or in flight are counted. A per-event budget at drain time was rejected: with a target that
  never answers, 256 queued events at four 30-second attempts each would hold `listen` for hours after it was asked to
  stop. Governs R6.
- KTD13. **A panic anywhere in `xr` answers `internal` and exits 1.** The release profile aborts on a panic
  (`Cargo.toml:45`), so no join handle ever reports one. `run_argv`, the binary's entry, installs one panic hook before
  it calls `run`, and the runner publishes the resolved output format to a process-wide cell the hook reads, so
  in-process callers of `run_with_store_path` never install it (T2): it writes the `internal` error (reason `internal`,
  `exit_code` 1, the `report-issue` step, the panic message and location, and a `backtrace` field when `RUST_BACKTRACE`
  asks) in the active format to stderr, flushes stdout, and exits 1 before the abort runs; a panic before the format is
  resolved is written as text. A `listen` that panics drains nothing, and the checkpoint (KTD7, R7) keeps every
  undelivered event replayable. Restarting a worker was rejected (DX D11): a bug that panics on every event would loop.
  Watching join handles under `panic = "unwind"` was rejected (R8, second-pass D2): it grows the release binary 13.7%
  and leaves every other command's panic at exit 101 in plain text. Governs R5, R6.
- KTD14. **One signed-sample builder, in the library, outside the testing feature.** `xdk::webhooks` builds a signed
  sample event (the marked body, the signature header name and value) from the app's signing secrets, and exports the
  two signature header names; `xdk::testing` adds a sender that posts that sample to a `Receiver` for embedders' tests;
  `listen`'s `test_command` (U9) is composed from the same builder (DX D12). The builder stays outside `testing` because
  that feature pulls in `wiremock` (`crates/xdk/Cargo.toml`), which the release `xr` must not link; `xr` enables
  `testing` only as a dev-dependency (`crates/xurl-cli/Cargo.toml:72`).

### High-Level Technical Design

Event flow through a running `listen` (KTD3, KTD4, KTD5, KTD7, KTD10):

```mermaid
flowchart TB
  X[X delivers a POST] --> R[Receiver: verify signature]
  R -->|401| X
  R -->|queued: 200| C[(64-slot channel)]
  C --> L[Consume loop]
  L --> O[stdout: one JSON line]
  L --> QU[(URL queue, 256)]
  L --> QP[(Program queue, 256)]
  QU --> WU[URL worker: retry budget, no redirects]
  QP --> WP[Program worker: retry budget]
  WU -->|POST body + signature + Idempotency-Key + -H| T1[Caller's endpoint]
  WP -->|spawn, body on stdin, XR_EVENT_SHA256| T2[Caller's program]
  WU -->|forwarded or forward-failed| E[stderr record]
  WP -->|forwarded or forward-failed| E
  WU -->|finished| L
  WP -->|finished| L
  L --> K[Checkpoint: oldest unfinished arrival, when its minute moves]
```

Shutdown, on SIGINT or the last of `--max-events` (R6):

```mermaid
sequenceDiagram
  participant S as Signal or max-events
  participant L as Consume loop
  participant W as Target workers
  participant E as stderr
  S->>L: stop
  L->>L: stop taking events, release the port
  L->>W: close queues
  W->>W: deliver what is queued until the KTD12 deadline
  W->>E: forward-incomplete record, when any were undelivered
  W->>L: done
  Note over L: the checkpoint stays at the first undelivered minute (KTD7)
  L->>S: exit 0
  Note over L,E: a panic anywhere instead writes the internal record and exits 1 (KTD13)
```

Restart recovery, run by the caller (KTD7, KTD8):

```mermaid
sequenceDiagram
  participant A as Agent or supervisor
  participant L as xr webhooks listen
  participant F as Checkpoint file
  participant P as xr webhooks replay
  participant X as X
  L->>F: last_event_at: oldest undelivered arrival, when its minute moves
  Note over L: listen stops
  A->>L: start again with --checkpoint
  A->>P: --from-checkpoint FILE
  P->>F: read last_event_at
  P->>P: window: checkpoint minute to now, at most 24 h
  P->>X: POST /2/webhooks/replay (bearer)
  X->>L: deliveries again, through the caller's public URL
```

What each flag combination does:

| Flags                                      | stdout          | URL target | Program target | Checkpoint    |
| ------------------------------------------ | --------------- | ---------- | -------------- | ------------- |
| none                                       | events          | none       | none           | none          |
| `--forward`                                | events          | each event | none           | none          |
| `-- PROGRAM`                               | events          | none       | each event     | none          |
| both                                       | events          | each event | each event     | none          |
| any of the above, plus `--checkpoint`      | events          | as above   | as above       | once a minute |
| `-H` or `--forward-timeout` with no target | refused by clap |            |                |               |

### System-Wide Impact

- **Library public API.** `Event` gains fields and `Error` a variant, both on `#[non_exhaustive]` types; `webhooks`
  gains the forward and checkpoint modules. Every change is additive, so `xdk-rs` takes the minor it is already due.
- **The error contract.** `address-in-use` joins the closed reason set, so the README's "Which Errors Carry a Step"
  table, `crates/xurl-cli/tests/next_step_tests.rs`, and a reason golden move together (U2).
- **The dry-run contract.** `listen` and `replay` gain context keys, additively, and `dry_run_guard.rs` must still see
  nothing sent, bound, run, or written for both.
- **Help, completions, and goldens.** The listen and replay pages, the examples page, and the completions move; no other
  fixture should.
- **Binary size.** The forward reuses the `reqwest` already linked; `tokio`'s process feature is the one likely
  addition, under the 12,000,000-byte ceiling. `windows-sys` is compiled for Windows targets only, so the Unix binaries
  gain nothing from it.
- **Every command's panic.** A panic in any `xr` command answers the `internal` error in the active format and exits 1,
  where it exited 134 with Rust's message (KTD13, R8).
- **The skill bundle** documents the released binary and follows the release (Deferred to Follow-Up Work).

### Risks & Dependencies

- **X's replay behavior is taken from its spec and docs, not observed.** Whether minute bounds are inclusive as
  documented, and what a replay bills, are unverified, by R15. A `forward-failed` template run within a minute of the
  event names a `--to` minute that has not ended (R13); whether X accepts that is as unverified as the bounds.
- **GitHub's Windows runners may refuse a nested Job Object.** process-wrap's tracker reports `AssignProcessToJobObject`
  returning `ERROR_ACCESS_DENIED` on GitHub-hosted Windows runners, where the job's own process already sits in a job.
  U8's Windows job (R6, D8) is where that shows first; check it before writing the Windows kill module, and if the
  runner refuses, make the failure visible in the test rather than skip it.

---

## Implementation Units

### U1. Library: event headers and arrival time

- **Goal:** An event carries what a forward and a checkpoint need.
- **Requirements:** R16; KTD2.
- **Dependencies:** none.
- **Files:** `crates/xdk/src/webhooks/mod.rs`, `crates/xdk/tests/webhook_receiver_tests.rs`.
- **Approach:** Capture the signature headers and `Content-Type` in `receive_event`, and stamp the arrival time before
  the event is queued.
- **Execution note:** Write the header test first and watch it fail against today's `Event`.
- **Patterns to follow:** the existing receiver tests for a bound receiver driven over loopback.
- **Test scenarios:**
  - A signed POST yields an event whose headers hold the OAuth2 signature header and `Content-Type` exactly as sent.
  - A legacy-signed POST yields an event holding the legacy header and not the OAuth2 one.
  - An event's arrival time lies between the times taken just before and just after the POST.
- **Doc example:** a rustdoc example on `Receiver` that binds, answers, and loops `next_event()`, compiled as a `no_run`
  doctest, since it binds a listener and loops (DX D8, T4).
- **Verification:** The receiver tests pass, and `cargo-semver-checks` reports the additions as minor.

### U2. Library and CLI: the taken-port error and its step

- **Goal:** A taken port is its own error, and an agent that hits one gets a step it can run.
- **Requirements:** R12, R13, R16; KTD9; R1 in the Decision ledger (D2: the error and its reason land together).
- **Dependencies:** none.
- **Files:** `crates/xdk/src/webhooks/mod.rs`, `crates/xdk/src/error.rs`, `crates/xdk/tests/webhook_receiver_tests.rs`,
  `crates/xdk/tests/error_tests.rs`, `crates/xurl-cli/src/cli/envelope.rs`, `crates/xurl-cli/src/cli/hints.rs`,
  `crates/xurl-cli/src/cli/runner.rs`, `crates/xurl-cli/src/cli/output/mod.rs`, `crates/xurl-cli/README.md` ("Which
  Errors Carry a Step"), `crates/xurl-cli/tests/next_step_tests.rs`, `crates/xurl-cli/tests/webhooks_tests.rs`, a
  `reason-address-in-use` golden under `crates/xurl-cli/tests/golden/`.
- **Approach:**
  1. Add `Error::AddressInUse` with its `kind`, `exit_code`, `next_action`, and `docs_url` arms, each matched
     exhaustively as rule 4 of "Where a change goes" requires.
  2. Map `AddrInUse` from `TcpListener::bind` in `Receiver::bind`; every other bind failure stays `io`.
  3. Add `address-in-use` to the closed reason set and the README table in the same PR, so `Reason::from_kind` never
     meets the kind unknown (`crates/xurl-cli/src/cli/output/mod.rs:721`).
  4. Add the hint to the runner's chain in `crates/xurl-cli/src/cli/runner.rs` beside `resume_wait_hint`, taking the
     invocation the runner already passes to `choose_hint`. Rewrite that invocation: drop any `--port`, `-p`, or
     `--port=` value, insert `--port 0` before a `--`, keep the program argv after `--` unchanged, and replace each `-H`
     value with `<value>`, which turns the step into a `template` (KTD9).
  5. Text output names the address and suggests a free port or stopping the other process.
- **Execution note:** Write the receiver test that binds a held port first and watch it report `io`.
- **Patterns to follow:** `Error::ProcessingTimeout` for a fact-carrying variant; #325 (`5fa91c6`) for one error change
  across both crates; `resume_wait_hint` and `NextStep::resume_wait` in `crates/xurl-cli/src/cli/hints.rs` for the hint;
  `NextStep::confirm` in the same file for building the step's words from the invocation with `shell_word`; the
  reason-golden fixtures.
- **Test scenarios:**
  - Binding a port another listener holds returns `AddressInUse` naming that address, with kind `address-in-use` and
    exit code 5.
  - Binding an address the host does not have stays an `io` error.
  - `listen` on a held port under `--output json` exits 5 with reason `address-in-use`, `next_step.action` `fix-input`,
    and a `command` of `xr webhooks listen --port 0` plus the flags given.
  - The same with `-H 'Authorization: Bearer abc'` carries a `template` in which `abc` does not appear and `<value>`
    does.
  - With `--port 8080 -- ./handle.sh --port 9`, the step reads `--port 0 -- ./handle.sh --port 9`: the program's own
    `--port 9` is untouched and `8080` is gone.
  - `next_step_tests.rs` holds the README table to the new reason.
- **Verification:** The golden fixture shows the envelope, the README row and `next_step_tests.rs` agree, and
  `cargo-semver-checks` reports the variant as minor.

### U9. Library and CLI: a signed sample event and listen's test command

- **Goal:** An agent proves its wiring before X is involved, and an embedder tests a receiver in a few lines: one
  library builder makes a signed sample event, `listen`'s startup line hands the agent a ready command built from it,
  and `xdk::testing` posts it for embedders' tests.
- **Requirements:** R7 (stdout stays one record per line), R16; KTD14; DX D5 (a signed test command) and DX D12 (one
  builder for both callers) in the DX Review.
- **Dependencies:** none.
- **Files:** `crates/xdk/src/webhooks/signing.rs` (the builder and the exported header names),
  `crates/xdk/src/webhooks/mod.rs` (re-exports), `crates/xdk/src/testing/mod.rs` (the sender),
  `crates/xdk/tests/webhook_receiver_tests.rs`, `crates/xurl-cli/src/cli/commands/webhooks_listen.rs`,
  `crates/xurl-cli/tests/webhooks_tests.rs`, `crates/xurl-cli/src/cli/family_help.rs` (the listen page names the
  command), `crates/xurl-cli/tests/golden/help-webhooks-listen.golden`.
- **Approach:**
  1. Library: a builder that takes the signing secrets and returns a fixed sample body that marks itself as a test
     (`{"xr_test_event":true,...}`) with the signature header name and value for the secret `listen` verifies with;
     export the two header-name constants (KTD14).
  2. `xdk::testing`: a sender that posts the sample to a bound `Receiver`'s address and path.
  3. CLI: after binding, compose a `curl` from the builder's output that POSTs to the bound URL; text output adds it to
     the stderr notice, and a structured format adds it as `test_command` on the `listening` record, an additive key,
     quoted for a POSIX shell like every command string xr prints. Beside it, a structured format adds `test_request`
     (`method`, `url`, `headers`, `body`), the same request for a caller with no POSIX shell, Windows included (R10,
     second-pass D4).
  4. The signature covers that exact body only, so a replay of the logged command delivers only the marked sample; the
     README says to ignore events carrying `xr_test_event`.
- **Patterns to follow:** `testing::MockX` for a helper behind the `testing` feature; the `listening` record in
  `webhooks_listen.rs`; `shell_word` (`crates/xurl-cli/src/cli/hints.rs:552`) for quoting the command; it is private to
  that module, so the command is composed in `hints.rs` beside the step builders, or `shell_word` becomes `pub(crate)`
  (S2).
- **Test scenarios:**
  - The builder's sample verifies against a bound `Receiver` on the OAuth2 path and, with only a consumer secret, on the
    legacy path.
  - The `testing` sender's POST is yielded by the `Receiver` as a verified event holding `xr_test_event`.
  - Under `--output json` the `listening` record carries `test_request`, and posting it with `reqwest` yields a verified
    event; the test runs on every OS, the U8 Windows job included (R10).
  - Unix only (`cfg(unix)`): running the record's `test_command` through `sh` against the listener yields a verified
    event.
  - Neither form contains a secret: the client secret and the consumer secret appear in neither `test_command` nor
    `test_request`.
  - The release `xr` build does not enable `xdk`'s `testing` feature
    (`cargo tree -p xurl-rs -e features,no-dev -i xdk-rs` lists no `testing` feature; `-e features` alone includes dev
    edges, S3).
- **Doc example:** a running doctest on the builder, and one on the `testing` sender that binds a `Receiver` on port 0
  in process and runs under `--features testing` (DX D8, DX D12, T4).
- **Verification:** The sample verifies on every signing path from the library, `test_request`, and the printed command,
  and the listen help golden moves only for the mention.

### U3. Library: forwarding to a URL under the retry budget

- **Goal:** An embedder can forward an event to a URL with the same request shape and retry rule `xr` uses.
- **Requirements:** R1, R5, R16; KTD4, KTD5.
- **Dependencies:** U1.
- **Files:** `crates/xdk/src/webhooks/forward.rs` (new; `mod.rs` is past the 200-line trigger),
  `crates/xdk/src/webhooks/mod.rs` (module and re-exports), `crates/xdk/tests/webhook_forward_tests.rs` (new).
- **Approach:**
  1. A retry-budget type holding the attempt count, the waits, and the per-attempt timeout, with a runner that takes any
     async attempt and returns the outcome and the attempts made, so the binary's program target reuses it.
  2. A URL target built once from the URL and headers: it validates the scheme, refuses `Host`, `Content-Length`, and
     `Idempotency-Key` among the headers, and owns an HTTP client built like the one at
     `crates/xdk/src/api/request/mod.rs:311` but with redirects off (`reqwest::redirect::Policy::none()`, R11).
  3. Sending one event: the KTD5 request, the response read up to a cap, the status classified as retryable or final per
     KTD4.
  4. A forward emits `tracing` events on `xdk::webhooks` and prints nothing.
- **Execution note:** Start with the request-shape test against a local mock and see it fail before the target exists.
- **Patterns to follow:** the receiver's `tracing` kinds;
  `docs/solutions/best-practices/rust-bounded-http-reads-with-take-2026-04-20.md` for the capped read.
- **Test scenarios:**
  - A forward to a mock answering 200 sends one POST whose body equals the event bytes, with the signature header,
    `Content-Type`, and two caller headers.
  - The POST's `Idempotency-Key` is the body's SHA-256 as quoted lowercase hex, and a retried attempt sends the same
    value (DX D9, R9).
  - A `-H 'Idempotency-Key: x'` is refused when the target is built (DX D9, R9).
  - A mock answering 503 twice and then 200 gets three attempts, and the outcome is delivered after three.
  - A mock answering 400 gets one attempt, and the outcome is a final failure naming 400.
  - A mock answering 302, 307, or 308 with a `Location` pointing at a second mock gets one attempt; the outcome is a
    final failure naming the status, its cause does not contain the `Location`, and the second mock receives nothing
    (R11).
  - A mock answering 204 counts as delivered (R11).
  - A mock that never answers gets four attempts each cut at the timeout, and the outcome names the timeout.
  - A closed port gets four attempts and a connection-failure outcome.
  - A `ftp://` URL, a URL with `user:pass@`, and a header set naming `Host` are each refused when the target is built,
    before any request.
  - A forward that fails against a closed port reports a cause that does not contain the URL.
  - Headers `X-Token abc`, `: v`, and a name holding a space are each refused when the target is built, and the message
    names the header's position, not `abc`.
  - A response body larger than the cap is not read past the cap.
- **Doc example:** a `no_run` doctest on the URL target, since it sends a request, and a running doctest on the retry
  budget (DX D8, T4).
- **Verification:** The forward tests pass with paused tokio time where waits are involved, and no test exceeds a few
  seconds of real time.

### U4. CLI: `listen --forward` and the per-target queues

- **Goal:** `listen` delivers every verified event to the caller's endpoint, in order, and says when one did not arrive.
- **Requirements:** R1, R3 through R7, R14; KTD3, KTD4, KTD10, KTD11, KTD12; the Key Decision on acknowledging first.
- **Dependencies:** U3, U9 (its sample is one forward scenario's input, S1).
- **Files:** `crates/xurl-cli/src/cli/mod.rs` (`Listen` arguments), `crates/xurl-cli/src/cli/family_help.rs` (the listen
  page), `crates/xurl-cli/src/cli/commands/webhooks.rs` (argument mapping),
  `crates/xurl-cli/src/cli/commands/webhooks_listen.rs`, `crates/xurl-cli/src/cli/runner.rs` and a
  `crates/xurl-cli/src/cli/panic_hook.rs` (new: the KTD13 hook), `crates/xurl-cli/tests/panic_hook_tests.rs` (new) with
  its panicking test program, `crates/xurl-cli/tests/binary_contract_tests.rs` (`sigpipe_smoke`, R12),
  `crates/xurl-cli/src/cli/commands/webhooks_forward.rs` (new: a target worker over a bounded queue, the stderr records,
  and the shutdown drain), `crates/xurl-cli/tests/common/mod.rs` (the `listen_while` helper moves here from
  `webhooks_tests.rs`), `crates/xurl-cli/tests/webhooks_forward_tests.rs` (new),
  `crates/xurl-cli/tests/dry_run_guard.rs` if its synthesized arguments need an override,
  `crates/xurl-cli/tests/golden/help-webhooks-listen.golden`, the `completions/` files.
- **Approach:**
  1. Clap: `--forward`, repeatable `-H/--header`, and `--forward-timeout`, the last two requiring a target.
  2. Build the URL target before binding (U3); a refusal is reason `validation` with the `show-help` step.
  3. After each printed line, put the event on the target's queue (KTD3). The worker is generic over the kind of
     attempt, so U8 adds a second target without a second worker.
  4. On cancel or `--max-events`, close the queues, wait for the workers, and write the KTD10 records.
  5. The dry-run context gains the URL, the header names, and the timeout (KTD11).
  6. Install the KTD13 panic hook in `run_argv` (`crates/xurl-cli/src/cli/runner.rs:63`) alone, before it calls `run`,
     through a function the doc-hidden `xurl` library target exposes so a test program installs the same hook; `run`
     publishes the resolved output format to a process-wide cell the hook reads, and a panic before that is written as
     text. In-process tests, which call `run_with_store_path`, never install the hook (R8, T2).
- **Execution note:** Implement test-first, starting from the success criterion's URL half.
- **Patterns to follow:** the `listening` line in `webhooks_listen.rs` for structured output; the cancel token from
  `crates/xurl-cli/src/cli/shutdown.rs`.
- **Test scenarios:**
  - `--max-events 1 --forward <mock>` with one signed event: the mock holds the body byte for byte with the signature
    header, and `listen` exits 0.
  - Three events reach the mock in the order they were posted.
  - With `--forward <mock>`, running U9's printed `test_command` delivers the sample to the mock with the signature
    header (S1).
  - Each delivered event yields one `forwarded` record on stderr with the mock's 200, one attempt, and the time taken;
    stdout is unchanged (DX D7).
  - With the mock answering 500 to every attempt, X's POST is still answered 200, a `forward-failed` JSON line with 4
    attempts reaches stderr, the next event is still printed, and `listen` exits 0.
  - Under `--allow-unsigned`, an unsigned event is forwarded with no signature header and `Content-Type` as sent (KTD5).
  - Under `--output jsonl`, stdout holds exactly the `listening` line and one line per event while forwards fail.
  - A `forward-failed` record carries `event_sha256`, `side_effects`, and a `next_step` template whose `--from` and
    `--to` are the event's arrival minute and the minute after it, across an hour and a day boundary too (DX D10, R13).
  - A program the suite builds installs the shipped panic hook under `--output json` and panics: it exits 1, stderr
    holds one JSON line with reason `internal`, `exit_code` 1, and the `report-issue` step, and stdout lines written
    before the panic survive; its message keeps Rust's `panicked at <file>:<line>:<col>` wording; under text output the
    message names the issue tracker (KTD13, R8). The test runs in the test profile and proves the release behavior,
    since the hook exits before the abort.
  - SIGINT with two events queued behind a slow mock delivers both before exit; with the mock down, the
    `forward-incomplete` record counts them.
  - With `--forward-timeout 1`, a mock that never answers, and three events queued, SIGINT ends `listen` within the
    KTD12 deadline of 8 seconds and the record counts 3; a second SIGINT during the drain exits at once.
  - `-H 'X: y'` without `--forward` is a clap usage error, exit 2.
  - CRITICAL regression: `sigpipe_smoke` also asserts the child exited 0 or was killed by signal 13
    (`ExitStatusExt::signal`) and keeps its two text checks, so a SIGPIPE drift fails it whatever the hook prints (R12).
  - In process, a panic raised after `run_with_store_path` returns still unwinds to `catch_unwind`: no path but
    `run_argv` installs the hook (T2).
  - `--forward ftp://x` and `--forward https://user:pass@127.0.0.1:9/x` are each refused with reason `validation` before
    anything binds, and the second's message names `-H` and does not contain `pass`.
  - A `forward-failed` record holds no part of the forward URL.
  - `-H 'Authorization Bearer abc'` is refused with reason `validation` before anything binds, the message names the
    first `-H`, and `abc` appears nowhere in the output.
  - `--dry-run --forward <URL> -H 'Authorization: Bearer abc'` reports the URL and the header name, never `abc`, and
    binds no port.
- **Verification:** The forward tests and the dry-run guard pass, the listen help golden moves for the new flags only,
  and completions are regenerated.

### U8. CLI: a program after `--` as a forward target

- **Goal:** `listen` runs the caller's program once per verified event, with the event on stdin, beside or instead of
  the URL target.
- **Requirements:** R2, R3, R5, R7, R14; KTD4, KTD6, KTD11; the Key Decisions on the program shape and one process per
  event.
- **Dependencies:** U4.
- **Files:** `crates/xurl-cli/src/cli/mod.rs` (the trailing argv on `Listen`), `crates/xurl-cli/src/cli/family_help.rs`,
  `crates/xurl-cli/src/cli/commands/webhooks_forward.rs` (the program attempt beside the URL attempt),
  `crates/xurl-cli/src/cli/commands/webhooks_listen.rs`, `crates/xurl-cli/src/cli/env.rs` (the `PATH` and `PATHEXT`
  reads), `crates/xurl-cli/tests/agentic_tests.rs` (both exempt beside `HOME`, with the reason), `AGENTS.md` (rule 2's
  list), a kill module with `#[cfg(unix)]` and `#[cfg(windows)]` halves behind one function,
  `crates/xurl-cli/Cargo.toml` (`windows-sys` 0.61 in a `[target.'cfg(windows)'.dependencies]` table with the job and
  thread features, beside the `cfg(unix)` table that holds `libc`), `Cargo.lock`,
  `crates/xurl-cli/tests/webhooks_forward_tests.rs`, test programs the suite builds instead of `sh` scripts (so the same
  tests run on Windows), `.github/workflows/ci.yml` (a `windows-latest` job running them),
  `crates/xurl-cli/tests/golden/help-webhooks-listen.golden`, the `completions/` files.
- **Approach:**
  1. Clap: trailing `-- PROGRAM ARGS`; `--forward-timeout` accepts it as a target.
  2. Resolve the program before binding (KTD6); a failure is reason `validation` with the `show-help` step.
  3. The attempt: a pure builder makes the command and a separate step spawns it with stdin piped, writes the body from
     its own task and then closes stdin, sends stdout and stderr to `listen`'s stderr, and kills the program's process
     group or Job Object when the timeout around the write and the wait expires (KTD6). It runs under the U3 budget in
     the U4 worker.
  4. The dry-run context gains the program and its arguments.
  5. The test programs (one that echoes stdin to a file, one that hangs, one that starts a long-running child and hangs,
     one that never reads stdin) are small Rust programs the test suite builds, not `sh` scripts, so every
     program-target test runs on Linux, macOS, and Windows alike.
  6. A `windows-latest` job in `.github/workflows/ci.yml`, gated like the other Rust jobs by the `scope` job, runs the
     program-target tests and U9's `test_request` test (R10), including a Windows twin of the group-kill test that
     proves the Job Object takes a grandchild with it (R6, D8). Whether the job joins the required checks in
     `.github/rulesets/protect-dev.json` is decided in that PR, since Brett applies rulesets.
- **Execution note:** Implement test-first, starting from the success criterion's program half.
- **Patterns to follow:** `crates/xurl-cli/src/cli/skill_install/git.rs` (a pure command builder separate from the
  spawn, with a typed reduction of spawn failures);
  `docs/solutions/conventions/non-interactive-orchestrator-detach-stdin-and-re-gate.md`.
- **Test scenarios:**
  - `--max-events 1 --forward <mock> -- <the suite's echo-to-file program> <file>` with one signed event: the mock and
    the file each hold the body byte for byte, and `listen` exits 0.
  - Three events reach the file in the order they were posted, one process each.
  - The program sees `XR_EVENT_SHA256` equal to the SHA-256 of what it read on stdin (DX D9).
  - Each program run that exits 0 yields one `forwarded` record naming the program target and exit code 0 (DX D7).
  - Under `--output jsonl`, stdout holds exactly the `listening` line and one line per event while the program writes to
    its own stdout; the program's output appears on stderr.
  - A program that exits 3 is reported as failed after four attempts, and the next event still runs.
  - A program that outlives `--forward-timeout 1` is killed, and no child is left running afterwards.
  - Unix: the suite's program that starts a long-running child of its own and then hangs is killed with its group at the
    timeout, and no process from that group survives.
  - Windows (the `windows-latest` job): a program that starts a long-running child of its own and then hangs is
    terminated with its Job Object at the timeout, and the child is gone.
  - A program that never reads its stdin, sent a body larger than a pipe buffer (200 KB) under `--forward-timeout 1`, is
    killed and reported within the timeout, and `listen` handles the next event.
  - An event body holding `$(touch pwned)` and `; rm` reaches the program's stdin as bytes and runs nothing.
  - With `--forward` pointed at a mock that never answers and a program given too, the program still receives each event
    as it arrives: one stalled target never delays the other (KTD3).
  - A program that does not resolve is refused with reason `validation` before anything binds.
  - `--dry-run -- ./handle.sh --verbose` reports the program and its argument, binds no port, and runs nothing.
- **Verification:** The program-target tests pass on Linux and on the `windows-latest` job, no child process outlives a
  test, and the listen help golden moves for the new argument only.

### U5. Library: the checkpoint and the replay window

- **Goal:** An embedder can keep a checkpoint and turn it into the window X can replay.
- **Requirements:** R8, R10, R16; KTD7, KTD8.
- **Dependencies:** U1.
- **Files:** `crates/xdk/src/webhooks/checkpoint.rs` (new), `crates/xdk/src/webhooks/mod.rs` (module and re-exports),
  `crates/xdk/src/store/atomic.rs` (reached from the webhooks module; it stays crate-private), the UTC formatter moved
  from `crates/xurl-cli/src/cli/output/rfc3339.rs` into `xdk` with its test, `crates/xurl-cli/src/cli/output/mod.rs`
  (`retry_times` calls the moved formatter), `crates/xdk/tests/webhook_checkpoint_tests.rs` (new).
- **Approach:**
  0. Move the UTC formatter from `crates/xurl-cli/src/cli/output/rfc3339.rs` into `xdk` with its table test, add the
     `yyyymmddhhmm` form beside it, and point `retry_times` in `crates/xurl-cli/src/cli/output/mod.rs` at it (R2).
  1. Write: a checkpoint writer that takes the time to record (the binary passes the oldest undelivered arrival, KTD7),
     writes the KTD7 record through `write_atomically` when the minute differs from the last one written, and skips the
     write otherwise.
  2. Read: parse the record, refusing a missing file, an unparseable one, or a time later than the clock passed in.
  3. Window: take the checkpoint time and the current time as arguments, so tests pass both, and return `from`, `to` as
     `yyyymmddhhmm`, whether it was clamped, and the first unrecoverable minute when it was.
- **Patterns to follow:** the store's error mapping for file failures; `validate_replay_time` in
  `crates/xdk/src/api/shortcuts.rs` for the twelve-digit form.
- **Test scenarios:**
  - A checkpoint at 2026-10-22T15:04:59Z with now at 15:31:10Z yields `from` 202610221504 and `to` 202610221531, not
    clamped.
  - A checkpoint 30 hours old yields a `from` 24 hours before now, clamped, naming the first unrecoverable minute.
  - A checkpoint and now in the same minute yield `from` equal to `to`.
  - A checkpoint ahead of now is refused with reason `validation`.
  - Arrivals at 15:04:10 and 15:05:02 leave the second time readable, and no temp file remains beside it.
  - Arrivals at 15:04:10 and 15:04:50 write once; the file holds 15:04:10.
  - A missing file, and a file holding `not json`, are each refused with reason `io` naming the path.
  - A file holding `{"last_event_at": "soon"}` or a negative number is refused with reason `io` naming the path.
  - CRITICAL regression: the moved formatter's table test passes unchanged in `xdk`, the `rate-limited-with-reset`
    golden stays byte-identical, and the `retry_at` assertion in `crates/xurl-cli/tests/cli_tests.rs` still holds.
- **Doc example:** rustdoc examples on the checkpoint writer and the replay window, compiled as doctests (DX D8).
- **Verification:** The checkpoint tests pass with fixed times and need no clock.

### U6. CLI: `listen --checkpoint` and `replay --from-checkpoint`

- **Goal:** A restarted `listen` plus one replay command recovers the gap, and the dry run shows the window first.
- **Requirements:** R8 through R11, R14; KTD7, KTD8, KTD11, KTD12; R7 in the second-pass ledger; the Key Decision on
  replay as its own invocation.
- **Dependencies:** U4, U5.
- **Files:** `crates/xurl-cli/src/cli/mod.rs` (`Listen` and `Replay` arguments),
  `crates/xurl-cli/src/cli/family_help.rs`, `crates/xurl-cli/src/cli/commands/webhooks.rs` (`replay`),
  `crates/xurl-cli/src/cli/commands/webhooks_listen.rs`, `crates/xurl-cli/src/cli/commands/webhooks_forward.rs` (the
  completion report), `crates/xurl-cli/tests/webhooks_tests.rs`, `crates/xurl-cli/tests/webhooks_forward_tests.rs`,
  `crates/xurl-cli/tests/dry_run_guard.rs`, `crates/xurl-cli/tests/golden/help-webhooks-replay.golden`,
  `crates/xurl-cli/tests/golden/help-webhooks-listen.golden`, the `completions/` files.
- **Approach:**
  1. Each target worker reports every finished event (delivered, or reported `forward-failed`) to the consume loop in
     queue order. `listen --checkpoint` keeps each target's oldest pending arrival and hands the oldest of those, or the
     newest arrival when nothing is pending, to the U5 writer (KTD7, R7). A write failure is reported on stderr like a
     forward failure and does not stop `listen`.
  2. `replay`: `--from-checkpoint` in a clap group against `--from` with `--to`. The window comes from U5 with the
     current time, and goes through the existing `create_webhook_replay` call, so the request, the bearer requirement,
     and `send_or_report` stay as they are.
  3. The structured output of `replay` adds `clamped` and, when set, the first unrecoverable minute, beside X's
     response.
- **Patterns to follow:** the existing `replay` dispatch and its `validate_replay_time` checks; the `dry_run_guard.rs`
  override rows for `webhooks replay`.
- **Test scenarios:**
  - `listen --checkpoint <file> --max-events 2` with two events and no forward target leaves a checkpoint naming the
    second event's minute.
  - With `--forward` to a mock that holds its answers, events arriving in three successive minutes and `listen` then
    killed leave a checkpoint naming the first minute (R7).
  - With the mock down at the KTD12 drain deadline and two events undelivered, the checkpoint names the first one's
    minute and `forward-incomplete` counts 2 (R7).
  - An event reported `forward-failed` stops holding the checkpoint back: the next delivered event's minute is written
    (R7).
  - With a URL target delivering and a program target stalled, the checkpoint follows the stalled target's oldest
    pending event (R7).
  - `replay <id> --from-checkpoint <file>` against the mock posts `webhook_id`, a `from_date` equal to the checkpoint's
    minute, and a `to_date` equal to the current minute, under the bearer.
  - `--dry-run` on the same prints `from`, `to`, and `clamped: false`, and the mock receives nothing.
  - A checkpoint 30 hours old makes the dry run report `clamped: true` and the first unrecoverable minute.
  - `--from-checkpoint` with `--from` is a clap usage error; `--from` alone still demands `--to`.
  - A missing checkpoint file is refused with reason `io`, exit 5, and sends nothing; its message names both fixes:
    `--from` with `--to`, and starting `listen` with `--checkpoint` (Pass 3 fix).
  - `listen --checkpoint` pointed into a read-only directory reports each failed write on stderr and keeps printing and
    forwarding events.
  - `replay` with `--from-checkpoint` and no bearer is `auth-method-mismatch`, as it is with `--from` and `--to`.
- **Verification:** The webhooks tests and the dry-run guard pass, and the replay and listen help goldens move for the
  new flags only.

### U7. Documents

- **Goal:** A reader learns the forward flags, the recovery recipe, and its cost from the shipped docs.
- **Requirements:** R5, R10, R11.
- **Dependencies:** U2, U3, U4, U5, U6, U8, U9.
- **Files:** `crates/xurl-cli/README.md` (the webhooks section), `crates/xurl-cli/src/cli/commands/examples.rs`,
  `crates/xurl-cli/tests/golden/examples.golden`, `KNOWN_DIFFERENCES.md`, `crates/xdk/README.md` (the `webhooks` module
  line, for forwarding and the checkpoint, pointing to the example), `crates/xdk/examples/webhook_forward.rs` (new).
- **Approach:**
  1. README: forwarding, the program target and its stdin contract, the failure records, that a redirect answer is a
     failed forward so `--forward` names the final URL (R11), repeats after a replay, and the restart recipe with the
     dry run first and the billing note; the checkpoint covers events still queued when `listen` stopped, so a target
     sees repeats (R7).
  2. Examples page: one forwarding line and the restart pair.
  3. `KNOWN_DIFFERENCES.md`: Go `xurl` neither forwards nor replays from a checkpoint.
  4. `crates/xdk/examples/webhook_forward.rs`: binds a `Receiver`, forwards each event under the retry budget, keeps a
     checkpoint, and prints the replay window it would request at start; pre-push's `cargo build --examples -p xdk-rs`
     builds it (DX D8).
  5. README and examples page open with the signed `test_command` from U9, and `test_request` for a caller with no POSIX
     shell, before the tunnel and X setup (DX D5, R10).
  6. README: the `Idempotency-Key` header and `XR_EVENT_SHA256` (repeats share the key, DX D9, R9), the `forwarded` and
     `forward-failed` records with the replay template (DX D7, DX D10), and that a panic in any `xr` command answers
     `internal` with exit 1 (KTD13); the README's step table documents `run-command`'s template form.
- **Test expectation:** none beyond the examples golden; `agentic_tests.rs` already requires the family on the examples
  page.
- **Verification:** markdownlint passes, and every flag the docs name appears in `xr webhooks listen --help` or
  `xr webhooks replay --help`.

---

## Verification Contract

| Gate                       | Command                                                                                                | Applies to           |
| -------------------------- | ------------------------------------------------------------------------------------------------------ | -------------------- |
| Unit and integration tests | `cargo test --workspace --locked`                                                                      | every unit           |
| One suite while iterating  | `cargo test -p xdk-rs --test webhook_forward_tests`, and the matching `-p xurl-rs --test <name>`       | the unit's own tests |
| Lints                      | `cargo clippy --workspace --all-targets -- -D warnings`                                                | every unit           |
| Help and examples fixtures | `XURL_GOLDEN_BLESS=1 cargo test -p xurl-rs --test golden_tests`, only where a unit says the help moved | U2, U4, U6, U7, U8   |
| Completions                | `scripts/generate-completions.sh`                                                                      | U4, U6, U8           |
| Local CI mirror            | `scripts/hooks/pre-push` (it runs on every push)                                                       | every PR             |
| Public-API semver          | CI's semver gate; `Event` fields and the `Error` variant are additive on `#[non_exhaustive]` types     | U1, U2, U3, U5       |
| Release binary size        | CI's agent-native audit against the 12,000,000-byte ceiling                                            | U3, U4, U8           |
| Windows program target     | the `windows-latest` CI job running the program-target tests (R6, D8)                                  | U8                   |

No gate sends a request to X (R15).

---

## Definition of Done

- Every R-ID is covered by a passing test or, for R9 and R15, by the absence of any request to X in the suites and the
  dry-run guard.
- The recipe walks, the dry-run guard, and `next_step_tests.rs` pass with no walk's test edited.
- Each PR's CI rollup is green, with each check judged by its latest run.
- No code from an abandoned approach is left in the diff.

| Unit | Done when                                                                                                                                              |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| U1   | An event exposes its headers and arrival time.                                                                                                         |
| U9   | The startup line's `test_command` posts a signed sample that verifies.                                                                                 |
| U2   | A held port binds to `AddressInUse`, and the envelope golden shows `address-in-use` with a `--port 0` command, and a template when headers were given. |
| U3   | A forward's request shape, retries, and refusals pass against local mocks.                                                                             |
| U4   | One signed event and U9's sample reach a URL byte for byte, and failures and shutdown produce their stderr records.                                    |
| U8   | One signed event reaches a program's stdin byte for byte, and a failing or hung program is reported and reaped.                                        |
| U5   | Window, clamp, and refusals pass at fixed times; a write survives as the last record.                                                                  |
| U6   | `replay --from-checkpoint` sends the computed window, and its dry run shows the window and sends nothing.                                              |
| U7   | The README, examples page, and `KNOWN_DIFFERENCES.md` describe what the binary does.                                                                   |

---

## Eng Review

Target: `docs/plans/2026-10-22-1551-feat-webhooks-listen-forwarding-plan.md`, reviewed with `/plan-eng-review` on
2026-10-22 against `dev` at `9f692ba`.

Scope record: feature answers: none proposed; structure: A (D1, Original arrangement); accepted scope: U1 through U8 as
planned; pending remedies: F1, F2, F4b.

### Scope Challenge findings

1. F1 [P2] (confidence: 9/10) `crates/xurl-cli/src/cli/output/mod.rs:721` —
   `Reason::from_kind(error.kind()).unwrap_or(Reason::Internal)`: U1 adds `Error::AddressInUse` before U2 adds its
   reason, so between the two merges a taken port reports `internal`. Reviewer: plan-eng-review, prior learning
   `unit-ordering-breaks-zero-broken-tests`.
2. F2 [P1] (confidence: 9/10) `crates/xurl-cli/src/cli/output/rfc3339.rs:7` —
   `pub(crate) fn utc_rfc3339(epoch_secs: u64) -> String` is the tree's only date code, in the binary and format-only;
   KTD7 and KTD8 need the library to write, read, and render `yyyymmddhhmm`, and the plan never says how. Reviewer:
   plan-eng-review.
3. F3 [P3] (confidence: 10/10) plan line 13 — the Base names `docs/webhooks-06-documents`, merged at `9f692ba` and
   deleted. Factual correction, no question.
4. F4a [P2] (confidence: 8/10) KTD6 — "stdin carries the body and is then closed": an inline write blocks past the
   attempt timeout when the body (up to 1 MB, `DEFAULT_MAX_BODY_BYTES`) outgrows the pipe buffer and the program never
   reads; the timeout the plan promises must cover write and wait together. Correction, no question.
5. F4b [P1] (confidence: 8/10) KTD6 — "An attempt past its timeout kills the child": the kill reaches the direct child
   only, so a script's own children outlive every timeout (tokio's `kill_on_drop` test shows a surviving grandchild).
   Reviewer: plan-eng-review, search check.
6. F5 [P3] (confidence: 7/10) KTD12 — the drain must close each queue, drain it to `None`, and abort what is left at the
   deadline, or it can wait on a leftover sender. Mechanism under the plan's own deadline; correction, no question.

## Decision ledger

### R1: Where the taken-port error lands (F1)

- **Finding:** F1, P2, confidence 9/10, `crates/xurl-cli/src/cli/output/mod.rs:721`, plan-eng-review
- **Plan baseline:** original proposal — U1 adds `Error::AddressInUse` in `crates/xdk/src/error.rs`; U2 adds
  `address-in-use` to the closed reason set, the hint, and the README row.
- **Runtime evidence:** an unknown kind renders as `Reason::Internal` (`output/mod.rs:721`); #325 (`5fa91c6`) changed
  `crates/xdk/src/error.rs` with `envelope.rs` and `hints.rs` in one PR.
- **Comparison grid:**

| Choice                                                     | Current                       | A                    | B                                             | C                             |
| ---------------------------------------------------------- | ----------------------------- | -------------------- | --------------------------------------------- | ----------------------------- |
| R1 placement of `Error::AddressInUse`                      | U1                            | U2                   | U1                                            | U1                            |
| R1 placement of the `address-in-use` reason and README row | U2                            | U2                   | U1                                            | U2                            |
| R1 behavior on `dev` between the U1 and U2 merges          | taken port reports `internal` | unchanged `io`       | `address-in-use`, with no `--port 0` step yet | taken port reports `internal` |
| D1 structure (approved)                                    | Original arrangement          | Original arrangement | Original arrangement                          | Original arrangement          |
| R2 checkpoint date handling                                | pending                       | pending              | pending                                       | pending                       |
| R3 program process-group kill                              | pending                       | pending              | pending                                       | pending                       |

```text
Question D2:
D2 — Which PR should carry the new taken-port error?
Project/branch/task: dev, listen forwarding plan, units U1 and U2.
ELI10: U1 teaches the library a new error kind, "address-in-use". The binary only learns that word in U2. If U1 merges first, the binary sees an unknown word and reports the taken port as an internal xr bug, telling the agent to file an issue, until U2 merges.
Stakes if we pick wrong: a merge order other than a single atomic stack merge leaves dev mislabeling a taken port as an xr bug.
Recommendation: A because it keeps U1 library-only and puts the whole error change in one PR, as #325 did, so dev is right at every merge.
Completeness: A=10/10, B=10/10, C=6/10
Pros / cons:
A) Move it into U2 (recommended)
  ✅ U1 stays a pure library change: event headers and arrival time, nothing else
  ✅ The variant, its reason, its step, and the README row land and get tested together
  ❌ U2 now spans both crates, so its diff mixes xdk and xr files in one review
B) Keep it in U1, with the reason
  ✅ The library variant still lands with the earliest library unit, as the plan says
  ✅ dev reports address-in-use from U1 onward, never internal
  ❌ U1 grows a binary change, and the step without --port 0 ships one PR early
C) Leave it as planned
  ✅ No change to the plan; an atomic gh stack merge never exposes the gap
  ❌ Any piecemeal merge leaves dev calling a taken port an xr bug, untested
Net: one PR per error contract against the plan's strict library-then-CLI split.
Header: Error placement
Options:
A) Move it into U2 (recommended)
U2 adds `Error::AddressInUse` (kind, exit code 5, next action, docs arms) together with the `address-in-use` reason, the hint, the README row, and their tests. U1 keeps only the event headers and arrival time. (human: ~10 min / CC: ~2 min)
B) Keep it in U1, with the reason
U1 adds `Error::AddressInUse` and the `address-in-use` reason plus the README row; U2 adds only the hint with its `--port 0` step. (human: ~10 min / CC: ~2 min)
C) Leave it as planned
U1 adds the variant and U2 the reason, as written; correctness between the two merges relies on landing the stack atomically. (human: none / CC: none)
```

- **State:** approved
- **Actual answer:** A) Move it into U2 (D2, 2026-10-22)
- **Accepted scope:** U2 adds `Error::AddressInUse` (kind, exit code 5, next action, docs arms) with the
  `address-in-use` reason, the hint, the README row, and their tests; U1 keeps only the event headers and arrival time.
- **History:** none

### Corrections applied (no question needed)

- F3: the Base now names `dev`, which carries #333 to #339 after `gh stack merge 335` (`9f692ba`); the execution profile
  stacks on `dev`, and the risk about the unmerged stack is removed.
- F4a: KTD6 and U8 write the program's stdin from its own task under one timeout around the write and the wait; U8 gains
  a scenario with a program that never reads a 200 KB body. Evidence: tokio's `Child` docs on writing stdin from a
  separate task; `DEFAULT_MAX_BODY_BYTES` is 1 MB (`crates/xdk/src/webhooks/mod.rs:51`).
- F5: KTD12 closes each queue, drains it to its end, and at the deadline aborts the workers and counts what is still
  queued or in flight. Evidence: tokio's mpsc `Receiver::close` docs and the graceful-shutdown guide.

### R2: How the checkpoint stores and renders time (F2)

- **Finding:** F2, P1, confidence 9/10, `crates/xurl-cli/src/cli/output/rfc3339.rs:7`, plan-eng-review
- **Plan baseline:** original proposal — KTD7 writes `{"last_event_at": "<RFC 3339 UTC>"}`; KTD8 computes `yyyymmddhhmm`
  in the library; neither says where the date code lives.
- **Runtime evidence:** the tree's only date code is the binary's crate-private `utc_rfc3339`, a formatter built on
  `civil_from_days` (`rfc3339.rs:7`, `:24`); no crate in the workspace parses dates; `retry_at` uses that formatter.
- **Comparison grid:**

| Choice | Current | A | B | C |
| --- | --- | --- | --- | --- |
| R2 checkpoint field | RFC 3339 string | Unix seconds | RFC 3339 string | RFC 3339 string |
| R2 where date code lives | unspecified; binary-only formatter | `xdk`, one formatter that `retry_at` also uses | `xdk`, the moved formatter plus a hand-written parser | `xdk`, through a date crate |
| R2 new dependency | none | none | none | `jiff` in `xdk-rs` |
| R2 parsing | none written | none needed | hand-written RFC 3339 parser and its tests | the crate's |
| R1 error placement (approved) | U2 | U2 | U2 | U2 |
| D1 structure (approved) | Original arrangement | Original arrangement | Original arrangement | Original arrangement |
| R3 program process-group kill | pending | pending | pending | pending |

```text
Question D3:
D3 — How should the restart checkpoint store time?
Project/branch/task: dev, listen forwarding plan, units U5 and U6.
ELI10: The checkpoint file remembers when the last event arrived, and the replay command turns that into X's yyyymmddhhmm window. The plan writes a human-readable timestamp, but the codebase has no way to read one back: its only date code prints timestamps, lives in the binary, and parses nothing. The library needs to read and write this file.
Stakes if we pick wrong: the checkpoint is a stored file format, so changing it after release breaks every checkpoint a running agent already has.
Recommendation: A because a number needs no parser, adds no dependency, and moving the one formatter into the library gives retry_at and the replay window a single date routine.
Completeness: A=10/10, B=9/10, C=10/10
Pros / cons:
A) Unix seconds, shared formatter (recommended)
  ✅ No parser to write or test: the file holds a plain integer the library reads back
  ✅ One date formatter in xdk serves both retry_at and the replay window, no copy
  ❌ The file is less readable by eye: a person sees 1792702800, not a date
B) RFC 3339, hand-written parser
  ✅ The checkpoint stays readable by eye, exactly as the plan wrote it
  ✅ No new dependency, and the formatter still moves into xdk once
  ❌ About 60 more lines of parsing with its own edge cases: offsets, fractions, leap days
C) RFC 3339 through a date crate
  ✅ Battle-tested parsing and formatting with no hand-written date math at all
  ✅ Leaves room for richer time handling later without another decision
  ❌ A new dependency in every embedder's build and in the release binary's size
Net: a readable file against no parser and no new dependency.
Header: Checkpoint time
Options:
A) Unix seconds, shared formatter (recommended)
The checkpoint holds {"last_event_at": <Unix seconds>}. The binary's civil_from_days formatter moves into xdk, crate-internal there plus a doc-hidden pub path for xr's retry_at, and gains the yyyymmddhhmm form the window needs. (human: ~2 h / CC: ~15 min)
B) RFC 3339, hand-written parser
The checkpoint keeps the RFC 3339 string. The formatter moves into xdk as in A, and xdk gains a parser for the UTC Z form the writer produces, with tests for leap days and malformed input. (human: ~4 h / CC: ~30 min)
C) RFC 3339 through a date crate
xdk-rs takes jiff for writing, parsing, and the yyyymmddhhmm form; the binary's formatter is replaced by it. (human: ~2 h / CC: ~15 min, plus a dependency review)
```

- **State:** approved
- **Actual answer:** A) Unix seconds, shared formatter (D3, 2026-10-22)
- **Accepted scope:** the checkpoint holds `{"last_event_at": <Unix seconds>}`; the binary's `civil_from_days` formatter
  (`crates/xurl-cli/src/cli/output/rfc3339.rs`) moves into `xdk`, crate-internal there plus a doc-hidden `pub` path for
  `xr`'s `retry_at`, and gains the `yyyymmddhhmm` form the window needs; its existing table test moves with it and
  `retry_at` output stays byte-identical.
- **History:** none

### R3: What a timed-out program attempt kills (F4b)

- **Finding:** F4b, P1, confidence 8/10, plan KTD6 ("An attempt past its timeout kills the child"), plan-eng-review
  search check
- **Plan baseline:** original proposal — on timeout the direct child is killed; nothing is said about processes the
  program starts.
- **Runtime evidence:** tokio's `kill_on_drop` test spawns a grandchild that survives the kill; `xr` links `libc` on
  Unix (`crates/xurl-cli/Cargo.toml:66`) and already makes one `unsafe` `libc` call (`crates/xurl-cli/src/main.rs:6`);
  the CLI crate does not forbid `unsafe` (only `crates/xdk/src/lib.rs:6` does); `windows-sys` 0.61.2 is already locked
  through `tokio`, `mio`, `socket2`, `colored`, and `tempfile` (`Cargo.lock`); pre-push runs
  `cargo clippy --target x86_64-pc-windows-gnu` (`scripts/hooks/pre-push:326`), and no CI job runs Windows tests; cargo
  and rustc's bootstrap kill trees on Windows with a `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` Job Object.
- **Comparison grid:**

| Choice | Current | A | B | C |
| --- | --- | --- | --- | --- |
| R3 what a Unix timeout kills | direct child | the program's whole process group | direct child | the program's whole process group |
| R3 what a Windows timeout kills | direct child | direct child, documented | direct child | the program's Job Object |
| R3 new crate in the graph | none | none | none | none: a `cfg(windows)` edge to the locked `windows-sys` 0.61 |
| R3 code isolation | none | `#[cfg(unix)]` module | none | `#[cfg(unix)]` and `#[cfg(windows)]` modules behind one function |
| R3 test | none | Unix: a program that backgrounds `sleep` leaves no survivor | none | Unix as A; Windows compiled and linted by pre-push, never run |
| R1 error placement (approved) | U2 | U2 | U2 | U2 |
| R2 checkpoint time (approved) | Unix seconds | Unix seconds | Unix seconds | Unix seconds |
| D1 structure (approved) | Original arrangement | Original arrangement | Original arrangement | Original arrangement |

```text
Question D5:
D5 — When a program attempt times out, what should xr kill? (reopened with the Windows facts)
Project/branch/task: dev, listen forwarding plan, unit U8 (the program after --).
ELI10: When the caller's program takes too long, listen kills it. A script often starts helpers of its own, and killing only the script leaves them running. Unix fixes this with a process group; Windows with a Job Object, which is what cargo itself uses. The Windows crate is already in the lockfile, and target-specific dependency tables keep it out of Linux and macOS builds entirely.
Stakes if we pick wrong: every timed-out event can leave orphaned processes on the caller's machine, on whichever platforms the plan leaves out.
Recommendation: C because the Windows half adds no crate to the graph and no byte to the Unix binaries, and xr's quality bar names Windows (MSVC) as a supported platform.
Completeness: A=9/10, B=5/10, C=10/10
Pros / cons:
A) Kill the group on Unix only
  ✅ A timed-out script and its helpers stop together on Linux and macOS, with no orphans
  ✅ One unsafe libc call beside main.rs's, and a test that runs in CI
  ❌ Windows keeps leaving a script's helpers behind on every timeout, documented only
B) Direct child only, as written
  ✅ No unsafe code and no platform split: the simplest kill tokio offers
  ❌ A script's helpers outlive every timeout and accumulate on every platform
C) Unix group, Windows Job Object (recommended)
  ✅ No orphans on any platform xr supports, using the approach cargo itself relies on
  ✅ No new crate and no change to Unix binaries: a cfg(windows) edge to locked windows-sys
  ❌ The Windows path is compiled and linted on push but never run by any test
Net: Windows parity for about an hour more and an untested Windows path, against a documented Windows gap.
Header: Program kill
Options:
A) Kill the group on Unix only
On Unix the program spawns in its own process group, and a timeout, the drain deadline, or a second signal kills the group with killpg through the libc xr already links, then reaps the child. On Windows the direct child is killed, and the README says so. (human: ~3 h / CC: ~20 min)
B) Direct child only, as written
The timeout kills the direct child, as the plan says; the README tells callers that processes their program starts are theirs to stop. (human: ~10 min / CC: ~2 min)
C) Unix group, Windows Job Object (recommended)
As A on Unix. On Windows the program starts suspended, is assigned to a kill-on-close Job Object, then resumes, and a timeout, the drain deadline, or a second signal terminates the job. windows-sys 0.61 goes in a [target.'cfg(windows)'.dependencies] table with the job and thread features; the two platform modules sit behind one kill function; pre-push's cross-target clippy covers the Windows module. (human: ~1 day / CC: ~1 h)
```

- **State:** approved
- **Actual answer:** C) Unix group, Windows Job Object (D5, 2026-10-22)
- **Accepted scope:** on Unix the program spawns in its own process group and a timeout, the drain deadline, or a second
  signal kills the group with `killpg` through the linked `libc`, then reaps the child; on Windows the program starts
  suspended, is assigned to a kill-on-close Job Object, then resumes, and the same three events terminate the job;
  `windows-sys` 0.61 goes in a `[target.'cfg(windows)'.dependencies]` table with the job and thread features; a
  `#[cfg(unix)]` and a `#[cfg(windows)]` module sit behind one kill function; pre-push's cross-target clippy covers the
  Windows module; a Unix test proves a program that backgrounds a long-running child leaves no survivor.
- **History:** D4 asked 2026-10-22 with A recommended (Windows treated as a new dependency); the answer asked whether C
  is best practice and how Windows-only code and dependencies stay out of other targets. Research showed `windows-sys`
  0.61.2 already locked, `cfg(windows)` target tables keep it out of Unix builds, and cargo and rustc use kill-on-close
  Job Objects, so the brief was reopened as D5 with C recommended. The D4 brief:

```text
Question D4:
D4 — When a program attempt times out, what should xr kill?
Project/branch/task: dev, listen forwarding plan, unit U8 (the program after --).
ELI10: When the caller's program takes too long, listen kills it. But a script often starts helpers of its own, and killing the script leaves those helpers running with no parent: leftover processes that pile up on every timeout. Putting the program in its own process group lets xr kill the script and everything it started at once.
Stakes if we pick wrong: every timed-out event can leave orphaned processes running on the caller's machine.
Recommendation: A because it closes the orphan gap on Linux and macOS, where xr's tests and most agents run, with the libc xr already links; a Windows Job Object needs a new dependency on a platform no CI runner tests.
Completeness: A=9/10, B=5/10, C=10/10
Pros / cons:
A) Kill the group on Unix (recommended)
  ✅ A timed-out script and every process it started stop together, with no orphans left
  ✅ Uses the libc xr already links on Unix, one unsafe call beside the existing one
  ❌ Windows still kills only the direct child, which the README has to say plainly
B) Direct child only, as written
  ✅ No unsafe code and no platform split: the simplest kill tokio offers
  ❌ A script's helpers outlive every timeout and accumulate on the caller's machine
C) Group on Unix, Job Object on Windows
  ✅ Same no-orphan guarantee on every platform xr supports
  ✅ No README caveat about Windows behaving differently
  ❌ A new windows-sys dependency for a path that no CI runner exercises
Net: no orphans on Unix for one unsafe call, against Windows parity for a new dependency.
Header: Program kill
Options:
A) Kill the group on Unix (recommended)
On Unix the program spawns in its own process group, and a timeout, the drain deadline, or a second signal kills the whole group with killpg through the libc xr already links, then reaps the child. On Windows the direct child is killed, and the README says so. Test: a program that backgrounds sleep 300 and hangs leaves no process from its group. (human: ~3 h / CC: ~20 min)
B) Direct child only, as written
The timeout kills the direct child, as the plan says; the README tells callers that processes their program starts are theirs to stop. (human: ~10 min / CC: ~2 min)
C) Group on Unix, Job Object on Windows
As A on Unix, and on Windows the program is assigned to a Job Object that is terminated on timeout, through a new windows-sys dependency. (human: ~1 day / CC: ~45 min)
```

### Section 1 findings: Architecture

1. A1 [P2] (confidence: 9/10) `reqwest-0.13.5/src/error.rs:301` — `write!(f, " for url ({url})")?;` prints a request's
   full URL, credentials included, and KTD11 echoes the forward URL in the dry run; `--forward https://user:pass@host/x`
   would put that password in dry-run output, every `forward-failed` cause, and `tracing`. Reviewer: plan-eng-review.

Boundaries follow KTD1; each new path's production failure has an answer (target down, program hang, missing program,
full disk under the checkpoint, X refusing a window, a full queue); no new artifact needs distribution.

### R4: A forward URL that carries credentials (A1)

- **Finding:** A1, P2, confidence 9/10, `reqwest-0.13.5/src/error.rs:301`, plan-eng-review
- **Plan baseline:** original proposal — KTD5 takes `http` or `https`; KTD11 reports the forward URL in the dry run;
  KTD10 records the cause of a failed forward.
- **Runtime evidence:** `reqwest`'s error `Display` appends `for url ({url})` with userinfo intact; `Error::without_url`
  (`error.rs:91`) drops it.
- **Comparison grid:**

| Choice | Current | A | B | C |
| --- | --- | --- | --- | --- |
| R4 `--forward` with `user:pass@` | accepted, echoed verbatim | refused offline, reason `validation`, pointing to `-H 'Authorization: ...'` | accepted, password shown as `<redacted>` wherever xr prints the URL | accepted, echoed verbatim |
| R4 URL inside a failure's cause | full URL from `reqwest` | dropped with `without_url`; the record names the target | dropped with `without_url`; the record names the target | full URL from `reqwest` |
| R1 error placement (approved) | U2 | U2 | U2 | U2 |
| R2 checkpoint time (approved) | Unix seconds | Unix seconds | Unix seconds | Unix seconds |
| R3 program kill (approved) | group and Job Object | group and Job Object | group and Job Object | group and Job Object |

```text
Question D6:
D6 — What should listen do with a forward URL that has a username and password in it?
Project/branch/task: dev, listen forwarding plan, units U3 and U4.
ELI10: A URL can carry a login, like https://user:pass@host/hook. The plan prints the forward URL in the dry run, and the HTTP library prints the full URL, password included, in its error messages, which become listen's failure records. An agent that logs stderr would write the password to its logs on every failed event.
Stakes if we pick wrong: a credential typed into --forward ends up in dry-run output, stderr records, and traces.
Recommendation: A because refusing the URL up front is one check, and -H 'Authorization: ...' already carries a credential without ever echoing its value.
Completeness: A=10/10, B=9/10, C=4/10
Pros / cons:
A) Refuse it, point to -H (recommended)
  ✅ One offline check; no code path can ever print a password typed into the URL
  ✅ The caller is steered to -H, whose values xr already never echoes (KTD9, KTD11)
  ❌ A target that only accepts user:pass in the URL needs -H with Basic auth instead
B) Accept it and redact the password
  ✅ Any URL the caller has works unchanged, including ones with an embedded login
  ❌ Every place xr prints the URL must redact, and one missed path leaks the password
C) Leave it as planned
  ✅ No extra code at all; the caller owns what they typed on their own command line
  ❌ The password reaches the dry run, every failure record, and traces
Net: one refusal now against redaction at every place a URL is printed.
Header: Forward URL login
Options:
A) Refuse it, point to -H (recommended)
--forward refuses a URL with a username or password offline, with reason validation and a message that names -H 'Authorization: ...'. A failure's cause drops the URL with reqwest's without_url and names the target instead. Tests: a user:pass URL is refused before binding; a failed forward's record holds no URL. (human: ~1 h / CC: ~10 min)
B) Accept it and redact the password
--forward accepts the URL; the dry run, failure records, and tracing show it with the password replaced by <redacted>, and a failure's cause drops the URL with without_url. Tests: each output path shows <redacted> and never the password. (human: ~2 h / CC: ~15 min)
C) Leave it as planned
The URL is accepted and printed as given; a failure's cause includes reqwest's full URL. (human: none / CC: none)
```

- **State:** approved
- **Actual answer:** A) Refuse it, point to -H (D6, 2026-10-22)
- **Accepted scope:** `--forward` refuses a URL with a username or password offline, with reason `validation` and a
  message that names `-H 'Authorization: ...'`; a failure's cause drops the URL with `reqwest`'s `without_url` and names
  the target instead; tests: a `user:pass` URL is refused before binding, and a failed forward's record holds no URL.
- **History:** none

### Section 2 findings: Code quality

1. C1 [P2] (confidence: 9/10) `crates/xdk/src/api/request/transport.rs:226` —
   `if let Some((key, value)) = header.split_once(':') {`: raw mode skips a header with no colon, and the plan never
   says what the forward does with one; copying raw mode would drop `-H 'X-Token abc'` without a word, and the target
   would reject every event. Reviewer: plan-eng-review.
2. C2 [P3] (confidence: 7/10) KTD5 lets `-H` set any name but `Host` and `Content-Length`, so a caller can replace X's
   signature headers; the target then fails to verify. Self-inflicted and visible on the first event; no change
   proposed.

Raw mode's parser is not shared with the forward: raw mode skips a malformed header and the forward refuses one, an
incompatible contract under the shared-code rubric. The shared pieces are already settled: one date formatter (R2) and
one retry budget for two callers (KTD4).

### R5: What the forward does with a malformed `-H` (C1)

- **Finding:** C1, P2, confidence 9/10, `crates/xdk/src/api/request/transport.rs:226`, plan-eng-review
- **Plan baseline:** original proposal — KTD5 forwards the caller's `-H` headers and refuses only `Host` and
  `Content-Length`; nothing is said about a malformed one.
- **Runtime evidence:** raw mode skips an entry with no colon (`transport.rs:226`), and its test
  `user_supplied_header_false_for_unparseable_entry` (`transport.rs:546`) pins that.
- **Comparison grid:**

| Choice | Current | A | B | C |
| --- | --- | --- | --- | --- |
| R5 `-H` with no colon, an empty name, or an invalid name or value | unspecified | refused offline before binding, reason `validation`, step `show-help` | skipped, as raw mode does | unspecified; the implementer decides |
| R5 raw mode's own header handling | skips malformed | unchanged | unchanged | unchanged |
| R4 forward URL login (approved) | refused | refused | refused | refused |
| R1, R2, R3 (approved) | as approved | as approved | as approved | as approved |

```text
Question D7:
D7 — What should listen do with a malformed -H header on the forward?
Project/branch/task: dev, listen forwarding plan, units U3 and U4.
ELI10: Raw mode quietly ignores a header written without a colon, like -H 'X-Token abc'. If the forward copies that, listen starts fine, drops the header, and every event reaches the target without its credential, so the target rejects them all and the caller sees only failures.
Stakes if we pick wrong: one typo in a header turns into every forwarded event failing, with nothing pointing at the typo.
Recommendation: A because listen runs for hours, and refusing the typo before it binds is the only place the caller is still watching.
Completeness: A=10/10, B=6/10, C=5/10
Pros / cons:
A) Refuse it before binding (recommended)
  ✅ The typo is caught in the first second, with the bad header named in the message
  ✅ Raw mode's existing behavior stays exactly as it is, so no other command changes
  ❌ The forward and raw mode treat the same malformed -H differently
B) Skip it, as raw mode does
  ✅ One behavior for -H everywhere in xr, matching the existing raw-mode test
  ❌ listen runs with the header silently missing, and every event fails at the target
C) Leave it unspecified
  ✅ No decision now; the implementer picks when writing the parser
  ❌ The behavior is chosen by accident and nothing tests it
Net: catching a typo at startup against one shared rule for -H.
Header: Malformed header
Options:
A) Refuse it before binding (recommended)
A -H value with no colon, an empty name, or a name or value HTTP does not allow is refused before listen binds, with reason validation, the show-help step, and the offending header named. Raw mode keeps skipping. Tests: each malformed shape is refused offline. (human: ~30 min / CC: ~5 min)
B) Skip it, as raw mode does
The forward drops a malformed -H the way transport.rs:226 does, and listen starts. Test: the header is absent from the forwarded request. (human: ~15 min / CC: ~3 min)
C) Leave it unspecified
The plan says nothing; the implementer decides while writing the forward's header handling. (human: none / CC: none)
```

- **State:** approved
- **Actual answer:** A) Refuse it before binding (D7, 2026-10-22)
- **Accepted scope:** a `-H` value with no colon, an empty name, or a name or value HTTP does not allow is refused
  before `listen` binds, with reason `validation`, the `show-help` step, and the offending header identified by its
  position among the `-H` flags (and its name when one parses), never by a value, which carries the never-echo rule of
  KTD9 and KTD11; raw mode keeps skipping; tests: each malformed shape is refused offline.
- **History:** none

### Section 3: Test review

Framework: `cargo test` (AGENTS.md § Testing). Three behaviors the plan states had no scenario; each is required proof
of that behavior, added without a question:

- G1: a failed checkpoint write is reported and `listen` keeps running (U6 approach) — added to U6.
- G2: a stalled target never delays the other target (KTD3) — added to U8.
- G3: an unsigned event under `--allow-unsigned` is forwarded without a signature header (KTD5) — added to U4.

```text
CODE PATHS                                              USER FLOWS
[+] xdk::webhooks Receiver (U1, U2)                     [+] Agent forwards events (U4, U8)
  ├── receive_event headers + arrival   [★★★ U1]          ├── [★★★] URL + program, max-events 1 [→E2E] (success criterion)
  └── bind: AddrInUse vs other io       [★★★ U2]          ├── [★★★] target down: retried, reported, listen keeps going
[+] xdk::webhooks forward (U3)                            ├── [★★★] stalled URL target, program unaffected (G2)
  ├── request shape, signature, -H      [★★★ U3]          └── [★★★] SIGINT drain, deadline, second signal
  ├── retry: 503,503,200 / 400 / hang   [★★★ U3]        [+] Agent recovers after a restart (U5, U6)
  ├── refusals: scheme, user:pass, -H   [★★★ U3, U4]      ├── [★★★] checkpoint once a minute, read back
  └── capped response read              [★★  U3]          ├── [★★★] replay --dry-run shows window, sends nothing
[+] xr program target (U8)                                ├── [★★★] window clamped at 24 h, reported
  ├── stdin from own task, timeout      [★★★ U8]          └── [★★★] checkpoint unwritable (G1)
  ├── group / Job Object kill           [★★★ U8 Unix + windows-latest job]
  └── no shell: event bytes on stdin    [★★★ U8]        [+] Agent hits a taken port (U2)
[+] xdk::webhooks checkpoint (U5)                         └── [★★★] address-in-use + --port 0 command or template
  ├── window, clamp, future, same min   [★★★ U5]
  ├── io refusals: missing, not json    [★★★ U5]        REGRESSION (CRITICAL)
  └── moved UTC formatter               [★★★ U5]          ├── retry_at byte-identical (rate-limited-with-reset golden)
                                                          ├── listen stdout: listening line + one line per event
                                                          └── replay --from/--to unchanged
COVERAGE: 26/26 planned paths have a ★★ or ★★★ scenario after G1–G3  |  GAPS: 0
Windows Job Object path: run by U8's windows-latest CI job (R6, D8)
```

Tests made obsolete by this plan: none.

### Section 4: Performance review

1. P1 [P3] (confidence: 7/10) KTD3 — each target's queue holds up to 256 events, and if each holds its own copy of the
   body, two targets at the receiver's 1 MB cap (`crates/xdk/src/webhooks/mod.rs:51`) could hold 512 MB. X's event
   bodies are a few KB, so the realistic peak is about 1 MB. Sharing one copy of the body across targets halves even the
   worst case; an implementation detail with no behavior change, left to U4.

The checkpoint costs one fsync a minute (KTD7); a process per event at X's per-account delivery rate has no measured
cost (scale unknown); no query, cache, or unbounded collection is added.

### Outside voice

Disabled: `codex_reviews` is `disabled`; no outside process or native reviewer ran, and the disabled result is recorded
in the review log.

### R6: A Windows test run for the program target (TODO)

- **Finding:** TODO candidate from R3, P2, confidence 9/10, `.github/workflows/ci.yml` (every job
  `runs-on: ubuntu-latest`), plan-eng-review
- **Plan baseline:** R3 (D5) accepted a Windows Job Object path that pre-push compiles and lints
  (`scripts/hooks/pre-push:326`) and no test runs.
- **Runtime evidence:** no CI job uses a Windows runner; the release workflow builds Windows binaries without running
  tests.
- **What:** run the program-target tests (and the suite) on a `windows-latest` runner, with a Windows twin of the
  group-kill test that proves a grandchild dies with the Job Object.
- **Why:** the Job Object code is the one path in this plan no test executes; a bug there reaches only Windows users,
  unseen.
- **Pros:** proves the Windows half; catches the suspended-spawn and handle-inheritance mistakes the search turned up in
  other projects.
- **Cons:** a new CI job on every PR, slower than the Linux jobs, plus porting the `sh`-based test programs to something
  that runs on Windows.
- **Context:** three months from now, start at `.github/workflows/ci.yml` and the program-target tests in
  `crates/xurl-cli/tests/webhooks_forward_tests.rs`; process-wrap's issue tracker lists nested-job `ERROR_ACCESS_DENIED`
  on GitHub-hosted Windows runners, worth checking first.
- **Depends on / blocked by:** U8 landing.
- **Comparison grid:**

| Choice | Current | A | B | C |
| --- | --- | --- | --- | --- |
| R6 Windows test run | none | recorded in `TODO.md` for after U8 | none | built in this plan as a CI job beside U8 |
| R3 program kill (approved) | group and Job Object | group and Job Object | group and Job Object | group and Job Object |
| R1, R2, R4, R5 (approved) | as approved | as approved | as approved | as approved |

```text
Question D8:
D8 — Should running the program-target tests on Windows be tracked as a TODO?
Project/branch/task: dev, listen forwarding plan, after unit U8.
ELI10: You chose to kill a timed-out program's whole tree on Windows with a Job Object. That code compiles and lints on every push, but no test ever runs it, because CI only has Linux machines. A Windows CI job would run it for real.
Stakes if we pick wrong: a bug in the Windows kill path ships unseen and leaves orphans only on Windows machines.
Recommendation: A because the plan is already eight PRs, and a Windows runner is its own CI change best made once U8 exists to test.
Completeness: A=8/10, B=5/10, C=10/10
Pros / cons:
A) Add to TODO.md (recommended)
  ✅ The gap is written down with where to start, so it is not forgotten after U8 lands
  ✅ Keeps this plan's CI unchanged while the forwarding work is still in review
  ❌ Until someone picks it up, the Windows Job Object path stays untested in practice
B) Skip it
  ✅ No new work and no CI cost; the pre-push cross-target clippy stays the only check
  ❌ Nothing records that the Windows half has never run, so the gap is easy to forget
C) Build it now in this plan
  ✅ The Windows kill path is proven before release, matching the Unix guarantee
  ❌ Adds a Windows CI job and test porting to a plan that is already eight PRs
Net: tracking the Windows gap for later against proving it before release.
Header: Windows tests
Options:
A) Add to TODO.md (recommended)
A TODO.md entry under Open records the What, Why, Context, and the U8 dependency, for a windows-latest CI job and a Windows twin of the group-kill test. (human: ~5 min / CC: ~1 min)
B) Skip it
No entry; the Windows path keeps only pre-push's cross-target clippy. (human: none / CC: none)
C) Build it now in this plan
U8 grows a windows-latest CI job running the program-target tests, with the test programs ported off sh. (human: ~1 day / CC: ~1 h)
```

- **State:** approved
- **Actual answer:** C) Build it now in this plan (D8, 2026-10-22)
- **Accepted scope:** U8 grows a `windows-latest` CI job that runs the program-target tests, with the test programs
  ported off `sh`, and a Windows twin of the group-kill test that proves a grandchild dies with the Job Object; no
  `TODO.md` entry, since the work is in the plan.
- **History:** none

Approval readiness: PASS — R1 (D2: A), R2 (D3: A), R3 (D5: C; D4 reopened), R4 (D6: A), R5 (D7: A), R6 (D8: C);
structure by the D1 scope record; F3, F4a, F5 applied as corrections; G1 to G3 added as required proof.

### NOT in scope

- Query-string secrets in a `--forward` URL: only userinfo is refused (R4); a token in the query is the caller's to keep
  off a printed URL, and the dry run prints it as given.
- Changing raw mode's handling of a malformed `-H`: it keeps skipping (R5); only the forward refuses.
- Required-check status for the new `windows-latest` job: decided in U8's PR, since rulesets are applied by Brett.
- Everything the plan's own Scope Boundaries lists: tunnel, typed payloads, the Account Activity to X Activity
  migration, dedupe across restarts, dead-letter file, re-signing, fan-out, TLS skip-verify.

### What already exists

- The receiver, `Event`, and the bind path: `crates/xdk/src/webhooks/mod.rs` (U1 and U2 extend them).
- Atomic writes: `write_atomically`, `crates/xdk/src/store/atomic.rs:142` (reused by the checkpoint, KTD7).
- The one UTC formatter: `crates/xurl-cli/src/cli/output/rfc3339.rs:7`, moved into `xdk` and shared with `retry_at`
  (R2).
- Step building from the invocation: `NextStep::confirm` with `shell_word`, `crates/xurl-cli/src/cli/hints.rs:210`
  (reused by U2's `--port 0` step).
- The hint chain: `crates/xurl-cli/src/cli/runner.rs:371` (U2 adds beside `resume_wait_hint`).
- A hardened spawn split into builder and spawn: `crates/xurl-cli/src/cli/skill_install/git.rs:52` (pattern for U8).
- `libc` on Unix: `crates/xurl-cli/Cargo.toml:66`; `windows-sys` 0.61.2 already locked (R3).
- The replay request: `create_webhook_replay`, `crates/xdk/src/api/shortcuts.rs:1159`, and `validate_replay_time`,
  `:325` (U6 reuses both).
- Not reused, on purpose: raw mode's header parser (`transport.rs:226`) skips malformed headers, an incompatible
  contract (R5); `reqwest`'s built-in retries cannot cover the program target (KTD4).

### Diagrams

The plan's High-Level Technical Design carries the event flow, the shutdown sequence, the restart recovery, and the flag
matrix, all updated for R2, R3, and KTD12. The Section 3 coverage diagram above maps every planned path. No touched file
needs an inline diagram.

### Failure modes

| New path | Realistic failure | Handled by | Test | What the caller sees |
| --- | --- | --- | --- | --- |
| URL forward | target down or answering 5xx | retry budget, then a `forward-failed` record (KTD4, KTD10) | U3, U4 | a stderr record; events keep flowing |
| URL forward | URL with a login | refused before binding (R4) | U3, U4 | `validation`, pointing to `-H` |
| `-H` | typo with no colon | refused before binding (R5) | U3, U4 | `validation`, naming the header's position |
| Program target | never reads stdin | write in its own task under the timeout (F4a) | U8 | killed at the timeout, reported |
| Program target | hangs with children of its own | group or Job Object kill (R3) | U8, Windows job | killed with its children, reported |
| Program target | not on `PATH` | resolved before binding (KTD6) | U8 | `validation` at startup |
| Shutdown | target never answers with events queued | one drain deadline, then abort and count (KTD12, F5) | U4 | `forward-incomplete` with the count |
| Checkpoint | directory not writable | write failure reported, listen continues (G1) | U6 | a stderr record per failed write |
| Replay | checkpoint missing or corrupt | refused, reason `io` (KTD8) | U5, U6 | exit 5, `fix-input` |
| Replay | gap longer than 24 hours | window clamped, reported (KTD8) | U5, U6 | `clamped: true` and the first lost minute |
| Bind | port taken | `address-in-use` with a `--port 0` step (U2) | U2 | exit 5, a runnable command or template |

Critical gaps: none. Every path has handling, a test, and a visible result.

### Worktree parallelization strategy

| Step | Modules touched | Depends on |
| --- | --- | --- |
| U1 event headers and arrival time | `xdk` webhooks | — |
| U2 taken-port error and step | `xdk` error and webhooks, `xr` output and hints | — |
| U3 URL forward and retry budget | `xdk` webhooks | U1 |
| U4 `--forward` and queues | `xr` commands and CLI | U3 |
| U8 program target and Windows job | `xr` commands, CLI, CI | U4 |
| U5 checkpoint and window | `xdk` webhooks and store, `xr` output | U1 |
| U6 `--checkpoint` and `--from-checkpoint` | `xr` commands and CLI | U4, U5 |
| U7 documents | docs and examples | U2, U4, U6, U8 |

Lanes exist on paper (U2 and U5 do not depend on the forward path), but xurl-rs runs with no worktrees or subagents, so
the work lands as one stack in the order U1, U2, U3, U4, U8, U5, U6, U7. Shared module: `crates/xdk/src/webhooks/mod.rs`
(U1, U2, U3, U5), which is why the stack is sequential either way.

## Implementation Tasks

Synthesized from this review's findings. Each task derives from a specific finding above.

- [ ] **T1 (P1, human: ~2 h / CC: ~15 min)** — xdk checkpoint — Store Unix seconds and move the UTC formatter into `xdk`
  - Surfaced by: Scope Challenge — F2, R2 (D3: A)
  - Files: `crates/xdk/src/webhooks/checkpoint.rs`, the moved `rfc3339.rs`, `crates/xurl-cli/src/cli/output/mod.rs`
  - Verify: `cargo test -p xdk-rs --test webhook_checkpoint_tests`; the `rate-limited-with-reset` golden is unchanged
- [ ] **T2 (P1, human: ~1 day / CC: ~1 h)** — xr program target — Kill the process group on Unix and the Job Object on
  Windows
  - Surfaced by: Scope Challenge — F4b, R3 (D5: C)
  - Files: the kill module, `crates/xurl-cli/Cargo.toml`, `crates/xurl-cli/tests/webhooks_forward_tests.rs`
  - Verify: the group-kill test leaves no survivor on Linux; the Windows twin passes on `windows-latest`
- [ ] **T3 (P2, human: ~1 day / CC: ~1 h)** — CI — Add a `windows-latest` job and port the test programs off `sh`
  - Surfaced by: Final planning decisions — R6 (D8: C)
  - Files: `.github/workflows/ci.yml`, the test programs, `crates/xurl-cli/tests/webhooks_forward_tests.rs`
  - Verify: the job is green on the U8 PR
- [ ] **T4 (P2, human: ~30 min / CC: ~5 min)** — U2 — Land `Error::AddressInUse` with its reason, step, and README row
  in one PR
  - Surfaced by: Scope Challenge — F1, R1 (D2: A)
  - Files: `crates/xdk/src/error.rs`, `crates/xdk/src/webhooks/mod.rs`, `crates/xurl-cli/src/cli/envelope.rs`,
    `hints.rs`, `runner.rs`
  - Verify: `next_step_tests.rs` and the `reason-address-in-use` golden
- [ ] **T5 (P2, human: ~1 h / CC: ~10 min)** — xdk forward — Refuse a `--forward` URL with a login and keep URLs out of
  causes
  - Surfaced by: Architecture — A1, R4 (D6: A)
  - Files: `crates/xdk/src/webhooks/forward.rs`, `crates/xdk/tests/webhook_forward_tests.rs`
  - Verify: the `user:pass` refusal and the no-URL cause scenarios in U3 and U4
- [ ] **T6 (P2, human: ~30 min / CC: ~5 min)** — xdk forward — Refuse a malformed `-H` before binding, by position
  - Surfaced by: Code quality — C1, R5 (D7: A)
  - Files: `crates/xdk/src/webhooks/forward.rs`, `crates/xurl-cli/tests/webhooks_forward_tests.rs`
  - Verify: the malformed-header scenarios in U3 and U4
- [ ] **T7 (P2, human: ~30 min / CC: ~5 min)** — xr program target — Write stdin from its own task under one timeout
  - Surfaced by: Scope Challenge — F4a
  - Files: `crates/xurl-cli/src/cli/commands/webhooks_forward.rs`
  - Verify: the 200 KB never-read scenario in U8
- [ ] **T8 (P3, human: ~30 min / CC: ~5 min)** — xr forward workers — Close, drain, and abort at the deadline
  - Surfaced by: Scope Challenge — F5
  - Files: `crates/xurl-cli/src/cli/commands/webhooks_forward.rs`
  - Verify: the drain-deadline and second-signal scenarios in U4
- [ ] **T9 (P3, human: ~1 h / CC: ~10 min)** — tests — Add the three gap scenarios
  - Surfaced by: Test review — G1, G2, G3
  - Files: `crates/xurl-cli/tests/webhooks_tests.rs`, `crates/xurl-cli/tests/webhooks_forward_tests.rs`
  - Verify: the read-only checkpoint, stalled-target, and unsigned-forward scenarios pass
- [ ] **T10 (P3, human: ~15 min / CC: ~2 min)** — xr forward workers — Share one copy of each event body across target
  queues
  - Surfaced by: Performance — P1
  - Files: `crates/xurl-cli/src/cli/commands/webhooks_forward.rs`
  - Verify: no behavior change; the forward tests still pass

Effort ratios assumed: features ~30x, tests ~50x, architecture ~5x human to CC.

### Unresolved decisions

None. Every choice in this review has an answer.

### Suppressed findings

None: no finding scored below 5.

### Completion summary

- Step 0: Scope Challenge — scope accepted as-is (D1: Original arrangement); 6 findings, all resolved
- Reality rows: 11/11 applicable rows emitted
  (`PLAN_REALITY: phase=eng rows=7/7 missing=none unverified=1 verdict=complete`), 1 unverified premise
- Architecture Review: 1 issue found (A1)
- Code Quality Review: 2 issues found (C1, C2)
- Test Review: diagram produced, 3 gaps identified (G1 to G3, all added)
- Performance Review: 1 issue found (P1)
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 1 item proposed to user (built now, D8: C)
- Failure modes: 0 critical gaps flagged
- Unresolved decisions: 0 in this review
- Outside voice: codex, disabled (`codex_reviews` disabled)
- Parallelization: 3 lanes on paper, 0 parallel / 8 sequential (no worktrees in xurl-rs)
- Lake Score: 6/6

## DX Review

Target: this plan, reviewed with `/plan-devex-review` on 2026-10-22 against `dev` at `9f692ba`. Product type: CLI and
library, both held to best in class (DX D1). Mode: DX Expansion (DX D6).

### Target developer persona

```text
TARGET DEVELOPER PERSONA
========================
Who:       an AI coding agent running xr webhooks listen and replay unattended, and a Rust developer embedding xdk::webhooks in a service (DX D2)
Context:   the agent wants X's events in its own app without polling; the embedder builds the same on the library
Tolerance: the agent reads errors literally, cannot ask, retries, and pays per X call; the embedder reads types and docs.rs
Expects:   one flag to forward, structured records for every outcome, a next_step on every error, and no surprise bill
```

### Developer perspective (today, before this plan)

Agent: "I run `xr webhooks listen --output jsonl` and the first line is
`{"status":"listening","url":"http://127.0.0.1:8080/webhook"}` [O: `webhooks_listen.rs:75`]. I expose it with
cloudflared, then `webhooks add` and `subscriptions add` [O: README Webhooks]. Events arrive as JSON lines on stdout
[O]. To get them into my app I write my own relay, since listen's help has no forward flag [O:
`help-webhooks-listen.golden`]. When listen or my relay restarts, events X delivered meanwhile are gone [O: X docs]. To
recover I work out `yyyymmddhhmm` myself and run `webhooks replay` under a bearer, and X bills it [O: replay help,
README]. A taken port is reason `io` with no command [O: `mod.rs:161`]."

Embedder: "I call `Receiver::bind` and loop `next_event()`, getting `Event { body, signature }` [O: `mod.rs:122`]. No
arrival time and no headers, so I cannot pass X's signature on [O]. A taken port is `Error::Io` with a message [P].
Nothing computes a replay window [O: `shortcuts.rs:325`]."

Confirmed accurate (DX D3).

### Competitive benchmark

Clock: from `listen` up behind the caller's public URL to the first verified event in the agent's own app. The tunnel is
outside the clock: `xr` opens none, by the webhooks plan's settled scope. Every time below is estimated.

| Tool | Start to result | Time, evidence | DX choice | Source |
| --- | --- | --- | --- | --- |
| Stripe CLI | logged in to event at a local URL | under 2 min, estimated | `--forward-to`, `-H "K:V, K:V"`, re-signs with a CLI secret; recovery resends one event at a time | docs.stripe.com/cli/listen |
| Hookdeck CLI | account and source to event at localhost | 2 to 5 min, estimated | forwards to a port or URL; events arriving while stopped are discarded (`CLI_DISCONNECTED`); replay is manual, per event | hookdeck.com/docs/cli |
| svix CLI | none to event at a local URL | under 2 min, estimated | hosted relay URL stable across restarts; replay in its dashboard | svix.com/guides/testing-webhooks-locally-svix-cli |
| smee | channel to event at a local URL | under 2 min, estimated | hosted relay, `--target` | github.com/probot/smee-client |
| adnanh/webhook | `hooks.json` to command run | 5 to 10 min, estimated | a command per hook, request data as args, env, or a file | github.com/adnanh/webhook |
| `xr` today | listen up to event in the agent's app | 10+ min, estimated | stdout only; the agent writes a relay | help, README |
| `xr` with this plan | listen up to event in the agent's app | under 1 min, estimated | `--forward` or a program after `--`; recovery is one `replay --from-checkpoint` | this plan |

Target: Champion, under 2 minutes (DX D4).

### Magical moment

The agent's own app receives an event the instant it arrives, with no relay written. Vehicle: `listen`'s startup line
hands the agent a ready command that posts a sample event signed with the app's secret, so the first test passes
signature verification with no tunnel, no X call, and no bill (DX D5, chosen over an unsigned `curl` recipe and over the
plan's docs as they were). Built as U9. The vehicle's library half is KTD14's builder in `xdk::webhooks`, which
`xdk::testing` also uses for embedders' tests (DX D12).

### Decisions

| ID | Question | Answer | Lands in |
| --- | --- | --- | --- |
| DX D1 | Product type | A) CLI and library, both | every pass |
| DX D2 | Persona | A) AI agent plus Rust embedder | every pass |
| DX D3 | Empathy narrative | A) Accurate | Developer perspective |
| DX D4 | Time target | A) Champion, under 2 min | Pass 1, scorecard |
| DX D5 | First success | B) `listen` prints a signed test command | U9, U7 |
| DX D6 | Review mode | A) DX Expansion | every pass |
| DX D7 | Success visibility | A) a `forwarded` record per success | KTD10, U4, U8 |
| DX D8 | Library teaching | A) a runnable example plus doc examples | U1, U3, U5, U7, U9 |
| DX D9 | Dedupe key | A) a content-hash key on every forward | KTD5, KTD6, U3, U8, U7 |
| DX D10 | Failure recovery | A) a replay template in `next_step` | KTD10, U4, U7 |
| DX D11 | Worker stops | A) fail loudly and exit | KTD13, U4 |
| DX D12 | Test helper | A) one signed-event builder in the library | KTD14, U9 |

### Developer journey map

```text
STAGE          | DEVELOPER DOES                                        | FRICTION POINTS                              | STATUS
---------------|-------------------------------------------------------|----------------------------------------------|-------
1. Discover    | listen --help, examples page, README; crate README    | no webhooks example for embedders (D8)        | fixed
2. Install     | brew or cargo; embedder adds xdk-rs                   | none                                          | ok
3. Hello World | listen --forward, then runs test_command              | first success needed a billed X event (D5)    | fixed
4. Real Usage  | forwards, restarts, replay --from-checkpoint          | repeats undetectable by the target (D9)       | fixed
5. Debug       | reads stderr records                                  | success invisible (D7); failure had no step   | fixed
               |                                                       | (D10); a stopped worker was silent (D11)      |
6. Upgrade     | additive flags, record keys, and library fields       | none                                          | ok
```

### First-time developer confusion report

```text
FIRST-TIME DEVELOPER REPORT
============================
Persona: AI coding agent (with a Rust embedder alongside)
Attempting: xr webhooks listen forwarding, getting started

CONFUSION LOG:
T+0:00  Runs xr webhooks listen --output jsonl --forward http://localhost:3000/x &. Reads the listening record. [addressed: U9 adds test_command]
T+0:10  Runs test_command. The app gets a signed sample; stderr shows a forwarded record. [addressed: D5, D7]
T+0:40  Wonders whether the sample is a real event. The body carries xr_test_event and the README says to ignore it. [addressed: U9, U7]
T+1:30  Exposes the port, runs webhooks add and subscriptions add; real events flow. [outside the clock: no tunnel by settled scope]
T+5:00  Its app was down for a minute: a forward-failed record carries a replay template for that minute. [addressed: D10]
T+6:00  After a restart, runs xr --dry-run webhooks replay <ID> --from-checkpoint FILE, then the real one; repeats share Idempotency-Key. [addressed: D9, U6]
T+7:00  Forgot --checkpoint the first time: replay refuses with reason io and names --from/--to and --checkpoint. [addressed: Pass 3 routine fix]
Final state: succeeded, with no relay written and no lost event.
```

### NOT in scope (DX)

- A shutdown summary record (events received, forwarded, failed): useful polish, not requested; the `forwarded` and
  `forward-failed` records already let an agent count.
- A default checkpoint path: two listeners on one host would collide on it, and an explicit path keeps `listen` and
  `replay` visibly in step.
- Measuring the approved clock with telemetry: `xr` sends none; the `/devex-review` boomerang after release measures it.
- A hosted relay or built-in tunnel, as Stripe, svix, and smee provide: outside `xr` by the webhooks plan's settled
  scope.

### TODOS.md (DX)

No TODO proposed: each deferral above is a settled scope call with its reason, and the one DX debt (measurement) is
answered by the `/devex-review` run after release, not by a TODO.

### What already exists (DX)

- `--allow-unsigned` for local tests, and the public `xdk::webhooks::sign` the U9 builder wraps.
- The `listening` record (`crates/xurl-cli/src/cli/commands/webhooks_listen.rs:75`), which gains `test_command`
  additively.
- `xdk::testing::MockX` and the `testing` feature, the home of the new sender.
- Six `xdk` examples (`crates/xdk/examples/`) and pre-push's `cargo build --examples -p xdk-rs`, which
  `webhook_forward.rs` joins.
- The `next_step` contract with `run-command`, which D10 extends with a template form.
- `family_help` verb pages and the help goldens every new flag updates.

## DX Implementation Tasks

Synthesized from this review's findings. Each task derives from a specific finding above.

- [ ] **T1 (P1, human: ~3 h / CC: ~25 min)** — U9 — Build the signed-sample builder, the `testing` sender, and
  `test_command`
  - Surfaced by: Step 0D and Pass 6 — DX D5, DX D12, KTD14
  - Files: `crates/xdk/src/webhooks/signing.rs`, `crates/xdk/src/testing/mod.rs`,
    `crates/xurl-cli/src/cli/commands/webhooks_listen.rs`
  - Verify: the U9 scenarios, and `cargo tree -p xurl-rs -e features` shows `testing` only under dev-dependencies
- [ ] **T2 (P1, human: ~2 h / CC: ~15 min)** — U4 workers — Supervise workers and exit `internal` when one stops
  - Surfaced by: Pass 3 — CAP4:worker-stop:channel, DX D11, KTD13
  - Files: `crates/xurl-cli/src/cli/commands/webhooks_forward.rs`, `crates/xurl-cli/tests/webhooks_forward_tests.rs`
  - Verify: the panicking-target scenario exits 1 with reason `internal`
- [ ] **T3 (P2, human: ~1 h / CC: ~10 min)** — records — Add the replay template to `forward-failed`
  - Surfaced by: Pass 3 — CAP6:forward-failed:fix,recovery, DX D10
  - Files: `crates/xurl-cli/src/cli/commands/webhooks_forward.rs`, `crates/xurl-cli/README.md`
  - Verify: the template's `--from` is the event's arrival minute and its `--to` the minute after (R13)
- [ ] **T4 (P2, human: ~1 h / CC: ~10 min)** — forward — Send `Idempotency-Key` and `XR_EVENT_SHA256`, and record
  `event_sha256`
  - Surfaced by: Pass 2 — DX D9
  - Files: `crates/xdk/src/webhooks/forward.rs`, `crates/xurl-cli/src/cli/commands/webhooks_forward.rs`
  - Verify: a retried and a replayed delivery of the same bytes carry the same key
- [ ] **T5 (P2, human: ~1 h / CC: ~10 min)** — records — Write a `forwarded` record per success
  - Surfaced by: Step 0F Debug — DX D7
  - Files: `crates/xurl-cli/src/cli/commands/webhooks_forward.rs`
  - Verify: one `forwarded` record per delivered event per target; stdout unchanged
- [ ] **T6 (P2, human: ~3 h / CC: ~20 min)** — xdk docs — Add `examples/webhook_forward.rs` and doc examples
  - Surfaced by: Step 0F Discover — DX D8
  - Files: `crates/xdk/examples/webhook_forward.rs`, `crates/xdk/src/webhooks/`
  - Verify: `cargo build --examples -p xdk-rs` and `cargo test --doc -p xdk-rs`
- [ ] **T7 (P3, human: ~20 min / CC: ~3 min)** — errors — Keep the OS error on `AddressInUse` and name both fixes for a
  missing checkpoint
  - Surfaced by: Pass 3 — the taken-port context and missing-checkpoint fix items
  - Files: `crates/xdk/src/error.rs`, `crates/xdk/src/webhooks/checkpoint.rs`
  - Verify: the U2 and U6 scenarios

Effort ratios assumed: features ~30x, tests ~50x human to CC.

## DX Review Scorecard

```text
+====================================================================+
|              DX PLAN REVIEW — SCORECARD                             |
+====================================================================+
| Dimension            | Score  | Prior  | Trend  |
|----------------------|--------|--------|--------|
| Getting Started      |  9/10  |  6/10  |  +3 ↑  |
| API/CLI/SDK          |  9/10  |  7/10  |  +2 ↑  |
| Error Messages       |  9/10  |  5/10  |  +4 ↑  |
| Documentation        |  9/10  |  7/10  |  +2 ↑  |
| Upgrade Path         |  9/10  |  9/10  |   =    |
| Dev Environment      |  9/10  |  7/10  |  +2 ↑  |
| Community            |  8/10  |  8/10  |   =    |
| DX Measurement       |  4/10  |  3/10  |  +1 ↑  |
+--------------------------------------------------------------------+
| TTHW                 | <1 min | 10+ min| est.  ↑ (listen up -> first event in the app) |
| Competitive Rank     | Champion                                     |
| Magical Moment       | designed via the signed test command (U9)    |
| Product Type         | CLI tool and Rust library                    |
| Mode                 | EXPANSION                                    |
| Overall DX           | 8.3/10 | 6.5/10 | +1.8 ↑ |
+====================================================================+
| DX PRINCIPLE COVERAGE                                               |
| Zero Friction      | covered                                        |
| Learn by Doing     | covered                                        |
| Fight Uncertainty  | covered                                        |
| Opinionated + Escape Hatches | covered                             |
| Code in Context    | covered                                        |
| Magical Moments    | covered                                        |
| Agents Are Users   | covered                                        |
+====================================================================+
```

Prior scores are this review's starting ratings of the plan as written; prior DX reviews of other plans recorded no
dimension scores (rubric changed, not comparable). Every time is estimated.

Critical DX debt: DX Measurement at 4/10. Nothing measures the approved clock; run `/devex-review` after release to
measure "listen up to the first verified event in the agent's app" against the under-2-minute target.

```text
DX IMPLEMENTATION CHECKLIST
============================
[ ] Time to hello world < 2 min (listen up -> first verified event in the agent's app)
[x] Installation is one command (brew or cargo; xdk-rs is one dependency)
[ ] First run produces meaningful output (the listening record with test_command, U9)
[ ] Magical moment delivered via the signed test command (U9)
[ ] Every error message meets the agent-ready error contract (Pass 3 matrix)
[x] API/CLI naming is guessable without docs
[x] Every parameter has a sensible default
[ ] Docs have copy-paste examples that actually work (U7, U9)
[ ] Examples show real use cases, not just hello world (crates/xdk/examples/webhook_forward.rs)
[x] Upgrade path: every change additive, no migration needed
[x] Breaking changes: none
[x] Works in CI/CD without special configuration
[x] Changelog exists and is maintained (generated from PR bodies)
```

Unresolved decisions in this DX review: none.

## Eng Review: second pass

Target: this plan at `03e9ac3`, reviewed again with `/plan-eng-review` on 2026-10-22 after the DX review added KTD13,
KTD14, U9, the content-hash key, the `forwarded` record, the replay template, and the doc examples. Reviewer: Claude,
in-host. Prior learning applied: md-wrap-reflows-ledger-fields (confidence 9/10, 2026-10-22).

### Second-pass scope record

- feature answers: none asked; the DX answers (DX D1 to DX D12) set the added scope.
- structure: Original arrangement (eng D1, first pass), with U9 added as its own unit by DX D5 and DX D12; no new
  structure question, since both answers cover the arrangement exactly.
- accepted scope: as the plan stands at `03e9ac3`, plus the corrections below.
- pending remedies: R7, R8, R9, R10 (Sections 1 and 2).
- Scope Challenge result: scope accepted as-is.

### Second-pass Scope Challenge findings

1. S1 [P2] (confidence: 9/10) plan U9 — `Dependencies: none`, but its scenario "With `--forward <mock>`, running the
   printed command delivers the sample to the mock" and its Definition of Done row ("verifies and forwards") need U4's
   forward, and U9 lands before U3 and U4 in the stack. Correction applied: the scenario and the "forwards" clause move
   to U4, which gains U9 as a dependency.
2. S2 [P3] (confidence: 9/10) plan U9 Patterns — "`NextStep::confirm`'s `shell_word`" names a function that is private
   to `crates/xurl-cli/src/cli/hints.rs:552` (`fn shell_word(arg: &str) -> String {`), and U9 composes its command in
   `webhooks_listen.rs`. Correction applied: U9 builds the command in `hints.rs` beside the other step builders, or
   makes `shell_word` `pub(crate)`.
3. S3 [P3] (confidence: 10/10) plan U9 test scenario — `cargo tree -p xurl-rs -e features` includes dev-dependency
   edges, so it shows `xdk-rs feature "testing"` today (probe, `03e9ac3`);
   `cargo tree -p xurl-rs -e features,no-dev -i xdk-rs` shows only `default` and `rustls`. Correction applied: the
   scenario names the second command.
4. No issue: SHA-256 for the content-hash key needs no new dependency; `sha2 = "0.11"` is already in
   `crates/xdk/Cargo.toml:73`, and hex is a format loop.
5. No issue: `crates/xdk/examples/webhook_forward.rs` is built, never run, by pre-push (`scripts/hooks/pre-push:221`,
   `cargo build --examples -p xdk-rs`); the one example run is `offline_search` (`:226`), so a looping example cannot
   hang the hook.

TODOS cross-reference: `TODO.md` lines 118 and 123 are this plan's two items; line 120 (Account Activity deprecation)
stays deferred, per Scope Boundaries; line 108 (the webhooks stack "in review") is stale, since #333 to #339 merged on
2026-10-22.

Search check: RFC 6648 §3 ("SHOULD NOT prefix their parameter names with "X-""), RFC 9530 (`Content-Digest`), and
draft-ietf-httpapi-idempotency-key-header-07 (expired 2026-04-18, used by Stripe and Adyen) bear on the content-hash
header (R9). Panic supervision under `panic = "abort"` is Layer 1: an aborting process runs no `JoinError::Panic` arm.

```text
REALITY: premise-table finding unverified=1 (X accepting from_date equal to to_date; spec allows it, live is paid) plan:264
REALITY: already-done pass no forwarding, checkpoint, or address-in-use code on origin/dev since 2026-10-22; newest is the plan commit 03e9ac3
REALITY: surface-check pass listen port/bind/path/secret/allow-unsigned/max-events and replay from/to exist crates/xurl-cli/src/cli/mod.rs:1662
REALITY: binding-decisions pass eng D1-D8 and DX D1-D12 honored; R9 reopens DX D9's header name on RFC 6648 plan:219
REALITY: numbers pass 256-slot queues plan:208; 7 s of waits plan:216; 8 s drain at --forward-timeout 1 plan:567 [estimated from KTD4]; 9,753,712-byte binary .github/workflows/ci.yml:447 [measured: first pass, re-measure before gate]
REALITY: deferred-asks pass both TODO items planned; Account Activity replay endpoint deferred with reason TODO.md:118
REALITY: pr-count pass nine PRs stacked on dev, one per unit, landed by one gh stack merge so no restack reruns plan:27
```

Premise table:

| Premise | Claim | Receipt | Verdict |
| --- | --- | --- | --- |
| Release panics | the release binary aborts on a panic | `Cargo.toml:45` `panic = "abort"` | true |
| testing feature | `xr` enables `testing` only as a dev-dependency | `crates/xurl-cli/Cargo.toml:38`, `:72` | true |
| cargo tree | `-e features` alone hides dev edges | probe at `03e9ac3` | false |
| Hash dependency | SHA-256 is already linked in `xdk` | `crates/xdk/Cargo.toml:73` | true |
| Replay bounds | both bounds inclusive, no ordering rule | vendored `CreateWebhookReplayJobRequest`; X docs | true |
| Equal bounds | X accepts `from_date` equal to `to_date` | none (a live call is billed) | unverified |
| shell_word | reachable as `NextStep::confirm`'s | `crates/xurl-cli/src/cli/hints.rs:552` (private) | false |
| Example run | pre-push runs only `offline_search` | `scripts/hooks/pre-push:226` | true |
| Env guard | a `var_os("PATH")` read in shipped source fails the help guard | `crates/xurl-cli/tests/agentic_tests.rs:412`, `:451` | true |
| Checkpoint | the record follows receipt, not delivery | plan KTD7 | true |

### Second-pass Section 1 findings: Architecture

1. A1 [P1] (confidence: 10/10) `Cargo.toml:45` `panic = "abort"` — KTD13 and U4's supervision step watch each worker's
   join handle for an unexpected end, a panic included, then drain and exit 1 with reason `internal`. In the release
   profile a panic aborts the process before any handle reports it: exit 134 from SIGABRT, the default hook's plain-text
   message on stderr, no drain, no record. The U4 panic test runs under the test profile, which unwinds, so it passes
   while the shipped binary does something else. Decision R8.
2. A2 [P2] (confidence: 8/10) plan KTD7 ("written after an event's line prints when that event arrived in a later
   minute") with KTD3 ("prints the event line, writes the checkpoint, then puts the event on each target's bounded queue
   (capacity 256)") — the checkpoint follows receipt, not delivery. A hard stop with a backlog (a kill, an OOM kill, or
   the A1 abort) leaves up to 256 acknowledged and undelivered events per target before the checkpoint's minute, where
   `replay --from-checkpoint` never reaches them; at a graceful drain deadline, `forward-incomplete` counts them and
   names no window. Decision R7.
3. A3 [P2] (confidence: 8/10) plan U9 Approach 3 — `test_command` is a `curl` line quoted by `shell_word`
   (`crates/xurl-cli/src/cli/hints.rs:561`, `format!("'{}'", arg.replace('\'', r"'\''"))`), POSIX single quotes around a
   JSON body. `cmd.exe` has no single-quote quoting, and Windows PowerShell 5.1 maps `curl` to `Invoke-WebRequest`, so
   the printed line does not run on Windows, the platform U8's CI job adds. Decision R10.
4. A4 [P3] (confidence: 9/10) plan High-Level Technical Design — the event-flow chart shows only a `failed` edge to
   stderr and no content-hash header; the shutdown sequence ends in `exit 0` only. The `forwarded` record (KTD10), the
   hash header (KTD5), and KTD13's exit are missing. Correction applied after R8 and R9: the event-flow chart gains the
   success record, the `Idempotency-Key` and `XR_EVENT_SHA256` keys, the no-redirect worker, and the completion edges
   that move the checkpoint; the shutdown sequence gains the checkpoint note and KTD13's exit.

5. A5 [P1] (confidence: 9/10) plan U3 Approach 2 ("owns an HTTP client built like the one at
   `crates/xdk/src/api/request/mod.rs:311`") — that client is `reqwest::Client::builder().build()`, which follows up to
   10 redirects, and on a 301 or 302 tower-http turns the `POST` into a `GET` with an empty body
   (`tower-http-0.6.11/src/follow_redirect/mod.rs:274-281`). A target behind an http-to-https or trailing-slash redirect
   answers the empty `GET` with 200, and the forward counts as delivered; with R7 the checkpoint then moves past it.
   Raised by Brett's question "are we listening for an ack from the forwarded location or just broadcasting?". Decision
   R11.

### Second-pass decision ledger

### R7: What the checkpoint records while targets lag (A2)

- **Finding:** A2, P2, confidence 8/10, plan KTD7 and KTD3, plan-eng-review
- **Plan baseline:** original proposal: R8 records "when the most recent verified event arrived"; KTD7 writes it after
  the event's line prints. Eng R2 (D3) approved only the file's form, Unix seconds.
- **Runtime evidence:** unknown; the code is proposed. Layer 1 practice is a consumer committing its offset after it
  processes a record, not when the record arrives (Kafka consumer offsets).
- **Comparison grid:**

| Choice | Current | A | B |
| --- | --- | --- | --- |
| R7 what the checkpoint records | arrival of the newest verified event (KTD7) | arrival of the oldest event not yet finished (delivered, or reported `forward-failed`) at every target; the newest arrival when nothing is pending | arrival of the newest verified event |
| R7 hard stop with a backlog | queued events unrecoverable by `--from-checkpoint` | `--from-checkpoint` covers them; the faster target sees repeats | unrecoverable; the README says recovery covers only events that arrived while `listen` was down |
| R7 drain deadline reached | `forward-incomplete` count, no window | count, and the checkpoint still covers them | count, no window |
| R7 no forward target | newest arrival | newest arrival (unchanged) | newest arrival |
| R7 write rate | at most one a minute | at most one a minute | at most one a minute |
| R2 checkpoint form (approved, D3) | Unix seconds | Unix seconds | Unix seconds |
| R8 panic mechanism | pending | pending | pending |
| R9 content-hash header | pending | pending | pending |
| R10 Windows test request | pending | pending | pending |

```text
Question D1:
D1 — What should the checkpoint record while a forward target lags?
Project/branch/task: dev, listen forwarding plan, second eng pass, KTD7 with U5 and U6.
ELI10: The checkpoint marks where a later replay starts. As planned it marks the newest event listen received, even while that event and up to 255 more wait in a slow target's queue. If listen dies then (kill -9, an OOM kill, or a panic, which aborts in the release build), those events were acknowledged to X, never delivered, and sit before the checkpoint, so replay --from-checkpoint skips them.
Stakes if we pick wrong: an agent that runs the documented recovery exactly still loses up to 256 events per target, silently.
Recommendation: A because a consumer commits its position after it processes a record, not when it arrives, and that costs one completion signal per event here.
Completeness: A=10/10, B=6/10
Pros / cons:
A) Oldest undelivered (recommended)
  ✅ A crash, an OOM kill, or the drain deadline leaves --from-checkpoint covering every undelivered event
  ✅ With no forward target it records exactly what the plan records today, so stdout-only use is unchanged
  ❌ Each worker reports every finished event to the loop, and the faster target sees repeats after a replay
B) Newest received, as planned
  ✅ No completion signal between the workers and the loop; KTD7, U5, and U6 stay as written
  ❌ A hard stop with a backlog loses those events for good, and the README must call recovery partial
Net: one completion signal per event buys a recovery recipe that is complete instead of best effort.
Header: Checkpoint
Options:
A) Oldest undelivered (Recommended)
The checkpoint records the arrival minute of the oldest event not yet delivered or reported forward-failed at every target, and the newest arrival when none is pending; with no target it is unchanged. Workers report each finished event to the loop; still at most one write a minute. Tests: a kill with a backlog and a drain deadline each leave the checkpoint at the oldest undelivered minute. (human: ~3 h / CC: ~20 min)
B) Newest received, as planned
KTD7 as written: the checkpoint records the newest verified event's arrival. A hard stop with a backlog loses the queued events; the README says recovery covers only events that arrived while listen was down. (human: ~15 min / CC: ~2 min)
```

- **State:** approved
- **Actual answer:** A) Oldest undelivered (second-pass D1, 2026-10-22)
- **Accepted scope:** the checkpoint records the arrival minute of the oldest event not yet delivered or reported
  `forward-failed` at every target, and the newest arrival when none is pending; with no target it is unchanged. Each
  worker reports every finished event to the loop; still at most one write a minute. Tests: a kill with a backlog and a
  drain deadline each leave the checkpoint at the oldest undelivered minute (U6), plus a failed event releasing it, a
  stalled second target holding it, and no target following the newest event. Applied to R8, KTD3, KTD7, KTD12, the
  restart diagram, U5, U6, and U7; the field keeps its R2 name, `last_event_at`.
- **History:** none

### R8: What a panic does in the release binary (A1)

- **Finding:** A1, P1, confidence 10/10, `Cargo.toml:45`, plan-eng-review
- **Plan baseline:** approved, DX D11 ("fail loudly and exit"), built as KTD13 and U4 Approach 6: watch each worker's
  join handle, and on an unexpected end, a panic included, stop taking events, drain the other target under the KTD12
  deadline, and exit 1 with reason `internal` and the `report-issue` step. This reopens only the mechanism: the evidence
  below contradicts the assumption that a panic reaches a join handle in the shipped binary.
- **Runtime evidence:** `Cargo.toml:41-45` sets `panic = "abort"` for `[profile.release]`; `cargo test` builds with
  unwinding, so the planned U4 panic test passes while the release binary aborts with SIGABRT (exit 134) and Rust's
  plain-text panic message. `crates/xurl-cli/src/main.rs:1-9` runs a `current_thread` runtime and has no panic hook.
  `Error::Internal` exits 1 (`crates/xdk/src/error.rs:662-663`). Release size at `03e9ac3`, built with
  `cargo build --release -p xurl-rs --locked`: 9,520,336 bytes with `abort`, and 10,826,304 bytes with
  `CARGO_PROFILE_RELEASE_PANIC=unwind`, 1,305,968 more (13.7%) [measured: local x86_64 Linux builds in scratch target
  directories].
- **Comparison grid:**

| Choice | Current | A | B |
| --- | --- | --- | --- |
| R8 release panic strategy | `abort` | `abort` | `unwind` |
| R8 a panic in a `listen` worker | planned: drain, then `internal` and exit 1; shipped: exit 134, plain text, no record | `internal` record in the active format, exit 1; no drain, and the checkpoint (R7) keeps the backlog replayable | drain the other target under KTD12, then `internal` and exit 1 (KTD13 as written) |
| R8 a panic in any other command | exit 134, plain text | `internal` record, exit 1 | exit 101, plain text |
| R8 how the test proves it | test profile only, which unwinds | a program the suite builds installs the shipped hook and panics; the test reads its exit code and record | test profile, which then matches release |
| R8 release size | 9,520,336 bytes | about the same | 10,826,304 bytes (+13.7%, 1,173,696 under the ceiling) |
| R7 checkpoint (approved, D1) | oldest undelivered | oldest undelivered | oldest undelivered |
| R9 content-hash header | pending | pending | pending |
| R10 Windows test request | pending | pending | pending |
| R11 forward redirects | pending | pending | pending |

```text
Question D2:
D2 — How should xr handle a panic when the release binary aborts on panics?
Project/branch/task: dev, listen forwarding plan, second eng pass, KTD13 and U4.
ELI10: The DX review decided that a forward worker that crashes should stop listen with a structured internal error. The plan does that by watching each worker for a crash. But the release build is set to abort on any panic (Cargo.toml:45), so the process dies before anything can watch it: exit 134, a plain-text message, no record. Tests run with unwinding, so the planned test passes while the shipped binary does something else.
Stakes if we pick wrong: agents get an unparseable exit 134 instead of the internal record DX D11 promised, and a green test hides it.
Recommendation: A because one panic hook gives every xr command the structured internal error at release-identical behavior, and the checkpoint from D1 already keeps a crashed backlog replayable.
Completeness: A=9/10, B=8/10
Pros / cons:
A) Panic hook, keep abort (recommended)
  ✅ Every xr command, not only listen, answers a panic with the internal record and exit 1
  ✅ The test runs the shipped hook in a real process, so green means the release does the same
  ❌ The other target's queue is not drained before exit; those events need a replay, which may be billed
B) Unwind in release, as KTD13 plans
  ✅ A crashed worker's sibling target still drains its queue before listen exits
  ❌ Other commands' panics become exit 101 plain text, and the binary grows by 1.3 MB (13.7%)
Net: one hook for structured crashes everywhere against a drain that the checkpoint already makes unnecessary.
Header: Release panics
Options:
A) Panic hook, keep abort (Recommended)
The release profile keeps panic = "abort". xr installs one panic hook once its output format is known: it writes the internal error (reason internal, exit_code 1, the report-issue step, the panic message and location, and a backtrace field when RUST_BACKTRACE asks) in the active format to stderr and exits 1 before the abort. KTD13's join-handle supervision and drain-on-panic are dropped. Test: a suite-built program installs the shipped hook and panics; exit 1 and one internal record. (human: ~2 h / CC: ~15 min)
B) Unwind in release, as KTD13 plans
The release profile sets panic = "unwind". KTD13 stays as written: a panicking worker makes listen stop taking events, drain the other target under the KTD12 deadline, and exit 1 with reason internal. Panics elsewhere in xr exit 101 with Rust's message. The binary grows by 1.3 MB (13.7%) against the 12,000,000-byte ceiling. (human: ~2 h / CC: ~15 min)
```

- **State:** approved
- **Actual answer:** A) Panic hook, keep abort (second-pass D2, 2026-10-22)
- **Accepted scope:** the release profile keeps `panic = "abort"`; `xr` installs one panic hook once its output format
  is known, which writes the `internal` error (reason `internal`, `exit_code` 1, the `report-issue` step, the panic
  message and location, and a `backtrace` field when `RUST_BACKTRACE` asks) in the active format to stderr and exits 1
  before the abort; KTD13's join-handle supervision and drain-on-panic are dropped. Test: a suite-built program installs
  the shipped hook and panics, exiting 1 with one `internal` record. Applied to KTD13, R5, U4 (Files, Approach 6, the
  panic scenario), U7, and System-Wide Impact; the hook also flushes stdout before it exits.
- **History:** DX D11 (approved): A) fail loudly and exit; reopened here on the release profile's `panic = "abort"`.

### R11: What a forward does with a redirect (A5)

- **Finding:** A5, P1, confidence 9/10, plan U3 Approach 2 ("owns an HTTP client built like the one at
  `crates/xdk/src/api/request/mod.rs:311`"), raised by Brett's question on 2026-10-22 ("are we listening for an ack from
  the forwarded location or just broadcasting?"), plan-eng-review
- **Plan baseline:** original proposal: KTD4 retries a URL attempt on a connection failure, a timeout, 408, 429, or 5xx,
  and not on another 4xx; it names no success set and no redirect rule. U3 copies the API client's construction.
- **Runtime evidence:** `crates/xdk/src/api/request/mod.rs:311` builds `reqwest::Client::builder().build()`, whose
  default policy is `Policy::limited(10)` (`reqwest-0.13.5/src/redirect.rs:161-163`). reqwest follows redirects through
  tower-http (`async_impl/client.rs:1035`), which on a 301 or 302 to a `POST` sets the method to `GET`, empties the
  body, and drops the payload headers (`tower-http-0.6.11/src/follow_redirect/mod.rs:274-281`), and on a 303 switches to
  `GET` (`:283-287`). Stripe's webhook docs: "We consider redirect responses to webhook requests as failures"
  (`docs.stripe.com/events/manage-webhook-endpoints`); PostHog fails webhook steps on any 3xx (PostHog/posthog#91378).
- **Comparison grid:**

| Choice | Current | A | B | C |
| --- | --- | --- | --- | --- |
| R11 forward client's redirect policy | reqwest default, up to 10 | none | 307 and 308 only | reqwest default, up to 10 |
| R11 what counts as delivered | unstated (U3 tests a 200) | a 2xx answer from the forward URL itself | a 2xx after at most the 307 or 308 hops | any 2xx after redirects |
| R11 a 301, 302, or 303 answer | followed as a `GET` with no body; a 200 there counts as delivered | final failure, not retried; the record names the status, never the `Location` | final failure, not retried | followed as a `GET` with no body |
| R11 a 307 or 308 answer | followed with body, signature, and `-H` headers, to any host | final failure, not retried | followed with the body, signature, and `-H` headers, to any host | followed, to any host |
| R7 checkpoint (approved, D1) | oldest undelivered | oldest undelivered | oldest undelivered | oldest undelivered |
| R8 release panics (approved, D2) | hook, keep abort | hook, keep abort | hook, keep abort | hook, keep abort |
| R9 content-hash header | pending | pending | pending | pending |
| R10 Windows test request | pending | pending | pending | pending |

```text
Question D3:
D3 — What should a forward do when the target answers with a redirect?
Project/branch/task: dev, listen forwarding plan, second eng pass, KTD4 and U3.
ELI10: listen waits for each forward's answer and counts a 2xx as delivered. The plan builds its HTTP client like xr's API client, which follows up to 10 redirects. On a 301 or 302 the library turns the POST into a GET and throws the body away, so an app behind an http-to-https or trailing-slash redirect gets an empty GET, answers 200, and xr records the event as delivered. With D1, the checkpoint then moves past it too.
Stakes if we pick wrong: an event is lost while every record says it was delivered, the one failure nothing can recover.
Recommendation: A because a webhook sender that follows redirects can mistake a login page or an empty GET for delivery; Stripe counts every redirect answer as a failure, and the fix is one client setting.
Completeness: A=10/10, B=7/10, C=3/10
Pros / cons:
A) No redirects, 3xx fails (recommended)
  ✅ Delivered means the forward URL itself answered 2xx; nothing is ever counted delivered by a body-less GET
  ✅ Event bytes, signature, and -H headers never travel to a host the caller did not name
  ❌ A target behind a redirect fails until the caller points --forward at the final URL
B) Follow 307 and 308 only
  ✅ A trailing-slash or https redirect that keeps the method and body still delivers
  ❌ A 307 to another host carries the event, its signature, and every -H value there
C) Follow every redirect, as planned
  ✅ No extra client setting; any URL that eventually answers 2xx counts
  ❌ A 301 or 302 turns the forward into an empty GET that can be counted delivered
Net: one client setting against delivery records that can be wrong without anyone noticing.
Header: Redirects
Options:
A) No redirects, 3xx fails (Recommended)
The forward client is built with redirects off, and only a 2xx from the forward URL counts as delivered. A 3xx is a final failure, not retried, and its forward-failed record names the status, never the Location header. KTD4 names the success set. Tests: a 302, a 307, and a 308 each give one attempt and a forward-failed naming the status while the redirect target receives nothing; a 204 counts as delivered. (human: ~30 min / CC: ~5 min)
B) Follow 307 and 308 only
The client follows only 307 and 308, which keep the method, body, and headers, up to the default 10 hops to any host; a 301, 302, or 303 is a final failure. Tests: a 307 to a second mock delivers the body there; a 302 fails. (human: ~1 h / CC: ~10 min)
C) Follow every redirect, as planned
The client keeps reqwest's default policy; a 301 or 302 becomes a GET with no body, and its 2xx counts as delivered. (human: none / CC: none)
```

- **State:** approved
- **Actual answer:** A) No redirects, 3xx fails (second-pass D3, 2026-10-22)
- **Accepted scope:** the forward client is built with redirects off, and only a 2xx from the forward URL counts as
  delivered; a 3xx is a final failure, not retried, whose `forward-failed` record names the status, never the `Location`
  header; KTD4 names the success set. Tests: a 302, a 307, and a 308 each give one attempt and a final failure naming
  the status while the redirect target receives nothing; a 204 counts as delivered. Applied to KTD4, U3 (Approach 2 and
  two scenarios), and U7's README list.
- **History:** none

### R10: How a Windows caller runs the signed test request (A3)

- **Finding:** A3, P2, confidence 8/10, plan U9 Approach 3 and `crates/xurl-cli/src/cli/hints.rs:561`, plan-eng-review
- **Plan baseline:** approved, DX D5: `listen`'s startup line hands the agent a signed test command, built in U9 as a
  `curl` line on the `listening` record (`test_command`) and in the text notice.
- **Runtime evidence:** every `command` and `template` xr prints is quoted by `shell_word`, POSIX single quotes on every
  platform (`hints.rs:561`, `format!("'{}'", arg.replace('\'', r"'\''"))`). The test body is JSON, so the line always
  carries quoted text. `cmd.exe` treats single quotes as literal characters, and Windows PowerShell 5.1 aliases `curl`
  to `Invoke-WebRequest`, whose flags differ; neither runs the line as printed [in-distribution knowledge; no Windows
  probe run]. U8 adds a `windows-latest` CI job, so the plan treats Windows as a target.
- **Comparison grid:**

| Choice | Current | A | B | C |
| --- | --- | --- | --- | --- |
| R10 `test_command` | POSIX `curl` line | POSIX `curl` line, unchanged | POSIX line on Unix; `curl.exe` with `cmd.exe` double-quote escaping on Windows | POSIX `curl` line, unchanged |
| R10 structured request on the `listening` record | none | `test_request`: `method`, `url`, `headers`, `body` | none | none |
| R10 what a Windows agent runs | nothing that works | sends `test_request` with any HTTP client, no shell | the `cmd.exe` line (PowerShell still differs) | nothing; the README says a POSIX shell is needed |
| R10 how tests prove it | the string run through `sh`, which the Windows job lacks | `test_request` posted with `reqwest` on every OS; the string run through `sh` under `cfg(unix)` | each OS runs its own string | `cfg(unix)` only |
| R7 checkpoint (approved, D1) | oldest undelivered | oldest undelivered | oldest undelivered | oldest undelivered |
| R8 release panics (approved, D2) | hook, keep abort | hook, keep abort | hook, keep abort | hook, keep abort |
| R11 redirects (approved, D3) | none, 3xx fails | none, 3xx fails | none, 3xx fails | none, 3xx fails |
| R9 content-hash header | pending | pending | pending | pending |

```text
Question D4:
D4 — How should a Windows caller run listen's signed test request?
Project/branch/task: dev, listen forwarding plan, second eng pass, U9.
ELI10: The DX review added a ready-to-run curl line to listen's startup record so an agent can prove its wiring. xr quotes every command it prints with Unix shell single quotes, and this one always carries a JSON body. On Windows, cmd.exe does not understand single quotes, and PowerShell 5.1 treats curl as a different tool, so the line fails there. U8 adds a Windows CI job, so Windows is a platform the plan supports.
Stakes if we pick wrong: the first-success step DX D5 designed fails for every Windows agent, and its test cannot run on the Windows job.
Recommendation: A because a structured request is the agent-facing form xr already favors (text for humans, JSON for agents), runs on any OS without a shell, and leaves the curl line as it is.
Completeness: A=10/10, B=6/10, C=4/10
Pros / cons:
A) Add a structured test_request (recommended)
  ✅ Any agent on any OS sends method, url, headers, and body with its own HTTP client, no quoting at all
  ✅ The test posts it on the Windows job too, so the cross-platform claim is proven, not assumed
  ❌ Two forms of one request on the listening record, which the README has to explain
B) Render per platform
  ✅ A Windows human gets a line to paste into cmd.exe without reading JSON
  ❌ cmd.exe escaping of a JSON body is fragile, and PowerShell still needs its own form
C) POSIX only, as planned
  ✅ No extra field or rendering code; matches every other command string xr prints
  ❌ Windows agents get a first-success step that fails, and the README has to say so
Net: one additive key buys a first success that works everywhere.
Header: Windows test
Options:
A) Add a structured test_request (Recommended)
Under a structured format, the listening record adds test_request: {method, url, headers, body} beside test_command, which stays the POSIX curl line, as every xr command string is. Tests: posting test_request with reqwest yields a verified event on every OS, the Windows job included; running test_command through sh is cfg(unix). README names both. (human: ~30 min / CC: ~5 min)
B) Render per platform
test_command is the POSIX curl line on Unix and a curl.exe line with cmd.exe double-quote escaping on Windows. PowerShell gets no working form. Tests: each OS runs its own line. (human: ~2 h / CC: ~15 min)
C) POSIX only, as planned
test_command stays the POSIX curl line on every OS; its test runs under cfg(unix) only, and the README says the command needs a POSIX shell. (human: ~10 min / CC: ~2 min)
```

- **State:** approved
- **Actual answer:** A) Add a structured test_request (second-pass D4, 2026-10-22)
- **Accepted scope:** under a structured format the `listening` record adds `test_request` (`method`, `url`, `headers`,
  `body`) beside `test_command`, which stays the POSIX `curl` line; tests: posting `test_request` with `reqwest` yields
  a verified event on every OS, the Windows job included, and running `test_command` through `sh` is `cfg(unix)`; the
  README names both. Applied to U9 (Approach 3, scenarios, Verification), U8 Approach 6 (the Windows job runs the
  `test_request` test), and U7.
- **History:** none

### Second-pass Section 2 findings: Code quality

1. C1 [P2] (confidence: 8/10) plan KTD5 ("an `X-Xr-Event-Sha256` header holding the SHA-256 of the event's exact bytes")
   — RFC 6648 §3: creators of new parameters "SHOULD NOT prefix their parameter names with "X-" or similar constructs".
   Two existing names fit the job: `Idempotency-Key` (draft-ietf-httpapi-idempotency-key-header-07, expired 2026-04-18,
   used by Stripe and Adyen and keyed on by idempotency middleware) and `Content-Digest` (RFC 9530). The value's
   encoding is unstated. Decision R9.
2. C2 [P2] (confidence: 9/10) plan KTD6 ("searching `PATH` itself (and `PATHEXT` on Windows)") — a `var_os("PATH")` or
   `var_os("PATHEXT")` read in shipped source matches the guard's pattern at
   `crates/xurl-cli/tests/agentic_tests.rs:412` (`env::var(?:_os)?\("([A-Z_]+)"\)`), and only `HOME` is exempt (`:451`),
   so `test_help_advertises_every_env_var_the_source_reads` fails; `AGENTS.md` rule 2 also puts every environment read
   in `crates/xurl-cli/src/cli/env.rs`. Correction applied: U8 reads both through `env.rs`, exempts them beside `HOME`
   with the reason (OS search paths, not `xr` settings), and adds them to rule 2's list.
3. C3 [P3] (confidence: 9/10) plan Key Technical Decisions — KTD13 and KTD14 sit between KTD11 and KTD12. Correction
   applied: KTD12 moves back after KTD11.
4. C4 [P3] (confidence: 6/10) plan KTD6 — `XR_EVENT_SHA256` breaks the `XURL_` prefix of xr's variables. Kept: `XURL_`
   names the variables `xr` reads, each advertised in `--help` by the guard above; this one `xr` sets for its child, and
   `XR_` matches the binary's name. No change.

### R9: The content-hash header's name and value (C1)

- **Finding:** C1, P2, confidence 8/10, plan KTD5, plan-eng-review
- **Plan baseline:** approved, DX D9: "a content-hash key on every forward", written into KTD5 as `X-Xr-Event-Sha256`,
  with `event_sha256` on `forward-failed` and `XR_EVENT_SHA256` for the program; the encoding is unstated. This reopens
  the header's name and value only: RFC 6648 contradicts the name, and the record and environment need one stated
  encoding.
- **Runtime evidence:** `xr` sends no header of its own today; the `X-` names in the code are X's (`x-rate-limit-*` at
  `crates/xdk/src/api/request/mod.rs:204-206`, `x-twitter-webhooks-signature` at
  `crates/xdk/src/webhooks/signing.rs:16`). Go `xurl` does not forward (`KNOWN_DIFFERENCES.md`, U7). The idempotency
  draft makes the value a Structured Field String, quoted; Stripe sends its keys unquoted. RFC 9530 makes
  `Content-Digest` a dictionary whose `sha-256` member is a base64 byte sequence between colons.
- **Comparison grid:**

| Choice | Current | A | B | C |
| --- | --- | --- | --- | --- |
| R9 header name | `X-Xr-Event-Sha256` (DX D9) | `Idempotency-Key` | `Content-Digest` | `X-Xr-Event-Sha256` |
| R9 header value | SHA-256, encoding unstated | the body's SHA-256 as quoted lowercase hex, `"9f86d0…"` | `sha-256=:<base64 of the SHA-256>:` | the body's SHA-256 as lowercase hex |
| R9 standard behind the name | none; RFC 6648 §3 advises against `X-` | IETF httpapi draft-07 (expired), common practice through Stripe and Adyen | RFC 9530 | none |
| R9 `-H` may set that header | no | no | no | no |
| R9 `event_sha256` and `XR_EVENT_SHA256` | SHA-256, encoding unstated | lowercase hex, 64 characters (new, shared) | lowercase hex, 64 characters (new, shared) | lowercase hex, 64 characters (new, shared) |
| R7 checkpoint (approved, D1) | oldest undelivered | oldest undelivered | oldest undelivered | oldest undelivered |
| R8 release panics (approved, D2) | hook, keep abort | hook, keep abort | hook, keep abort | hook, keep abort |
| R10 Windows test (approved, D4) | `test_request` added | `test_request` added | `test_request` added | `test_request` added |
| R11 redirects (approved, D3) | none, 3xx fails | none, 3xx fails | none, 3xx fails | none, 3xx fails |

```text
Question D5:
D5 — What header should carry the forward's content-hash key?
Project/branch/task: dev, listen forwarding plan, second eng pass, KTD5 and U3.
ELI10: The DX review added a hash of each event's bytes to every forward so the app can drop repeats after a retry or a replay. The plan names the header X-Xr-Event-Sha256, but the HTTP standards body asks new headers not to start with X- (RFC 6648). Two existing headers already do this job: Idempotency-Key, which many web frameworks' middleware already drops repeats on, and Content-Digest, the standard header for a body's hash. In every option the record and the program's variable carry the same hash as 64 lowercase hex characters.
Stakes if we pick wrong: apps write their dedupe code against a name xr would have to keep, or rename, forever.
Recommendation: A because Idempotency-Key names the job the key does, existing middleware dedupes on it with no app code, and it avoids the X- prefix RFC 6648 advises against; its draft has expired, so this rests on common practice, not an RFC.
Note: options differ in kind, not coverage — no completeness score.
Pros / cons:
A) Idempotency-Key, quoted hex (recommended)
  ✅ Idempotency middleware in common frameworks drops a repeat with no app code, the job DX D9 wanted
  ✅ The value is the same 64 hex characters the record and XR_EVENT_SHA256 carry, quoted per the draft
  ❌ The draft expired in April 2026 without becoming an RFC, so details such as quoting could still move
B) Content-Digest, RFC 9530
  ✅ A published RFC, and integrity middleware can check the body against it
  ❌ It names integrity, not dedupe, and its base64 value differs from the hex in the record and environment
C) Keep X-Xr-Event-Sha256
  ✅ Matches the DX review's wording, and the record, header, and variable share one obvious name
  ❌ RFC 6648 advises against new X- names, and no middleware knows this header
Net: a header that existing tools already dedupe on, resting on common practice, against a strict RFC or xr's own name.
Header: Hash header
Options:
A) Idempotency-Key, quoted hex (Recommended)
Each URL forward sends Idempotency-Key: "<SHA-256 of the body as 64 lowercase hex characters>", quoted as the IETF draft's Structured Field String. -H may not set Idempotency-Key. event_sha256 and XR_EVENT_SHA256 carry the same hex, unquoted. Tests: the header equals the body's hash, quoted, on the first attempt and a retry; -H 'Idempotency-Key: x' is refused. (human: ~30 min / CC: ~5 min)
B) Content-Digest, RFC 9530
Each URL forward sends Content-Digest: sha-256=:<base64 of the SHA-256>:. -H may not set Content-Digest. event_sha256 and XR_EVENT_SHA256 carry the hash as lowercase hex. Tests: the header parses as RFC 9530 and matches the body on a retry; -H 'Content-Digest: x' is refused. (human: ~30 min / CC: ~5 min)
C) Keep X-Xr-Event-Sha256
Each URL forward sends X-Xr-Event-Sha256: <SHA-256 as lowercase hex>, as DX D9 wrote it. -H may not set it. event_sha256 and XR_EVENT_SHA256 carry the same hex. Tests as planned in U3. (human: ~10 min / CC: ~2 min)
```

- **State:** approved
- **Actual answer:** A) Idempotency-Key, quoted hex (second-pass D5, 2026-10-22)
- **Accepted scope:** each URL forward sends `Idempotency-Key: "<SHA-256 of the body as 64 lowercase hex characters>"`,
  quoted as the IETF draft's Structured Field String; `-H` may not set `Idempotency-Key`; `event_sha256` and
  `XR_EVENT_SHA256` carry the same hex, unquoted. Tests: the header equals the body's hash, quoted, on the first attempt
  and a retry; `-H 'Idempotency-Key: x'` is refused. Applied to KTD5, KTD6, KTD10, U3 (Approach 2 and two scenarios),
  U7, the DX confusion report, and DX task T4.
- **History:** DX D9 (approved): A) a content-hash key on every forward, named `X-Xr-Event-Sha256` in KTD5; reopened
  here on RFC 6648.

### Second-pass Section 3 findings: Tests

Test framework: `cargo test`, per `AGENTS.md` § Testing; integration tests drive `xr` through `common::xr()` or in
process through `run_with_store_path`.

1. T1 [P1] (confidence: 9/10) `crates/xurl-cli/tests/binary_contract_tests.rs:164` (`!stderr.contains("panicked at")`) —
   CRITICAL regression: `sigpipe_smoke` catches drift in `main`'s SIGPIPE restoration only by Rust's panic wording on
   stderr. Under R8 the hook replaces that output with the `internal` record and exit 1, so the guard can go quiet.
   Decision R12.
2. T2 [P2] (confidence: 9/10) plan U4 Approach 6 ("once `run_argv` has resolved the output format") — `run_argv`
   (`crates/xurl-cli/src/cli/runner.rs:63-71`) resolves nothing; `run` and `run_with_overrides` resolve the format, and
   in-process tests call `run_with_store_path` (`crates/xurl-cli/tests/cli_run_tests.rs`). A hook installed where the
   format resolves would turn any later assertion failure in such a test binary into `exit(1)`, killing the whole
   binary. Correction applied: `run_argv` alone installs the hook, before it calls `run`; the runner publishes the
   resolved format to a process-wide cell the hook reads; a new in-process test shows a panic after
   `run_with_store_path` still unwinds to `catch_unwind`.
3. T3 [P2] (confidence: 9/10) plan U8 test scenarios — the success scenario runs `-- sh -c 'cat > "$1"' sh <file>`, and
   the group-kill scenario "backgrounds a long `sleep`", while U8 Approach 5 builds Rust test programs "so every
   program-target test runs on Linux, macOS, and Windows alike". Correction applied: both scenarios name the suite's
   echo-to-file and child-starting programs.
4. T4 [P3] (confidence: 9/10) plan U1, U3, and U9 doc examples ("compiled as a doctest") — `cargo test --doc` runs a
   doctest unless it is marked `no_run`, and the U1 example loops on `next_event()` while the U3 examples send requests.
   Correction applied: examples that bind a listener or send a request are `no_run`; the U5 checkpoint examples and the
   U9 builder example run.

### R12: How `sigpipe_smoke` keeps catching a SIGPIPE drift (T1)

- **Finding:** T1, P1, confidence 9/10, `crates/xurl-cli/tests/binary_contract_tests.rs:164`, plan-eng-review
- **Plan baseline:** none: the approved hook (R8, D2) does not name this guard.
- **Runtime evidence:** `sigpipe_smoke` (`binary_contract_tests.rs:131-170`, `#[cfg(unix)]`) runs `xr --help`, closes
  the pipe, and asserts stderr holds neither `panicked at` nor `BrokenPipe`; it does not read the exit status. Its doc
  comment names the drift it guards against: a Rust panic, "signal-killed via SIGABRT". After R8 the same drift exits 1
  and prints the `internal` record.
- **Comparison grid:**

| Choice | Current | A | B |
| --- | --- | --- | --- |
| R12 what `sigpipe_smoke` asserts | stderr lacks `panicked at` and `BrokenPipe` | those two, plus the child exited 0 or died by signal 13 | those two only |
| R12 the hook's wording | no hook | text and JSON `message` keep Rust's `panicked at <file>:<line>:<col>` | text and JSON `message` must keep `panicked at` |
| R12 a SIGPIPE drift after R8 | not applicable | fails on the exit status and on the text | fails on the text only |
| R8 release panics (approved, D2) | hook, keep abort | hook, keep abort | hook, keep abort |
| R7, R9, R10, R11 (approved) | as recorded | as recorded | as recorded |

```text
Question D6:
D6 — How should the SIGPIPE guard keep catching its regression once panics answer with a record?
Project/branch/task: dev, listen forwarding plan, second eng pass, the D2 panic hook and binary_contract_tests.rs.
ELI10: A test makes sure that piping xr --help into a reader that closes early ends quietly. It notices a regression only by spotting Rust's "panicked at" text on stderr. The D2 hook replaces that text with xr's own internal record and exit 1, so the test could keep passing while the bug it guards comes back.
Stakes if we pick wrong: a pipe like xr ... | head breaks for every agent and the suite stays green.
Recommendation: A because the exit status is the invariant the guard means (exit 0 or death by SIGPIPE), it holds whatever the hook prints, and the text checks stay as a second net.
Completeness: A=10/10, B=6/10
Pros / cons:
A) Assert the exit status too (recommended)
  ✅ The guard fails on exit 1, SIGABRT, or anything but exit 0 or signal 13, whatever stderr says
  ✅ The hook keeps Rust's "panicked at" wording, so the old text checks still fire as well
  ❌ One more assertion and an ExitStatusExt import in a Unix-only test
B) Keep the text check only
  ✅ No change to the test; the hook's wording carries the whole contract
  ❌ A later edit to the hook's message silences the guard with nothing failing
Net: one status assertion makes the guard independent of what the hook prints.
Header: SIGPIPE guard
Options:
A) Assert the exit status too (Recommended)
Preserve: a SIGPIPE drift fails sigpipe_smoke. Intentional change: such a drift now exits 1 with the internal record instead of SIGABRT. sigpipe_smoke also asserts the child exited 0 or was killed by signal 13 (ExitStatusExt::signal), keeps its two text checks, and the hook's text and JSON message keep Rust's "panicked at <file>:<line>:<col>". (human: ~15 min / CC: ~3 min)
B) Keep the text check only
Preserve: a drift fails sigpipe_smoke only while the hook's text and JSON message contain "panicked at"; the test is unchanged. (human: ~5 min / CC: ~1 min)
```

- **State:** approved
- **Actual answer:** A) Assert the exit status too (second-pass D6, 2026-10-22)
- **Accepted scope:** preserve: a SIGPIPE drift fails `sigpipe_smoke`; intentional change: such a drift exits 1 with the
  `internal` record instead of SIGABRT. `sigpipe_smoke` also asserts the child exited 0 or was killed by signal 13
  (`ExitStatusExt::signal`), keeps its two text checks, and the hook's text and JSON `message` keep Rust's
  `panicked at <file>:<line>:<col>`. Applied to U4 (Files and a CRITICAL regression scenario).
- **History:** none

### Second-pass coverage diagram

Every path below is proposed; `PLANNED` names the test the plan now requires, not code that exists.

```text
CODE PATHS (second pass)                                        USER FLOWS
[+] xdk::webhooks sample builder and testing sender (U9)        [+] Agent proves its wiring before X (U9, R10)
  ├── [★★★ PLANNED] OAuth2 and legacy signing both verify          ├── [★★★ PLANNED] test_request posted, every OS
  └── [★★  PLANNED] testing sender → Receiver yields the event     ├── [★★  PLANNED] test_command through sh, cfg(unix)
[+] xdk forward URL target (U3)                                  └── [★★★ PLANNED] neither form holds a secret
  ├── [★★★ PLANNED] Idempotency-Key quoted hex, same on a retry  [+] Target drops repeats (R9)
  ├── [★★★ PLANNED] -H Idempotency-Key refused                     └── [★★  PLANNED] retry and replay share the key
  ├── [★★★ PLANNED] 302, 307, 308: one attempt, final failure    [+] Target behind a redirect (R11)
  └── [★★  PLANNED] 204 counts as delivered                        └── [★★★ PLANNED] record names status, no Location
[+] xr workers and records (U4, U6)                              [+] Crash with a backlog (R7)
  ├── [★★  PLANNED] forwarded record per delivery                  ├── [★★★ PLANNED] kill → checkpoint at oldest undelivered
  ├── [★★★ PLANNED] forward-failed: hash, side_effects, template   ├── [★★★ PLANNED] drain deadline → same, count 2
  └── [★★★ PLANNED] completion report moves the checkpoint         ├── [★★  PLANNED] failed event releases it
[+] xr panic hook (U4, KTD13)                                      └── [★★  PLANNED] stalled second target holds it
  ├── [★★★ PLANNED] suite program: exit 1, one internal record   [+] A panic in any command (R8)
  ├── [★★★ PLANNED] in process: no hook, panic still unwinds (T2)  ├── [★★★ PLANNED] text names the issue tracker
  └── [★★★ PLANNED] CRITICAL sigpipe_smoke on exit status (R12)    └── [★★★ PLANNED] stdout lines before it survive
[+] program target PATH lookup (U8, C2)
  └── [★★  EXISTING] agentic_tests.rs env guard, PATH and PATHEXT exempt with the reason
[+] doc examples (T4): bind or send → no_run; checkpoint, budget, builder, sender run

COVERAGE (second-pass paths): 24/24 with a planned or existing test | GAPS: 0 after the corrections
QUALITY: ★★★ 16, ★★ 8, ★ 0
```

Test Plan artifact: `~/.gstack/projects/brettdavies-xurl-rs/brett-dev-eng-review-test-plan-20261022-190522.md`.

### Second-pass Section 4: Performance

No issues found. Each event adds one SHA-256 of a body of a few KB (`sha2`, microseconds [estimated]), one completion
message per target, and one `forwarded` line per target on stderr; the pending arrival times per target are bounded by
the 256-slot queue; the panic hook costs nothing until a panic. Scale of X's delivery rate: unknown.

### Second-pass Outside voice

codex, plan-review phase: disabled by config (`codex_reviews`); no outside process and no native replacement ran.
Recorded as a skipped `codex-plan-review` entry.

### R13: A one-minute replay window, unverified with X (TODO)

- **Finding:** premise row "Equal bounds", unverified, plan KTD10 (`next_step` template with both minutes set to the
  event's arrival minute), plan-eng-review
- **Plan baseline:** approved, DX D10: the `forward-failed` record carries a replay template for the event's arrival
  minute.
- **Runtime evidence:** the vendored `CreateWebhookReplayJobRequest` and X's docs make both bounds inclusive and state
  no ordering or minimum window; whether X accepts `from_date` equal to `to_date` is unverified, since a live call is
  billed (R15 and the pay-per-use rule).
- **Comparison grid:**

| Choice | Current | A | B | C |
| --- | --- | --- | --- | --- |
| R13 TODO for the equal-bounds check | none | a `TODO.md` entry: confirm on the first real replay; if X refuses, widen the template's `--to` by one minute | none | none, and the template's `--to` is the arrival minute plus one now |
| R13 template window | one minute (from equal to to) | one minute | one minute | two minutes, possibly billing one more minute of events |
| R7 to R12 (approved) | as recorded | as recorded | as recorded | as recorded |

```text
Question D7:
D7 — Track the unverified one-minute replay window as a TODO?
Project/branch/task: dev, listen forwarding plan, second eng pass, KTD10's replay template.
ELI10: When a forward fails, its record hands the agent a replay command for the minute the event arrived, so from and to are the same minute. X's docs say both ends are inclusive and set no minimum, so this should work, but nobody has checked it with X, because a replay is a paid call.
Stakes if we pick wrong: if X refuses an equal from and to, every forward-failed record hands the agent a command that fails.
Recommendation: A because the first real recovery checks it for free, and the fallback is a one-line change to the template.
Note: options differ in kind, not coverage — no completeness score.
Pros / cons:
A) Add to TODO.md (recommended)
  ✅ The first paid replay doubles as the check, with no extra spend
  ✅ The fallback, one more minute on --to, is written down before anyone needs it
  ❌ Until that first replay, the template rests on X's docs alone
B) Skip
  ✅ No entry; the docs say inclusive bounds and set no minimum
  ❌ If X refuses, the first agent to recover finds out through an invalid-request error
C) Widen the window now
  ✅ Works whether or not X accepts equal bounds
  ❌ Every recovery replays one more minute of events, which X may bill
Net: a free check on first use against paying for an extra minute on every recovery.
Header: Replay TODO
Options:
A) Add to TODO.md (Recommended)
A TODO.md entry: on the first real replay from a forward-failed template, confirm X accepts from_date equal to to_date; if it refuses, widen the template's --to by one minute in KTD10 and its test. Context, depends on: the forwarding release. (human: ~5 min / CC: ~1 min)
B) Skip
No entry; the template keeps the one-minute window on the strength of X's docs. (human: none / CC: none)
C) Widen the window now
The template's --to becomes the arrival minute plus one; U4's template test changes to match. (human: ~10 min / CC: ~2 min)
```

- **State:** approved
- **Actual answer:** C) Widen the window now (second-pass D7, 2026-10-22)
- **Accepted scope:** the template's `--to` becomes the arrival minute plus one; U4's template test changes to match. No
  `TODO.md` entry. Applied to KTD10, the U4 template scenario (with hour and day boundaries), DX task T3, and Risks,
  which notes that a template run within a minute of the event names a `--to` minute that has not ended.
- **History:** none

Approval readiness: PASS. Checked: R7 (second-pass D1, A), R8 (D2, A), R11 (D3, A), R10 (D4, A), R9 (D5, A), R12 (D6,
A), R13 (D7, C). S1 to S3, A4, C2, C3, and T2 to T4 are corrections of fact or work an approved contract requires (KTD6,
KTD13, DX D5, DX D8), applied without a question; C4 changed nothing.

### Second-pass NOT in scope

- Restarting a worker after a panic: rejected by DX D11, since a bug that panics on every event would loop.
- Following 307 and 308 redirects (R11 B): a cross-host 307 would carry the event, its signature, and every `-H` value
  to a host the caller did not name.
- Sending `Content-Digest` beside `Idempotency-Key` (R9 B): no target asked for an integrity header, and the signature
  already proves the bytes.
- A per-platform `test_command` (R10 B): `test_request` serves every OS without a second quoting scheme.
- An `xr webhooks test` verb that sends the sample itself: a new family member with every surface the recipe names;
  `test_request` gives the same first success.
- A live check that X accepts the template's window, including an unfinished `--to` minute: every replay is billed, so
  it stays a Risk.
- Renaming `last_event_at`: the name is approved in R2, and KTD7 now states what it holds.

### Second-pass What already exists

- `sha2 = "0.11"` (`crates/xdk/Cargo.toml:73`) computes the key; hex is a format loop, so no new dependency.
- `reqwest::redirect::Policy::none()` turns redirects off on the forward client; the API client at
  `crates/xdk/src/api/request/mod.rs:311` keeps its default.
- `std::panic::set_hook` and `std::process::exit` are the whole hook; no crate.
- `shell_word` (`crates/xurl-cli/src/cli/hints.rs:552`) quotes `test_command`, as it quotes every `command` and
  `template`.
- `sigpipe_smoke` (`crates/xurl-cli/tests/binary_contract_tests.rs:131`) is extended, not replaced.
- The help-advertising guard (`crates/xurl-cli/tests/agentic_tests.rs:408`) and its `HOME` exemption take `PATH` and
  `PATHEXT` beside it.
- `write_atomically` (`crates/xdk/src/store/atomic.rs`) still writes the checkpoint; only the time handed to it changes.

### Second-pass diagrams

The event-flow chart, the shutdown sequence, and the restart sequence in the High-Level Technical Design carry the
second-pass changes (A4). The checkpoint's pending-arrival state:

```text
event arrives   ──► push its arrival onto every target's pending FIFO (each at most 256)
target finishes ──► pop that target's head (delivered, or reported forward-failed)
checkpoint time  =  oldest head across non-empty FIFOs, else the newest arrival
write            ──► when that time's minute differs from the minute last written
```

No inline code diagram is needed; the tracking sits in `crates/xurl-cli/src/cli/commands/webhooks_forward.rs` beside the
workers.

### Second-pass failure modes

| Path | Realistic failure | Test or handling | What the caller sees |
| --- | --- | --- | --- |
| `test_request` | the sample is signed for the other secret mode | U9 scenarios on the OAuth2 and legacy paths | a verified event, or a 401 from the listener |
| `Idempotency-Key` | a caller `-H` overwrites the key | U3 refusal test | `validation` before binding |
| Forward redirect | the target answers 302 to `https` | U3 redirect tests | `forward-failed` naming 302 |
| Checkpoint | `listen` is killed with a backlog | U6 kill test | `--from-checkpoint` covers the backlog |
| Checkpoint | one event fails forever and would pin the window | U6 test: a failed event releases it | `forward-failed` with its template |
| Panic hook | a panic before the output format resolves | text path in the hook | the `internal` error in text, exit 1 |
| Panic hook | an in-process test binary dies on its first assertion | T2 in-process test | not reachable: the hook stays on `run_argv` |
| SIGPIPE | restoration drifts after the hook | `sigpipe_smoke` exit-status check (R12) | the suite fails |
| Replay template | X refuses a `--to` minute that has not ended | none: a paid call (Risks) | `invalid-request` from X, visible, not silent |

Critical gaps: 0. The one untested path fails visibly.

### Second-pass parallelization

Sequential implementation, no parallelization opportunity: one stack of nine PRs on `dev`, worked inline in one session,
with no subagents or worktrees in this repository.

### Second-pass Implementation Tasks

Synthesized from this pass's findings. Each task derives from a specific finding above. Effort ratios assumed: features
~30x, bug fix with regression ~20x, tests ~50x human to CC.

- [ ] **T11 (P1, human: ~2 h / CC: ~15 min)** — U4 xr runner — Install the panic hook on the `run_argv` path only
  - Surfaced by: Section 1 A1 and Section 3 T2 — R8 (D2)
  - Files: `crates/xurl-cli/src/cli/runner.rs`, `crates/xurl-cli/src/cli/panic_hook.rs`,
    `crates/xurl-cli/tests/panic_hook_tests.rs`
  - Verify: the suite-built panicking program exits 1 with one `internal` record; the in-process `catch_unwind` test
    passes
- [ ] **T12 (P1, human: ~15 min / CC: ~3 min)** — U4 tests — Assert the exit status in `sigpipe_smoke`
  - Surfaced by: Section 3 T1 — R12 (D6)
  - Files: `crates/xurl-cli/tests/binary_contract_tests.rs`
  - Verify: `cargo test -p xurl-rs --test binary_contract_tests sigpipe_smoke`, observed failing against a hook that
    prints no `panicked at`
- [ ] **T13 (P1, human: ~30 min / CC: ~5 min)** — U3 xdk forward — Turn redirects off and name 2xx as delivered
  - Surfaced by: Section 1 A5 — R11 (D3)
  - Files: `crates/xdk/src/webhooks/forward.rs`, `crates/xdk/tests/webhook_forward_tests.rs`
  - Verify: the 302, 307, 308, and 204 scenarios in U3
- [ ] **T14 (P2, human: ~3 h / CC: ~20 min)** — U6 xr listen — Record the oldest undelivered arrival in the checkpoint
  - Surfaced by: Section 1 A2 — R7 (D1)
  - Files: `crates/xurl-cli/src/cli/commands/webhooks_forward.rs`,
    `crates/xurl-cli/src/cli/commands/webhooks_listen.rs`, `crates/xurl-cli/tests/webhooks_forward_tests.rs`
  - Verify: the kill, drain-deadline, failed-event, and stalled-target scenarios in U6
- [ ] **T15 (P2, human: ~30 min / CC: ~5 min)** — U3 xdk forward — Send `Idempotency-Key` as quoted hex and refuse it in
  `-H`
  - Surfaced by: Section 2 C1 — R9 (D5)
  - Files: `crates/xdk/src/webhooks/forward.rs`, `crates/xdk/tests/webhook_forward_tests.rs`
  - Verify: the U3 key and refusal scenarios
- [ ] **T16 (P2, human: ~30 min / CC: ~5 min)** — U9 xr listen — Add `test_request` to the `listening` record
  - Surfaced by: Section 1 A3 — R10 (D4)
  - Files: `crates/xurl-cli/src/cli/commands/webhooks_listen.rs`, `crates/xurl-cli/tests/webhooks_tests.rs`
  - Verify: `test_request` posted with `reqwest` verifies, on the U8 Windows job too
- [ ] **T17 (P2, human: ~30 min / CC: ~5 min)** — U8 xr env — Read `PATH` and `PATHEXT` through `env.rs` and exempt them
  in the help guard
  - Surfaced by: Section 2 C2
  - Files: `crates/xurl-cli/src/cli/env.rs`, `crates/xurl-cli/tests/agentic_tests.rs`, `AGENTS.md`
  - Verify: `cargo test -p xurl-rs --test agentic_tests test_help_advertises_every_env_var_the_source_reads`
- [ ] **T18 (P2, human: ~20 min / CC: ~3 min)** — U4 records — Set the template's `--to` to the minute after arrival
  - Surfaced by: R13 (D7)
  - Files: `crates/xurl-cli/src/cli/commands/webhooks_forward.rs`, `crates/xurl-cli/tests/webhooks_forward_tests.rs`
  - Verify: the U4 template scenario across an hour and a day boundary
- [ ] **T19 (P3, human: ~15 min / CC: ~3 min)** — U1, U3, U9 docs — Mark binding or sending doctests `no_run`
  - Surfaced by: Section 3 T4
  - Files: `crates/xdk/src/webhooks/`
  - Verify: `cargo test --doc -p xdk-rs --features testing` finishes without binding a fixed port or sending
- [ ] **T20 (P3, human: ~15 min / CC: ~3 min)** — U8 tests — Use the suite's programs in the success and group-kill
  scenarios
  - Surfaced by: Section 3 T3
  - Files: `crates/xurl-cli/tests/webhooks_forward_tests.rs`
  - Verify: the U8 scenarios pass on the `windows-latest` job

*No new tasks from Section 4 (Performance).*

### Second-pass unresolved decisions

None.

### Second-pass suppressed findings

None below confidence 5.

### Second-pass completion summary

- Step 0: Scope Challenge — scope accepted as-is
- Reality rows: 7/7 emitted (`PLAN_REALITY: phase=eng rows=7/7 verdict=complete`), 1 unverified premise (equal replay
  bounds, now hedged by R13)
- Architecture Review: 5 issues found (A1 to A5)
- Code Quality Review: 3 issues found (C1 to C3), 1 finding kept (C4)
- Test Review: diagram produced, 4 gaps identified (T1 to T4)
- Performance Review: 0 issues found
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 1 item proposed to user (R13), answered "build it now"
- Failure modes: 0 critical gaps flagged
- Unresolved decisions: 0 in this review
- Outside voice: codex, disabled (`codex_reviews`)
- Parallelization: 1 lane, 0 parallel / 9 sequential
- Lake Score: 4/5 (D1, D3, D4, D6 picked a 10/10 option; D2 picked 9/10; D5 and D7 differ in kind)
- Scope Challenge findings, reported apart: S1 to S3, all corrected
- Test Plan artifact: `~/.gstack/projects/brettdavies-xurl-rs/brett-dev-eng-review-test-plan-20261022-190522.md`
- Task artifact: `~/.gstack/projects/brettdavies-xurl-rs/tasks-eng-review-20261022-190919.jsonl`

## GSTACK REVIEW REPORT

| Review         | Trigger                                                          | Why                             | Runs | Status      | Findings                                                   |
| -------------- | ---------------------------------------------------------------- | ------------------------------- | ---- | ----------- | ---------------------------------------------------------- |
| CEO Review     | `/plan-ceo-review`                                               | Scope & strategy                | 0    | —           | —                                                          |
| Outside Review | codex, `/plan-eng-review` and `/plan-devex-review` outside voice | Independent 2nd opinion         | 18   | disabled    | none (`codex_reviews` disabled)                            |
| Eng Review     | `/plan-eng-review`                                               | Architecture & tests (required) | 13   | ISSUES OPEN | 12 issues, 0 critical gaps                                 |
| Design Review  | `/plan-design-review`                                            | UI/UX gaps                      | 0    | —           | —                                                          |
| DX Review      | `/plan-devex-review`                                             | Developer experience gaps       | 6    | CLEAR       | score: 6.5/10 → 8.3/10, TTHW: 10+ min (estimated) → <2 min |

- **OUTSIDE COVERAGE:** codex, plan-review phase of the eng first pass, the DX review, and the eng second pass, disabled
  by config; no outside or native reviewer ran.
- **VERDICT:** DX CLEARED. Eng Review's second pass found 12 issues and resolved every one into the plan, revising the
  mechanism of DX D11 (a panic hook) and the shapes of DX D9 (`Idempotency-Key`) and DX D10 (a two-minute window);
  status issues_open because findings were counted; eng review required.

NO UNRESOLVED DECISIONS
