#!/usr/bin/env bash
# update-types.sh — Diff the hand-written response types against reference
# types generated from the vendored X API OpenAPI spec.
#
# Usage:
#   scripts/update-types.sh              # spec summary only
#   scripts/update-types.sh --typify     # generate reference types + diff
#
# Reads vendor/x-api-openapi.json — the same vendored snapshot build.rs
# consumes for auth-matrix codegen. scripts/refresh-x-openapi.sh is the only
# place the spec is fetched; this script never touches the network, so the
# diff always describes the spec revision the crate actually builds against.
#
# The generated types are a reference for a human pass, never a drop-in:
# the hand-written types in src/api/response/types.rs carry serde attrs,
# extra-field capture, and docs that generation would destroy.

set -euo pipefail

cd "$(dirname "$0")/.."

SPEC_FILE="vendor/x-api-openapi.json"
TYPES_FILE="src/api/response/types.rs"

echo "=== update-types.sh ==="

if [ ! -f "${SPEC_FILE}" ]; then
    echo "error: ${SPEC_FILE} not found — run scripts/refresh-x-openapi.sh first" >&2
    exit 1
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
