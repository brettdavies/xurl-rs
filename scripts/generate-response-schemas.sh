#!/usr/bin/env bash
# Generate per-command response schemas to schema/responses/<file>.
#
# Source of truth is SCHEMA_ENTRIES in crates/xurl-cli/src/cli/commands/schema/mod.rs. This script
# reads the records `xr schema --list --output tsv` prints, one per registered command, and dumps
# each command's schema into the file its record names. The envelope lives separately at
# schema/output.schema.json.
#
# Drift guard: crates/xurl-cli/tests/schema_tests.rs asserts byte-equality between the
# committed files and the runtime emitter; regenerate via this script after
# any change to SCHEMA_ENTRIES or any JsonSchema-deriving response type.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "Building xr (release)..."
cargo build --release --quiet --bin xr

XR="./target/release/xr"
out="schema/responses"
mkdir -p "$out"

# Command names hold spaces, so columns are read by header name, never split on whitespace.
"$XR" schema --list --output tsv | {
  IFS=$'\t' read -r -a header
  for i in "${!header[@]}"; do
    case "${header[$i]}" in
      command) command_col=$i ;;
      file) file_col=$i ;;
    esac
  done
  while IFS=$'\t' read -r -a row; do
    command="${row[$command_col]}"
    file="${row[$file_col]}"
    read -r -a words <<<"$command"
    "$XR" schema --output json "${words[@]}" >"$out/$file"
    echo "Generated: $out/$file"
  done
}

echo "Done. Commit schema/responses/ when ready."
