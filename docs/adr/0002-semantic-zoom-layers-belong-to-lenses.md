# Semantic zoom layers belong to lenses

Semantic zoom layers are ordered presentation rules saved inside one Lens. A layer explicitly assigns the base level for existing concepts and relationships, with optional fade-in and fade-out scale ranges and a card scale. Concepts and relationships remain single workspace records. Once a card's base layer is active, it remains as context through every deeper active layer; the base layer supplies its card scale. This avoids disconnected per-scale diagrams while making progressive disclosure editable and locally understandable.

## Considered Options

Duplicating a Lens or its graph for each zoom level would make simple visual changes diverge into conflicting maps. Inferring layers from type or relationship kind would impose organization-specific meaning. Explicit membership costs a small authoring action but keeps each layer readable and adjustable.

## Consequences

The canvas projection must be a pure Lens-and-scale query. Each card and relationship has no more than one base layer in a Lens. A relationship can only render when its base layer and both endpoint cards are visible.

New cards begin in the first layer of an already layered Lens, so direct creation stays visible. New relationships begin in the first layer where both endpoint cards are present. Removing a layer reassigns its base records to the nearest remaining layer, so presentation edits do not remove graph records from the map.
