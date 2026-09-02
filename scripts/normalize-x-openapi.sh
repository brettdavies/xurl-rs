#!/usr/bin/env bash
# Canonicalize an X API OpenAPI spec so byte comparison means semantic drift.
#
# The live endpoint serializes OAuth scope arrays in nondeterministic
# per-request order (identical elements, shuffled), so raw bytes differ
# between two fetches of the same spec revision. Scopes are sets, so sorting
# every security requirement's scope list — plus object keys — makes the
# output stable. jq is required (not jaq): the two emit subtly different
# formatting, and both the vendored artifact and the CI drift gate must
# produce byte-identical output.
#
# Usage: scripts/normalize-x-openapi.sh <spec.json>   # canonical form on stdout
set -euo pipefail

if [ "$#" -ne 1 ] || [ ! -f "$1" ]; then
  echo "Usage: $0 <spec.json>" >&2
  exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required" >&2
  exit 3
fi

jq -S 'walk(
  if type == "object" and has("security") and (.security | type == "array") then
    .security |= map(if type == "object" then map_values(if type == "array" then sort else . end) else . end)
  else . end
)' "$1"
