# Product Roadmap

> **Planning document.** These are desired outcomes and open work, not a list
> of capabilities promised by the current application. See
> [using Reason Room](using-reason-room.md) for current behavior and
> [validation status](validation.md) for checked evidence.

## Direction

Help a person understand a complex system by arranging meaningful items,
relationships, and visual context, then moving between overview and detail.
Support different organizational vocabularies without making diagram layout
silently assert ownership, responsibility, or other facts.

The current application already has local documents, saved views, typed
relationships, visual boundaries, semantic zoom, and local MCP access. Its
built-in types and relationship restrictions remain implementation constraints;
the broader product direction is not a claim that arbitrary models are already
supported. Naming across the UI and technical interfaces is still open.

## Near-Term Outcomes

- **Reliable first use:** a newcomer can install, open an example, create a
  separate document, understand saving, and find recovery guidance.
- **Understandable composition:** people can create, name, connect, move, and
  resize cards while distinguishing visual nesting from meaningful relations.
- **Understandable detail:** people can author and revise zoom layers without
  losing orientation or important context.
- **Clear agent collaboration:** an agent can inspect the intended document,
  make bounded requested changes, and report verified results without implying
  control of a window or a shared browser session.
- **Public release evidence:** validate the final hosted site, browser file
  actions, storage failure handling, and the clean-install experience before
  presenting them as release guarantees.

## Questions Still Open

- Which names should become canonical across the app, domain, and APIs?
- Which built-in classifications and relationship rules help users, and which
  need to become configurable?
- Which context must remain visible as people change scale or perspective?
- How should competing human and agent edits be coordinated beyond the current
  per-save checks and deferred refresh?

## How the Plan Changes

Use representative examples and observed user behavior to revise these goals.
Keep current capabilities in the guides, proposed behavior in product-planning
pages, and non-obvious durable decisions in ADRs. Do not turn a successful build
or a prototype observation into a claim of validated usability.

Related planning: [vision](product/vision.md),
[interaction direction](product/core-interaction-model.md),
[lenses and semantic zoom](product/lenses-and-semantic-zoom.md), and
[discovery plan](product/discovery-plan.md).
Past research and implementation chronology are in the [archive](history.md).
