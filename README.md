# Reason Room

Reason Room is an experimental, local-first application for mapping
complex systems. It lets one person arrange concepts, relationships, visual
boundaries, and semantic zoom layers in a navigable canvas. Workspace data is
stored as local JSON.

Reason Room is the working product name.

The project is in active discovery. Its file format and user experience can
change before a stable release. Keep a separate copy of important workspaces.

## Features

- Create, open, save, and copy local canvas documents.
- Arrange cards, relationships, and visual boundaries on an infinite canvas.
- Use versions, lenses, containment, and semantic zoom to view a system at
  different levels of detail.
- Undo and redo workspace edits during the current session.
- Inspect local workspaces with the optional command-line interface.
- Connect an agent through local MCP tools and the installable Reason Room skill.
- Run the same canvas in a local WebAssembly browser build.

## Installation

Follow the [installation guide](docs/installation.md) for the complete macOS
setup, optional browser tools, and troubleshooting. For a Mac with Apple
developer tools already installed:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.97.1 --component rustfmt,clippy
export PATH="$HOME/.cargo/bin:$PATH"
git clone https://github.com/SnekCode/Reason-Room.git
cd Reason-Room
./start.sh
```

## Requirements

### Desktop

The supported desktop path currently requires macOS, Rust `1.97.1`, and the
Apple developer tools (Xcode Command Line Tools or full Xcode). The repository pins the Rust toolchain
in `rust-toolchain.toml`.

### Browser Build

The local browser build also requires:

- The Rust `wasm32-unknown-unknown` target.
- `wasm-bindgen-cli` `0.2.129`.
- Python 3 for the local static-file server.

## Start The Application

Use one command:

```sh
./start.sh
```

This is the only supported application entry point. On first use it creates a
writable evaluation workspace at
`~/Library/Application Support/Infinite Note Canvas/`. Later runs reopen the
last document you opened. To use another local file, set
`INFINITE_NOTE_CANVAS_WORKSPACE` to a writable path outside this repository.
An explicit path takes priority over the saved last document.

Do not start the application with `cargo run` or an internal `scripts/start-*`
helper. Those commands are not supported user entry points.

Use the **File** menu to manage local JSON documents:

- **New Canvas...** creates an empty, separate document.
- **Open...** loads a selected document.
- **Save** verifies and saves the active document.
- **Save As...** creates a separate copy. It does not replace an existing file.

## Run In A Browser

Start the local WebAssembly build:

```sh
./start.sh --web
```

The command prints a loopback address such as `http://127.0.0.1:8080`. Open
that address in a modern browser. The browser build stores the last canvas in
that browser's local storage for the site origin. **Open** imports a
user-selected JSON file. **Save** and **Save As** download JSON files because a
browser cannot write back to an arbitrary local path.

See [WebAssembly browser build](docs/webassembly.md) for the build output,
static-hosting requirements, and browser data boundary. To generate the static
site without starting a server, use `./start.sh --web --build-only`.

## Data And Recovery

Desktop files are local JSON documents. Before a desktop save replaces a file,
the application checks for changes made outside the app, writes a sibling
`.bak` backup, persists a candidate source, and verifies the saved result. It
restores the previous source when post-replacement verification fails.

These checks reduce the chance of accidental loss. They do not replace normal
backups or version control. The application does not encrypt workspace files.
Do not store data in a workspace unless local-file and browser-storage access
on the device are acceptable for that data.

## Agent Skill and MCP

The [Reason Room skill](skills/reason-room/SKILL.md) gives a skill-capable agent
a workflow for reading and editing canvases through the local MCP server.
It covers six tools and 18 typed changes, including cards, relationships,
visual boundaries, and semantic zoom layers.

Install the complete `skills/reason-room/` folder into your agent's skill
directory. Follow the [skill setup guide](skills/reason-room/references/setup.md)
to build the CLI, connect one explicitly selected workspace, and choose
read-only or write-enabled access. Then ask your agent to use `$reason-room`.
The skill and MCP connection are separate installations.

## Compatibility During Rebranding

Existing JSON documents continue to use the same schema and
`.infinite-note.json` suffix. The existing application-support directory,
browser storage keys, environment variables, and Rust package names retain
their original identifiers during this working-name phase. This keeps local
data and development integrations compatible. The app and browser page display
**Reason Room**.

## Development

Run the workspace checks from the repository root:

```sh
cargo fmt --check
cargo test --workspace
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/check-active-doc-guidance.sh
```

The CLI provides deterministic local inspection and validation. See
[Workspace schema](docs/workspace-schema.md) for historical schema context and
[External agent support](docs/product/external-agent-support.md) for the local
MCP boundary.

## Documentation

- [Installation](docs/installation.md) covers dependencies and troubleshooting.
- [Architecture](ARCHITECTURE.md) explains the module and persistence
  boundaries.
- [Security policy](SECURITY.md) explains how to report vulnerabilities.
- [Product vision](docs/product/vision.md) describes the current discovery
  scope.
- [Glossary](GLOSSARY.md) defines the domain terms used by the application.
- [WebAssembly browser build](docs/webassembly.md) documents the local web
  path.

Historical research, work items, and validation notes remain under `docs/`.
They record past decisions. They are not a release guarantee.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
