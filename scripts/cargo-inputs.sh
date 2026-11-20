#!/usr/bin/env bash
# Canonical version: 2026.11.05 (rust)
#
# Filter changed paths down to the ones that can change what cargo builds,
# tests, documents, or packages. Reads paths on stdin, one per line, and prints
# the cargo inputs among them in their input order; prints nothing for a change
# that is documentation only.
#
# First, a `!` entry in scripts/cargo-inputs.paths excludes a path and everything
# under it, counted from the repository root. Entries are plain paths, not globs
# (`!docs/**` excludes nothing), and their order in the file does not matter. A
# `!` entry is a repository's way out of a verdict these rules get wrong, so it
# outranks each of them, crates/, README.md, and a file-name match included,
# except that a path entry with more characters than the `!` entry (a path
# literal under rule 2, or a declared path) keeps what it names an input, and a
# tie goes to the `!`. So
# `!docs` beside a test that reads `"docs/matrix.md"` excludes everything under
# docs/ but that file, while a test that names only `"matrix.md"` keeps nothing
# under docs/ unless the repo also declares `docs/matrix.md`. A literal rule 2
# counts as a path is a path entry even when it also matches by file name, so
# `"../AGENTS.md"` (9 characters once normalized) keeps every AGENTS.md under a
# `!` entry of 8 characters or fewer, and loses them to one of 9 or more, such as
# `!docs/arch` or `!docs/archive/old`.
#
# Rules, in the order they apply:
#
#   1. Every path under crates/ is an input, markdown included, because cargo
#      packages a workspace member's files.
#   2. A path some tracked Rust source reads is an input, whatever its type:
#      - its file name matches the file name of a string literal (a test that
#        reads a document, an `include_str!`), so `../AGENTS.md` covers every
#        AGENTS.md. README.md always counts: cargo packages one from a package
#        root even when the manifest names none.
#      - a string literal holding a `/` names the path itself, a directory
#        above it, or a trailing part of it, so a build script that walks
#        `src/spec/` through `fs::read_dir` covers every file under it. The
#        slash is read before normalizing, so a one-part literal such as
#        `"spec/"` or `"./tools"` names its directory. `"/docs"` is spelled the
#        same as a URL path, so a one-part literal that starts with a slash,
#        `"/docs"` or `"/docs/"`, names its directory only within 3 lines of
#        `CARGO_MANIFEST_DIR`, `CARGO_WORKSPACE_DIR` (the workspace root a repo
#        sets in `.cargo/config.toml` `[env]`), `include_str!`,
#        `include_bytes!`, or one of the filesystem calls `fs::`, `File::`,
#        `Path::new`, `PathBuf::`, `read_dir`, `read_to_string`,
#        `canonicalize`, `OpenOptions`, `WalkDir`, and `.join(`, as in
#        `concat!(env!("CARGO_MANIFEST_DIR"), "/docs")`. The anchors are
#        matched as text, so `.join(` on any type, or an anchor's name in a
#        comment, counts too, which errs toward an input. A read through any
#        other call does not count, so a repo declares that directory in
#        scripts/cargo-inputs.paths.
#      - an entry of scripts/cargo-inputs.paths with no `!` does the same. The
#        file is the repository's own: one path or directory per line, `#`
#        comments and blank lines ignored, for an input no literal names (a path
#        assembled at run time).
#      Literals are normalized before matching: leading `/`, `./`, and `../`
#      and trailing `/` are stripped, and an empty result names nothing.
#   3. Any other markdown file is documentation only.
#   4. A few other paths no build, test, or package step reads are
#      documentation too: the gitignored .context/, the license texts, and the
#      .github files no test reads (issue forms, rulesets, the Dependabot
#      config). Rule 2 runs first, so a test that reads one keeps it an input.
#      Workflows stay inputs, since a repo's tests may read them.
#   5. Every other path is an input, including one this script has never seen.
#
# The literals are collected on every run, so a test that starts reading a
# file is covered the day it lands. The pre-push hook and CI's change-scope job
# skip the Rust battery on this verdict, and a wrong skip costs more than a
# wrong run, which is why matching errs toward an input.
#
# Exit codes: 0 = filtered (the output may be empty). Any non-zero exit means
# the classifier failed and its output is no verdict: 2 outside a git work tree,
# and otherwise the failing step's own code, such as 2 from GNU sed on an
# unreadable scripts/cargo-inputs.paths or 123 from GNU xargs when a tracked Rust
# source cannot be read; other implementations may use other codes. The pre-push
# hook (`needs_cargo` in github-repo-setup's hooks library) and CI's change-scope
# job then run the Rust battery, whatever the code.
set -euo pipefail

root=$(git rev-parse --show-toplevel 2>/dev/null) || {
  echo "cargo-inputs: not inside a git work tree" >&2
  exit 2
}
cd "$root"

# Strips leading `/`, `./`, `../` and trailing `/` from each line, and drops
# lines left empty.
normalize() {
  sed -E -e ':a' -e 's#^\.{0,2}/##' -e 'ta' -e 's#/+$##' | grep -v '^$' || true
}

# Prints the one-part `"/x"` literals on a line within 3 lines of a path-root
# variable or something that reads the filesystem. Three reaches across a call
# rustfmt breaks over several lines, in either order. A literal further from its read needs
# scripts/cargo-inputs.paths.
readonly ROOTED_READS='
  function flush(   i, j) {
    for (i = 1; i <= n; i++) {
      if (!(i in found)) continue
      for (j = i - 3; j <= i + 3; j++) {
        if (j in reads) {
          printf "%s", found[i]
          break
        }
      }
    }
    split("", found)
    split("", reads)
    n = 0
  }
  FNR == 1 { flush() }
  {
    n = FNR
    if ($0 ~ /CARGO_MANIFEST_DIR|CARGO_WORKSPACE_DIR|include_str!|include_bytes!|fs::|File::|Path::new|PathBuf::|read_dir|read_to_string|canonicalize|OpenOptions|WalkDir|\.join\(/) reads[FNR] = 1
    line = $0
    while (match(line, /"\/+[A-Za-z0-9_.-]+\/*"/)) {
      found[FNR] = found[FNR] substr(line, RSTART + 1, RLENGTH - 2) "\n"
      line = substr(line, RSTART + RLENGTH)
    }
  }
  END { flush() }
'

raw=$(git ls-files -z -- '*.rs' | xargs -0 -r grep -hoE '"[A-Za-z0-9_./-]+"' | tr -d '"') || true
rooted=$(git ls-files -z -- '*.rs' | xargs -0 -r awk "$ROOTED_READS")
literals=$(printf '%s\n' "$raw" | normalize)
paths=$({
  printf '%s\n' "$raw" | grep / | grep -vE '^/+[^/]+/*$' || true
  printf '%s\n' "$rooted"
} | normalize)

declared=''
excluded=''
if [ -f scripts/cargo-inputs.paths ]; then
  entries=$(sed -e 's/#.*//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' scripts/cargo-inputs.paths)
  declared=$(printf '%s\n' "$entries" | { grep -v '^!' || true; } | normalize)
  excluded=$(printf '%s\n' "$entries" | sed -n 's/^![[:space:]]*//p' | normalize)
fi

# The read set, one entry per line, tagged `N ` for a file name, `P ` for a path
# or directory, and `X ` for an exclusion. Each entry carries its own kind rather
# than sitting below a separator line, because any separator could also be a
# literal (`"--"` is a common one).
read_set() {
  printf 'N README.md\n'
  printf '%s\n' "$literals" | sed -e 's#.*/##' -e 's/^/N /'
  printf '%s\n' "$paths" | LC_ALL=C sort -u | sed 's/^/P /'
  printf '%s\n' "$declared" | sed 's/^/P /'
  printf '%s\n' "$excluded" | sed 's/^/X /'
}

awk '
  FNR == NR {
    entry = substr($0, 3)
    if (entry == "") next
    kind = substr($0, 1, 1)
    if (kind == "N") name[entry] = 1
    else if (kind == "X") excluded[++m] = entry
    else if (!(entry in seen)) { seen[entry] = 1; read_path[++n] = entry }
    next
  }
  function at_or_under(p, d) {
    return p == d || (length(p) > length(d) && substr(p, 1, length(d) + 1) == d "/")
  }
  function named_by(p, d) {
    return at_or_under(p, d) || (length(p) > length(d) && substr(p, length(p) - length(d)) == "/" d)
  }
  function excluded_by_repo(p,   i, longest) {
    longest = 0
    for (i = 1; i <= m; i++) {
      if (length(excluded[i]) > longest && at_or_under(p, excluded[i])) longest = length(excluded[i])
    }
    if (longest == 0) return 0
    for (i = 1; i <= n; i++) {
      if (length(read_path[i]) > longest && named_by(p, read_path[i])) return 0
    }
    return 1
  }
  function read_by_rust(p,   base, i) {
    base = p
    sub(/.*\//, "", base)
    if (base in name) return 1
    for (i = 1; i <= n; i++) {
      if (named_by(p, read_path[i])) return 1
    }
    return 0
  }
  $0 == "" { next }
  excluded_by_repo($0) { next }
  /^crates\// { print; next }
  read_by_rust($0) { print; next }
  /\.md$/ { next }
  /^(\.context\/|LICENSE-|\.github\/ISSUE_TEMPLATE\/|\.github\/rulesets\/)/ { next }
  $0 == ".github/dependabot.yml" { next }
  { print }
' <(read_set) -
