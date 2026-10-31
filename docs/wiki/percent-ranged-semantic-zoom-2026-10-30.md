# Percent-Ranged Semantic Zoom 2026-10-30

On 2026-10-30, the user replaced separate fade-in and fade-out controls with
an active zoom-percent range and one symmetric fade distance. For example, a
fade of 11 means the layer fades for 11 percentage points before its active
start and after its active end.

On the same date, the workspace model added a Lens-local card display policy.
An automatic card chooses exactly one closest layer from its size relative to a
new 240 by 120 card at 100% zoom. A manual card selects one or more named
layers. These are presentation settings, not graph facts or organization
taxonomy.

On the same date, semantic-zoom projection retained card context from deeper
layers and made both endpoints of a relationship visible when either endpoint
layer is viewed. Schema version 4 fade ranges and schema version 5 card modes
migrate deterministically to schema version 6.

Source: user-directed semantic-zoom and resize refinement on 2026-10-30; see
[ADR 0004](../adr/0004-percent-ranged-semantic-zoom-card-policies.md).
