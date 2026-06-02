#!/usr/bin/env bash
# Output discipline guard.
#
# Fails when any `println!`, `eprintln!`, `print!`, or `eprint!` macro
# appears outside `src/output.rs`. The single-owner invariant is documented
# in the project plan as KTD4 and in the corpus best-practice doc
# `cli-unified-log-module-with-no-color-support-2026-04-20.md`.
#
# Doc comments (`///`, `//!`) and ordinary comments (`//`, `/* */`) are
# ignored so prose references to the macros remain readable.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Strip line and block comments before searching. The Rust source files are
# small enough that doing this with a one-pass awk avoids a separate
# pre-processing step.
matches=$(
  rg \
    --glob 'src/**/*.rs' \
    --glob '!src/output.rs' \
    --no-heading \
    --line-number \
    --pcre2 \
    '^(?!\s*///|\s*//!|\s*//|\s*\*).*\b(println|eprintln|print|eprint)!' \
    || true
)

if [[ -n "$matches" ]]; then
  echo "FAIL: naked println!/eprintln!/print!/eprint! outside src/output.rs:" >&2
  echo "$matches" >&2
  echo "" >&2
  echo "Route through OutputConfig::{info,status,verbose,warning,progress,print_*} or crate::output::warn_stderr." >&2
  exit 1
fi

echo "OK: no naked stdio macros outside src/output.rs"
