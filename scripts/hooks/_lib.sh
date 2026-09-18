#!/usr/bin/env bash
# Shared helpers for the git hook pair. Sourced by scripts/hooks/pre-commit and
# scripts/hooks/pre-push, never executed directly.
#
# Contract every check_* helper honours, so the two hooks stay interchangeable:
#   - takes the file list as positional arguments
#   - returns 0 on a pass, $HOOK_SKIPPED when it did not run, and the tool's own
#     exit status on a failure. Three outcomes, not two: a skip that returned 0
#     would print a pass line, which is the lie the skip notice exists to
#     prevent.
#   - announces a skip caused by a missing tool on stderr. A silent skip is
#     indistinguishable from a passing check. An empty file list is not
#     announced: nothing of a given type in scope is not a gap in the gate.
#   - is driven by `step`, which turns those three outcomes into output and a
#     0/1 status every caller aggregates the same way: `step ... || fail=1`.
#
# This file owns how a tool runs; each hook owns which files it runs on.
# pre-commit passes the staged set, pre-push passes what the push carries.
#
# Callers cd to the repo root before sourcing, so relative paths resolve.

# Require bash >= 4.4: mapfile and safe empty-array expansion under `set -u`.
# Sourced, so return rather than exit, which would kill an interactive shell.
if ((BASH_VERSINFO[0] < 4 || (BASH_VERSINFO[0] == 4 && BASH_VERSINFO[1] < 4))); then
  printf 'error: bash >= 4.4 required, but this is bash %s.\n' "${BASH_VERSION:-unknown}" >&2
  printf 'Install a newer bash: brew install bash\n' >&2
  return 1 2>/dev/null || exit 1
fi

# ── Output and step dispatch ──────────────────────────────────────────────

# Colour only when stdout is a terminal: a hook run through a pager, a CI log,
# or an agent harness should not carry escape codes.
if [ -t 1 ]; then
  _HOOK_RED=$'\033[0;31m'
  _HOOK_GREEN=$'\033[0;32m'
  _HOOK_BOLD=$'\033[1m'
  _HOOK_DIM=$'\033[2m'
  _HOOK_RESET=$'\033[0m'
else
  _HOOK_RED='' _HOOK_GREEN='' _HOOK_BOLD='' _HOOK_DIM='' _HOOK_RESET=''
fi

banner() { printf '%s%s%s\n' "$_HOOK_BOLD" "$1" "$_HOOK_RESET"; }
pass() { printf '  %s✓%s %s\n' "$_HOOK_GREEN" "$_HOOK_RESET" "$1"; }
step_failed() { printf '  %s✗%s %s\n' "$_HOOK_RED" "$_HOOK_RESET" "$1" >&2; }
note() { printf '  %s-%s %s\n' "$_HOOK_DIM" "$_HOOK_RESET" "$1"; }

# The status meaning "this check did not run": distinct from both 0 and a tool
# failure, so `step` can tell a skip from a pass.
HOOK_SKIPPED=99

# Every announced skip goes through here, so no missing tool skips silently.
# Returns HOOK_SKIPPED, so a check guards with `have foo || skip ... || return`.
skip() {
  printf '  - %s (skipped: %s)\n' "$1" "$2" >&2
  return "$HOOK_SKIPPED"
}

have() { command -v "$1" >/dev/null 2>&1; }

# Run one check and report it. `label` names the pass line, `hint` names the
# command to rerun after a failure. Returns 0 on a pass or a skip, 1 on a
# failure.
step() {
  local label="$1" hint="$2" rc
  shift 2
  "$@"
  rc=$?
  if [ "$rc" -eq 0 ]; then
    pass "$label"
  elif [ "$rc" -ne "$HOOK_SKIPPED" ]; then
    step_failed "$hint"
    return 1
  fi
}

# ── File selection ────────────────────────────────────────────────────────

# Staged paths matching the given pathspecs. ACMR only: a deletion cannot fail
# a lint, and the linter would fail trying to open the missing path.
staged() { git diff --cached --name-only --diff-filter=ACMR -- "$@"; }

# Tracked paths matching the given pathspecs that still exist on disk. A file
# deleted from the working tree but not yet committed stays in the index, so
# `git ls-files` still lists it.
tracked() {
  local f
  while IFS= read -r f; do
    [ -e "$f" ] && printf '%s\n' "$f"
  done < <(git ls-files -- "$@")
}

# Tracked markdown, minus the generated changelog. A pathspec cannot leave the
# repository, which a `**/*.md` glob can: a working clone carries symlinked
# directories, and the shared markdownlint config sets `fix: true`, so a glob
# that descends one edits files outside the repo. In a clean checkout this set
# is exactly what the `lint / markdownlint` CI job globs.
tracked_markdown() { tracked '*.md' ':!:CHANGELOG.md' ':!:*/CHANGELOG.md'; }

# Every tracked shell artifact that still exists, the hooks included.
tracked_shell() { tracked '*.sh' 'scripts/hooks/pre-commit' 'scripts/hooks/pre-push' 'scripts/hooks/_lib.sh'; }

# ── Checks ────────────────────────────────────────────────────────────────

# Rust format check on individual files. `cargo fmt --check` is whole-crate, so
# a file-scoped hook drives rustfmt directly; the edition matches rustfmt.toml,
# which rustfmt does not read for a direct file invocation. Reports every
# offending file rather than stopping at the first.
check_rustfmt() {
  [ "$#" -gt 0 ] || return "$HOOK_SKIPPED"
  have rustfmt || skip "rustfmt" "ships with rustup" || return
  local f rc=0
  for f in "$@"; do
    [ -f "$f" ] || continue
    rustfmt --edition 2024 --check "$f" || rc=1
  done
  return "$rc"
}

# Markdown lint over the given files.
#
# The shared markdownlint config sets `fix: true`, so this rewrites what it
# matches rather than only reading it. A clean exit therefore does not mean
# "nothing was wrong", and a caller that stopped there would let the fixes sit
# uncommitted while the committed content stays unfixed.
check_markdownlint() {
  [ "$#" -gt 0 ] || return "$HOOK_SKIPPED"
  have markdownlint-cli2 || skip "markdownlint" "install via \`brew install markdownlint-cli2\`" || return
  local rc=0
  markdownlint-cli2 "$@" >/dev/null 2>&1 || rc=1
  if ! git diff --quiet -- "$@"; then
    printf '  markdownlint fixed markdown in place; review and stage the result\n' >&2
    rc=1
  fi
  return "$rc"
}

# Workflow lint. actionlint discovers .github/workflows itself, so the file
# list gates whether it runs rather than what it reads.
check_actionlint() {
  [ "$#" -gt 0 ] || return "$HOOK_SKIPPED"
  have actionlint || skip "actionlint" "install via \`brew install actionlint\`" || return
  actionlint
}

# Concurrency-group namespacing. The file list gates whether it runs; the script
# reads every workflow itself, because the rule is a property of the set.
check_workflow_concurrency() {
  [ "$#" -gt 0 ] || return "$HOOK_SKIPPED"
  # Its success line goes to stdout and its findings to stderr, so dropping
  # stdout keeps a pass quiet like every other check and leaves a failure loud.
  ./scripts/check-workflow-concurrency.sh >/dev/null
}

# Shell correctness — severity=warning catches real bugs (quoting, unused vars,
# missing exits) while leaving info/style noise out.
check_shellcheck() {
  [ "$#" -gt 0 ] || return "$HOOK_SKIPPED"
  have shellcheck || skip "shellcheck" "install via \`brew install shellcheck\`" || return
  shellcheck --severity=warning "$@"
}
