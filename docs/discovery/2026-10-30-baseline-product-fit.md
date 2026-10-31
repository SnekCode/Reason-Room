# Baseline Fit Discovery

**Date:** 2026-10-30\
**Baseline inspected:** `0ebe27b` (`feat: prepare eframe candidate next stage`)

## Assessment

On 2026-10-30, the committed baseline provides a sound local, direct-manipulation graph canvas. It is a useful launch point for an organization-scale visualization product. It does not yet represent nested organizational boundaries or make one connected model understandable at progressively deeper levels of detail.

## Reusable Strengths

- On 2026-10-30, `crates/canvas` isolates camera transforms, viewport queries, hit testing, selection, movement, and generated-data behavior from rendering. Its deterministic tests make it a strong base for interaction prototypes.
- On 2026-10-30, the canvas supports pan, pointer-centered zoom, marquee selection, multi-selection, and movement previews. This directly supports the intended create, connect, move, and inspect workflow.
- On 2026-10-30, the workspace document stores entities, free-form summaries and details, relationships, independent lens placements, and planning scenarios. Local persistence provides staged mutations, undo/redo, validation, backup, and recovery boundaries without UI dependencies.
- On 2026-10-30, the eframe application keeps painter and platform UI code in `crates/app`, while canvas and session logic remain testable. This matches the project boundary required for iterative visual experiments.
- On 2026-10-30, the baseline already applies bounded overview rendering and semantic detail levels, and it has relationship-path routing tests. Those capabilities can inform dense-graph and zoom-transition experiments.

## Architectural Constraints

- On 2026-10-30, the workspace schema defines a curated set of entity and relationship variants. Custom entity types provide display metadata only, while relationship kinds remain a Rust enum. This is too restrictive for a product that must adapt to each organization's language and connections.
- On 2026-10-30, containment exists only as a relationship. The model has no first-class spatial boundary, parent placement, inherited visibility, collapse state, or boundary interaction. A card cannot yet become a visual container for other cards.
- On 2026-10-30, a lens is a filtered graph with its own manually stored placements. It does not specify how a user's view changes as they zoom, how context persists across levels, or how multiple dimensions are composed without duplicate layout work.
- On 2026-10-30, relationship display is optimized for relationships associated with a selected entity. The baseline does not prove that many simultaneous cross-boundary relationships remain readable, selectable, and explainable.
- On 2026-10-30, the eframe shell coordinates a broad set of interaction and inspector behavior in `crates/app/src/main.rs`. New visual modes need clear app-level seams before they can remain independently testable and easy to change.

## Unknowns To Validate

- On 2026-10-30, it is unknown whether people understand a nested boundary as membership, ownership, responsibility, deployment scope, or only a visual grouping. The product must not imply semantics that a user did not choose.
- On 2026-10-30, it is unknown which zoom transitions preserve orientation: scale-only changes, progressive disclosure, boundary collapse and expansion, lens changes, or a combination of these.
- On 2026-10-30, it is unknown how to show relationships that cross several boundaries without line clutter or hiding the reason for a connection.
- On 2026-10-30, it is unknown whether lightweight creation can collect enough meaning through direct manipulation and short text, without forcing users into forms or a predefined taxonomy.
- On 2026-10-30, generated-data performance does not establish usability for a real, irregular organization graph. Realistic density, label lengths, boundary depth, and interaction latency need separate evidence.

## Next Discovery Slices

1. **Boundary interaction prototype.** Test one visual boundary that can contain entities, be moved, selected, and expanded or collapsed. Record what containment means to the participant before defining persistence semantics.
2. **Progressive-detail prototype.** Use one shared sample organization graph at overview, working, and detail scales. Compare continuity, orientation, and the minimum information visible at each scale.
3. **Cross-boundary relationship prototype.** Test several visual treatments for connections that enter and leave nested boundaries. Measure whether a participant can trace a connection and explain its purpose.
4. **Low-friction authoring prototype.** Test creation, naming, short summary entry, connection, and nesting as a single canvas flow. Identify the smallest prompt needed for useful structure without designing a fixed taxonomy.
5. **Realistic-density evidence slice.** Build a deterministic representative fixture and measure rendering, hit testing, movement, and readability across varied boundary depth and relationship density.

## Direction

On 2026-10-30, the recommended next stage is prototype-led discovery, not schema expansion. Each slice should establish a user-observed interaction result and a technical constraint before the data model or production UI is extended.
