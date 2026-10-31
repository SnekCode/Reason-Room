# Roadmap Handoff

Related: [Product Decisions](product-decisions.md) · [Eframe Thin-Client Build
Path](eframe-build-path.md) · [Native MVP
Validation](native-mvp-validation.md) · [Relationship Inspector](relationship-inspector.md) · [Deletion and Undo/Redo](deletion-undo.md) · [Generic Edit History](generic-edit-history.md) · [Agent Environment](agent-environment.md) · [Wiki Index](index.md)

> **Historical record — superseded for active planning.**
>
> On 2026-10-30, the user reset the product direction. This page preserves
> pre-reset roadmap and work-item evidence; it does not prescribe the retired
> work-item process or product direction. See [Product Reset](product-reset-2026-10-30.md)
> and the active [Product Roadmap](../roadmap.md).

Primary records: [Product Roadmap](../roadmap.md) · [Native macOS Prototype](../project-memory/macos-native-prototype.md) · [Product Decisions](../project-memory/product-decisions.md) · [macOS Tester Guide](../tester-macos-mvp.md) · [GPUI Validation Route](../project-memory/gpui-prototype.md) · [Work-Item Queue](../archive/work-items-2026-10-30/README.md)

## 2026-10-29 — Accepted Manual macOS Behavior

The dated records show user acceptance for native canvas navigation, no-line relationship linking, selected-concept Tab baseline behavior, Lens-local session camera behavior, limited deletion cancellation and deletion-only undo/redo, and generic edit-history round trips for concept creation, completed moves, relationship creation, relationship removal, and deletion. Each acceptance has stated limits in its linked record.

Sources: [Native MVP Validation](native-mvp-validation.md); [Native macOS Prototype](../project-memory/macos-native-prototype.md); [Deletion and Undo/Redo](deletion-undo.md); [Generic Edit History](generic-edit-history.md).

## 2026-10-29 — Acceptance Clarification

The user's manual feedback explicitly accepted deletion cancellation, confirmed deletion, deletion-only undo/redo, generic-history round trips for the five covered edit types, and Lens-local session camera behavior. The remaining validation entries apply only to their stated boundaries. Selected-concept opt-in relationship-line visualization remains an active Milestone 2 roadmap item and is not deferred.

Sources: [Product Roadmap](../roadmap.md); [Deletion and Undo/Redo](deletion-undo.md); [Generic Edit History](generic-edit-history.md); [Native MVP Validation](native-mvp-validation.md).

## 2026-10-29 — Implemented Behavior With Remaining Validation

The records still require manual validation for the full-height Relationship inspector and two-step removal. Deletion retains unaccepted boundaries for active-version isolation, exact cascade counts, successful-save-only history, and save-failure recovery. Generic history retains unaccepted boundaries for redo clearing after a divergent edit, the 64-entry cap, excluded navigation/draft/modal behavior, and failed-save or candidate-byte verification behavior.

Sources: [Native MVP Validation](native-mvp-validation.md); [Relationship Inspector](relationship-inspector.md); [Deletion and Undo/Redo](deletion-undo.md); [Generic Edit History](generic-edit-history.md).

## 2026-10-29 — Ordered Outstanding Roadmap Chunks

1. **Milestone 1:** Keep schema migration, deterministic serialization, Custom Types, scenarios, Lenses, and per-Lens placement acceptance covered by fixtures.
2. **Milestone 2:** Complete concept editing, notes and assertions, Relationship inspector validation, selected-concept opt-in relationship lines, local recovery behavior, and remaining deletion/history validation boundaries.
3. **Milestone 3:** Add Lens creation and adding existing concepts to a Lens. Continue Lens templates, filters, semantic zoom, and Lens-scoped visual composition work.
4. **Milestones 4–6:** Deliver dependency analysis, Planning Scenario workflows, and the controlled local-agent contract.
5. **Milestones 7–8:** Complete macOS product-slice acceptance, then run Windows beta validation.

Source: [Product Roadmap](../roadmap.md).

## 2026-10-29 — Deferred Experiments

Shared canonical positions remain Prototype B. Drag-into-concept must use a chooser for visual nesting or a proposed typed relationship and must not create a semantic relationship automatically. Custom Type field schemas, evidence metadata, Access representation, and scenario retention policy remain deferred.

Sources: [Product Decisions](product-decisions.md); [Product Roadmap](../roadmap.md).

## 2026-10-29 — Eframe Queue And Process Reconciliation

Native and GPUI records remain historical evidence. The active implementation
path is the project-owned eframe candidate in Work Item `0021`. Its manual
result must be recorded in the same canonical Testing file before dependent
work can proceed. `./start.sh` is the only supported application launch entry
point.

The chief now watches numeric canonical work-item files in the root, `HITL`,
and `Testing` directories. A file moves between those directories without a
companion record. Only root `Ready` and `Done` items are schedulable, and a
`Ready` item requires every listed prerequisite to be Complete. The audit
returned `0003`, `0004`, `0005`, and `0008` to root `Ready` state with `0021`
as the gate.

Sources: [Work-Item Queue](../archive/work-items-2026-10-30/README.md); [Eframe Thin-Client Build
Path](eframe-build-path.md); [Work Item
0021](../archive/work-items-2026-10-30/Testing/0021.md); [Product Roadmap](../roadmap.md).
