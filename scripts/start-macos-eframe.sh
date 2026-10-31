#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'ERROR: %s\n' "$1" >&2
  printf 'ACTION: %s\n' "$2" >&2
  exit 1
}

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$repository_root"

workspace_fixture="$repository_root/workspaces/gpui-evaluation.fixture.json"
bundle_info_template="$repository_root/crates/app/resources/macos/eframe-evaluation-Info.plist"
workspace_environment_variable='INFINITE_NOTE_CANVAS_WORKSPACE'
fallback_workspace_environment_variable='INFINITE_NOTE_CANVAS_FALLBACK_WORKSPACE'
candidate_bundle="$repository_root/target/release/Reason Room Evaluation.app"
candidate_contents="$candidate_bundle/Contents"
candidate_executable="$candidate_contents/MacOS/infinite-note-canvas"
candidate_info="$candidate_contents/Info.plist"
suggested_workspace="${HOME:-/tmp}/infinite-note-canvas-eframe-evaluation.json"
printf -v copy_workspace_command 'cp %q %q' "$workspace_fixture" "$suggested_workspace"
printf -v set_workspace_command 'export INFINITE_NOTE_CANVAS_WORKSPACE=%q' "$suggested_workspace"

copy_workspace_instructions() {
  printf 'Create a writable copy outside this repository: %s; then run: %s' \
    "$copy_workspace_command" \
    "$set_workspace_command"
}

resolve_workspace_path() {
  local path="$1"
  local link_target

  if [[ "$path" != /* ]]; then
    path="$PWD/$path"
  fi

  while [[ -L "$path" ]]; do
    link_target="$(readlink "$path")"
    if [[ "$link_target" = /* ]]; then
      path="$link_target"
    else
      path="$(dirname "$path")/$link_target"
    fi
  done

  printf '%s/%s\n' "$(cd "$(dirname "$path")" && pwd -P)" "$(basename "$path")"
}

candidate_process_ids() {
  local process_id
  local process_command

  while IFS= read -r process_id; do
    [[ -n "$process_id" ]] || continue
    process_command="$(ps -p "$process_id" -o command= 2>/dev/null || true)"
    if [[ "$process_command" == "$candidate_executable" ]]; then
      printf '%s\n' "$process_id"
    fi
  done < <(pgrep -f 'infinite-note-canvas' 2>/dev/null || true)
}

candidate_is_running() {
  [[ -n "$(candidate_process_ids)" ]]
}

# `open -W` waits for the application but does not make it a child process.
# Keep the development candidate owned by this launcher, so stopping
# ./start.sh also closes the candidate that it launched.
candidate_started_by_launcher=false

stop_launched_candidate() {
  [[ "$candidate_started_by_launcher" == true ]] || return 0

  local process_ids
  process_ids="$(candidate_process_ids)"
  [[ -n "$process_ids" ]] || return 0

  append_startup_diagnostic 'event=candidate_stop_requested' || true
  while IFS= read -r process_id; do
    [[ -n "$process_id" ]] || continue
    kill -TERM "$process_id" 2>/dev/null || true
  done <<< "$process_ids"

  local attempt
  for attempt in {1..30}; do
    [[ -z "$(candidate_process_ids)" ]] && return 0
    sleep 0.1
  done

  append_startup_diagnostic 'event=candidate_stop_forced' || true
  while IFS= read -r process_id; do
    [[ -n "$process_id" ]] || continue
    kill -KILL "$process_id" 2>/dev/null || true
  done <<< "$(candidate_process_ids)"
}

cleanup_launched_candidate() {
  local exit_status=$?
  trap - EXIT
  stop_launched_candidate
  exit "$exit_status"
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  fail \
    'This eframe candidate launcher requires macOS.' \
    'Run scripts/start-macos-eframe.sh on a Mac with the Rust toolchain installed.'
fi

if [[ ! -f Cargo.toml || ! -f crates/app/Cargo.toml ]]; then
  fail \
    'The current directory does not contain the eframe candidate Cargo package.' \
    'Run this script from an intact Reason Room checkout.'
fi

if ! command -v cargo >/dev/null 2>&1; then
  fail \
    'Required command is unavailable: cargo.' \
    'Install Rust 1.97 or later, then run this script again.'
fi

if ! grep -Eq '^[[:space:]]*eframe([[:space:]]*=[[:space:]]*|[.]workspace[[:space:]]*=[[:space:]]*)' crates/app/Cargo.toml; then
  fail \
    'The application package is not configured for the eframe candidate.' \
    'Integrate the eframe application package changes before using this launcher.'
fi

if [[ ! -f "$workspace_fixture" ]]; then
  fail \
    "Evaluation workspace fixture is missing: $workspace_fixture" \
    'Restore the tracked evaluation fixture before launching the candidate.'
fi

if [[ ! -f "$bundle_info_template" ]]; then
  fail \
    "The eframe evaluation bundle template is missing: $bundle_info_template" \
    'Restore the tracked eframe evaluation bundle template before launching the candidate.'
fi

if [[ -n "${INFINITE_NOTE_CANVAS_WORKSPACE:-}" ]]; then
  launch_workspace_environment_variable="$workspace_environment_variable"
  workspace_path="$INFINITE_NOTE_CANVAS_WORKSPACE"
elif [[ -n "${INFINITE_NOTE_CANVAS_FALLBACK_WORKSPACE:-}" ]]; then
  launch_workspace_environment_variable="$fallback_workspace_environment_variable"
  workspace_path="$INFINITE_NOTE_CANVAS_FALLBACK_WORKSPACE"
  unset INFINITE_NOTE_CANVAS_WORKSPACE
else
  fail \
    'No workspace or fallback workspace was supplied.' \
    "$(copy_workspace_instructions)"
fi

if [[ ! -f "$workspace_path" ]]; then
  fail \
    "Workspace file does not exist: $workspace_path" \
    "$(copy_workspace_instructions)"
fi

if [[ ! -w "$workspace_path" ]]; then
  fail \
    "Workspace file is not writable: $workspace_path" \
    "Choose a writable copy outside this repository. $(copy_workspace_instructions)"
fi

workspace_path="$(resolve_workspace_path "$workspace_path")"
repository_workspaces="$repository_root/workspaces"
if [[ "$workspace_path" == "$repository_workspaces" || "$workspace_path" == "$repository_workspaces"/* ]]; then
  fail \
    "Workspace file is inside this repository's workspaces/ directory: $workspace_path" \
    "$(copy_workspace_instructions)"
fi

export "$launch_workspace_environment_variable=$workspace_path"
startup_diagnostic_path="$(dirname "$workspace_path")/infinite-note-canvas-startup.log"

if ! : >> "$startup_diagnostic_path"; then
  fail \
    "Startup diagnostic log cannot be written: $startup_diagnostic_path" \
    'Choose a writable external workspace directory, then run ./start.sh again.'
fi

append_startup_diagnostic() {
  printf '%s\n' "$1" >> "$startup_diagnostic_path"
}

printf 'Starting Reason Room eframe candidate from: %s\n' "$repository_root"
printf 'Workspace path: %s\n' "$workspace_path"
printf 'Workspace copy command: %s\n' "$copy_workspace_command"
printf 'Startup diagnostic log: %s\n' "$startup_diagnostic_path"

append_startup_diagnostic 'event=launcher_start'
append_startup_diagnostic "workspace_path=$workspace_path"
append_startup_diagnostic "repository_root=$repository_root"

trap cleanup_launched_candidate EXIT
trap 'exit 130' INT TERM HUP

if candidate_is_running; then
  printf 'The eframe evaluation app is already open. It continues to use its existing workspace and executable image.\n'
  printf 'Close that window, then run ./start.sh again to open the rebuilt candidate.\n'
  append_startup_diagnostic 'event=candidate_reused'
  append_startup_diagnostic "bundle_path=$candidate_bundle"
  append_startup_diagnostic "requested_workspace_path=$workspace_path"
  exit 0
fi

set +e
cargo build --release --package infinite-note-canvas 2>&1 | tee -a "$startup_diagnostic_path"
pipeline_status=("${PIPESTATUS[@]}")
set -e

launch_status="${pipeline_status[0]}"
tee_status="${pipeline_status[1]}"

if [[ "$tee_status" -ne 0 ]]; then
  fail \
    "Startup diagnostic log could not be written: $startup_diagnostic_path" \
    'Choose a writable external workspace, then run ./start.sh again.'
fi

if [[ "$launch_status" -ne 0 ]]; then
  fail \
    "The eframe app could not build (exit $launch_status). Diagnostic log: $startup_diagnostic_path" \
    'Review the diagnostic log and docs/installation.md, confirm the pinned Rust toolchain works, then run ./start.sh again.'
fi

if ! mkdir -p "$(dirname "$candidate_executable")"; then
  fail \
    "The eframe evaluation bundle cannot be prepared: $candidate_bundle" \
    'Check write access to the repository target directory, then run ./start.sh again.'
fi

if ! cp "$repository_root/target/release/infinite-note-canvas" "$candidate_executable"; then
  fail \
    "The eframe executable could not be staged in: $candidate_bundle" \
    'Rebuild the eframe candidate, then run ./start.sh again.'
fi

if ! cp "$bundle_info_template" "$candidate_info"; then
  fail \
    "The eframe evaluation bundle metadata could not be staged in: $candidate_bundle" \
    'Restore the eframe evaluation bundle template, then run ./start.sh again.'
fi

if ! chmod u+x "$candidate_executable"; then
  fail \
    "The eframe evaluation bundle executable is not runnable: $candidate_executable" \
    'Check write access to the repository target directory, then run ./start.sh again.'
fi

printf 'Starting eframe evaluation app bundle: %s\n' "$candidate_bundle"
append_startup_diagnostic 'event=candidate_bundle_staged'
append_startup_diagnostic "bundle_path=$candidate_bundle"
append_startup_diagnostic 'bundle_identifier=com.local.infinite-note-canvas.eframe-evaluation'
candidate_started_by_launcher=true

set +e
open -W -n --env "$launch_workspace_environment_variable=$workspace_path" "$candidate_bundle" 2>&1 | tee -a "$startup_diagnostic_path"
pipeline_status=("${PIPESTATUS[@]}")
set -e

launch_status="${pipeline_status[0]}"
tee_status="${pipeline_status[1]}"
append_startup_diagnostic "event=application_exit"
append_startup_diagnostic "exit_status=$launch_status"

if [[ "$tee_status" -ne 0 ]]; then
  fail \
    "Startup diagnostic log could not be written: $startup_diagnostic_path" \
    'Choose a writable external workspace, then run ./start.sh again.'
fi

if [[ "$launch_status" -ne 0 ]]; then
  fail \
    "The eframe evaluation app stopped during startup (exit $launch_status). Diagnostic log: $startup_diagnostic_path" \
    'Review the diagnostic log and docs/installation.md, confirm the pinned Rust toolchain works, then run ./start.sh again.'
fi

printf 'The eframe app exited normally. Startup diagnostic log: %s\n' "$startup_diagnostic_path"
