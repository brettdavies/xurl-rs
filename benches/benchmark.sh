#!/usr/bin/env bash
# benchmark.sh — Benchmark xurl-rs vs xurl using hyperfine.
#
# Usage:
#   ./benches/benchmark.sh [--original PATH] [--port PATH] [--output DIR]
#
# Environment:
#   XURL_ORIGINAL_BIN  — path to original xurl binary (default: xurl)
#   XURL_PORT_BIN      — path to xurl-rs binary (default: ./target/release/xurl-rs)
#
# Requires: hyperfine (https://github.com/sharkdp/hyperfine)

set -euo pipefail

# ── Defaults ────────────────────────────────────────────────────────────────

ORIGINAL="${XURL_ORIGINAL_BIN:-xurl}"
PORT="${XURL_PORT_BIN:-./target/release/xurl-rs}"
OUTPUT_DIR="${OUTPUT_DIR:-./benchmark-results}"
WARMUP=3
MIN_RUNS=10

# ── Parse args ──────────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
    case $1 in
        --original) ORIGINAL="$2"; shift 2 ;;
        --port)     PORT="$2"; shift 2 ;;
        --output)   OUTPUT_DIR="$2"; shift 2 ;;
        --warmup)   WARMUP="$2"; shift 2 ;;
        --runs)     MIN_RUNS="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# ── Validate ────────────────────────────────────────────────────────────────

if ! command -v hyperfine &>/dev/null; then
    echo "Error: hyperfine is not installed."
    echo "Install with: cargo install hyperfine"
    exit 1
fi

if ! command -v "$ORIGINAL" &>/dev/null && [[ ! -f "$ORIGINAL" ]]; then
    echo "Error: original binary not found: $ORIGINAL"
    echo "Set XURL_ORIGINAL_BIN or use --original PATH"
    exit 1
fi

if [[ ! -f "$PORT" ]]; then
    echo "Error: port binary not found: $PORT"
    echo "Build with: cargo build --release"
    echo "Or set XURL_PORT_BIN or use --port PATH"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

echo "=== xurl-rs Benchmark Suite ==="
echo "Original: $ORIGINAL ($(command -v "$ORIGINAL" || echo "$ORIGINAL"))"
echo "Port:     $PORT"
echo "Output:   $OUTPUT_DIR"
echo ""

# ── Benchmark 1: Startup time (--version) ──────────────────────────────────

echo "--- Benchmark 1: Startup Time (--version) ---"
hyperfine \
    --warmup "$WARMUP" \
    --min-runs 50 \
    --export-json "$OUTPUT_DIR/startup_time.json" \
    --export-markdown "$OUTPUT_DIR/startup_time.md" \
    --command-name "xurl (Go)" "$ORIGINAL --version" \
    --command-name "xurl-rs (Rust)" "$PORT --version"

echo ""

# ── Benchmark 2: Help text rendering ───────────────────────────────────────

echo "--- Benchmark 2: Help Text Rendering (--help) ---"
hyperfine \
    --warmup "$WARMUP" \
    --min-runs "$MIN_RUNS" \
    --export-json "$OUTPUT_DIR/help_text.json" \
    --export-markdown "$OUTPUT_DIR/help_text.md" \
    --command-name "xurl (Go)" "$ORIGINAL --help" \
    --command-name "xurl-rs (Rust)" "$PORT --help"

echo ""

# ── Benchmark 3: Invalid command (error path) ──────────────────────────────

echo "--- Benchmark 3: Error Path (invalid flag) ---"
hyperfine \
    --warmup "$WARMUP" \
    --min-runs "$MIN_RUNS" \
    --export-json "$OUTPUT_DIR/error_path.json" \
    --export-markdown "$OUTPUT_DIR/error_path.md" \
    --command-name "xurl (Go)" "$ORIGINAL --nonexistent 2>/dev/null || true" \
    --command-name "xurl-rs (Rust)" "$PORT --nonexistent 2>/dev/null || true"

echo ""

# ── Benchmark 4: Subcommand help ───────────────────────────────────────────

echo "--- Benchmark 4: Subcommand Help (search --help) ---"
hyperfine \
    --warmup "$WARMUP" \
    --min-runs "$MIN_RUNS" \
    --export-json "$OUTPUT_DIR/subcommand_help.json" \
    --export-markdown "$OUTPUT_DIR/subcommand_help.md" \
    --command-name "xurl (Go)" "$ORIGINAL search --help" \
    --command-name "xurl-rs (Rust)" "$PORT search --help"

echo ""

# ── Binary size comparison ──────────────────────────────────────────────────

echo "--- Binary Size Comparison ---"
ORIGINAL_SIZE=$(du -h "$(command -v "$ORIGINAL" 2>/dev/null || echo "$ORIGINAL")" 2>/dev/null | cut -f1 || echo "N/A")
PORT_SIZE=$(du -h "$PORT" | cut -f1)

echo "  Original (Go):  $ORIGINAL_SIZE"
echo "  Port (Rust):    $PORT_SIZE"
echo ""

# ── Summary report ──────────────────────────────────────────────────────────

cat > "$OUTPUT_DIR/summary.md" << EOF
# xurl-rs Benchmark Report

**Date:** $(date -Iseconds)
**Original:** $ORIGINAL
**Port:** $PORT

## Binary Size

| Binary | Size |
|--------|------|
| xurl (Go) | $ORIGINAL_SIZE |
| xurl-rs (Rust) | $PORT_SIZE |

## Startup Time (--version)

$(cat "$OUTPUT_DIR/startup_time.md" 2>/dev/null || echo "See startup_time.json")

## Help Text Rendering (--help)

$(cat "$OUTPUT_DIR/help_text.md" 2>/dev/null || echo "See help_text.json")

## Error Path (invalid flag)

$(cat "$OUTPUT_DIR/error_path.md" 2>/dev/null || echo "See error_path.json")

## Subcommand Help (search --help)

$(cat "$OUTPUT_DIR/subcommand_help.md" 2>/dev/null || echo "See subcommand_help.json")
EOF

echo "=== Benchmark Complete ==="
echo "Results written to $OUTPUT_DIR/"
echo "  startup_time.json   — startup time data"
echo "  help_text.json      — help text rendering data"
echo "  error_path.json     — error path data"
echo "  subcommand_help.json — subcommand help data"
echo "  summary.md          — combined report"
