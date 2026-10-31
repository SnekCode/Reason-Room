# Authoring Refinement 2026-10-30

## Outcome

The active eframe path now uses a top-bar `Create` menu. It opens the shared
right pane in one focused mode at a time:

- **Card** asks for a name and short summary. Type is optional.
- **Boundary** asks for a name and short context.
- **Zoom layer** asks only for a name. Its fade ranges and card scale remain
  available in the layer editor after creation.

The concept inspector and all creation modes use the same scrollable pane.
There is no separate authoring panel.

## Relationship Creation

Relationship creation is no longer a top-level action. Two flows create the
default `depends on` relationship without a confirmation dialog:

1. Select two cards with Command-click. The inspector identifies the primary
   card. It offers an immediate action from that card to each other selected
   card. A person can make another selected card primary before they create
   the relationship.
2. Drag from a card's blue right-edge port to another card's white left-edge
   port or body. Releasing creates the relationship at once. Releasing on the
   source card cancels the drag.

## Boundary Treatment

Relationship paths now split at each visual-boundary edge. The portion outside
all boundaries is route blue. A portion inside one or more boundaries is
control amber. Overlapping boundaries keep the path in control context. This
is a visual cue only. It does not create membership, ownership, responsibility,
containment, hierarchy, or a relationship.

## Validation Boundary

Deterministic coverage includes boundary entry and exit, overlapping boundary
segments, source and target port hit testing, body drops, self-drop rejection,
primary-card retention, shared-pane creation modes, and the absence of a
relationship-creation confirmation path.

Manual eframe evaluation remains deferred. If it resumes, first confirm that
the current evaluation window is safe to close. Then start the rebuilt
candidate with `./start.sh` only.
