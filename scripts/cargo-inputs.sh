#!/usr/bin/env bash
# Filter changed paths down to the ones that can change what cargo builds,
# tests, documents, or packages. Reads paths on stdin, one per line, and
# prints the cargo inputs among them in their input order; prints nothing
# for a change that is documentation only.
#
# A path is documentation only when it is markdown, sits outside crates/, and
# no Rust source under crates/ names its file in a string literal. The last
# clause keeps a guard test's subject in the battery: `recipe_guard` reads
# AGENTS.md, the README landing test reads the root README.md, the
# surface-bump test reads the PR template, and `include_str!("../README.md")`
# makes the library README its crate doc. The names are collected from the
# source on every run, so a test that starts reading a document is covered the
# day it lands. Matching is by file name, so `../README.md` covers every
# README.md.
#
# Every other path is a cargo input, including one this script has never
# seen. The pre-push hook and CI skip their Rust battery on this verdict, and
# a wrong skip costs more than a wrong run.
#
# Exit codes: 0 = filtered (the output may be empty), 2 = no crates/ to read.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)" || exit 2
[ -d crates ] || {
  echo "cargo-inputs: no crates/ directory to collect document names from" >&2
  exit 2
}

# The file name of every quoted literal ending in `.md` under crates/.
read_by_rust=$(
  grep -rhoE --include='*.rs' '"[A-Za-z0-9_./-]*\.md"' crates \
    | tr -d '"' | sed 's#.*/##' | LC_ALL=C sort -u
) || true

while IFS= read -r path; do
  [ -n "$path" ] || continue
  case "$path" in
    crates/*) ;;
    *.md) grep -qxF -- "${path##*/}" <<<"$read_by_rust" || continue ;;
  esac
  printf '%s\n' "$path"
done
