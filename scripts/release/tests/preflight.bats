#!/usr/bin/env bats
#
# scripts/release/preflight.sh's smoke and multi-app gates, offline: how they
# address the token store, never what X answers. `xr` is a stub that logs each
# call with the store's state at that moment and writes a store the way xr's
# registration commands do, `cargo` and the 1Password reader are stubs, and HOME
# and TMPDIR sit under the test's own directory.
#
# The canon-drift gate runs against a stubbed `gh` that serves a tarball of a
# fixture tree, whose check-canonical-drift.sh prints canned records: the tests
# hold how the gate fetches, runs, and reads that check, never canon's own
# verdicts.

setup() {
  REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/../../.." && pwd)"
  T="$BATS_TEST_TMPDIR"
  JQ="$(command -v jq || command -v jaq)"
  mkdir -p "$T/bin" "$T/op" "$T/home/tmp"

  if ! command -v jaq >/dev/null; then
    printf '#!/usr/bin/env bash\nexec "%s" "$@"\n' "$JQ" >"$T/bin/jaq"
  fi

  cat >"$T/bin/xr" <<'EOF'
#!/usr/bin/env bash
store="${XURL_TOKEN_STORE:?}"
printf 'store=%s default=%s dev_without_oauth2=%s prod_without_oauth2=%s args=%s\n' "$store" \
  "$(yq '.default_app' "$store" 2>/dev/null)" \
  "$(yq '.apps.bird_dev.oauth2_tokens == null' "$store" 2>/dev/null)" \
  "$(yq '.apps.bird_prod.oauth2_tokens == null' "$store" 2>/dev/null)" "$*" >>"$STUB_XR_LOG"
app=""
args=("$@")
for ((i = 0; i < ${#args[@]}; i++)); do
  [[ "${args[i]}" == --app ]] && app="${args[i + 1]}"
done
case "$*" in
  "auth apps add "*)
    mkdir -p "$(dirname "$store")"
    touch "$store"
    NAME="$4" yq -i '.apps[strenv(NAME)] = {} | .default_app = (.default_app // strenv(NAME))' "$store"
    ;;
  "auth app "*) APP="$app" yq -i '.apps[strenv(APP)].bearer = "stub"' "$store" ;;
  "auth oauth1 "*) APP="$app" yq -i '.apps[strenv(APP)].oauth1 = {"consumer_key": "stub"}' "$store" ;;
  "whoami --app bird_dev --output json")
    if [ -n "${STUB_XR_FAIL_WINDOW:-}" ]; then
      echo "stub: refused by the test" >&2
      exit 1
    fi
    echo '{"data":{"username":"stub"}}'
    ;;
  "whoami --auth app "*)
    echo '{"reason":"auth-method-mismatch"}'
    exit 2
    ;;
  "whoami --app bird_prod --auth oauth1 "*)
    echo '{"reason":"auth-required","message":"TokenNotFound"}'
    exit 77
    ;;
  "whoami --app bird_prod --output json")
    if [ "$(yq '.apps.bird_prod.oauth2_tokens == null' "$store")" = true ]; then
      echo '{"reason":"auth-method-mismatch"}'
      exit 2
    fi
    echo '{"data":{"username":"stub"}}'
    ;;
  "search x "*)
    echo '{"other_apps_with_creds":["bird_dev"]}'
    exit 2
    ;;
  whoami*) echo '{"data":{"username":"stub"}}' ;;
  search*) echo '{"data":[{"id":"1"}]}' ;;
  media*) echo '{"data":{"id":"9"}}' ;;
esac
EOF

  cat >"$T/bin/cargo" <<'EOF'
#!/usr/bin/env bash
echo "cargo $*" >>"$STUB_XR_LOG"
EOF

  cat >"$T/op/read_field.sh" <<'EOF'
#!/usr/bin/env bash
echo "$1|$2" >>"$STUB_OP_LOG"
echo "stub-value"
EOF
  chmod +x "$T"/bin/* "$T/op/read_field.sh"
}

refute_in() {
  ! grep -q -e "$1" "$2"
}

teardown() {
  [[ -z "${OUTSIDE_TMP:-}" ]] || rm -rf -- "$OUTSIDE_TMP"
}

preflight() {
  cd "$REPO_ROOT" && HOME="${PF_HOME:-$T/home}" TMPDIR="${PF_HOME:-$T/home}/tmp" BIN_PATH="$T/bin/xr" OP_SKILL="$T/op" \
    PATH="$T/bin:$PATH" STUB_XR_LOG="$T/xr.log" STUB_OP_LOG="$T/op.log" \
    run bash scripts/release/preflight.sh "$@"
}

# A store in the directory layout, as xr keeps it at ~/.xurl/auth.yml: both
# apps hold an OAuth2 login, and the default is the prod app.
named_store() {
  mkdir -p "$1/.xurl"
  yq -n '.default_app = "bird_prod"
    | .apps.bird_dev.oauth2_tokens.someone = {"type": "oauth2"}
    | .apps.bird_dev.default_user = "someone"
    | .apps.bird_prod.oauth2_tokens.someone = {"type": "oauth2"}' >"$1/.xurl/auth.yml"
  chmod 600 "$1/.xurl/auth.yml"
}

@test "a store --smoke-home names in the directory layout is the one both gates run against, and both leave it as found" {
  named_store "$T/home"
  cp -p "$T/home/.xurl/auth.yml" "$T/before.yml"
  for gate in smoke multi-app; do
    preflight --smoke-home "$T/home" --no-cleanup "$gate"
    [ "$status" -eq 0 ]
  done
  cmp "$T/before.yml" "$T/home/.xurl/auth.yml"
  [ ! -e "$T/home/.xurl/auth.yml.bak" ]
  [ -z "$(find "$T/home/tmp" -name 'xr-preflight-*')" ]
  refute_in "OAuth2 (" "$T/op.log"
  grep -q "^store=$T/home/.xurl/auth.yml default=default .*args=search x " "$T/xr.log"
  grep -q "^store=$T/home/.xurl/auth.yml default=bird_prod dev_without_oauth2=false prod_without_oauth2=true args=whoami --app bird_prod --output json" "$T/xr.log"
  grep -q "^store=$T/home/.xurl/auth.yml default=bird_prod dev_without_oauth2=true prod_without_oauth2=false args=whoami --app bird_dev --output json" "$T/xr.log"
  [ "$(grep -vc -e "^store=$T/home/.xurl/auth.yml " -e "^store=$T/home/tmp/" -e '^cargo ' "$T/xr.log" || true)" -eq 0 ]
}

@test "a store with no default app has none after the smoke gate" {
  named_store "$T/home"
  yq -i 'del(.default_app)' "$T/home/.xurl/auth.yml"
  cp -p "$T/home/.xurl/auth.yml" "$T/before.yml"
  preflight --smoke-home "$T/home" --no-cleanup smoke
  [ "$status" -eq 0 ]
  grep -q "^store=$T/home/.xurl/auth.yml default=default .*args=search x " "$T/xr.log"
  cmp "$T/before.yml" "$T/home/.xurl/auth.yml"
}

@test "a call that fails inside the multi-app window leaves the store as found" {
  named_store "$T/home"
  cp -p "$T/home/.xurl/auth.yml" "$T/before.yml"
  STUB_XR_FAIL_WINDOW=1 preflight --smoke-home "$T/home" --no-cleanup multi-app
  grep -q "^store=$T/home/.xurl/auth.yml default=bird_prod dev_without_oauth2=true prod_without_oauth2=false args=whoami --app bird_dev --output json" "$T/xr.log"
  cmp "$T/before.yml" "$T/home/.xurl/auth.yml"
  [ ! -e "$T/home/.xurl/auth.yml.bak" ]
}

@test "a --smoke-home that holds no store refuses naming the path it read, calls nothing, and is not shredded" {
  for gate in smoke multi-app; do
    mkdir -p "$T/home/named"
    echo keep >"$T/home/named/notes.txt"
    preflight --smoke-home "$T/home/named" "$gate"
    [ "$status" -eq 2 ]
    [[ "$output" == *"no token store at $T/home/named/.xurl/auth.yml"* ]]
    [ ! -e "$T/xr.log" ]
    [ ! -e "$T/op.log" ]
    [ -z "$(find "$T/home/tmp" -name 'xr-preflight-*')" ]
    [ "$(cat "$T/home/named/notes.txt")" = keep ]
  done
}

@test "a single-file store at <home>/.xurl is not read as the store, and is left alone" {
  yq -n '.default_app = "bird_prod"' >"$T/home/.xurl"
  cp -p "$T/home/.xurl" "$T/before.yml"
  preflight --smoke-home "$T/home" --no-cleanup smoke
  [ "$status" -eq 2 ]
  [[ "$output" == *"no token store at $T/home/.xurl/auth.yml"* ]]
  [ ! -e "$T/xr.log" ]
  cmp "$T/before.yml" "$T/home/.xurl"
}

@test "a store the gate seeds is written in the directory layout, through the store path xr takes" {
  preflight --no-cleanup smoke
  [ "$status" -eq 0 ]
  seeded=$(find "$T/home/tmp" -maxdepth 1 -name 'xr-preflight-*')
  [ -n "$seeded" ]
  [ -f "$seeded/.xurl/auth.yml" ]
  [ "$(yq '.apps | keys | join(",")' "$seeded/.xurl/auth.yml")" = "bird_dev,bird_prod" ]
  [ "$(yq '.default_app' "$seeded/.xurl/auth.yml")" = bird_dev ]
  refute_in "^store=$seeded/.xurl " "$T/xr.log"
}

@test "a store the gate seeds is shredded on exit unless --no-cleanup is given" {
  preflight smoke
  [ "$status" -eq 0 ]
  [ -z "$(find "$T/home/tmp" -maxdepth 1 -name 'xr-preflight-*')" ]
}

# The exit trap shreds only under /tmp and $HOME, and matches `"$HOME"/*`, which
# a trailing slash or a `..` brings the home itself under. The home sits outside
# /tmp so that only the $HOME arm of the guard can let it through.
@test "a --smoke-home that names \$HOME by another spelling is not shredded" {
  OUTSIDE_TMP=$(mktemp -d /var/tmp/xr-preflight-bats-XXXXXX)
  PF_HOME="$OUTSIDE_TMP"
  mkdir -p "$PF_HOME/tmp" "$PF_HOME/sub"
  for spelling in "$PF_HOME/" "$PF_HOME/sub/.." "$PF_HOME//./"; do
    named_store "$PF_HOME"
    echo keep >"$PF_HOME/notes.txt"
    preflight --smoke-home "$spelling" smoke
    [[ "$output" == *"refusing to shred outside /tmp or \$HOME: $PF_HOME"* ]]
    [ -f "$PF_HOME/.xurl/auth.yml" ]
    [ "$(cat "$PF_HOME/notes.txt")" = keep ]
  done
}

IN_SYNC='{"reason":"in-sync","dest":".editorconfig","message":"matches","copy_stamp":null,"source_stamp":null,"copy_variant":null,"source_variant":null}
{"reason":"in-sync","dest":"rustfmt.toml","message":"matches","copy_stamp":null,"source_stamp":null,"copy_variant":null,"source_variant":null}
{"reason":"repo-owned","dest":"scripts/generate-completions.sh","message":"declared repo-owned in the copy","copy_stamp":"2026.06.22","source_stamp":null,"copy_variant":"rust","source_variant":null}
{"reason":"skeleton","dest":"scripts/hooks/pre-push","message":"present","copy_stamp":null,"source_stamp":null,"copy_variant":null,"source_variant":null}
{"reason":"absent","dest":"cliff.toml","message":"not vendored in this repo","copy_stamp":null,"source_stamp":null,"copy_variant":null,"source_variant":null}'
DRIFTED='{"reason":"drift","dest":"scripts/cargo-inputs.sh","message":"differs from skills:rust-tool-release/scripts/cargo-inputs.sh (copy stamp 2026.11.03 (rust), source stamp 2026.11.05 (rust)); refresh: cp /gone/cargo-inputs.sh /repo/scripts/cargo-inputs.sh","copy_stamp":"2026.11.03","source_stamp":"2026.11.05","copy_variant":"rust","source_variant":"rust"}'
SHA=0123456789abcdef0123456789abcdef01234567

# canon_fixture RECORDS EXIT: `gh` answers the commit lookup with $SHA and the
# tarball request with a tree, under a top directory as GitHub names it, whose
# check prints RECORDS and exits EXIT. The check logs its arguments and the
# credential and home it was given.
canon_fixture() {
  local top="brettdavies-agent-skills-0123456" check
  CANON="$T/canon/$top"
  check="$CANON/github-repo-setup/scripts/check-canonical-drift.sh"
  mkdir -p "$(dirname "$check")"
  printf '%s\n' "$1" >"$CANON/records.jsonl"
  printf '%s\n' "$2" >"$CANON/exit"
  cat >"$check" <<EOF
#!/usr/bin/env bash
printf 'args=%s\n' "\$*" >>"$T/check.log"
printf 'gh_token=%s\n' "\${GH_TOKEN-unset}" >>"$T/check.log"
printf 'home=%s\n' "\$HOME" >>"$T/check.log"
cat "\$(dirname "\$0")/../../records.jsonl"
exit "\$(cat "\$(dirname "\$0")/../../exit")"
EOF
  cat >"$T/bin/gh" <<EOF
#!/usr/bin/env bash
echo "gh \$*" >>"$T/gh.log"
[ -z "\${STUB_GH_FAIL:-}" ] || { echo "gh: HTTP 404: Not Found" >&2; exit 1; }
case "\$*" in
  "api repos/brettdavies/agent-skills/commits/main --jq .sha") echo $SHA ;;
  "api repos/brettdavies/agent-skills/tarball/$SHA") tar -czf - -C "$T/canon" "$top" ;;
  *) echo "stub gh: \$*" >&2; exit 1 ;;
esac
EOF
  chmod +x "$check" "$T/bin/gh"
}

canon_tree_left() {
  find "$T/home/tmp" -maxdepth 1 -name 'xr-canon-*'
}

@test "canon-drift passes when every copy is in sync, names the canon commit, and hands the check no credential" {
  canon_fixture "$IN_SYNC" 0
  GH_TOKEN=leak-canary preflight canon-drift
  [ "$status" -eq 0 ]
  [[ "$output" == *"canon: brettdavies/agent-skills main at $SHA"* ]]
  [[ "$output" == *"2 copies match brettdavies/agent-skills at ${SHA:0:12}; declared repo-owned, so not compared: scripts/generate-completions.sh"* ]]
  grep -qx "args=--json $REPO_ROOT" "$T/check.log"
  grep -qx "gh_token=unset" "$T/check.log"
  grep -qx "home=$T/home" "$T/check.log"
  [ -z "$(canon_tree_left)" ]
}

@test "canon-drift fails a drifted copy, naming the file, both stamps, and the source, and removes the tree" {
  canon_fixture "$IN_SYNC
$DRIFTED" 1
  preflight canon-drift
  [ "$status" -eq 1 ]
  [[ "$output" == *"main at $SHA"* ]]
  [[ "$output" == *"scripts/cargo-inputs.sh: drift, copy stamp 2026.11.03 (rust), canon stamp 2026.11.05 (rust)"* ]]
  [[ "$output" == *"differs from skills:rust-tool-release/scripts/cargo-inputs.sh"* ]]
  [[ "$output" != *"refresh: cp"* ]]
  [[ "$output" == *"fix: refresh each file from its source in brettdavies/agent-skills at ${SHA:0:12}"* ]]
  [ -z "$(canon_tree_left)" ]
}

@test "canon-drift fails a copy whose canonical source the check could not read, though the check exits 0" {
  canon_fixture "$IN_SYNC
"'{"reason":"source-missing","dest":".editorconfig","message":"canonical source not found: ~/dotfiles/stow/claude/dot-editorconfig","copy_stamp":null,"source_stamp":null,"copy_variant":null,"source_variant":null}' 0
  preflight canon-drift
  [ "$status" -eq 1 ]
  [[ "$output" == *".editorconfig: source-missing, copy stamp none, canon stamp none"* ]]
}

@test "canon-drift fails when canon cannot be fetched, runs no check, and says how to log in" {
  canon_fixture "$IN_SYNC" 0
  STUB_GH_FAIL=1 preflight canon-drift
  [ "$status" -eq 1 ]
  [[ "$output" == *"could not read the current commit of brettdavies/agent-skills main"* ]]
  [[ "$output" == *"HTTP 404"* ]]
  [[ "$output" == *"fix: check gh auth status"* ]]
  [ ! -e "$T/check.log" ]
  [ -z "$(canon_tree_left)" ]
}

@test "canon-drift fails when the tarball is not a tree with the check in it" {
  canon_fixture "$IN_SYNC" 0
  rm -f "$CANON/github-repo-setup/scripts/check-canonical-drift.sh"
  preflight canon-drift
  [ "$status" -eq 1 ]
  [[ "$output" == *"has no github-repo-setup/scripts/check-canonical-drift.sh"* ]]
  [ -z "$(canon_tree_left)" ]
}

@test "canon-drift fails when the check cannot run, and when it compares nothing" {
  canon_fixture "$IN_SYNC" 2
  preflight canon-drift
  [ "$status" -eq 1 ]
  [[ "$output" == *"the check exited 2 before it compared the copies"* ]]
  canon_fixture '{"reason":"absent","dest":"cliff.toml","message":"not vendored in this repo","copy_stamp":null,"source_stamp":null,"copy_variant":null,"source_variant":null}' 0
  preflight canon-drift
  [ "$status" -eq 1 ]
  [[ "$output" == *"the check reported no copy in sync, so it compared nothing"* ]]
  [ -z "$(canon_tree_left)" ]
}

@test "canon-drift is a subcommand, is listed in --help, and is run by all" {
  preflight --help
  [ "$status" -eq 0 ]
  [[ "$output" == *"canon-drift"* ]]
  sed -n '/^  all)/,/^    ;;/p' "$REPO_ROOT/scripts/release/preflight.sh" | grep -qx '    gate_canon_drift'
}
