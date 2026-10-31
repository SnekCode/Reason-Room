# Boundary Discovery Protocol

Use this protocol for a small, observed composition-prototype session. It
supports Phase 2 of the [Discovery Plan](discovery-plan.md). A visual boundary
can be saved as Lens-local context. This protocol does not define a fixed
organization taxonomy, semantic containment, or a final canvas interface.

## Objective

Learn how a person interprets and revises one nested visual boundary while
they compose an organizational map. Test whether the person can distinguish
visual placement from a stated business fact, retain context after changing
the boundary view, and explain a connection that crosses it.

The session investigates the open questions in the [Product Vision](vision.md)
and follows the meaning-and-safety constraints in the [Core Interaction
Model](core-interaction-model.md).

## Before The Session

Use a representative question and situation collected in Phase 1. The
participant must want to answer the question. Remove sensitive names when
needed. The example should include all of the following:

- Six to twelve named entities with short, understandable descriptions.
- One boundary with entities both inside and outside it.
- One nested boundary or an alternative boundary treatment to compare.
- At least one connection that enters or leaves a boundary.
- One practical question, such as who shares responsibility for a delivery
  path or what sits in an operational environment.

Use the participant's organization language. Do not pre-label a boundary as
membership, ownership, responsibility, or containment unless the participant
has chosen that meaning. Prepare one observer to record actions and words. The
observer must not explain the intended meaning while the participant works.

## Participant Tasks

Give one task at a time. Do not prescribe a specific gesture, control, or
visual treatment.

1. Ask the participant to orient themselves in the example and answer the
   practical question.
2. Ask the participant to create or revise a boundary around part of the
   example.
3. Ask the participant to place and then move an entity in relation to that
   boundary.
4. Ask the participant to select, open, close, expand, or collapse the
   boundary, where the prototype supports those actions.
5. Ask the participant to trace the connection that crosses the boundary and
   explain why it matters.
6. Ask the participant to revise the boundary or connection if its visual
   treatment no longer communicates their intended meaning.
7. Ask the participant to recover the broad context after inspecting the
   boundary closely.

## Neutral Prompts

Use short, open questions. Do not supply an interpretation in the question.

- "What does this area mean to you?"
- "What changed when you moved this item?"
- "What would you expect this boundary to change?"
- "How would you describe this connection?"
- "What tells you what is inside, outside, or related?"
- "What would you change so another person reads this the same way?"
- "Can you find your way back to the wider context?"

If the participant asks what a visual treatment means, record the question.
Give only the minimum clarification needed to continue the task. Record that
clarification as an intervention.

## Evidence To Record

Create a dated session record in `docs/discovery/`. Record observations before
interpretations. Do not record personal data that is not needed for the product
question.

```md
# Boundary Discovery Session — YYYY-MM-DD — short-example-name

## Scope

- Participant context: [role or experience, without identifying data]
- Example: [de-identified situation]
- Question to answer: [question]
- Prototype version: [commit or local revision]
- Observer: [role]

## Task Observations

| Task | What the participant did or said | Hesitation or intervention | Result |
| --- | --- | --- | --- |
| Orient in the example |  |  |  |
| Create or revise a boundary |  |  |  |
| Place or move an entity |  |  |  |
| Change boundary view |  |  |  |
| Trace a crossing connection |  |  |  |
| Recover broad context |  |  |  |

## Meaning Check

- Boundary meaning stated by participant:
- Difference between layout and business meaning:
- Meaning of the crossing connection:
- Terms the participant used:
- Terms or visual cues that caused uncertainty:

## Evidence Summary

- What the participant completed without guidance:
- What required explanation or correction:
- Assumptions changed by this session:
- Follow-up prototype question:
- Decision status: no decision / candidate pattern / repeated evidence needed
```

## Decision Gates

Do not make a schema, persistence, or final-interface decision from one
session. Advance a boundary treatment only when evidence from representative
sessions shows all of the following:

- Participants can state what the boundary means in their own words.
- Participants can tell a layout change from a semantic change.
- Participants can explain a crossing connection without repeated guidance.
- Participants can recover useful wider context after focused inspection.
- The treatment supports direct revision without a long form or a fixed
  vocabulary.

When evidence does not meet a gate, record the uncertainty and test a changed
visual treatment. When a pattern repeats across sessions, record the dated
product decision and its evidence in the product and wiki records before
implementing a durable model.

## Related Material

- [Discovery Plan](discovery-plan.md)
- [Product Vision](vision.md)
- [Core Interaction Model](core-interaction-model.md)
- [Baseline Fit Discovery](../discovery/2026-10-30-baseline-product-fit.md)
