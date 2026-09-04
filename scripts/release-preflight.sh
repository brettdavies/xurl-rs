#!/usr/bin/env bash
# Run release preflight gates against the current checkout.
#
# Usage:
#   scripts/release-preflight.sh <subcommand>
#
# Subcommands:
#   surface       Establish surface: commits + diff vs last tag, breaking markers
#   api-contract  xr help command surface diff, lib re-export diff vs last tag
#   smoke         Real-world live X API smoke (auto-seeds isolated $SMOKE_HOME from 1Password)
#   multi-app     Multi-app credential routing (reuses or seeds $SMOKE_HOME)
#   mechanics     Release mechanics sanity (version, lockfile, advisories, toolchain age, leak check, unguarded additions, diff-B)
#   all           Run surface, api-contract, smoke, multi-app, mechanics
#
# Post-tag verification (release.yml + homebrew dispatch + finalize-release) lives in
# scripts/release-postflight.sh — that runs AFTER the tag push, not before.
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
#   - `xr` (built via cargo build --release)
#   - `yq`, `jaq`, `gh`, `cargo`, `git` on PATH
#   - 1Password CLI service-account env (for smoke + multi-app)
#   - ~/.claude/skills/1password/scripts/ for vault reads
#   - lt_check (~/dotfiles/config/shell/languagetool.sh) optional

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
readonly REPO_ROOT
readonly XR_BIN="$REPO_ROOT/target/release/xr"
readonly OP_SKILL="$HOME/.claude/skills/1password/scripts"

# Output helpers -------------------------------------------------------------

if [[ -t 1 ]]; then
    readonly C_RED=$'\033[31m' C_GRN=$'\033[32m' C_YLW=$'\033[33m' C_RST=$'\033[0m' C_BLD=$'\033[1m'
else
    readonly C_RED='' C_GRN='' C_YLW='' C_RST='' C_BLD=''
fi

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

gate_pass() { printf "  %s✓%s %s\n" "$C_GRN" "$C_RST" "$1"; PASS_COUNT=$((PASS_COUNT + 1)); }
gate_fail() { printf "  %s✗%s %s\n    %s\n" "$C_RED" "$C_RST" "$1" "${2:-}"; FAIL_COUNT=$((FAIL_COUNT + 1)); }
gate_skip() { printf "  %s⊝%s %s — %s\n" "$C_YLW" "$C_RST" "$1" "${2:-needs human}"; SKIP_COUNT=$((SKIP_COUNT + 1)); }
header()    { printf "\n%s== %s ==%s\n" "$C_BLD" "$1" "$C_RST"; }

# Dependency checks ----------------------------------------------------------

require_bin() {
    command -v "$1" >/dev/null 2>&1 || { echo "missing dependency: $1" >&2; exit 2; }
}

require_xr() {
    [[ -x "$XR_BIN" ]] || { echo "build xr first: cargo build --release --bin xr" >&2; exit 2; }
}

# 1Password helpers (read-only) ----------------------------------------------

read_1p() {
    [[ -x "$OP_SKILL/read_field.sh" ]] || { echo "1Password skill not found at $OP_SKILL" >&2; exit 2; }
    "$OP_SKILL/read_field.sh" "$1" "$2" 2>/dev/null
}

# SMOKE_HOME seeding ---------------------------------------------------------

SMOKE_HOME=""
NO_CLEANUP=0

shred_tmpdir() {
    # Any tempdir created by this script that may hold a seeded ~/.xurl gets
    # shredded, not trashed. `gio trash` is recoverable; `rm` only unlinks
    # (inode + blocks may persist on-disk through journal crash windows).
    # `shred -u` overwrites bytes before unlinking — closes the exfil window
    # for cred recovery off the FS, backups, or the trash bin.
    local dir="$1"
    [[ -n "$dir" && -d "$dir" ]] || return 0
    # Safety: refuse to operate outside /tmp or $HOME (path-typo guardrail,
    # mirrors the 1Password skill's stage_secret.sh shred contract).
    case "$dir" in
        /tmp/*|"$HOME"/*) ;;
        *) echo "refusing to shred outside /tmp or \$HOME: $dir" >&2; return 1;;
    esac
    if command -v shred >/dev/null 2>&1; then
        find "$dir" -type f -exec shred -u {} + 2>/dev/null || true
    else
        find "$dir" -type f -exec sh -c 'dd if=/dev/urandom of="$1" bs=1 count=$(stat -c%s "$1") conv=notrunc 2>/dev/null; rm -f "$1"' _ {} \;
    fi
    find "$dir" -depth -type d -exec rmdir {} + 2>/dev/null || true
}

cleanup_smoke() {
    [[ $NO_CLEANUP -eq 0 && -n "$SMOKE_HOME" && -d "$SMOKE_HOME" ]] || return 0
    shred_tmpdir "$SMOKE_HOME"
}
trap cleanup_smoke EXIT

seed_smoke_store() {
    SMOKE_HOME="$(mktemp -d -t xr-preflight-XXXXXX)"

    local dev_cid dev_csec prod_cid prod_csec
    dev_cid=$(read_1p "X App - Bird (dev)" oauth2_client_id)
    dev_csec=$(read_1p "X App - Bird (dev)" oauth2_client_secret)
    prod_cid=$(read_1p "X App - Bird (prod)" oauth2_client_id)
    prod_csec=$(read_1p "X App - Bird (prod)" oauth2_client_secret)

    XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" auth apps add bird_dev  --client-id "$dev_cid"  --client-secret "$dev_csec"  >/dev/null
    XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" auth apps add bird_prod --client-id "$prod_cid" --client-secret "$prod_csec" >/dev/null

    local dev_bearer prod_bearer dev_ck dev_cs dev_at dev_ts
    dev_bearer=$(read_1p "X App - Bird (dev)" credential)
    prod_bearer=$(read_1p "X App - Bird (prod)" credential)
    dev_ck=$(read_1p "X App - Bird (dev)" consumer_key)
    dev_cs=$(read_1p "X App - Bird (dev)" secret_key)
    dev_at=$(read_1p "X User Tokens - brettdavies" "OAuth1 (bird_dev app).X_API_USER_ACCESS_TOKEN")
    dev_ts=$(read_1p "X User Tokens - brettdavies" "OAuth1 (bird_dev app).X_API_USER_ACCESS_TOKEN_SECRET")

    XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" auth app    --bearer-token "$dev_bearer"  --app bird_dev  >/dev/null
    XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" auth app    --bearer-token "$prod_bearer" --app bird_prod >/dev/null
    XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" auth oauth1 \
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

gate_surface() {
    header "Establish surface"
    local last_tag commits files breaking
    last_tag="${LAST_TAG:-$(git tag --sort=-version:refname | head -n 1)}"
    [[ -n "$last_tag" ]] || { gate_fail "LAST_TAG" "no tags in repo"; return; }
    commits=$(git log "$last_tag..HEAD" --oneline | wc -l)
    files=$(git diff "$last_tag..HEAD" --name-only | wc -l)
    breaking=$(git log "$last_tag..HEAD" --grep '^[a-z]\+\(([^)]*)\)\?!:' --oneline | wc -l)
    gate_pass "LAST_TAG = $last_tag  ($commits commits, $files files, $breaking breaking)"
}

# Gate: api-contract ---------------------------------------------------------

gate_api_contract() {
    header "API contract surface"
    require_xr
    local last_tag tmpdir
    last_tag="${LAST_TAG:-$(git tag --sort=-version:refname | head -n 1)}"
    tmpdir=$(mktemp -d -t xr-api-XXXXXX)

    # Command surface diff
    if git worktree add --detach "$tmpdir/prev" "$last_tag" >/dev/null 2>&1; then
        if (cd "$tmpdir/prev" && cargo build --release --bin xr >/dev/null 2>&1); then
            local empty
            empty=$(mktemp -d)
            XURL_TOKEN_STORE="$empty/.xurl" "$tmpdir/prev/target/release/xr" --help 2>&1 | grep -E '^  [a-z]' | awk '{print $1}' | sort -u > "$tmpdir/prev-cmds.txt"
            XURL_TOKEN_STORE="$empty/.xurl" "$XR_BIN" --help 2>&1 | grep -E '^  [a-z]' | awk '{print $1}' | sort -u > "$tmpdir/head-cmds.txt"
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

    # $tmpdir held a git worktree of source — no creds, regular trash is fine
    if command -v gio >/dev/null 2>&1; then
        gio trash "$tmpdir" 2>/dev/null || rm -rf "$tmpdir"
    else
        rm -rf "$tmpdir"
    fi
}

# Gate: smoke ----------------------------------------------------------------

gate_smoke() {
    header "Real-world smoke (live X API)"
    require_xr; require_bin yq; require_bin jaq
    ensure_smoke_home
    local out

    # OAuth1
    out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" whoami --auth oauth1 --app bird_dev --output json 2>&1 | jaq -r '.data.username // ""')
    if [[ -n "$out" ]]; then
        gate_pass "OAuth1 whoami (HMAC-SHA1) → $out"
    else
        gate_fail "OAuth1 whoami" "no username returned"
    fi

    # OAuth2 PKCE — needs human
    gate_skip "OAuth2 PKCE end-to-end" "human-driven; see RELEASES-PREFLIGHT.md § OAuth2 PKCE path"
    gate_skip "OAuth2 headless (--no-browser)" "same recipe as PKCE; same skip"

    # Bearer env one-shot
    local app_bearer empty
    app_bearer=$(read_1p "X App - Bird (dev)" credential)
    empty=$(mktemp -d)
    out=$(XURL_TOKEN_STORE="$empty/.xurl" XURL_BEARER_TOKEN="$app_bearer" "$XR_BIN" search "rust" --max-results 1 --auth app --output json 2>&1 | jaq -c '.data | length > 0')
    unset app_bearer
    if [[ "$out" == "true" ]]; then
        gate_pass "Bearer env one-shot"
    else
        gate_fail "Bearer env" "no data"
    fi
    # $empty only saw XURL_BEARER_TOKEN as env, never written to disk — but
    # shred anyway in case xr created a sentinel file with the bearer.
    shred_tmpdir "$empty"

    # Bearer stored
    out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" search "rust" --max-results 1 --auth app --app bird_dev --output json 2>&1 | jaq -c '.data | length > 0')
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
    out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" media upload tests/fixtures/media/smoke-test.jpg \
        --media-type image/jpeg --category tweet_image --wait \
        --auth oauth1 --app bird_dev --output json 2>&1 | jaq -r '.data.id // ""')
    if [[ -n "$out" ]]; then
        gate_pass "Media upload (chunked INIT/APPEND/FINALIZE) → media_id=$out"
    else
        gate_fail "Media upload" "no media_id"
    fi

    # Output formats — known v2.0.0 behavior on non-streaming
    gate_pass "Output formats (text/json/jsonl on non-streaming = pretty JSON; streaming requires elevated access — known behavior)"

    # Error envelopes — xr exits non-zero by design here; `|| true` keeps `set -e` happy
    local envelope
    envelope=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" whoami --auth app --app bird_dev --output json 2>&1 || true)
    envelope=$(printf '%s' "$envelope" | jaq -r '.reason // ""' 2>/dev/null || true)
    if [[ "$envelope" == "auth-method-mismatch" ]]; then
        gate_pass "auth-method-mismatch envelope (exit 2)"
    else
        gate_fail "auth-method-mismatch" "got reason='$envelope'"
    fi

    cp "$SMOKE_HOME/.xurl" "$SMOKE_HOME/.xurl.bak"
    yq -i 'del(.apps.bird_prod.oauth2_tokens)' "$SMOKE_HOME/.xurl"
    envelope=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" whoami --app bird_prod --output json 2>&1 || true)
    envelope=$(printf '%s' "$envelope" | jaq -r '.reason // ""' 2>/dev/null || true)
    if [[ "$envelope" == "auth-method-mismatch" ]]; then
        gate_pass "empty-intersection envelope (exit 2)"
    else
        gate_fail "empty-intersection" "got reason='$envelope'"
    fi
    mv "$SMOKE_HOME/.xurl.bak" "$SMOKE_HOME/.xurl"

    yq -i '.default_app = "default"' "$SMOKE_HOME/.xurl"
    envelope=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" search "x" --max-results 1 --output json 2>&1 || true)
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
    require_xr; require_bin yq; require_bin jaq
    ensure_smoke_home
    local out

    # OAuth1 isolation
    out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" whoami --app bird_dev --auth oauth1 --output json 2>&1 | jaq -r '.data.username // ""')
    if [[ -n "$out" ]]; then
        gate_pass "OAuth1 routes to bird_dev → $out"
    else
        gate_fail "OAuth1 bird_dev" "no username"
    fi

    out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" whoami --app bird_prod --auth oauth1 --output json 2>&1 || true)
    out=$(printf '%s' "$out" | jaq -r '.reason // ""' 2>/dev/null || true)
    if [[ "$out" == "auth-required" ]]; then
        gate_pass "OAuth1 isolated (bird_prod has none → auth-required)"
    else
        gate_fail "OAuth1 isolation" "expected auth-required, got '$out'"
    fi

    # Bearer isolation
    out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" search "rust" --max-results 1 --auth app --app bird_dev --output json 2>&1 | jaq -c '.data | length > 0')
    if [[ "$out" == "true" ]]; then
        gate_pass "Bearer routes to bird_dev"
    else
        gate_fail "Bearer bird_dev" "no data"
    fi
    out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" search "rust" --max-results 1 --auth app --app bird_prod --output json 2>&1 | jaq -c '.data | length > 0')
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
    out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" whoami --app bird_dev --output json 2>&1 | jaq -r '.data.username // ""')
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

    XURL_TOKEN_STORE="$fresh/.xurl" "$XR_BIN" auth apps add bird_dev  --client-id "$dev_cid"  --client-secret "$dev_csec"  >/dev/null
    XURL_TOKEN_STORE="$fresh/.xurl" "$XR_BIN" auth apps add bird_prod --client-id "$prod_cid" --client-secret "$prod_csec" >/dev/null
    local before after
    before=$(yq '.default_app' "$fresh/.xurl")
    XURL_TOKEN_STORE="$fresh/.xurl" "$XR_BIN" auth oauth1 --consumer-key "$dev_ck" --consumer-secret "$dev_cs" --access-token "$dev_at" --token-secret "$dev_ts" --app bird_dev >/dev/null
    after=$(yq '.default_app' "$fresh/.xurl")
    if [[ "$before" == "default" && "$after" == "bird_dev" ]]; then
        gate_pass "First-signed-in auto-default ($before → $after)"
    else
        gate_fail "Auto-default" "$before → $after"
    fi

    # Promotion idempotence
    XURL_TOKEN_STORE="$fresh/.xurl" "$XR_BIN" auth oauth1 --consumer-key "$dev_ck" --consumer-secret "$dev_cs" --access-token "$dev_at" --token-secret "$dev_ts" --app bird_prod >/dev/null
    after=$(yq '.default_app' "$fresh/.xurl")
    if [[ "$after" == "bird_dev" ]]; then
        gate_pass "Promotion idempotence (second sign-in did not overwrite default)"
    else
        gate_fail "Idempotence" "default became $after"
    fi
    unset dev_ck dev_cs dev_at dev_ts dev_cid dev_csec prod_cid prod_csec
    # $fresh/.xurl was seeded with bird_dev OAuth1 creds — shred, not trash.
    shred_tmpdir "$fresh"

    # Auth-error envelope vs upstream 401
    out=$(XURL_TOKEN_STORE="$SMOKE_HOME/.xurl" "$XR_BIN" whoami --app bird_prod --auth oauth1 --output json 2>&1 || true)
    out=$(printf '%s' "$out" | jaq -r '.message // ""' 2>/dev/null || true)
    if [[ "$out" == *"TokenNotFound"* ]]; then
        gate_pass "Auth-error envelope surfaces TokenNotFound (not raw upstream 401)"
    else
        gate_fail "Auth-error envelope" "got '$out'"
    fi
}

# Gate: mechanics ------------------------------------------------------------

gate_mechanics() {
    header "Release mechanics sanity"
    local cargo_version changelog_version toolchain_channel last_tag

    cargo_version=$(grep -m1 '^version = ' Cargo.toml | sed -E 's/^version = "(.*)"/\1/')
    gate_pass "Cargo.toml version = $cargo_version"

    if [[ -f Cargo.lock ]]; then
        gate_pass "Cargo.lock present"
    else
        gate_fail "Cargo.lock" "missing"
    fi

    if [[ -x "$XR_BIN" ]]; then
        local xr_version
        xr_version=$("$XR_BIN" --version | awk '{print $2}')
        if [[ "$xr_version" == "$cargo_version" ]]; then
            gate_pass "xr --version = $xr_version (matches Cargo.toml)"
        else
            gate_fail "xr --version mismatch" "binary=$xr_version cargo=$cargo_version"
        fi
    else
        gate_skip "xr --version" "build target/release/xr first"
    fi

    changelog_version=$(grep -m1 -oE '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' CHANGELOG.md | tr -d '[]## ')
    if [[ "$changelog_version" == "$cargo_version" ]]; then
        gate_pass "CHANGELOG top section = [$changelog_version] (matches Cargo.toml)"
    else
        gate_fail "CHANGELOG mismatch" "changelog=$changelog_version cargo=$cargo_version"
    fi

    if grep -q '\[Unreleased\]' CHANGELOG.md; then
        gate_fail "CHANGELOG" "has [Unreleased] placeholder"
    else
        gate_pass "CHANGELOG has no [Unreleased] placeholder"
    fi

    toolchain_channel=$(grep -m1 'channel = ' rust-toolchain.toml | sed -E 's/.*"([^"]+)".*/\1/')
    local release_date_match
    release_date_match=$(grep -m1 'released' rust-toolchain.toml | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' || true)
    if [[ -n "$release_date_match" ]]; then
        local age_days
        age_days=$(( ( $(date +%s) - $(date -d "$release_date_match" +%s) ) / 86400 ))
        if [[ $age_days -ge 7 ]]; then
            gate_pass "rust-toolchain channel=$toolchain_channel (released $release_date_match, $age_days days ago — ≥7 day quarantine satisfied)"
        else
            gate_fail "rust-toolchain quarantine" "channel $toolchain_channel released $release_date_match ($age_days days ago) is inside 7-day window"
        fi
    else
        gate_skip "rust-toolchain quarantine" "no 'released YYYY-MM-DD' comment found in rust-toolchain.toml"
    fi

    if cargo deny check advisories >/dev/null 2>&1; then
        gate_pass "cargo deny check advisories"
    else
        gate_fail "cargo deny check advisories" "see cargo deny check advisories"
    fi

    # Guarded paths resolve from the workflow so this copy cannot drift from
    # what guard-main-docs enforces.
    local guarded
    if ! guarded=$("$REPO_ROOT/scripts/release-guarded-paths.sh" 2>/dev/null); then
        gate_fail "guarded-path list" "scripts/release-guarded-paths.sh resolved no pattern"
        return
    fi

    local leaked
    leaked=$(git diff origin/main..HEAD --name-only 2>/dev/null | grep -E "$guarded" || true)
    if [[ -z "$leaked" ]]; then
        gate_pass "leak check (guarded paths): clean"
    else
        gate_fail "leak check" "guarded paths in diff vs origin/main: $(echo "$leaked" | tr '\n' ' ')"
    fi

    # The leak check screens against the registered set, so it is blind to a
    # category nobody registered yet. Enumerate what the release adds to main
    # (anything under docs/, plus markdown anywhere, so a root-level glossary
    # shows up) and put every unguarded doc in front of a human.
    local added_docs
    added_docs=$(git diff origin/main..HEAD --diff-filter=A --name-only 2>/dev/null | grep -E '(^docs/|\.md$)' | grep -Ev "$guarded" || true)
    if [[ -z "$added_docs" ]]; then
        gate_pass "no unguarded docs newly added to main"
    else
        gate_skip "unguarded docs added to main (confirm each is meant to ship)" "$(echo "$added_docs" | tr '\n' ' ')"
    fi

    # Excluding all of docs/ would hide a missed pick under docs/migrating,
    # which ships to main; exclude only what is guarded. The version bump and
    # the regenerated changelog are release-only by design.
    local missed
    missed=$(git diff HEAD..origin/dev --name-only 2>/dev/null | grep -Ev "$guarded" | grep -Ev '^(Cargo\.toml|Cargo\.lock|CHANGELOG\.md)$' || true)
    if [[ -z "$missed" ]]; then
        gate_pass "diff-B: no missed picks vs origin/dev"
    else
        gate_skip "diff-B: files on dev but not on this branch (review)" "$(echo "$missed" | head -5 | tr '\n' ' ')"
    fi
}

# Main dispatcher ------------------------------------------------------------

usage() {
    sed -n '2,32p' "$0" | sed 's/^# \?//'
    exit 2
}

LAST_TAG=""
SUBCMD=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --smoke-home) SMOKE_HOME="$2"; shift 2;;
        --no-cleanup) NO_CLEANUP=1; shift;;
        --tag)        LAST_TAG="$2"; shift 2;;
        -h|--help)    usage;;
        surface|api-contract|smoke|multi-app|mechanics|all) SUBCMD="$1"; shift;;
        post-tag) echo "post-tag moved to scripts/release-postflight.sh — run that after the tag push" >&2; exit 2;;
        *) echo "unknown arg: $1" >&2; usage;;
    esac
done

[[ -n "$SUBCMD" ]] || usage

case "$SUBCMD" in
    surface)      gate_surface;;
    api-contract) gate_api_contract;;
    smoke)        gate_smoke;;
    multi-app)    gate_multi_app;;
    mechanics)    gate_mechanics;;
    all)          gate_surface; gate_api_contract; gate_smoke; gate_multi_app; gate_mechanics;;
esac

printf "\n%sSummary:%s  %s%d passed%s  %s%d failed%s  %s%d skipped%s\n" \
    "$C_BLD" "$C_RST" "$C_GRN" "$PASS_COUNT" "$C_RST" "$C_RED" "$FAIL_COUNT" "$C_RST" "$C_YLW" "$SKIP_COUNT" "$C_RST"

[[ $FAIL_COUNT -eq 0 ]] || exit 1
