#!/usr/bin/env bash
# update-types.sh — Fetch the latest X API v2 OpenAPI spec, optionally run
# cargo-typify to generate reference types, and diff against hand-written types.
#
# Usage:
#   scripts/update-types.sh              # Fetch spec + diff (no typify)
#   scripts/update-types.sh --typify     # Fetch spec + generate + diff
#
# The cached spec is stored at tests/fixtures/openapi/ for CI reproducibility.

set -euo pipefail

SPEC_URL="https://raw.githubusercontent.com/xdevplatform/twitter-api-openapi/main/openapi/openapi.json"
SPEC_DIR="tests/fixtures/openapi"
SPEC_FILE="${SPEC_DIR}/twitter-api-openapi.json"
TYPES_FILE="src/api/response/types.rs"

echo "=== update-types.sh ==="

# Fetch latest spec
echo "Fetching latest X API v2 OpenAPI spec..."
mkdir -p "${SPEC_DIR}"
if curl -sSfL "${SPEC_URL}" -o "${SPEC_FILE}.tmp" 2>/dev/null; then
    mv "${SPEC_FILE}.tmp" "${SPEC_FILE}"
    echo "  Saved to ${SPEC_FILE}"
else
    echo "  Warning: Could not fetch spec from ${SPEC_URL}"
    echo "  Using cached copy if available."
    rm -f "${SPEC_FILE}.tmp"
fi

if [ ! -f "${SPEC_FILE}" ]; then
    echo "  No spec file available. Skipping diff."
    exit 0
fi

# Optionally run cargo-typify
if [ "${1:-}" = "--typify" ]; then
    if command -v cargo-typify &>/dev/null; then
        echo "Running cargo-typify..."
        GENERATED="/tmp/xurl-typify-generated.rs"
        cargo typify "${SPEC_FILE}" -o "${GENERATED}" 2>/dev/null || true
        if [ -f "${GENERATED}" ]; then
            echo ""
            echo "=== Diff: generated types vs hand-written types ==="
            diff -u "${GENERATED}" "${TYPES_FILE}" || true
            echo ""
            echo "Generated types saved to: ${GENERATED}"
        else
            echo "  cargo-typify produced no output."
        fi
    else
        echo "  cargo-typify not installed. Install with: cargo install cargo-typify"
        echo "  Skipping type generation."
    fi
fi

# Show spec stats
echo ""
echo "=== Spec summary ==="
if command -v jaq &>/dev/null; then
    echo "  Endpoints: $(jaq '.paths | length' "${SPEC_FILE}" 2>/dev/null || echo 'N/A')"
    echo "  Schemas: $(jaq '.components.schemas | length' "${SPEC_FILE}" 2>/dev/null || echo 'N/A')"
else
    echo "  (Install jaq for spec stats)"
fi

echo ""
echo "Review the spec and promote new fields from 'extra' to named struct fields as needed."
echo "Then run: cargo test --test spec_validation"
