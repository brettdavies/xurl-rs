#!/usr/bin/env bash
# Render vendor/README.md from the vendored spec and its metadata sidecar.
#
# Usage:
#   scripts/render-vendor-readme.sh > vendor/README.md
#
# Reads vendor/spec-metadata.json (info.version, content SHA256, refresh
# date, upstream URL) and vendor/x-api-openapi.json (path count, byte size)
# and prints the README to stdout. Every value comes from the vendored
# artifacts, so the output is reproducible offline and tests/spec_scripts.rs
# holds the committed README to it.
#
# Exit codes:
#   0  README rendered
#   1  vendored artifacts missing, or the sidecar SHA256 does not match the
#      spec (stale sidecar; re-run scripts/refresh-x-openapi.sh)
#   3  required tools missing

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
VENDOR_PATH="${REPO_ROOT}/vendor/x-api-openapi.json"
METADATA_PATH="${REPO_ROOT}/vendor/spec-metadata.json"

for tool in jq sha256sum stat; do
    if ! command -v "${tool}" >/dev/null 2>&1; then
        echo "error: required tool not found: ${tool}" >&2
        exit 3
    fi
done

for path in "${VENDOR_PATH}" "${METADATA_PATH}"; do
    if [ ! -f "${path}" ]; then
        echo "error: ${path} not found; run scripts/refresh-x-openapi.sh" >&2
        exit 1
    fi
done

upstream_url="$(jq -r '.source_url' "${METADATA_PATH}")"
spec_version="$(jq -r '.info_version' "${METADATA_PATH}")"
spec_sha256="$(jq -r '.content_sha256' "${METADATA_PATH}")"
refresh_date="$(jq -r '.refreshed_at' "${METADATA_PATH}")"
path_count="$(jq -r '.paths | length' "${VENDOR_PATH}")"
size_bytes="$(stat -c %s "${VENDOR_PATH}" 2>/dev/null || stat -f %z "${VENDOR_PATH}")"

actual_sha256="$(sha256sum "${VENDOR_PATH}" | awk '{print $1}')"
if [ "${actual_sha256}" != "${spec_sha256}" ]; then
    echo "error: vendor/spec-metadata.json content_sha256 does not match vendor/x-api-openapi.json; run scripts/refresh-x-openapi.sh" >&2
    exit 1
fi

# Column widths fit the widest cells (the `Spec \`info.version\`` label and
# the backticked SHA256), so every refresh renders an aligned table.
row() {
    printf '| %-19s | %-66s |\n' "$1" "$2"
}

cat <<EOF
# Vendored X API OpenAPI Spec

This directory contains a checked-in copy of X's public OpenAPI spec, used at build time to generate the auth-method
matrix in \`src/api/auth_matrix.rs\` (see \`build.rs\`).

## Provenance

EOF
row "Field" "Value"
row "-------------------" "------------------------------------------------------------------"
row "Upstream URL" "${upstream_url}"
row "Spec \`info.version\`" "${spec_version}"
row "Path count" "${path_count}"
row "File size" "${size_bytes} bytes"
row "SHA256" "\`${spec_sha256}\`"
row "Refreshed (UTC)" "${refresh_date}"
cat <<'EOF'

## Refresh

Run from the repo root before each release cycle:

```bash
scripts/refresh-x-openapi.sh
```

The script downloads the current spec, validates it as JSON, replaces this directory's copy, and rewrites this README
through `scripts/render-vendor-readme.sh`. The CI drift check (`.github/workflows/spec-drift.yml`) flags divergence
between runs and, depending on the trigger, writes a job summary, comments on the PR, or opens a refresh PR to `dev`.

## Why vendor?

A checked-in spec gives reproducible builds (Homebrew bottle CI, offline builds), an auditable supply chain (the spec is
greppable from source), and CI that does not need to reach `api.x.com` on every push. Manual refresh is the trade-off we
accept; the drift-check workflow shortens time-to-notice.
EOF
