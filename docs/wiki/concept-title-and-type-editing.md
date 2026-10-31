# Concept Title and Type Editing

Related: [Product Decisions](product-decisions.md) · [Native MVP Validation](native-mvp-validation.md) · [Wiki Index](index.md)

Primary records: [Work Item 0003](../archive/work-items-2026-10-30/0003.md) · [Complete 0003](../archive/work-items-2026-10-30/Complete/0003.md) · [Product Roadmap](../roadmap.md)

## 2026-10-29 — Implementation Entered Manual Testing

On 2026-10-29, bounded implementation work added a selected-concept native
title-and-type editing flow and canonical concept mutation. The type picker
uses curated core types and Custom Types stored in the selected workspace.
The user-selected data-preserving rule rejects a Process retype to a
non-Process type when that Process stores an execution mode. The parent
integration passed formatting, workspace tests, workspace build, and Clippy
with warnings denied. The work item entered manual macOS Testing; no manual
acceptance is recorded.

Sources: `docs/archive/work-items-2026-10-30/Complete/0003.md` (2026-10-29);
`docs/archive/work-items-2026-10-30/Complete/0003.md` (2026-10-29); user decision in
`docs/archive/work-items-2026-10-30/Complete/0003.md` (2026-10-29).

## 2026-10-29 — Manual Quality Check Returned the Item to Remediation

The macOS quality check reported that title editing could not insert spaces or
use the pointer to place or select text. It also reported that the type control
was not a searchable dropdown and that no test workspace exposed a Process
with a stored execution mode. The chief returned the same canonical work item
to the root queue, added these requirements, and delegated remediation. The
earlier accepted data-preserving Process retype rule remains unchanged.

Sources: `docs/archive/work-items-2026-10-30/Complete/0003.md` (2026-10-29); tester record in the same
canonical file (2026-10-29); `docs/roadmap.md` (2026-10-29).

## 2026-10-29 — Remediation Entered Focused Retesting

Bounded remediation added title cursor and selection behavior, typed-space
handling, a searchable valid-type dropdown, and a deterministic workspace
fixture with an automated Process. Parent integration passed formatting,
workspace tests, workspace build, and Clippy with warnings denied. The same
canonical work item moved to Testing for a focused macOS retest; no new manual
acceptance is recorded.

Sources: `docs/archive/work-items-2026-10-30/Complete/0003.md` (2026-10-29);
`workspaces/concept-editing.fixture.json` (2026-10-29); `docs/roadmap.md`
(2026-10-29).

## 2026-10-29 — Focused Retest Found Final UI Rendering Defects

The focused macOS retest found that the type dropdown could not scroll and its
unsupported arrow glyph rendered as a question mark. It also found that the
title cursor was misaligned for long text. The chief measured the renderer
mismatch: text uses a nine-cell glyph advance while the cursor and selection
used a smaller fixed advance. The same canonical item returned to the root
queue and entered bounded UI remediation. No manual acceptance is recorded.

Sources: `docs/archive/work-items-2026-10-30/Complete/0003.md` (2026-10-29); tester screenshot
`docs/archive/work-items-2026-10-30/Testing/i0003_test.png` (2026-10-29); `docs/roadmap.md`
(2026-10-29).

## 2026-10-29 — Final UI Correction Entered Complete Retesting

Bounded remediation replaced the divergent cursor metric with the renderer's
nine-cell bitmap glyph advance. It also replaced the unsupported dropdown glyph
and made every filtered type match reachable through visible controls, pointer,
wheel, and keyboard scrolling. Parent integration passed formatting, workspace
tests, workspace build, and Clippy with warnings denied. The same canonical
item moved to Testing for a complete macOS retest; no manual acceptance is
recorded.

Sources: `docs/archive/work-items-2026-10-30/Complete/0003.md` (2026-10-29);
`docs/roadmap.md` (2026-10-29).
