#!/usr/bin/env bash
#
# Installs every tool CI and the git hooks use, at the versions .github/tool-versions
# names, so a fresh clone passes pre-push after this one command. Rust tools go
# through cargo binstall and the rest through Homebrew, into the user's own
# prefixes: nothing here runs sudo or writes to a system path.
#
# A tool at or above its listed version is left alone and reported as current, so a
# rerun changes nothing. A newer local tool stays, with both versions in its line:
# CI's pinned run is the gate of record, and the hooks are its fast approximation.
#
# Exit codes: 0 = every tool is current or was installed (--dry-run: nothing was
# touched, and --root: a tool with no root is only reported), 1 = a prerequisite is
# missing or an install failed, named as dev-setup/<tool> with the manual command,
# 2 = usage.
#
# Stays within bash 3.2, the macOS /bin/bash: the bash the hooks need is one of the
# tools this installs, so it cannot be assumed here.
set -euo pipefail

usage() {
  cat <<'EOF'
Install the tools CI and the git hooks use, at the versions .github/tool-versions names.

Usage: scripts/dev-setup.sh [--root DIR] [--dry-run]

Options:
  --root DIR   Put Rust tools under DIR/bin and look there first. Homebrew has no
               install root, so a missing linter is reported as "would install".
  --dry-run    Report what would be installed and change nothing.
  -h, --help   Print this help.

Each tool prints one line: current, installed, or would install.

Exit codes: 0 = done, 1 = dev-setup/<tool> and the manual command, 2 = usage.
EOF
}

usage_error() {
  printf 'dev-setup/usage: %s; run scripts/dev-setup.sh --help\n' "$1" >&2
  exit 2
}

root=''
dry_run=false
while [ "$#" -gt 0 ]; do
  case "$1" in
    --root)
      [ "$#" -ge 2 ] || usage_error '--root needs a directory'
      root=$2
      shift 2
      ;;
    --root=*)
      root=${1#--root=}
      shift
      ;;
    --dry-run)
      dry_run=true
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *) usage_error "unknown argument '$1'" ;;
  esac
done

if [ -n "$root" ]; then
  case "$root" in
    /*) ;;
    *) root="$PWD/$root" ;;
  esac
  PATH="$root/bin:$PATH"
  export PATH
fi

# From the repo root, so cargo resolves rust-toolchain.toml's pin for every probe.
cd "$(dirname "$0")/.."

failed=false
log="$(mktemp)"
trap 'rm -f "$log"' EXIT

report() { printf '%-18s %s\n' "$1" "$2"; }

# dev-setup/<tool>, the reason, then the command that installs it by hand.
stop() {
  printf "dev-setup/%s: %s; install via \`%s\`\n" "$1" "$2" "$3" >&2
  failed=true
}

have() { command -v "$1" >/dev/null 2>&1; }

# Runs an installer with its output held back, and shows that output on a failure.
quietly() {
  if "$@" >"$log" 2>&1 </dev/null; then
    return 0
  fi
  cat "$log" >&2
  return 1
}

# Sets how a listed tool installs and how to ask for its version, the way the hooks
# reach it. markdownlint-cli2 has no version flag: it reads `--version` as a glob and
# goes looking for files, and its help carries the same version line.
recipe() {
  case "$1" in
    agentnative | cargo-deny | cargo-hack | cargo-nextest) installer=cargo ;;
    actionlint | bash | bats-core | markdownlint-cli2 | ripgrep | shellcheck) installer=brew ;;
    *) return 1 ;;
  esac
  case "$1" in
    agentnative) probe=(anc --version) ;;
    bats-core) probe=(bats --version) ;;
    cargo-*) probe=(cargo "${1#cargo-}" --version) ;;
    markdownlint-cli2) probe=(markdownlint-cli2 --help) ;;
    ripgrep) probe=(rg --version) ;;
    *) probe=("$1" --version) ;;
  esac
}

# The first x.y.z the probe prints, or nothing. A missing tool fails the probe, and
# markdownlint-cli2's help exits non-zero, so the output decides and not the status.
found_version() {
  local output
  output="$("${probe[@]}" 2>/dev/null </dev/null || true)"
  if [[ $output =~ [0-9]+\.[0-9]+\.[0-9]+ ]]; then
    printf '%s' "${BASH_REMATCH[0]}"
  fi
}

# True when version $1 is at least version $2.
at_least() {
  local newest
  newest="$(printf '%s\n%s\n' "$1" "$2" | sort -t. -k1,1n -k2,2n -k3,3n | tail -n 1)"
  [ "$newest" = "$1" ]
}

# Resolved at the first Rust install, so a machine that needs none is never asked
# for cargo-binstall. Without it, Rust tools build from source through cargo install.
binstall_state=unknown
binstall_ready() {
  if [ "$binstall_state" = unknown ]; then
    binstall_state=absent
    if cargo binstall -V >/dev/null 2>&1; then
      binstall_state=ready
    elif [ "$dry_run" = true ] || [ -n "$root" ]; then
      report cargo-binstall "missing: Rust tools would build from source with cargo install"
    elif have brew && quietly brew install cargo-binstall && cargo binstall -V >/dev/null 2>&1; then
      binstall_state=ready
      report cargo-binstall "installed"
    else
      report cargo-binstall "missing: Rust tools build from source with cargo install"
    fi
  fi
  [ "$binstall_state" = ready ]
}

# Sets `install` to the command that puts a Rust tool in place.
rust_install_command() {
  if binstall_ready; then
    install=(cargo binstall "$1@$2" --force --no-confirm)
  else
    install=(cargo install "$1@$2" --locked --force)
  fi
  if [ -n "$root" ]; then
    install+=(--root "$root")
  fi
}

ensure_tool() {
  local tool=$1 version=$2 found was manual installer
  local -a probe install
  if ! recipe "$tool"; then
    stop "$tool" "the list names it, but scripts/dev-setup.sh has no recipe for it" \
      "add a '$tool' case to recipe() in scripts/dev-setup.sh"
    return 0
  fi

  found="$(found_version)"
  if [ -n "$found" ] && at_least "$found" "$version"; then
    if [ "$found" = "$version" ]; then
      report "$tool" "current $found"
    else
      report "$tool" "current $found (the list names $version)"
    fi
    return 0
  fi
  was=${found:-missing}

  if [ "$installer" = cargo ]; then
    rust_install_command "$tool" "$version"
    manual="cargo install $tool@$version --locked"
    if [ -n "$root" ]; then
      manual="$manual --root $root"
    fi
  else
    install=(brew install "$tool")
    manual="brew install $tool"
  fi

  if [ "$dry_run" = true ]; then
    report "$tool" "would install $version (now $was): ${install[*]}"
    return 0
  fi
  if [ "$installer" = brew ]; then
    if [ -n "$root" ]; then
      report "$tool" "would install $version (now $was): ${install[*]}; Homebrew has no install root"
      return 0
    fi
    if ! have brew; then
      stop "$tool" "$was, below the listed $version, and Homebrew is not on PATH (https://brew.sh)" "$manual"
      return 0
    fi
  fi

  if ! quietly "${install[@]}"; then
    stop "$tool" "\`${install[*]}\` failed" "$manual"
    return 0
  fi

  # The same check CI's install steps make: an older copy earlier on PATH, or an
  # install directory PATH does not reach, leaves the tool unusable despite the
  # installer's success.
  found="$(found_version)"
  if [ -z "$found" ] || ! at_least "$found" "$version"; then
    stop "$tool" "installed, but \`${probe[*]}\` reports ${found:-nothing} where the list names $version; check PATH order" "$manual"
    return 0
  fi
  report "$tool" "installed $found (was $was)"
}

if ! have cargo; then
  stop cargo "not on PATH, and it runs every Rust tool" \
    "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  exit 1
fi

# Read on its own descriptor: an installer that reads stdin would otherwise eat the
# rest of the list.
while IFS='=' read -r tool version <&3; do
  case "$tool" in
    '' | '#'*) continue ;;
  esac
  ensure_tool "$tool" "$version"
done 3<.github/tool-versions

if [ "$failed" = true ]; then
  exit 1
fi
