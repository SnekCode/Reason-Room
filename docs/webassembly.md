# WebAssembly Browser Build

## Purpose

The browser build runs the canvas engine and the `eframe` UI in WebAssembly.
It does not add a second JavaScript engine or a second user interface.

The browser build starts with a bundled workspace document for the current
page session. Open loads a user-selected local JSON file. Save and Save As
download local JSON files. The last canvas is retained in browser local storage and restored on reload.
Download important work as JSON; browser storage can be cleared or unavailable.

## Boundaries

| Module | Browser responsibility |
| --- | --- |
| `crates/canvas` | Pure canvas data, camera behavior, visibility, and generated data. |
| `crates/workspace-session` | Typed actions, validation, undo and redo, and page-session workspace state. |
| `crates/app` | The `WebHandle`, `eframe::WebRunner`, and the shared canvas UI. |
| `web/index.html` | The static HTML canvas and the small Wasm boot sequence. |

The browser session uses an in-memory `WorkspaceSession`. Desktop sessions
keep the local JSON save, backup, verification, and recovery boundary.

## Browser File Menu

The browser File menu uses local-file actions. It does not request ambient
access to desktop paths.

| Command | Browser behavior |
| --- | --- |
| New canvas | Creates and opens an empty named page-session canvas. |
| Open canvas | Requests a user-selected local JSON file and validates it. |
| Open for live refresh | Watches a selected JSON file where the browser supports file handles; otherwise uses ordinary Open. |
| Save | Downloads local JSON with the workspace name. |
| Save as | Downloads a local JSON copy with the entered name. |

The browser only receives a local JSON source after the user chooses it in the
browser file picker. It does not receive an ambient desktop path or write back
to the selected file. Browsers control the download destination and do not
expose a general local-file write API.

## Local Run

Install the pinned Rust toolchain and browser dependencies using the
[installation guide](installation.md#4-optional-build-and-run-in-a-browser).

Use the supported application entry point:

```sh
./start.sh --web
```

The launcher does these actions:

1. Builds the `crates/app` library for `wasm32-unknown-unknown` in release
   mode.
2. Runs the matching `wasm-bindgen` tool.
3. Copies the static HTML page with the generated JavaScript and Wasm files.
4. Serves only those files on `127.0.0.1`.

The default address is `http://127.0.0.1:8080`. Set
`INFINITE_NOTE_CANVAS_WEB_PORT` before the command to use another local port.

The launcher requires these tools:

- Rust 1.97 or later with the `wasm32-unknown-unknown` target.
- `wasm-bindgen-cli` 0.2.129.
- Python 3 for the local static server.

## Generated Site

The launcher creates these files under `target/web-wasm/`:

| File | Purpose |
| --- | --- |
| `index.html` | Static page that creates the canvas and starts the app. |
| `infinite_note_canvas.js` | Generated JavaScript bindings. |
| `infinite_note_canvas_bg.wasm` | The compiled Rust engine and UI. |

Keep these files in the same published directory. The HTML uses relative
paths. It can run from a site root or from a project subdirectory.

## Static Hosting

The generated site can run on static hosting. It needs no application server,
database, or network API.

Build without starting a local server:

```sh
./start.sh --web --build-only
```

This produces `target/web-wasm/`, including the project license. Python is not
required for this build-only path. Publish only this directory, never the
repository root or a workspace directory. The host must serve `.wasm` files
with the `application/wasm` MIME type.

The prepared `.github/workflows/pages.yml` builds this directory on pushes to
`master` and on manual dispatch, then deploys it to GitHub Pages. In the GitHub
repository settings, choose **Pages → Source → GitHub Actions**. The deployment
job uses the `github-pages` environment and the workflow-provided site URL.
Pull requests build the same site without deploying it.

The workflow is prepared but has not yet been run on GitHub. A successful
build and a browser smoke check at the published project URL are required
before treating the site as released.

GitHub project sites under the same account share an origin. Browser storage
keys are therefore shared across those sites when their keys match. Keep the
final project's storage key unique, and plan any key migration during rebranding.

## Acceptance Evidence

The browser path is accepted when all of these checks pass:

1. `cargo build --target wasm32-unknown-unknown --release --lib` succeeds.
2. `wasm-bindgen` produces the JavaScript and Wasm files.
3. `./start.sh --web` starts the local static site.
4. A browser shows the canvas, controls, and inspector.
5. The File menu provides New, Open, Save, and Save As local-file actions.
