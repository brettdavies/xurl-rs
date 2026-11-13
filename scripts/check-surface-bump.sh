#!/usr/bin/env bash
# Canonical version: 2026.11.12 (rust)
#
# Fail a pull request whose changelog block uses a heading outside the six, or
# whose CLI surface grows while the block names no minor-or-higher section (or,
# for growth in the schemas alone, no `### Fixed` bullet), or shrinks without a
# `### Breaking changes` bullet starting "Remove".
#
# Each PR's changelog sections decide the release's version bump
# (github-repo-setup's references/versioning.md), and an addition to the
# contract is a minor. An addition filed under `### Changed`
# reads as a patch and nothing in review makes it stand out, so this compares
# the surface two trees describe:
#
#   - every command, subcommand, flag, and enumerated flag value, read from
#     completions/<BIN>.bash, which the repo's completions freshness check
#     holds equal to the clap definition;
#   - every leaf of every JSON schema under SCHEMA_DIR except `description`,
#     `title`, `$id`, and `examples`, so a new envelope key, `reason`, `action`,
#     or response field counts, and reworded prose or a versioned `$id` does not,
#     plus every key declared under `properties`, `patternProperties`, `$defs`, or
#     `definitions`. A repo without the directory compares the CLI alone.
#
# Every `###` heading in the block must be one of the six, and no deeper or indented
# heading may appear, whether or not the surface changed: the release's generator
# files any other `###` as a patch and reads neither of the others, so a removal
# under any of them would ship without its major. Growth passes when
# the block has a bullet under `### Added`, `### Deprecated`, or
# `### Breaking changes`. Growth in the schemas alone also passes with a bullet
# under `### Fixed`, a patch, for a schema catching up to output the binary
# already prints and its docs already describe; a new command, flag, or flag
# value still needs one of the three, since the completions grow only when the
# binary does. A removal passes only with a `### Breaking changes`
# bullet starting "Remove", so an unrelated bullet elsewhere cannot satisfy it. A
# rename or retype reads as a removal and an addition. Env vars, exit codes, and
# the store format have no generated artifact and are not compared.
#
# This repository reads the `xr` block, `## Changelog (xurl-rs)`, only. The
# `xdk-rs` block is not read: the library's surface is its Rust API, which
# cargo-semver-checks compares in preflight's api-contract gate.
#
# Usage:
#   check-surface-bump.sh surface <tree>
#   check-surface-bump.sh check <base-tree> <head-tree> <pr-body-file>
#   check-surface-bump.sh --help
#
# Skeleton: set the CUSTOMIZE values below for this repository. The changelog
# block is the one generate-changelog.py reads: `## Changelog`, or the heading
# MANIFEST's [package.metadata.changelog] table names.
#
# Exit codes: 0 = pass, 1 = a heading outside the six, or the surface grew or
# shrank and the block lacks what that change needs, 2 = a usage error or a tree
# the parser cannot read. Each failure names what it found and ends on a `fix:`
# line.
set -euo pipefail

# CUSTOMIZE: the binary whose completions/<BIN>.bash describes the CLI, the
# manifest that names the PR body's changelog heading, and the schema directory.
readonly BIN="${SURFACE_BIN:-xr}"
readonly MANIFEST="${SURFACE_MANIFEST:-crates/xurl-cli/Cargo.toml}"
readonly SCHEMA_DIR="${SURFACE_SCHEMA_DIR:-schema}"

readonly MINOR_OR_HIGHER='Added|Deprecated|Breaking changes'
readonly LISTED=20

die() {
  echo "check-surface-bump: $1" >&2
  exit 2
}

# annotate <level> <message>: an Actions annotation in CI, plain text elsewhere.
annotate() {
  if [ "${GITHUB_ACTIONS:-}" = true ]; then
    echo "::$1 title=Changelog bump::$2" >&2
  else
    echo "$1: $2" >&2
  fi
}

# surface <tree>: one line per contract element, unsorted.
surface() {
  local tree="$1" completions="$1/completions/$BIN.bash" cli file
  [ -f "$completions" ] || die "no completions/$BIN.bash under $tree; set BIN in this script"

  # clap_complete writes one `case "${cmd}"` arm per command path, labelled
  # with the words joined by `__subcmd__`; each arm lists its words in
  # `opts` and the values of an enumerated flag in a `compgen -W` under that
  # flag's own arm.
  cli=$(awk '
        /^    case "\$\{cmd\}" in$/ { dispatch = 1; next }
        !dispatch { next }
        /^        [A-Za-z0-9_]+\)$/ {
            cmd = $1; sub(/\)$/, "", cmd); gsub(/__subcmd__/, " ", cmd); flag = ""; next
        }
        /^            opts="/ {
            line = $0; sub(/^ *opts="/, "", line); sub(/"$/, "", line)
            n = split(line, words, " ")
            for (i = 1; i <= n; i++) printf "cli\t%s\t%s\n", cmd, words[i]
            next
        }
        /^                -[^ ]*\)$/ { flag = $1; sub(/\)$/, "", flag); next }
        flag != "" && /compgen -W "/ {
            line = $0; sub(/.*compgen -W "/, "", line); sub(/".*/, "", line)
            n = split(line, values, " ")
            for (i = 1; i <= n; i++) printf "cli\t%s\t%s=%s\n", cmd, flag, values[i]
            flag = ""; next
        }
    ' "$completions")
  [ -n "$cli" ] || die "found no commands in $completions; did the clap_complete output format change?"
  printf '%s\n' "$cli"

  # Array indices become `[]` so reordering a list is not growth; everything from
  # a schema is JSON-escaped so each entry stays on one line, since a key holding a
  # newline would otherwise start a CI log line of its own. A declared key gets its
  # own line, since one whose only leaves are annotations has no other.
  while IFS= read -r file; do
    jq -r --arg file "$file" '
            def container: . == "properties" or . == "patternProperties" or . == "$defs" or . == "definitions";
            def in_examples:
                . as $q
                | any(range($q | length); . as $i | $q[$i] == "examples" and ($i == 0 or ($q[$i - 1] | container | not)));
            def annotation:
                . as $q
                | $q[-1] == "description" or $q[-1] == "title" or $q[-1] == "$id" or ($q | in_examples);
            def escaped: tojson | .[1:-1];
            def rendered: map(if type == "number" then "[]" else . end) | join(".") | escaped;
            (paths(scalars) as $p
                | select($p | annotation | not)
                | "schema\t\($file | escaped)\t\($p | rendered)=\(getpath($p) | tojson)"),
            (paths as $p
                | select(($p | length) >= 2)
                | select($p[-2] | container)
                | select($p | in_examples | not)
                | "schema\t\($file | escaped)\t\($p | rendered)=declared")
        ' "$tree/$file" || die "cannot read $tree/$file as JSON"
  done < <([ -d "$tree/$SCHEMA_DIR" ] && cd "$tree" && find "$SCHEMA_DIR" -type f -name '*.json' | LC_ALL=C sort)
}

# changelog_block <head-tree> <body-file>: tab-separated lines describing the
# changelog block: `six` and the allowed headings, `outside` and each heading
# beyond them, deeper ones included, `bullet`, its section, and the first line of
# its text. The block
# and the six are read from the release's own generate-changelog.py, under
# `## Changelog` or the heading a `[package.metadata.changelog]` table in
# Cargo.toml names, so this check and the release notes cannot disagree about
# which sections a PR filled.
changelog_block() {
  python3 -B - "$1" "$2" "$MANIFEST" <<'PY'
import importlib.util
import re
import sys
import types
from pathlib import Path

# generate-changelog.py imports tomllib for its cliff.toml reader, which this
# check never calls, and Python 3.10 (the reusable CI's ubuntu-22.04) has no
# tomllib; the heading below is read without it for the same reason.
try:
    import tomllib  # noqa: F401
except ModuleNotFoundError:
    sys.modules["tomllib"] = types.ModuleType("tomllib")

tree, body = Path(sys.argv[1]), Path(sys.argv[2])
spec = importlib.util.spec_from_file_location(
    "generate_changelog", tree / "scripts/generate-changelog.py"
)
generator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generator)


def table_value(manifest: str, table: str, key: str) -> str | None:
    """A quoted string value under `[table]` in a Cargo manifest."""
    section = re.search(
        rf"^\[{re.escape(table)}\][ \t]*$(.*?)(?=^\[|\Z)", manifest, re.M | re.S
    )
    if section is None:
        return None
    value = re.search(rf'^{re.escape(key)}[ \t]*=[ \t]*"([^"]*)"', section.group(1), re.M)
    return value.group(1) if value else None


manifest = (tree / sys.argv[3]).read_text(encoding="utf-8")
label = table_value(manifest, "package.metadata.changelog", "heading") or "Changelog"
text = body.read_text(encoding="utf-8")
heading = rf"^## {re.escape(label)}\s*$"
sections = generator.extract_changelog_sections(text, heading)
print("six\t" + ", ".join(f"### {name}" for name in generator.CATEGORIES))
for name, bullets in sections.items():
    if name not in generator.CATEGORIES:
        print(f"outside\t### {name}")
    for bullet in bullets:
        first = bullet.removeprefix("- ").splitlines()
        print(f"bullet\t{name}\t{first[0] if first else ''}")
fence = None
held = None
for line in (generator.slice_below(text, heading) or "").split("\n"):
    if fence is not None:
        marker, indent = fence
        if line.strip() and len(line) - len(line.lstrip(" ")) < indent:
            fence = None
            held = marker
        else:
            if line.strip().startswith(marker):
                fence = None
            continue
    if held is not None:
        if line.strip().startswith(held):
            held = None
        else:
            read_as_text = re.match(r"### (.+)", line)
            if read_as_text:
                print(f"outside\t### {read_as_text.group(1).strip()} (the generator reads it as fenced text)")
    opener = re.match(r"(\s*)(`{3,}(?!.*`)|~{3,})", line)
    if opener:
        fence = (opener.group(2), len(opener.group(1).expandtabs()))
    elif re.match(r"[ \t]*#{3,6}[ \t]+\S", line) and not re.match(r"### ", line):
        print("outside\t" + re.sub(r"\s+", " ", line.strip()))
PY
}

# list <lines>: the first LISTED lines, indented, and a count of the rest.
# sed reads the whole here-string: a `| head` stage would stop reading early,
# and under pipefail the writer's SIGPIPE would end the script.
list() {
  local total
  total=$(wc -l <<<"$1")
  sed -n "1,${LISTED}s/^/  /p" <<<"$1" >&2
  [ "$total" -le "$LISTED" ] || echo "  ... and $((total - LISTED)) more" >&2
}

check() {
  local base="$1" head="$2" body="$3" base_surface head_surface added added_cli removed block outside sections rc=0
  [ -f "$body" ] || die "no PR body at $body"
  base_surface=$(surface "$base" | LC_ALL=C sort -u)
  head_surface=$(surface "$head" | LC_ALL=C sort -u)
  added=$(LC_ALL=C comm -13 <(printf '%s\n' "$base_surface") <(printf '%s\n' "$head_surface"))
  removed=$(LC_ALL=C comm -23 <(printf '%s\n' "$base_surface") <(printf '%s\n' "$head_surface"))
  block=$(changelog_block "$head" "$body") \
    || die "could not read the changelog block with $head/scripts/generate-changelog.py"
  sections=$(awk -F'\t' '$1 == "bullet" { print $2 }' <<<"$block")

  if [ -z "$added$removed" ]; then
    echo "check-surface-bump: the $BIN surface did not grow or shrink"
  fi

  if [ -n "$removed" ]; then
    if awk -F'\t' '$1 == "bullet" && $2 == "Breaking changes" && index($3, "Remove") == 1 { found = 1 } END { exit !found }' <<<"$block"; then
      echo "check-surface-bump: the $BIN surface lost entries and ### Breaking changes has a bullet starting \"Remove\"; confirm it covers each:"
      list "$removed"
    else
      annotate error "the $BIN surface lost entries, and ### Breaking changes has no bullet starting \"Remove\""
      list "$removed"
      echo "fix: add a ### Breaking changes bullet starting \"Remove\" that names the removal (a major); a rename or retype counts as a removal here, so open its bullet with \"Remove\" and name what replaces it" >&2
      rc=1
    fi
  fi

  if [ -n "$added" ]; then
    added_cli=$(awk -F'\t' '$1 == "cli"' <<<"$added")
    if grep -qxE "$MINOR_OR_HIGHER" <<<"$sections"; then
      echo "check-surface-bump: the $BIN surface grew and the changelog block names a minor-or-higher section; confirm it covers each:"
      list "$added"
    elif [ -z "$added_cli" ] && grep -qx Fixed <<<"$sections"; then
      echo "check-surface-bump: only the $BIN schemas grew and ### Fixed has a bullet; confirm each entry describes output the binary already prints and its docs already describe:"
      list "$added"
    elif [ -z "$added_cli" ]; then
      annotate error "the $BIN schemas grew, so the changelog block needs a bullet under ### Added (or ### Deprecated or ### Breaking changes), or under ### Fixed when the schema catches up to output the binary already prints"
      list "$added"
      echo "fix: file a new field or document under ### Added, ### Deprecated, or ### Breaking changes, and a schema catching up to documented output under ### Fixed" >&2
      rc=1
    else
      annotate error "the $BIN surface grew, so the changelog block needs a bullet under ### Added (or ### Deprecated or ### Breaking changes); an addition is a minor"
      list "$added"
      echo "fix: file the addition under ### Added, ### Deprecated, or ### Breaking changes" >&2
      rc=1
    fi
  fi

  outside=$(awk -F'\t' '$1 == "outside" { printf "%s%s", sep, $2; sep = ", " }' <<<"$block")
  if [ -n "$outside" ]; then
    annotate error "the changelog block has a heading outside the six in RELEASES.md: $outside"
    echo "fix: use one of $(awk -F'\t' '$1 == "six" { print $2 }' <<<"$block"); a contract removal is a ### Breaking changes bullet starting \"Remove\"" >&2
    rc=1
  fi
  return "$rc"
}

case "${1:-}" in
  -h | --help)
    awk 'NR > 1 && !/^#/ { exit } NR > 1 { sub(/^# ?/, ""); print }' "$0"
    exit 0
    ;;
esac

command -v jq >/dev/null || die "jq is not installed"

case "${1:-}" in
  surface)
    [ "$#" -eq 2 ] || die "usage: $0 surface <tree>"
    surface "$2"
    ;;
  check)
    [ "$#" -eq 4 ] || die "usage: $0 check <base-tree> <head-tree> <pr-body-file>"
    command -v python3 >/dev/null || die "python3 is not installed"
    check "$2" "$3" "$4"
    ;;
  *)
    die "usage: $0 surface <tree> | check <base-tree> <head-tree> <pr-body-file>"
    ;;
esac
