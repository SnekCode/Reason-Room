#!/usr/bin/env bash

set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
if [[ "$(pwd -P)" != "$repository_root" ]]; then
  printf 'ERROR: Run this check from the repository root: %s\n' "$repository_root" >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  printf 'ERROR: Documentation checks require Python 3.\n' >&2
  exit 1
fi
exec python3 "$repository_root/scripts/check-active-doc-guidance.py"
