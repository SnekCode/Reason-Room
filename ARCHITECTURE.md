# Architecture

## Overview

Reason Room is a local-first Rust application. It has a native macOS
desktop path and a WebAssembly browser path. Both paths use the same canvas
and workspace-session logic.

The application has no network service, cloud storage, telemetry, or account
system. The browser build uses a loopback server only for local development.

```text
Native desktop                     Local browser build
--------------                     -------------------
./start.sh                         ./start.sh --web
    |                                   |
crates/app                       web/index.html + crates/app
    |                                   |
    +---------------+-------------------+
                    |
      crates/workspace-session
                    |
             crates/canvas
                    |
          local JSON workspace data
```

## Module Boundaries

### `crates/canvas`

This crate owns the pure domain and canvas behavior. It contains workspace
data types, validation, geometry, camera transformations, visibility queries,
relationship routing, containment, and generated data. It does not depend on
the UI framework or on file persistence.

### `crates/workspace-session`

This crate owns validated workspace sessions and typed mutations. It stages
actions, keeps undo and redo history, and selects the active version and lens.
On native builds it also owns local-file creation, save verification, backup,
and restoration. It stays free of `eframe` and painter types.

### `crates/app`

This crate contains the `eframe` application, desktop entry point, controls,
and all rendering code. It translates user interaction into
`workspace-session` actions and draws the visible canvas. It is the only crate
that depends on the UI framework.

The same crate builds the native binary and the WebAssembly library. Its web
entry point mounts the shared application in the HTML canvas.

### `crates/cli`

This crate provides local command-line inspection, validation, query, and MCP
adapter support. It uses the canvas and session crates. It does not provide a
network service.

### Launch And Browser Files

- `start.sh` is the supported application entry point.
- `scripts/start-macos-eframe.sh` builds and stages the macOS evaluation app.
- `scripts/start-web-wasm.sh` builds WebAssembly and serves generated files on
  `127.0.0.1`.
- `web/index.html` starts the WebAssembly application, imports user-selected
  files, downloads saved files, and stores the current browser workspace.

## Workspace Model

A workspace document contains versions. Each version contains a graph of
concepts and typed relationships plus one or more lenses. A lens contains
placements, visual boundaries, semantic zoom layers, and the presentation
settings that control what the canvas shows at each scale.

The model distinguishes graph facts from visual context. Relationships express
typed connections. Placement, containment, visual boundaries, and zoom layers
describe how the graph appears in a lens.

## Persistence

### Native Desktop

The desktop path stores a workspace as local JSON. A new document uses an
exclusive create operation and does not replace an existing path.

Before an existing document is saved, `workspace-session` reads the current
file and compares its digest with the source opened by the app. It rejects the
save if another process changed the file. For an accepted save, it writes a
sibling `.bak` backup, writes a temporary candidate, replaces the active file,
and reads and validates the exact persisted source. If the final verification
fails, it tries to restore the previous source and leaves the backup in place.

### Browser

The WebAssembly build has no ambient desktop-path access. A person selects a
JSON file through the browser file picker before the application reads it.
The current workspace is serialized to `localStorage` for the same browser
origin and restored after a page reload. Save and Save As create browser
downloads; they do not write back to the selected source file.

Browser storage follows the browser profile's retention and access rules. It
is not a substitute for a downloaded backup of important work.

## Design Rules

- Keep canvas mathematics, visibility, and generated data pure and testable.
- Keep `eframe` rendering and painter code inside `crates/app`.
- Keep `crates/workspace-session` independent from the UI framework.
- Do not create a UI element for every workspace record.
- Pin dependencies to exact versions or immutable Git revisions.
- Do not add network services, telemetry, cloud storage, or external-format
  compatibility without a product decision.

## Further Reading

- [README](README.md) provides install, run, and development guidance.
- [Security policy](SECURITY.md) documents the security boundary and reporting
  process.
- [WebAssembly browser build](docs/webassembly.md) documents generated files
  and static hosting.
- [Product vision](docs/product/vision.md) explains the product-discovery
  scope.
