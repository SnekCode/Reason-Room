# Reason Room Documentation

Reason Room is experimental. These guides describe the current application;
product-planning pages are labeled separately. File formats and interactions
may change before a stable release.

## Start Here

1. [Install and run](installation.md): macOS tools, browser tools, and troubleshooting.
2. [Use Reason Room](using-reason-room.md): create a canvas, arrange cards, inspect relationships, and save your work.
3. [Browser build](webassembly.md): browser storage, local file access, and static hosting.
4. [Glossary](../GLOSSARY.md): domain meanings and distinctions.

## Developers and Agents

- [Built-in examples](examples.md): walkthrough contents and MCP authoring.
- [Architecture](../ARCHITECTURE.md): module boundaries, graph versus presentation, and persistence.
- [Workspace format](workspace-schema.md): current version 7 JSON, validation, and migrations.
- [MCP and agent support](product/external-agent-support.md): scoped access to local files.
- [Installable skill](../skills/reason-room/SKILL.md) and its [setup guide](../skills/reason-room/references/setup.md).
- [Validation status](validation.md): what has been checked and what remains unverified.
- [Security policy](../SECURITY.md).

## Current Decisions

- [Visual boundaries are Lens-local context](adr/0003-visual-boundaries-are-lens-local-context.md).
- [Percent-ranged semantic zoom and card policies](adr/0004-percent-ranged-semantic-zoom-card-policies.md).

The numbering preserves the original decision IDs. Earlier decisions are in
the [archive](history.md), rather than being renumbered or treated as current.

## Future Direction

These describe intended outcomes and open questions, not delivered capabilities:

- [Roadmap](roadmap.md)
- [Product vision](product/vision.md)
- [Interaction direction](product/core-interaction-model.md)
- [Lenses and semantic zoom direction](product/lenses-and-semantic-zoom.md)
- [Discovery plan](product/discovery-plan.md)

## History

Dated research, prototype records, and superseded decisions live on the
[dedicated archive branch](history.md). Follow the guides on `master` for
current setup and behavior.
