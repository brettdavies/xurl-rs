#!/usr/bin/env bash
# Canonical version: 2026.11.03 (generic)
#
# Run release preflight gates against the current checkout.
#
# Usage:
#   scripts/release/preflight.sh <subcommand>
#
# Subcommands:
#   drift         Branch drift: what main carries that dev never received (delegated to drift.sh)
#   surface       Establish surface: commits + diff vs last tag, breaking markers
#   api-contract  xr help command surface diff, public API semver check vs last tag, stale semver overrides
#   smoke         Real-world live X API smoke (auto-seeds isolated $SMOKE_HOME from 1Password)
#   multi-app     Multi-app credential routing (reuses or seeds $SMOKE_HOME)
#   oauth2-interactive
#                 The release binary's interactive `xr auth oauth2`, offline: under a pty, with a
#                 fake browser and a stub for X (oauth2-interactive.py). Also runs the previous
#                 release's xr, when installed, and reports whether the two behave differently
#   changelog-sections
#                 No PR this release carries leaves its changelog entry to its title for want
#                 of a ## Changelog section (generate-changelog.py --audit-sections)
#   mechanics     Release mechanics sanity (version, lockfile, advisories, toolchain age, leak check,
#                 unguarded docs added to main, diff-B vs origin/dev)
#   all           Run drift, surface, api-contract, smoke, multi-app, oauth2-interactive,
#                 changelog-sections, mechanics (and surface-smoke if present)
#
# Post-tag verification (release.yml + homebrew dispatch + finalize-release) lives in
# scripts/release/postflight.sh, that runs AFTER the tag push, not before.
#
# Flags:
#   --smoke-home PATH   Reuse an existing seeded $SMOKE_HOME instead of creating + seeding
#   --no-cleanup        Keep $SMOKE_HOME after exit (default: shred on exit)
#   --tag TAG           Override LAST_TAG resolution (default: the newest `v[0-9]*` tag)
#   --staging-url URL   Run surface-smoke against this deployed URL (default: STAGING_URL_DEFAULT
#                       from release.env; with neither, the local server at $LOCAL_URL)
#
# Environment:
#   CHANGELOG_PR_BASE   Branch whose merged PRs changelog-sections reads (default: dev, or main
#                       when origin/dev does not exist)
#   PREVIOUS_BIN_PATH   The xr oauth2-interactive compares the release binary with (default:
#                       Homebrew's)
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
#   - Linux, `script` from util-linux, and `python3` for oauth2-interactive, which skips without them
#   - ~/.claude/skills/1password/scripts/ for vault reads (via _lib.sh's read_1p)
#
# The shared scaffolding (gate helpers, 1Password reads, shred cleanup, dispatch, drift,
# surface, changelog-sections, mechanics) is the github-repo-setup skill's skeleton; the api-contract, smoke,
# multi-app, and oauth2-interactive gates and the seed recipe are this project's.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
readonly REPO_ROOT

# Shared output helpers, gate counters, dependency checks, 1Password helper,
# SMOKE_HOME cleanup. Same _lib.sh as postflight.sh.
# shellcheck disable=SC1091  # sibling _lib.sh, always vendored alongside
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

  # Every secret reaches xr through a `-file` flag fed by a process
  # substitution, so no process's arguments carry it; `printf` is a builtin.
  XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" auth apps add bird_dev --client-id "$dev_cid" --client-secret-file <(printf '%s' "$dev_csec") >/dev/null
  XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" auth apps add bird_prod --client-id "$prod_cid" --client-secret-file <(printf '%s' "$prod_csec") >/dev/null

  local dev_bearer prod_bearer dev_ck dev_cs dev_at dev_ts
  dev_bearer=$(read_1p "X App - Bird (dev)" credential)
  prod_bearer=$(read_1p "X App - Bird (prod)" credential)
  dev_ck=$(read_1p "X App - Bird (dev)" consumer_key)
  dev_cs=$(read_1p "X App - Bird (dev)" secret_key)
  dev_at=$(read_1p "X User Tokens - brettdavies" "OAuth1 (bird_dev app).X_API_USER_ACCESS_TOKEN")
  dev_ts=$(read_1p "X User Tokens - brettdavies" "OAuth1 (bird_dev app).X_API_USER_ACCESS_TOKEN_SECRET")

  XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" auth app --bearer-token-file <(printf '%s' "$dev_bearer") --app bird_dev >/dev/null
  XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" auth app --bearer-token-file <(printf '%s' "$prod_bearer") --app bird_prod >/dev/null
  XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" auth oauth1 \
    --consumer-key "$dev_ck" --consumer-secret-file <(printf '%s' "$dev_cs") \
    --access-token-file <(printf '%s' "$dev_at") --token-secret-file <(printf '%s' "$dev_ts") \
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
# Generic: confirms what's actually changing since the last release on the
# binary's tag line. Counts feed the human's gut-check on release scope and the
# breaking-marker tally drives the major-version decision.
#
# The window is read on the integration branch (origin/dev, or HEAD in a repo
# without one). Every release squash-merges into main, so its tag shares no
# recent history with dev, and a log from the tag counts dev's whole past. When
# the tag is not an ancestor, the window opens at the commit that synced the tag
# back into dev instead. A PR merged after the previous cut and before that
# commit sits outside this window; generate-changelog.py reads those PRs too, so
# its section, not this count, is the complete list.

# The commit on REF whose subject syncs TAG back, the way generate-changelog.py
# finds it: `chore(release): sync dev after <tag>`, or `backport <tag>` from
# older backport scripts. Only the subject counts, because a later message can
# quote it. Empty when no commit names the tag. awk reads the whole log rather
# than exiting at the match: an early exit leaves git log writing into a closed
# pipe, and under pipefail the SIGPIPE ends the script on any long history.
release_backport_commit() {
  git log --format='%H %s' "$1" 2>/dev/null | awk -v tag="$2" '
    !found {
      subject = substr($0, index($0, " ") + 1)
      split("chore(release): sync dev after |chore(release): backport ", prefixes, "|")
      for (i = 1; i <= 2; i++) {
        lead = prefixes[i] tag
        rest = substr(subject, length(lead) + 1)
        if (index(subject, lead) == 1 && (rest == "" || substr(rest, 1, 1) == " ")) {
          print $1
          found = 1
          break
        }
      }
    }'
}

gate_surface() {
  header "Establish surface"
  local last_tag head_ref window anchor="" commits files breaking
  last_tag="${LAST_TAG:-$(last_release_tag)}"
  [[ -n "$last_tag" ]] || {
    gate_skip "LAST_TAG" "no tags in repo yet (first release); surface is everything on the branch"
    return
  }
  head_ref=origin/dev
  git rev-parse --verify --quiet "$head_ref" >/dev/null 2>&1 || head_ref=HEAD
  window="$last_tag..$head_ref"
  if ! git merge-base --is-ancestor "$last_tag" "$head_ref" 2>/dev/null; then
    anchor=$(release_backport_commit "$head_ref" "$last_tag")
    if [[ -z "$anchor" ]]; then
      gate_skip "LAST_TAG" \
        "$last_tag is no ancestor of $head_ref and no commit there syncs it back; read the surface as origin/main..$head_ref"
      return
    fi
    window="$anchor..$head_ref"
  fi
  # Files compare trees, so they read from the tag directly, minus the guarded
  # set: dev carries engineering docs that never ship.
  local guarded
  guarded=$("$(dirname "$0")/guarded-paths.sh" 2>/dev/null) || guarded='^$'
  # BSD wc pads its count to eight columns, so each count drops its whitespace
  # before it reaches the pass line.
  commits=$(git log "$window" --oneline | wc -l | tr -d '[:space:]')
  files=$(git diff "$last_tag" "$head_ref" --name-only | { grep -Ev "$guarded" || true; } | wc -l | tr -d '[:space:]')
  # Scoped markers count too: `feat(api)!:` is breaking as much as `feat!:`.
  breaking=$(git log "$window" --grep '^[a-z]\+\(([^)]*)\)\?!:' --oneline | wc -l | tr -d '[:space:]')
  gate_pass "LAST_TAG = $last_tag  ($commits commits, $files files, $breaking breaking${anchor:+, counted from its backport ${anchor:0:12} on $head_ref})"
}

# Gate: api-contract ---------------------------------------------------------

# A required-update entry in crates/xdk/Cargo.toml reclassifies one release's
# break. Once a tag moves the baseline past that break its lint stops firing,
# and the entry would reclassify the next break of the same kind unseen, so an
# entry whose lint the patch-level probe does not name fails here.
check_semver_overrides() {
  local lib_tag="$1" overridden probe fired stale
  overridden=$(cargo metadata --no-deps --format-version 1 --offline 2>/dev/null \
    | jaq -r '.packages[] | select(.name == "xdk-rs") | .metadata["cargo-semver-checks"].lints // {}
      | to_entries[] | select(.value["required-update"] != null) | .key' | sort -u)
  if [[ -z "$overridden" ]]; then
    gate_pass "semver overrides: crates/xdk/Cargo.toml reclassifies no break"
    return
  fi
  probe=$(cargo semver-checks check-release --package xdk-rs \
    --baseline-rev "$lib_tag" --release-type patch 2>&1) || true
  if ! grep -q '^ *Checked' <<<"$probe"; then
    gate_skip "semver overrides" "the patch-level probe did not compare xdk-rs with $lib_tag"
    return
  fi
  fired=$(sed -n 's/^--- failure \([a-z_]*\):.*/\1/p' <<<"$probe" | sort -u)
  stale=$(comm -23 <(printf '%s\n' "$overridden") <(printf '%s\n' "$fired") | paste -sd ' ')
  if [[ -n "$stale" ]]; then
    gate_fail "semver overrides whose lint no longer fires vs $lib_tag: $stale (delete them from crates/xdk/Cargo.toml)"
  else
    gate_pass "semver overrides: each still reclassifies a break vs $lib_tag"
  fi
}

gate_api_contract() {
  header "API contract surface"
  require_built_binary
  local last_tag tmpdir
  last_tag="${LAST_TAG:-$(last_release_tag)}"
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

  # Public library surface. A textual diff cannot answer this: it misses `pub`
  # fields, `pub const`, `pub type`, and `pub trait`. cargo-semver-checks
  # compares rustdoc JSON against the baseline tag and knows Rust's own semver
  # rules. The library is the only published API, and it baselines on its own
  # `xdk-rs-v*` tag line, as CI does; the CLI's `v*` tags would name the wrong
  # crate. Breaks deliberately taken into a minor are recorded in
  # crates/xdk/Cargo.toml under [package.metadata.cargo-semver-checks.lints].
  if command -v cargo-semver-checks >/dev/null 2>&1; then
    local lib_tag semver_out semver_rc
    lib_tag=$(git tag --list 'xdk-rs-v[0-9]*' --sort=-version:refname | head -n 1)
    if [[ -z "$lib_tag" ]]; then
      gate_skip "cargo-semver-checks" "no xdk-rs-v* tag yet; the first library release has no baseline"
    else
      semver_out=$(cargo semver-checks check-release --package xdk-rs \
        --baseline-rev "$lib_tag" --release-type minor 2>&1) && semver_rc=0 || semver_rc=$?
      if [[ $semver_rc -eq 0 ]]; then
        gate_pass "cargo-semver-checks: xdk-rs minor bump sufficient vs $lib_tag"
      elif grep -q '^--- failure' <<<"$semver_out"; then
        printf '%s\n' "$semver_out" | grep -E '^--- failure|^  (field|variant|struct|enum|fn|method) ' || true
        gate_fail "cargo-semver-checks: xdk-rs breaks vs $lib_tag (record the break in crates/xdk/Cargo.toml lints)"
      else
        # A break arrives as `--- failure` blocks. A non-zero exit without one
        # means the tool compared nothing, typically a rustdoc build that
        # failed, so the question is still open; CI's semver gate answers it.
        gate_skip "cargo-semver-checks" \
          "did not compare xdk-rs with $lib_tag (exit $semver_rc): $(grep -m1 '^error:' <<<"$semver_out" || tail -n 1 <<<"$semver_out"); CI's public-API semver gate decides on the PR"
      fi
      check_semver_overrides "$lib_tag"
    fi
  else
    gate_skip "cargo-semver-checks" "not installed (cargo binstall cargo-semver-checks)"
  fi

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

  # The oauth2-interactive gate drives the sign-in against a stub. A real browser
  # against X needs a person, so it stays an optional check.
  gate_skip "OAuth2 PKCE against X" "optional real-browser check; see RELEASES-PREFLIGHT.md § OAuth2 PKCE path"
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
    gate_pass "Typed wire vocabulary (one post read + one user read deserialize into the library's typed structs)"
  else
    gate_fail "Typed wire vocabulary" "$(printf '%s\n' "$out" | grep -m1 -E 'panicked|error' || printf '%s' "$out" | tail -n 3)"
  fi

  # Media upload
  out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$BIN_PATH" media upload crates/xurl-cli/tests/fixtures/media/smoke-test.jpg \
    --media-type image/jpeg --category tweet_image --wait \
    --auth oauth1 --app bird_dev --output json 2>&1 | jaq -r '.data.id // ""')
  if [[ -n "$out" ]]; then
    gate_pass "Media upload (chunked INIT/APPEND/FINALIZE) → media_id=$out"
  else
    gate_fail "Media upload" "no media_id"
  fi

  # Nothing here reads a stream: the streaming endpoints need elevated access.
  # The mocked suite holds the formats on a document (json indented, jsonl on
  # one line).
  gate_skip "Output formats on a stream" "streaming requires elevated access; the mocked suite covers the formats on a document"

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
    gate_pass "OAuth2 per-app oauth2_tokens slot present (sideload-verifiable; the oauth2-interactive gate drives a sign-in)"
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

  # First-registered app becomes the default.
  #
  # `add_app` promotes when the store holds no apps or the standing default
  # carries neither a client id nor a token, and registration never
  # materializes a `default` app (crates/xdk/src/store/mod.rs). So a fresh store names
  # its first registration as the default before any sign-in happens, and the
  # sign-in handler's job here is to leave that answer alone.
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

  local first second after
  XURL_TOKEN_STORE="$fresh/.xurl" "$BIN_PATH" auth apps add bird_dev --client-id "$dev_cid" --client-secret-file <(printf '%s' "$dev_csec") >/dev/null
  first=$(yq '.default_app' "$fresh/.xurl")
  XURL_TOKEN_STORE="$fresh/.xurl" "$BIN_PATH" auth apps add bird_prod --client-id "$prod_cid" --client-secret-file <(printf '%s' "$prod_csec") >/dev/null
  second=$(yq '.default_app' "$fresh/.xurl")
  XURL_TOKEN_STORE="$fresh/.xurl" "$BIN_PATH" auth oauth1 --consumer-key "$dev_ck" --consumer-secret-file <(printf '%s' "$dev_cs") --access-token-file <(printf '%s' "$dev_at") --token-secret-file <(printf '%s' "$dev_ts") --app bird_dev >/dev/null
  after=$(yq '.default_app' "$fresh/.xurl")
  if [[ "$first" == "bird_dev" && "$second" == "bird_dev" && "$after" == "bird_dev" ]]; then
    gate_pass "First-registered app is the default and sign-in leaves it (add→$first, add→$second, sign-in→$after)"
  else
    gate_fail "First-registered default" "add→$first, add→$second, sign-in→$after (each should be bird_dev)"
  fi

  # Promotion idempotence
  XURL_TOKEN_STORE="$fresh/.xurl" "$BIN_PATH" auth oauth1 --consumer-key "$dev_ck" --consumer-secret-file <(printf '%s' "$dev_cs") --access-token-file <(printf '%s' "$dev_at") --token-secret-file <(printf '%s' "$dev_ts") --app bird_prod >/dev/null
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

# Gate: oauth2-interactive ----------------------------------------------------
#
# Project-authored: the one gate that makes the shipped binary serve an OAuth2
# callback. smoke and multi-app seed their stores, and the headless flow never
# binds the listener. The harness gives every process it starts an environment
# it builds from nothing and a scratch store, so this gate needs no secret and
# cannot reach the machine's own store or X.

# Homebrew's `xr`: the previous release, on a machine that installs it. Empty
# when Homebrew or the formula is missing.
homebrew_xr() {
  have_bin brew || return 0
  local bin
  bin="$(brew --prefix xurl-rs 2>/dev/null || true)/bin/xr"
  if [[ -x "$bin" ]]; then
    printf '%s\n' "$bin"
  fi
}

oauth2_harness() {
  python3 -B -I "$REPO_ROOT/scripts/release/oauth2-interactive.py" "$1"
}

# The modes a harness run reported failed, space-separated.
failed_modes() {
  awk -F'\t' '$1 == "FAIL" { print $2 }' <<<"$1" | paste -sd ' ' -
}

# Whether the harness tells the release binary from the previous release. It
# informs and never fails the release: once the previous release carries every
# fix, the two are meant to agree, and an older binary failing proves only that
# the harness still discriminates.
compare_with_previous_release() {
  local failed="$1" previous results rc=0 previous_failed name
  previous="${PREVIOUS_BIN_PATH:-$(homebrew_xr)}"
  if [[ -z "$previous" ]]; then
    gate_skip "previous release comparison" "no Homebrew xr installed; set PREVIOUS_BIN_PATH to name one"
    return
  fi
  name=$("$previous" --version 2>/dev/null || printf '%s' "$previous")
  results=$(oauth2_harness "$previous") || rc=$?
  previous_failed=$(failed_modes "$results")
  if [[ $rc -ne 0 && -z "$previous_failed" ]]; then
    gate_skip "previous release comparison" "the harness could not run against $name (exit $rc)"
  elif [[ "$previous_failed" == "$failed" ]]; then
    gate_pass "previous release comparison: $name and this build fail the same modes [${failed:-none}], so the harness does not tell the two apart"
  else
    gate_pass "previous release comparison: $name fails [${previous_failed:-none}], this build [${failed:-none}], so the harness tells the two apart"
  fi
}

gate_oauth2_interactive() {
  header "OAuth2 interactive sign-in (offline)"
  local tool
  if [[ "$(uname -s)" != "Linux" ]]; then
    gate_skip "OAuth2 interactive sign-in" "needs Linux, for script(1) from util-linux and the fake xdg-open"
    return
  fi
  for tool in script python3; do
    if ! have_bin "$tool"; then
      gate_skip "OAuth2 interactive sign-in" "$tool not installed"
      return
    fi
  done
  require_built_binary

  local results rc=0 failed verdict mode detail
  results=$(oauth2_harness "$BIN_PATH") || rc=$?
  failed=$(failed_modes "$results")
  if [[ $rc -ne 0 && -z "$failed" ]]; then
    gate_fail "OAuth2 interactive harness" "could not run (exit $rc); its error is above"
    return
  fi
  while IFS=$'\t' read -r verdict mode detail; do
    if [[ "$verdict" == "PASS" ]]; then
      gate_pass "$mode: $detail"
    else
      gate_fail "$mode" "$detail"
    fi
  done <<<"$results"
  compare_with_previous_release "$failed"
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
  # Project picks the local URL the surface runs against; common defaults:
  #   bunx wrangler dev   → http://localhost:8787
  #   uvicorn / fastapi   → http://localhost:8000
  #   cargo run --release → http://localhost:3000
  # A deployed staging URL (--staging-url, else STAGING_URL_DEFAULT from
  # release.env, the same default postflight reads) is smoked as given: an
  # unreachable deployment is a failure, not a skip. Without one, the local
  # server is the target, and a local server that is not running skips.
  local url="${STAGING_URL:-${STAGING_URL_DEFAULT:-}}"
  if [[ -z "$url" ]]; then
    url="${LOCAL_URL:-http://localhost:8787}"
    if ! curl -fsS --max-time 2 "$url/" >/dev/null 2>&1; then
      gate_skip "surface-smoke" "local server not running at $url (start it and re-run)"
      return
    fi
  fi
  delegate_to_subscript "$surface_script" "$url"
}

# Gate: changelog-sections ---------------------------------------------------
#
# generate-changelog.py owns the rules for reading a PR body, so it runs the
# audit: `--audit-sections` names the PRs whose entry would fall back to their
# title because the body never offered the changelog section. It reads the PRs
# from the integration branch's history since the previous release on the tag
# line, as the changelog itself does, so stacked PRs are among them and a
# member's tag never moves the binary's window. A section left empty on purpose
# passes, because the generator reads it as "nothing for this crate".
#
# One audit per changelog the release writes: the release package's, and each
# workspace member whose release is pending (no tag yet at its version), each
# under the heading its [package.metadata.changelog] table declares.
gate_changelog_sections() {
  header "PR changelog sections"
  if [[ ! -x "$REPO_ROOT/scripts/generate-changelog.py" ]]; then
    gate_skip "PR changelog sections" "scripts/generate-changelog.py not vendored"
    return
  fi
  have_bin gh || {
    gate_skip "PR changelog sections" "gh not installed"
    return
  }

  local base
  base="${CHANGELOG_PR_BASE:-dev}"
  git -C "$REPO_ROOT" rev-parse --verify --quiet "origin/$base" >/dev/null 2>&1 || base="main"

  local targets=() name prefix version
  targets+=("$(cd "$REPO_ROOT" && changelog_crate_args)")
  while IFS=$'\t' read -r name _ prefix _ version; do
    [[ -n "$name" && -n "$version" ]] || continue
    git -C "$REPO_ROOT" rev-parse --verify --quiet "refs/tags/$prefix$version" >/dev/null && continue
    targets+=("--crate $name --tag $prefix$version")
  done < <(cd "$REPO_ROOT" && release_members)

  local target label out
  for target in "${targets[@]}"; do
    label="PR changelog sections"
    if [[ "$target" == --crate* ]]; then
      read -r _ name _ <<<"$target"
      label="$label ($name)"
    fi
    # shellcheck disable=SC2086  # target is a flag list, split on purpose
    if out=$(cd "$REPO_ROOT" && scripts/generate-changelog.py --audit-sections --dev-branch "$base" $target 2>&1); then
      gate_pass "$label: ${out%%$'\n'*}"
    else
      gate_fail "$label" "$out"
    fi
  done
}

# Gate: mechanics ------------------------------------------------------------
#
# Mostly generic; the Rust-specific lines (Cargo.toml, rust-toolchain.toml,
# cargo deny) self-gate on file presence.

# migration_guide_check CHANGELOG VERSION: a 1.0+ release that keeps its major
# and carries `### Breaking changes` takes the small-break exception of
# github-repo-setup's references/versioning.md, so it ships
# docs/migrating/v<VERSION>.md, whose first line names the exception. A 0.x
# release, a major, and a first release need no guide.
migration_guide_check() {
  local changelog=$1 version=$2 major previous guide
  [[ "$version" =~ ^([0-9]+)\.[0-9]+\.[0-9]+$ ]] || return 0
  major=${BASH_REMATCH[1]}
  [[ "$major" -ge 1 ]] || return 0
  # The match runs inside awk: a `grep -q` after a pipe would stop reading at the
  # first match, and under pipefail the writer's SIGPIPE on a large section
  # would read as no breaking changes.
  awk '/^## \[/ { if (seen) exit; seen = 1 } seen && /^### Breaking changes/ { found = 1; exit } END { exit !found }' \
    "$changelog" || return 0
  previous=$(git tag --list 'v[0-9]*' --sort=-version:refname | grep -vxF "v$version" | head -n 1 || true)
  [[ "$previous" =~ ^v([0-9]+)\. && "${BASH_REMATCH[1]}" -eq "$major" ]] || return 0
  guide="docs/migrating/v$version.md"
  if [[ -f "$guide" ]] && head -n 1 "$guide" | grep -qi 'small-break exception'; then
    gate_pass "$guide opens by naming the small-break exception ($version keeps major $major over $previous with ### Breaking changes)"
  else
    gate_fail "migration guide" "$version keeps major $major over $previous, its changelog carries ### Breaking changes, and $guide $([[ -f "$guide" ]] && echo "does not name the small-break exception on its first line" || echo "does not exist")
    fix: release $((major + 1)).0.0, or, when the maintainer takes the small-break exception, write $guide opening with a line that names it (references/versioning.md in github-repo-setup shows one)"
  fi
}

gate_mechanics() {
  header "Release mechanics sanity"
  local project_version changelog_version site=""

  # The version a release tag names: the release manifest's for Rust, which a
  # virtual workspace names in release.env. A site's is the deploy stamp its
  # changelog heading carries, checked below with site-version.sh. Non-Rust:
  # swap for the project's source of truth.
  if [[ -n "${SITE_VERSION_BASE:-}" ]]; then
    site=1
    project_version=""
    gate_pass "site version: the deploy stamp at base $SITE_VERSION_BASE (scripts/release/site-version.sh)"
  elif [[ -f Cargo.toml ]]; then
    project_version=$(project_version)
    gate_pass "$(release_manifest) version = $project_version"
    if [[ -f Cargo.lock ]]; then
      gate_pass "Cargo.lock present"
    else
      gate_fail "Cargo.lock" "missing"
    fi
  elif [[ -f package.json ]]; then
    project_version=$(jaq -r .version package.json)
    gate_pass "package.json version = $project_version"
  elif [[ -f pyproject.toml ]]; then
    project_version=$(grep -m1 '^version = ' pyproject.toml | sed -E 's/^version = "(.*)"/\1/' || true)
    gate_pass "pyproject.toml version = $project_version"
  elif [[ -f VERSION ]]; then
    project_version=$(<VERSION)
    gate_pass "VERSION = $project_version"
  else
    gate_skip "project version" "no Cargo.toml / package.json / pyproject.toml / VERSION found"
    project_version=""
  fi

  if [[ -n "$site" ]]; then
    :
  elif [[ -x "$BIN_PATH" && -n "$project_version" ]]; then
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

  # The changelog the release notes are cut from. One that release.env names
  # must exist; a repo that declares none and keeps no CHANGELOG.md has
  # nothing to check.
  local release_changelog
  release_changelog=$(release_changelog)
  if [[ -f "$release_changelog" ]]; then
    changelog_version=$(grep -m1 -oE '^## \[[0-9]+\.[0-9]+\.[0-9]+(-g[0-9a-f]{7})?\]' "$release_changelog" | tr -d '[]## ' || true)
    if [[ -n "$site" ]]; then
      local verified
      if [[ ! -x scripts/release/site-version.sh ]]; then
        gate_fail "$release_changelog top section" "scripts/release/site-version.sh is not vendored
    fix: copy templates/scripts/release/site-version.sh from github-repo-setup to scripts/release/"
      elif git rev-parse --verify --quiet "refs/tags/v$changelog_version" >/dev/null; then
        gate_fail "$release_changelog top section = [$changelog_version]" "v$changelog_version is already tagged, so this is a released section, not this release's
    fix: re-cut the release with scripts/release/cut-release-branch.sh, which writes the section for the next stamp"
      elif verified=$(scripts/release/site-version.sh --verify "v$changelog_version" 2>&1); then
        gate_pass "$release_changelog top section = [$changelog_version] (site-version.sh: ${verified#v"$changelog_version" })"
      else
        gate_fail "$release_changelog top section = [${changelog_version:-none}]" "$verified"
      fi
    elif [[ -n "$project_version" ]]; then
      if [[ "$changelog_version" == "$project_version" ]]; then
        gate_pass "$release_changelog top section = [$changelog_version] (matches project version)"
      else
        gate_fail "$release_changelog mismatch" "changelog=$changelog_version project=$project_version"
      fi
    fi
    migration_guide_check "$release_changelog" "$project_version"
    # The heading, not the text: a released section may quote `[Unreleased]`.
    if grep -qE '^## \[Unreleased\]' "$release_changelog"; then
      gate_fail "$release_changelog" "has [Unreleased] placeholder"
    else
      gate_pass "$release_changelog has no [Unreleased] placeholder"
    fi
  elif [[ -n "${RELEASE_CHANGELOG:-}" ]]; then
    gate_fail "$release_changelog" "missing; release.env names it as the release changelog"
  fi

  # Each workspace member on its own tag line, checked while its release is
  # pending: no `<tag_prefix><version>` tag exists yet, so the library
  # pipeline will cut its notes from that section. The cut script writes only
  # the release package's changelog, so this is what catches a member's that
  # the operator forgot.
  local name prefix member_changelog member_version member_top
  while IFS=$'\t' read -r name _ prefix member_changelog member_version; do
    [[ -n "$name" && -n "$member_version" ]] || continue
    if git rev-parse --verify --quiet "refs/tags/$prefix$member_version" >/dev/null; then
      gate_pass "$member_changelog not checked ($name $member_version is released as $prefix$member_version)"
      continue
    fi
    if [[ ! -f "$member_changelog" ]]; then
      gate_fail "$member_changelog" "missing; a $name release is pending (scripts/generate-changelog.py --crate $name --from-dev-prs --tag $prefix$member_version)"
      continue
    fi
    member_top=$(grep -m1 -oE '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' "$member_changelog" | tr -d '[]## ' || true)
    if [[ "$member_top" == "$member_version" ]]; then
      gate_pass "$member_changelog top section = [$member_top] (matches $name $member_version)"
    else
      gate_fail "$member_changelog" "top section=${member_top:-none} $name=$member_version; a $name release is pending (regenerate with scripts/generate-changelog.py --crate $name --from-dev-prs --tag $prefix$member_version)"
    fi
    if grep -qE '^## \[Unreleased\]' "$member_changelog"; then
      gate_fail "$member_changelog" "has [Unreleased] placeholder"
    fi
  done < <(release_members)

  # Rust: toolchain quarantine. Skip for non-Rust.
  if [[ -f rust-toolchain.toml ]]; then
    local toolchain_channel release_date_match
    toolchain_channel=$(grep -m1 'channel = ' rust-toolchain.toml | sed -E 's/.*"([^"]+)".*/\1/' || true)
    release_date_match=$(grep -m1 'released' rust-toolchain.toml | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' || true)
    if [[ -n "$release_date_match" ]]; then
      local age_days
      age_days=$((($(date +%s) - $(epoch_of_date "$release_date_match")) / 86400))
      if [[ $age_days -ge 7 ]]; then
        gate_pass "rust-toolchain channel=$toolchain_channel (released $release_date_match, $age_days days ago; 7-day quarantine satisfied)"
      else
        gate_fail "rust-toolchain quarantine" "channel $toolchain_channel released $release_date_match ($age_days days ago) is inside 7-day window"
      fi
    else
      gate_skip "rust-toolchain quarantine" "no 'released YYYY-MM-DD' comment found in rust-toolchain.toml"
    fi
  fi

  # Rust: cargo deny check advisories. Swap for the project's scanner.
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

  # Leak check: no guarded path added or modified in what the release ships to
  # main. --diff-filter=ACMR, as in cut-release-branch.sh's check B: a release
  # that removes a guarded doc main still carries from before the guard existed
  # lists the removal too, and that is cleanup, not a leak.
  local leaked
  leaked=$(git diff --diff-filter=ACMR "$ship_base..HEAD" --name-only 2>/dev/null | grep -E "$guarded" || true)
  if [[ -z "$leaked" ]]; then
    gate_pass "leak check (guarded paths): clean"
  else
    gate_fail "leak check" "guarded paths in diff vs $ship_base: $(echo "$leaked" | tr '\n' ' ')"
  fi

  # The leak check screens against the registered set, so it is blind to a
  # category nobody registered yet. Enumerate what the release adds to main
  # (anything under docs/, plus markdown anywhere, so a root-level glossary
  # shows up) and put every unguarded doc in front of a human. --no-renames,
  # because rename detection reports a doc moved from one main carries as R,
  # and the A filter then drops it.
  local added_docs
  added_docs=$(git diff --no-renames "$ship_base..HEAD" --diff-filter=A --name-only 2>/dev/null | grep -E '(^docs/|\.md$)' | grep -Ev "$guarded" || true)
  if [[ -z "$added_docs" ]]; then
    gate_pass "no unguarded docs newly added to main"
  else
    gate_skip "unguarded docs added to main (confirm each is meant to ship)" "$(echo "$added_docs" | tr '\n' ' ')"
  fi

  # diff-B: files on dev that this branch lacks. Excluding all of docs/ would
  # hide a missed pick under a directory that ships to main, so exclude only
  # the guarded set. Version files and the regenerated changelogs are
  # release-only by design, at any depth, so a workspace member's bump and
  # changelog read as release edits; cut-release-branch.sh's check A excludes
  # the same set.
  if git rev-parse --verify --quiet origin/dev >/dev/null 2>&1; then
    local missed
    missed=$(git diff HEAD..origin/dev --name-only 2>/dev/null | grep -Ev "$guarded" \
      | grep -Ev '^((.*/)?(Cargo\.toml|Cargo\.lock|package\.json|package-lock\.json|bun\.lock|pyproject\.toml|uv\.lock|VERSION|CHANGELOG\.md))$' || true)
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

# usage [CODE]: prints the header and exits CODE, 2 (a usage error) by default;
# --help passes 0.
usage() {
  print_usage_header
  exit "${1:-2}"
}

LAST_TAG=""
STAGING_URL=""
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
    --staging-url)
      STAGING_URL="$2"
      shift 2
      ;;
    -h | --help) usage 0 ;;
    drift | surface | api-contract | smoke | multi-app | oauth2-interactive | changelog-sections | mechanics | surface-smoke | all)
      SUBCMD="$1"
      shift
      ;;
    post-tag)
      echo "post-tag moved to scripts/release/postflight.sh, run that after the tag push" >&2
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
  oauth2-interactive) gate_oauth2_interactive ;;
  changelog-sections) gate_changelog_sections ;;
  mechanics) gate_mechanics ;;
  surface-smoke) gate_surface_smoke ;;
  all)
    gate_drift
    gate_surface
    gate_api_contract
    gate_smoke
    gate_multi_app
    gate_oauth2_interactive
    gate_surface_smoke
    gate_changelog_sections
    gate_mechanics
    ;;
esac

print_summary

[[ $FAIL_COUNT -eq 0 ]] || exit 1
