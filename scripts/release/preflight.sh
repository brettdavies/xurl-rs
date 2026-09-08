#!/usr/bin/env bash
# Run release preflight gates against the current checkout.
#
# Usage:
#   scripts/release/preflight.sh <subcommand>
#
# Subcommands:
#   drift         Branch drift: what main carries that dev never received (delegated to drift.sh)
#   surface       Establish surface: commits + diff vs last tag, breaking markers
#   api-contract  xr help command surface diff, lib re-export diff vs last tag
#   smoke         Real-world live X API smoke (auto-seeds isolated $SMOKE_HOME from 1Password)
#   multi-app     Multi-app credential routing (reuses or seeds $SMOKE_HOME)
#   mechanics     Release mechanics sanity (version, lockfile, advisories, toolchain age, leak check,
#                 unguarded docs added to main, diff-B vs origin/dev)
#   all           Run drift, surface, api-contract, smoke, multi-app, mechanics (and surface-smoke if present)
#
# Post-tag verification (release.yml + homebrew dispatch + finalize-release) lives in
# scripts/release/postflight.sh, which runs AFTER the tag push, not before.
#
# Flags:
#   --smoke-home PATH   Reuse an existing seeded $SMOKE_HOME instead of creating + seeding
#   --no-cleanup        Keep $SMOKE_HOME after exit (default: shred on exit)
#   --tag TAG           Override LAST_TAG resolution (default: git tag --sort=-version:refname | head -n 1)
#
# Exit codes:
#   0 = all gates passed (or skipped with reason)
#   1 = one or more gates failed
#   2 = setup error (missing dep, unreachable secrets store, etc.)
#
# Dependencies:
#   - `xr` (built via cargo build --release --bin xr)
#   - `yq`, `jaq`, `gh`, `cargo`, `git` on PATH
#   - 1Password CLI service-account env (for smoke + multi-app)
#   - ~/.claude/skills/1password/scripts/ for vault reads (via _lib.sh's read_1p)
#
# The shared scaffolding (gate helpers, 1Password reads, shred cleanup, dispatch, drift,
# surface, mechanics) is the github-repo-setup skill's skeleton; the api-contract, smoke,
# and multi-app gates and the seed recipe are this project's.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
readonly REPO_ROOT

# Shared output helpers, gate counters, dependency checks, 1Password helper,
# SMOKE_HOME cleanup. Same _lib.sh as postflight.sh.
. "$(dirname "$0")/_lib.sh"

BIN_PATH="${BIN_PATH:-$REPO_ROOT/target/release/xr}"

require_built_binary() {
  [[ -x "$BIN_PATH" ]] || {
    echo "build xr first: cargo build --release --bin xr ($BIN_PATH not found)" >&2
    exit 2
  }
}

# SMOKE_HOME EXIT trap (uses cleanup_smoke from _lib.sh) ---------------------

trap cleanup_smoke EXIT

seed_smoke_store() {
  SMOKE_HOME="$(mktemp -d -t xr-preflight-XXXXXX)"

  local dev_cid dev_csec prod_cid prod_csec
  dev_cid=$(read_1p "X App - Bird (dev)" oauth2_client_id)
  dev_csec=$(read_1p "X App - Bird (dev)" oauth2_client_secret)
  prod_cid=$(read_1p "X App - Bird (prod)" oauth2_client_id)
  prod_csec=$(read_1p "X App - Bird (prod)" oauth2_client_secret)

  XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" auth apps add bird_dev --client-id "$dev_cid" --client-secret "$dev_csec" >/dev/null
  XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" auth apps add bird_prod --client-id "$prod_cid" --client-secret "$prod_csec" >/dev/null

  local dev_bearer prod_bearer dev_ck dev_cs dev_at dev_ts
  dev_bearer=$(read_1p "X App - Bird (dev)" credential)
  prod_bearer=$(read_1p "X App - Bird (prod)" credential)
  dev_ck=$(read_1p "X App - Bird (dev)" consumer_key)
  dev_cs=$(read_1p "X App - Bird (dev)" secret_key)
  dev_at=$(read_1p "X User Tokens - brettdavies" "OAuth1 (bird_dev app).X_API_USER_ACCESS_TOKEN")
  dev_ts=$(read_1p "X User Tokens - brettdavies" "OAuth1 (bird_dev app).X_API_USER_ACCESS_TOKEN_SECRET")

  XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" auth app --bearer-token "$dev_bearer" --app bird_dev >/dev/null
  XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" auth app --bearer-token "$prod_bearer" --app bird_prod >/dev/null
  XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" auth oauth1 \
    --consumer-key "$dev_ck" --consumer-secret "$dev_cs" \
    --access-token "$dev_at" --token-secret "$dev_ts" \
    --app bird_dev >/dev/null

  local dev_at2 dev_rt2 exp
  dev_at2=$(read_1p "X User Tokens - brettdavies" "OAuth2 (bird_dev app).X_API_OAUTH2_USER_ACCESS_TOKEN")
  dev_rt2=$(read_1p "X User Tokens - brettdavies" "OAuth2 (bird_dev app).X_API_OAUTH2_REFRESH_TOKEN")
  exp=$(date -d '+1 hour' +%s)

  DEV_AT="$dev_at2" DEV_RT="$dev_rt2" EXP="$exp" yq -i '
      .apps.bird_dev.oauth2_tokens.brettdavies = {
        "type": "oauth2",
        "oauth2": {
          "access_token": strenv(DEV_AT),
          "refresh_token": strenv(DEV_RT),
          "expiration_time": (strenv(EXP) | to_number)
        }
      } | .apps.bird_dev.default_user = "brettdavies"
    ' "$SMOKE_HOME/.xurl"
}

ensure_smoke_home() {
  [[ -n "$SMOKE_HOME" && -f "$SMOKE_HOME/.xurl" ]] && return 0
  echo "  seeding isolated SMOKE_HOME from 1Password..."
  seed_smoke_store
}

# Gate: surface --------------------------------------------------------------
#
# Generic: confirms what's actually changing since the last tag. Counts feed
# the human's gut-check on release scope and the breaking-marker tally drives
# the major-version decision.

gate_surface() {
  header "Establish surface"
  local last_tag commits files breaking
  last_tag="${LAST_TAG:-$(git tag --sort=-version:refname | head -n 1)}"
  [[ -n "$last_tag" ]] || {
    gate_skip "LAST_TAG" "no tags in repo yet (first release); surface is everything on the branch"
    return
  }
  commits=$(git log "$last_tag..HEAD" --oneline | wc -l)
  files=$(git diff "$last_tag..HEAD" --name-only | wc -l)
  # Scoped markers count too: `feat(api)!:` is breaking as much as `feat!:`.
  breaking=$(git log "$last_tag..HEAD" --grep '^[a-z]\+\(([^)]*)\)\?!:' --oneline | wc -l)
  gate_pass "LAST_TAG = $last_tag  ($commits commits, $files files, $breaking breaking)"
}

# Gate: api-contract ---------------------------------------------------------

gate_api_contract() {
  header "API contract surface"
  require_built_binary
  local last_tag tmpdir
  last_tag="${LAST_TAG:-$(git tag --sort=-version:refname | head -n 1)}"
  tmpdir=$(mktemp -d -t xr-api-XXXXXX)

  # Command surface diff
  if git worktree add --detach "$tmpdir/prev" "$last_tag" >/dev/null 2>&1; then
    if (cd "$tmpdir/prev" && cargo build --release --bin xr >/dev/null 2>&1); then
      local empty
      empty=$(mktemp -d)
      XURL_TOKEN_STORE="$empty/.xurl" "$tmpdir/prev/target/release/xr" --help 2>&1 | grep -E '^  [a-z]' | awk '{print $1}' | sort -u >"$tmpdir/prev-cmds.txt"
      XURL_TOKEN_STORE="$empty/.xurl" "$BIN_PATH" --help 2>&1 | grep -E '^  [a-z]' | awk '{print $1}' | sort -u >"$tmpdir/head-cmds.txt"
      local removed added
      removed=$(comm -23 "$tmpdir/prev-cmds.txt" "$tmpdir/head-cmds.txt" | tr '\n' ' ' | sed 's/ $//')
      added=$(comm -13 "$tmpdir/prev-cmds.txt" "$tmpdir/head-cmds.txt" | tr '\n' ' ' | sed 's/ $//')
      gate_pass "xr help: removed=[${removed:-none}]  added=[${added:-none}]  (confirm any removed has !: + Breaking row)"
      # `$empty` for `xr --help` is a no-creds tempdir, but xr may write a
      # default `~/.xurl` skeleton. Shred for consistency with the seeded
      # paths so cleanup policy is the same everywhere.
      shred_tmpdir "$empty"
    else
      gate_skip "xr help diff" "prev build failed"
    fi
    git worktree remove "$tmpdir/prev" --force >/dev/null 2>&1 || true
  else
    gate_skip "xr help diff" "could not check out $last_tag"
  fi

  # Library re-export diff
  local exports
  exports=$(git diff "$last_tag..HEAD" -- src/lib.rs src/api/mod.rs | grep -cE '^[+-]\s*pub\s+(use|fn|struct|enum|mod)' || true)
  gate_pass "lib re-export delta: $exports lines (review against MIGRATING.md breaking rows)"

  # $tmpdir held a git worktree of source, no creds; regular trash is fine.
  if command -v gio >/dev/null 2>&1; then
    gio trash "$tmpdir" 2>/dev/null || rm -rf "$tmpdir"
  else
    rm -rf "$tmpdir"
  fi
}

# Gate: smoke ----------------------------------------------------------------
#
# Project-authored: live X API checks driven against the isolated $SMOKE_HOME
# so the dev machine's real ~/.xurl is never touched.

gate_smoke() {
  header "Real-world smoke (live X API)"
  require_built_binary
  require_bin yq
  require_bin jaq
  ensure_smoke_home
  local out

  # OAuth1
  out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" whoami --auth oauth1 --app bird_dev --output json 2>&1 | jaq -r '.data.username // ""')
  if [[ -n "$out" ]]; then
    gate_pass "OAuth1 whoami (HMAC-SHA1) → $out"
  else
    gate_fail "OAuth1 whoami" "no username returned"
  fi

  # OAuth2 PKCE needs a human
  gate_skip "OAuth2 PKCE end-to-end" "human-driven; see RELEASES-PREFLIGHT.md § OAuth2 PKCE path"
  gate_skip "OAuth2 headless (--no-browser)" "same recipe as PKCE; same skip"

  # Bearer env one-shot
  local app_bearer empty
  app_bearer=$(read_1p "X App - Bird (dev)" credential)
  empty=$(mktemp -d)
  out=$(XURL_TOKEN_STORE="$empty/.xurl" XURL_BEARER_TOKEN="$app_bearer" "$BIN_PATH" search "rust" --max-results 1 --auth app --output json 2>&1 | jaq -c '.data | length > 0')
  unset app_bearer
  if [[ "$out" == "true" ]]; then
    gate_pass "Bearer env one-shot"
  else
    gate_fail "Bearer env" "no data"
  fi
  # $empty only saw XURL_BEARER_TOKEN as env, never written to disk, but
  # shred anyway in case xr created a sentinel file with the bearer.
  shred_tmpdir "$empty"

  # Bearer stored
  out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" search "rust" --max-results 1 --auth app --app bird_dev --output json 2>&1 | jaq -c '.data | length > 0')
  if [[ "$out" == "true" ]]; then
    gate_pass "Bearer stored (per-app)"
  else
    gate_fail "Bearer stored" "no data"
  fi

  # Typed wire vocabulary
  if out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" XURL_LIVE_SMOKE=1 XURL_LIVE_SMOKE_AUTH=app XURL_APP=bird_dev \
    cargo test --test live_smoke -- --ignored 2>&1); then
    gate_pass "Typed wire vocabulary (one post read + one user read deserialize into the 3.x types)"
  else
    gate_fail "Typed wire vocabulary" "$(printf '%s\n' "$out" | grep -m1 -E 'panicked|error' || printf '%s' "$out" | tail -n 3)"
  fi

  # Media upload
  out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" media upload tests/fixtures/media/smoke-test.jpg \
    --media-type image/jpeg --category tweet_image --wait \
    --auth oauth1 --app bird_dev --output json 2>&1 | jaq -r '.data.id // ""')
  if [[ -n "$out" ]]; then
    gate_pass "Media upload (chunked INIT/APPEND/FINALIZE) → media_id=$out"
  else
    gate_fail "Media upload" "no media_id"
  fi

  # Output formats: known v2.0.0 behavior on non-streaming endpoints
  gate_pass "Output formats (text/json/jsonl on non-streaming = pretty JSON; streaming requires elevated access; known behavior)"

  # Error envelopes: xr exits non-zero by design here; `|| true` keeps `set -e` happy
  local envelope
  envelope=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" whoami --auth app --app bird_dev --output json 2>&1 || true)
  envelope=$(printf '%s' "$envelope" | jaq -r '.reason // ""' 2>/dev/null || true)
  if [[ "$envelope" == "auth-method-mismatch" ]]; then
    gate_pass "auth-method-mismatch envelope (exit 2)"
  else
    gate_fail "auth-method-mismatch" "got reason='$envelope'"
  fi

  cp "$SMOKE_HOME/.xurl" "$SMOKE_HOME/.xurl.bak"
  yq -i 'del(.apps.bird_prod.oauth2_tokens)' "$SMOKE_HOME/.xurl"
  envelope=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" whoami --app bird_prod --output json 2>&1 || true)
  envelope=$(printf '%s' "$envelope" | jaq -r '.reason // ""' 2>/dev/null || true)
  if [[ "$envelope" == "auth-method-mismatch" ]]; then
    gate_pass "empty-intersection envelope (exit 2)"
  else
    gate_fail "empty-intersection" "got reason='$envelope'"
  fi
  mv "$SMOKE_HOME/.xurl.bak" "$SMOKE_HOME/.xurl"

  yq -i '.default_app = "default"' "$SMOKE_HOME/.xurl"
  envelope=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" search "x" --max-results 1 --output json 2>&1 || true)
  envelope=$(printf '%s' "$envelope" | jaq -r '.other_apps_with_creds // [] | length' 2>/dev/null || echo 0)
  yq -i '.default_app = "bird_dev"' "$SMOKE_HOME/.xurl"
  if [[ "$envelope" -gt 0 ]] 2>/dev/null; then
    gate_pass "wrong-app envelope ($envelope other_apps_with_creds)"
  else
    gate_fail "wrong-app" "no other_apps_with_creds in envelope"
  fi

  gate_skip "429 (rate limited)" "hard to trigger reliably without burning quota"
}

# Gate: multi-app ------------------------------------------------------------

gate_multi_app() {
  header "Multi-app credential routing"
  require_built_binary
  require_bin yq
  require_bin jaq
  ensure_smoke_home
  local out

  # OAuth1 isolation
  out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" whoami --app bird_dev --auth oauth1 --output json 2>&1 | jaq -r '.data.username // ""')
  if [[ -n "$out" ]]; then
    gate_pass "OAuth1 routes to bird_dev → $out"
  else
    gate_fail "OAuth1 bird_dev" "no username"
  fi

  out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" whoami --app bird_prod --auth oauth1 --output json 2>&1 || true)
  out=$(printf '%s' "$out" | jaq -r '.reason // ""' 2>/dev/null || true)
  if [[ "$out" == "auth-required" ]]; then
    gate_pass "OAuth1 isolated (bird_prod has none → auth-required)"
  else
    gate_fail "OAuth1 isolation" "expected auth-required, got '$out'"
  fi

  # Bearer isolation
  out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" search "rust" --max-results 1 --auth app --app bird_dev --output json 2>&1 | jaq -c '.data | length > 0')
  if [[ "$out" == "true" ]]; then
    gate_pass "Bearer routes to bird_dev"
  else
    gate_fail "Bearer bird_dev" "no data"
  fi
  out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" search "rust" --max-results 1 --auth app --app bird_prod --output json 2>&1 | jaq -c '.data | length > 0')
  if [[ "$out" == "true" ]]; then
    gate_pass "Bearer routes to bird_prod"
  else
    gate_fail "Bearer bird_prod" "no data"
  fi

  # OAuth2 sideload structural isolation
  out=$(yq '.apps | with_entries(select(.value.oauth2_tokens != null)) | keys | length' "$SMOKE_HOME/.xurl")
  if [[ "$out" -ge 1 ]]; then
    gate_pass "OAuth2 per-app oauth2_tokens slot present (sideload-verifiable; PKCE end-to-end is human-driven)"
  else
    gate_fail "OAuth2 isolation" "no oauth2_tokens slot found"
  fi

  # Auto-detect with --app NAME
  cp "$SMOKE_HOME/.xurl" "$SMOKE_HOME/.xurl.bak"
  yq -i 'del(.apps.bird_dev.oauth2_tokens) | del(.apps.bird_dev.default_user)' "$SMOKE_HOME/.xurl"
  out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" whoami --app bird_dev --output json 2>&1 | jaq -r '.data.username // ""')
  mv "$SMOKE_HOME/.xurl.bak" "$SMOKE_HOME/.xurl"
  if [[ -n "$out" ]]; then
    gate_pass "Auto-detect falls through OAuth2→OAuth1 when OAuth2 absent → $out"
  else
    gate_fail "Auto-detect fallthrough" "no username"
  fi

  # First-signed-in auto-default
  local fresh dev_ck dev_cs dev_at dev_ts dev_cid dev_csec prod_cid prod_csec
  fresh=$(mktemp -d)
  dev_cid=$(read_1p "X App - Bird (dev)" oauth2_client_id)
  dev_csec=$(read_1p "X App - Bird (dev)" oauth2_client_secret)
  prod_cid=$(read_1p "X App - Bird (prod)" oauth2_client_id)
  prod_csec=$(read_1p "X App - Bird (prod)" oauth2_client_secret)
  dev_ck=$(read_1p "X App - Bird (dev)" consumer_key)
  dev_cs=$(read_1p "X App - Bird (dev)" secret_key)
  dev_at=$(read_1p "X User Tokens - brettdavies" "OAuth1 (bird_dev app).X_API_USER_ACCESS_TOKEN")
  dev_ts=$(read_1p "X User Tokens - brettdavies" "OAuth1 (bird_dev app).X_API_USER_ACCESS_TOKEN_SECRET")

  XURL_TOKEN_STORE="$fresh/.xurl" "$BIN_PATH" auth apps add bird_dev --client-id "$dev_cid" --client-secret "$dev_csec" >/dev/null
  XURL_TOKEN_STORE="$fresh/.xurl" "$BIN_PATH" auth apps add bird_prod --client-id "$prod_cid" --client-secret "$prod_csec" >/dev/null
  local before after
  before=$(yq '.default_app' "$fresh/.xurl")
  XURL_TOKEN_STORE="$fresh/.xurl" "$BIN_PATH" auth oauth1 --consumer-key "$dev_ck" --consumer-secret "$dev_cs" --access-token "$dev_at" --token-secret "$dev_ts" --app bird_dev >/dev/null
  after=$(yq '.default_app' "$fresh/.xurl")
  if [[ "$before" == "default" && "$after" == "bird_dev" ]]; then
    gate_pass "First-signed-in auto-default ($before → $after)"
  else
    gate_fail "Auto-default" "$before → $after"
  fi

  # Promotion idempotence
  XURL_TOKEN_STORE="$fresh/.xurl" "$BIN_PATH" auth oauth1 --consumer-key "$dev_ck" --consumer-secret "$dev_cs" --access-token "$dev_at" --token-secret "$dev_ts" --app bird_prod >/dev/null
  after=$(yq '.default_app' "$fresh/.xurl")
  if [[ "$after" == "bird_dev" ]]; then
    gate_pass "Promotion idempotence (second sign-in did not overwrite default)"
  else
    gate_fail "Idempotence" "default became $after"
  fi
  unset dev_ck dev_cs dev_at dev_ts dev_cid dev_csec prod_cid prod_csec
  # $fresh/.xurl was seeded with bird_dev OAuth1 creds: shred, not trash.
  shred_tmpdir "$fresh"

  # Auth-error envelope vs upstream 401
  out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" whoami --app bird_prod --auth oauth1 --output json 2>&1 || true)
  out=$(printf '%s' "$out" | jaq -r '.message // ""' 2>/dev/null || true)
  if [[ "$out" == *"TokenNotFound"* ]]; then
    gate_pass "Auth-error envelope surfaces TokenNotFound (not raw upstream 401)"
  else
    gate_fail "Auth-error envelope" "got '$out'"
  fi
}

# Gate: drift (delegated to drift.sh) ----------------------------------------
#
# Security PRs, hotfixes, and config edits land on main first. The release
# branch is cut from main and then takes dev's changes, so anything main holds
# that dev never received is reverted by the release or collides with it.
# drift.sh lists that set (commits since the last release whose changes dev
# lacks, .github/ parity, and lockfile packages main resolves newer) and
# fails while any exist. Run it before cutting the release branch. Repos
# without a dev branch skip it.

gate_drift() {
  local drift_script
  drift_script="$(dirname "$0")/drift.sh"
  [[ -x "$drift_script" ]] || return 0
  header "Branch drift (delegated to drift.sh)"
  if ! git rev-parse --verify --quiet origin/dev >/dev/null 2>&1; then
    gate_skip "drift" "no origin/dev branch (single-branch repo)"
    return
  fi
  delegate_to_subscript "$drift_script"
}

# Gate: surface-smoke (optional delegation) ----------------------------------
#
# If the project ships an HTTP / MCP / gRPC surface that needs a callable
# smoke suite (transport + tools + auth), put it in scripts/release/surface-
# smoke.sh and the `all` runner picks it up automatically. The sub-script
# must accept --result-file PATH per the contract in _lib.sh's
# delegate_to_subscript helper. xr ships no such surface, so the gate is a
# no-op until that script exists.

gate_surface_smoke() {
  local surface_script
  surface_script="$(dirname "$0")/surface-smoke.sh"
  [[ -x "$surface_script" ]] || return 0
  header "Surface smoke (delegated to surface-smoke.sh)"
  local local_url="${LOCAL_URL:-http://localhost:8787}"
  if ! curl -fsS --max-time 2 "$local_url/" >/dev/null 2>&1; then
    gate_skip "surface-smoke" "local server not running at $local_url (start it and re-run)"
    return
  fi
  delegate_to_subscript "$surface_script" "$local_url"
}

# Gate: mechanics ------------------------------------------------------------
#
# Mostly generic; the Rust-specific lines (Cargo.toml, rust-toolchain.toml,
# cargo deny) self-gate on file presence.

gate_mechanics() {
  header "Release mechanics sanity"
  local project_version changelog_version

  if [[ -f Cargo.toml ]]; then
    project_version=$(grep -m1 '^version = ' Cargo.toml | sed -E 's/^version = "(.*)"/\1/')
    gate_pass "Cargo.toml version = $project_version"
    if [[ -f Cargo.lock ]]; then
      gate_pass "Cargo.lock present"
    else
      gate_fail "Cargo.lock" "missing"
    fi
  elif [[ -f package.json ]]; then
    project_version=$(jaq -r .version package.json)
    gate_pass "package.json version = $project_version"
  elif [[ -f pyproject.toml ]]; then
    project_version=$(grep -m1 '^version = ' pyproject.toml | sed -E 's/^version = "(.*)"/\1/')
    gate_pass "pyproject.toml version = $project_version"
  elif [[ -f VERSION ]]; then
    project_version=$(<VERSION)
    gate_pass "VERSION = $project_version"
  else
    gate_skip "project version" "no Cargo.toml / package.json / pyproject.toml / VERSION found"
    project_version=""
  fi

  if [[ -x "$BIN_PATH" && -n "$project_version" ]]; then
    local bin_version
    bin_version=$("$BIN_PATH" --version 2>/dev/null | awk '{print $NF}')
    if [[ "$bin_version" == "$project_version" ]]; then
      gate_pass "$BIN_PATH --version = $bin_version (matches project version)"
    else
      gate_fail "$BIN_PATH --version mismatch" "binary=$bin_version project=$project_version"
    fi
  else
    gate_skip "binary --version" "build the release binary first ($BIN_PATH)"
  fi

  if [[ -f CHANGELOG.md ]]; then
    changelog_version=$(grep -m1 -oE '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' CHANGELOG.md | tr -d '[]## ')
    if [[ -n "$project_version" ]]; then
      if [[ "$changelog_version" == "$project_version" ]]; then
        gate_pass "CHANGELOG top section = [$changelog_version] (matches project version)"
      else
        gate_fail "CHANGELOG mismatch" "changelog=$changelog_version project=$project_version"
      fi
    fi
    if grep -q '\[Unreleased\]' CHANGELOG.md; then
      gate_fail "CHANGELOG" "has [Unreleased] placeholder"
    else
      gate_pass "CHANGELOG has no [Unreleased] placeholder"
    fi
  fi

  # Rust: toolchain quarantine.
  if [[ -f rust-toolchain.toml ]]; then
    local toolchain_channel release_date_match
    toolchain_channel=$(grep -m1 'channel = ' rust-toolchain.toml | sed -E 's/.*"([^"]+)".*/\1/')
    release_date_match=$(grep -m1 'released' rust-toolchain.toml | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' || true)
    if [[ -n "$release_date_match" ]]; then
      local age_days
      age_days=$((($(date +%s) - $(date -d "$release_date_match" +%s)) / 86400))
      if [[ $age_days -ge 7 ]]; then
        gate_pass "rust-toolchain channel=$toolchain_channel (released $release_date_match, $age_days days ago; 7-day quarantine satisfied)"
      else
        gate_fail "rust-toolchain quarantine" "channel $toolchain_channel released $release_date_match ($age_days days ago) is inside 7-day window"
      fi
    else
      gate_skip "rust-toolchain quarantine" "no 'released YYYY-MM-DD' comment found in rust-toolchain.toml"
    fi
  fi

  # Rust: cargo deny check advisories.
  if command -v cargo >/dev/null 2>&1 && [[ -f deny.toml ]]; then
    if cargo deny check advisories >/dev/null 2>&1; then
      gate_pass "cargo deny check advisories"
    else
      gate_fail "cargo deny check advisories" "see cargo deny check advisories"
    fi
  fi

  # Generic: the three screens below match against the guarded set the
  # workflow enforces, resolved by guarded-paths.sh, so this copy cannot drift
  # from what guard-main-docs rejects. A copy that omits a guarded path
  # reports a real leak as clean.
  local guarded ship_base
  if ! guarded=$("$(dirname "$0")/guarded-paths.sh" 2>/dev/null); then
    gate_fail "guarded-path list" "scripts/release/guarded-paths.sh resolved no pattern"
    return
  fi
  ship_base="${LAST_TAG:-origin/main}"
  git rev-parse --verify --quiet origin/main >/dev/null 2>&1 && ship_base=origin/main

  # Leak check: no guarded path in what the release adds to main.
  local leaked
  leaked=$(git diff "$ship_base..HEAD" --name-only 2>/dev/null | grep -E "$guarded" || true)
  if [[ -z "$leaked" ]]; then
    gate_pass "leak check (guarded paths): clean"
  else
    gate_fail "leak check" "guarded paths in diff vs $ship_base: $(echo "$leaked" | tr '\n' ' ')"
  fi

  # The leak check screens against the registered set, so it is blind to a
  # category nobody registered yet. Enumerate what the release adds to main
  # (anything under docs/, plus markdown anywhere, so a root-level glossary
  # shows up) and put every unguarded doc in front of a human.
  local added_docs
  added_docs=$(git diff "$ship_base..HEAD" --diff-filter=A --name-only 2>/dev/null | grep -E '(^docs/|\.md$)' | grep -Ev "$guarded" || true)
  if [[ -z "$added_docs" ]]; then
    gate_pass "no unguarded docs newly added to main"
  else
    gate_skip "unguarded docs added to main (confirm each is meant to ship)" "$(echo "$added_docs" | tr '\n' ' ')"
  fi

  # diff-B: files on dev that this branch lacks. Excluding all of docs/ would
  # hide a missed pick under docs/migrating, which ships to main, so exclude
  # only the guarded set. Version files and the regenerated changelog are
  # release-only by design.
  if git rev-parse --verify --quiet origin/dev >/dev/null 2>&1; then
    local missed
    missed=$(git diff HEAD..origin/dev --name-only 2>/dev/null | grep -Ev "$guarded" | grep -Ev '^(Cargo\.toml|Cargo\.lock|package\.json|package-lock\.json|pyproject\.toml|uv\.lock|VERSION|CHANGELOG\.md)$' || true)
    if [[ -z "$missed" ]]; then
      gate_pass "diff-B: no missed picks vs origin/dev"
    else
      gate_skip "diff-B: files on dev but not on this branch (review)" "$(echo "$missed" | head -5 | tr '\n' ' ')"
    fi
  else
    gate_skip "diff-B" "no origin/dev branch"
  fi
}

# Main dispatcher ------------------------------------------------------------

usage() {
  sed -n '2,38p' "$0" | sed 's/^# \?//'
  exit 2
}

LAST_TAG=""
SUBCMD=""

while [[ $# -gt 0 ]]; do
  # NO_CLEANUP is read by _lib.sh's EXIT-trap cleanup, not within this file.
  # shellcheck disable=SC2034
  case "$1" in
    --smoke-home)
      SMOKE_HOME="$2"
      shift 2
      ;;
    --no-cleanup)
      NO_CLEANUP=1
      shift
      ;;
    --tag)
      LAST_TAG="$2"
      shift 2
      ;;
    -h | --help) usage ;;
    drift | surface | api-contract | smoke | multi-app | mechanics | surface-smoke | all)
      SUBCMD="$1"
      shift
      ;;
    post-tag)
      echo "post-tag moved to scripts/release/postflight.sh; run that after the tag push" >&2
      exit 2
      ;;
    *)
      echo "unknown arg: $1" >&2
      usage
      ;;
  esac
done

[[ -n "$SUBCMD" ]] || usage

case "$SUBCMD" in
  drift) gate_drift ;;
  surface) gate_surface ;;
  api-contract) gate_api_contract ;;
  smoke) gate_smoke ;;
  multi-app) gate_multi_app ;;
  mechanics) gate_mechanics ;;
  surface-smoke) gate_surface_smoke ;;
  all)
    gate_drift
    gate_surface
    gate_api_contract
    gate_smoke
    gate_multi_app
    gate_surface_smoke
    gate_mechanics
    ;;
esac

print_summary

[[ $FAIL_COUNT -eq 0 ]] || exit 1
