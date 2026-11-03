# Install and Run Reason Room

Reason Room builds from source. The supported desktop launcher currently runs
on macOS. The declared desktop minimum is macOS 13; the recorded launch check
was on Apple Silicon with macOS 27.0.1, not a complete compatibility matrix. The browser build uses the same Rust application through WebAssembly.

## Required Tools

| Tool | Desktop | Local browser build | Notes |
| --- | --- | --- | --- |
| macOS 13 or later | Required | macOS is supported for local development | The desktop app bundle declares macOS 13 as its minimum. |
| Xcode Command Line Tools or full Xcode | Required on macOS | Required on macOS | Supplies Apple's compiler, linker, and SDK. |
| Git | Required to clone | Required to clone | Available with Apple's developer tools. |
| Rust through `rustup` | Required | Required | `rust-toolchain.toml` pins Rust **1.97.1**, including Cargo, rustfmt, and Clippy. |
| `wasm32-unknown-unknown` Rust target | No | Required | Install for the pinned toolchain. |
| `wasm-bindgen-cli` **0.2.129** | No | Required | Must match the version used by the application. |
| Python 3 | No | Required to package the skill and serve locally | Also used by documentation checks; not needed for visiting a hosted site. |
| Modern browser with WebGPU or WebGL2 support | No | Required to use the browser app | Browser and graphics support must be checked on the target device. |

Cargo downloads the pinned Rust library dependencies automatically. You do not
need to install Node.js, npm, Trunk, or wasm-pack for this build path. The first
build requires an internet connection and can take several minutes.

## 1. Install Apple's Developer Tools

Check for an existing installation:

```sh
xcode-select -p
clang --version
```

If the developer tools are missing, run:

```sh
xcode-select --install
```

Complete the macOS installer before continuing. Full Xcode also provides the
required tools; you do not need both installations.

## 2. Install Rust with rustup

Use the [official Rust installer](https://rust-lang.org/tools/install/):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.97.1 --component rustfmt,clippy
export PATH="$HOME/.cargo/bin:$PATH"
```

The installer configures supported shell startup files. If Homebrew's shell
setup later changes your PATH, make sure `$HOME/.cargo/bin` comes before
`/opt/homebrew/bin` for Rust commands. The supported `./start.sh` launcher
also prefers a rustup installation in `${CARGO_HOME:-$HOME/.cargo}/bin`.

An existing Homebrew Rust installation can remain installed. Reason Room needs
the rustup-managed toolchain so that the repository's exact version is used.
See the [rustup installation documentation](https://rust-lang.github.io/rustup/installation/other.html)
for installer options and alternate setups.

## 3. Clone and Start the Desktop App

```sh
git clone https://github.com/SnekCode/Reason-Room.git
cd Reason-Room
rustup show
rustc --version
cargo --version
./start.sh
```

If you already have a checkout, change to that directory instead of cloning
again. `rustup show` installs any missing components from
`rust-toolchain.toml`. Inside this repository, `rustc --version` should report
**1.97.1**.

The launcher builds the desktop application, prepares a macOS app bundle, and
opens it. Keep the terminal running while using the app; stopping the launcher
also closes the app instance it started.

On first launch, an evaluation workspace is copied to:

```text
~/Library/Application Support/Infinite Note Canvas/reason-room-static-react-website.infinite-note.json
```

That directory retains the original working name for compatibility. Later
runs can reopen the last document. Use the File menu to create or open your
own canvas. To explicitly select a different existing JSON document:

```sh
INFINITE_NOTE_CANVAS_WORKSPACE="/absolute/path/outside/the/repository/canvas.json" ./start.sh
```

Use a writable file outside the checkout. Important documents should have
separate backups. Continue with the [user guide](using-reason-room.md) to create
and arrange your own map.

## 4. Optional: Build and Run in a Browser

From the repository root, with rustup's tools on PATH:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
rustup target add wasm32-unknown-unknown --toolchain 1.97.1
cargo install wasm-bindgen-cli --version 0.2.129 --locked
python3 --version
./start.sh --web
```

If Python 3 is missing and you use Homebrew, install it with `brew install
python`. Then open the loopback URL printed by the launcher, normally
`http://127.0.0.1:8080`. Stop the server with Ctrl-C.

To generate static site files without running a server:

```sh
./start.sh --web --build-only
```

The output is `target/web-wasm/`. See [WebAssembly browser build](webassembly.md)
for file access, storage, and GitHub Pages deployment details.

## Troubleshooting

### macOS reports an LLVM symbol error before compilation

A failure like this means the compiler itself could not start:

```text
error: process didn't exit successfully: rustc -vV
Symbol not found: ... llvm ...
Referenced from: /opt/homebrew/Cellar/rust/1.89.0/...
Expected in: /opt/homebrew/Cellar/llvm/21.1.7/...
```

The reported Homebrew Rust 1.89 installation is both older than the project's
required toolchain and linked against an incompatible LLVM installation.
Changing the workspace file will not fix this compiler error.

Install rustup using step 2, then check from the repository root:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
command -v rustc
rustup show active-toolchain
rustc --version
./start.sh
```

`command -v rustc` should resolve to the rustup proxy under `.cargo/bin`, and
`rustc --version` should report 1.97.1. If the terminal still resolves the old
compiler, open a new terminal or repeat the PATH command above. There is no
need to delete your canvas, its backups, or the application-support directory.

### wasm-bindgen is missing or has the wrong version

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo install wasm-bindgen-cli --version 0.2.129 --locked --force
wasm-bindgen --version
```

The result must be `wasm-bindgen 0.2.129`.

### The browser's local port is already in use

```sh
INFINITE_NOTE_CANVAS_WEB_PORT=8081 ./start.sh --web
```

### The workspace cannot be opened or saved

Check the exact path printed by the launcher. Confirm the file exists, is
writable, and is outside this checkout. Read the startup diagnostic log at the
printed location. Preserve important workspace files before changing them.

## Development Checks

From the repository root, with rustup's tools on PATH:

```sh
cargo fmt --check
cargo test --locked --workspace
cargo build --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
bash scripts/check-active-doc-guidance.sh
```
