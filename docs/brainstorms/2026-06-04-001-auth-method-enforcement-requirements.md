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

X publishes `https://api.x.com/2/openapi.json` (around 720 KB). The spec defines three security schemes (`BearerToken`,
`OAuth2UserToken` with per-scope entries like `media.write`, `UserToken` for OAuth1) and roughly 151 `security:` entries
across endpoints listing which schemes each accepts. `POST /2/media/upload` declares `security: [{OAuth2UserToken:
["media.write"]}, {UserToken: []}]`. Bearer is explicitly absent. A sibling repo at
`~/dev/twarch/docs/x-api-openapi.json` already vendors this spec offline with a one-line refresh recipe. The xurl-rs
repo does not currently vendor it; `tests/fixtures/openapi/example_responses.json` is response-shape fixtures, not
security mappings.

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
streaming siblings), after URL templating, before the request body is built. Fail-fast lets the user see the structured
envelope without burning a network round-trip.

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

**KTD-6. Auto-detect becomes endpoint-aware.** `ApiClient::get_auth_header`'s autodetect path today walks OAuth2,
OAuth1, then Bearer in fixed order regardless of endpoint. The new design intersects "credentials the active app holds"
with "schemes the endpoint accepts" and picks the first match in a stated preference order. When the intersection is
empty, auto-detect returns `auth-method-mismatch` listing what the app holds versus what the endpoint accepts.

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
- **R5.** The validator resolves the endpoint to its OpenAPI path template (for example `/2/users/{id}/likes`), not the
  concrete URL with substitutions. Templating happens before validation so the matrix lookup is exact-match against
  template strings.
- **R6.** The validator uses HTTP method plus path template as the lookup key. GET and POST on the same path are
  distinct entries in OpenAPI and may have different security lists.

**Auto-detect**

- **R7.** When `RequestOptions.auth_type` is empty (no `--auth` flag), auto-detect intersects the set of schemes the
  endpoint accepts with the set of credentials the active app holds. It picks the first match in this preference order:
  OAuth1 then OAuth2 then Bearer. The preference order is bound by rationale in `src/api/auth.rs` documentation: OAuth1
  covers the broadest set of v2 endpoints in current X documentation, OAuth2 is the modern path with scope guarantees
  but more limited coverage, and Bearer is app-only and covers the fewest write endpoints.
- **R8.** When the intersection is empty, the user sees `XurlError::AuthMethodMismatch` populated with the active app's
  available methods and the endpoint's accepted methods.
- **R9.** The prior behavior (walk OAuth2, OAuth1, Bearer in fixed order regardless of endpoint) is removed in this
  release. No compatibility flag is added.

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
  &str) -> &'static [AuthScheme]`. A thin runtime wrapper in `src/api/auth_matrix.rs` `include!()`s the generated file.
- **R15.** `build.rs` is incremental. It declares `cargo:rerun-if-changed=vendor/x-api-openapi.json`. Touching the spec
  triggers rebuild; touching other files does not.
- **R16.** Generated code is not checked into git. `OUT_DIR` is excluded by Cargo convention. A unit test asserts the
  codegen produces a non-empty matrix containing at least one known entry (for example `POST /2/media/upload`).
- **R17.** `scripts/refresh-x-openapi.sh` is invoked manually before each release cycle. Automated drift detection is
  out of scope for v1.

**Raw mode opt-out**

- **R18.** `xr <URL>` raw mode skips matrix validation by default. The user accepted the contract by passing a raw URL.
- **R19.** Unknown endpoints (not present in the matrix) follow the same permissive path. The request goes out and X
  arbitrates. This handles spec drift and templating mistakes that would otherwise produce false negatives.

## Key Flows

- **F1. Validation rejection before HTTP.** A user runs a shortcut command with `--auth <method>` that the endpoint does
  not accept. `ApiClient` builds the path template, looks up `supported_auth(method, template)`, finds the requested
  scheme absent from the returned list, and returns `XurlError::AuthMethodMismatch` without building or sending the
  request. The process exits 2 with the typed envelope. No network round-trip occurs.
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

## Scope Boundaries

**Deferred for later:**

- OAuth2 scope checking. Validating the user's stored OAuth2 token's scope set against the endpoint's required scopes
  and producing a typed `auth-scope-mismatch` envelope.
- Drift detection. A CI job that compares `vendor/x-api-openapi.json` against the live upstream and warns when
  divergent.
- A `--strict` flag for raw mode that opts into validation even when the user passed a raw URL.

**Outside this product's identity:**

- Automatic OAuth2 scope re-acquisition when a scope mismatch is detected. This conflicts with the
  minimal-interactivity, explicit-action UX posture xurl-rs holds.
- Negotiating with X on behalf of the user, such as automatically falling back to a different auth method after a 403.
  Auto-detect picks one method when input is ambiguous; it does not retry across methods on failure.

## Dependencies / Assumptions

- X publishes a stable OpenAPI spec at `https://api.x.com/2/openapi.json` and the `security:` lists are reliable. The
  spec has been there for at least two years (the twarch sibling-repo vendoring is evidence) and the security entries
  are extensive (roughly 151 across endpoints). The assumption is that X does not regress the publishing or the schema
  format.
- The vendored spec is refreshed before each release cycle. Without refresh, the matrix accumulates false negatives as X
  adds endpoints. The refresh recipe is one curl; the assumption is human discipline. Drift detection (deferred above)
  would harden this.
- Path templating happens before validation. Each shortcut function in `src/api/shortcuts.rs` builds the path before
  calling into `ApiClient::send_request`. The validator can read the unrendered template from `RequestOptions` if the
  struct carries it, or recompute it from the rendered URL by reverse-templating. The latter is fragile and the planning
  decision should land on threading the template through `RequestOptions`.

## Outstanding Questions

**Resolve before planning:**

- Should `vendor/x-api-openapi.json` be checked into git, or fetched at build time? Checked-in gives reproducible builds
  and offline build. Fetched gives always-fresh. The brainstorm leans checked-in for v1.
- The auto-detect preference order. OAuth1 first or OAuth2 first when both are present on the active app? The trade-off:
  OAuth1 covers more endpoints in current X documentation; OAuth2 is the modern path with scope guarantees. R7 states
  OAuth1-first based on coverage; the planner should confirm by auditing which scheme covers more typical xurl-rs
  shortcuts.
- `RequestOptions` today carries the final URL (post-template), not the path template. The planner picks one of two
  paths: thread the template through to the validator, or extract the template from the URL by reverse-matching known
  templates. Threading is cleaner; reverse-matching avoids touching the option-passing surface.

**Deferred to planning:**

- The exact `build.rs` parsing approach. Options: `serde_json` against a hand-written struct, the `openapiv3` crate, or
  a hand-rolled walk. Performance matters because `build.rs` runs on cold clean builds and the spec is around 720 KB.
- Whether to prune the generated matrix to only endpoints xurl-rs shortcuts target (smaller binary, faster lookup) or
  include every X endpoint (broader raw-mode coverage). Recommend prune; raw mode is permissive anyway so the
  unknown-endpoint path catches everything outside the prune.
- How `AuthScheme` is represented in the generated code. Options: an enum `Bearer | OAuth1User | OAuth2User { scope:
  Option<&'static str> }` or a simpler string tag. Type-safety versus codegen complexity trade-off.

## Sources / Research

- `https://api.x.com/2/openapi.json`. Live upstream OpenAPI spec; the canonical source.
- `~/dev/twarch/docs/x-api-openapi.json` and `~/dev/twarch/docs/x-api-openapi-README.md`. Sibling-repo vendored copy
  with refresh recipe; establishes provenance and refresh pattern.
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
