---
date: 2026-06-04
topic: auth-method-enforcement
---

## Summary

Add client-side enforcement of which X API auth method each endpoint accepts. Derive the mapping at compile time from
X's published OpenAPI spec so the matrix tracks upstream rather than a hand-maintained table. Reject mismatched
invocations before the HTTP round-trip with a typed envelope agents can pattern-match.

## Problem Frame

During v1.3.0 release-preflight smoke testing, the driver ran `xr media upload <file> --app bird-prod --auth app`
against an app holding only a bearer token. X returned 403 `unsupported-authentication` with the prose "When
authenticating requests to the Twitter API v2 endpoints, you must use keys and tokens from a Twitter developer App that
is attached to a Project." The driver retried with `--auth oauth1` and the upload succeeded. The first attempt burned a
network round-trip and rate-limit credit on an invocation xurl-rs had enough local information to refuse.

Today the CLI accepts any combination of `--auth <method>` and endpoint without consulting any catalog. The 403 from X
is the user's first signal of mismatch. For agents (the primary audience per the agent-native spec), the failure
surfaces as `reason: network-error` with the raw X body embedded in `message:`. Agents must string-match the upstream
error rather than read a typed `reason` value, which forfeits the whole point of structured envelopes.

X publishes `https://api.x.com/2/openapi.json` (current upstream: 790 KB, 139 paths, spec version 2.165). The spec
defines three security schemes (`BearerToken`, `OAuth2UserToken` with per-scope entries like `media.write`, `UserToken`
for OAuth1) and a `security:` entry on each operation listing which schemes that method/path combination accepts. `POST
/2/media/upload` declares `security: [{OAuth2UserToken: ["media.write"]}, {UserToken: []}]`. Bearer is explicitly
absent. The xurl-rs repo does not currently vendor the spec; `tests/fixtures/openapi/example_responses.json` is
response-shape fixtures, not security mappings.

The fix is to vendor the spec, generate a method-plus-template lookup at build time, and validate at the start of
`ApiClient::send_request` before the request body is constructed.

## Key Decisions

**KTD-1. Source of truth: vendor X's published OpenAPI spec.** Three approaches were weighed. (A) hardcoded constants on
each shortcut function in `src/api/shortcuts.rs`. (B) a centralized hand-written table in `src/api/auth_matrix.rs`. (C)
derive at compile time from the vendored OpenAPI spec. (C) wins because the spec already exists upstream, X publishes
updates as the canonical contract, and the refresh pattern is a single command. (A) and (B) duplicate state that drifts
off the upstream contract. The cost of (C) is build-time codegen complexity; the cost of (A) or (B) is per-release human
auditing against an external doc that may already be stale.

**KTD-2. Validation fires before HTTP.** The check sits at the start of `ApiClient::send_request` (and the multipart and
streaming siblings), before URL rendering and before the request body is built. The shortcut hands the validator the
spec-named template directly (see KTD-8); no rendering or reverse-matching happens on the validation path. Fail-fast
lets the user see the structured envelope without burning a network round-trip.

**KTD-3. Unknown endpoints are permissive.** Raw mode (`xr <URL>`) and endpoints not present in the matrix get a
permissive default. The request goes out and X arbitrates. This protects against spec drift (X adds a new endpoint
before the next vendored-spec refresh) and against typos in path templates that would otherwise become hard 4xx-mapping
failures. An optional `--strict` flag for raw mode is a possible later addition. Not in v1.

**KTD-4. OAuth2 scope checks are a follow-up, not v1.** The OpenAPI spec encodes per-scope requirements
(`OAuth2UserToken: ["media.write"]`). v1 of the feature checks scheme-level only (`OAuth2UserToken` vs `UserToken` vs
`BearerToken`). A later iteration can extend the validator to inspect the stored OAuth2 token's scope set against the
endpoint's required scopes and produce a more specific `auth-scope-mismatch` envelope. Keeping v1 scheme-level avoids
coupling three concerns at once: token introspection, scope refresh on token rotation, and the user-visible recovery
flow when a token lacks a required scope.

**KTD-5. The typed envelope is the recovery signal.** Rather than rejecting opaquely, the envelope carries `endpoint`,
`method`, `requested`, and `supported` so agents can pick a different `--auth` flag and retry without parsing prose.

**KTD-6. Auto-detect becomes endpoint-aware, OAuth2-first.** `ApiClient::get_auth_header`'s autodetect path today walks
OAuth2, OAuth1, then Bearer in fixed order regardless of endpoint. The new design intersects "credentials the active app
holds" with "schemes the endpoint accepts" and picks OAuth2 first, then OAuth1, then Bearer. OAuth2-first is a
forward-compatibility bet: X's strategic direction is OAuth2 with per-scope security, and every xurl-rs write shortcut
documented in the current spec accepts both OAuth2 and OAuth1, so the preference rarely changes the outcome when both
are stored. The known cost is that v1 cannot detect scope mismatches locally (see KTD-4); under OAuth2-first, a token
present in the app but missing a required scope will still surface as X's 403 through the existing `network-error`
envelope until v2 of the feature lands scope checking. When the intersection is empty, auto-detect returns
`auth-method-mismatch` listing what the app holds versus what the endpoint accepts.

**KTD-7. Vendored spec is checked into git with a CI drift-check.** The vendored `vendor/x-api-openapi.json` is the
build's input contract: checked-in gives reproducible builds (Homebrew bottle CI, offline builds), an auditable
supply-chain (the spec is greppable from source), and CI that doesn't need to reach api.x.com on every push. Manual
refresh discipline is hardened by a CI drift-check that fetches upstream and flags divergence, pulling the
deferred-for-later "drift detection" item into v1.

**KTD-8. `RequestOptions` gains a typed `target: RequestTarget` enum.** Shortcuts today set `req.endpoint =
format!("/2/users/{user_id}/likes")`, which stores the rendered path and discards the template. The validator has
nothing to look up against without reconstructing the template. The refactor replaces `endpoint: String` with `target:
RequestTarget` where `RequestTarget = Template { path: String, params: HashMap<String, String> } | RawUrl(String)`.
Shortcuts set the `Template` variant with the spec's exact path string (e.g. `/2/users/{id}/likes`, not xurl-rs's local
`{user_id}`); the URL builder substitutes `params` at request time. The validator looks the matrix up by `Template.path`
directly — no rendering, no reverse-matching, no routing-precedence logic. The cost is a breaking change to public
`RequestOptions`, so the release that ships this is v2.0.0 rather than v1.4.0. The alternative (reverse-matching a
rendered path against parameterized templates) was rejected: the current X spec has 19 real literal-vs-parameterized
path collisions, including `/2/media/upload` vs `/2/media/{media_key}` — the exact endpoint this feature is built to
validate — and any reverse-matcher would need to encode OpenAPI's literal-beats-param precedence correctly or silently
misroute on the bug-triggering call.

**KTD-9. Pre-release coverage check fails the build if any shortcut targets a path the spec doesn't document.** A
build-time check (assertion or unit test against the generated matrix) enumerates every shortcut's spec template and
verifies each one resolves in the matrix. Catches three failure modes early: a shortcut typo, spec drift (X removed an
endpoint that xurl-rs still calls), and shortcuts targeting undocumented endpoints. R19 (permissive default for unknown
endpoints) covers raw mode and matrix lookups that miss at runtime; the coverage check is the build-time complement that
says "every shortcut we ship must be in the catalog we ship."

## Requirements

**Mapping source-of-truth**

- **R1.** The repo vendors X's published OpenAPI spec at `vendor/x-api-openapi.json`. A refresh script at
  `scripts/refresh-x-openapi.sh` contains the one-line curl recipe (`curl -sS -o vendor/x-api-openapi.json
  "https://api.x.com/2/openapi.json"`).
- **R2.** The OpenAPI spec is the only source for endpoint to supported-auth-methods mapping. No parallel hand-written
  table exists in `src/`.
- **R3.** The vendored spec ships with a `vendor/README.md` recording its provenance, the refresh date, the upstream
  URL, and the refresh command.

**Validation behavior**

- **R4.** `ApiClient::send_request`, `send_multipart_request`, and the streaming request path each call the matrix
  validator before constructing the HTTP request body. Mismatch yields `XurlError::AuthMethodMismatch { endpoint,
  method, requested, supported }` returned through the existing `?` propagation chain.
- **R5.** The validator reads the path template from `RequestOptions.target` directly. For `RequestTarget::Template {
  path, .. }`, the matrix lookup uses `path` (e.g. `/2/users/{id}/likes`) as the exact string key. For
  `RequestTarget::RawUrl`, validation is skipped (see R18). No rendering, reverse-matching, or path-routing logic exists
  on the validation path.
- **R6.** The validator uses HTTP method plus the `Template.path` string as the matrix lookup key (direct hash lookup,
  no precedence resolution). GET and POST on the same path are distinct entries in OpenAPI and may have different
  security lists.

**Auto-detect**

- **R7.** When `RequestOptions.auth_type` is empty (no `--auth` flag), auto-detect intersects the set of schemes the
  endpoint accepts with the set of credentials the active app holds. It picks the first match in this preference order:
  OAuth2 then OAuth1 then Bearer. The preference order is bound by rationale in `src/api/auth.rs` documentation: OAuth2
  is X's strategic direction with per-scope security guarantees, OAuth1 is the legacy path retained for compatibility
  with endpoints not yet OAuth2-covered, and Bearer is app-only and covers the fewest write endpoints. v1 does not
  inspect OAuth2 token scopes locally (KTD-4), so an OAuth2 token missing a required scope still surfaces X's 403 via
  the existing `network-error` envelope until v2 lands scope checking.
- **R8.** When the intersection is empty, the user sees `XurlError::AuthMethodMismatch` populated with the active app's
  available methods and the endpoint's accepted methods.
- **R9.** The prior behavior (walk OAuth2, OAuth1, Bearer in fixed order regardless of endpoint, ignoring whether the
  endpoint accepts the chosen scheme) is removed in this release. The preference order looks similar but the semantics
  differ: the new walk is intersected with the endpoint's accepted schemes first, so a Bearer-only-supporting endpoint
  no longer attempts OAuth2 before falling through. No compatibility flag is added.

**Envelope shape**

- **R10.** The `auth-method-mismatch` envelope renders under `--output json` as:

  ```json
  {"status":"error","reason":"auth-method-mismatch","exit_code":2,
   "endpoint":"/2/media/upload","method":"POST",
   "requested":"app","supported":["oauth1","oauth2"],
   "message":"Bearer (app) auth is not accepted at POST /2/media/upload. Use --auth oauth1 or --auth oauth2."}
  ```

- **R11.** The `reason` value `auth-method-mismatch` is a stable closed-set string. Add it to the closed set returned by
  `XurlError::kind()`.
- **R12.** Text mode emits the same `message:` content as a plain `Error: ...` line.
- **R13.** The `exit_code` is `2`. This matches `EX_USAGE` for user-fixable invocation errors and stays distinct from
  `EXIT_AUTH_REQUIRED`, which signals a missing credential rather than a wrong one.

**Codegen and refresh**

- **R14.** A `build.rs` step parses `vendor/x-api-openapi.json` at compile time and emits
  `OUT_DIR/auth_matrix_generated.rs`. The generated module exposes `pub fn supported_auth(method: &str, path_template:
  &str) -> &'static [AuthScheme]` backed by a static perfect-hash or `phf` map keyed on `(method, path_template)`. A
  thin runtime wrapper in `src/api/auth_matrix.rs` `include!()`s the generated file.
- **R15.** `build.rs` is incremental. It declares `cargo:rerun-if-changed=vendor/x-api-openapi.json`. Touching the spec
  triggers rebuild; touching other files does not.
- **R16.** Generated code is not checked into git. `OUT_DIR` is excluded by Cargo convention. A unit test asserts the
  codegen produces a non-empty matrix containing at least one known entry (for example `POST /2/media/upload`).
- **R17.** `scripts/refresh-x-openapi.sh` is invoked manually before each release cycle. CI drift detection (R24)
  shortens the time between an upstream change and a contributor noticing.

**Raw mode opt-out**

- **R18.** `xr <URL>` raw mode skips matrix validation by default. The user accepted the contract by passing a raw URL.
  In `RequestOptions` terms, raw mode sets `target: RequestTarget::RawUrl(...)` and the validator no-ops on that
  variant.
- **R19.** Unknown endpoints (not present in the matrix) follow the same permissive path. The request goes out and X
  arbitrates. This handles spec drift and templating mistakes that would otherwise produce false negatives.

**Request plumbing refactor**

- **R20.** `RequestOptions.endpoint: String` is replaced by `RequestOptions.target: RequestTarget`, where `RequestTarget
  = Template { path: String, params: HashMap<String, String> } | RawUrl(String)`. `Template.path` is the spec's exact
  path string (e.g. `/2/users/{id}/likes`). `Template.params` holds substitutions keyed on the spec's parameter names.
  `RawUrl` carries the full URL passed through from `xr <URL>` mode.
- **R21.** Every shortcut function in `src/api/shortcuts.rs` is rewritten to set `target: RequestTarget::Template {
  path, params }` where `path` uses the spec's parameter names (not xurl-rs's local variable names). Local variable
  names remain free to differ from the spec parameter name; the conversion happens at the `Template` construction site.
- **R22.** The URL builder owns substitution. `ApiClient` constructs the URL inside `send_request` (and siblings) by
  walking `Template.path`, substituting each `{name}` segment with `params[name]`, percent-encoding each substitution,
  and prefixing `api_base_url`. `RawUrl` bypasses substitution and uses the URL as given. The current `build_url(&str)`
  helper's "starts-with-`http`" heuristic is removed; the variant carries the distinction explicitly.
- **R23.** This is a breaking change to the public `RequestOptions` API. The release that ships this is v2.0.0.
  `CHANGELOG.md` and the v2.0.0 release notes explicitly list `RequestOptions::endpoint -> RequestOptions::target` as
  the breaking surface for library consumers and document the migration. No deprecated alias or compatibility shim is
  added.

**Spec hygiene**

- **R24.** A CI drift-check fires on two triggers: a weekly scheduled run and every PR that modifies
  `vendor/x-api-openapi.json`, `build.rs`, or `src/api/auth_matrix.rs`. The job fetches
  `https://api.x.com/2/openapi.json`, byte-compares against the vendored file, and posts a structured warning (PR
  comment or job summary) when divergent. The job is non-blocking on the weekly run (warn only) and informational on
  PRs. The intent is shortening time-to-notice on upstream drift, not gating contributions.
- **R25.** A build-time coverage check fails the build when any shortcut's spec template is absent from the generated
  matrix. The check enumerates a manually-maintained list of `(method, path_template)` tuples mirroring every shortcut
  in `src/api/shortcuts.rs` and asserts each resolves in the matrix. Failure indicates a shortcut typo, spec drift
  removing a previously-documented endpoint, or a shortcut targeting an undocumented endpoint (R26). A unit test in
  `tests/auth_matrix_coverage.rs` is the implementation locus.
- **R26.** Shortcuts whose target is not in the OpenAPI spec must be explicitly documented. The current `unblock_user`
  shortcut (calling `/2/users/{source_user_id}/blocking/{target_user_id}`) is the only known instance; that path is not
  present in the current spec (which only documents `/2/users/{id}/blocking` for the GET-list). Planning resolves this
  by one of: (a) removing the shortcut, (b) marking it explicitly raw-mode-only, or (c) adding it to an in-repo
  exception list consumed by R25's coverage check. Option (c) requires a documented rationale and an upstream link.

## Key Flows

- **F1. Validation rejection before HTTP.** A user runs a shortcut command with `--auth <method>` that the endpoint does
  not accept. The shortcut hands `ApiClient` a `RequestTarget::Template { path, .. }`; `ApiClient` looks up
  `supported_auth(method, path)`, finds the requested scheme absent from the returned list, and returns
  `XurlError::AuthMethodMismatch` without rendering the URL or sending the request. The process exits 2 with the typed
  envelope. No network round-trip occurs.
- **F2. Auto-detect with multi-app multi-method.** A user runs a shortcut without `--auth` against an app that has
  multiple stored auth methods. Auto-detect intersects supported schemes with stored credentials, picks the first
  preference-ordered match, constructs the Authorization header for that scheme, and sends the request. No prompting
  occurs.
- **F3. Auto-detect with no compatible credentials.** Same trigger as F2 but the intersection is empty. The validator
  returns `auth-method-mismatch` listing both sides of the empty intersection. The user reads what they need to add and
  which `xr auth ...` command runs against which app.
- **F4. Raw mode bypass and scope-mismatch surfacing for OAuth2.** A user runs `xr <URL>` against an X path with any
  `--auth` flag, or runs a shortcut whose OAuth2 token lacks a required scope. Raw mode skips the matrix lookup and the
  request goes out as constructed. For OAuth2 scope mismatches, v1 does not detect locally; X's 403 surfaces through the
  existing `network-error` envelope. v2 of the feature will surface a typed `auth-scope-mismatch` envelope at this
  point.

## Acceptance Examples

- **AE1.** Given a user has only bearer credentials on `bird-prod` and runs `xr media upload x.jpg --app bird-prod
  --auth app`. When the validator runs. Then the request is not sent, the exit code is 2, and the envelope is
  `auth-method-mismatch` with `supported: ["oauth1", "oauth2"]`.
- **AE2.** Given a user has OAuth1 credentials on `bird-dev` and runs `xr media upload x.jpg --app bird-dev --auth
  oauth1`. When the validator runs. Then validation passes and the request is sent.
- **AE3.** Given a user runs `xr media upload x.jpg --app bird-dev` with no `--auth` flag and `bird-dev` holds only
  OAuth1. When auto-detect resolves. Then OAuth1 is selected without prompting and the request is sent.
- **AE4.** Given a user runs `xr media upload x.jpg --app bird-prod` with no `--auth` flag and `bird-prod` holds only
  bearer. When auto-detect resolves. Then the intersection is empty and the user sees `auth-method-mismatch` with
  `requested: null`, `available_in_app: ["app"]`, and `supported: ["oauth1", "oauth2"]`.
- **AE5.** Given a user runs `xr /2/media/upload --auth app` in raw mode. When the request is dispatched. Then
  validation is skipped, the request is sent, and X's 403 surfaces unchanged from today's behavior.
- **AE6.** Given a user runs `xr media upload x.jpg --app bird-dev` with no `--auth` flag and `bird-dev` holds both
  OAuth1 and OAuth2 credentials. When auto-detect resolves. Then OAuth2 is selected per the preference order in R7 and
  the request is sent. If the OAuth2 token lacks `media.write` scope, X returns 403 and the user sees the existing
  `network-error` envelope (not `auth-method-mismatch`) — KTD-4 defers local scope detection to v2.
- **AE7.** Given xurl-rs ships a shortcut whose spec template is absent from the vendored matrix. When `cargo build` or
  `cargo test` runs. Then R25's coverage check fails with a message naming the offending shortcut and the missing
  template, before any release artifact is produced.

## Scope Boundaries

**Deferred for later:**

- OAuth2 scope checking. Validating the user's stored OAuth2 token's scope set against the endpoint's required scopes
  and producing a typed `auth-scope-mismatch` envelope. This is the v2 follow-up that makes AE6's failure surface a
  typed envelope rather than `network-error`.
- A `--strict` flag for raw mode that opts into validation even when the user passed a raw URL.
- A tier-aware variant of the validator that reflects X's access-tier gating (e.g. Enterprise-only endpoints). The
  OpenAPI `security:` field is access-tier-agnostic, so a free-tier app calling an Enterprise-only endpoint will pass
  the matrix and still 403 from X.

**Outside this product's identity:**

- Automatic OAuth2 scope re-acquisition when a scope mismatch is detected. This conflicts with the
  minimal-interactivity, explicit-action UX posture xurl-rs holds.
- Negotiating with X on behalf of the user, such as automatically falling back to a different auth method after a 403.
  Auto-detect picks one method when input is ambiguous; it does not retry across methods on failure.

## Dependencies / Assumptions

- X publishes a stable OpenAPI spec at `https://api.x.com/2/openapi.json` and the `security:` lists are reliable.
  Current upstream is spec version 2.165 with 139 paths and `security:` entries on every operation. The assumption is
  that X does not regress publishing cadence or schema format.
- The vendored spec is refreshed before each release cycle. Without refresh, the matrix accumulates false negatives as X
  adds endpoints. R24's CI drift-check shortens time-to-notice, but does not enforce refresh — the assumption remains
  human discipline at release-cut time.
- Shortcuts declare their spec template using the spec's parameter names. xurl-rs's local variable names (e.g.
  `user_id`, `post_id`, `source_user_id`) frequently differ from the spec's (`id`, `tweet_id`, `source_tweet_id`). The
  shortcut author is responsible for getting the spec-side name right at the `Template { path }` construction site;
  R25's coverage check fails the build when an author guesses wrong.
- The `unblock_user` shortcut targets `/2/users/{source_user_id}/blocking/{target_user_id}`, which is absent from the
  current OpenAPI spec. R26 captures the resolution path; planning makes the call.

## Outstanding Questions

**Resolve before planning:**

- `unblock_user` resolution under R26: remove the shortcut, redirect it to raw mode with a documented note, or carve a
  documented exception. Picking one before planning avoids a stop-the-world surprise when R25's coverage check first
  runs.

**Deferred to planning:**

- The exact `build.rs` parsing approach. Options: `serde_json` against a hand-written struct, the `openapiv3` crate, or
  a hand-rolled walk. Performance matters because `build.rs` runs on cold clean builds and the spec is around 790 KB.
- Whether to prune the generated matrix to only endpoints xurl-rs shortcuts target (smaller binary, faster lookup) or
  include every X endpoint (broader raw-mode coverage). Recommend prune; raw mode is permissive anyway so the
  unknown-endpoint path catches everything outside the prune.
- How `AuthScheme` is represented in the generated code. Options: an enum `Bearer | OAuth1User | OAuth2User { scope:
  Option<&'static str> }` or a simpler string tag. Type-safety versus codegen complexity trade-off.
- R24's CI drift-check failure surface: PR comment vs job summary vs both, and whether the weekly run opens an issue on
  divergence or just logs.
- Migration messaging for the v2.0.0 break (R23). The CHANGELOG entry shape and whether the release notes carry a small
  `RequestOptions::endpoint -> target` migration snippet for library consumers.

## Sources / Research

- `https://api.x.com/2/openapi.json`. Live upstream OpenAPI spec; the canonical source. Fetched 2026-06-04 for this
  brainstorm pass: 790 KB, 139 paths, spec version 2.165, three security schemes (`BearerToken`, `OAuth2UserToken`,
  `UserToken`).
- Spec coverage check against the current `src/api/shortcuts.rs` shortcut set. Every xurl-rs write shortcut accepts both
  `OAuth2UserToken` and `UserToken` per the spec's `security:` lists, so OAuth2-first auto-detect (R7) is
  well-supported. One outlier: `unblock_user` targets a path the spec doesn't document — captured in R26.
- Path-collision analysis on the current spec. 19 real literal-vs-parameterized collision pairs at the same depth,
  including `/2/media/upload` vs `/2/media/{media_key}` and five collisions in `/2/users/` alone (`/2/users/me`,
  `/2/users/by`, `/2/users/search`, `/2/users/personalized_trends`, `/2/users/reposts_of_me`, all colliding with
  `/2/users/{id}`). Locks in KTD-8's direct-template-lookup choice over reverse-matching.
- v1.3.0 PR #52 `fix(auth): multi-app credential routing across OAuth1, OAuth2, and bearer`. Fixed the bug class this
  feature builds on. Bug B in that PR (propagated auth errors) is what makes the new envelope visible to users; without
  it, validation rejections would be silently swallowed.
- v1.3.0 PR #51 `feat(auth): XURL_BEARER_TOKEN env fallback and --app client_id resolution fix`. Established the `--app
  NAME` plumbing this feature depends on.
- `RELEASES-PREFLIGHT.md`. The smoke gate that surfaced the gap; the bearer-on-media-upload mismatch was the first
  observation.
- April 2026 X API pricing reshuffle research. Moved `like`, `follow`, and `quote` to Enterprise-only access. The matrix
  will continue to list the scheme even for Enterprise-only endpoints because the OpenAPI `security:` field is
  access-tier-agnostic. A tier-aware variant of the validator is out of scope for v1.
