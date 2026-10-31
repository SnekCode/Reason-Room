# Relationship Inspector

Related: [Product Decisions](product-decisions.md) · [Native MVP Validation](native-mvp-validation.md) · [Wiki Index](index.md)

## 2026-10-29 — Selected-Concept Inspector

Roadmap development selected a relationship inspector for the active concept as the next relationship-management surface. It provides a readable place to inspect typed graph links without drawing persistent relationship lines across the canvas.

## 2026-10-29 — Two-Step Relationship Removal

Relationship removal requires two steps. This protects graph assertions from accidental deletion during canvas work while retaining the no-line canvas model.

Decision source: `docs/roadmap.md` (2026-10-29); user product decision context (2026-10-29). Operator procedure: `docs/tester-macos-mvp.md` (2026-10-29). Manual acceptance remains pending.

## 2026-10-29 — Kind-Editing Implementation Entered Manual Testing

Bounded model and native workers added an atomic relationship-kind change path,
neutral stored-direction inspector rows, explicit kind-edit controls, and
successful-save-only history integration. The implementation preserves
relationship IDs and endpoints, keeps two-step removal separate, and does not
draw persistent relationship lines. Parent integration passed formatting,
workspace tests, workspace build, and Clippy with warnings denied. The same
canonical work item moved to Testing; no macOS acceptance is recorded.

Sources: `docs/archive/work-items-2026-10-30/Complete/0005.md` (2026-10-29); `docs/roadmap.md`
(2026-10-29).

## 2026-10-29 — Selected-Shell Manual Pass and Closeout

The user recorded `Pass` in the canonical Work Item `0005` retest result for
the selected-shell relationship inspector. The tester noted that the older
detailed procedure was superseded after Work Item `0021`. The Work Item `0005`
closeout then completed and its roadmap acceptance entries were updated.

Source: `docs/archive/work-items-2026-10-30/Complete/0005.md` and
`docs/project-memory/eframe-candidate.md` (2026-10-29).

## 2026-10-29 — Quality Check Returned the Item to Native UI Remediation

The macOS quality check found that unsupported Unicode relationship glyphs
rendered as question marks. It also found that relationship-kind selection did
not match the searchable type-picker interaction. The chief returned the same
canonical work item to the root queue and delegated bounded native UI
remediation. The relationship model, candidate-save behavior, history, and
no-persistent-line rule remain unchanged.

Sources: `docs/archive/work-items-2026-10-30/Complete/0005.md` (2026-10-29); `docs/roadmap.md`
(2026-10-29).

## 2026-10-29 — ASCII-Safe Kind Picker Entered Retesting

A direct bounded native worker replaced unsupported relationship glyphs with
ASCII-safe labels and added searchable, scrollable relationship-kind selection.
The picker supports visible range controls, pointer selection, mouse-wheel
scrolling, and keyboard navigation. Parent integration passed formatting,
workspace tests, workspace build, and Clippy with warnings denied. The same
canonical item moved to Testing for focused macOS retest; no manual acceptance
is recorded.

Sources: `docs/archive/work-items-2026-10-30/Complete/0005.md` (2026-10-29); `docs/roadmap.md`
(2026-10-29).
