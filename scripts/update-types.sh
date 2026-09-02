#!/usr/bin/env bash
# update-types.sh — Summarize the vendored X API OpenAPI spec for the manual
# typed-response review pass.
#
# Usage:
#   scripts/update-types.sh
#
# Reads vendor/x-api-openapi.json — the same vendored snapshot build.rs
# consumes for auth-matrix codegen. scripts/refresh-x-openapi.sh is the only
# place the spec is fetched; this script never touches the network, so the
# summary always describes the spec revision the crate actually builds
# against.
#
# The hand-written types in src/api/response/types.rs are reviewed by hand:
# cargo-typify consumes JSON Schema documents, not OpenAPI, so generating
# reference types from the spec produces a typeless stub. Field-level drift
# review runs against the structural report from
# scripts/diff-x-openapi-spec.sh instead.

set -euo pipefail

cd "$(dirname "$0")/.."

SPEC_FILE="vendor/x-api-openapi.json"

echo "=== update-types.sh ==="

if [ ! -f "${SPEC_FILE}" ]; then
    echo "error: ${SPEC_FILE} not found — run scripts/refresh-x-openapi.sh first" >&2
    exit 1
fi

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
