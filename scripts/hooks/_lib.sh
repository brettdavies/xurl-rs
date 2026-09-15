#!/usr/bin/env bash
# Shared helpers for the git hooks. Sourced by pre-commit and pre-push, never
# executed directly. Each check_* helper wraps one tool, takes the file list as
# arguments, skips cleanly when given no files or when the tool is absent, and
# returns the tool's exit status so the caller can aggregate.
#
# The split is deliberate: this file owns how a tool runs, each hook owns which
# files it runs on. pre-commit passes the staged set, pre-push passes what the
# push carries, and neither duplicates the invocation or the missing-tool hint.
#
# Callers cd to the repo root before sourcing, so relative paths resolve.

# Require bash >= 4.4: mapfile and safe empty-array expansion under `set -u`.
# Sourced, so return rather than exit, which would kill an interactive shell.
if ((BASH_VERSINFO[0] < 4 || (BASH_VERSINFO[0] == 4 && BASH_VERSINFO[1] < 4))); then
  printf 'error: bash >= 4.4 required, but this is bash %s.\n' "${BASH_VERSION:-unknown}" >&2
  printf 'Install a newer bash: brew install bash\n' >&2
  return 1 2>/dev/null || exit 1
fi

have() { command -v "$1" >/dev/null 2>&1; }

# Markdown lint over the given files.
#
# The shared markdownlint config sets `fix: true`, so this rewrites what it
# matches rather than only reading it. A clean exit therefore does not mean
# "nothing was wrong", and a caller that stops there would let the fixes sit
# uncommitted while the committed content stays unfixed. Reporting the rewrite
# is this helper's job, because both hooks would otherwise have to remember it.
#
# Returns 1 when the lint fails or when a fix landed in place.
check_markdownlint() {
  [ "$#" -gt 0 ] || return 0
  if ! have markdownlint-cli2; then
    echo "hooks: markdownlint-cli2 not found — skipping (brew install markdownlint-cli2)" >&2
    return 0
  fi
  local status=0
  markdownlint-cli2 "$@" >/dev/null 2>&1 || status=1
  if ! git diff --quiet -- "$@"; then
    echo "hooks: markdownlint fixed markdown in place; review and stage the result" >&2
    status=1
  fi
  return "$status"
}

# Every tracked markdown file that still exists, minus the generated changelog.
#
# A pathspec cannot leave the repository, which a `**/*.md` glob can: a working
# clone carries symlinked directories, and the config's `fix: true` means a
# glob that descends one edits files outside the repo. In a clean checkout this
# set is exactly what the `lint / markdownlint` CI job globs.
#
# A tracked file deleted from disk but not yet committed stays in the index, so
# filtering to existing paths keeps the linter from failing on a missing file.
tracked_markdown() {
  local f
  while IFS= read -r f; do
    [ -f "$f" ] && printf '%s\n' "$f"
  done < <(git ls-files '*.md' ':!:CHANGELOG.md')
}

# Workflow lint. actionlint discovers .github/workflows itself, so the file
# list gates whether it runs rather than what it reads.
check_actionlint() {
  [ "$#" -gt 0 ] || return 0
  if ! have actionlint; then
    echo "hooks: actionlint not found — skipping (brew install actionlint)" >&2
    return 0
  fi
  actionlint
}

# Concurrency-group namespacing. The file list gates whether it runs; the script
# reads every workflow itself, because the rule is a property of the set.
check_workflow_concurrency() {
    [ "$#" -gt 0 ] || return 0
    ./scripts/lint-workflow-concurrency.sh
}

# Shell correctness — severity=warning catches real bugs (quoting, unused vars,
# missing exits) while leaving info/style noise out.
check_shellcheck() {
  [ "$#" -gt 0 ] || return 0
  if ! have shellcheck; then
    echo "hooks: shellcheck not found — skipping (brew install shellcheck)" >&2
    return 0
  fi
  shellcheck --severity=warning "$@"
}

# Every tracked shell artifact that still exists, the hooks included.
tracked_shell() {
  local f
  while IFS= read -r f; do
    [ -f "$f" ] && printf '%s\n' "$f"
  done < <(git ls-files '*.sh' 'scripts/hooks/pre-commit' 'scripts/hooks/pre-push' 'scripts/hooks/_lib.sh')
}

# Rust format check on individual files. `cargo fmt --check` is whole-crate, so
# a staged-file scope runs rustfmt directly; the edition matches rustfmt.toml,
# which rustfmt does not read for a direct file invocation.
check_rustfmt() {
  [ "$#" -gt 0 ] || return 0
  if ! have rustfmt; then
    echo "hooks: rustfmt not found — skipping (it ships with rustup)" >&2
    return 0
  fi
  rustfmt --edition 2024 --check "$@"
}
