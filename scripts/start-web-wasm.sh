#!/usr/bin/env bash

set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$repository_root"

build_only=false
if [[ "${1:-}" == "--build-only" ]]; then
  build_only=true
  shift
fi
if [[ "$#" -ne 0 ]]; then
  printf 'ERROR: Unknown web option: %s. Use ./start.sh --web [--build-only].\n' "$1" >&2
  exit 1
fi

web_port="${INFINITE_NOTE_CANVAS_WEB_PORT:-8080}"
web_host="127.0.0.1"
wasm_target="wasm32-unknown-unknown"
wasm_bindgen_version="0.2.129"
wasm_artifact="$repository_root/target/$wasm_target/release/infinite_note_canvas.wasm"
web_output="$repository_root/target/web-wasm"

fail() {
  printf 'ERROR: %s\n' "$1" >&2
  printf 'ACTION: %s\n' "$2" >&2
  exit 1
}

if [[ ! "$web_port" =~ ^[0-9]+$ ]] || (( web_port < 1 || web_port > 65535 )); then
  fail \
    "INFINITE_NOTE_CANVAS_WEB_PORT must be an integer from 1 through 65535." \
    'Set INFINITE_NOTE_CANVAS_WEB_PORT to an available local port, then run ./start.sh --web again.'
fi

if ! command -v cargo >/dev/null 2>&1; then
  fail \
    'Required command is unavailable: cargo.' \
    'Install Rust 1.97 or later, then run ./start.sh --web again.'
fi

if ! command -v rustup >/dev/null 2>&1; then
  fail \
    'Required command is unavailable: rustup.' \
    'Install the pinned Rust toolchain using rustup, then run ./start.sh --web again.'
fi

if ! rustup target list --installed | grep -Fxq "$wasm_target"; then
  fail \
    "Required Rust target is unavailable: $wasm_target." \
    "Run rustup target add $wasm_target, then run ./start.sh --web again."
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  fail \
    'Required command is unavailable: wasm-bindgen.' \
    "Install wasm-bindgen-cli $wasm_bindgen_version, then run ./start.sh --web again."
fi

if [[ "$(wasm-bindgen --version)" != "wasm-bindgen $wasm_bindgen_version" ]]; then
  fail \
    "wasm-bindgen must be version $wasm_bindgen_version." \
    "Install wasm-bindgen-cli $wasm_bindgen_version, then run ./start.sh --web again."
fi

if [[ "$build_only" == false ]] && ! command -v python3 >/dev/null 2>&1; then
  fail \
    'Required command is unavailable: python3.' \
    'Install Python 3, then run ./start.sh --web again.'
fi

printf 'Building Reason Room for %s.\n' "$wasm_target"
cargo build --locked --release --package infinite-note-canvas --target "$wasm_target" --lib

if [[ ! -f "$wasm_artifact" ]]; then
  fail \
    "WebAssembly artifact is missing: $wasm_artifact" \
    'Review the build output, then run ./start.sh --web again.'
fi

mkdir -p "$web_output"
wasm-bindgen \
  --target web \
  --no-typescript \
  --out-dir "$web_output" \
  --out-name infinite_note_canvas \
  "$wasm_artifact"
cp "$repository_root/web/index.html" "$web_output/index.html"
cp "$repository_root/LICENSE" "$web_output/LICENSE"

if [[ "$build_only" == true ]]; then
  printf 'Static site built at: %s\n' "$web_output"
  exit 0
fi

printf 'Reason Room is available at http://%s:%s\n' "$web_host" "$web_port"
printf 'Browser sessions use local JSON Open and browser-download Save actions.\n'
printf 'Press Ctrl-C to stop the local server.\n'
exec python3 -m http.server "$web_port" --bind "$web_host" --directory "$web_output"
