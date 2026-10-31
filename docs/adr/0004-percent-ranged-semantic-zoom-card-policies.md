# Percent-ranged semantic zoom and card presentation policies

On 2026-10-30, semantic zoom layers changed from independent directional fade ranges to a user-facing active zoom-percent range plus one symmetric fade distance. A card remains a shared workspace concept. Its Lens-local display is either automatic, which resolves exactly one size-derived layer, or manual, which stores an explicit list of visible layers. This persists the direct-manipulation meaning requested for resizing without interpreting card size as a type, relationship, hierarchy, membership, or other graph fact.

## Consequences

Automatic assignment compares the largest card dimension with the default
new-card dimensions at 100% zoom, and chooses the closest layer range. A
manual card can select one or more named layers. A relationship makes both
endpoint cards visible when either endpoint's layer is viewed. This preserves
a connected explanation across ranges while keeping the graph and Lens
presentation separate. Schema version 4 fade controls and schema version 5
dedicated or always-visible policies migrate deterministically to schema
version 6.
