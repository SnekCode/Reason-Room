#!/usr/bin/env bash

set -euo pipefail

# Prefer the rustup-managed tools so the pinned toolchain also applies when
# Homebrew's Rust appears earlier in the caller's PATH.
rustup_bin="${CARGO_HOME:-${HOME:-}/.cargo}/bin"
if [[ -x "$rustup_bin/rustup" ]]; then
  export PATH="$rustup_bin:$PATH"
fi

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
launcher="$repository_root/scripts/start-macos-eframe.sh"
web_launcher="$repository_root/scripts/start-web-wasm.sh"
fixture="$repository_root/workspaces/static-react-website.infinite-note.json"

fail() {
  printf 'ERROR: %s\n' "$1" >&2
  exit 1
}

print_refresh_instruction() {
  local workspace_path="$1"
  printf 'Evaluation workspace: %s\n' "$workspace_path"
  printf 'To refresh it: remove this copy, then run ./start.sh again.\n'
}

if [[ "${1:-}" == "--web" ]]; then
  shift
  if [[ ! -x "$web_launcher" ]]; then
    fail "Missing executable WebAssembly launcher: $web_launcher"
  fi
  exec "$web_launcher" "$@"
fi

if [[ "$#" -ne 0 ]]; then
  fail "Unknown start option: $1. Use ./start.sh or ./start.sh --web."
fi

if [[ ! -x "$launcher" ]]; then
  fail "Missing executable eframe launcher: $launcher"
fi

if [[ -n "${INFINITE_NOTE_CANVAS_WORKSPACE:-}" ]]; then
  print_refresh_instruction "$INFINITE_NOTE_CANVAS_WORKSPACE"
  exec "$launcher"
fi

if [[ -z "${HOME:-}" ]]; then
  fail 'HOME must be set to create the evaluation workspace.'
fi

if [[ ! -f "$fixture" ]]; then
  fail "Missing evaluation workspace fixture: $fixture"
fi

workspace_directory="$HOME/Library/Application Support/Infinite Note Canvas"
mkdir -p "$workspace_directory"
workspace_directory="$(cd "$workspace_directory" && pwd -P)"
workspace_path="$workspace_directory/reason-room-static-react-website.infinite-note.json"

repository_workspaces="$repository_root/workspaces"
if [[ "$workspace_directory" == "$repository_workspaces" || "$workspace_directory" == "$repository_workspaces"/* ]]; then
  fail "Evaluation workspace path must be outside repository workspaces: $workspace_path"
fi

if [[ ! -e "$workspace_path" ]]; then
  temporary_workspace="$(mktemp "$workspace_directory/.infinite-note-canvas-eframe-evaluation.XXXXXX")"
  trap 'unlink "$temporary_workspace"' EXIT
  cp "$fixture" "$temporary_workspace"

  if ln "$temporary_workspace" "$workspace_path" 2>/dev/null; then
    unlink "$temporary_workspace"
    trap - EXIT
  else
    unlink "$temporary_workspace"
    trap - EXIT
  fi
fi

export INFINITE_NOTE_CANVAS_FALLBACK_WORKSPACE="$workspace_path"
print_refresh_instruction "$INFINITE_NOTE_CANVAS_FALLBACK_WORKSPACE"
exec "$launcher"
