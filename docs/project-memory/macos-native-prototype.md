# Native macOS Prototype (Historical)

> **Historical pre-reset evidence.** This page preserves 2026-10-28 and
> 2026-10-29 native-shell results. It is not active product, shell, launch, or
> work-item guidance. The [2026-10-30 product reset](../wiki/product-reset-2026-10-30.md)
> supersedes its planning assumptions; use the [product roadmap](../roadmap.md)
> and [product vision](../product/vision.md) for active discovery. For manual
> application evaluation, use `./start.sh` only. Do not use legacy `cargo run`
> or `scripts/*` launch commands shown below.

## Scope

This prototype evaluates a native Rust macOS shell for the infinite canvas. It keeps the pure `crates/canvas` model and replaces the GPUI adapter with `winit` input and a `wgpu` Metal renderer.

The macOS scope includes native trackpad pinch delivery, Shift-plus-mouse-movement pan, selection, GPU drawing, and viewport culling. Windows is explicitly deferred by the user and has no acceptance claim in this record.

## Acceptance Evidence

| Requirement | Status | Evidence |
| --- | --- | --- |
| Native Metal release build | Passed | `cargo build --release --package infinite-note-canvas-macos` passed on 2026-10-28. |
| Native macOS window launch | Passed | The release process was visible as `infinite-note-canvas-macos` with the title `144 visible / 10000 concepts / 101%` on 2026-10-28. |
| Foreground Metal canvas rendering | Passed | The user directly observed concept cards in the foreground macOS window on 2026-10-28. Remote accessibility and window-capture tools do not reliably include the Metal layer while the app is occluded behind the terminal. |
| Viewport culling | Passed | The live title reported `144 visible / 10000 concepts`; only visible cards generate GPU vertices. |
| Shift hover-pan | Passed | The user manually confirmed Shift hover-pan on macOS on 2026-10-28. |
| Native trackpad pinch | Passed | The user manually confirmed native trackpad pinch zoom on macOS on 2026-10-28. The adapter maps `winit::WindowEvent::PinchGesture` to pointer-anchored `ZoomAt` commands. |
| Pointer-anchored zoom math | Passed | The pure canvas tests include pointer-anchor stability for magnification. |
| Native input-command mapping | Passed | Five deterministic `MacosInputAdapter` tests cover Shift hover-pan, pinch conversion, selection, and middle-drag pan. |
| Card and marquee selection | Pending manual acceptance | The native adapter maps primary press, move, and release events to the pure selection commands. |
| Concept move persistence | Passed | The user verified Option-drag movement in the native macOS app on 2026-10-28. |
| No-line relationship editing | Passed | The user verified Control-click source and target linking in the native macOS app on 2026-10-28. |
| No-line relationship linking, reconfirmed | Passed | The user manually confirmed linking works on macOS on 2026-10-29. |
| Selected-concept Relationship inspection and removal | Pending manual acceptance | The app provides a full-height inspector with neutral labels and `REMOVE` then `CONFIRM` controls. Follow `docs/tester-macos-mvp.md`. |
| Saved-view switching | Passed | The user verified Tab-based saved-view switching in the native macOS app on 2026-10-28. |
| Selection-aware Tab baseline | Passed | The user manually confirmed selected-concept centering and `SELECTED NOT IN LENS` behavior on 2026-10-29 before Lens-local camera caching. |
| Lens-local session camera state | Passed | The user manually confirmed on 2026-10-29 that it works as expected: the session-only cache is keyed by version and Lens; Tab stores the departing camera and restores the destination camera or default; a visible single selection centers at the destination Lens scale; absent or filtered selections restore the destination camera and show `SELECTED NOT IN LENS`; no or multiple selections restore the destination camera silently. |
| Active-version deletion and deletion undo/redo | Passed, limited scope | The user manually confirmed cancellation, confirmed deletion, and deletion-only `Cmd+Z`/`Cmd+Shift+Z` undo/redo on 2026-10-29. Active-version isolation, exact cascade counts, successful-save-only history, and save-failure recovery remain separate validation boundaries. |
| Generic edit history | Passed, limited scope | The user manually confirmed undo/redo round trips for concept create, completed Option-drag move, relationship create/removal, and concept deletion on 2026-10-29. Redo clearing after a divergent edit, the 64-entry cap, excluded navigation/draft/modal behavior, and save-failure handling remain separate validation boundaries. |

## Controls

- Hold Shift and move the mouse with no button pressed to pan.
- Pinch on the macOS trackpad to zoom around the current pointer location.
- Click to select.
- Drag with the primary button to select a rectangle.
- Command-click to extend selection.
- Hold Option and drag a concept to move it in the active saved view.
- Control-click a concept, then Control-click another concept to add a `depends_on` relationship without drawing a line.
- Click a concept to open its full-height Relationship inspector. A selected source remains selected through selected-source plus Control-click and two-Control-click linking. Click `REMOVE`, then `CONFIRM`, to remove one listed relationship without drawing a line.
- Press Tab to cycle Lenses. The app keeps session-only camera state per version and Lens. Tab stores the departing camera and restores the destination camera or default. With one visible selected concept, Tab centers it at the destination Lens scale. If absent or filtered, it restores the destination camera and shows `SELECTED NOT IN LENS`. With no or multiple selections, it restores the destination camera silently. Manual macOS acceptance was confirmed and reconfirmed on 2026-10-29.
- Middle-button drag and wheel zoom are fallbacks.

## Historical Run and Validation Record

The historical protocol used `scripts/start-macos-native.sh` to start one
packaged native app instance.

It used `scripts/validate-macos-native.sh` to run offline build checks and
print the manual macOS acceptance checklist.

## 2026-10-28: Native Input Adapter Verification

The macOS event adapter now has a framework-free `MacosInputAdapter` boundary. It maps cursor, primary-button, middle-button, wheel, and native pinch intents to the shared `CanvasCommand` values. Five deterministic unit tests passed for this boundary. The complete workspace passed `28` tests after the adapter tests were added.

The release app was relaunched after this verification. Its accessible window title reported a bounded visible-card count and an active wheel-zoom fallback state. This confirms launch, rendering, and event dispatch. It does not substitute for a hardware trackpad pinch observation.

## 2026-10-28: Native Navigation Acceptance

The user manually confirmed wheel zoom, native trackpad pinch zoom, and Shift hover-pan in the macOS release app. The macOS native-navigation acceptance gate passes.

## 2026-10-28: Infinite Navigation Range

The camera has no horizontal or vertical position clamp. Zoom now supports a 1% to 409,600% working range. The endpoints protect finite-precision camera transforms and do not represent document boundaries. A floating-origin upgrade remains the next precision improvement for navigation at extreme distances.

## 2026-10-28: Local Workspace Editing

The native app loads the local versioned JSON workspace by default. Hold Option and drag a concept to move its placement in the active saved view. The app preserves concept identity and typed relationships, writes a deterministic JSON replacement, and retains the prior file as a `.bak` backup. Relationship lines remain hidden.

## 2026-10-28: No-Line Relationship Editing

Control-click a source concept and Control-click a target concept to create a typed `depends_on` relationship. The source concept is highlighted while the link is pending. The app saves the typed relationship locally and still renders no relationship line.

## 2026-10-28: Local Editing Acceptance

The user verified the native app controls for moving a concept with Option-drag, creating a no-line relationship with two Control-clicks, and switching saved views with Tab. These controls persisted the expected local workspace changes and operated without network services.

## 2026-10-28: macOS App Bundle

`scripts/package-macos-native.sh` creates an ad-hoc-signed local `Infinite Note Canvas.app` bundle under `dist`. The bundle uses the native Rust executable. When started outside the repository without `INFINITE_NOTE_CANVAS_WORKSPACE`, it creates and uses a local workspace under `~/Library/Application Support/Infinite Note Canvas/`.

## 2026-10-28: Bundled-App Launch Evidence

The packaging script created `dist/Infinite Note Canvas.app`, and `codesign --verify --deep --strict` passed after ad-hoc signing. The bundled application launched successfully with bundle identifier `com.local.infinite-note-canvas`. It created the default local workspace under `~/Library/Application Support/Infinite Note Canvas/acme.infinite-note.json`; the workspace inspector validated its schema, six concepts, six typed relationships, and two saved views.

## 2026-10-29: MVP Workspace and Save-Safety Update

The native app now reads schema version two and migrates version-one workspaces in memory. Schema version two supports Current State, Planning Scenarios, Custom Types, dynamic Lenses, and independent Lens Layouts. The app now creates a core concept with `N`, text input, Up or Down type selection, and Enter. It still persists card movement and no-line `depends_on` relationships locally.

Before each save, the app compares the loaded workspace digest with the file on disk. If the file changed outside the app or a save fails, it restores the prior in-memory graph and reports `SAVE FAILED` in the window title. A local CLI provides deterministic JSON `inspect`, `validate`, and graph `query` commands.

Manual macOS acceptance for the new creation flow, selected-concept relationship inspection and removal, schema version-two save, and external-change recovery is pending. Follow `docs/tester-macos-mvp.md` and record the results in this file.

## 2026-10-29: Relationship Inspector Implementation Pending Manual Acceptance

The native app provides a full-height readable Relationship inspector with neutral labels. It retains a selected source during selected-source plus Control-click and two-Control-click relationship creation. Typed relationship lines remain absent. Manual macOS acceptance remains pending; follow `docs/tester-macos-mvp.md`.

## 2026-10-29: Relationship Linking Acceptance

The user manually confirmed that no-line relationship linking works in the macOS app. This acceptance covers linking only. It does not accept the Relationship inspector, relationship removal, or selection-aware Tab behavior.

## 2026-10-29: Deferred Drag-Into-Concept Work

Drag-into-concept is not implemented. The future experiment must present a chooser for visual nesting or a proposed typed relationship; it must never create a semantic relationship automatically.

## 2026-10-29: Selection-Aware Tab Baseline Acceptance

The user manually confirmed selected-concept centering and `SELECTED NOT IN LENS` behavior before Lens-local camera caching. This historical acceptance does not accept the newer session-camera behavior.

## 2026-10-29: Lens-Local Session Camera State Implemented

The native app implements a session-only camera cache per version and Lens. Tab stores the departing camera and restores the destination camera or default. With one visible selected concept, Tab centers it at the destination Lens scale. If absent or filtered, it restores the destination camera and shows `SELECTED NOT IN LENS`. With no or multiple selections, it restores the destination camera silently. Manual macOS acceptance was confirmed and reconfirmed on 2026-10-29; follow `docs/tester-macos-mvp.md` for regression checks.

This entry's pending manual-acceptance status is superseded only by the later same-date Lens-local session camera acceptance entry.

## 2026-10-29: Lens-Local Session Camera State Acceptance

The user manually confirmed that the Lens-local session camera behavior works as expected. This acceptance covers the session-only version-plus-Lens cache, departing-camera storage, destination-camera restoration or default, visible selected-concept centering at the destination Lens scale, `SELECTED NOT IN LENS`, and silent no-or-multiple-selection restoration. Keep `docs/tester-macos-mvp.md` as the regression checklist.

## 2026-10-29: Active-Version Deletion and Undo/Redo Implemented, Pending Manual Acceptance

The native app deletes a concept only from the active Workspace Version. Confirmation names the concept and reports exact counts for incident relationships and Lens placements that deletion will remove. The local save must succeed before in-memory deletion history changes. `Cmd+Z` and `Cmd+Shift+Z` undo and redo deletion only. Relationships remain free of persistent canvas lines. Manual macOS acceptance remains pending; follow `docs/tester-macos-mvp.md`.

This entry's pending manual-acceptance status is partially superseded by the later limited deletion acceptance entry. Active-version isolation, exact cascade counts, successful-save-only history, and save-failure recovery remain pending validation.

## 2026-10-29: Deletion and Deletion-Only Undo/Redo Acceptance

The user manually confirmed deletion cancellation, confirmed deletion, and deletion-only `Cmd+Z`/`Cmd+Shift+Z` undo/redo on macOS. This acceptance does not cover active-version isolation, exact cascade counts, successful-save-only history, or save-failure recovery. Generic edit history is implemented with limited manual acceptance; only its stated boundary tests remain pending.

## 2026-10-29: Generic Edit History Implemented, Pending Manual Acceptance

The native app implements bounded 64-entry, in-memory, successful-save-only undo/redo for concept creation, completed Option-drag moves, relationship creation, relationship removal, and concept deletion. `Cmd+Z` and `Cmd+Shift+Z` restore workspace and selection. A new successful edit clears redo. Camera and Lens navigation, inspector paging, and drafts are excluded. Drafts and deletion confirmation consume the shortcuts. Failed saves and candidate-byte verification failures leave history unchanged. Manual macOS acceptance remains pending; follow `docs/tester-macos-mvp.md`.

This entry's pending manual-acceptance status is partially superseded by the later limited generic-history round-trip acceptance entry. Redo clearing after a divergent edit, the 64-entry cap, excluded navigation/draft/modal behavior, and save-failure handling remain pending validation.

## 2026-10-29: Generic Edit History Round-Trip Acceptance

The user manually confirmed undo/redo round trips for concept creation, completed Option-drag moves, relationship creation, relationship removal, and concept deletion. This acceptance does not cover redo clearing after a divergent edit, the 64-entry cap, excluded navigation/draft/modal behavior, or save-failure handling.
