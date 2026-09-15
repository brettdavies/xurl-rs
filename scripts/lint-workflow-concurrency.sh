#!/usr/bin/env bash
# Concurrency-group namespacing guard.
#
# Fails when a workflow's `concurrency.group` begins with an expression rather
# than a literal token naming the workflow that owns it.
#
# Why the rule exists: inside a called workflow, `github.workflow` resolves to
# the *caller's* workflow name. A group keyed only on context is therefore a key
# a caller also produces for itself, both land in one group, and when either
# side cancels in progress the called workflow's queue cancels the run that
# called it. The run concludes failure with every visible job green and the
# called workflow's jobs missing, and nothing in the logs names a cause.
#
# A literal prefix is the part a caller and a callee cannot both produce by
# accident, because it names a file rather than a run.
#
# Exit codes: 0 = every group is namespaced (or none is declared), 1 = a
# violation, named with its file and line.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

violations=0

for wf in .github/workflows/*.yml .github/workflows/*.yaml; do
  [ -f "$wf" ] || continue
  # Only `group:` lines inside a concurrency block, so a `group:` key
  # belonging to anything else is never matched.
  while IFS=: read -r lineno value; do
    [ -n "${lineno:-}" ] || continue
    case "$value" in
      '$'*)
        echo "FAIL: $wf:$lineno concurrency group starts with an expression: $value" >&2
        violations=$((violations + 1))
        ;;
    esac
  done < <(
    awk '
            /^concurrency:/ { inblock = 1; next }
            /^[^[:space:]#]/ { inblock = 0 }
            inblock && /^[[:space:]]+group:/ {
                line = $0
                sub(/^[[:space:]]+group:[[:space:]]*/, "", line)
                print NR ":" line
            }
        ' "$wf"
  )
done

if [ "$violations" -gt 0 ]; then
  cat >&2 <<'EOF'

Key each concurrency group on a literal token naming the workflow that owns it:

  concurrency:
    group: ci-${{ github.ref }}

Keep any context parts that separate callers and refs; lead with the literal.
EOF
  exit 1
fi

echo "OK: every concurrency group is namespaced with a literal token"
