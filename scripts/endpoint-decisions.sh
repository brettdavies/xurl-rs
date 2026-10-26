#!/usr/bin/env bash
# Read endpoint-decisions.json, the record of what xr decided for every spec
# operation without a shortcut command: no command until a trigger fires
# (raw-only, not-supported), or a command family to build (planned).
#
# Usage:
#   scripts/endpoint-decisions.sh covered <old-spec-json> <new-spec-json>
#   scripts/endpoint-decisions.sh triggers
#
# covered: prints a markdown list of the operations <new-spec-json> adds over
#   <old-spec-json> that a decision covers, with what the decision says to do.
#   Prints one line saying so when none is covered.
#
# triggers: prints one line per decision: `fired <id>: <evidence>` once its
#   revisit_when has happened, `holding <id>: <what reopens it>` before,
#   `planned <id>: ...` for a command still to build, and `unknown <id>: <why>`
#   when the check could not run. A fired decision is reopened by a person, who
#   updates or retires its entry. Reads GitHub with `gh` (GH_TOKEN) and
#   crates.io over HTTPS; nothing is written.
#
# Exits 0 whatever the decisions say; 2 on a usage error or an unreadable file.
#
# Callers:
#   - .github/workflows/spec-drift.yml (the runbook's Endpoint decisions
#     section; the daily trigger report)
#   - manual: scripts/endpoint-decisions.sh triggers

set -uo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
registry="${repo_root}/endpoint-decisions.json"

usage() {
  sed -n '4,7p' "$0" | sed 's/^# \{0,1\}//' >&2
  exit 2
}

[ -r "${registry}" ] || {
  echo "endpoint-decisions: cannot read ${registry}" >&2
  exit 2
}

# The operations a spec declares, one `METHOD /path` per line.
operations() {
  jq -r '.paths | to_entries[] | .key as $path | .value | keys[]
        | select(test("^(get|post|put|patch|delete)$"))
        | "\(ascii_upcase) \($path)"' "$1"
}

covered() {
  local old="$1" new="$2"
  [ -r "${old}" ] && [ -r "${new}" ] || usage
  local added
  added="$(comm -13 <(operations "${old}" | sort -u) <(operations "${new}" | sort -u))"
  if [ -z "${added}" ]; then
    echo "The refresh adds no operation."
    return 0
  fi
  local lines
  lines="$(jq -r --arg added "${added}" '
        def covers($pattern; $op):
            ($pattern | split(" ")) as [$want_method, $want_path]
            | ($op | split(" ")) as [$method, $path]
            | ($want_method == "*" or $want_method == $method)
              and (if ($want_path | endswith("/**"))
                   then ($want_path | rtrimstr("/**")) as $prefix
                        | ($path == $prefix or ($path | startswith($prefix + "/")))
                   else $path == $want_path end);
        def reopens:
            if .kind == "never" then "never"
            elif .kind == "github-pr-merged" then "\(.repo)#\(.number) merges"
            elif .kind == "crate-published" then "crate `\(.crate)` is published"
            else .kind end;
        .decisions[] as $decision
        | [$added | split("\n")[] | select(length > 0)
           | select(. as $op | any($decision.operations[]; covers(.; $op)))]
        | select(length > 0)
        | "- \(map("`\(.)`") | join(", ")): decided `\($decision.id)` (\($decision.status), \($decision.decided), \($decision.by)). \($decision.reason) \(if $decision.status == "planned" then "Build it." else "No command; reopens when: \($decision.revisit_when | reopens)." end)"
    ' "${registry}")"
  if [ -z "${lines}" ]; then
    echo "No decision covers an operation this refresh adds."
  else
    printf '%s\n' "${lines}"
  fi
}

triggers() {
  local id kind repo number crate merged status
  # Fields are split on the unit separator: a tab is whitespace to `read`,
  # so empty fields between tabs would collapse and shift the rest.
  while IFS=$'\x1f' read -r id kind repo number crate; do
    case "${kind}" in
      planned)
        echo "planned ${id}: until the command ships"
        ;;
      never)
        echo "holding ${id}: never reopens"
        ;;
      github-pr-merged)
        if merged="$(gh api "repos/${repo}/pulls/${number}" --jq '.merged' 2>/dev/null)"; then
          if [ "${merged}" = "true" ]; then
            echo "fired ${id}: ${repo}#${number} merged"
          else
            echo "holding ${id}: until ${repo}#${number} merges"
          fi
        else
          echo "unknown ${id}: could not read ${repo}#${number}"
        fi
        ;;
      crate-published)
        status="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 20 \
          -A 'xurl-rs endpoint-decisions (github.com/brettdavies/xurl-rs)' \
          "https://crates.io/api/v1/crates/${crate}" 2>/dev/null)"
        case "${status}" in
          200) echo "fired ${id}: crate ${crate} is published" ;;
          404) echo "holding ${id}: until crate ${crate} is published" ;;
          *) echo "unknown ${id}: crates.io answered ${status:-nothing} for ${crate}" ;;
        esac
        ;;
      *)
        echo "unknown ${id}: trigger kind ${kind}"
        ;;
    esac
  done < <(jq -r '.decisions[] | [.id,
        (if .status == "planned" then "planned" else .revisit_when.kind end),
        (.revisit_when.repo // ""), (.revisit_when.number // "" | tostring),
        (.revisit_when.crate // "")] | join("\u001f")' "${registry}")
}

case "${1:-}" in
  covered)
    [ $# -eq 3 ] || usage
    covered "$2" "$3"
    ;;
  triggers)
    [ $# -eq 1 ] || usage
    triggers
    ;;
  *) usage ;;
esac
