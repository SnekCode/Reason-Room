# Visual boundaries are Lens-local context

On 2026-10-30, the product added a saved visual boundary for direct canvas
composition. A boundary has a name, optional short summary, and rectangular
bounds in one Lens. It provides visible context for cards arranged near or
within it.

## Considered Options

Making a boundary an entity or a `contains` relationship would state business
meaning when a person only arranged the map. Keeping it as a temporary study
fixture would prevent a person from building a real map. A Lens-local visual
record preserves the authored composition without turning spatial placement
into a graph fact.

## Consequences

A boundary can be created and rendered with a saved workspace. It stores no
member list. Moving a card into, out of, or across a boundary does not alter
the card, its relationships, ownership, responsibility, or hierarchy. A later
product decision may add an explicit semantic relationship when evidence calls
for it; it must not infer one from boundary geometry.
