#!/usr/bin/env bash
#
# Prints the version .github/tool-versions names for one tool. CI's install steps
# read through here so a name the list lacks stops the step, where a bare `sed`
# would hand the installer an empty version and exit 0.
#
# Exit codes: 0 = printed, 1 = the list names no such tool, 2 = usage.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: scripts/tool-version.sh <tool>" >&2
  exit 2
fi

list="$(dirname "$0")/../.github/tool-versions"
version="$(awk -F= -v tool="$1" '$1 == tool { print $2; exit }' "$list")"
if [ -z "$version" ]; then
  echo "tool-versions/unlisted-tool: .github/tool-versions names no '$1'; add a '$1=<version>' line" >&2
  exit 1
fi
printf '%s\n' "$version"
