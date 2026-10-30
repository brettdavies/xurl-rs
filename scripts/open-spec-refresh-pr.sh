#!/usr/bin/env bash
# Open the spec-refresh PR to dev, or update the open one, for the upstream X
# API spec the drift check fetched.
#
# Caller: .github/workflows/spec-drift.yml, step "Open or update spec refresh
# PR", on a scheduled or dispatched run that found drift.
#
# Inputs, from the environment:
#   UPSTREAM_SPEC      path of the fetched upstream spec (steps.fetch.outputs.tmp)
#   REFRESH_PR_NUMBER  number of the open refresh PR, empty when there is none
#                      (steps.dedupe.outputs.number)
#   REFRESH_BRANCH, VENDOR_PATH, UPSTREAM_URL  the job's env
#   GH_TOKEN           the token gh pushes and opens the PR with

# The braces make bash read the whole body before running it: the git switch
# below rewrites the working tree, this file included. Errexit is on, as in the
# `bash -e` shell GitHub gives a workflow's inline run step.
{
set -euo pipefail
upstream_version="$(jq -r '.info.version' "${UPSTREAM_SPEC}")"
title="feat(spec): refresh vendored X API spec to upstream ${upstream_version}"
existing_num="${REFRESH_PR_NUMBER}"

# The drift gate compares against this run's own ref (main on the
# cron), but the PR lands on dev, so the refresh branch is cut from
# origin/dev. The no-change guard below stands down when dev
# already carries the upstream revision.
git fetch --depth 1 origin dev:refs/remotes/origin/dev
git switch -c "${REFRESH_BRANCH}" origin/dev
# The gate's report compared against this run's own ref (main on
# the cron); the body's detail regenerates here against dev, the
# revision this PR actually changes.
body_dev="$(mktemp)"
scripts/diff-x-openapi-spec.sh "${VENDOR_PATH}" "${UPSTREAM_SPEC}" "${UPSTREAM_URL}" > "${body_dev}"
decisions_dev="$(mktemp)"
scripts/endpoint-decisions.sh covered "${VENDOR_PATH}" "${UPSTREAM_SPEC}" > "${decisions_dev}"
scripts/refresh-x-openapi.sh
# The refresh is script-only by design: the auth-method matrix
# regenerates from the vendored spec inside the opened PR's own CI
# build, which fails there if a shortcut template no longer
# resolves, and schema/responses/ derives from crate types a
# spec-only change cannot move.
git add crates/xdk/vendor/

# Both no-change checks compare the spec alone: the refresh stamps
# today's date into spec-metadata.json and vendor/README.md, so a
# whole-directory compare reads every new day as new content and
# stacks a date-only commit that also rewrites an agent-owned body.
if git diff --cached --quiet -- "${VENDOR_PATH}"; then
  echo "Refresh produced no change against dev; nothing to open." >> "$GITHUB_STEP_SUMMARY"
  exit 0
fi
# shellcheck disable=SC2016 # backticks are literal markdown
changed_files="$(git diff --cached --name-only | sed 's/^/- `/; s/$/`/')"

# dev requires verified signatures, so the commit is created with
# the createCommitOnBranch mutation: GitHub signs commits it
# creates, where a workflow-side git commit stays unsigned and
# unmergeable.
result=""
dev_sha="$(git rev-parse origin/dev)"
if [ -n "${existing_num}" ]; then
  # Update stacks the new commit on the branch head: resetting the
  # branch to the dev tip first would momentarily empty the PR,
  # and GitHub auto-closes a PR whose head reaches its base.
  git fetch --depth 1 origin "${REFRESH_BRANCH}:refs/remotes/origin/${REFRESH_BRANCH}"
  head_sha="$(git rev-parse "refs/remotes/origin/${REFRESH_BRANCH}")"
  if git diff --cached --quiet "refs/remotes/origin/${REFRESH_BRANCH}" -- "${VENDOR_PATH}"; then
    result="current"
    echo "Open refresh PR #${existing_num} already carries this upstream revision; branch content untouched." >> "$GITHUB_STEP_SUMMARY"
  fi
else
  head_sha="${dev_sha}"
  gh api -X POST "repos/${GITHUB_REPOSITORY}/git/refs" \
    -f ref="refs/heads/${REFRESH_BRANCH}" -f sha="${dev_sha}" >/dev/null 2>&1 \
    || gh api -X PATCH "repos/${GITHUB_REPOSITORY}/git/refs/heads/${REFRESH_BRANCH}" \
      -f sha="${dev_sha}" -F force=true >/dev/null
fi

if [ -z "${result}" ]; then
  # File contents route through files, never argv: a single
  # argument caps at 128KB on Linux and the base64 spec is ~1.2MB.
  additions_file="$(mktemp)"
  git diff --cached --name-only | while IFS= read -r f; do
    b64="$(mktemp)"
    base64 -w0 "$f" > "${b64}"
    jq -n --arg p "$f" --rawfile c "${b64}" '{path: $p, contents: $c}'
  done | jq -s . > "${additions_file}"
  payload="$(mktemp)"
  jq -n \
    --arg repo "${GITHUB_REPOSITORY}" \
    --arg branch "${REFRESH_BRANCH}" \
    --arg head "${head_sha}" \
    --arg msg "${title}" \
    --slurpfile additions "${additions_file}" \
    '{query: "mutation($input: CreateCommitOnBranchInput!) { createCommitOnBranch(input: $input) { commit { oid } } }",
      variables: {input: {branch: {repositoryNameWithOwner: $repo, branchName: $branch}, expectedHeadOid: $head, message: {headline: $msg}, fileChanges: {additions: $additions[0]}}}}' \
    > "${payload}"
  # GraphQL returns HTTP 200 with an errors array, so a swallowed
  # response hides a failed mutation.
  resp="$(mktemp)"
  gh api graphql --input "${payload}" > "${resp}"
  if jq -e '.errors' "${resp}" >/dev/null 2>&1; then
    echo "createCommitOnBranch failed:" >&2
    jq -r '.errors[].message' "${resp}" >&2
    exit 1
  fi
fi

# The body is an agent runbook, not the repo PR template: the
# agent that reconciles the codebase rewrites it to
# .github/pull_request_template.md when the PR is merge-ready. The
# marker on the first line records that the workflow still owns
# the body; the tail below rewrites only marker-owned bodies on a
# no-change run, so an agent's template rewrite survives until a
# real content update invalidates the reconciliation.
body_marker='<!-- spec-drift-runbook -->'
pr_body="$(mktemp)"
{
  echo "${body_marker}"
  echo ""
  echo "## Spec refresh (agent runbook)"
  echo ""
  echo "You are the reconciling agent for this spec refresh. Executing this runbook is the deliverable; a review or a report is not. This PR vendors upstream ${upstream_version}; the diff carries only vendored artifacts, and the codebase must be reconciled to them on this branch before merge."
  echo ""
  echo "Latest workflow pass: [run ${GITHUB_RUN_ID}](${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}) on $(date -u +%Y-%m-%d)."
  echo ""
  echo "### Files in this PR"
  echo ""
  printf '%s\n' "${changed_files}"
  echo ""
  echo "### Drift detail"
  echo ""
  cat "${body_dev}"
  echo ""
  echo "### Endpoint decisions"
  echo ""
  echo "Operations this refresh adds that \`endpoint-decisions.json\` already decides. Build a \`planned\` one with the rest of the refresh; build no command for a \`raw-only\` or \`not-supported\` one while it holds. An added operation no decision covers takes the judgment in step 4: a command where it fits \`xr\`, a proposed decision where it does not. The daily run's summary reports a decision whose trigger has fired."
  echo ""
  cat "${decisions_dev}"
  echo ""
  echo "### Automated gates (this PR's CI)"
  echo ""
  echo "- \`cargo build\` regenerates the auth-method matrix from the new spec; a build panic names a shortcut template that no longer resolves. Update the endpoint in \`crates/xdk/src/api/shortcuts.rs\` and the \`SHORTCUT_TEMPLATES\` allowlist in \`crates/xdk/build.rs\`."
  echo "- \`crates/xdk/tests/spec_types_validation.rs\` fails when a typed response still declares a field, or maps a component schema, that the spec renamed or removed. \`ALLOWED_DIVERGENCES\` applies only to field-level divergences with a documented upstream reason; it cannot suppress a missing component schema. For a renamed schema, repoint the test's mapping table and rework the affected struct in \`crates/xdk/src/api/response/types.rs\`."
  echo ""
  echo "### Execute, in order"
  echo ""
  echo "1. Run the gates locally (\`cargo build\`, \`cargo test\`) and fix everything they force."
  echo "2. Apply every rename the spec mandates: wire params and expansions in \`crates/xdk/src/api/shortcuts.rs\`, field and schema names in \`crates/xdk/src/api/response/types.rs\`. Grep for old names across \`crates/\` so no call site survives. Record each under Breaking changes below."
  echo "3. Retire shortcuts whose endpoints the spec removed, including their typed responses and help examples. Record under Breaking changes."
  echo "4. Judge each added endpoint that Endpoint decisions does not cover. A command fits \`xr\` when the endpoint belongs to a family \`xr\` has or a domain it serves, uses a credential \`xr\` holds, and is a read or an action a user or agent would run from a CLI. Build each one that fits through every surface in \`AGENTS.md\` § \"Adding a command family\". For one that does not (a feature \`xr\` does not support, or supports only in part, or one that needs keys or a client flow \`xr\` lacks), build nothing: propose under Open decisions the \`endpoint-decisions.json\` entry it would take (\`raw-only\` or \`not-supported\`, the reason, and \`revisit_when\`), grounded in how similar endpoints were decided. Until a person adds that entry, \`crates/xdk/tests/endpoint_decisions.rs\` fails for the operation; name that failure in the same entry. Retitle this PR for what it adds, and keep \"refresh vendored X API spec\" in the title: the workflow finds this PR by it, and opens a new one from \`dev\` over this branch when it does not."
  echo "5. You may verify a renamed wire param with read-only calls against the live API using stored credentials (\`xr whoami\`, \`xr search\` with the renamed param); never mutating calls."
  echo "6. Regenerate derived artifacts: \`scripts/generate-response-schemas.sh\` and \`scripts/generate-completions.sh\`."
  echo "7. Update \`README.md\` and \`AGENTS.md\` where the CLI surface changed."
  echo "8. Rewrite this PR body per \`.github/pull_request_template.md\`, carrying Breaking changes and Open decisions into it. Use the template verbatim: its first line is \`## Summary\`; add no headings of your own above or around it."
  echo ""
  echo "### Decision authority"
  echo ""
  echo "- Decide and implement without asking: spec-mandated renames, mapping repoints, retirements forced by removed endpoints, derived-artifact regeneration, documentation updates."
  echo "- Judge, and show the precedent: whether each added endpoint no decision covers fits \`xr\`, by the test in step 4. Build a command for one that fits. Ground every call in \`endpoint-decisions.json\` and in how similar endpoints were decided, and name that precedent in the PR body."
  echo "- Propose under Open decisions, never implement unprompted: leaving an added endpoint without a command, a new or changed \`endpoint-decisions.json\` entry, \`ALLOWED_DIVERGENCES\` entries, version-bump classification, anything the spec leaves ambiguous. Every proposal carries your recommendation and the contextual reasoning behind it, grounded in repo precedent (for example: this endpoint resembles existing shortcuts X and Y, or similar endpoints deliberately have no shortcut)."
  echo "- A command is a judgment, not a default: \`xr\` adds commands only where they make sense. Describe the PR as adding commands for the endpoints that fit and proposing decisions for the rest, never as a command for every added endpoint."
  echo ""
  echo "### Open decisions"
  echo ""
  echo "(none yet; append entries as: the question, your recommendation, and the precedent-based reasoning. A human resolves each by editing this section or commenting.)"
  echo ""
  echo "### Breaking changes"
  echo ""
  echo "(none yet; append each user-observable change here as you make it)"
  echo ""
  echo "### Completion criteria"
  echo ""
  echo "- [ ] All CI checks green"
  echo "- [ ] Breaking changes enumerated above"
  echo "- [ ] Derived artifacts regenerated"
  echo "- [ ] Body rewritten to the repo template, commits signed"
  echo ""
  echo "A later run that vendors newer upstream content rewrites this body and restarts the reconciliation. A no-change run refreshes only the run-link line, so sections appended under the marker survive."
} > "${pr_body}"

if [ "${result}" = "current" ]; then
  # Refresh only the run-link line: a full rewrite would wipe the
  # Open decisions and Breaking changes sections an agent appends
  # while reconciling under the marker.
  existing_body="$(mktemp)"
  gh pr view "${existing_num}" --json body --jq '.body | gsub("\r"; "")' > "${existing_body}"
  if [ "$(head -n1 "${existing_body}")" = "${body_marker}" ] && grep -q '^Latest workflow pass:' "${existing_body}"; then
    sed -i "s|^Latest workflow pass:.*|Latest workflow pass: [run ${GITHUB_RUN_ID}](${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}) on $(date -u +%Y-%m-%d).|" "${existing_body}"
    gh pr edit "${existing_num}" --body-file "${existing_body}"
  else
    echo "PR #${existing_num} body is agent-owned; left untouched." >> "$GITHUB_STEP_SUMMARY"
  fi
elif [ -n "${existing_num}" ]; then
  gh pr edit "${existing_num}" --title "${title}" --body-file "${pr_body}"
else
  gh pr create --base dev --head "${REFRESH_BRANCH}" --title "${title}" --body-file "${pr_body}"
fi
exit
}
