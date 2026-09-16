#!/usr/bin/env bash
# Output discipline guard.
#
# Fails when any `println!`, `eprintln!`, `print!`, or `eprint!` macro
# appears outside `src/cli/output/mod.rs`. The single-owner invariant is
# documented in the project plan as KTD4 and in the corpus best-practice doc
# `cli-unified-log-module-with-no-color-support-2026-04-20.md`.
#
# Doc comments (`///`, `//!`) and ordinary comments (`//`, `/* */`) are
# ignored so prose references to the macros remain readable.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# `src` is passed explicitly: with no path and a non-terminal stdin, rg
# searches stdin instead of the tree and blocks until it closes.
#
# The exemption names each file holding a stdio macro rather than the whole
# `src/cli/output/` directory. `OutputConfig` owns no I/O handles — its print
# methods take `&mut dyn Write` — so a sibling in that module reaches its
# caller's writer without a naked macro, and a sibling that trips this guard
# is bypassing that writer rather than needing an exemption.
matches=$(
  rg \
    --glob 'src/**/*.rs' \
    --glob '!src/cli/output/mod.rs' \
    --no-heading \
    --line-number \
    --pcre2 \
    '^(?!\s*///|\s*//!|\s*//|\s*\*).*\b(println|eprintln|print|eprint)!' \
    src \
    || true
)

if [[ -n "$matches" ]]; then
  echo "FAIL: naked println!/eprintln!/print!/eprint! outside src/cli/output/mod.rs:" >&2
  echo "$matches" >&2
  echo "" >&2
  echo "Route through OutputConfig::{info,status,verbose,warning,progress,print_*} or crate::cli::output::warn_stderr." >&2
  exit 1
fi

echo "OK: no naked stdio macros outside src/cli/output/mod.rs"
