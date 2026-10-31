# Operational Atlas Scale Feedback — 2026-10-30

Related: [Product Roadmap](../roadmap.md) · [Proposed Visual Direction](../product/visual-direction.md) · [Semantic Zoom Layer Foundation](semantic-zoom-layer-foundation-2026-10-30.md) · [Eframe Candidate](../project-memory/eframe-candidate.md)

## 2026-10-30 — Reversible Eframe Presentation Slice

On 2026-10-30, implementation work in the uncommitted working tree applied a
reversible Operational Atlas presentation slice to the eframe candidate. The
slice centralized canvas chrome tokens for ground, panel, text, quiet context,
route, control, and recovery roles. It kept workspace-authored colors separate
from those roles.

The same slice added a canvas-local semantic-scale rail. The rail maps existing
camera thresholds to Broad, Working, and Detail labels. It does not write a
workspace record, change a Lens, alter a camera, or add an interactive zoom
control. Its bounds stay fixed while its active-band emphasis changes, so it
uses no positional transition that needs an alternate reduced-motion path.

The top bar also presents the selected Lens in plain language as a View. That
change follows the participant's 2026-10-30 distinction between a Lens (the
perspective) and semantic zoom (the detail inside it). It does not rename or
migrate the saved Lens model.

Source: `crates/app/src/main.rs` and [Proposed Visual Direction](../product/visual-direction.md) inspected on 2026-10-30. This entry records implementation evidence only; it does not record participant readability or usability evidence.

## 2026-10-30 — Saved-Layer Alignment Correction

Later integration review found that a fixed camera-threshold rail could label a
detail state differently from a Lens with user-authored fade ranges. The eframe
candidate therefore began reading resolved layer opacities from the pure
semantic-zoom frame. When a Lens has saved layers, the rail selects the
strongest visible layer and uses its ordered position for Broad, Working, or
Detail. At an equal-opacity transition, it selects the deeper layer. A Lens
with no saved layers continues to say `Camera scale`, because no saved detail
state exists. A saved Lens with no active layer states that condition instead
of substituting a camera label.

The correction adds no workspace field, taxonomy, boundary, or interaction.
The rail keeps its fixed geometry. Deterministic tests cover a custom fade
transition, an empty-layer transition, and the pure projection's resolved
layer opacity. This supersedes only the earlier claim that the rail always
maps camera thresholds.

Source: `crates/canvas/src/workspace.rs` and `crates/app/src/main.rs` inspected
on 2026-10-30. This remains implementation evidence only.

## 2026-10-30 — Deferred Validation Question

When a participant is available, observe whether the non-interactive scale
rail makes a zoom transition understandable without competing with the canvas,
and whether the View label separates the perspective from the detail level.
Record that observation before treating this presentation slice as a product
decision.

Source: [Autonomous Roadmap Execution Boundary](../discovery/2026-10-30-autonomous-roadmap-execution.md) (2026-10-30).
