# Product Decisions

Related: [Deletion and Undo/Redo](deletion-undo.md) · [Generic Edit History](generic-edit-history.md) · [Relationship Inspector](relationship-inspector.md) · [Native MVP Validation](native-mvp-validation.md) · [Roadmap Handoff](roadmap-handoff.md) · [Agent Environment](agent-environment.md) · [Wiki Index](index.md)

> **Historical record — model decisions superseded for active planning.**
>
> This page preserves decisions dated 2026-10-28 and 2026-10-29. On
> 2026-10-30, the product reset adopted organization-defined discovery and
> superseded its fixed taxonomy, relationship, and work-item assumptions. See
> [Product Reset](product-reset-2026-10-30.md) and the active [product
> vision](../product/vision.md).

## 2026-10-28 — Product Boundary

The user accepted a local-first, single-user desktop product. macOS was selected for the active slice; Windows beta work was deferred. Network services, telemetry, cloud storage, live collaboration, chat, Obsidian compatibility, and secret values require a later product decision.

Source: `docs/roadmap.md` (2026-10-28); `docs/project-memory/product-decisions.md` (2026-10-28).

## 2026-10-28 — Desktop Foundation

The native `winit` plus `wgpu` Metal shell was selected for the macOS product slice. GPUI remained feasibility reference code and not the selected application shell.

Source: `docs/roadmap.md` (2026-10-28); `docs/project-memory/product-decisions.md` (2026-10-28).

## 2026-10-28 — Workspace Graph Direction

The user accepted a graph with curated core types and display-only Custom Types; Team-originated `owns` and `operates` responsibility edges; one Current State; named Planning Scenarios; dynamic Lenses; and independent Lens Layouts. The accepted shape avoids Git-like history, rigid layers, custom field schemas, and relationship constraints in version one.

Source: `docs/adr/0001-workspace-graph-shape.md` (2026-10-28); `GLOSSARY.md` (2026-10-28); `docs/project-memory/product-decisions.md` (2026-10-28).

## 2026-10-29 — Storage Migration Priority

The roadmap placed a durable, deterministic workspace graph ahead of additional canvas interaction. The migration must preserve local, human-readable, AI-readable data and introduce scenarios, lenses, Custom Types, and accepted relationship vocabulary before broad editing features.

Source: `docs/roadmap.md` (2026-10-29); `docs/workspace-schema.md` (2026-10-29).

## 2026-10-29 — Eframe Thin-Client Parity Candidate

The user selected a project-owned `egui` plus `eframe` thin-client application
architecture for the next parity candidate. The candidate inherits the current
experiment's pointer-anchored wheel zoom and Shift-wheel, Shift-hover, and
middle-button pan behavior. Its `eframe` presentation must stay separate from
the UI-free workspace session and pure canvas model.

This selects the interim candidate direction, not removal of the accepted
native shell. `crates/macos-app` remains until the user makes a final decision
after automated and manual validation.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).

## 2026-10-29 — Relationship Inspection and Removal

Roadmap development selected a relationship inspector for a selected concept. Relationship removal uses two steps to avoid accidental deletion. The interaction keeps relationships off the canvas as persistent drawn lines.

Source: `docs/roadmap.md` (2026-10-29); user product decision context (2026-10-29). Operator procedure: `docs/tester-macos-mvp.md` (2026-10-29). Manual acceptance remains pending. See [Relationship Inspector](relationship-inspector.md).

## 2026-10-29 — Relationship Inspector Behavior

The implemented native behavior uses a full-height readable Relationship inspector with neutral labels. A selected source remains selected during selected-source plus Control-click and two-Control-click link creation. Typed relationships remain free of persistent canvas lines. This behavior requires manual macOS acceptance before it is recorded as passed.

Source: user product decision context (2026-10-29). Operator procedure: `docs/tester-macos-mvp.md` (2026-10-29). Manual acceptance remains pending.

## 2026-10-29 — Future Lens Visual Composition

The approved future direction assigns deterministic visual treatments to core types: Organization uses a dashed border with no fill; Team uses a dotted border with no fill; Project uses a solid border with diagonal hatch; Process uses a solid border with solid fill; Repository uses a solid border with cross hatch; Service uses a diagonal-stroke border with solid fill; and Environment uses a thick dashed border with translucent fill. Every concept may have explicit style overrides. Visual nesting, transforms, size, and rotation remain Lens Layout data and do not create semantic relationships. Zoom-aware creation is future work. Drag-into-concept remains a deferred experiment and cannot automatically create semantic relationships.

Source: user product decision context (2026-10-29); `docs/roadmap.md` (2026-10-29). This direction is not implementation evidence.

## 2026-10-29 — Drag-Into-Concept Requires a Chooser

The deferred drag-into-concept experiment must use a chooser for visual nesting or a proposed typed relationship. It must not create a semantic relationship automatically. No implementation or acceptance is recorded.

Source: user product decision context (2026-10-29); `docs/roadmap.md` (2026-10-29).

## 2026-10-29 — Execution-Mode-Safe Retyping

The user selected a data-preserving rule for concept retyping. A Process with
a stored execution mode cannot change to a non-Process core type or Custom
Type. The application must reject that edit without changing the workspace or
history until an approved execution-mode editing flow exists. A Process without
a stored execution mode may use the normal canonical type-validation path.

Source: user decision in `docs/archive/work-items-2026-10-30/Complete/0003.md` (2026-10-29); copied
to `docs/archive/work-items-2026-10-30/Complete/0003.md` (2026-10-29).

## 2026-10-29 — Selection-Aware Tab Baseline Accepted

The user manually accepted selected-concept centering and `SELECTED NOT IN LENS` behavior before Lens-local camera caching. This historical acceptance does not accept the newer session-camera behavior.

Source: user product decision context (2026-10-29); `docs/roadmap.md` (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29).

## 2026-10-29 — Lens-Local Session Camera State Implemented, Pending Manual Acceptance

Lens-local session camera state is implemented and pending manual macOS acceptance. The session-only cache is keyed by version and Lens. Tab stores the departing camera and restores the destination camera or default. A visible single selection centers at the destination Lens scale; an absent or filtered selection restores the destination camera and shows `SELECTED NOT IN LENS`; no or multiple selections restore the destination camera silently. No acceptance result is recorded.

This entry's pending manual-acceptance status is superseded only by the later same-date Lens-local session camera acceptance entry.

Source: user product decision context (2026-10-29); `docs/roadmap.md` (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29).

## 2026-10-29 — Lens-Local Session Camera State Accepted

The user manually confirmed that Lens-local session camera behavior works as expected. The acceptance covers the session-only cache per version and Lens, departing-camera storage, destination-camera restoration or default, selected-concept centering at the destination Lens scale, `SELECTED NOT IN LENS`, and silent no-or-multiple-selection restoration.

Source: user manual validation context (2026-10-29); regression procedure: `docs/tester-macos-mvp.md` (2026-10-29). See [Roadmap Handoff](roadmap-handoff.md).

## 2026-10-29 — Active-Version Deletion and Deletion-Only Undo/Redo

Deletion applies only to the active Workspace Version. Confirmation names the concept and reports exact incident relationship and Lens-placement counts. A local save must succeed before in-memory deletion history changes. `Cmd+Z` and `Cmd+Shift+Z` undo and redo deletion only. Relationships remain free of persistent canvas lines. The implementation awaits manual macOS acceptance.

This entry's pending manual-acceptance status is partially superseded by the later limited deletion acceptance entry. Active-version isolation, exact cascade counts, successful-save-only history, and save-failure recovery remain pending validation.

Source: user product decision context (2026-10-29); `docs/roadmap.md` (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29). See [Deletion and Undo/Redo](deletion-undo.md).

## 2026-10-29 — Limited Deletion and Undo/Redo Acceptance

The user manually accepted deletion cancellation, confirmed deletion, and deletion-only `Cmd+Z`/`Cmd+Shift+Z` undo/redo on macOS. This acceptance does not cover active-version isolation, exact cascade counts, successful-save-only history, or save-failure recovery. Generic edit history is implemented with limited manual acceptance; only its stated boundary tests remain pending.

Source: user manual validation context (2026-10-29). See [Deletion and Undo/Redo](deletion-undo.md).

## 2026-10-29 — Generic Edit History Implemented, Pending Manual Acceptance

Generic edit history is bounded to 64 in-memory entries and changes only after a successful local save. It covers concept creation, completed Option-drag moves, relationship creation, relationship removal, and concept deletion. `Cmd+Z` and `Cmd+Shift+Z` restore workspace and selection; a new successful edit clears redo. Camera and Lens navigation, inspector paging, and drafts are excluded. Drafts and deletion confirmation consume the shortcuts. Failed saves and candidate-byte verification failures leave history unchanged. No manual acceptance is recorded.

This entry's pending manual-acceptance status is partially superseded by the later limited generic-history round-trip acceptance entry. Redo clearing after a divergent edit, the 64-entry cap, excluded navigation/draft/modal behavior, and save-failure handling remain pending validation.

Source: user product decision context (2026-10-29); `docs/roadmap.md` (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29). See [Generic Edit History](generic-edit-history.md).

## 2026-10-29 — Limited Generic History Round-Trip Acceptance

The user manually accepted undo/redo round trips for concept creation, completed Option-drag moves, relationship creation, relationship removal, and concept deletion. Redo clearing after a divergent edit, the 64-entry cap, excluded navigation/draft/modal behavior, and save-failure handling remain unaccepted.

Source: user manual validation context (2026-10-29). See [Generic Edit History](generic-edit-history.md).

## 2026-10-29 — Selected-Concept Relationship Overlay Control

The user selected a canvas toolbar button for the opt-in selected-concept
relationship overlay. The button starts off, lasts only for the active
application session, and does not write workspace graph or Lens-layout data.
Persistent relationship lines remain absent by default.

Source: [Work Item 0007](../archive/work-items-2026-10-30/0007.md) (2026-10-29).
