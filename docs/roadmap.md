# Product Roadmap

## Purpose

This roadmap began a new product-planning phase from the committed baseline.
It replaces the earlier experiment-driven implementation backlog. Historical
work items and records remain as evidence. They do not define this product plan.

## Product Direction

Build a direct-manipulation canvas that helps people understand and shape the
relationships inside a large organization. A person can move from a broad view
of teams, systems, and responsibilities to the operational detail that explains
how work reaches an outcome.

The product must adapt to different organizations. It must not require one
fixed vocabulary, hierarchy, or relationship model.

See [Product Vision](product/vision.md) and [Core Interaction Model](product/core-interaction-model.md).

## Evidence Since The Reset

On 2026-10-30, a participant confirmed that a three-level semantic-zoom study
made the intended broad-to-detail journey understandable. The study established
that a Lens is a view and that semantic zoom changes the detail within that
view. The participant also specified that a card assigned to a broad base level
must remain as context when deeper detail appears.

On 2026-10-30, the implementation work added editable semantic zoom layers to
the saved workspace model. The automated checks cover layer persistence,
fade-range validation, base-level uniqueness, context continuity, relationship
endpoint visibility, direct creation, layer deletion, and undo. This is
implementation evidence, not a completed human experience result: a desktop
tool continued to attach to the separate historical application window instead
of the eframe candidate during the direct visual check.

Source: participant study sessions and semantic-zoom layer implementation
recorded on 2026-10-30. See [Semantic Zoom Layer Foundation
2026-10-30](wiki/semantic-zoom-layer-foundation-2026-10-30.md) and [Eframe
Candidate](project-memory/eframe-candidate.md).

Later that day, the user directed a visible layer cross-section, precise
resize-to-layer feedback, full current-layer labels, and always-present
relationship lines. The implementation added those authoring aids without
making a new parent-child layer hierarchy or turning visual placement into a
graph fact. Automated checks pass. Manual eframe evaluation remains deferred.
See [Semantic-Zoom UI Contract](wiki/semantic-zoom-ui-contract-2026-10-30.md).

On 2026-10-30, the user directed the application to replace the temporary
question-led study with direct authoring. The active canvas therefore adds
named visual boundaries to a Lens. A boundary provides visual context only; it
does not infer hierarchy, membership, ownership, responsibility, or a graph
relationship from placement. See [ADR 0003](adr/0003-visual-boundaries-are-lens-local-context.md).

## Product Principles

- Put understanding before administration.
- Keep common work direct: create, name, summarize, connect, move, and nest.
- Use the canvas as the primary workspace. Do not make people manage a large
  collection of forms, fields, or controls.
- Preserve context while a person changes scale or focus.
- Reveal useful detail when it helps answer a question. Do not show every record
  at once.
- Let an organization choose its own meanings. Guide without imposing a fixed
  taxonomy.

See [Experience Principles](product/vision.md#experience-principles).

## Discovery Milestones

### 1. Direct Semantic-Zoom Authoring

Ask a person unfamiliar with a representative map to define a Lens, create
base and deeper layers, assign cards and relationships, set transitions, and
then use zoom to answer a question. Observe whether the controls feel like a
way to shape an explanation, rather than a settings exercise.

**Outcome:** people can author and revise a semantic-zoom journey while keeping
orientation and without repeated guidance.

### 2. Shared Understanding

Define the real questions that a canvas must answer. Use representative
organizational situations, including teams, operational environments, delivery
paths, ownership, and controls. Identify what a person needs at each level of
detail.

**Outcome:** a small set of testable user journeys and language that does not
lock the product into a fixed entity taxonomy.

### 3. Canvas Composition

Test how people create and arrange entities, boundaries, and connections. Test
how nesting communicates containment, responsibility, or context without
silently changing meaning.

**Outcome:** a low-friction interaction pattern for create, name, summarize,
connect, move, and nest.

### 4. Lenses and Semantic Zoom

Use the saved layer model to compare views that let a person inspect the same
organization from different perspectives and levels of detail. Test which
context cards, relationships, labels, and card sizes remain useful at broad,
working, and close scale. A lens must clarify a question without duplicating or
fragmenting the underlying organization.

**Outcome:** an understandable lens model and zoom behavior that preserves
orientation while it reveals more detail.

### 5. Real-World Sensemaking

Use realistic, complex examples to test whether people can trace ownership,
dependencies, delivery paths, and shared controls. Improve readability,
navigation, and editing from observed use.

**Outcome:** evidence that the canvas helps people explain a complex system to
themselves and to others.

### 6. Product Foundation

Choose durable model, persistence, and implementation decisions only after the
interaction model and discovery evidence make them necessary.

**Outcome:** a product foundation that supports the validated experience
without narrowing future organizational use cases.

## Planning Rules

- Treat product discovery as the source for future delivery work.
- Keep milestones outcome-focused. Do not turn this roadmap into a technical
  task backlog.
- Record new product decisions in `docs/product/` before implementation starts.
- Revisit this roadmap after each discovery milestone. Replace assumptions with
  observed evidence.

## Related Documents

- [Product Vision](product/vision.md)
- [Core Interaction Model](product/core-interaction-model.md)
- [Lenses and Semantic Zoom](product/lenses-and-semantic-zoom.md)
- [Discovery Plan](product/discovery-plan.md)
- [Semantic Zoom Layer Foundation 2026-10-30](wiki/semantic-zoom-layer-foundation-2026-10-30.md)
