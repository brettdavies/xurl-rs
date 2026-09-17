#!/usr/bin/env bash
# Output discipline guard.
#
# Two invariants, both checked over `src/`:
#
# 1. No `println!`, `eprintln!`, `print!`, or `eprint!` macro appears outside
#    `src/cli/output/mod.rs`: every printed line goes through `OutputConfig`.
# 2. No library module reaches a terminal handle: `std::io::stdout()` and
#    `std::io::stderr()` appear only under `src/cli/`, where the binary owns
#    them. Library diagnostics are `tracing` events.
#
# Doc comments (`///`, `//!`) and ordinary comments (`//`, `/* */`) are
# ignored so prose references remain readable.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

NOT_COMMENT='^(?!\s*///|\s*//!|\s*//|\s*\*)'

# `src` is passed explicitly: with no path and a non-terminal stdin, rg
# searches stdin instead of the tree and blocks until it closes.
#
# The exemption names each file holding a stdio macro rather than the whole
# `src/cli/output/` directory. `OutputConfig` owns no I/O handles: its print
# methods take `&mut dyn Write`, so a sibling in that module reaches its
# caller's writer without a naked macro, and a sibling that trips this guard
# is bypassing that writer rather than needing an exemption.
macros=$(
  rg \
    --glob 'src/**/*.rs' \
    --glob '!src/cli/output/mod.rs' \
    --no-heading \
    --line-number \
    --pcre2 \
    "${NOT_COMMENT}.*\b(println|eprintln|print|eprint)!" \
    src \
    || true
)

if [[ -n "$macros" ]]; then
  echo "FAIL: naked println!/eprintln!/print!/eprint! outside src/cli/output/mod.rs:" >&2
  echo "$macros" >&2
  echo "" >&2
  echo "Route through OutputConfig::{info,status,verbose,warning,progress,print_*}." >&2
  exit 1
fi

handles=$(
  rg \
    --glob 'src/**/*.rs' \
    --glob '!src/cli/**' \
    --no-heading \
    --line-number \
    --pcre2 \
    "${NOT_COMMENT}.*\bio::(stdout|stderr)\(\)" \
    src \
    || true
)

if [[ -n "$handles" ]]; then
  echo "FAIL: terminal handle reached from library code outside src/cli/:" >&2
  echo "$handles" >&2
  echo "" >&2
  echo "Emit a tracing event and let the binary's subscriber render it." >&2
  exit 1
fi

echo "OK: no naked stdio macros outside src/cli/output/mod.rs, no terminal handles outside src/cli/"
