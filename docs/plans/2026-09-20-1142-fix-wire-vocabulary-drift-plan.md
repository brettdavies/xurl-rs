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
  one, and normalize the remaining keys as they deserialize. Emit a `tracing` event when a normalization fires, so
  ordinary use reports drift instead of a paid probe.
- **Authority:** The wire is ground truth and is allowed to disagree with the spec. The library's job is to make that
  disagreement invisible to the caller and visible to the maintainer.
- **Execution profile:** Ships as a patch. One visible change: responses report post vocabulary throughout, so `xr post
  --output json` reports `edit_history_post_ids` where it reported `edit_history_tweet_ids`. Raw mode is untouched and
  still prints exactly what X sent.
- **Stop conditions:** Stop and ask if a derived pair turns out to be two distinct fields rather than a rename. The
  `flatten` composition risk is closed: KTD3 records the spike that settled it.

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
- **R4.** Values are passed through as received. Normalization renames keys and never repairs, synthesizes, or drops a
  value. The empty array X sends stays empty.
- **R5.** A normalization that fires emits a `tracing` event carrying schema identifiers only: the legacy key, its
  current spelling, and the JSON type of the value, plus a length for a string, array, or object. Never a value, never
  an id, never a path containing one, never anything about the app, the user, or the credential.
- **R6.** Raw mode is untouched. `xr /2/tweets/<id>` prints exactly what X sent, because a caller asking for the raw
  body is asking for the raw body.
- **R7.** No API call is required to adopt or maintain any of this.
- **R8.** This ships as a patch. No public type changes, no signature changes, nothing `cargo semver-checks` reports at
  `--release-type patch`.

### Success Criteria

- A fixture spelling any of the 12 either way produces identical typed output.
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

Confirm each pair is a rename and not two distinct fields before trusting it. `post_id` and `tweet_id` both existing in
the spec is the case that needs an eye; if they are distinct, exclude the pair and say so in a comment.

**KTD2. Alias what is declared, normalize what is not.** Three one-line aliases close `posts`, `referenced_posts`, and
`repost_count`. Normalization covers the other eight and every future rename without adding public API. Both are driven
by the same table, so the two mechanisms cannot disagree.

**KTD3. Normalize inside typed deserialization, not at the transport.** Rewriting the response body before parsing would
be one chokepoint and would also corrupt raw mode, where the caller explicitly asked for what X sent (R6). Placing it on
the `extra` field keeps the two paths honest.

Prefer `#[serde(flatten, deserialize_with = "...")]` on each `extra` field, which keeps the public type
`BTreeMap<String, Value>` and breaks nothing. If `flatten` and `deserialize_with` do not compose, the fallback is a
newtype with its own `Deserialize`, which changes `extra`'s public type and needs a declared break and a version bump
per R8. Verify which applies before writing the rest.

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

| Field        | Example                  | Why it is safe                                                     |
| ------------ | ------------------------ | ------------------------------------------------------------------ |
| `legacy`     | `edit_history_tweet_ids` | A name from X's public spec                                        |
| `normalized` | `edit_history_post_ids`  | A name from X's public spec                                        |
| `value_type` | `array`                  | JSON type name; carries no content                                 |
| `value_len`  | `1`                      | Length only, for string, array, or object; catches the `[""]` case |

Never the value. A value can hold post text, a username, a DM, or an id. Never a request path, which carries ids
(`/2/tweets/2101712260468977783`). Never the app name, the authenticated user, the credential, or the scheme. If context
beyond the key pair is ever wanted, use the struct name, which contains no instance data.

*Rate.* Deduplicate per legacy key per process, so a consistently-legacy endpoint reports once rather than per call.

**KTD5. The live write gate is optional, not required.** Probing catches drift that does not follow the
`post`-to-`tweet` pattern, which is real but speculative. KTD4 covers the same ground continuously and for free, and a
non-empty `extra` surfaces the rest whenever anyone looks. If it is added later, it costs one post and one delete per
run and belongs behind the existing `XURL_LIVE_SMOKE=1` opt-in.

### Sequencing

U1 first; everything reads its table. U2 and U3 are independent of each other. U4 depends on U3. U5 closes the loop.

## Implementation Units

### U1. Derive the name table

**Goal.** R1, R7, KTD1.

**Files.** `crates/xdk/build.rs` or a shared module; `crates/xdk/vendor/x-api-openapi.json` as input, unchanged.

**Approach.** Emit the table where both the aliases' tests and the normalizer can read it. The build script is the
natural home, beside the existing spec codegen, because the normalizer is production code and not test-only.

**Test scenarios.** The table contains all four known pairs. A spec fixture with an added post-named property extends it
with no source edit. A pair that is not a rename is excluded and the exclusion is commented.

### U2. Alias the three declared fields

**Goal.** R2.

**Files.** `crates/xdk/src/api/response/types.rs`.

**Approach.** `#[serde(alias = "...")]` on `posts`, `referenced_posts`, and `repost_count`, matching the shape
`post_count` already uses. A test asserts every declared field whose name appears in the table carries the alias the
table names, so a future declared field cannot ship without one.

**Test scenarios.** A response spelling `retweet_count` fills `repost_count`. A response spelling `referenced_tweets`
fills `referenced_posts`. Both spellings of each produce identical output. Observed failing first.

### U3. Normalize undeclared keys

**Goal.** R3, R4, R6, R8, KTD3.

**Files.** `crates/xdk/src/api/response/types.rs`, plus wherever the deserializer helper lives.

**Approach.** A helper deserializing the flattened remainder and renaming any key the table lists. Keys not in the table
pass through. Values are never inspected.

**Test scenarios.** `edit_history_tweet_ids` deserializes to `edit_history_post_ids` with its value byte-identical,
empty array included. An unrelated unknown key survives unchanged. A raw request still prints the original spelling.
Both `extra` type options were tried and the chosen one is recorded.

### U4. Report a firing

**Goal.** R5, KTD4.

**Files.** the normalizer, `crates/xdk/src/api/` for the new target constant,
`crates/xurl-cli/src/cli/output/diagnostics.rs` for the render arm.

**Approach.** Declare a target beside `WIRE_TARGET` and `MEDIA_TARGET`, emit at `debug` carrying the four fields KTD4
tabulates, and add a `render` arm gated by `verbose_enabled()`. Deduplicate per legacy key per process.

**Test scenarios.** A planted legacy key produces exactly one event carrying the key pair, the JSON type, and the
length, and no other field. Repeated calls with the same key produce one event. The event is absent from stdout in every
structured format and absent from stderr under `--quiet` and without `--verbose`, asserted by `scripts/lint-stdio.sh`
and the cross-mode table. A grep guard asserts the emit site passes no value, no path, and no identifier.

### U5. Record what the wire does

**Goal.** Keep instance four from being rediscovered.

**Files.** `docs/solutions/`, `RELEASES-PREFLIGHT.md`, the release notes.

**Approach.** One entry: the spec is mid-migration, the wire lags per-endpoint, a snapshot of the wire is worth less
than accepting both spellings, and the detector is telemetry rather than a probe. Name the three instances. Note the
unconfirmed lead that writes answer in legacy vocabulary because they carry no field-selection parameter, with #118 as
its counterexample. Update `RELEASES-PREFLIGHT.md:242-249` to describe what the live smoke now does and does not cover.
Commit with `sd-commit-doc`.

## Verification Contract

| Gate             | Command                                                                 | Done signal                            |
| ---------------- | ----------------------------------------------------------------------- | -------------------------------------- |
| Format           | `cargo fmt -- --check`                                                  | No diff                                |
| Lint             | `cargo clippy --all-targets -- -D warnings`                             | Clean                                  |
| Tests            | `cargo test`                                                            | All pass, both-spelling cases included |
| Golden           | `cargo test --test golden_tests`                                        | Only create-post fixtures changed      |
| Schema freshness | `cargo test --test schema_tests`                                        | Drift test passes                      |
| Public API       | `cargo semver-checks --baseline-rev xdk-rs-v0.1.0 --release-type minor` | Passes, or the break is declared       |
| Raw passthrough  | A raw request against a legacy-spelling fixture                         | Original spelling preserved            |
| Markdown         | `markdownlint-cli2 RELEASES-PREFLIGHT.md`                               | Zero issues                            |

## Definition of Done

- All 12 pairs are covered: four by alias, eight by normalization, and a thirteenth injected into a spec fixture is
  covered with no source edit.
- `xr post --output json` reports `edit_history_post_ids` carrying the empty array X sent, unrepaired.
- A raw request still prints X's own spelling.
- `cargo semver-checks --release-type patch` passes, and the release is cut as a patch.
- The telemetry emit site carries only the four fields KTD4 names, asserted by a guard rather than by review.
- The `docs/solutions/` entry is written and pushed with `sd-commit-doc`.
- The PR body's `## Changelog (xdk-rs)` names the normalization with a before and after snippet, which
  `crates/xdk/README.md` makes a release gate.
