#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'ERROR: %s\n' "$1" >&2
  printf 'ACTION: %s\n' "$2" >&2
  exit 1
}

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"

current_directory="$(pwd -P)"
if [[ "$current_directory" != "$repository_root" ]]; then
  fail \
    "Run this guard from the repository root: $repository_root." \
    'Change to the repository root, then run bash scripts/check-active-doc-guidance.sh.'
fi

guidance_files=(
  'README.md'
  'AGENTS.md'
  'docs/roadmap.md'
)

while IFS= read -r guidance_file; do
  guidance_files+=("$guidance_file")
done < <(find docs/product docs/discovery -type f -name '*.md' -print | LC_ALL=C sort)

for guidance_file in "${guidance_files[@]}"; do
  if [[ ! -f "$guidance_file" ]]; then
    fail \
      "The active-guidance file is missing: $guidance_file." \
      'Restore the file or update this guard with the new active-guidance location.'
  fi
done

assert_absent() {
  local description="$1"
  local pattern="$2"
  local matches
  local grep_status

  if matches="$(grep -nH -E "$pattern" "${guidance_files[@]}")"; then
    printf 'Prohibited active-guidance references (%s):\n%s\n' \
      "$description" "$matches" >&2
    fail \
      "Active guidance contains $description." \
      'Remove the reference from active guidance. Keep dated historical evidence outside the checked paths.'
  else
    grep_status=$?
    if [[ "$grep_status" -ne 1 ]]; then
      fail \
        "The active-guidance scan failed for $description." \
        'Fix the scan environment, then run bash scripts/check-active-doc-guidance.sh again.'
    fi
  fi
}

require_text() {
  local file="$1"
  local required_text="$2"
  local description="$3"

  if ! grep -qF -- "$required_text" "$file"; then
    fail \
      "$file does not retain $description." \
      'Restore the supported-entry-point or retired-workflow guidance, then run this guard again.'
  fi
}

assert_absent \
  'a retired work-item script' \
  'scripts/(new-work-item|run-work-item-agent)[.]sh'
assert_absent \
  'a live numbered work-item path' \
  'docs/work-items/[0-9][0-9]*([/.]|$)'

require_text \
  'README.md' \
  './start.sh' \
  'the ./start.sh application entry point'
require_text \
  'README.md' \
  'This is the only supported application entry point.' \
  'the only-supported application entry-point statement'
require_text \
  'README.md' \
  'Do not start the application with `cargo run` or an internal `scripts/start-*`' \
  'the cargo run and internal-launch-helper prohibition'
require_text \
  'README.md' \
  'helper. Those commands are not supported user entry points.' \
  'the cargo run and internal-launch-helper prohibition'
require_text \
  'AGENTS.md' \
  './start.sh` is the only supported application launch entry point.' \
  'the ./start.sh-only launch rule'
require_text \
  'AGENTS.md' \
  'Do not instruct a user to start the application with `cargo run` or an' \
  'the cargo run prohibition'
require_text \
  'AGENTS.md' \
  'internal `scripts/start-*` helper.' \
  'the internal-launch-helper prohibition'
printf 'PASS: Active project guidance retains the supported entry point and does not revive the retired work-item workflow.\n'
