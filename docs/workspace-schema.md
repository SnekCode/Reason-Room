# Workspace Schema (Pre-Reset Technical Baseline)

> **Historical pre-reset evidence.** This document records a prior workspace
> schema and migration proposal. The [2026-10-30 product reset](wiki/product-reset-2026-10-30.md)
> supersedes its fixed entity taxonomy, relationship vocabulary, and
> persistence assumptions. It is not an active schema contract. Use the
> [product roadmap](roadmap.md) and [product vision](product/vision.md) for
> discovery before making a model decision. For manual application evaluation,
> use `./start.sh` only.

The record below described a local JSON workspace file with the
`.infinite-note.json` extension and a Rust validator for semantic validation.

## Historical Version 2 Proposal

The proposal used `schema_version: 2`. It gave a workspace a Current State and
zero or more Planning Scenarios. Each state stored a complete graph and its own
Lens Layouts.

```text
WorkspaceDocument
├── workspace_id
├── title
├── custom_types
├── current_state
└── planning_scenarios
```

- A `WorkspaceVersion` has an ID, title, concepts, typed relationships, and Lenses.
- A Planning Scenario has a `based_on_version_id` and an independent version snapshot.
- A Lens has an optional template, dynamic filters, and manually maintained placements.
- Moving a concept changes only its placement in the active Lens of the active version.

## Historical Concept-Type Proposal

Curated core `type_id` values are:

```text
organization, team, project, process, repository, service, environment, access
```

Custom Types are local definitions with an ID, title, `#RRGGBB` color, and optional icon. A custom type does not create custom fields or relationship constraints in version one.

An Automated Process is a `process` with `execution_mode: "automated"`. Do not use an Automation concept type. A Pipeline is modeled as an automated Process.

## Historical Relationship-Type Proposal

```text
contains, owns, operates, depends_on, implemented_in, delivers, deployed_to
```

- `contains` is a structural relationship. It is not a responsibility relationship.
- `owns` and `operates` require the Team-to-Concept direction.
- `depends_on` runs from the dependent concept to what it requires.
- `implemented_in` runs from a Process to its Repository.
- A direct dependency on a Team is valid only when the dependency is on participation, approval, expertise, or capacity.

## Historical Lens Proposal

Lens filters can list concept type IDs and relationship kinds. An empty list means all values. A placement can remain stored even if the current filter hides its concept. This keeps manual layout work when the filter changes.

Starter templates are `ownership`, `processes_and_dependencies`, `services_and_access`, and `deployment`. Templates are editable metadata, not fixed layers.

## Historical Version 1 Migration Proposal

The app and CLI read schema version 1 and migrate it in memory to version 2. The next successful local save writes schema version 2 and creates a `.bak` copy of the prior file.

- Old `operated_by` becomes reversed `operates`.
- Old `managed_by` becomes reversed `operates`.
- Old `owns` from a non-Team source becomes `contains`.
- Old views become Lenses with empty filters.
- Concept IDs, titles, details, attributes, relationship IDs, and placement bounds are preserved.

## Historical Local CLI Contract

The proposal described a local, read-only-by-default CLI whose successful
commands emitted one JSON document on standard output. The following invocations
are historical command examples, not current user launch instructions.

```sh
cargo run -p infinite-note-canvas-cli -- inspect --workspace PATH --format json
cargo run -p infinite-note-canvas-cli -- validate --workspace PATH --format json
cargo run -p infinite-note-canvas-cli -- query --workspace PATH --concept ID --direction both --depth 1 --format json
```

`query` accepts `--version ID`, `--direction in|out|both`, `--kind KIND`, and `--depth N`. `validate` exits with status `2` for invalid workspace data.

## Historical Safe Local Save Record

The macOS app writes a `<workspace>.bak` backup before replacing the workspace file. It checks whether the workspace changed outside the app before a save. If a move or link cannot be saved, the app restores the unsaved in-memory change and shows `SAVE FAILED` in the window title.
