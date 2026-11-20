#!/usr/bin/env bats
#
# scripts/release/tag-release.sh against a throwaway workspace whose remote is a
# local bare repository. `gh` is a stub that answers from fixture files, and
# `git push` goes through a wrapper that can be told to fail.

setup() {
  REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/../../.." && pwd)"
  T="$BATS_TEST_TMPDIR"
  REAL_GIT="$(command -v git)"
  JQ="$(command -v jq || command -v jaq)"
  mkdir -p "$T/bin" "$T/gh/runs" "$T/gh/jobs"

  cat >"$T/bin/git" <<EOF
#!/usr/bin/env bash
if [ "\$1" = push ]; then
  echo "\$*" >>"$T/push.log"
  if [ -n "\${STUB_PUSH_FAIL:-}" ]; then
    echo "remote: refused by the test" >&2
    exit 1
  fi
fi
exec "$REAL_GIT" "\$@"
EOF

  # Answers `gh api <path> --jq <filter>` from \$T/gh, and `gh repo view`.
  cat >"$T/bin/gh" <<EOF
#!/usr/bin/env bash
if [ -n "\${STUB_GH_FAIL:-}" ]; then
  echo "HTTP 401: Bad credentials" >&2
  exit 1
fi
if [ "\$1 \$2" = "repo view" ]; then
  echo o/app
  exit 0
fi
[ "\$1" = api ] || { echo "gh stub: unhandled \$*" >&2; exit 1; }
shift
path="" filter="."
while [ \$# -gt 0 ]; do
  case "\$1" in
    --jq) filter="\$2"; shift 2 ;;
    --paginate) shift ;;
    *) path="\$1"; shift ;;
  esac
done
case "\$path" in
  user/repos\?*type=owner*) file="$T/gh/repos-owned.json" ;;
  repos/*/actions/runs/*/jobs\?per_page=1)
    if [ -n "\${STUB_JOBS_FAIL:-}" ]; then
      echo "HTTP 502: jobs read refused by the test" >&2
      exit 1
    fi
    repo="\${path#repos/}"; repo="\${repo%/actions/*}"
    id="\${path#*/runs/}"; id="\${id%%/*}"
    file="$T/gh/jobs/\${repo/\//_}-\${id}.json"
    [ -f "\$file" ] || { echo "gh stub: no jobs fixture for \$path" >&2; exit 1; }
    ;;
  user/repos*) file="$T/gh/repos-all.json" ;;
  repos/*/actions/runs\?status=*)
    repo="\${path#repos/}"; repo="\${repo%/actions/*}"
    status="\${path#*status=}"; status="\${status%%&*}"
    file="$T/gh/runs/\${repo/\//_}-\${status}.json"
    [ -f "\$file" ] || file="$T/gh/runs/none.json"
    ;;
  *) echo "gh stub: unhandled path \$path" >&2; exit 1 ;;
esac
exec "$JQ" -r "\$filter" "\$file"
EOF

  if ! command -v jaq >/dev/null; then
    printf '#!/usr/bin/env bash\nexec "%s" "$@"\n' "$JQ" >"$T/bin/jaq"
  fi
  chmod +x "$T"/bin/*

  echo '{"workflow_runs": []}' >"$T/gh/runs/none.json"
  cat >"$T/gh/repos-owned.json" <<'EOF'
[{"full_name": "o/app", "archived": false, "pushed_at": "2026-11-07T00:00:00Z"},
 {"full_name": "o/bird", "archived": false, "pushed_at": "2026-11-07T00:00:00Z"},
 {"full_name": "o/quiet", "archived": false, "pushed_at": "2025-01-01T00:00:00Z"},
 {"full_name": "o/old", "archived": true, "pushed_at": "2024-01-01T00:00:00Z"}]
EOF
  "$JQ" '. + [{"full_name": "x/foreign", "archived": false, "pushed_at": "2026-11-07T00:00:00Z"}]' \
    "$T/gh/repos-owned.json" >"$T/gh/repos-all.json"

  "$REAL_GIT" init --quiet --bare "$T/origin.git"
  W="$T/work"
  mkdir -p "$W/crates/app/src" "$W/crates/corelib/src" "$W/scripts/release" "$W/.github/workflows"
  printf '[workspace]\nmembers = ["crates/app", "crates/corelib"]\nresolver = "2"\n' >"$W/Cargo.toml"
  printf '[package]\nname = "app"\nversion = "1.2.0"\nedition = "2021"\n' >"$W/crates/app/Cargo.toml"
  printf '[package]\nname = "corelib"\nversion = "0.3.0"\nedition = "2021"\n' >"$W/crates/corelib/Cargo.toml"
  echo 'fn main() {}' >"$W/crates/app/src/main.rs"
  : >"$W/crates/corelib/src/lib.rs"
  printf 'on:\n  push:\n  workflow_dispatch:\n' >"$W/.github/workflows/release.yml"
  printf 'target/\n' >"$W/.gitignore"
  cp "$REPO_ROOT/scripts/release/tag-release.sh" "$REPO_ROOT/scripts/release/_lib.sh" "$W/scripts/release/"
  echo 'RELEASE_MANIFEST="crates/app/Cargo.toml"' >"$W/scripts/release/release.env"
  (
    cd "$W" || exit 1
    cargo generate-lockfile --offline --quiet
    "$REAL_GIT" init --quiet -b main
    "$REAL_GIT" config user.name test
    "$REAL_GIT" config user.email test@example.invalid
    "$REAL_GIT" config commit.gpgsign false
    "$REAL_GIT" config tag.gpgsign false
    "$REAL_GIT" add -A
    "$REAL_GIT" commit --quiet -m "release: v1.2.0"
    "$REAL_GIT" remote add origin "$T/origin.git"
    "$REAL_GIT" push --quiet -u origin main
  )
}

tag_release() {
  cd "$W" && PATH="$T/bin:$PATH" run bash scripts/release/tag-release.sh "$@"
}

run_record() {
  # a queued or in-progress run for a repo: run_record <owner/repo> <status> <event> [age-hours]
  local created
  created=$("$JQ" -n -r --argjson hours "${4:-0}" 'now - ($hours * 3600) | floor | todate')
  printf '{"workflow_runs": [{"id": 7, "repository": {"full_name": "%s"}, "name": "CI", "status": "%s", "event": "%s", "html_url": "https://github.com/%s/actions/runs/7", "created_at": "%s"}]}\n' \
    "$1" "$2" "$3" "$1" "$created" >"$T/gh/runs/${1/\//_}-$2.json"
}

run_created() { "$JQ" -r '.workflow_runs[0].created_at' "$T/gh/runs/${1/\//_}-$2.json"; }

jobs_record() {
  # the jobs a repo's run 7 reports: jobs_record <owner/repo> <total_count>
  printf '{"total_count": %s, "jobs": []}\n' "$2" >"$T/gh/jobs/${1/\//_}-7.json"
}

remote_tags() { "$REAL_GIT" --git-dir "$T/origin.git" tag --list | sort | tr '\n' ' '; }
local_tags() { "$REAL_GIT" -C "$W" tag --list | tr '\n' ' '; }

@test "a release that moves both crates pushes two annotated tags in one push" {
  tag_release
  [ "$status" -eq 0 ]
  [ "$(remote_tags)" = "corelib-v0.3.0 v1.2.0 " ]
  [ "$("$REAL_GIT" --git-dir "$T/origin.git" cat-file -t refs/tags/v1.2.0)" = tag ]
  [ "$("$REAL_GIT" --git-dir "$T/origin.git" cat-file -t refs/tags/corelib-v0.3.0)" = tag ]
  [ "$(wc -l <"$T/push.log")" -eq 1 ]
  [[ "$(cat "$T/push.log")" == "push --atomic origin refs/tags/corelib-v0.3.0 refs/tags/v1.2.0" ]]
}

@test "a library version already tagged leaves one tag to push" {
  "$REAL_GIT" -C "$W" tag -a -m old corelib-v0.3.0
  "$REAL_GIT" -C "$W" push --quiet origin corelib-v0.3.0
  : >"$T/push.log"
  tag_release
  [ "$status" -eq 0 ]
  [[ "$output" == *"corelib 0.3.0 is already tagged corelib-v0.3.0"* ]]
  [[ "$(cat "$T/push.log")" == "push --atomic origin refs/tags/v1.2.0" ]]
}

@test "success names the runs to watch and the postflight runs" {
  tag_release
  [ "$status" -eq 0 ]
  [[ "$output" == *"gh run list --repo o/app --branch corelib-v0.3.0"* ]]
  [[ "$output" == *"gh run list --repo o/app --branch v1.2.0"* ]]
  [[ "$output" == *"gh run list --repo o/app --workflow release.yml --event workflow_dispatch --limit 3"* ]]
  [[ "$output" == *"scripts/release/postflight.sh --tag corelib-v0.3.0 all"* ]]
  [[ "$output" == *"scripts/release/postflight.sh --tag v1.2.0 all"* ]]
}

@test "a run in another owned repository holds the cut, and --force pushes" {
  run_record o/bird in_progress push
  tag_release
  [ "$status" -eq 1 ]
  [[ "$output" == *"error[runners-busy]"* ]]
  [[ "$output" == *"o/bird"*"https://github.com/o/bird/actions/runs/7"* ]]
  [[ "$output" == *"see: RELEASES.md#runners-busy"* ]]
  [ -z "$(local_tags)" ]
  [ -z "$(remote_tags)" ]

  tag_release --force
  [ "$status" -eq 0 ]
  [ "$(remote_tags)" = "corelib-v0.3.0 v1.2.0 " ]
}

@test "a scheduled run in an owned repository with no recent push holds the cut" {
  run_record o/quiet queued schedule
  tag_release
  [ "$status" -eq 1 ]
  [[ "$output" == *"o/quiet"* ]]
  [ -z "$(local_tags)" ]
}

@test "runs in a repository the account does not own, or has archived, are not listed" {
  run_record x/foreign in_progress push
  run_record o/old in_progress push
  tag_release
  [ "$status" -eq 0 ]
  [[ "$output" != *"x/foreign"* ]]
  [[ "$output" != *"o/old"* ]]
}

@test "a queued run a day old with no job passes the gate and is named as skipped" {
  run_record o/app queued push 25
  jobs_record o/app 0
  tag_release
  [ "$status" -eq 0 ]
  created=$(run_created o/app queued)
  line=$(grep "skipped run in" <<<"$output")
  [ "$(wc -l <<<"$line")" -eq 1 ]
  [[ "$line" == *"o/app"*"CI"*"$created"*"https://github.com/o/app/actions/runs/7"* ]]
  [[ "$output" == *"0 skipped"* ]]
  [ "$(remote_tags)" = "corelib-v0.3.0 v1.2.0 " ]
}

@test "a queued run under a day old holds the cut even with no job, beside a skipped older one" {
  run_record o/app queued push 23
  jobs_record o/app 0
  run_record o/quiet queued schedule 25
  jobs_record o/quiet 0
  tag_release
  [ "$status" -eq 1 ]
  [[ "$output" == *"error[runners-busy]: 1 workflow run(s) hold the account's runners"* ]]
  [[ "$output" == *"o/app"*"https://github.com/o/app/actions/runs/7"* ]]
  line=$(grep "skipped run in" <<<"$output")
  [[ "$line" == *"o/quiet"* ]]
  [[ "$line" != *"o/app"* ]]
  [ -z "$(local_tags)" ]
}

@test "a queued run a day old that has a job holds the cut, beside a skipped one with none" {
  run_record o/app queued push 25
  jobs_record o/app 1
  run_record o/quiet queued schedule 25
  jobs_record o/quiet 0
  tag_release
  [ "$status" -eq 1 ]
  [[ "$output" == *"error[runners-busy]: 1 workflow run(s) hold the account's runners"* ]]
  [[ "$output" == *"o/app"*"https://github.com/o/app/actions/runs/7"* ]]
  line=$(grep "skipped run in" <<<"$output")
  [[ "$line" == *"o/quiet"* ]]
  [[ "$line" != *"o/app"* ]]
  [ -z "$(local_tags)" ]
}

@test "an in-progress run a day old holds the cut whatever its jobs" {
  run_record o/app in_progress push 25
  jobs_record o/app 0
  tag_release
  [ "$status" -eq 1 ]
  [[ "$output" == *"error[runners-busy]: 1 workflow run(s) hold the account's runners"* ]]
  [[ "$output" == *"o/app"*"https://github.com/o/app/actions/runs/7"* ]]
  [[ "$output" != *"skipped run in"* ]]
}

@test "a skipped line keeps its fields when the run's workflow has no name" {
  run_record o/app queued push 25
  "$JQ" '.workflow_runs[0].name = null' "$T/gh/runs/o_app-queued.json" >"$T/gh/runs/unnamed.json"
  cp "$T/gh/runs/unnamed.json" "$T/gh/runs/o_app-queued.json"
  jobs_record o/app 0
  tag_release
  [ "$status" -eq 0 ]
  created=$(run_created o/app queued)
  [[ "$(grep "skipped run in" <<<"$output")" == *"o/app: -, queued since ${created} with no job, https://github.com/o/app/actions/runs/7"* ]]
}

@test "a failed jobs read refuses runner-check-failed, and the next run after it clears succeeds" {
  run_record o/app queued push 25
  jobs_record o/app 0
  STUB_JOBS_FAIL=1 tag_release
  [ "$status" -eq 2 ]
  [[ "$output" == *"error[runner-check-failed]"*"jobs read refused by the test"*"(reading the jobs of run 7 in o/app)"* ]]
  [ -z "$(local_tags)" ]

  tag_release
  [ "$status" -eq 0 ]
}

@test "an unreadable runner check refuses, and the next run after it clears succeeds" {
  STUB_GH_FAIL=1 tag_release
  [ "$status" -eq 2 ]
  [[ "$output" == *"error[runner-check-failed]"*"Bad credentials"* ]]
  [ -z "$(local_tags)" ]

  tag_release
  [ "$status" -eq 0 ]
}

@test "main behind origin/main refuses with the pull, and the next run after it succeeds" {
  "$REAL_GIT" -C "$W" commit --quiet --allow-empty -m "later"
  "$REAL_GIT" -C "$W" push --quiet origin main
  "$REAL_GIT" -C "$W" reset --quiet --hard HEAD~1
  tag_release
  [ "$status" -eq 2 ]
  [[ "$output" == *"error[main-not-current]: main is 1 commit(s) behind origin/main"* ]]
  [[ "$output" == *"next: git pull --ff-only"* ]]
  [ -z "$(local_tags)" ]

  "$REAL_GIT" -C "$W" pull --quiet --ff-only
  tag_release
  [ "$status" -eq 0 ]
}

@test "a checkout off main, or with local commits, refuses" {
  "$REAL_GIT" -C "$W" checkout --quiet -b feat
  tag_release
  [ "$status" -eq 2 ]
  [[ "$output" == *"error[main-not-current]: the checkout is on feat, not main"* ]]

  "$REAL_GIT" -C "$W" checkout --quiet main
  "$REAL_GIT" -C "$W" commit --quiet --allow-empty -m "unpushed"
  tag_release
  [ "$status" -eq 2 ]
  [[ "$output" == *"main carries 1 commit(s) origin/main does not"* ]]
  [ -z "$(local_tags)" ]
}

@test "a dirty working tree refuses" {
  echo change >>"$W/Cargo.toml"
  tag_release
  [ "$status" -eq 2 ]
  [[ "$output" == *"error[dirty-tree]"*"Cargo.toml"* ]]
  [ -z "$(local_tags)" ]
}

@test "the binary tag on the remote refuses, and a local-only one names its deletion" {
  "$REAL_GIT" -C "$W" tag -a -m old v1.2.0
  tag_release
  [ "$status" -eq 2 ]
  [[ "$output" == *"error[tag-exists]: v1.2.0 exists in this checkout but not on origin"* ]]
  [[ "$output" == *"next: git tag -d v1.2.0"* ]]

  "$REAL_GIT" -C "$W" push --quiet origin v1.2.0
  tag_release
  [ "$status" -eq 2 ]
  [[ "$output" == *"error[tag-exists]: v1.2.0 is already on origin"* ]]
  [ "$(local_tags)" = "v1.2.0 " ]
}

@test "a workspace cargo cannot read refuses before any tag" {
  real_cargo="$(command -v cargo)"
  printf '#!/usr/bin/env bash\nif [ "$1" = metadata ]; then echo "error: toolchain 1.96.0 is not installed" >&2; exit 101; fi\nexec "%s" "$@"\n' "$real_cargo" >"$T/bin/cargo"
  chmod +x "$T/bin/cargo"
  tag_release
  [ "$status" -eq 2 ]
  [[ "$output" == *"error[workspace-unreadable]"* ]]
  [[ "$output" == *"toolchain 1.96.0 is not installed"* ]]
  [ -z "$(local_tags)" ]
  [ -z "$(remote_tags)" ]
}

@test "an unreachable remote refuses before any tag" {
  "$REAL_GIT" -C "$W" remote set-url origin "$T/missing.git"
  tag_release
  [ "$status" -eq 2 ]
  [[ "$output" == *"error[remote-unreachable]"* ]]
  [ -z "$(local_tags)" ]
}

@test "a failed push deletes the tags it created and names the failure" {
  STUB_PUSH_FAIL=1 tag_release
  [ "$status" -eq 1 ]
  [[ "$output" == *"error[push-failed]"*"remote: refused by the test"* ]]
  [ -z "$(local_tags)" ]
  [ -z "$(remote_tags)" ]

  tag_release
  [ "$status" -eq 0 ]
}

@test "--dry-run prints the tags and the runner check and creates nothing" {
  tag_release --dry-run
  [ "$status" -eq 0 ]
  [[ "$output" == *"will tag corelib-v0.3.0"* ]]
  [[ "$output" == *"no queued or in-progress run holds the account's runners in the 3 repositories the account owns"* ]]
  [[ "$output" == *"Dry run: would tag corelib-v0.3.0 v1.2.0"* ]]
  [ -z "$(local_tags)" ]
  [ ! -e "$T/push.log" ] || ! grep -q refs/tags "$T/push.log"
}
