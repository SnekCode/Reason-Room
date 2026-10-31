# Architecture (Pre-Reset Historical Record)

> **Historical pre-reset evidence.** This page preserves dated architecture and
> prototype-boundary notes. It does not prescribe the active product model,
> shell, launch procedure, or work-item workflow. The
> [2026-10-30 product reset](../wiki/product-reset-2026-10-30.md) and
> [product roadmap](../roadmap.md) supersede its planning assumptions. Use
> `AGENTS.md` for active technical constraints and `./start.sh` only for manual
> application evaluation.

## 2026-10-27: Prototype Boundaries

The Rust workspace separates the following responsibilities:

- `crates/canvas` will contain pure camera transforms, deterministic generated data, and viewport queries.
- `crates/app` will contain the GPUI application shell and rendering adapter.
- A future graph-domain module will own concepts, typed relationships, schema validation, and transactions.
- A future persistence module will own versioned local workspace files and rebuildable indexes.
- A future agent API module will expose validated reads and reviewable proposed changes.

## 2026-10-27: Verified Pure Canvas Capabilities

The bootstrap implementation in `crates/canvas` provides pure, dependency-free camera and synthetic-data operations. It includes pan; world-to-screen and screen-to-world transforms; pointer-anchored zoom with a `25%` to `400%` clamp; deterministic generation of 10,000 cards and 50,000 relationships; generated summaries; bounded overview clusters; and `Overview`, `Card`, and `Detail` semantic levels.

The viewport operations use a uniform-grid spatial index. The module exposes viewport, point-hit, and normalized selection-rectangle queries. These operations are separate from GPUI rendering and native input handling.

## 2026-10-27: Rendering Rule

The application must render only visible records. The pure canvas module uses a uniform-grid spatial index for viewport queries and creates bounded overview clusters from visible cards. The GPUI adapter uses overview clusters at overview zoom or when visible cards exceed `750`, and displays an explicit fallback state for the count-triggered cluster path. GPUI rendering behavior remains unvalidated until a native application build and interactive run pass.

## 2026-10-27: Implemented GPUI Adapter Responsibilities

The GPUI application adapter implements viewport-local interaction handling after its sidebar. It maps middle-button drag events through a capture listener to pure camera pan and wheel events to pointer-anchored pure-camera zoom. It maps click, Shift-click, and GPUI drag-move gestures to pure point-hit and normalized selection-rectangle queries.

The adapter renders viewport-visible cards only, adds selected-card highlights, and selects an overview cluster, card, or detail-card presentation from the pure semantic zoom level. Detail cards render generated summaries. The application source stores 50,000 relationships and renders no relationship lines. Native compilation and runtime behavior remain unvalidated because the host lacked full Xcode Metal tools.

## 2026-10-27: Canvas Rule

Camera and generated-data logic must not depend on GPUI types. This makes the logic unit-testable and lets the desktop shell change if the prototype gate rejects GPUI.

## 2026-10-27: GPUI Static Type-Check Boundary

An isolated temporary workspace copy enabled only GPUI's `runtime_shaders` feature and passed offline workspace `cargo check`. The check type-checked the application adapter after its interaction type errors were fixed.

The production manifests retain GPUI's default renderer. The temporary static-check configuration must not be treated as a renderer replacement or as native runtime evidence. Default-renderer build, shader, rendering, input, performance, macOS Retina scaling, Windows scaling, native trackpad pinch, and platform behavior require separate macOS and Windows validation.

## 2026-10-28: Default Renderer Validation Boundary

Xcode `26.6` and `xcrun` paths for `metal` and `metallib` were found on the macOS host. The default GPUI renderer then stopped during shader compilation because its Metal Toolchain component was missing. The required component-download command did not run because the automated approval service returned `404` before execution.

The default-renderer boundary remains unchanged: static checks and pure-canvas tests do not prove default-renderer compilation, rendering, or input. The adapter remains isolated so a failed macOS or Windows acceptance gate can still lead to a different desktop shell.

## 2026-10-28: User-Terminal Default-Renderer Execution Boundary

The user's normal Terminal reported the Metal Toolchain installed and resolved `metal` and `metallib` through `xcrun --no-cache`. The agent sandbox cannot access that toolchain state. Its default-renderer GPUI builds still stop during shader compilation before application compilation.

Run default-renderer build and runtime validation in the user's normal Terminal. Keep the adapter boundary intact until the separate macOS and Windows acceptance records prove the production desktop foundation.

## 2026-10-28: Sandboxed Xcode Component-Mapping Boundary

The agent and user processes reported the same macOS account, home directory, and selected Xcode developer directory. The agent sandbox could execute the mounted Metal Toolchain binaries directly, but it could not persist `~/Library/Developer/Xcode/XcodeToMetalToolchainIndexMapping.plist`. This prevented sandboxed `xcrun` from resolving the mounted toolchain while the user's normal Terminal resolved it correctly.

The environment difference is outside the rendering adapter. It does not alter the adapter seam or select GPUI. Use `scripts/validate-macos-gpui.sh` in the user's normal Terminal for build validation. Keep interactive macOS and Windows acceptance evidence separate from its build checks.

## 2026-10-28: Interaction Adapter Dispatch Correction

The pure `CanvasInteraction` controller owns gesture decisions. The GPUI adapter translates each relevant framework event into one viewport command and requests one redraw. During review, a captured mouse-move event was found to dispatch both pan and selection continuation commands during a left-button selection drag. The adapter was corrected so captured movement dispatches only middle-button pan; left-button drag movement dispatches only selection continuation.

The controller keeps eleven deterministic tests. The workspace passed `22` tests and its release build after the correction. A macOS release window was restarted after the refactor without shared-pointer automation. That restart does not validate native event delivery, native trackpad pinch, rendering performance, display scaling, or Windows behavior. Keep the controller and adapter boundary intact until the pending macOS and Windows acceptance gates provide those results.

## 2026-10-28: Pure-Model Scale-Benchmark Boundary

`crates/canvas/examples/scale_benchmark.rs` measures only the pure canvas boundary: deterministic data generation, spatial viewport queries, overview-cluster queries, and `CanvasInteraction` command traces for zoom, pan, click selection, and marquee selection. The release run used `10,000` concepts and `50,000` relationships. It reported `7.370 ms` data generation; viewport-query total `0.634 ms` with `p99 0.009 ms`; cluster-query total `5.359 ms` with `p99 0.077 ms`; zoom `0.023 ms`; pan `0.024 ms`; clicks `0.443 ms`; and marquee `1.149 ms`.

The benchmark has no GPUI or GPU dependency. It proves neither application frame rate nor native pointer-event delivery. GPUI rendering, display scaling, native input, and macOS or Windows FPS acceptance require separate platform evidence.

## 2026-10-28: Windows Validator Boundary

`scripts/validate-windows-gpui.ps1` belongs to the platform-validation boundary, not the GPUI rendering or pure-canvas interaction boundaries. It checks the Windows toolchain and runs offline Rust validation, release build, and bounded executable-liveness steps. It writes its report under `$env:TEMP`.

The script prints, but does not automate, native input, display-scaling, and performance tests. Those checks require separate Windows-device evidence in `gpui-prototype.md`. Native PowerShell parsing remains unverified on the macOS host because PowerShell is not installed.

## 2026-10-28: Live Visual Rendering Boundary

The live-session screenshot showed a visible macOS release window at `100%` card level. The displayed counters were `10,000` concepts, `50,000` relationships, and `30 visible / 30 rendered` cards.

Source: live-session screenshot provided on 2026-10-28.

This confirms the visible release window and displayed culling state. It does not measure native event delivery, pinch gestures, frame rate, Retina scaling, or Windows rendering.

## 2026-10-28: CAD-Style Navigation Foundation Boundary

Version one requires native CAD-style trackpad navigation: Shift-based pan and two-finger zoom or magnification behavior. This is a product requirement, not an emulated wheel-input behavior.

The pure canvas interaction boundary can accept framework-neutral navigation
commands, but the desktop foundation must expose and reliably deliver native
platform gesture events. The 2026-10-28 GPUI adapter had no native pinch or
magnify event path, so GPUI remained prototype-only. No replacement rendering
or desktop foundation was selected by that record.

## 2026-10-29: Eframe Presentation And Session Boundary

The 2026-10-29 candidate replaced the prior GPUI presentation experiment with a
project-owned `egui` plus `eframe` thin client in `crates/app`. The presentation
layer reads deterministic session data, paints only the bounded visible canvas
set, and sends typed actions through `crates/workspace-session`. It does not
own another persistence model.

`crates/workspace-session` owns staged workspace actions, local candidate-save
verification, backup, and restoration. `crates/canvas` remains independent of
eframe types and owns pure camera, visibility, and generated-data operations.
The shell therefore remains replaceable without moving persistence into UI
code.

The 2026-10-29 record designated `./start.sh` as the normal user launch path.
It sat outside the crate boundary and created or reused a safe external
evaluation workspace before invoking the candidate launcher. The native shell
and GPUI records above are historical evidence; they do not validate the
eframe candidate.

## 2026-10-30: WebAssembly Delivery Boundary

`crates/app` now builds both the native desktop binary and a WebAssembly
library from the same application source. The thin native `desktop_main.rs`
entry point calls the native shell. The WebAssembly `WebHandle` owns an
`eframe::WebRunner` and mounts the same `InfiniteNoteCanvasApp` in an HTML
canvas. The page in `web/index.html` loads only the generated bindings and
artifact; it does not add a separate JavaScript engine or UI.

For a browser session, `crates/app::workspace_session` creates the bundled,
validated workspace document only on first use. It then opens the selected
browser-local `WorkspaceSession`. `crates/workspace-session` keeps the same
candidate-save boundary for browser and desktop sessions: it retains a backup,
verifies the exact candidate source, and restores the prior source after a
verification failure. Browser persistence remains UI-free and is scoped to the
site origin. The shared File menu maps New, Open, Save, and Save As to named
browser-local workspaces. Open receives source only from a browser
user-selected file. Save and Save As create browser downloads. No browser
action claims ambient desktop-path access.

`./start.sh --web` is the supported local browser entry point. It compiles the
`wasm32-unknown-unknown` library, produces matching browser bindings, and
serves only the generated static files on `127.0.0.1`. Acceptance requires a
successful release Wasm build and a browser page that displays the loaded
canvas, controls, and inspector from the generated artifact. A saved browser
edit must remain after a page reload.
