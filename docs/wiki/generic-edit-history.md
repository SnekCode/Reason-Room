# Generic Edit History

Related: [Deletion and Undo/Redo](deletion-undo.md) · [Product Decisions](product-decisions.md) · [Native MVP Validation](native-mvp-validation.md) · [Wiki Index](index.md)

## 2026-10-29 — Successful-Save-Only History

Generic edit history is bounded to 64 in-memory entries and changes only after a successful local save. It covers concept creation, completed Option-drag moves, relationship creation, relationship removal, and concept deletion. Failed saves and candidate-byte verification failures leave history unchanged.

## 2026-10-29 — Undo/Redo Scope

`Cmd+Z` and `Cmd+Shift+Z` restore workspace and selection. A new successful edit clears redo. Camera and Lens navigation, inspector paging, and drafts are excluded. Drafts and deletion confirmation consume the shortcuts.

## 2026-10-29 — Manual Acceptance Boundary

The user manually accepted undo/redo round trips for concept creation, completed Option-drag moves, relationship creation, relationship removal, and concept deletion. Redo clearing after a divergent edit, the 64-entry cap, excluded camera/Lens navigation, inspector paging, draft and deletion-confirmation shortcut consumption, and save-failure handling remain pending manual macOS acceptance. The limited deletion acceptance remains distinct.

Source: user product decision context (2026-10-29); `docs/roadmap.md` (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29).

## 2026-10-29 — Selected-Shell Boundary Validation Passed

The user passed Work Item `0006` selected-shell macOS validation for undo and
redo ordering, divergent-edit redo clearing, 65 successful saved edits,
excluded navigation and draft interactions, and draft or deletion-dialog
shortcut consumption. The external `./start.sh` evaluation launch remained
open. The known limits remain a 64-entry successful-save history, no history
entries for navigation or drafts, and no history across relaunch.

Source: [Work Item 0006](../archive/work-items-2026-10-30/Complete/0006.md) (2026-10-29).
