#!/usr/bin/env bash
# Diff two X API OpenAPI JSON snapshots and emit a markdown report body
# suitable for posting as a PR comment, a job summary, or showing in a
# terminal during a manual refresh.
#
# Usage:
#   scripts/diff-x-openapi-spec.sh <vendored-json> <upstream-json> [<upstream-label>]
#
# The optional third argument sets the upstream label rendered in the
# report header (e.g. `https://api.x.com/2/openapi.json` when the workflow
# is comparing against the live X spec; the upstream file path is used
# when omitted, which is fine for local manual diffs).
#
# Exits 0 on success regardless of whether the snapshots differ; the
# caller decides what to do with the printed body. Both files must parse
# as JSON and contain `.info.version`.
#
# Callers:
#   - .github/workflows/spec-drift.yml (PR comment + issue body)
#   - scripts/hooks/pre-push (optional drift warning)
#   - manual: scripts/diff-x-openapi-spec.sh vendor/x-api-openapi.json /tmp/upstream.json
#
# Diff shape (in order):
#   - info.version line (silent-bump vs. real bump distinguished).
#   - Path inventory: added/removed path names.
#   - Schemas: added/removed component schema names.
#   - Categorical-value changes: enum-array and discriminator-mapping
#     additions/removals at each location they appear. Catches the
#     in-place "X added enum values to an existing schema" pattern that
#     the path/schema inventories miss.
#   - Refresh-locally footer: rendered only when none of the structural
#     diffs surfaced anything. Means X edited descriptions, types, or
#     other fields the structural diff doesn't model.

set -uo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    echo "Usage: $0 <vendored-json> <upstream-json> [<upstream-label>]" >&2
    exit 2
fi

VENDOR_PATH="$1"
UPSTREAM_PATH="$2"
UPSTREAM_LABEL="${3:-${UPSTREAM_PATH}}"

for path in "${VENDOR_PATH}" "${UPSTREAM_PATH}"; do
    if [ ! -f "${path}" ]; then
        echo "error: ${path} does not exist" >&2
        exit 2
    fi
done

if ! command -v jq >/dev/null 2>&1; then
    echo "error: jq is required" >&2
    exit 3
fi

# ── Scalar extracts ─────────────────────────────────────────────────────
# Values flow through bash variables only, never the command line, so
# spec-controlled strings cannot influence shell expansion.

local_version="$(jq -r '.info.version' "${VENDOR_PATH}")"
upstream_version="$(jq -r '.info.version' "${UPSTREAM_PATH}")"
local_paths_count="$(jq -r '.paths | length' "${VENDOR_PATH}")"
upstream_paths_count="$(jq -r '.paths | length' "${UPSTREAM_PATH}")"

if [ "${local_version}" = "${upstream_version}" ]; then
    version_line="info.version unchanged at \`${local_version}\` (X bumped spec content without bumping the version field)."
else
    version_line="info.version: \`${local_version}\` (vendored) -> \`${upstream_version}\` (upstream)."
fi

# ── Set-difference helpers ──────────────────────────────────────────────

# Emit keys present in $right but not $left, sorted. Both files run
# through jq so the input is JSON-validated.
diff_keys() {
    local left="$1" right="$2" expr="$3"
    local left_tmp right_tmp
    left_tmp="$(mktemp)"
    right_tmp="$(mktemp)"
    jq -r "${expr}" "${left}" | sort > "${left_tmp}"
    jq -r "${expr}" "${right}" | sort > "${right_tmp}"
    comm -13 "${left_tmp}" "${right_tmp}"
    rm -f "${left_tmp}" "${right_tmp}"
}

paths_added="$(diff_keys "${VENDOR_PATH}" "${UPSTREAM_PATH}" '.paths | keys[]')"
paths_removed="$(diff_keys "${UPSTREAM_PATH}" "${VENDOR_PATH}" '.paths | keys[]')"
schemas_added="$(diff_keys "${VENDOR_PATH}" "${UPSTREAM_PATH}" '.components.schemas // {} | keys[]')"
schemas_removed="$(diff_keys "${UPSTREAM_PATH}" "${VENDOR_PATH}" '.components.schemas // {} | keys[]')"

# Categorical-value diff: walks both specs and extracts every location
# where an `enum` array or a `discriminator.mapping` object lives, then
# emits TSV rows (`location\tadded\tremoved`) for locations whose value
# set differs. Catches the in-place "X added enum values to an existing
# schema" pattern that paths + schemas miss.
enum_diff_tsv="$(jq -n -r \
    --slurpfile local "${VENDOR_PATH}" \
    --slurpfile upstream "${UPSTREAM_PATH}" \
    '
    def extract:
      ([paths(type == "object") as $p
        | getpath($p)
        | (
            select(has("enum") and (.enum | type) == "array" and (.enum | length) > 0)
            | { ptr: (($p | map(tostring) | join(".")) + ".enum"),
                values: (.enum | map(tostring)) }
          ),
          (
            select(.discriminator != null and (.discriminator.mapping // null) != null)
            | { ptr: (($p | map(tostring) | join(".")) + ".discriminator.mapping"),
                values: (.discriminator.mapping | keys) }
          )
        ]);

    ($local[0] | extract) as $loc
    | ($upstream[0] | extract) as $ups
    | (reduce $loc[] as $e ({}; .[$e.ptr] = $e.values)) as $loc_map
    | (reduce $ups[] as $e ({}; .[$e.ptr] = $e.values)) as $ups_map
    | (($loc_map | keys) + ($ups_map | keys) | unique) as $all_keys
    | $all_keys[]
    | . as $key
    | ($loc_map[$key] // []) as $l
    | ($ups_map[$key] // []) as $u
    | (($u - $l) | sort) as $added
    | (($l - $u) | sort) as $removed
    | select(($added | length) + ($removed | length) > 0)
    | [$key,
       (if ($added | length) > 0 then ($added | join(",")) else "-" end),
       (if ($removed | length) > 0 then ($removed | join(",")) else "-" end)] | @tsv
    ')"

count_lines() {
    if [ -z "$1" ]; then echo 0; else printf '%s\n' "$1" | wc -l; fi
}
paths_added_count="$(count_lines "${paths_added}")"
paths_removed_count="$(count_lines "${paths_removed}")"
schemas_added_count="$(count_lines "${schemas_added}")"
schemas_removed_count="$(count_lines "${schemas_removed}")"
enum_changes_count="$(count_lines "${enum_diff_tsv}")"

# ── Formatters ──────────────────────────────────────────────────────────

# Format a newline-separated list as a comma-joined inline list of
# backticked identifiers, capped at 10 entries with a `+N more` tail.
fmt_list() {
    local input="$1" count="$2"
    local head
    # shellcheck disable=SC2016 # backticks are literal markdown, not
    # command substitution.
    # awk reads to EOF; head would exit after 10 lines and SIGPIPE the
    # printf under pipefail when the list is large.
    head="$(printf '%s\n' "${input}" | awk 'NR<=10' \
        | sed 's/^/`/; s/$/`/' \
        | paste -sd ',' - \
        | sed 's/,/, /g')"
    if [ "${count}" -gt 10 ]; then
        printf '%s, _+%d more_' "${head}" $(( count - 10 ))
    else
        printf '%s' "${head}"
    fi
}

# Format a comma-joined value list (no newlines) by prefixing each value
# with sign+backtick and suffixing each with backtick. Result joins with
# ", " so adjacent values render legibly in markdown.
fmt_signed_values() {
    local input="$1" sign="$2"
    # shellcheck disable=SC2016 # backticks are literal markdown.
    printf '%s' "${input}" | sed "s|,|\`, ${sign}\`|g; s|^|${sign}\`|; s|\$|\`|"
}

# ── Render the report body ──────────────────────────────────────────────

echo "Vendored \`${VENDOR_PATH}\` does not match upstream \`${UPSTREAM_LABEL}\`."
echo ""
echo "${version_line}"
echo ""
echo "**Path inventory:** ${paths_added_count} added, ${paths_removed_count} removed (${local_paths_count} -> ${upstream_paths_count})."
if [ "${paths_added_count}" -gt 0 ]; then
    echo ""
    echo "Added: $(fmt_list "${paths_added}" "${paths_added_count}")"
fi
if [ "${paths_removed_count}" -gt 0 ]; then
    echo ""
    echo "Removed: $(fmt_list "${paths_removed}" "${paths_removed_count}")"
fi
echo ""
echo "**Schemas:** ${schemas_added_count} added, ${schemas_removed_count} removed."
if [ "${schemas_added_count}" -gt 0 ]; then
    echo ""
    echo "Added: $(fmt_list "${schemas_added}" "${schemas_added_count}")"
fi
if [ "${schemas_removed_count}" -gt 0 ]; then
    echo ""
    echo "Removed: $(fmt_list "${schemas_removed}" "${schemas_removed_count}")"
fi
if [ "${enum_changes_count}" -gt 0 ]; then
    echo ""
    echo "**Categorical-value changes:** ${enum_changes_count} location(s) with enum or discriminator-mapping additions or removals."
    echo ""
    # awk reads to EOF; head would exit after 10 lines and SIGPIPE the
    # printf under pipefail when the drift is large.
    printf '%s\n' "${enum_diff_tsv}" | awk 'NR<=10' | while IFS=$'\t' read -r ptr added removed; do
        line="- \`${ptr}\`"
        sep=": "
        # "-" is the explicit empty marker: tab is IFS whitespace, so an
        # empty middle column would collapse and shift removals into the
        # added slot, rendering removal-only rows with the wrong sign.
        if [ "${added}" != "-" ]; then
            line="${line}${sep}$(fmt_signed_values "${added}" "+")"
            sep=", "
        fi
        if [ "${removed}" != "-" ]; then
            line="${line}${sep}$(fmt_signed_values "${removed}" "-")"
        fi
        echo "${line}"
    done
    if [ "${enum_changes_count}" -gt 10 ]; then
        echo ""
        echo "_+$(( enum_changes_count - 10 )) more_"
    fi
fi

total_structural=$(( paths_added_count + paths_removed_count + schemas_added_count + schemas_removed_count + enum_changes_count ))
if [ "${total_structural}" = "0" ]; then
    echo ""
    echo "Neither paths, schemas, nor categorical values changed but the file still differs. X likely edited descriptions, types, or other in-place fields. Refresh locally to see the field-level diff:"
    echo ""
    echo '```bash'
    echo 'scripts/refresh-x-openapi.sh'
    echo 'git diff vendor/x-api-openapi.json'
    echo '```'
fi
