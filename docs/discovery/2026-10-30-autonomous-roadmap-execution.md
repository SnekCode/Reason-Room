# Autonomous Roadmap Execution Boundary

**Date:** 2026-10-30\
**Scope:** product roadmap execution while participant observation is deferred

## Purpose

On 2026-10-30, the user directed continued roadmap work and deferred manual
testing. This note separates preparation that automation can verify from the
participant evidence required by the roadmap. It records no completed
discovery milestone.

## Evidence Boundary

| Roadmap milestone | Work automation can verify | Evidence that remains deferred |
| --- | --- | --- |
| Direct semantic-zoom authoring | Persisted layer rules, transition-range validation, projection continuity, and safe local save behavior | A person can define and revise a journey without guidance or a settings-heavy mental model |
| Shared understanding | Deterministic, de-identified scenario fixtures and question-to-projection consistency | People agree on useful questions and the information needed at each level |
| Canvas composition | Pointer-input routes, temporary boundary movement, connection geometry, and no unintended workspace mutation | People interpret placement, nesting, and crossing connections as intended and can compose a useful map directly |
| Lenses and semantic zoom | Deterministic broad, working, and close projections; layer opacity; card-size behavior; and context-card continuity | People retain orientation and understand the difference between a Lens and a zoom level |
| Real-world sensemaking | Bounded rendering, deterministic irregular-density fixtures, and repeatable route or hit-test measurements | People can trace responsibility, dependencies, delivery paths, and controls in unfamiliar realistic examples |
| Product foundation | Migration, validation, persistence, recovery, and implementation-boundary tests for decisions already supported by evidence | A durable model commitment is justified by repeated observed interaction evidence |

Automation must not mark the second column as a substitute for the third. The
roadmap outcomes are participant outcomes, not code-complete states.

## Observed Preparation Seams

On 2026-10-30, the uncommitted working tree contained three useful,
testable seams:

- `crates/canvas/src/boundary_study.rs` projects deterministic temporary
  scenarios and handles temporary boundary input without persistence.
- `crates/canvas/src/workspace.rs` projects saved semantic-zoom layers and
  validates one base-layer assignment for a card or relationship.
- `crates/app/src/main.rs` keeps candidate painter and input routing in the
  eframe boundary, while `crates/workspace-session` remains UI-free.

The desktop-capture repair recorded in
[Eframe Candidate](../project-memory/eframe-candidate.md) makes a later visual
session address the intended candidate window. It does not supply the deferred
participant evidence.

## Autonomous Delivery Sequence

The following sequence is permitted without a product-model decision:

1. Establish a reusable, deterministic scenario and projection test matrix for
   broad, working, and close scale. Include short and long labels, crossing
   connections, and cards retained as context.
2. Improve the eframe candidate's visual hierarchy and direct-manipulation
   affordances from the documented product principles. Keep all presentation
   changes in `crates/app`. Do not add a fixed organization taxonomy or a
   durable boundary record.
3. Add machine-verifiable tests for every changed camera transformation,
   generated canvas data path, projection rule, and pointer route. Keep
   persisted workspace tests separate from temporary-study tests.
4. Preserve a reproducible `./start.sh` candidate launch and a candidate-specific
   desktop identity. Record launch or capture evidence separately from human
   experience evidence.
5. Prepare short, neutral human-session protocols and fixture reset steps. Do
   not record an inferred participant result while testing is deferred.

## Guardrails

- Do not persist a boundary, infer its semantic meaning, or add parent-child
  behavior before repeated participant evidence passes the Boundary Discovery
  Protocol decision gates.
- Do not turn sample scenario categories into required workspace types.
- Do not interpret an automated visual layout check as readability,
  discoverability, or understanding evidence.
- Do not claim a roadmap milestone is complete until the milestone's stated
  participant outcome has been observed and recorded.

## Completion Definition While Manual Testing Is Deferred

Autonomous execution is complete when every allowed preparation slice has
deterministic coverage, a passing project verification gate, and a dated
record of its deferred human question. The product roadmap remains open until
the deferred evidence is collected.

## Sources

- [Product Roadmap](../roadmap.md) (inspected 2026-10-30)
- [Discovery Plan](../product/discovery-plan.md) (inspected 2026-10-30)
- [Boundary Discovery Protocol](../product/boundary-discovery-protocol.md)
  (inspected 2026-10-30)
- [Baseline Fit Discovery](2026-10-30-baseline-product-fit.md)
  (inspected 2026-10-30)
- [Eframe Candidate](../project-memory/eframe-candidate.md) (inspected
  2026-10-30)
