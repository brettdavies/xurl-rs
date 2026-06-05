# Pre-release verification: `xurl-rs`

Operational pre-flight checklist. Runs **before** step 1 of
[`RELEASES.md` § Releasing dev to main](./RELEASES.md#releasing-dev-to-main). Gates the cut of the `release/v<version>`
branch, not the daily dev integration. Each box is an explicit go/no-go. If any item is unchecked or red, hold the
release.

CI (fmt, clippy, test, cargo-deny, Windows-compat, package-check) catches mechanical regressions inside this repo. This
checklist covers what CI structurally can't:

- Behavioral drift against the live X (Twitter) API. CI runs unit and integration tests against mocked HTTP, not the
  real endpoints, so API-contract regressions land silently until a user hits them.
- Distribution paths that only exercise on real artifacts (cross-compile binaries, `cargo install` from a clean machine,
  `cargo-binstall` against a published GitHub Release, `brew install` once the Homebrew dispatch finishes).
- Token-store correctness on a fresh machine (the YAML at `~/.xurl` is not exercised by the in-repo tests in the same
  way it is on first run).
- TLS stack correctness on Windows (the rustls + rustls-platform-verifier path differs from the dynamic-linker stack on
  Linux/macOS and CI's `ci / Windows check` only covers compile-time correctness).

## Establish the surface

Everything below assumes you know what's changing. Run this first.

```bash
LAST_TAG=$(git tag --sort=-version:refname | head -n 1)
git log "$LAST_TAG..dev" --oneline                              # commits going out
git diff "$LAST_TAG..dev" --stat                                # file-level scope
git diff "$LAST_TAG..dev" -- src/api/ src/auth/ src/cli/        # surface area: HTTP, auth, CLI
git log "$LAST_TAG..dev" --grep '^[a-z]\+!:' --oneline          # Conventional-Commits breaking markers
```

Every `!:` commit drives the major-version decision and gets a row in the release's `### Breaking changes` section.

## Quick start: run the automated gates

Most of this checklist now runs from one script. Build `xr` first, then:

```bash
cargo build --release --bin xr
scripts/release-preflight.sh all          # surface + api-contract + smoke + multi-app + mechanics
```

After `git push origin vX.Y.Z` triggers the release pipeline, run
[`scripts/release-postflight.sh all`](./RELEASES-POSTFLIGHT.md) to verify the downstream chain.

The script (`scripts/release-preflight.sh`) covers 28 of the 31 automatable pre-tag gates. It exits non-zero if any gate
fails; human-required gates (OAuth2 PKCE end-to-end, OAuth2 headless, 429 rate-limit) are skipped with a `⊝` and a
pointer to the recipe below. Sub-commands let you re-run one gate group in isolation:

| Sub-command    | What it runs                                                                                                               | Live API?                 |
| -------------- | -------------------------------------------------------------------------------------------------------------------------- | ------------------------- |
| `surface`      | LAST_TAG resolution, commit/file/breaking-marker counts                                                                    | no                        |
| `api-contract` | `xr help` command surface diff vs LAST_TAG, lib re-export delta                                                            | no (builds prev tag once) |
| `smoke`        | OAuth1 whoami, Bearer (env + stored), media upload, all three error envelopes                                              | yes                       |
| `multi-app`    | OAuth1/Bearer/OAuth2 isolation, auto-detect, first-signed-in default, idempotence, auth-error envelope                     | yes                       |
| `mechanics`    | Cargo.toml version, lockfile presence, `xr --version` match, CHANGELOG match, toolchain quarantine, advisories, leak check | no                        |
| `all`          | every above                                                                                                                | yes                       |

Flags:

- `--smoke-home PATH` — reuse an existing seeded store (skip the 1Password seed)
- `--no-cleanup` — keep the temp store after exit (useful for follow-up `xr` probes)
- `--tag TAG` — override LAST_TAG auto-detection

The script seeds an isolated `$SMOKE_HOME` from 1Password (`secrets-dev` vault) and **`shred -u`s** every tempdir that
held credentials on exit (overwrites bytes with three passes before unlinking; falls back to `dd if=/dev/urandom + rm`
if `shred` isn't on `PATH`; refuses to operate outside `/tmp` or `$HOME` as a path-typo guardrail). The detailed recipes
in the gate sections below still document what the script does and serve as the manual fallback when 1Password is
unavailable or you want to iterate on a single gate by hand.

## Checklist

### API-contract surface

xurl-rs is a thin client over the live X API. The contract that ships is the union of the 29 shortcut commands, the
generic `xr request` path, and the library re-exports in `src/lib.rs`.

- [ ] `xr help` lists the same shortcut commands as the previous release plus any net additions / removals. Diff
  `$LAST_TAG`'s `xr help` against `dev`'s and confirm every removed or renamed command has a `!:` commit and a `###
  Changed` (or `### Breaking changes`) bullet in the release changelog.
- [ ] `xr schema` (typed response introspection, added in v1.1.0) still emits a parseable JSON shape; downstream agents
  feature-detect from this. Diff the shape against `$LAST_TAG`'s output and surface any field rename / removal as a
  breaking row.
- [ ] Public library surface (`xurl_rs::*`): run `cargo public-api diff` if available, or `git diff "$LAST_TAG..dev" --
  src/lib.rs src/api/mod.rs` and confirm every removed / renamed export is captured as a breaking row. Library consumers
  feature-detect on type names.

### Real-world smoke (live X API)

The in-repo tests mock the HTTP layer. The four auth paths and the three output formats only exercise end-to-end on the
live API. Pick fresh targets each release.

**Isolated-store recipe (run all smokes without touching real `~/.xurl`):**

```bash
SMOKE_HOME=$(mktemp -d -t xr-smoke-XXXXXX)
alias xrs="HOME=$SMOKE_HOME ./target/release/xr"

# Seed from 1Password (vault: secrets-dev). Items used:
#   "X App - Bird (dev)"            -> oauth2_client_id, oauth2_client_secret, consumer_key, secret_key, credential (Bearer)
#   "X App - Bird (prod)"           -> same shape
#   "X User Tokens - brettdavies"   -> "OAuth1 (bird_dev app).X_API_USER_ACCESS_TOKEN" + _SECRET (single-app OAuth1 user)
#                                   -> "OAuth2 (bird_dev app).X_API_OAUTH2_USER_ACCESS_TOKEN" + REFRESH_TOKEN
#                                   -> "OAuth2 (bird_prod app).X_API_OAUTH2_USER_ACCESS_TOKEN" + REFRESH_TOKEN

# Register two apps (replace stub values with 1P reads via scripts/read_field.sh).
xrs auth apps add bird_dev  --client-id "$DEV_CID"  --client-secret "$DEV_CSEC"
xrs auth apps add bird_prod --client-id "$PROD_CID" --client-secret "$PROD_CSEC"

# Seed bearer + OAuth1 via CLI (these accept tokens as args).
xrs auth app    --bearer-token "$DEV_BEARER"  --app bird_dev
xrs auth app    --bearer-token "$PROD_BEARER" --app bird_prod
xrs auth oauth1 --consumer-key "$DEV_CK" --consumer-secret "$DEV_CS" \
                --access-token "$DEV_AT" --token-secret "$DEV_TS" --app bird_dev

# OAuth2 has no CLI sideload flag (PKCE-only). Inject via yq strenv() to avoid argv exposure:
DEV_AT=$(read_field "OAuth2 (bird_dev app).X_API_OAUTH2_USER_ACCESS_TOKEN") \
DEV_RT=$(read_field "OAuth2 (bird_dev app).X_API_OAUTH2_REFRESH_TOKEN") \
EXP=$(date -d '+1 hour' +%s) \
yq -i '.apps.bird_dev.oauth2_tokens.brettdavies = {
  "type":"oauth2",
  "oauth2":{"access_token":strenv(DEV_AT),"refresh_token":strenv(DEV_RT),"expiration_time":(strenv(EXP)|to_number)}
} | .apps.bird_dev.default_user = "brettdavies"' "$SMOKE_HOME/.xurl"
```

**Never `cat` the seeded `~/.xurl`** — it round-trips plaintext OAuth1/Bearer secrets through the transcript. Use `xr
auth status` (redacts) or `yq '... | path'` for shape probes only.

- [ ] **OAuth1 path** (automatable): `xrs whoami --auth oauth1 --app bird_dev --output json | jaq -c
  '{u:.data.username}'` → expect `{"u":"BrettDavies"}` (or whichever account is seeded). Confirms HMAC-SHA1 signing
  didn't regress.
- [ ] **OAuth2 PKCE path** (needs human ONLY for the browser approval): drive end-to-end with the recipe below. The
  human's only job is opening a URL and pasting the redirect URL back — the agent handles every CLI step.

  ```bash
  FRESH=$(mktemp -d -t xr-pkce-XXXXXX)
  HOME=$FRESH ./target/release/xr auth apps add bird_dev \
      --client-id "$(read_field 'X App - Bird (dev)' oauth2_client_id)" \
      --client-secret "$(read_field 'X App - Bird (dev)' oauth2_client_secret)"

  # Step 1: agent runs, prints the authorize URL to stdout (URL is public — client_id,
  # state, code_challenge, no secrets).
  HOME=$FRESH ./target/release/xr auth oauth2 --no-browser --step 1 --app bird_dev

  # Human action: open the printed URL in a browser logged in as the target account,
  # click Authorize, copy the full redirect URL from the address bar (browser will show
  # a connection-refused error — that is expected).

  # Step 2: pipe the redirect URL into the agent via stdin. The `?code=…` is one-shot
  # and consumed immediately by the exchange, so stdin redirection keeps it off argv.
  echo '<paste-redirect-url-here>' | HOME=$FRESH ./target/release/xr auth oauth2 \
      --no-browser --step 2 --auth-url - --app bird_dev

  # Verify: tokens issued + auto-default fired.
  HOME=$FRESH ./target/release/xr whoami --auth oauth2 --app bird_dev --output json \
    | jaq -c '{u:.data.username, exit:(.exit_code//0)}'   # expect {"u":"<handle>","exit":0}
  yq '.default_app' "$FRESH/.xurl"                         # expect "bird_dev"

  # Refresh-token rotation gate: force-expire the access token, retry, confirm a fresh
  # access_token + refresh_token + expiration_time get written.
  HANDLE=$(yq '.apps.bird_dev.oauth2_tokens | keys | .[0]' "$FRESH/.xurl")
  yq -i ".apps.bird_dev.oauth2_tokens[\"$HANDLE\"].oauth2.expiration_time = 1" "$FRESH/.xurl"
  HOME=$FRESH ./target/release/xr whoami --auth oauth2 --app bird_dev --output json \
    | jaq -c '{u:.data.username, exit:(.exit_code//0)}'   # expect same {"u":...,"exit":0}
  EXP_NOW=$(date +%s); NEW_EXP=$(yq ".apps.bird_dev.oauth2_tokens[\"$HANDLE\"].oauth2.expiration_time" "$FRESH/.xurl")
  echo "refresh wrote new expiration: $((NEW_EXP - EXP_NOW))s in the future"
  ```

  **Critical gotcha — case-sensitive username key:** step 2 stores the token under the actual handle returned by
  `/2/users/me` (`BrettDavies` with mixed case, NOT the lowercased `brettdavies` you might assume). Probe the real key
  via `yq '.apps.bird_dev.oauth2_tokens | keys | .[0]'` before any yq edit. If you write to the wrong-cased path, yq
  silently CREATES a stub entry while the real tokens stay under the correct key, and xr's auto-refresh then finds the
  stub (or the wrong entry first via `default_user`) and dies with `RefreshTokenError: no access_token in response` or
  falls back to a fresh PKCE attempt with `client_id=` (empty in the URL — the fallback misses the `--app NAME`
  threading). This looks like a v2.0.0 bug but is just the case-sensitivity tripwire.

- [ ] **OAuth2 headless** (`--no-browser`) path: identical to PKCE above on this machine — `--no-browser` auto-engages
  when stdout isn't a TTY (the headless auto-engage shipped in v1.3.0). The two-step ceremony is the same; passing
  `--no-browser` explicitly is the only difference. The recipe above already uses `--no-browser`, so it satisfies both
  gates in one run.
- [ ] **Bearer token (env var, one-shot)** (automatable): with an empty `$HOME`, run `HOME=$(mktemp -d)
  XURL_BEARER_TOKEN=$(read_field 'X App - Bird (dev)' credential) xr search "rust" --max-results 1 --auth app`. Confirms
  `Auth::get_bearer_token_header` honors the env var without a persisted store entry.
- [ ] **Bearer token (stored, two-step)** (automatable): after the seed recipe above, `xrs search "rust" --max-results 1
  --auth app --app bird_dev --output json | jaq -c '{has_data:(.data|length>0)}'` → expect `{"has_data":true}`.
- [ ] **Media upload** (automatable): `xrs media upload tests/fixtures/media/smoke-test.jpg --media-type image/jpeg
  --category tweet_image --wait --auth oauth1 --app bird_dev --output json | jaq -c '{media_id:.data.id}'`. **Gotcha:**
  defaults are `video/mp4` + `amplify_video`; for the JPG fixture you MUST pass `--media-type image/jpeg --category
  tweet_image` or the API returns `invalid-args`. Small images return no `processing_info` (set immediately) — `state`
  is `n/a`, presence of `media_id` is the success signal.
- [ ] **Output formats** (partially automatable): `--output text`, `--output json`, `--output jsonl` for one
  non-streaming endpoint (e.g. `xr search`). **Known v2.0.0 behavior:** for non-streaming endpoints, `text` and `jsonl`
  both produce the same pretty-printed JSON as `json`. The jsonl-per-line semantic is only meaningful on streaming
  endpoints (`/2/tweets/search/stream`, `/2/tweets/sample/stream`, `/2/tweets/firehose/*`), which require elevated X API
  access and aren't exercisable on a dev account.
- [ ] **Error paths** (automatable): three envelope shapes plus an upstream propagation. All produce structured JSON
  under `--output json`:
- **`auth-method-mismatch` (exit 2)**: `xrs whoami --auth app --app bird_dev` — Bearer rejected at `/2/users/me`.
  Envelope includes `endpoint`, `rendered_url`, `requested`, `supported`, `available_in_app`, `app`.
- **Empty-intersection mismatch (exit 2)**: temporarily `yq -i 'del(.apps.bird_prod.oauth2_tokens)' "$SMOKE_HOME/.xurl"`
  then `xrs whoami --app bird_prod` — only Bearer in store, `/2/users/me` doesn't accept it.
- **Wrong-app envelope (exit 2)**: `yq -i '.default_app = "default"' "$SMOKE_HOME/.xurl"` then `xrs search "x"
  --max-results 1` (auto-detect, no `--app`) — empty default app, others have creds; envelope includes
  `other_apps_with_creds`. Restore `default_app = "bird_dev"` after.
- **Upstream 401 propagation (exit 77)**: with stale OAuth2 token, `xrs whoami --auth oauth2 --app bird_dev` returns
  `{"reason":"auth-required","message":"... RefreshTokenError ..."}` — xr's mapping, not raw upstream JSON.

  **429 (rate limited)** is hard to trigger reliably without burning quota; skip unless a specific regression suspicion
  motivates it.

**v2.0.0 design observation worth noting (not a regression):** when an app has multiple stored methods and the preferred
one fails its refresh (e.g. expired OAuth2), auto-detect does NOT fall back to the next-preferred. Method selection is
at request-construction time; refresh failure is treated as an auth-error for the chosen method. `xrs whoami --app
bird_dev` with stale OAuth2 + valid OAuth1 returns `auth-required` rather than retrying with OAuth1. The intersection IS
rechecked when OAuth2 is absent from the store; the gap is specifically refresh-time failures.

### Multi-app credential routing

Auth methods exist on `Auth` as `--app NAME`-aware reads and writes. The legacy code paths used the store's no-arg
accessors, which fell back to the default app and silently bypassed NAME's credentials; the v1.3.0 multi-app credential
routing fix scoped every read and write to the active app. These gates verify the routing stays correct across OAuth1,
OAuth2, and bearer, and that the auto-default UX still fires on the first signed-in app. Each gate needs at least two
registered apps to exercise the cross-app path; the `bird_dev` + `bird_prod` entries in 1Password are the canonical
substitutes for `alpha` / `beta`.

All gates below use the isolated `$SMOKE_HOME` seed recipe from § Real-world smoke. **Never `cat` `$SMOKE_HOME/.xurl`**
— use `xr auth status` for human inspection or `yq 'keys | .[]' "$SMOKE_HOME/.xurl"` / `yq '.. | path' ...` for
structural probes.

- [ ] **OAuth2 `--app NAME` save and read isolation** (sideload-verifiable; PKCE end-to-end needs human): the seed
  recipe injects per-app `oauth2_tokens.brettdavies` for both `bird_dev` and `bird_prod`. Verify isolation by inspecting
  the structure without printing values: `yq '.apps | to_entries | map({app: .key, has_oauth2: (.value.oauth2_tokens !=
  null), users: (.value.oauth2_tokens // {} | keys)})' "$SMOKE_HOME/.xurl"`. Each app holds its own
  `oauth2_tokens.brettdavies` entry and neither overwrites the other. End-to-end PKCE verification (live token issued
  during this preflight) requires the human-driven authorize step described in § Real-world smoke.
- [ ] **OAuth1 `--app NAME` save and read isolation** (automatable): after the seed recipe puts OAuth1 only on
  `bird_dev`, run `xrs whoami --app bird_dev --auth oauth1 --output json | jaq -c '{u:.data.username}'` → expect
  `{"u":"BrettDavies"}`. Then `xrs whoami --app bird_prod --auth oauth1 --output json | jaq -c '{r:.reason,
  m:.message}'` → expect `{"r":"auth-required", "m":"Auth Error: TokenNotFound: OAuth1 token not found"}`. Confirms the
  resolver scopes to the active app rather than silently falling back.
- [ ] **Bearer `--app NAME` save and read isolation** (automatable): both apps have distinct bearers in 1Password. Run
  `xrs search "rust" --max-results 1 --auth app --app bird_dev --output json | jaq -c '{ok:(.data|length>0)}'` and the
  same with `--app bird_prod`. Both should return `{"ok":true}` against their respective dev portal apps. The proof of
  isolation is that the per-app `bearer_token` slot is distinct in the YAML (probe via `yq '.apps | to_entries |
  map({app:.key, has_bearer:(.value.bearer_token != null)})'`).
- [ ] **Auto-detect with `--app NAME`** (automatable): the seed recipe leaves `bird_dev` with OAuth1 + Bearer + stale
  OAuth2, `bird_prod` with Bearer + stale OAuth2. Test the OAuth2-preferred selection by stripping `bird_dev`'s OAuth2:
  `yq -i 'del(.apps.bird_dev.oauth2_tokens) | del(.apps.bird_dev.default_user)' "$SMOKE_HOME/.xurl"`, then `xrs whoami
  --app bird_dev --output json | jaq -c '{u:.data.username,e:(.exit_code//0)}'` → expect `{"u":"BrettDavies","e":0}`
  (auto-detect picked OAuth1 since OAuth2 is now absent). Restore from the backup before moving on.
- [ ] **First-signed-in-app auto-default** (automatable): use a *fresh* tempdir (not `$SMOKE_HOME`). `FRESH=$(mktemp
  -d); HOME=$FRESH xr auth apps add bird_dev --client-id … --client-secret …; HOME=$FRESH xr auth apps add bird_prod …`.
  Confirm `yq '.default_app' "$FRESH/.xurl"` is `"default"`. Run `HOME=$FRESH xr auth oauth1 --app bird_dev …`. Confirm
  `default_app` flipped to `"bird_dev"`. Repeat in a second fresh tempdir using `xr auth app --bearer-token …` and a
  third using the OAuth2 PKCE flow (the OAuth2 one needs the human authorize step from § Real-world smoke). All three
  sign-in handlers must promote.
- [ ] **Promotion idempotence** (automatable): continue from the previous test in the *same* `$FRESH`. `HOME=$FRESH xr
  auth oauth1 --app bird_prod …` (sign in on the OTHER app). Confirm `yq '.default_app'` is still `"bird_dev"`, not
  `"bird_prod"`. The auto-default fires once; a credentialed default is not overwritten.
- [ ] Auth-error envelope vs upstream 401: with no token stored for the requested auth path (e.g., `xr search "x" --auth
  oauth1` against an app that has no OAuth1 entry), the error envelope is `auth-required` with `message:` mentioning
  `TokenNotFound` rather than a wrapped X 401 body. Confirms `ApiClient::send_request` propagates auth errors instead of
  silently sending an unauthenticated request that surfaces as an upstream rejection.

### Distribution and install paths

The release builds cross-compiled binaries and the homebrew tap dispatches downstream. None of this runs in `cargo
test`.

- [ ] Last green run of `release.yml` (on this branch or a sibling) cross-compiled all five targets listed in
  `RELEASES.md` § Tagging and publishing. If the workflow has changed since, dry-run with `cargo build --release
  --target <target>` for each.
- [ ] In a clean container or fresh machine: download a prior release archive (`xurl-rs-<target>.tar.gz` or `.zip` for
  Windows), run `xr --version` and one read-only shortcut. Confirms the archive layout (binary + completions + licenses)
  still works without the project's toolchain.
- [ ] `cargo install xurl-rs --version <new>` from a clean environment resolves and runs once the crates.io publish
  completes (post-tag check, see below).
- [ ] Token-store roundtrip on a fresh machine: `xr auth` produces a `~/.xurl` YAML, subsequent `xr` invocations
  authenticate from it without re-prompting, and `xr auth status` correctly identifies multi-app entries.

### Release mechanics sanity

These items duplicate steps in `RELEASES.md` deliberately: easy to skip, expensive to recover from. Confirm explicitly.

- [ ] `Cargo.toml` `version` bumped to the new tag value (`check-version` in `release.yml` enforces this; catch early).
- [ ] `Cargo.lock` regenerated via `cargo update -p xurl-rs`, committed.
- [ ] Rebuild locally, confirm `xr --version` prints the new tag value.
- [ ] Every PR merged since `$LAST_TAG` has a non-empty `## Changelog` section. Spot-check via `gh pr list --base dev
  --state merged --search "merged:>$(git log -1 --format=%aI $LAST_TAG)"` then `gh pr view <num> --json body`.
- [ ] `rust-toolchain.toml` last bumped ≥7 days ago (supply-chain quarantine). If a bump landed inside the window, hold
  or revert it before tagging.
- [ ] No unmerged dependency advisories from `cargo deny check advisories`. The full local pre-push check
  (`scripts/hooks/pre-push`) mirrors CI; run it explicitly before pushing the release branch.
- [ ] Leak check: `git diff origin/main..HEAD --name-only | grep -E
  '^(docs/plans|docs/brainstorms|docs/ideation|docs/reviews|docs/solutions|\.context)'` returns nothing. If cherry-picks
  pulled in guarded paths via rename detection, resolve per `RELEASES.md` § Cherry-pick conflicts on guarded paths.
- [ ] `CHANGELOG.md` versioned section has no `[Unreleased]` placeholder and matches the bumped `Cargo.toml` version.

### Post-tag verification

Moved to [`RELEASES-POSTFLIGHT.md`](./RELEASES-POSTFLIGHT.md) — tagging happens **after** the release-branch cut and
PR-to-main merge, so verification of the tag-triggered pipeline (release.yml → homebrew-tap → finalize-release →
crates.io publish → fresh-machine install smokes) is post-flight, not pre-flight. Run `scripts/release-postflight.sh
all` immediately after `git push origin vX.Y.Z`.

## Related docs

- [`RELEASES-POSTFLIGHT.md`](./RELEASES-POSTFLIGHT.md): runs AFTER the tag push to verify the downstream pipeline.
- [`RELEASES.md`](./RELEASES.md): operational runbook this checklist gates.
- [`RELEASES-RATIONALE.md`](./RELEASES-RATIONALE.md): release-flow rationale.
- [`AGENTS.md`](./AGENTS.md): project structure, auth paths, output formats.
