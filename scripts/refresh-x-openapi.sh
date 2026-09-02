#!/usr/bin/env bash
# Refresh the vendored X API OpenAPI spec at vendor/x-api-openapi.json.
#
# Usage:
#   scripts/refresh-x-openapi.sh
#
# Downloads the current spec from https://api.x.com/2/openapi.json, replaces
# vendor/x-api-openapi.json, and updates vendor/README.md with the new refresh
# date, upstream `info.version`, file size, and SHA256.
#
# Run before each release cycle. CI drift-check (.github/workflows/spec-drift.yml)
# flags divergence between runs.
#
# Exit codes:
#   0  spec fetched and vendored, README updated
#   1  curl failed (network down, 5xx, non-zero size)
#   2  fetched file is not valid JSON
#   3  required tools missing

set -euo pipefail

UPSTREAM_URL="https://api.x.com/2/openapi.json"
VENDOR_PATH="vendor/x-api-openapi.json"
README_PATH="vendor/README.md"
METADATA_PATH="vendor/spec-metadata.json"

# Resolve repo root from this script's location, so the script works from any cwd.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

# Tool gating
for tool in curl sha256sum stat; do
    if ! command -v "${tool}" >/dev/null 2>&1; then
        echo "error: required tool not found: ${tool}" >&2
        exit 3
    fi
done

# Prefer jaq, fall back to jq for `.info.version` extraction. Both produce raw output.
if command -v jaq >/dev/null 2>&1; then
    json_get='jaq -r'
elif command -v jq >/dev/null 2>&1; then
    json_get='jq -r'
else
    echo "error: required tool not found: jaq or jq" >&2
    exit 3
fi

# Fetch into a temp file so a failed download never half-writes the vendored copy.
TMP_FILE="$(mktemp -t x-api-openapi.XXXXXX.json)"
trap 'rm -f "${TMP_FILE}"' EXIT

echo "==> Fetching ${UPSTREAM_URL}"
if ! curl -sS --fail --max-time 60 -o "${TMP_FILE}" "${UPSTREAM_URL}"; then
    echo "error: failed to fetch ${UPSTREAM_URL}" >&2
    exit 1
fi

if [ ! -s "${TMP_FILE}" ]; then
    echo "error: fetched file is empty" >&2
    exit 1
fi

# Validate it parses as JSON before replacing the vendored copy.
if ! ${json_get} '.info.version' "${TMP_FILE}" >/dev/null 2>&1; then
    echo "error: fetched file is not valid JSON or missing .info.version" >&2
    exit 2
fi

# Canonicalize before vendoring so refresh diffs and the CI drift gate
# never see the endpoint's nondeterministic scope-array ordering.
"${SCRIPT_DIR}/normalize-x-openapi.sh" "${TMP_FILE}" > "${TMP_FILE}.norm"
mv "${TMP_FILE}.norm" "${TMP_FILE}"

mv "${TMP_FILE}" "${VENDOR_PATH}"
# mktemp creates the temp file with 600 permissions; the vendored copy is a
# checked-in artifact and should be world-readable.
chmod 644 "${VENDOR_PATH}"
trap - EXIT

# Capture metadata for the README.
SPEC_VERSION="$(${json_get} '.info.version' "${VENDOR_PATH}")"
SPEC_PATH_COUNT="$(${json_get} '.paths | length' "${VENDOR_PATH}")"
SPEC_SHA256="$(sha256sum "${VENDOR_PATH}" | awk '{print $1}')"
SPEC_SIZE_BYTES="$(stat -c %s "${VENDOR_PATH}" 2>/dev/null || stat -f %z "${VENDOR_PATH}")"
REFRESH_DATE="$(date -u +"%Y-%m-%d")"

# Vendor the metadata alongside the spec so build.rs reads provenance from
# the repo, not from git context. This makes the metadata correct for
# uncommitted-refresh local builds and for crates.io tarball installs.
cat > "${METADATA_PATH}" <<EOF
{
  "info_version": "${SPEC_VERSION}",
  "content_sha256": "${SPEC_SHA256}",
  "refreshed_at": "${REFRESH_DATE}",
  "source_url": "${UPSTREAM_URL}"
}
EOF
chmod 644 "${METADATA_PATH}"

cat > "${README_PATH}" <<EOF
# Vendored X API OpenAPI Spec

This directory contains a checked-in copy of X's public OpenAPI spec, used at
build time to generate the auth-method matrix in \`src/api/auth_matrix.rs\` (see
\`build.rs\`).

## Provenance

| Field              | Value                                  |
| ------------------ | -------------------------------------- |
| Upstream URL       | ${UPSTREAM_URL} |
| Spec \`info.version\` | ${SPEC_VERSION}                          |
| Path count         | ${SPEC_PATH_COUNT}                                    |
| File size          | ${SPEC_SIZE_BYTES} bytes                            |
| SHA256             | \`${SPEC_SHA256}\` |
| Refreshed (UTC)    | ${REFRESH_DATE}                             |

## Refresh

Run from the repo root before each release cycle:

\`\`\`bash
scripts/refresh-x-openapi.sh
\`\`\`

The script downloads the current spec, validates it as JSON, replaces this
directory's copy, and rewrites this README. CI drift-check
(\`.github/workflows/spec-drift.yml\`) flags divergence between runs and posts
either a job summary, a PR comment, or a tracked issue depending on the
trigger.

## Why vendor?

A checked-in spec gives reproducible builds (Homebrew bottle CI, offline
builds), an auditable supply chain (the spec is greppable from source), and CI
that does not need to reach \`api.x.com\` on every push. Manual refresh is the
trade-off we accept; the drift-check workflow shortens time-to-notice.

See \`docs/brainstorms/2026-06-04-001-auth-method-enforcement-requirements.md\`
and \`docs/plans/2026-06-04-001-feat-auth-method-enforcement-plan.md\` for the
full rationale.
EOF

echo "==> Vendored ${VENDOR_PATH}"
echo "    info.version : ${SPEC_VERSION}"
echo "    path count   : ${SPEC_PATH_COUNT}"
echo "    size         : ${SPEC_SIZE_BYTES} bytes"
echo "    sha256       : ${SPEC_SHA256}"
echo "==> Wrote ${METADATA_PATH}"
echo "==> Wrote ${README_PATH}"
