#!/usr/bin/env bash
# Emits one ERE matching every path kept off `main`, for `grep -E`.
#
# `guard-main-docs` rejects two sets on a PR to main: a base list hardcoded in
# the reusable workflow, and this repo's `extra_paths` in
# .github/workflows/guard-main-docs.yml. Every local screen that kept its own
# copy of that union drifted from the workflow, because nothing tied the copies
# to the source: a path the workflow guards but a copy omits passes the local
# check and reaches an open release with a green `guard-docs`.
#
# The `extra_paths` half is read from the workflow here, so registering a path
# there is the only edit a new guarded path needs.
#
# Usage:
#   GUARDED="$(scripts/release-guarded-paths.sh)"
#   git diff origin/main..HEAD --name-only | grep -E "$GUARDED"

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORKFLOW="$REPO_ROOT/.github/workflows/guard-main-docs.yml"

# Hardcoded in brettdavies/.github/.github/workflows/guard-main-docs.yml, which
# is a different repo and cannot be read from here. This is the one list that
# still needs a manual edit when the reusable changes.
REUSABLE_BASE=(
    'docs/architecture/'
    'docs/brainstorms/'
    'docs/ideation/'
    'docs/plans/'
    'docs/research/'
    'docs/reviews/'
    'docs/solutions/'
)

# Gitignored, so it cannot be committed by accident. Screened anyway: `git add
# -f` defeats the ignore, and the cost of catching that here is one array entry.
LOCAL_ONLY=('.context/')

# `extra_paths: 'a/,b.txt,c/'` -> one path per line.
read_extra_paths() {
    [[ -f "$WORKFLOW" ]] || return 0
    sed -n "s/^[[:space:]]*extra_paths:[[:space:]]*'\(.*\)'[[:space:]]*\$/\1/p" "$WORKFLOW" \
        | tr ',' '\n'
}

# A path is a literal, not a pattern: escape every ERE metacharacter so
# `.vale.ini` cannot also match `Xvale!ini`.
escape_ere() { sed 's/[][\.^$*+?(){}|\\]/\\&/g'; }

frags=()
declare -A seen=()
for path in "${REUSABLE_BASE[@]}" "${LOCAL_ONLY[@]}" $(read_extra_paths); do
    path="${path#"${path%%[![:space:]]*}"}"
    path="${path%"${path##*[![:space:]]}"}"
    [[ -n "$path" ]] || continue
    # `extra_paths` may repeat a path the reusable's base already covers.
    [[ -n "${seen[$path]:-}" ]] && continue
    seen[$path]=1
    escaped=$(printf '%s' "$path" | escape_ere)
    # A trailing slash means "this directory and everything under it"; anything
    # else names one exact file, which must not match a longer sibling path.
    if [[ "$path" == */ ]]; then
        frags+=("^${escaped}")
    else
        frags+=("^${escaped}\$")
    fi
done

if [[ ${#frags[@]} -eq 0 ]]; then
    echo "guarded-paths: no guarded paths resolved (is $WORKFLOW present?)" >&2
    exit 1
fi

printf '(%s)\n' "$(
    IFS='|'
    echo "${frags[*]}"
)"
