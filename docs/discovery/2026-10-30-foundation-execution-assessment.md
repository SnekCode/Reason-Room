# Foundation Execution Assessment — 2026-10-30

**Inspected baseline:** `0ebe27b` (`feat: prepare eframe candidate next stage`)

## Decision Boundary

On 2026-10-30, the product evidence did not justify a persisted boundary,
parent, containment, or flexible-relationship schema. The roadmap requires
people to state and revise boundary meaning before such a commitment. The
existing temporary boundary study must remain separate from saved workspace
data until that evidence exists.

The smallest safe autonomous implementation slice is a **semantic-zoom
inspection contract**. It makes each saved Lens projection observable and
repeatable without adding new product concepts or controls. It gives later
visual evaluation a deterministic input and an exact expected projection.

## Proposed Slice

Add one `semantic-zoom` command to `crates/cli`. Given a workspace path,
version, Lens, and scale, it returns deterministic JSON for the existing
`WorkspaceDocument::semantic_zoom_frame` projection.

The result should contain only presentation facts already resolved by the
workspace model:

- card ID, displayed bounds, opacity, and card scale;
- relationship ID, endpoints, and opacity; and
- selected version, Lens, and requested scale.

The command must sort records by stable ID. It must reject a missing version,
Lens, or non-finite scale with a stable error. It must not write a workspace.

Add a small tracked evaluation fixture with a broad layer, a deeper layer, and
overlapping fade ranges. The fixture language must stay generic. It is a test
artifact, not a required organization taxonomy or an example product template.

## Acceptance Criteria

The slice is complete when all of the following are true:

1. Repeated command invocations against the fixture return byte-identical JSON.
2. The output at the broad, overlap, and close scales proves the stored base
   layer context rule: a broad-layer card remains visible after a deeper layer
   becomes active.
3. A relationship is output only when its saved base layer and both endpoint
   cards are visible in the resolved frame.
4. The command returns a non-success exit status for invalid input without
   changing the fixture.
5. Unit tests cover the projection contract. Existing workspace-session tests
   continue to cover staged save, undo, redo, backup, and recovery behavior.
6. No workspace schema version, persisted entity type, relationship kind,
   boundary field, or eframe interaction changes as part of this slice.

## Module Ownership

| Module | Responsibility in this slice | Must not do |
| --- | --- | --- |
| `crates/canvas` | Keep `WorkspaceDocument::semantic_zoom_frame` pure; add projection invariants only when a missing invariant is identified. | Add persisted boundaries, hierarchy, or UI code. |
| `crates/cli` | Parse the read-only command and serialize the resolved frame deterministically. | Duplicate semantic-zoom calculations or edit local files. |
| `crates/workspace-session` | Remain unchanged; it already owns staged local save and history. | Learn display-only projection behavior. |
| `crates/app` | Remain unchanged for this slice. Later visual work reads the same saved Lens projection. | Add settings fields or a control for every record. |

## Existing Evidence and Gaps

On 2026-10-30, the workspace model already stores ordered semantic-zoom
layers, fade ranges, card scale, and one base-layer assignment per concept or
relationship. `crates/canvas/src/workspace.rs` resolves those records to a
pure frame. Its tests cover fade validation, uniqueness, context continuity,
relationship endpoint visibility, direct creation, deletion reassignment, and
persistence migration.

On 2026-10-30, `crates/workspace-session/src/evaluation_actions.rs` tests the
staged semantic-zoom create, update, assignment, save, and history path. The
session and local-store modules separately test validation, backup, recovery,
and failure behavior. `crates/app/src/main.rs` holds all eframe painter and
control code, and its temporary `BoundaryStudy` remains explicitly isolated
from workspace mutation.

The missing machine-verifiable seam is a stable external view of the resolved
Lens frame. The proposed read-only command fills that gap. It does not prove
that people understand the controls, visual fades, labels, boundaries, or
crossing relationships. Those remain deferred human observations.

## Explicit Deferrals

Do not autonomously implement any of the following while manual discovery is
deferred:

- a saved boundary or nested-boundary data model;
- automatic semantic meaning from spatial containment;
- a replacement for the relationship-kind vocabulary;
- additional authoring fields or form-heavy boundary controls; or
- a roadmap completion claim for the discovery milestones.

The repaired desktop capture path makes a later observed eframe session
possible through `./start.sh`. When a participant is available, use the
[Boundary Discovery Protocol](../product/boundary-discovery-protocol.md) and
record the resulting evidence before changing the deferred boundaries.

## Recommended Order

1. Implement and verify the read-only semantic-zoom inspection contract.
2. Use its fixture and output to run reproducible visual-capture setup checks.
3. Defer subjective visual and usability results until a participant returns.
4. Revisit the next product-foundation decision only after the discovery
   protocol provides repeated evidence.
