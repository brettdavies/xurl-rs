#!/usr/bin/env bats
#
# scripts/release/postflight.sh's release and tap gates against a `gh` stub
# that answers `gh run list` from one fixture of runs across both repositories.

setup() {
  REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/../../.." && pwd)"
  T="$BATS_TEST_TMPDIR"
  JQ="$(command -v jq || command -v jaq)"
  mkdir -p "$T/bin"

  # Filters $T/runs.json by --repo, --workflow, --event, and --branch, then
  # applies --jq, as `gh run list` does.
  cat >"$T/bin/gh" <<EOF
#!/usr/bin/env bash
[ "\$1 \$2" = "run list" ] || { echo "gh stub: unhandled \$*" >&2; exit 1; }
shift 2
repo="" wf="" event="" branch="" filter="."
while [ \$# -gt 0 ]; do
  case "\$1" in
    --repo|-R) repo="\$2"; shift 2 ;;
    --workflow) wf="\$2"; shift 2 ;;
    --event) event="\$2"; shift 2 ;;
    --branch) branch="\$2"; shift 2 ;;
    --jq) filter="\$2"; shift 2 ;;
    *) shift ;;
  esac
done
"$JQ" -c --arg r "\$repo" --arg w "\$wf" --arg e "\$event" --arg b "\$branch" \
  '[.[] | select(.repo == \$r and (\$w == "" or .workflow == \$w) and (\$e == "" or .event == \$e) and (\$b == "" or .headBranch == \$b))]' \
  "$T/runs.json" | "$JQ" -c "\$filter"
EOF
  if ! command -v jaq >/dev/null; then
    printf '#!/usr/bin/env bash\nexec "%s" "$@"\n' "$JQ" >"$T/bin/jaq"
  fi
  chmod +x "$T"/bin/*
  : >"$T/runs.ndjson"
}

# run_entry ID REPO WORKFLOW EVENT BRANCH TITLE CONCLUSION CREATED
# shellcheck disable=SC2016  # $id and the rest are jq variables, not shell ones
run_entry() {
  "$JQ" -nc --argjson id "$1" --arg repo "$2" --arg wf "$3" --arg event "$4" --arg branch "$5" \
    --arg title "$6" --arg conclusion "$7" --arg created "$8" \
    '{databaseId: $id, repo: $repo, workflow: $wf, event: $event, headBranch: $branch, displayTitle: $title,
      status: "completed", conclusion: $conclusion, createdAt: $created}' >>"$T/runs.ndjson"
}

dispatch_run() { run_entry 1 o/r release.yml push v9.9.9 "Dispatch v9.9.9" success 2026-11-08T01:00:00Z; }
release_run() { run_entry 2 o/r release.yml workflow_dispatch main "Release v9.9.9" "$1" 2026-11-08T01:00:05Z; }
tap_run() { run_entry "$1" brettdavies/homebrew-tap update-formula.yml "$2" main "$3" "$4" "$5"; }

postflight() {
  "$JQ" -s . "$T/runs.ndjson" >"$T/runs.json"
  run env PATH="$T/bin:$PATH" bash "$REPO_ROOT/scripts/release/postflight.sh" --repo o/r --tag v9.9.9 "$1"
}

@test "a failed dispatched release fails the gate though the tag's own run passed" {
  dispatch_run
  release_run failure
  postflight release
  [ "$status" -eq 1 ]
  [[ "$output" == *"✗ release.yml run 2"* ]]
}

@test "with only the tag's dispatch run, the release gate waits for the dispatched run" {
  dispatch_run
  postflight release
  [ "$status" -eq 0 ]
  [[ "$output" == *"no run titled Release v9.9.9 has started yet"* ]]
  [[ "$output" != *"✓"* ]]
}

@test "a failed dispatch job fails the release gate rather than waiting" {
  run_entry 1 o/r release.yml push v9.9.9 "Dispatch v9.9.9" failure 2026-11-08T01:00:00Z
  postflight release
  [ "$status" -eq 1 ]
  [[ "$output" == *"✗ release.yml run 1"* ]]
  [[ "$output" == *"RELEASES.md#when-the-dispatch-job-fails"* ]]
}

@test "a caller that releases on the tag push is read from the tag's own run" {
  run_entry 7 o/r release.yml push v9.9.9 "chore(release): v9.9.9" success 2026-11-08T01:00:00Z
  postflight release
  [ "$status" -eq 0 ]
  [[ "$output" == *"✓ release.yml run 7"* ]]
}

@test "a dispatched release of another tag is not this tag's" {
  run_entry 3 o/r release.yml workflow_dispatch main "Release v9.9.8" success 2026-11-08T01:00:05Z
  postflight release
  [ "$status" -eq 0 ]
  [[ "$output" != *"run 3"* ]]
}

@test "the tap gate passes on the formula's update-formula run, dispatched or run by hand" {
  dispatch_run
  release_run success
  tap_run 50 repository_dispatch "Update xurl-rs to v9.9.9" success 2026-11-08T01:06:00Z
  postflight tap
  [ "$status" -eq 0 ]
  [[ "$output" == *"✓ tap update-formula run 50 success"* ]]

  : >"$T/runs.ndjson"
  dispatch_run
  release_run success
  tap_run 55 workflow_dispatch "Update xurl-rs to v9.9.9" success 2026-11-08T01:09:00Z
  postflight tap
  [[ "$output" == *"✓ tap update-formula run 55 success"* ]]
}

@test "the tap gate fails on a failed update-formula run" {
  dispatch_run
  release_run success
  tap_run 53 repository_dispatch "Update xurl-rs to v9.9.9" failure 2026-11-08T01:06:00Z
  postflight tap
  [ "$status" -eq 1 ]
  [[ "$output" == *"✗ tap update-formula run 53"* ]]
}

@test "a rehearsal, another formula's bump, and a run before the release do not count" {
  dispatch_run
  release_run success
  tap_run 51 workflow_dispatch "Update xurl-rs to v9.9.9 (dry run)" success 2026-11-08T01:06:00Z
  tap_run 52 repository_dispatch "Update bird to v9.9.9" success 2026-11-08T01:06:00Z
  tap_run 54 repository_dispatch "Update xurl-rs to v9.9.9" success 2026-11-08T00:50:00Z
  postflight tap
  [ "$status" -eq 0 ]
  [[ "$output" == *'no run titled "Update xurl-rs to v9.9.9"'* ]]
  [[ "$output" != *"✓"* ]]
}
