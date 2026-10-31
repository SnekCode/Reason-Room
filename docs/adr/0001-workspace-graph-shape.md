# Use scenario-based workspace states and editable dynamic lenses (Historical)

> **Historical pre-reset decision.** This ADR preserves a 2026-10-28 workspace
> graph proposal. The [2026-10-30 product reset](../wiki/product-reset-2026-10-30.md)
> supersedes its fixed taxonomy, relationship, and workspace-shape claims. It
> is not active product or persistence guidance. Use the
> [product roadmap](../roadmap.md) and [product vision](../product/vision.md)
> for discovery before making a model decision.

The 2026-10-28 proposal gave the workspace graph one Current State and named
Planning Scenarios rooted in a Workspace Version, rather than Git-like history.
It directed responsibility edges from Team to Concept with `owns` and
`operates`. It described dynamic filtered Lenses with manual layouts and
starter Lens Templates, plus curated core types extended with lightweight
Custom Types. This is retained as baseline evidence only.

## Consequences

- The proposal required the provisional `managed_by` relationship to migrate
  to `owns` or `operates` during the workspace schema redesign.
- The proposal gave a Planning Scenario its own graph and lens-layout changes
  without modifying the Current State.
- The proposal deferred custom fields and type-specific relationship validation.
