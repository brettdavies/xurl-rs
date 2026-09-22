---
title: Wire Vocabulary Drift - Plan
type: fix
date: 2026-09-20
status: implementation-ready
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
---

# Wire Vocabulary Drift - Plan

## Goal Capsule

- **Objective:** A caller reads one field name per concept, whichever spelling X sends, on every endpoint, without the
  project having to know in advance which endpoints drifted. Drift stops being an incident and becomes a no-op.
- **Means:** Derive the legacy-to-current name table from the vendored spec, alias the three declared fields that lack
  one, and normalize legacy keys once in `decode()`, the typed-only path, before serde parses the body. Emit a `tracing`
  event when a normalization fires, so ordinary use reports drift instead of a paid probe.
- **Authority:** The wire is ground truth and is allowed to disagree with the spec. The library's job is to make that
  disagreement invisible to the caller and visible to the maintainer.
- **Execution profile:** Ships as a patch. One visible change: responses report post vocabulary throughout, so `xr post
  --output json` reports `edit_history_post_ids` where it reported `edit_history_tweet_ids`. Raw mode is untouched and
  still prints exactly what X sent.
- **Stop conditions:** Stop and ask if a derived pair passes the KTD1 admission rule yet turns out to be two distinct
  fields rather than a rename.

## Product Contract

### Problem Frame

X renamed its post vocabulary, the vendored spec followed, and the wire followed unevenly. Three instances surfaced one
at a time, each found by a user rather than a gate:

| Found      | Field                    | Spec                    | Wire                     | Fix  |
| ---------- | ------------------------ | ----------------------- | ------------------------ | ---- |
| v3.0.0     | user `public_metrics`    | `post_count`            | `tweet_count`            | #118 |
| 2026-09-17 | `send_dm` response shape | `dm_event_id`           | required `id` absent     | #194 |
| 2026-09-20 | `POST /2/tweets`         | `edit_history_post_ids` | `edit_history_tweet_ids` | none |

The third was found by posting the launch announcement with `xr post`. The response carried `"edit_history_tweet_ids":
[""]`: the legacy name, and an empty string where the post id belongs. `GET /2/tweets/<id>` returns the id correctly, so
the drift is the create response alone.

**The spec is mid-migration, which is why this recurs.** A scan of the vendored 2.168 spec finds 12 property names
containing `post` and 19 containing `tweet`, with `post_count`/`tweet_count` and `post_id`/`tweet_id` present under
both. This is a rename in progress with no announced end.

**A snapshot of today's wire is not worth much.** Probing every endpoint once would record what X sends today, and X may
change any of it without notice. That makes detection-by-probe a poor foundation: it costs credits, ages immediately,
and still leaves the caller handling two spellings in the meantime. Accepting both spellings costs nothing and does not
age.

**What the scan actually found.** Of the 12 derived pairs, only four are declared struct fields, and only one carries an
alias:

| Spec name               | Legacy spelling          | Declared | Aliased |
| ----------------------- | ------------------------ | -------- | ------- |
| `post_count`            | `tweet_count`            | yes      | yes     |
| `posts`                 | `tweets`                 | yes      | no      |
| `referenced_posts`      | `referenced_tweets`      | yes      | no      |
| `repost_count`          | `retweet_count`          | yes      | no      |
| `edit_history_post_ids` | `edit_history_tweet_ids` | no       | n/a     |
| `pinned_post_id`        | `pinned_tweet_id`        | no       | n/a     |
| `most_recent_post_id`   | `most_recent_tweet_id`   | no       | n/a     |
| `total_post_count`      | `total_tweet_count`      | no       | n/a     |
| `previous_post_id`      | `previous_tweet_id`      | no       | n/a     |
| `post_id`               | `tweet_id`               | no       | n/a     |
| `note_post`             | `note_tweet`             | no       | n/a     |
| `cluster_posts_results` | `cluster_tweets_results` | no       | n/a     |

The eight undeclared names land in the `#[serde(flatten)] extra` bucket and reach the caller exactly as X spelled them.
That is why `edit_history_tweet_ids` arrived untouched. An alias cannot help a field that does not exist, and declaring
all eight would add public API for names nobody calls.

Two of the 12 are not safe to rename everywhere. `tweet_count` is the current spec name on `Trend`, and `tweet_id` is
the current spec name on `Broadcast`, `CreateUsersBookmarkRequest`, `CreateUsersBookmarkResponse`, `LikePostRequest`,
and `RepostPostRequest`. KTD1 excludes both from normalization; user metrics keep their `tweet_count` alias.

**Nothing catches this today.** `crates/xdk/tests/live_smoke.rs:79` asserts two hand-written names are absent from one
`extra` bucket on one read. Those two were added by hand after #118 and the list has not grown across two further
incidents. The mocked suite validates against the vendored spec, so it agrees with the spec by construction and cannot
see the wire disagree, which `RELEASES-PREFLIGHT.md:245` already states.

### Requirements

- **R1.** The legacy-to-current table is derived from the vendored spec, never hand-maintained. A spec refresh that
  renames another field extends coverage with no source edit.
- **R2.** A declared field is filled by either spelling.
- **R3.** An undeclared legacy key is normalized to its current spelling as it deserializes, so `extra` presents one
  vocabulary.
- **R4.** Values are passed through as received. Normalization renames keys and never repairs or synthesizes a value. It
  drops a value in one case only: when an object carries both spellings, the current spelling and its value are kept,
  the legacy key is removed, and the event reports `collision = true`. The empty array X sends stays empty.
- **R5.** A normalization that fires emits a `tracing` event carrying schema identifiers only: the legacy key, its
  current spelling, the JSON type of the value, a length for a string, array, or object, and whether the rename collided
  with an existing current key. Never a value, never an id, never a path containing one, never anything about the app,
  the user, or the credential.
- **R6.** Raw mode is untouched. `xr /2/tweets/<id>` prints exactly what X sent, because a caller asking for the raw
  body is asking for the raw body.
- **R7.** No API call is required to adopt or maintain any of this.
- **R8.** This ships as a patch. Changes to the public API are additive only: one new `pub const` tracing target in
  `xdk::api`. No type or signature changes, nothing `cargo semver-checks` reports at `--release-type patch`.

### Success Criteria

- A fixture spelling any of the 10 admitted pairs either way produces identical typed output, and `tweet_count` on user
  metrics fills `post_count` through its alias.
- `xr post --output json` reports `edit_history_post_ids` carrying the empty array X sent.
- Injecting a thirteenth renamed property into a spec fixture extends coverage with no source edit.

### Scope Boundaries

**In scope.** Response vocabulary on the typed path, the derivation, the aliases, the normalization, and its telemetry.

**Out of scope.**

- Request-side vocabulary. The library sends `post.fields` everywhere and X accepts it; the spec still names
  `tweet.fields` on 10 endpoints against `post.fields` on 1. Recorded, not changed.
- Declaring the eight undeclared names as public fields. Normalization makes that unnecessary.
- Synthesizing a value for X's empty array, which R4 forbids.
- A live probe of every endpoint, which the Problem Frame rejects on its own terms.
- `send_dm`, fixed by #194.

## Planning Contract

### Key Technical Decisions

**KTD1. Derive the pairs from the spec; never hand-list them.** Walk every `properties` object in
`crates/xdk/vendor/x-api-openapi.json`, collect names containing `post`, and map each to its legacy spelling by
substituting `repost` to `retweet` and `post` to `tweet`. Hand-listing is what left `live_smoke.rs` frozen at two names
across three incidents.

A pair is admitted only when its legacy spelling is not itself a property name anywhere in the spec. Against 2.168 that
excludes `tweet_count` and `tweet_id`, leaving 10 admitted pairs. The build script emits the excluded pairs as a second
table so the exclusion is visible and testable, and a spec refresh re-evaluates the rule with no source edit.

**KTD2. Normalize in one place; alias for direct serde users.** Normalization in `decode()` covers all 10 admitted pairs
on the library's own path, declared fields included, and every future rename without adding public API. Three one-line
aliases on `posts`, `referenced_posts`, and `repost_count` serve embedders that deserialize the types with serde
directly. A test ties the aliases to the table, so the two mechanisms cannot disagree.

**KTD3. Normalize in `decode()`, the typed-only chokepoint.** Every typed call reaches `decode()`
(`crates/xdk/src/api/response/types.rs:458`): `call.rs:176` and `deserialize_response` both route through it. Raw mode
calls `send_request` directly (`crates/xurl-cli/src/cli/commands/mod.rs:359`) and never does, so R6 holds by
construction. `decode()` owns the `Value`, so one in-place walk renames table keys at any depth before
`serde_json::from_value`. No response struct carries a normalization attribute, so a struct added later cannot forget
one.

On collision, when an object already carries the current spelling, the current key and value are kept and the legacy key
is removed before serde sees it; serde would otherwise fail the whole response with `duplicate field` on an aliased
field (observed with serde 1).

**KTD4. Telemetry replaces the probe, and it stays local and schema-only.**

*Destination.* The library emits into the `tracing` facade and never touches a terminal, which
`crates/xurl-cli/src/cli/output/diagnostics.rs:4` states as its contract. `xurl-cli` installs `Diagnostics` as a
subscriber scoped to one dispatch (`runner.rs:274`, `runner.rs:299`) and renders to stderr. Nothing is written to disk,
nothing crosses the network, and no telemetry service exists. An embedder that installs no subscriber drops the event at
the macro.

*Level.* Not `warn`. `Diagnostics::render` returns early for `Level::WARN` before any gate, so a warn prints on every
invocation including `--quiet` and `--output json`, which `scripts/lint-stdio.sh` exists to prevent. A write endpoint
that always answers in legacy vocabulary would emit on every `xr post`. Declare a target beside `WIRE_TARGET` and
`MEDIA_TARGET`, emit at `debug`, and gate it in `render` the way `WIRE_TARGET` is gated, so it surfaces under
`--verbose` and nowhere else.

*Payload, and nothing beyond it.* Schema identifiers only:

| Field        | Example                  | Why it is safe                                              |
| ------------ | ------------------------ | ----------------------------------------------------------- |
| `legacy`     | `edit_history_tweet_ids` | A name from X's public spec                                 |
| `normalized` | `edit_history_post_ids`  | A name from X's public spec                                 |
| `value_type` | `array`                  | JSON type name; carries no content                          |
| `value_len`  | `1`                      | Length only, for string, array, or object                   |
| `collision`  | `false`                  | Whether the current spelling was already present; a boolean |

Never the value. A value can hold post text, a username, a DM, or an id. Never a request path, which carries ids
(`/2/tweets/2101712260468977783`). Never the app name, the authenticated user, the credential, or the scheme. If context
beyond the key pair is ever wanted, use the struct name, which contains no instance data.

*Rate.* Deduplicate per legacy key per `decode()` call, with a set local to the walk, so a response that repeats a key
across many posts reports once. The library holds no global state for this; a paginated call reports once per page.

**KTD5. The live write gate is optional, not required.** Probing catches drift that does not follow the
`post`-to-`tweet` pattern, which is real but speculative. KTD4 covers the same ground continuously and for free, and a
non-empty `extra` surfaces the rest whenever anyone looks. If it is added later, it costs one post and one delete per
run and belongs behind the existing `XURL_LIVE_SMOKE=1` opt-in.

### Sequencing

U1 first; everything reads its table. U2 and U3 are independent of each other. U4 depends on U3. U5 depends on U4,
because the live smoke reads U4's events.

## Implementation Units

### U1. Derive the name table

**Goal.** R1, R7, KTD1.

**Files.** `crates/xdk/build.rs` or a shared module; `crates/xdk/vendor/x-api-openapi.json` as input, unchanged.

**Approach.** Emit the admitted and excluded tables where both the aliases' tests and the normalizer can read them. The
build script is the natural home, beside the existing spec codegen, because the normalizer is production code. Write the
derivation as a pure function over a spec `Value` in a file the build script and a unit test both include, so a test can
feed it a spec fixture.

**Test scenarios.** The admitted table holds the 10 pairs, including `repost_count` → `retweet_count`. The excluded
table holds `tweet_count` and `tweet_id`. A spec fixture with an added post-named property extends the admitted table
with no source edit. A fixture where the added property's legacy spelling is also a spec property lands in the excluded
table.

### U2. Alias the three declared fields

**Goal.** R2.

**Files.** `crates/xdk/src/api/response/types.rs`.

**Approach.** `#[serde(alias = "...")]` on `posts`, `referenced_posts`, and `repost_count`, matching the shape
`post_count` already uses. A test asserts every declared field whose name appears in the table carries the alias the
table names, so a future declared field cannot ship without one.

**Test scenarios.** Through direct serde, a response spelling `retweet_count` fills `repost_count`, `referenced_tweets`
fills `referenced_posts`, and `tweets` under `includes` fills `posts`. Both spellings of each produce identical output.
Observed failing first. `spec_validation.rs:65` (`tweet_count` → `post_count`) stays green.

### U3. Normalize legacy keys in decode()

**Goal.** R3, R4, R6, R8, KTD3.

**Files.** `crates/xdk/src/api/response/types.rs` (`decode()`), plus a small normalizer module beside it.

**Approach.** `decode()` calls a walk over its owned `Value` before `serde_json::from_value`. The walk renames any key
the admitted table lists, at any depth, and applies the KTD3 collision rule. Keys not in the table pass through. Values
are inspected only for their JSON type and length.

**Test scenarios.** `edit_history_tweet_ids` decodes to `edit_history_post_ids` with its value byte-identical, `[""]`
and `[]` included. Legacy keys nested under `data[*]`, `includes`, and `public_metrics` all normalize. An unrelated
unknown key survives unchanged. `tweet_id` on a bookmark body is never renamed. An object carrying both `retweet_count`
and `repost_count` parses, keeps the `repost_count` value, and drops `retweet_count`. The empty-body and errors-only
paths (`types.rs:898`, `types.rs:1021`) stay green. A raw request still prints the original spelling.

### U4. Report a firing

**Goal.** R5, KTD4.

**Files.** the normalizer, `crates/xdk/src/api/` for the new target constant,
`crates/xurl-cli/src/cli/output/diagnostics.rs` for the render arm.

**Approach.** Declare a `pub const` target beside `WIRE_TARGET` and `MEDIA_TARGET`, emit at `debug` carrying the five
fields KTD4 tabulates, and add a `render` arm gated by `verbose_enabled()`. Deduplicate per legacy key per `decode()`
call.

**Test scenarios.** A planted legacy key produces exactly one event carrying the key pair, the JSON type, the length,
and `collision`, and no other field. A key repeated across 3 posts in one response produces one event. A collision
produces one event with `collision = true`. A `Diagnostics` render test in the style of `diagnostics.rs:239` shows the
line under `--verbose` in text mode and nothing under `--quiet`, `--output json`, `--output jsonl`, or without
`--verbose`. An end-to-end test on the `cli_diagnostics_tests.rs` harness runs `xr --verbose post` against a
legacy-spelled wiremock and checks stdout and stderr. A grep guard asserts the emit site passes no value, no path, and
no identifier.

### U5. Record what the wire does

**Goal.** Keep instance four from being rediscovered.

**Files.** `crates/xdk/tests/live_smoke.rs`, `docs/solutions/`, `RELEASES-PREFLIGHT.md`, the release notes.

**Approach.** One entry: the spec is mid-migration, the wire lags per-endpoint, a snapshot of the wire is worth less
than accepting both spellings, and the detector is telemetry rather than a probe. Name the three instances. Note the
unconfirmed lead that writes answer in legacy vocabulary because they carry no field-selection parameter, with #118 as
its counterexample. Commit with `sd-commit-doc`.

The live smoke installs a test `tracing` subscriber for the U4 target around its two existing reads, one post and one
user, and fails listing every `legacy → normalized` pair reported. The hand-written list at `live_smoke.rs:79` is
removed; the typed-metrics-nonzero and media-key assertions stay. Paid reads stay at 2. `RELEASES-PREFLIGHT.md:242-249`
describes this check and its blind spot: excluded pairs and alias-only fields never emit, so `tweet_count` drift on user
metrics stays silent.

## Verification Contract

| Gate             | Command                                                                 | Done signal                            |
| ---------------- | ----------------------------------------------------------------------- | -------------------------------------- |
| Format           | `cargo fmt -- --check`                                                  | No diff                                |
| Lint             | `cargo clippy --all-targets -- -D warnings`                             | Clean                                  |
| Tests            | `cargo test`                                                            | All pass, both-spelling cases included |
| Golden           | `cargo test --test golden_tests`                                        | No golden changes                      |
| Schema freshness | `cargo test --test schema_tests`                                        | Drift test passes                      |
| Public API       | `cargo semver-checks --baseline-rev xdk-rs-v0.1.0 --release-type patch` | Passes                                 |
| Raw passthrough  | A raw request against a legacy-spelling fixture                         | Original spelling preserved            |
| Markdown         | `markdownlint-cli2 RELEASES-PREFLIGHT.md`                               | Zero issues                            |

## Definition of Done

- The 10 admitted pairs are normalized in `decode()`, `tweet_count` and `tweet_id` sit in the excluded table,
  `tweet_count` on user metrics stays covered by its alias, and a thirteenth property injected into a spec fixture is
  admitted with no source edit.
- An object carrying both spellings parses, keeps the current value, and reports `collision = true`.
- The live smoke fails on any normalization event during its two reads, with no hand-written key list.
- `xr post --output json` reports `edit_history_post_ids` carrying the empty array X sent, unrepaired.
- A raw request still prints X's own spelling.
- `cargo semver-checks --release-type patch` passes, and the release is cut as a patch.
- The telemetry emit site carries only the five fields KTD4 names, asserted by a guard rather than by review.
- The `docs/solutions/` entry is written and pushed with `sd-commit-doc`.
- The PR body's `## Changelog (xdk-rs)` names the normalization with a before and after snippet, which
  `crates/xdk/README.md` makes a release gate.

## Decision ledger

Review target: `docs/plans/2026-09-20-1142-fix-wire-vocabulary-drift-plan.md` (/plan-eng-review, 2026-09-22).

### S0: Scope record (complexity gate)

- **Feature answers:** none proposed.
- **Structure:** B, Smaller arrangement (D1, answered 2026-09-22).
- **Accepted scope:** undeclared-key normalization runs once inside `decode()`
  (`crates/xdk/src/api/response/types.rs:458`) over the JSON tree before `serde_json::from_value`, renaming table keys
  at any depth. No per-struct `deserialize_with` attributes. R1-R8, the build.rs-derived table, the three aliases, and
  the telemetry are unchanged.
- **Pending remedies:** R2, R3, plus any raised in later sections.

### R1: Legacy spellings that are also current spec names

- **Finding:** 1, P1, confidence 9/10, KTD1 (plan line 123), reviewer: plan-eng-review.
- **Plan baseline:** KTD1 derives every pair by `post`→`tweet` substitution and applies it as one global table; the stop
  condition asks a human to catch a pair that is two distinct fields.
- **Runtime evidence:** `jq` over `crates/xdk/vendor/x-api-openapi.json` shows `tweet_count` is a declared property of
  `Trend` and `tweet_id` a declared property of `Broadcast`, `CreateUsersBookmarkRequest`,
  `CreateUsersBookmarkResponse`, `LikePostRequest`, `RepostPostRequest`. A global rename rewrites those correct fields.
- **Comparison grid:**

| Choice                          | Current            | A                                                                               | B                                     | C                                 |
| ------------------------------- | ------------------ | ------------------------------------------------------------------------------- | ------------------------------------- | --------------------------------- |
| Pair admission rule             | every derived pair | exclude a pair whose legacy spelling is a current property anywhere in the spec | per-schema table keyed by spec schema | every derived pair; human reviews |
| Derivation stays automatic (R1) | yes                | yes                                                                             | yes                                   | no, depends on review             |
| Excluded pairs today            | none               | `tweet_count`, `tweet_id`                                                       | none, scoped instead                  | none                              |
| `post_count` on user metrics    | alias              | alias (unchanged)                                                               | alias                                 | alias                             |

- **Question D2:**
  - D2 — How should the derived table handle legacy names that are also real spec names?
  - Project/branch/task: dev, wire-vocabulary-drift plan, KTD1/U1.
  - ELI10: The plan builds its rename list by swapping `post` for `tweet`. Two of the results, `tweet_count` and
    `tweet_id`, are not old names at all in some places: the spec uses them today on Trend and on five
    bookmark/like/repost/broadcast schemas. A single global rename would rewrite those correct fields into wrong ones.
  - Stakes if we pick wrong: a correct `tweet_id` on a bookmark or like response gets renamed to `post_id`, and the
    table silently corrupts the vocabulary it was meant to fix.
  - Recommendation: A because it keeps R1 fully automatic with one rule a build script can check, and the only casualty,
    `tweet_count` on user metrics, is already covered by its per-struct alias.
  - Completeness: A=10/10, B=9/10, C=6/10
- **Header:** Ambiguous pairs
- **Options:**
  - A) Exclude ambiguous (recommended): build.rs admits a pair only when its legacy spelling is not itself a property
    name anywhere in the spec, emits the excluded pairs as a second table, and a test pins that `tweet_count` and
    `tweet_id` are excluded. Today that drops 2 of 12 pairs from normalization. Per-struct aliases stay the tool for
    declared fields. Effort human ~1h / CC ~10min. Maintenance: none; a spec refresh re-evaluates the rule.
  - B) Per-schema table: build.rs keys the table by spec schema and renames only inside objects of a schema that
    declares the current name. Precise, but the decode() walk sees untyped JSON and would need a Rust-type-to-schema map
    to know which schema an object is. Effort human ~4h / CC ~40min. Maintenance: the type-to-schema map is a new
    hand-kept list.
  - C) Keep plan as written: Global table of all 12; rely on the stop condition for a human to spot distinct fields.
    Breaks R1's never-hand-maintained rule the first time the human has to act. Effort none now.
- **State:** approved
- **Actual answer:** A) Exclude ambiguous (D2, answered 2026-09-22)
- **Accepted scope:** build.rs admits a derived pair only when its legacy spelling is not a property name anywhere in
  the vendored spec; the excluded pairs are emitted as a second table; a test pins `tweet_count` and `tweet_id` as
  excluded. Normalization covers 10 pairs today. Per-struct aliases remain the mechanism for declared fields, and
  `UserPublicMetrics.post_count` keeps its `tweet_count` alias.
- **History:** none

### R2: Both spellings arrive in one object

- **Finding:** 2, P1, confidence 9/10, U2 (plan line 204) and U3 (plan line 215), reviewer: plan-eng-review.
- **Plan baseline:** no collision policy. R4 says normalization never drops a value.
- **Runtime evidence:** local serde probe (serde 1, serde_json 1): a struct with `#[serde(default, alias =
  "tweet_count")] post_count` plus `#[serde(flatten)] extra` fails on `{"tweet_count":1,"post_count":2}` with `duplicate
  field post_count`. The shipped `UserPublicMetrics.post_count` alias (`types.rs:215`) already has this latent failure.
  A naive rename in decode() would overwrite one value.
- **Comparison grid:**

| Choice                              | Current                                              | A                                                                  | B                                                      | C                        |
| ----------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------ | ------------------------ |
| Collision outcome                   | parse error on aliased fields; unspecified for extra | current spelling kept, legacy dropped, event records the collision | legacy left unrenamed; aliases removed so both survive | parse error (status quo) |
| R4 wording                          | never drops                                          | amended: drops only the legacy copy on collision, reported         | unchanged                                              | unchanged                |
| Aliases (R2)                        | 1 today, 3 planned                                   | kept                                                               | removed, including shipped `post_count`                | kept                     |
| Direct-serde embedders on collision | error                                                | error (alias)                                                      | no error                                               | error                    |

- **Question D3:**
  - D3 — What happens when X sends both the old and new spelling in one object?
  - Project/branch/task: dev, wire-vocabulary-drift plan, U2/U3 plus R4.
  - ELI10: During a rename, an API often sends both names for a while. Today, if a user object carried both
    `tweet_count` and `post_count`, serde would refuse the whole response with "duplicate field". The plan adds three
    more aliases with the same trap and has no rule for the rename step either.
  - Stakes if we pick wrong: `xr user` or `xr read` hard-fails the day X starts dual-sending, which is the likeliest
    next step in a migration that is already half done.
  - Recommendation: A because the call never fails, the kept value is the spec's own, and the telemetry event makes the
    drop visible to the maintainer.
  - Completeness: A=10/10, B=8/10, C=4/10
- **Header:** Collision policy
- **Options:**
  - A) Current wins (recommended): In decode(), when both keys exist in one object, keep the current spelling, remove
    the legacy key before serde sees it, and emit the event with `collision = true`. R4 is amended to say so. Aliases
    stay for direct-serde embedders, who keep the parse error on collision. Tests: both spellings in one object for an
    aliased field and an extra field, each parses, current value kept, one event. Effort human ~1h / CC ~10min.
  - B) Keep both, drop aliases: decode() leaves the legacy key in place on collision so it lands in `extra`; the three
    planned aliases are not added and the shipped `post_count` alias is removed so serde never sees a duplicate. Both
    values survive. Direct-serde embedders lose alias fill on the normal non-collision path, which is a behavior
    regression for them. Effort human ~1h / CC ~10min.
  - C) Keep status quo: Collision stays a hard parse error for aliased fields and is unspecified for extra keys. No work
    now.
- **State:** approved
- **Actual answer:** A) Current wins (D3, answered 2026-09-22)
- **Accepted scope:** in decode(), when the legacy and current spelling both exist in one object, the current key and
  its value are kept, the legacy key is removed before serde sees it, and the event fires with `collision = true`. R4 is
  amended: normalization drops a value only in that case, and the event reports it. The three new aliases and the
  shipped `post_count` alias stay; a direct-serde embedder still gets the serde duplicate-field error on collision.
  Tests: both spellings in one object for an aliased field and for an extra field, each parses, the current value is
  kept, one event fires with `collision = true`.
- **History:** none

### R3: Where the one-event-per-key dedup lives

- **Finding:** 3, P2, confidence 8/10, KTD4 "Rate" (plan line 172) and U4 (plan line 232), reviewer: plan-eng-review.
- **Plan baseline:** deduplicate per legacy key per process, which needs a process-global mutable set in the library.
- **Runtime evidence:** `crates/xdk/src` holds no mutable global state today (the one static, `store/mod.rs:539`, is an
  immutable `LazyLock<App>`). Tests in one test binary share a process, so a global set makes "exactly one event" depend
  on which test fired first.
- **Comparison grid:**

| Choice                       | Current                  | A                                         | B                                               |
| ---------------------------- | ------------------------ | ----------------------------------------- | ----------------------------------------------- |
| Dedup scope                  | per process (global set) | per decode() call (local set in the walk) | per process (global set)                        |
| Library mutable global state | none today               | none                                      | first instance                                  |
| Test determinism             | n/a                      | deterministic                             | order-dependent unless tests reset or serialize |
| Events per `xr` invocation   | 1 per key                | 1 per key per response page               | 1 per key                                       |

- **Question D4:**
  - D4 — Where should "report each legacy key once" be enforced?
  - Project/branch/task: dev, wire-vocabulary-drift plan, KTD4/U4.
  - ELI10: The plan wants each legacy key reported once per process, which needs a shared global list inside the
    library. The library has no mutable globals today, and tests share one process, so "exactly one event" would pass or
    fail depending on test order. Deduping inside one decode() call gives the same result for a normal `xr` run and
    needs no shared state.
  - Stakes if we pick wrong: a flaky telemetry test, plus the library's first global mutable state, against the
    project's test-isolation rule.
  - Recommendation: A because it is deterministic, stateless, and matches the project's explicit-injection test rule;
    the only cost is one event per page on a paginated call.
  - Completeness: A=10/10, B=7/10
- **Header:** Dedup scope
- **Options:**
  - A) Per decode() call (recommended): The tree walk carries a local set and emits at most one event per legacy key per
    response. No global state. Tests assert exactly one event for a response that repeats a key across 3 posts. A
    paginated call emits once per page. Effort human ~30min / CC ~5min.
  - B) Per process (plan): A process-global `LazyLock<Mutex<HashSet>>` in the library. Tests need a reset hook or
    serialization to stay deterministic, which the project's no-`#[serial]` rule forbids. Effort human ~1h / CC ~10min.
- **State:** approved
- **Actual answer:** A) Per decode() call (D4, answered 2026-09-22)
- **Accepted scope:** the decode() tree walk carries a local set and emits at most one event per legacy key per
  response. The library holds no global state for this. A test asserts exactly one event for a response that repeats a
  legacy key across 3 posts. A paginated call emits once per page.
- **History:** none

### FC: Factual corrections (no behavior change, no question required)

- **FC1.** Goal Capsule says KTD3 records a settling spike; KTD3 says to verify first. A local probe confirmed
  `#[serde(flatten, deserialize_with)]` composes, and S0 moves normalization into `decode()`, so KTD3 is rewritten to
  the decode() decision.
- **FC2.** Verification Contract runs `cargo semver-checks --release-type minor`; R8 and the DoD require patch. Use
  `--release-type patch`. R8's "no public type changes" is corrected to "additive only": the new tracing target is a
  `pub const` in `xdk::api` beside `WIRE_TARGET` and `MEDIA_TARGET`, because `diagnostics.rs:17` imports it.
- **FC3.** Golden gate expects create-post fixtures to change. No fixture under `crates/` carries `edit_history_*` or
  any legacy key, so the expectation is zero golden changes.
- **FC4.** R5/KTD4 say `value_len` catches the `[""]` case. A length of 1 does not distinguish `[""]` from a real id;
  the claim is removed.
- **FC5.** DoD "four by alias, eight by normalization" becomes: 10 pairs normalized in decode(), `tweet_count` covered
  on user metrics by its existing alias, `tweet_id` excluded (R1/D2). The three new aliases serve embedders that call
  serde directly.
- **FC6.** U4 says `scripts/lint-stdio.sh` asserts the event is absent under `--quiet` and structured output. The script
  is a static grep for print macros and terminal handles; the runtime gate is proven by a `Diagnostics` render test in
  the style of `crates/xurl-cli/src/cli/output/diagnostics.rs:239` plus the cross-mode table.

### R4: The live smoke loses its drift signal (regression)

- **Finding:** 4, P1, confidence 9/10, `crates/xdk/tests/live_smoke.rs:79` and U5 (plan line 241), reviewer:
  plan-eng-review. REGRESSION RULE applies.
- **Plan baseline:** U5 rewrites the `RELEASES-PREFLIGHT.md:242-249` prose; the test itself is unchanged.
- **Runtime evidence:** `live_smoke.rs:79` loops `for legacy in ["referenced_tweets", "edit_history_tweet_ids"]` and
  asserts `!post.data.extra.contains_key(legacy)`. After S0, decode() renames both keys before serde, so the assertion
  passes whatever X sends. The one pre-tag gate that sees the wire goes blind, and its list stays hand-written.
- **Behavior to preserve:** the live smoke fails a release when X answers the typed reads in legacy vocabulary; the
  typed-metrics-nonzero and media-key assertions stay.
- **Comparison grid:**

| Choice                     | Current                               | A                                                  | B                                                          |
| -------------------------- | ------------------------------------- | -------------------------------------------------- | ---------------------------------------------------------- |
| Drift signal source        | `extra` absence of 2 hand-listed keys | normalization events captured by a test subscriber | raw body scanned for every legacy key in the derived table |
| Key list                   | hand-written, 2 names                 | derived table (via the events)                     | derived table                                              |
| Extra API calls per run    | 0 (2 reads total)                     | 0                                                  | 2 (one raw read each for post and user)                    |
| Also proves telemetry live | no                                    | yes                                                | no                                                         |

- **Question D5:**
  - D5 — How should the live smoke keep catching wire drift once normalization hides it?
  - Project/branch/task: dev, wire-vocabulary-drift plan, U5 and `crates/xdk/tests/live_smoke.rs:79`.
  - ELI10: The live smoke is the only pre-release check that talks to the real X API. It catches drift by looking for
    old key names in the parsed result. After this change, the library always renames those keys first, so the check can
    never fail again. It needs a new place to look.
  - Stakes if we pick wrong: the next spec-vs-wire drift ships unnoticed, the exact failure behind all three incidents.
  - Recommendation: A because it costs zero extra paid calls, uses the derived table instead of a hand list, and proves
    the telemetry detector works against the real wire.
  - Completeness: A=10/10, B=9/10
- **Header:** Live smoke
- **Options:**
  - A) Capture events (recommended): the smoke installs a test `tracing` subscriber for the new target around its two
    existing reads and fails listing every `legacy → normalized` pair reported. The hand list at `live_smoke.rs:79` is
    deleted. `RELEASES-PREFLIGHT.md:242-249` is updated to say this. Same 2 paid reads. Effort human ~1h / CC ~10min.
  - B) Scan raw body: the smoke adds one raw read each for the post and the user and fails when any legacy key from the
    derived table appears anywhere in the body. Independent of telemetry. Doubles the paid reads to 4. Effort human ~1h
    / CC ~10min.
- **State:** approved
- **Actual answer:** A) Capture events (D5, answered 2026-09-22)
- **Accepted scope:** `live_smoke.rs` installs a test `tracing` subscriber for the new target around its two existing
  reads (one post, one user) and fails listing every `legacy → normalized` pair reported. The hand list at
  `live_smoke.rs:79` is deleted. The typed-metrics-nonzero and media-key assertions stay.
  `RELEASES-PREFLIGHT.md:242-249` describes the event-based check. Paid reads stay at 2.
- **History:** none

Approval readiness: PASS. Checked S0 (D1), R1 (D2), R2 (D3), R3 (D4), R4 (D5, regression contract); each cites its own
answer from 2026-09-22. FC1-FC6 are factual corrections with no behavior change.

## Engineering review notes

### NOT in scope

- Request-side vocabulary (`tweet.fields` on 10 spec endpoints): the library sends `post.fields` and X accepts it; the
  plan's Scope Boundaries already record it.
- Drift that does not follow the `post`→`tweet` pattern: lands in `extra` unrenamed and emits nothing; KTD5 keeps the
  live write probe optional.
- Telemetry for excluded pairs and alias-only fields: inherent to D2, since the walk cannot tell a user-metrics
  `tweet_count` from a `Trend` one; documented in `RELEASES-PREFLIGHT.md` per U5.
- Distribution: no new artifact; both release lines ship through their existing pipelines.

### What already exists

- `decode()` (`crates/xdk/src/api/response/types.rs:458`): the typed-only chokepoint the plan now reuses instead of
  adding 21 per-struct attributes.
- `build.rs:194`: already reads the vendored spec with `rerun-if-changed`; U1 extends it rather than adding a script.
- `UserPublicMetrics.post_count` alias (`types.rs:215`) and its wire-shape test
  (`crates/xdk/tests/spec_validation.rs:65`): the pattern U2 copies.
- `Diagnostics` with `WIRE_TARGET`/`MEDIA_TARGET` gating (`crates/xurl-cli/src/cli/output/diagnostics.rs:52-70`) and its
  render tests (`:239`): U4's arm and tests follow them.
- `crates/xurl-cli/tests/cli_diagnostics_tests.rs`: a binary-against-wiremock harness for the end-to-end stderr check.

### Data flow

```text
X response body
      │
      ▼
send_request ──────────────────────────────► raw mode (commands/mod.rs:359): printed verbatim (R6)
      │
      ▼
decode(value)  types.rs:458
  ├─ empty-body / errors-only guards (unchanged)
  ├─ normalize(&mut value, ADMITTED)  ── local seen-set (D4)
  │     for each object, each key k in ADMITTED:
  │        current present? ── yes ─► remove k, emit {collision=true}      (D3)
  │                         └─ no ──► rename k → current, emit {collision=false}
  │     events: target = new pub const, level = debug, fields = 5 (KTD4)
  └─ serde_json::from_value::<T>  (aliases also accept legacy for direct-serde embedders)
      │
      ▼
typed struct ──► print_typed (commands/mod.rs:167) ──► stdout in post vocabulary

build.rs: vendored spec ──► derive(spec) ──► ADMITTED (10) + EXCLUDED (tweet_count, tweet_id)   (D2)
Diagnostics subscriber: renders the event only when verbose_enabled()   (FC6)
live_smoke: test subscriber on the same target; any event fails the preflight   (D5)
```

The normalizer module gets a short version of the `decode` part of this diagram as its module doc.

### Failure modes

| New path    | Realistic failure                                          | Test                   | Handling                             | User sees                                         |
| ----------- | ---------------------------------------------------------- | ---------------------- | ------------------------------------ | ------------------------------------------------- |
| derivation  | a spec refresh adds a name that is both current and legacy | U1 excluded-table test | admission rule excludes it           | nothing wrong                                     |
| decode walk | X dual-sends both spellings                                | U3 collision test      | current wins, event fires            | correct value; `--verbose` line                   |
| decode walk | legacy key nested in `includes` or `data[*]`               | U3 nested test         | walk is recursive                    | current spelling                                  |
| decode walk | a data-keyed map uses a table name as a key                | none                   | table holds spec property names only | silent rename; judged unrealistic for X v2 bodies |
| emit site   | a value or id leaks into the event                         | grep guard             | five-field payload                   | n/a                                               |
| render arm  | line prints under `--quiet` or `--output json`             | render test + e2e      | `verbose_enabled()` gate             | nothing                                           |
| live smoke  | wire drifts on an excluded or alias-only field             | none                   | documented blind spot                | silent; accepted                                  |

Critical gaps: 0. No path lacks both a test and handling while failing silently in a realistic case.

### Worktree parallelization strategy

Sequential implementation, no parallelization opportunity. U2 and U3 both edit `crates/xdk/src/api/response/types.rs`,
and U4 and U5 read U3's events.

## Implementation Tasks

Synthesized from this review's findings. Each task derives from a specific finding above. Run with Claude Code or Codex;
checkbox as you ship.

- [ ] **T1 (P1, human: ~1h / CC: ~10min)** — build.rs — derive admitted and excluded vocabulary tables
  - Surfaced by: Architecture — R1/D2 (`tweet_count`, `tweet_id` are current spec names)
  - Files: `crates/xdk/build.rs`, a derivation file shared with a unit test
  - Verify: `cargo test -p xdk-rs` (admitted 10, excluded 2, injected-fixture cases)
- [ ] **T2 (P1, human: ~2h / CC: ~15min)** — xdk decode — normalize legacy keys in `decode()` with the collision rule
  - Surfaced by: Scope — S0/D1; Architecture — R2/D3 (`duplicate field` on both spellings)
  - Files: `crates/xdk/src/api/response/types.rs`, new normalizer module
  - Verify: `cargo test -p xdk-rs` (nested, collision, excluded, pass-through, empty-body cases)
- [ ] **T3 (P2, human: ~30min / CC: ~5min)** — xdk types — three aliases plus the table-tie guard test
  - Surfaced by: FC5 (aliases now serve direct-serde embedders)
  - Files: `crates/xdk/src/api/response/types.rs`, `crates/xdk/tests/spec_validation.rs`
  - Verify: `cargo test -p xdk-rs --test spec_validation`
- [ ] **T4 (P1, human: ~2h / CC: ~15min)** — telemetry — target const, per-decode dedup, render arm, gating tests
  - Surfaced by: Architecture — R3/D4; FC6 (lint-stdio.sh proves no runtime gating)
  - Files: `crates/xdk/src/api/mod.rs`, normalizer module, `crates/xurl-cli/src/cli/output/diagnostics.rs`,
    `crates/xurl-cli/tests/cli_diagnostics_tests.rs`
  - Verify: `cargo test` (render test, e2e `xr --verbose post`, emit-site grep guard)
- [ ] **T5 (P1, human: ~1h / CC: ~10min)** — live smoke — capture events instead of the hand list
  - Surfaced by: Tests — R4/D5 (regression: `live_smoke.rs:79` goes blind)
  - Files: `crates/xdk/tests/live_smoke.rs`, `RELEASES-PREFLIGHT.md`
  - Verify: `XURL_LIVE_SMOKE=1 cargo test --test live_smoke -- --ignored` at preflight; `cargo test` compiles it
- [ ] **T6 (P2, human: ~15min / CC: ~2min)** — release gates — run the corrected verification contract
  - Surfaced by: FC2, FC3
  - Files: none
  - Verify: `cargo semver-checks --baseline-rev xdk-rs-v0.1.0 --release-type patch`; golden suite shows no changes

## Review completion summary

- Step 0: Scope Challenge — scope reduced per recommendation (structure only: normalization in `decode()`; no feature
  cuts)
- Architecture Review: 3 issues found (R1, R2, R3), all resolved
- Code Quality Review: 6 issues found (FC1-FC6, factual corrections)
- Test Review: diagram produced, 22 gaps identified (all folded as required proof), 1 regression resolved (R4)
- Performance Review: 0 issues found
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 items proposed to user
- Failure modes: 0 critical gaps flagged
- Unresolved decisions: 0 in this review
- Outside voice: codex, disabled (`codex_reviews` disabled; no native fallback by design)
- Parallelization: 1 lane, 0 parallel / 6 sequential
- Lake Score: 4/4 (D2, D3, D4, D5 each chose the 10/10 option; D1 differs in kind)

### Suppressed findings

- (confidence 4/10) KTD1 derivation walks request-body schemas as well as responses. With the D2 admission rule this
  changes nothing today; noted only in case request-only names ever produce a spurious pair.

## GSTACK REVIEW REPORT

| Review         | Trigger                                    | Why                             | Runs | Status                                  | Findings                                               |
| -------------- | ------------------------------------------ | ------------------------------- | ---- | --------------------------------------- | ------------------------------------------------------ |
| CEO Review     | `/plan-ceo-review`                         | Scope & strategy                | 0    | —                                       | —                                                      |
| Outside Review | codex via `/plan-eng-review` outside voice | Independent 2nd opinion         | 10   | disabled                                | none (codex_reviews disabled)                          |
| Eng Review     | `/plan-eng-review`                         | Architecture & tests (required) | 7    | ISSUES OPEN (PLAN)                      | 10 issues, 0 critical gaps; all resolved into the plan |
| Design Review  | `/plan-design-review`                      | UI/UX gaps                      | 0    | —                                       | —                                                      |
| DX Review      | `/plan-devex-review`                       | Developer experience gaps       | 4    | issues_found (2026-09-17, another plan) | not this plan                                          |

- **OUTSIDE COVERAGE:** codex, plan-review phase, disabled by config (`codex_reviews disabled`); no outside findings.
- **VERDICT:** no review CLEAR. Eng Review logs `issues_open` because it found 10 issues, every one resolved in the
  ledger with 0 unresolved; eng review required by the dashboard rule until a clean re-run is logged.

NO UNRESOLVED DECISIONS
