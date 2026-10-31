# Workspace JSON Format: Version 6

This is a reference for the **currently implemented** saved format, checked
against `WorkspaceDocument` and its validators in
[`crates/canvas/src/workspace.rs`](../crates/canvas/src/workspace.rs).
It describes an experimental format, not a stable compatibility promise.
Keep a separate copy before loading older or important files into a new build.

The public terminology decision remains open. This reference uses the actual
JSON field names. See [the user guide's name mapping](using-reason-room.md#understand-the-names-on-screen)
for how Canvas, Card, and View relate to workspace, concept, and lens.
MCP tool inputs are a separate contract; see the
[MCP reference](../skills/reason-room/references/mcp-contract.md).

## Complete Examples

- [Empty document](examples/empty-workspace.json): the minimal useful document
  with one version and one lens.
- [Layered document](examples/layered-workspace.json): cards, placements, a typed
  relationship, a visual boundary, percent ranges, and a manual layer override.

Both examples are validated by the CLI and the documentation check. Inspect
an example without modifying it:

```sh
cargo build --locked --release --package infinite-note-canvas-cli
./target/release/infinite-note-canvas-cli validate --workspace docs/examples/layered-workspace.json --format json
```

This CLI command validates data; it does not launch the application. Use
`./start.sh` for the desktop app. Copy examples outside the checkout before
using them as writable documents.

## Reading the Tables

“Default” describes a field that may be omitted during parsing. A field with
no default is required. “Omitted when empty” describes serialized output, not
an additional required field. Strings identified as nonblank must contain
non-whitespace characters.

### Document

| Field | Type | Default / constraint |
| --- | --- | --- |
| `schema_version` | Unsigned integer | Required; current output is `6`. Supported input versions are described below. |
| `workspace_id` | String | Required, nonblank. |
| `title` | String | Required, nonblank. |
| `custom_types` | Array of custom types | Default `[]`; serialized even when empty. |
| `current_state` | Version | Required; the document's permanent root version. |
| `planning_scenarios` | Array of scenarios | Default `[]`; serialized even when empty. |

### Version and Planning Scenario

A version requires `id`, `title`, `concepts`, `relationships`, and `lenses`.
`id` and `title` are nonblank strings. The three collection fields are arrays;
`lenses` must contain at least one lens. A new empty document can have no
concepts or relationships.

A scenario contains **the version fields at the same level** as its required
`based_on_version_id` string. There is no nested `version` field:

```json
{
  "based_on_version_id": "current",
  "id": "alternative",
  "title": "Alternative",
  "concepts": [],
  "relationships": [],
  "lenses": [{"id":"canvas","title":"Canvas","placements":[]}]
}
```

Version IDs must be unique across the document. Each scenario's base must
exist, and the chain of scenario bases must not contain a cycle. The schema
stores complete versions; it is not a diff, a Git branch, or a commit history.

### Concept

| Field | Type | Default / constraint |
| --- | --- | --- |
| `id` | Unsigned 32-bit integer | Required; unique within its version. |
| `type_id` | String | Required; one built-in or declared custom type. |
| `title` | String | Required, nonblank. |
| `summary` | String | Required; can be empty. |
| `details` | String | Default `""`. |
| `execution_mode` | String enum | Optional; omitted when absent. Only valid for type `process`. |
| `attributes` | Object of string values | Default `{}`; serialized even when empty. |

The eight built-in type IDs are `organization`, `team`, `project`, `process`,
`repository`, `service`, `environment`, and `access`. Valid execution modes
are `manual`, `automated`, and `hybrid`.

A custom type requires nonblank `id` and `title`, plus a `color` in `#RRGGBB`
form. `icon` is optional, must be nonblank when present, and is omitted when
absent. Custom IDs are unique across the document and cannot reuse built-in
IDs. A custom type adds a classification; it does not define an arbitrary
field schema or new relationship kinds.

### Relationship

| Field | Type | Default / constraint |
| --- | --- | --- |
| `id` | String | Required, nonblank; unique within its version. |
| `kind` | String enum | Required; use the exact JSON spellings below. |
| `source_id` | Unsigned 32-bit integer | Required; an existing concept in this version. |
| `target_id` | Unsigned 32-bit integer | Required; a different existing concept in this version. |
| `label` | String | Default `""`. |

| JSON `kind` | Endpoint restriction |
| --- | --- |
| `contains` | Any supported concept types. |
| `owns` | Source must have type `team`. |
| `operates` | Source must have type `team`. |
| `depends_on` | Any supported concept types. |
| `implemented_in` | Source `process`, target `repository`. |
| `delivers` | Any supported concept types. |
| `deployed_to` | Any supported concept types. |

Relationships are directed source → target. Self-links and duplicate
(kind, source, target) combinations are rejected. JSON enum values use
underscores even where the UI uses spaces or generated identifiers use
hyphens. Validation does not impose a general tree, a single containment
parent, or an acyclic `contains` graph.

### Lens

| Field | Type | Default / constraint |
| --- | --- | --- |
| `id`, `title` | Strings | Required, nonblank; lens ID unique within its version. |
| `template` | String enum | Optional; omitted when absent. Persisted template metadata. |
| `filter` | Object | Default empty filter. |
| `placements` | Array of placements | Required; can be empty. |
| `visual_boundaries` | Array of boundaries | Default `[]`; omitted when empty. |
| `semantic_zoom_layers` | Ordered array of layers | Default `[]`; omitted when empty. |
| `semantic_zoom_card_modes` | Array of card overrides | Default `[]`; omitted when empty. |

Template values are `ownership`, `processes_and_dependencies`,
`services_and_access`, and `deployment`. A template label is not a separate
stored graph or a guarantee of dynamic query behavior.

A filter has `concept_type_ids` (known type IDs) and `relationship_kinds`
(the relationship enums above). Both default to `[]`, are omitted when empty,
and use an empty list to mean no restriction for that category.

A placement requires `concept_id` and `bounds`. Its concept must exist in the
same version. A concept may have at most one placement in each lens, but can
have different positions in different lenses or no placement in a lens.

Bounds require all of these numbers in canvas units:

```json
{"origin":{"x":0.0,"y":0.0},"size":{"x":240.0,"y":120.0}}
```

Origins and sizes must be finite; both size components must be positive.
A boundary requires nonblank `id` and `title` and valid `bounds`; `summary`
defaults to `""`. Boundary IDs are unique within the lens. A boundary stores
no member IDs and never implies a typed relationship.

### Semantic Zoom Layer

| Field | Type | Default / constraint |
| --- | --- | --- |
| `id`, `title` | Strings | Required, nonblank; ID unique within the lens. |
| `card_scale` | Number | Required, finite, greater than zero. |
| `percent_settings` | Percent settings object | Optional in input; normalized during parsing. Present in normalized v6 output. |
| `concept_ids` | Array of concept IDs | Default `[]`; omitted when empty. Authored base membership. |
| `relationship_ids` | Array of relationship IDs | Default `[]`; omitted when empty. Authored base membership. |
| `fade_in`, `fade_out` | Legacy range objects | Optional input compatibility fields; cleared during parsing. |

Percent settings have this shape:

```json
{"active_zoom_range":{"start_percent":25.0,"end_percent":100.0},"fade_percent":10.0}
```

The active range is inclusive and must satisfy
`1 <= start_percent <= end_percent <= 409600`. Fade is finite and nonnegative;
it applies symmetrically on both sides. These values are percentages, whereas
MCP preview scale `1.0` means 100%.

Authored membership references must exist in the version. A concept or
relationship cannot be listed more than once within or across the lens's base
membership lists. A concept need not have authored base membership. Manual
card visibility can select several layers without duplicating a concept or
its base membership. See [ADR 0004](adr/0004-percent-ranged-semantic-zoom-card-policies.md)
for the separation of graph meaning and presentation.

### Card Layer Override

Omitting an override for a placed concept means **Automatic**. An explicit
`"automatic"` entry is rejected as an unnecessary default. A current manual
override has the following persisted shape:

```json
{"concept_id":1,"mode":{"manual":{"layer_ids":["overview","detail"]}}}
```

Each override must refer to an existing, placed concept and be unique for that
concept in the lens. Its layer list is nonempty, contains existing layer IDs,
and contains no duplicates. The wire shape differs from MCP's
`set_card_layer_mode` input; do not interchange them.

## Loading, Migration, and Saving

`parse_json` reads the version, performs supported migration and presentation
normalization, then validates the result. Unsupported schema versions and
invalid documents are rejected. `to_pretty_json` validates and serializes; it
does not itself migrate an arbitrary in-memory legacy document.

| Input | Current loader behavior |
| --- | --- |
| Version 1 | Uses the legacy model conversion, including views → lenses, concept kinds → type IDs, and responsibility-edge normalization. |
| Versions 2–5 | Deserializes the compatible document shape, raises its version to 6, and normalizes presentation. |
| Version 6 | Deserializes, normalizes presentation, and validates. |
| Other versions | Rejected. |

For all accepted inputs, missing percent settings are derived from legacy fade
ranges; absent ranges default to 1–409600% with no fade. Legacy `dedicated`
card modes become single-layer manual selections, and `always_visible` becomes
manual selection of the lens's configured layers. Legacy directional fade
fields are cleared. Their legacy range shape is `{"start":0.25,"end":1.0}`,
using scale ratios rather than percentages. Migration succeeds only if the
normalized result is valid.

Loading migrates in memory; the app's later save/export writes the current
representation. Preserve the original file separately if you need its old
representation. Arbitrary unknown object fields are not guaranteed to survive
loading and reserialization.

## State Outside This Format

Camera state, saved queries, undo/redo history, desktop preferences, and MCP
connection configuration are not fields in the version 6 workspace document.
A browser stores a serialized workspace in its origin's local storage; that
storage location does not add new fields to this format.
