# Product Decisions (Pre-Reset Historical Record)

> **Historical pre-reset evidence.** This page preserves dated decisions made
> before the [2026-10-30 product reset](../wiki/product-reset-2026-10-30.md).
> Its fixed taxonomy, schema, shell, and work-item claims are not active
> guidance. Use the [product roadmap](../roadmap.md) and
> [product vision](../product/vision.md) for active discovery. For manual
> application evaluation, use `./start.sh` only.

## 2026-10-27: Product Focus

The product will be a local-first desktop tool for one person. It will target macOS first and include a Windows beta validation target. It will help users understand complex organizations, teams, projects, systems, services, and environments.

## 2026-10-27: Structured Graph

Concepts will have stable IDs, structured fields, detailed note content, and typed relationships. One concept may appear in multiple saved views. Moving a concept in one view must not change its identity or graph relationships.

## 2026-10-27: Layers and Visibility

Layers will be saved views over one graph. Relationships will be stored as data and hidden by default. Contextual relationship display will be tested after the core canvas works.

## 2026-10-28: Dynamic Lenses With Manual Layouts

Version one uses dynamic filtered lenses over one workspace graph. Each lens has its own manually arranged concept placements. Moving a concept changes its placement in the active lens only. A shared canonical-position model is deferred as a comparison prototype rather than assumed by the product architecture.

## 2026-10-28: Extensible Concept and Responsibility Model

Version one has a curated core type set plus user-defined custom types. Responsibility uses `owns` for accountability and `operates` for day-to-day running responsibility; `managed_by` is not part of the target version-one vocabulary. Concepts and relationships are user assertions, with optional explanatory notes rather than required evidence metadata. Workspaces support named local versions for current and planned states without Git-style commit tracking.

Traversal boundaries are deferred as lens-query configuration. They are not a fixed global schema rule until concrete use cases establish their behavior.

## 2026-10-28: Infinite Navigation

The canvas has no document boundary for horizontal or vertical navigation. Camera movement is not position-clamped. Zoom is continuous and uses a wide 1% to 409,600% working range. Those endpoints are numerical-safety guards, not conceptual canvas limits. A future floating-origin implementation can extend the safe range further without changing workspace data.

## 2026-10-27: Local Data and Agents

Workspaces will use a versioned portable schema. Local agent tools will read structured data and create reviewable proposals. Approved changes will be recorded as reversible local transactions.

## 2026-10-27: Deferred Features

The first product version will not require chat, live collaboration, cloud services, telemetry, freehand drawing, or Obsidian compatibility.

## 2026-10-27: Desktop Foundation

GPUI is prototype-gated. It has not been selected as the production desktop foundation. A macOS and Windows acceptance run must provide the evidence for that decision.

## 2026-10-27: Prototype Evidence Boundary

The workspace bootstrap and pure canvas module provide implementation evidence only. They do not select GPUI and do not provide native GPUI, macOS, or Windows validation evidence. The platform acceptance run remains required before a production desktop-foundation decision.

## 2026-10-27: GPUI Prototype Progress Boundary

The prototype source implements the required first interaction set: capture-listener pan, pointer-anchored wheel zoom, click and Shift-click selection, drag-move rectangle selection, viewport culling, selected highlights, and three semantic render levels. The semantic implementation includes generated summaries, summary-bearing detail cards, bounded overview clusters, and an explicit cluster fallback when more than `750` cards are visible. This implementation evidence does not select GPUI. A successful native build and interactive acceptance run on macOS and Windows remain required before the desktop-foundation decision.

## 2026-10-27: Static Check Does Not Select GPUI

An isolated temporary workspace copy passed offline GPUI workspace type checking with only the `runtime_shaders` feature enabled. The production manifests retain the default GPUI renderer and were not changed. Application interaction type errors identified by that check were fixed.

The static check does not validate the default renderer, native runtime behavior, performance, macOS Retina scaling, Windows scaling, native trackpad pinch, or either platform acceptance run. GPUI remains prototype-gated until the default-renderer application passes the macOS and Windows validation requirements.

## 2026-10-28: Metal Component Evidence Does Not Select GPUI

Xcode `26.6`, `metal`, and `metallib` were found on the macOS host. The default GPUI shader build then reported that its Metal Toolchain component was missing. The required component-download command did not run because the automated approval service returned `404` before execution.

This evidence does not select GPUI. The default macOS renderer has not built or run, and Windows native validation remains unproven. The macOS and Windows acceptance runs remain required before the production desktop-foundation decision.

## 2026-10-28: User-Terminal Default-Renderer Boundary

The user's normal Terminal reported an installed Metal Toolchain and resolved both `metal` and `metallib` with `xcrun --no-cache`. The agent sandbox cannot access that toolchain state, so its default-renderer GPUI builds still stop before application compilation.

macOS default-renderer build and runtime validation must run in the user's normal Terminal. GPUI remains prototype-gated until those macOS results and the required Windows results are recorded.

## 2026-10-28: Validation Context Does Not Select GPUI

The agent and user processes reported the same macOS account, home directory, and selected Xcode developer directory. The agent sandbox could not persist `~/Library/Developer/Xcode/XcodeToMetalToolchainIndexMapping.plist`, so its `xcrun` lookup failed even though it could execute the mounted Metal Toolchain binaries directly. The user's normal Terminal resolved the toolchain correctly.

`scripts/validate-macos-gpui.sh` is the standard project-local runner for user-Terminal macOS build checks. It runs offline format, test, Clippy, and release-build checks without downloads or an application launch. This execution-context finding and the runner do not select GPUI. The required interactive macOS and Windows acceptance evidence remains necessary.

## 2026-10-28: Interaction Refactor Evidence Does Not Select GPUI

The prototype now keeps interaction decisions in a pure controller with eleven deterministic tests. The GPUI adapter duplicate-dispatch defect was corrected, the workspace passed `22` tests and a release build, and a macOS release window was restarted after the refactor without shared-pointer automation.

This evidence does not select GPUI. Native trackpad pinch remains pending. Windows release-build, launch, input, performance, and display-scaling requirements remain pending. The production desktop-foundation decision still requires the full macOS and Windows acceptance evidence.

## 2026-10-28: Pure-Model Benchmark Does Not Select GPUI

The release benchmark for the deterministic `10,000`-concept and `50,000`-relationship pure canvas path reported: dataset generation `7.370 ms`; viewport-query total `0.634 ms` with `p99 0.009 ms`; cluster-query total `5.359 ms` with `p99 0.077 ms`; zoom `0.023 ms`; pan `0.024 ms`; clicks `0.443 ms`; and marquee `1.149 ms`.

These measurements support the pure data-model and interaction-controller design. They do not prove GPUI GPU FPS, native input behavior, macOS Retina behavior, Windows scaling, or either platform's sustained pan-and-zoom performance. GPUI remains prototype-gated.

## 2026-10-28: Windows Validator Does Not Select GPUI

`scripts/validate-windows-gpui.ps1` standardizes the Windows beta-machine build and process-liveness checks. It improves repeatability of toolchain inspection, offline Rust validation, release building, executable launch, and timestamped report capture.

The validator does not perform native input, display-scaling, or performance testing. It does not select GPUI as the production desktop foundation. The Windows acceptance record still requires separate native device evidence, and native PowerShell parsing of the validator remains unverified on the macOS host because PowerShell is not installed.

## 2026-10-28: Live Visual Evidence Does Not Select GPUI

The live-session screenshot showed the macOS release window on screen at `100%` card level. It displayed `10,000` concepts, `50,000` relationships, and `30 visible / 30 rendered` cards.

Source: live-session screenshot provided on 2026-10-28.

This is visual macOS launch and viewport-culling display evidence only. It does not prove native input delivery, native trackpad pinch, sustained FPS, Retina scaling, or Windows behavior. It does not select GPUI as the production desktop foundation.

## 2026-10-28: CAD-Style Trackpad Navigation Is Required

The user confirmed that version one requires native CAD-style trackpad navigation. The required behavior includes Shift-based pan and two-finger zoom or magnification behavior. Wheel-only zoom is not an acceptable replacement.

The 2026-10-28 GPUI implementation had no native pinch or magnify event
handling. GPUI was prototype-only and was not selected as the production
desktop foundation. No replacement foundation was selected by this decision.

The input contract is now explicit: hold Shift and move the mouse to pan; a native two-finger pinch or magnify gesture zooms around the gesture location; primary-button drag creates a marquee selection; and Command-click on macOS or Ctrl-click on Windows extends a selection. Middle-button drag and wheel zoom are fallback controls only. The next desktop-foundation prototype must prove the native pinch behavior on both macOS and Windows.

## 2026-10-28: macOS-First Native Shell Slice

The user approved the macOS slice and deferred Windows work. The native Rust prototype in `crates/macos-app` uses `winit` for macOS input and `wgpu` for Metal rendering while retaining the pure shared canvas model. GPUI remains the completed feasibility reference rather than the active macOS product-shell candidate.

The macOS-native shell must still pass a manual hardware-trackpad pinch test. Windows has no current implementation or acceptance claim.

## 2026-10-28: macOS Native Shell Selected

The user accepted wheel zoom, native trackpad pinch zoom, and Shift hover-pan in the macOS release app. The `winit` and `wgpu` implementation in `crates/macos-app` is selected as the active macOS application shell. GPUI remains the completed feasibility reference. Windows remains deferred.

## 2026-10-28: Lightweight Custom Types in Version One

Version one supports curated core concept types and user-defined custom types. A custom type has display metadata only: a name, color, and optional icon. It does not add custom fields or restrict allowed relationships. This preserves user flexibility while deferring schema and validation complexity until usage establishes a need.

## 2026-10-28: Current State and Planning Scenarios

A workspace has one named Current State and may have multiple named Planning Scenarios. A Planning Scenario is rooted in a selected Workspace Version and models an alternative without changing the Current State. Workspace Versions are local graph revisions, not Git-like commit history.

## 2026-10-28: Team-Directed Responsibility Relationships

Version one stores responsibility as `Team ─owns→ Concept` and `Team ─operates→ Concept`. This matches the domain language and directly supports team-centered questions. It replaces the provisional `managed_by` vocabulary when the workspace schema is redesigned.

## 2026-10-28: Editable Lens Templates

The application provides editable starter Lens Templates for Ownership, Processes & Dependencies, Services & Access, and Deployment. Each template creates a dynamic Lens that the user can change or replace. Templates do not impose fixed layers, canonical positions, or global traversal boundaries.

## 2026-10-29: Structural Contains Relationship

`contains` is a structural relationship distinct from responsibility. It preserves organization-to-team and system-boundary mappings without overloading `owns` or `operates`, which remain Team-to-Concept responsibility relationships. The version-one migration converts legacy non-Team `owns` records to `contains`.

## 2026-10-29: Drag-Into-Concept Requires a Chooser

Drag-into-concept is a deferred interaction experiment. When explored, it must present a chooser for visual nesting or a proposed typed relationship. It must not create a semantic relationship automatically.

## 2026-10-29: Selection-Aware Tab Accepted

The user manually accepted selection-aware Tab behavior on macOS. With exactly one selected concept, Tab centers it in the next Lens at unchanged zoom when visible; if absent or filtered, the camera remains unchanged and shows `SELECTED NOT IN LENS`. No or multiple selections retain existing camera behavior.

## 2026-10-29: Lens-Local Session Camera State Is Pending

Lens-local session camera state is implemented and pending manual macOS acceptance. The cache is session-only and keyed by workspace version and Lens. Tab stores the departing camera and restores the destination camera or default. A visible single selection centers at the destination Lens scale; an absent or filtered selection restores the destination camera and shows `SELECTED NOT IN LENS`; no or multiple selections restore the destination camera silently.

## 2026-10-29: Lens-Local Session Camera State Accepted

The user manually confirmed that Lens-local session camera state works as expected. The acceptance covers the session-only cache per workspace version and Lens, departing-camera storage, destination-camera restoration or default, selected-concept centering at the destination Lens scale, `SELECTED NOT IN LENS`, and silent no-or-multiple-selection restoration.

## 2026-10-29: Active-Version Deletion and Deletion-Only Undo/Redo

Deletion applies only to the active Workspace Version. Confirmation names the concept and reports exact incident relationship and Lens-placement counts. The local save must succeed before in-memory deletion history changes. `Cmd+Z` and `Cmd+Shift+Z` undo and redo deletion only. The user manually accepted deletion cancellation, confirmed deletion, and deletion-only undo/redo on 2026-10-29. General undo/redo for create, move, link, and unlink remains pending implementation.

## 2026-10-29: Generic Edit History Pending Manual Acceptance

Generic edit history is bounded to 64 in-memory entries and changes only after a successful local save. It covers concept creation, completed Option-drag moves, relationship creation, relationship removal, and concept deletion. `Cmd+Z` and `Cmd+Shift+Z` restore workspace and selection; a new successful edit clears redo. Camera and Lens navigation, inspector paging, and drafts are excluded. Drafts and deletion confirmation consume the shortcuts. Failed saves and candidate-byte verification failures leave history unchanged. The user manually accepted round trips for all five covered edit types on 2026-10-29. Redo clearing after a divergent edit, the 64-entry cap, excluded navigation/draft/modal behavior, and save-failure handling remain pending manual acceptance.

## 2026-10-29: Eframe Thin-Client Build Path Selection

The 2026-10-29 implementation path selected a project-owned `egui` plus
`eframe` thin-client candidate. The application shell remained a client of the
UI-free workspace session and pure canvas model. It retained pointer-anchored
wheel zoom and the selected pan interactions.

The 2026-10-29 plan used Work Item `0021` as a manual-evaluation gate before
dependent selected-shell work. That retired work-item gate does not prescribe
new work after the product reset. The record left the final shell decision to
the user and did not treat historical native results as eframe product
acceptance.

The 2026-10-29 record designated `./start.sh` as the only supported
application launch entry point. It protected repository workspace data by
creating or using a writable external evaluation workspace.
