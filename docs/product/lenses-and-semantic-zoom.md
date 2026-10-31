# Lenses and Semantic Zoom

## Direction

A single organizational map can answer many questions. A lens gives a person a
purposeful view of that map. It can make a perspective easier to read without
forcing the person to create a separate, disconnected diagram.

Semantic zoom changes the level of detail as a person moves closer to or farther
from an area. It should reveal meaningful structure, not only make the same
objects larger or smaller.

## Example Journey

At a broad scale, a person might see operational environments and the groups
that interact with them. When they approach one environment, they might see the
delivery paths and groups involved. At closer detail, they might inspect a
specific path, its source, its stages, and shared controls that participate in
it.

This is an example, not a required organizational model. The same interaction
must adapt to other kinds of entities and relationships.

## Lens Responsibilities

A lens may change one or more of these aspects of the canvas:

- what is visible;
- what is grouped or represented as a boundary;
- what receives visual emphasis;
- what detail appears at a given zoom level; and
- which relationship meanings are easiest to inspect.

A lens must make its effect understandable. It must preserve orientation by
showing enough context to relate the selected view to the wider map.

## Design Constraints

- A lens must answer a human question, not expose an implementation feature.
- Changing a lens must not make people fear that their map or facts changed.
- Zooming must not hide important context without a clear way to recover it.
- The view must avoid clutter at broad scale and unexplained abstraction at
  close scale.
- A person must be able to return to a familiar broad view quickly.

## Discovery Questions

- What lens names and controls feel understandable without training?
- When should detail appear because of zoom, and when should a lens decide it?
- Which context must remain visible during focused inspection?
- How should shared responsibility appear when several groups participate in
  one path or control?
- When do people need a different perspective versus a different level of
  detail?

See [Product Vision](vision.md), [Core Interaction Model](core-interaction-model.md), and [Discovery Plan](discovery-plan.md).
