#!/usr/bin/env bash
# Verify the vX.Y.Z tag's downstream pipeline landed cleanly.
#
# Usage:
#   scripts/release-postflight.sh <subcommand>
#
# Runs AFTER the release/v<X.Y.Z> -> main PR merges and the tag is pushed,
# triggering release.yml. Companion to scripts/release-preflight.sh which
# runs BEFORE the release branch cut.
#
# Subcommands:
#   release      release.yml on the tag push (conclusion=success)
#   tap          homebrew-tap update-formula + Publish bottles SUCCESS
#   finalize     finalize-release.yml callback ran (cross-repo dispatch loop closed)
#   make-latest  GitHub Release v<X.Y.Z> is non-draft, non-prerelease, releases/latest matches
#   crates       crates.io index shows xurl-rs v<X.Y.Z> published
#   all          run every above sequentially
#
# Flags:
#   --repo OWNER/REPO       Override the auto-detected nameWithOwner
#   --tap-repo OWNER/REPO   Override the homebrew-tap repo (default: brettdavies/homebrew-tap)
#   --tag vX.Y.Z            Override the tag (default: derived from Cargo.toml version)
#
# Exit codes:
#   0 = all gates passed (or skipped with reason)
#   1 = one or more gates failed
#   2 = setup error (missing dep, unauthenticated gh, etc.)
#
# Install-on-fresh-machine smokes (cargo install, brew install, cargo binstall) are NOT
# driven from here. Running them on the local dev machine pollutes the toolchain and
# doesn't actually exercise "fresh machine" semantics. See RELEASES-POSTFLIGHT.md
# § Checklist for the recipes; drive on a throwaway container or sibling machine.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
readonly REPO_ROOT
readonly DEFAULT_TAP_REPO="brettdavies/homebrew-tap"

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
gate_skip() { printf "  %s⊝%s %s — %s\n" "$C_YLW" "$C_RST" "$1" "${2:-not yet ready}"; SKIP_COUNT=$((SKIP_COUNT + 1)); }
header()    { printf "\n%s== %s ==%s\n" "$C_BLD" "$1" "$C_RST"; }

require_bin() {
    command -v "$1" >/dev/null 2>&1 || { echo "missing dependency: $1" >&2; exit 2; }
}

# Argument parsing -----------------------------------------------------------

REPO=""
TAP_REPO="$DEFAULT_TAP_REPO"
TAG=""
SUBCMD=""

usage() {
    sed -n '2,32p' "$0" | sed 's/^# \?//'
    exit 2
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)     REPO="$2"; shift 2;;
        --tap-repo) TAP_REPO="$2"; shift 2;;
        --tag)      TAG="$2"; shift 2;;
        -h|--help)  usage;;
        release|tap|finalize|make-latest|crates|all) SUBCMD="$1"; shift;;
        *) echo "unknown arg: $1" >&2; usage;;
    esac
done

[[ -n "$SUBCMD" ]] || usage

resolve_repo() {
    [[ -n "$REPO" ]] && { echo "$REPO"; return; }
    gh repo view --json nameWithOwner --jq .nameWithOwner 2>/dev/null \
        || { echo "could not resolve repo (pass --repo OWNER/REPO)" >&2; exit 2; }
}

resolve_tag() {
    if [[ -n "$TAG" ]]; then
        echo "$TAG"; return
    fi
    local cargo_version
    cargo_version=$(grep -m1 '^version = ' "$REPO_ROOT/Cargo.toml" | sed -E 's/^version = "(.*)"/\1/')
    echo "v${cargo_version}"
}

# Gate: release.yml ----------------------------------------------------------

gate_release() {
    header "release.yml on tag push"
    require_bin gh; require_bin jaq
    local repo tag run
    repo=$(resolve_repo); tag=$(resolve_tag)

    run=$(gh run list --repo "$repo" --branch "$tag" --workflow release.yml --limit 1 \
        --json databaseId,status,conclusion --jq '.[0]' 2>/dev/null || true)
    if [[ -z "$run" || "$run" == "null" ]]; then
        gate_skip "release.yml run for $tag" "no run found on tag $tag yet (push the tag?)"
        return
    fi

    local status conclusion run_id
    status=$(printf '%s' "$run" | jaq -r .status)
    conclusion=$(printf '%s' "$run" | jaq -r .conclusion)
    run_id=$(printf '%s' "$run" | jaq -r .databaseId)

    if [[ "$status" != "completed" ]]; then
        gate_skip "release.yml run $run_id" "status=$status (still running; re-run after watcher exits)"
        return
    fi
    [[ "$conclusion" == "success" ]] \
        && gate_pass "release.yml run $run_id conclusion=success" \
        || gate_fail "release.yml run $run_id" "conclusion=$conclusion (see gh run view $run_id --log-failed)"
}

# Gate: homebrew-tap ---------------------------------------------------------

gate_tap() {
    header "homebrew-tap dispatch + bottles publish"
    require_bin gh; require_bin jaq
    local tap=$TAP_REPO tag
    tag=$(resolve_tag)

    # update-formula = repository_dispatch from release.yml
    local uf
    uf=$(gh run list --repo "$tap" --event repository_dispatch --limit 10 \
        --json databaseId,status,conclusion,displayTitle,createdAt \
        --jq "[.[] | select(.displayTitle == \"update-formula\")] | .[0]" 2>/dev/null || true)
    if [[ -z "$uf" || "$uf" == "null" ]]; then
        gate_skip "tap update-formula dispatch" "no recent run on $tap (release.yml may still be running)"
    else
        local uf_status uf_conclusion uf_id
        uf_status=$(printf '%s' "$uf" | jaq -r .status)
        uf_conclusion=$(printf '%s' "$uf" | jaq -r .conclusion)
        uf_id=$(printf '%s' "$uf" | jaq -r .databaseId)
        if [[ "$uf_status" == "completed" && "$uf_conclusion" == "success" ]]; then
            gate_pass "tap update-formula run $uf_id (dispatch from release.yml) success"
        elif [[ "$uf_status" == "completed" ]]; then
            gate_fail "tap update-formula run $uf_id" "conclusion=$uf_conclusion (see gh run view $uf_id -R $tap --log-failed)"
        else
            gate_skip "tap update-formula run $uf_id" "status=$uf_status"
        fi
    fi

    # Publish bottles = workflow_run triggered by the CI completion on the formula-bump PR
    local pb
    pb=$(gh run list --repo "$tap" --event workflow_run --limit 5 \
        --json databaseId,status,conclusion,displayTitle \
        --jq "[.[] | select(.displayTitle == \"Publish bottles\")] | .[0]" 2>/dev/null || true)
    if [[ -z "$pb" || "$pb" == "null" ]]; then
        gate_skip "tap Publish bottles" "no recent run (CI on PR may still be running)"
        return
    fi
    local pb_status pb_conclusion pb_id
    pb_status=$(printf '%s' "$pb" | jaq -r .status)
    pb_conclusion=$(printf '%s' "$pb" | jaq -r .conclusion)
    pb_id=$(printf '%s' "$pb" | jaq -r .databaseId)
    if [[ "$pb_status" == "completed" && "$pb_conclusion" == "success" ]]; then
        gate_pass "tap Publish bottles run $pb_id success (bottle commit pushed to $tap main)"
    elif [[ "$pb_status" == "completed" ]]; then
        gate_fail "tap Publish bottles run $pb_id" "conclusion=$pb_conclusion"
    else
        gate_skip "tap Publish bottles run $pb_id" "status=$pb_status"
    fi
}

# Gate: finalize-release.yml -------------------------------------------------

gate_finalize() {
    header "finalize-release.yml callback"
    require_bin gh; require_bin jaq
    local repo
    repo=$(resolve_repo)

    local fr
    fr=$(gh run list --repo "$repo" --event repository_dispatch --workflow finalize-release.yml --limit 3 \
        --json databaseId,status,conclusion --jq '.[0]' 2>/dev/null || true)
    if [[ -z "$fr" || "$fr" == "null" ]]; then
        gate_skip "finalize-release.yml run" "no callback yet (Publish bottles may still be running on $TAP_REPO)"
        return
    fi
    local fr_status fr_conclusion fr_id
    fr_status=$(printf '%s' "$fr" | jaq -r .status)
    fr_conclusion=$(printf '%s' "$fr" | jaq -r .conclusion)
    fr_id=$(printf '%s' "$fr" | jaq -r .databaseId)
    if [[ "$fr_status" == "completed" && "$fr_conclusion" == "success" ]]; then
        gate_pass "finalize-release.yml run $fr_id success (cross-repo dispatch loop closed)"
    elif [[ "$fr_status" == "completed" ]]; then
        gate_fail "finalize-release.yml run $fr_id" "conclusion=$fr_conclusion"
    else
        gate_skip "finalize-release.yml run $fr_id" "status=$fr_status"
    fi
}

# Gate: make_latest flip -----------------------------------------------------

gate_make_latest() {
    header "GitHub Release marked latest"
    require_bin gh; require_bin jaq
    local repo tag
    repo=$(resolve_repo); tag=$(resolve_tag)

    # Release exists + non-draft + non-prerelease + correct asset count
    local rel
    rel=$(gh release view "$tag" --repo "$repo" --json isDraft,isPrerelease,assets 2>/dev/null || true)
    if [[ -z "$rel" ]]; then
        gate_skip "Release $tag" "release.yml hasn't created it yet"
        return
    fi
    local is_draft is_prerelease asset_count
    is_draft=$(printf '%s' "$rel" | jaq -r .isDraft)
    is_prerelease=$(printf '%s' "$rel" | jaq -r .isPrerelease)
    asset_count=$(printf '%s' "$rel" | jaq -r '.assets | length')

    if [[ "$is_draft" == "true" ]]; then
        gate_fail "Release $tag draft" "isDraft=true (release.yml should publish non-draft)"
    elif [[ "$is_prerelease" == "true" ]]; then
        gate_fail "Release $tag prerelease" "isPrerelease=true (release.yml should publish stable)"
    else
        gate_pass "Release $tag published non-draft, non-prerelease, $asset_count assets"
    fi

    # /releases/latest must resolve to this tag (set by finalize-release flipping make_latest)
    local latest
    latest=$(gh api "repos/$repo/releases/latest" --jq .tag_name 2>/dev/null || true)
    if [[ "$latest" == "$tag" ]]; then
        gate_pass "releases/latest = $tag (finalize-release flipped make_latest=true)"
    elif [[ -n "$latest" ]]; then
        gate_skip "releases/latest" "currently $latest, expected $tag (homebrew dispatch chain may still be running)"
    else
        gate_skip "releases/latest" "no latest release found"
    fi
}

# Gate: crates.io ------------------------------------------------------------

gate_crates() {
    header "crates.io publish"
    require_bin cargo
    local tag version
    tag=$(resolve_tag); version="${tag#v}"

    local found
    found=$(cargo search xurl-rs --limit 1 2>/dev/null | grep -E "^xurl-rs = " | head -1 || true)
    if [[ -z "$found" ]]; then
        gate_skip "crates.io xurl-rs" "no matching crate in index (publish may still be in flight)"
        return
    fi

    # cargo search prints `name = "version" # description`
    local published
    published=$(printf '%s' "$found" | sed -E 's/.*"([^"]+)".*/\1/')
    if [[ "$published" == "$version" ]]; then
        gate_pass "crates.io shows xurl-rs $published (matches $tag)"
    else
        gate_skip "crates.io xurl-rs" "index shows $published, expected $version (publish may still be replicating)"
    fi
}

# Main dispatcher ------------------------------------------------------------

case "$SUBCMD" in
    release)     gate_release;;
    tap)         gate_tap;;
    finalize)    gate_finalize;;
    make-latest) gate_make_latest;;
    crates)      gate_crates;;
    all)         gate_release; gate_tap; gate_finalize; gate_make_latest; gate_crates;;
esac

printf "\n%sSummary:%s  %s%d passed%s  %s%d failed%s  %s%d skipped%s\n" \
    "$C_BLD" "$C_RST" "$C_GRN" "$PASS_COUNT" "$C_RST" "$C_RED" "$FAIL_COUNT" "$C_RST" "$C_YLW" "$SKIP_COUNT" "$C_RST"

[[ $FAIL_COUNT -eq 0 ]] || exit 1
