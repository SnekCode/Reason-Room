# Native MVP Validation

Related: [Deletion and Undo/Redo](deletion-undo.md) · [Generic Edit History](generic-edit-history.md) · [Relationship Inspector](relationship-inspector.md) · [Product Decisions](product-decisions.md) · [Roadmap Handoff](roadmap-handoff.md) · [Agent Environment](agent-environment.md) · [Wiki Index](index.md)

> **Historical record — pre-reset native validation evidence.**
>
> On 2026-10-30, the product reset superseded this page as active manual-test
> guidance. It preserves dated evidence only. The reset handoff identifies
> `./start.sh` as the supported entry point for later manual evaluation; see
> [Product Reset](product-reset-2026-10-30.md) and [project
> instructions](../../AGENTS.md).

## 2026-10-28 — Accepted macOS Navigation

Manual acceptance evidence recorded launch and foreground card rendering, Shift hover-pan, native trackpad pinch zoom, wheel zoom fallback, concept movement, no-line dependency creation, and saved-view switching. The record limits this evidence to macOS; it does not establish Windows behavior.

Source: `docs/project-memory/macos-native-prototype.md` (2026-10-28); `docs/project-memory/product-decisions.md` (2026-10-28).

## 2026-10-28 — Validation Boundary

Remote accessibility and window-capture output can omit an occluded Metal layer. Such captures are not evidence that foreground native rendering failed. Native hardware interaction and visual checks require operator observation.

Source: `docs/project-memory/macos-native-prototype.md` (2026-10-28).

## 2026-10-29 — MVP Test Procedure

The test procedure uses `scripts/start-macos-native.sh` to launch one packaged instance. It covers navigation, concept creation, type choice, cancellation, move persistence, backup creation, no-line links, Lens switching, save-conflict protection, and read-only CLI inspection.

Source: `docs/tester-macos-mvp.md` (2026-10-29); `README.md` (2026-10-29).

## 2026-10-29 — Remaining Acceptance Scope

The roadmap places editing completeness, removal and inspection of typed relationships, clear local-file recovery behavior, Lens workflows, dependency analysis, scenarios, and controlled agent edits after the durable graph work. A passing build alone does not prove those behaviors.

Source: `docs/roadmap.md` (2026-10-29); `docs/project-memory/development.md` (2026-10-28).

## 2026-10-29 — Relationship Inspector Test Boundary

Future manual validation must cover opening the selected-concept relationship inspector and completing its two-step removal flow. The existing no-line relationship test remains applicable: graph links must not become persistent canvas lines.

Decision source: `docs/roadmap.md` (2026-10-29); user product decision context (2026-10-29). Operator procedure: `docs/tester-macos-mvp.md` (2026-10-29). This is not manual acceptance evidence. See [Relationship Inspector](relationship-inspector.md).

## 2026-10-29 — Inspector Behavior Pending Manual Acceptance

Manual macOS testing must verify the full-height readable Relationship inspector, neutral labels, source-selection retention for both link flows, relationship removal, auto-save, backup creation, and continued absence of relationship lines. No acceptance result is recorded by this entry.

Source: user product decision context (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29).

## 2026-10-29 — Relationship Linking Accepted

The user manually confirmed that no-line relationship linking works in the macOS app. This entry does not accept Relationship inspection, removal, or selection-aware Tab behavior.

Source: user manual validation context (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29).

## 2026-10-29 — Selection-Aware Tab Baseline Accepted

The user manually confirmed selected-concept centering and `SELECTED NOT IN LENS` behavior before Lens-local camera caching. This historical acceptance does not accept the newer session-camera behavior.

Source: user product decision context (2026-10-29); `docs/roadmap.md` (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29).

## 2026-10-29 — Lens-Local Session Camera State Implemented, Pending Manual Acceptance

Lens-local session camera state is implemented and pending manual macOS acceptance. The session-only cache is keyed by version and Lens. Tab stores the departing camera and restores the destination camera or default. A visible single selection centers at the destination Lens scale; an absent or filtered selection restores the destination camera and shows `SELECTED NOT IN LENS`; no or multiple selections restore the destination camera silently. No acceptance result is recorded.

This entry's pending manual-acceptance status is superseded only by the later same-date Lens-local session camera acceptance entry.

Source: user product decision context (2026-10-29); `docs/roadmap.md` (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29).

## 2026-10-29 — Lens-Local Session Camera State Accepted

The user manually confirmed that Lens-local session camera behavior works as expected. The acceptance covers the session-only cache per version and Lens, departing-camera storage, destination-camera restoration or default, selected-concept centering at the destination Lens scale, `SELECTED NOT IN LENS`, and silent no-or-multiple-selection restoration.

Source: user manual validation context (2026-10-29); regression procedure: `docs/tester-macos-mvp.md` (2026-10-29). See [Roadmap Handoff](roadmap-handoff.md).

## 2026-10-29 — Active-Version Deletion and Undo/Redo Pending Manual Acceptance

Manual macOS testing must verify that deletion applies only to the active Workspace Version; confirmation names the concept and reports exact incident relationship and Lens-placement counts; cancellation changes nothing; a successful save precedes deletion history; `Cmd+Z` and `Cmd+Shift+Z` restore and reapply deletion only; save failure leaves no deletion history; and no persistent relationship lines appear. No acceptance result is recorded.

This entry's pending manual-acceptance status is partially superseded by the later limited deletion acceptance entry. Active-version isolation, exact cascade counts, successful-save-only history, and save-failure recovery remain pending validation.

Source: user product decision context (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29). See [Deletion and Undo/Redo](deletion-undo.md).

## 2026-10-29 — Limited Deletion and Undo/Redo Acceptance

The user manually confirmed deletion cancellation, confirmed deletion, and deletion-only `Cmd+Z`/`Cmd+Shift+Z` undo/redo on macOS. Active-version isolation, exact cascade counts, successful-save-only history, and save-failure recovery remain unaccepted. Generic edit history is implemented with limited manual acceptance; only its stated boundary tests remain pending.

Source: user manual validation context (2026-10-29). See [Deletion and Undo/Redo](deletion-undo.md).

## 2026-10-29 — Generic Edit History Pending Manual Acceptance

Manual macOS testing must verify the bounded 64-entry, in-memory, successful-save-only history for concept create, completed Option-drag move, relationship create/removal, and concept deletion. It must verify workspace and selection restoration, redo clearing after a successful edit, excluded navigation/inspector paging/drafts, draft and deletion-confirmation shortcut consumption, and unchanged history after failed saves or candidate-byte verification failures. No acceptance result is recorded.

This entry's pending manual-acceptance status is partially superseded by the later limited generic-history round-trip acceptance entry. Redo clearing after a divergent edit, the 64-entry cap, excluded navigation/draft/modal behavior, and save-failure handling remain pending validation.

Source: user product decision context (2026-10-29); operator procedure: `docs/tester-macos-mvp.md` (2026-10-29). See [Generic Edit History](generic-edit-history.md).

## 2026-10-29 — Limited Generic History Round-Trip Acceptance

The user manually confirmed undo/redo round trips for concept creation, completed Option-drag moves, relationship creation, relationship removal, and concept deletion. Redo clearing after a divergent edit, the 64-entry cap, excluded navigation/draft/modal behavior, and save-failure handling remain unaccepted.

Source: user manual validation context (2026-10-29). See [Generic Edit History](generic-edit-history.md).
