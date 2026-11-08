#!/usr/bin/env bash
#
# Tag the release on main and push every tag it needs in one push.
#
# Usage:
#   scripts/release/tag-release.sh [--dry-run] [--force] [--remote NAME]
#
# Runs after the release/v<version> -> main PR merges, from a clean checkout of
# main that equals origin/main. It tags the release package's version as
# `v<version>`, and each publishable workspace member whose version has no tag
# yet as its own `<crate>-v<version>`, all on main's head. The tags are
# annotated, so the repository's tag-signing configuration applies, and go out
# in one atomic push: either every tag reaches the remote or none does.
#
# Before it creates a tag it lists the queued and in-progress workflow runs in
# every repository the signed-in account owns and has not archived, because
# those runs share the account's concurrent-job limit with the release. A run
# listed there holds the cut, unless --force.
#
# Options:
#   --dry-run       Run every check and print the tags; create and push nothing.
#   --force         Push while the account's runs are listed, or when they
#                   could not be read.
#   --remote NAME   The remote to push to (default: origin).
#   -h, --help      This text.
#
# Exit codes:
#   0 = the tags are pushed (under --dry-run: every check passed)
#   1 = a check failed: runs hold the account's runners, or the push failed,
#       in which case the tags this run created are deleted
#   2 = a refusal or setup error: a dirty worktree, main not at origin/main, a
#       remote that does not answer, a tag that already exists, an unreadable
#       runner check, or a missing tool
#
# Each refusal prints `error[<code>]:` on stderr with the values behind it, the
# next step, and the RELEASES.md section that answers it.

set -euo pipefail

# shellcheck disable=SC1091  # sibling _lib.sh, always vendored alongside
. "$(dirname "$0")/_lib.sh"

usage() {
  print_usage_header
  exit "${1:-2}"
}

DRY_RUN=""
FORCE=""
REMOTE="origin"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --force)
      FORCE=1
      shift
      ;;
    --remote)
      [[ $# -ge 2 ]] || usage
      REMOTE="$2"
      shift 2
      ;;
    -h | --help) usage 0 ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      ;;
  esac
done

DOCS="RELEASES.md"

# refuse CODE EXIT MESSAGE NEXT: the one way this script stops on a check.
refuse() {
  printf '%serror[%s]%s: %s\n' "$C_RED" "$1" "$C_RST" "$3" >&2
  printf 'next: %s\n' "$4" >&2
  printf 'see: %s#%s\n' "$DOCS" "$1" >&2
  exit "$2"
}

indent() { sed 's/^/  /'; }

require_bin git
require_bin gh
require_bin cargo
require_bin jaq

cd "$(git rev-parse --show-toplevel)"

# Preconditions -------------------------------------------------------------

header "Checkout"

if [[ -n "$(git status --porcelain)" ]]; then
  refuse dirty-tree 2 "the working tree has uncommitted changes:
$(git status --porcelain | indent)" "commit or stash them, then run this again"
fi
gate_pass "working tree clean"

branch=$(git symbolic-ref --quiet --short HEAD || echo "a detached HEAD")
if [[ "$branch" != "main" ]]; then
  refuse main-not-current 2 "the checkout is on ${branch}, not main" "git checkout main && git pull --ff-only"
fi
git fetch --quiet "$REMOTE" main 2>/dev/null \
  || refuse remote-unreachable 2 "git fetch ${REMOTE} main failed" "git remote -v and the network, then run this again"
local_head=$(git rev-parse HEAD)
remote_head=$(git rev-parse "$REMOTE/main")
if [[ "$local_head" != "$remote_head" ]]; then
  read -r ahead behind < <(git rev-list --left-right --count "HEAD...$REMOTE/main")
  if [[ "$ahead" -eq 0 ]]; then
    refuse main-not-current 2 "main is ${behind} commit(s) behind ${REMOTE}/main" "git pull --ff-only"
  fi
  refuse main-not-current 2 "main carries ${ahead} commit(s) ${REMOTE}/main does not (and lacks ${behind}), so a tag on it would release a commit the remote's main never received" \
    "a person decides: reset main to ${REMOTE}/main, or land those commits through a pull request first"
fi
gate_pass "main is at ${REMOTE}/main (${local_head:0:12})"

# Tags ----------------------------------------------------------------------

header "Tags"

version=$(project_version)
[[ -n "$version" ]] || {
  echo "no version in $(release_manifest); set RELEASE_MANIFEST in scripts/release/release.env" >&2
  exit 2
}

remote_tags=$(git ls-remote --tags --refs "$REMOTE" 2>/dev/null | sed 's#.*refs/tags/##') \
  || refuse remote-unreachable 2 "git ls-remote --tags ${REMOTE} failed" "git remote -v and the network, then run this again"
on_remote() { grep -qxF "$1" <<<"$remote_tags"; }
on_local() { git rev-parse --quiet --verify "refs/tags/$1" >/dev/null; }

binary_tag="v${version}"
if on_remote "$binary_tag"; then
  refuse tag-exists 2 "${binary_tag} is already on ${REMOTE}: this version was cut" \
    "bump the version for a new release, or follow the release already running: scripts/release/postflight.sh --tag ${binary_tag} all"
fi
if on_local "$binary_tag"; then
  refuse tag-exists 2 "${binary_tag} exists in this checkout but not on ${REMOTE}" \
    "git tag -d ${binary_tag}, then run this again"
fi

tags=()
while IFS=$'\t' read -r name _manifest prefix _changelog member_version; do
  [[ -n "$name" ]] || continue
  tag="${prefix}${member_version}"
  if on_remote "$tag"; then
    gate_pass "${name} ${member_version} is already tagged ${tag}"
    continue
  fi
  if on_local "$tag"; then
    refuse tag-exists 2 "${tag} exists in this checkout but not on ${REMOTE}" \
      "git tag -d ${tag}, then run this again"
  fi
  tags+=("$tag")
done < <(release_members)
tags+=("$binary_tag")

for tag in "${tags[@]}"; do
  gate_pass "will tag ${tag} on ${local_head:0:12}"
done

# Runners -------------------------------------------------------------------

header "Runners"

# One repository's queued and in-progress runs, as repo, workflow, status, URL.
list_active_runs() {
  local status
  for status in queued in_progress; do
    gh api "repos/$1/actions/runs?status=${status}&per_page=100" \
      --jq '.workflow_runs[] | [.repository.full_name, .name, .status, .html_url] | @tsv' || return 1
  done
}
export -f list_active_runs

if [[ -n "$FORCE" ]]; then
  gate_skip "account runners" "not checked (--force)"
else
  check_err=$(mktemp)
  trap 'rm -f "$check_err"' EXIT
  if ! repos=$(gh api --paginate 'user/repos?type=owner&per_page=100' \
    --jq '.[] | select(.archived | not) | .full_name' 2>"$check_err"); then
    refuse runner-check-failed 2 "could not list the account's repositories: $(tr '\n' ' ' <"$check_err")" \
      "gh auth status, then run this again; or pass --force to push without the check"
  fi
  # shellcheck disable=SC2016  # $1 expands in the child bash, not here
  if ! active=$(printf '%s\n' "$repos" | xargs -r -P 8 -n 1 bash -c 'list_active_runs "$1"' _ 2>"$check_err"); then
    refuse runner-check-failed 2 "could not read the runs of every repository: $(tr '\n' ' ' <"$check_err")" \
      "gh auth status, then run this again; or pass --force to push without the check"
  fi
  if [[ -n "$active" ]]; then
    count=$(grep -c . <<<"$active")
    refuse runners-busy 1 "${count} workflow run(s) hold the account's runners:
$(indent <<<"$active")" \
      "run this again once they finish, or pass --force to push now"
  fi
  gate_pass "no queued or in-progress run in the $(grep -c . <<<"$repos") repositories the account owns"
fi

# Push ----------------------------------------------------------------------

repo=$(gh repo view --json nameWithOwner --jq .nameWithOwner 2>/dev/null || git remote get-url "$REMOTE")

if [[ -n "$DRY_RUN" ]]; then
  print_summary
  printf '\nDry run: would tag %s on %s and push to %s. Nothing was created.\n' \
    "${tags[*]}" "${local_head:0:12}" "$REMOTE"
  exit 0
fi

header "Push"

created=()
for tag in "${tags[@]}"; do
  git tag -a -m "Release ${tag}" "$tag" "$local_head"
  created+=("$tag")
done

refspecs=()
for tag in "${created[@]}"; do
  refspecs+=("refs/tags/${tag}")
done
if ! push_out=$(git push --atomic "$REMOTE" "${refspecs[@]}" 2>&1); then
  git tag -d "${created[@]}" >/dev/null
  refuse push-failed 1 "git push to ${REMOTE} failed, and the tags ${created[*]} were deleted here; ${REMOTE} received none:
$(indent <<<"$push_out")" "fix what git names, then run this again"
fi
gate_pass "pushed ${created[*]} to ${REMOTE}"
print_summary

printf '\nNext:\n\n  1. Watch the runs the tags started:\n'
for tag in "${created[@]}"; do
  printf '       gh run list --repo %s --branch %s\n' "$repo" "$tag"
done
if grep -q '^  workflow_dispatch:' .github/workflows/release.yml 2>/dev/null; then
  printf '       gh run list --repo %s --workflow release.yml --event workflow_dispatch --limit 3\n' "$repo"
fi
printf '  2. When each finishes:\n'
for tag in "${created[@]}"; do
  printf '       scripts/release/postflight.sh --tag %s all\n' "$tag"
done
