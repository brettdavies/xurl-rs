#!/usr/bin/env bash
# Generate per-command response schemas to schema/responses/<cmd>.schema.json.
#
# Source of truth is SCHEMA_ENTRIES in src/cli/commands/schema.rs. This script
# enumerates commands via `xr schema --list`, dumps each one, and skips the
# envelope (which lives separately at schema/output.schema.json).
#
# Drift guard: tests/schema_tests.rs asserts byte-equality between the
# committed files and the runtime emitter; regenerate via this script after
# any change to SCHEMA_ENTRIES or any JsonSchema-deriving response type.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "Building xr (release)..."
cargo build --release --quiet --bin xr

XR="./target/release/xr"
out="schema/responses"
mkdir -p "$out"

# Enumerate commands via the canonical list (text mode is one-cmd-per-line).
while IFS= read -r line; do
  cmd=$(awk '{print $1}' <<<"$line")
  [ -z "$cmd" ] && continue
  [ "$cmd" = "envelope" ] && continue
  "$XR" schema "$cmd" --output json > "$out/${cmd}.schema.json"
  echo "Generated: $out/${cmd}.schema.json"
done < <("$XR" schema --list --output text)

echo "Done. Commit schema/responses/ when ready."
