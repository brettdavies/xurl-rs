#!/usr/bin/env bash
# generate-completions.sh — Generate or verify shell completions for a Rust CLI.
#
# Usage:
#   ./generate-completions.sh [repo-path]         # generate (default: .)
#   ./generate-completions.sh --check [repo-path]  # verify freshness, exit 1 if stale
#
# Detects the binary name from the CLI package's metadata, builds it in release
# mode, and generates completions for bash, zsh, fish, elvish, and powershell
# into completions/.
#
# Supports two completion interfaces:
#   Standard:     <binary> completions <shell>        (subcommand)
#   Non-standard: <binary> --generate-completion <shell>  (hidden flag, warns)
#
# Requires: cargo, jaq (or jq)

set -euo pipefail

MODE="generate"
REPO_PATH="."

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check) MODE="check"; shift ;;
    *) REPO_PATH="$1"; shift ;;
  esac
done

cd "$REPO_PATH"

# Detect the binary name from the one workspace member that declares a bin
# target; `.packages[0]` is whichever member cargo lists first.
BIN_FILTER='[.packages[].targets[] | select(.kind[] == "bin") | .name] | first // empty'
BIN=$(cargo metadata --no-deps --format-version 1 2>/dev/null \
  | jaq -r "$BIN_FILTER" 2>/dev/null \
  || cargo metadata --no-deps --format-version 1 \
  | jq -r "$BIN_FILTER")

if [[ -z "$BIN" ]]; then
  echo "error: no binary target found in the workspace" >&2
  exit 1
fi

SHELLS=(bash zsh fish elvish powershell)
BINARY="./target/release/$BIN"

# Build if binary is missing or older than any source file
if [[ ! -x "$BINARY" ]] || [[ -n "$(find crates/ Cargo.toml -newer "$BINARY" 2>/dev/null | head -1)" ]]; then
  echo "Building $BIN (release)..."
  cargo build --release --locked --bin "$BIN" 2>&1
fi

# Detect completion interface
COMP_STYLE=""
if "$BINARY" completions bash &>/dev/null; then
  COMP_STYLE="subcommand"
elif "$BINARY" --generate-completion bash &>/dev/null; then
  COMP_STYLE="flag"
  echo "WARNING: $BIN uses --generate-completion (hidden flag), not the standard 'completions' subcommand." >&2
  echo "         Recommend migrating to: $BIN completions <shell>" >&2
  echo "         See: ~/.claude/skills/rust-tool-release/SKILL.md#shell-completions" >&2
  echo "" >&2
else
  echo "error: $BIN has no completions interface." >&2
  echo "       Expected: '$BIN completions <shell>' or '$BIN --generate-completion <shell>'" >&2
  exit 1
fi

# Helper to invoke the detected completions interface
gen() {
  local shell="$1"
  if [[ "$COMP_STYLE" == "subcommand" ]]; then
    "$BINARY" completions "$shell"
  else
    "$BINARY" --generate-completion "$shell"
  fi
}

if [[ "$MODE" == "check" ]]; then
  STALE=0
  for shell in "${SHELLS[@]}"; do
    FILE="completions/$BIN.$shell"
    if [[ ! -f "$FILE" ]]; then
      echo "MISSING: $FILE"
      STALE=1
      continue
    fi
    FRESH=$(gen "$shell")
    if ! diff -q <(echo "$FRESH") "$FILE" &>/dev/null; then
      echo "STALE:   $FILE"
      STALE=1
    else
      echo "OK:      $FILE"
    fi
  done
  if [[ $STALE -ne 0 ]]; then
    echo ""
    echo "Run: $(basename "$0") $(pwd)"
    exit 1
  fi
  echo "All completions are fresh."
else
  mkdir -p completions
  for shell in "${SHELLS[@]}"; do
    gen "$shell" > "completions/$BIN.$shell"
    echo "Generated: completions/$BIN.$shell"
  done
  echo "Done. Commit completions/ when ready."
fi
