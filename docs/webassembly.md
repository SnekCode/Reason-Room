# WebAssembly Browser Build

The browser build runs the same Rust canvas and `eframe` UI as the desktop
application. It can be served as static files without an application server,
database, account system, or network API.

## Local Run

Follow the [installation guide](installation.md#4-optional-build-and-run-in-a-browser)
for Rust **1.97.1**, the `wasm32-unknown-unknown` target,
`wasm-bindgen-cli` **0.2.129**, and Python 3. A compatible browser and working
graphics backend are also required.

```sh
./start.sh --web
```

The supported launcher builds the application library, generates matching
JavaScript bindings, copies the page and license, and serves the output on
`127.0.0.1`. Open the printed loopback URL, normally `http://127.0.0.1:8080`.
Use `INFINITE_NOTE_CANVAS_WEB_PORT=8081 ./start.sh --web` for another port.
Stop the local server with Ctrl-C.

The web-only **Download usage skill** button downloads `reason-room-skill.zip`.
Extract the complete `reason-room/` folder into your agent's skill directory.
The bundle includes installation, MCP tool usage, and the
[webview MCP workflow](../skills/reason-room/references/setup.md#use-mcp-with-the-webview).
MCP still requires a separately configured local server and a selected JSON file.

## Browser File Access

| File menu command | Behavior |
| --- | --- |
| **New canvas...** | Creates a new browser-local canvas. Use Save to download it. |
| **Open canvas...** | Imports and validates a user-selected JSON file. |
| **Open and refresh from file...** | Connects a selected file for polling when file handles are supported; otherwise falls back to ordinary Open. |
| **Save** | Downloads the current workspace as JSON. |
| **Save As...** | Downloads a separately named JSON copy. |

The browser never receives arbitrary desktop paths. The user must choose a
file before the application can read it. Downloads do not replace the imported
file; the browser controls their destination and collision behavior.

The optional refresh connection follows changes made by MCP or another author
to that selected file. It defers replacement while authoring is active. It is
not bidirectional synchronization: browser edits are not written back through
the connection. Avoid competing edits, and download important browser work.
See [agent support](product/external-agent-support.md).

## Storage and Reload

The app serializes the active workspace to browser `localStorage` and attempts
to restore it on reload. If no saved canvas exists, it starts with bundled
evaluation data. History and preference state are separate from the
[workspace JSON format](workspace-schema.md).

Browser storage is scoped to the site origin and browser profile. Localhost
and a hosted site do not share storage. GitHub project sites under the same
account share an origin; matching storage keys can therefore collide. The
current key retains the legacy product identifier during the working-name
rebrand. A future key change needs an explicit migration plan.

Storage can be unavailable, full, or cleared. This implementation still has
storage calls without complete failure handling, so persistence must not be
treated as a guaranteed backup. Use downloaded JSON and separate copies for
important work. See [validation status](validation.md).

## Generated Files

Build without starting a server:

```sh
./start.sh --web --build-only
```

Python 3 packages the downloadable skill in this mode too. The generated directory is
`target/web-wasm/`:

| File | Purpose |
| --- | --- |
| `index.html` | Static canvas page and browser bridge. |
| `infinite_note_canvas.js` | Generated JavaScript bindings. |
| `infinite_note_canvas_bg.wasm` | Compiled Rust application. |
| `LICENSE` | Project Apache 2.0 license. |
| `reason-room-skill.zip` | Complete installable usage skill, including webview MCP guidance. |

The legacy asset names match the page's imports. Keep the generated files
together. Imports are relative, allowing both a site root and a project
subdirectory. Publish only this output directory, not the repository or user
workspace files. Serve `.wasm` as `application/wasm`.

## GitHub Pages

[The WebAssembly workflow](../.github/workflows/pages.yml) builds on pushes to
`master`, pull requests, and manual dispatch. Pull requests build without
deploying; successful builds on `master` can deploy through the `github-pages`
environment. Repository Pages settings must use **GitHub Actions** as source.

The repository is configured for
[Reason Room on GitHub Pages](https://snekcode.github.io/Reason-Room/).
See the [workflow history](https://github.com/SnekCode/Reason-Room/actions/workflows/pages.yml)
for the current deployment result. A successful deployment establishes that
the workflow published the files; browser interactions at the final URL need
their own smoke check. Local browser evidence and remaining checks are recorded
in [validation status](validation.md).
