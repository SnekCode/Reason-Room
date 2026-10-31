# Authoring Controls 2026-10-30

Related: [Authoring Refinement 2026-10-30](authoring-refinement-2026-10-30.md)
· [Deletion and Undo/Redo](deletion-undo.md)
· [Semantic-Zoom Authoring Repair 2026-10-30](semantic-zoom-authoring-repair-2026-10-30.md)
· [Wiki Index](index.md)

## 2026-10-30 — Workspace Authoring Decisions

The user requested that copy and paste duplicate selected concept cards without
duplicating typed graph relationships. A pasted card retained its Lens-local
placement, so spatial containment inside a card or visual boundary remained a
geometric result rather than a graph link.

The user requested Lens-view create, rename, and delete actions. Creating a
view copied the selected view so that its Lens-local arrangement, filters,
boundaries, and semantic-zoom presentation were retained. A version retained
at least one view.

The user requested Version create, rename, and delete actions. Creating a new
version copied the selected version. The resulting planning version retained
the source concepts, relationships, and views. The permanent current version
remained non-deletable, and a source version remained non-deletable while a
planning version depended on it.

## 2026-10-30 — Inspector and Canvas Interaction Decisions

The user requested direct concept-name editing on card double-click. Text
input autosave was added while preserving an unresolved assertion draft as a
save boundary. The inspector movement buttons were removed. Assertions moved
to a collapsed section at the bottom of the inspector. Relationship-kind
editing changed from a modal flow to a compatible-kind dropdown.

The user requested keyboard deletion for selected cards, visual boundaries,
and relationship lines. Delete and Backspace were ignored while a modal or
text editor held focus. A blank inspector could be collapsed when no entity
was selected.

The user clarified that an enclosing boundary became noninteractive context
when it fully covered the viewport on either the horizontal or vertical axis.
This prevented a distant parent boundary from taking movement focus while a
nested entity remained the intended target.

## Evidence Boundary

The implementation commits recorded deterministic coverage for the authoring
and interaction changes. Manual eframe evaluation remained deferred at the
user's direction. The commits did not record a participant acceptance result.

Source: user request (2026-10-30); commits `e80aa2d` (2026-10-30) and
`f6ad3e0` (2026-10-30).
