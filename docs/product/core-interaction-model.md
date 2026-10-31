# Core Interaction Model

> **Planning document.** This describes intended experience and open questions,
> not a guarantee of current capabilities. For current behavior, read the
> [user guide](../using-reason-room.md) and [validation status](../validation.md).

## Canvas Objects

The canvas has a small set of visual building blocks. Their meaning must stay
explicit and adjustable for each organization.

- **Entity:** a named thing a person wants to understand. It starts with a name
  and can have a short summary.
- **Boundary:** a named visual area within one Lens. It provides context for
  nearby entities but does not automatically state containment, membership,
  ownership, or responsibility.
- **Connection:** a visible link that states or proposes a relationship between
  two entities.
- **Lens:** a selected perspective that changes what the canvas emphasizes or
  reveals.

These are interaction concepts, not a fixed data taxonomy. An organization can
decide whether an entity represents a team, account, system, process, control,
or another meaningful thing.

## Primary Actions

The normal workflow must support these actions with minimal interruption:

1. Create an entity on the canvas.
2. Give it a name.
3. Add a short summary when useful.
4. Draw a connection to another entity.
5. Move an entity to improve the map.
6. Place an entity within a boundary.
7. Select a lens or change zoom to inspect a different level of detail.

The product must make these actions discoverable. It must not require a person
to open a long form before they can build a useful map.

## Meaning and Safety

Visual actions help a person compose a map. They do not automatically establish
business meaning that the person did not express.

- Moving an entity changes layout, not responsibility or ownership.
- Placing a card in or across a boundary changes visual composition only. It
  must not silently create a semantic relationship.
- Connecting entities must make the relationship understandable, either at the
  time of creation or through a clear follow-up cue.
- A person must be able to inspect and revise the meaning of a boundary or
  connection without rebuilding the map.

## Progressive Detail

The canvas presents the smallest useful amount of information first. Detail can
appear through zoom, selection, focus, or a lens. At each step, the product must
retain enough surrounding structure for a person to understand where the detail
fits.

See [Lenses and Semantic Zoom](lenses-and-semantic-zoom.md) and [Product Vision](vision.md).
