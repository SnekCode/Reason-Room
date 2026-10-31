# Deletion and Undo/Redo

Related: [Product Decisions](product-decisions.md) · [Native MVP Validation](native-mvp-validation.md) · [Wiki Index](index.md)

## 2026-10-29 — Active-Version Deletion

Deletion applies only to the active Workspace Version. Confirmation names the concept and reports exact counts for all incident relationships and Lens placements that deletion will remove. Relationships remain free of persistent canvas lines.

## 2026-10-29 — Successful-Save-Only Deletion History

The local save must succeed before in-memory deletion history changes. `Cmd+Z` and `Cmd+Shift+Z` undo and redo deletion only. Save failure must leave the graph unchanged and create no deletion undo/redo history entry.

Source: user product decision context (2026-10-29); `docs/roadmap.md` (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29). The implementation awaits manual macOS acceptance.

## 2026-10-29 — Limited Manual Acceptance

The user manually accepted deletion cancellation, confirmed deletion, and deletion-only `Cmd+Z`/`Cmd+Shift+Z` undo/redo on macOS. This acceptance does not cover active-version isolation, exact incident-count reporting, successful-save-only history, or save-failure recovery. Generic history has limited round-trip acceptance for its five covered edit types; redo clearing, capacity, excluded interactions, and failure handling remain pending; see [Generic Edit History](generic-edit-history.md).

Source: user manual validation context (2026-10-29).
