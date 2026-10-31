# Reason Room MCP Contract

This reference describes agent contract version 1 from the Reason Room CLI.
Prefer the connected server's discovery and returned values if a later version
differs. Transport is newline-delimited JSON-RPC over stdin/stdout; a regular
MCP client handles that transport for you. Protocol version: `2025-06-18`.

## Tools

| Tool | Arguments | Result and scope |
| --- | --- | --- |
| `create_workspace` | Required `title`; optional `workspace_id` | Creates only the missing configured file. Write mode only. Returns workspace/version/Lens IDs and digest. Parent folder must already exist. |
| `workspace_overview` | `{}` | Path, digest, workspace identity, schema version, `current_version_id`, `custom_types`, versions, Lens summaries, `write_enabled`. |
| `read_lens` | Optional `version_id`, `lens_id`, `offset` (default 0), `limit` (default 100, maximum 250) | `cards`, incident `relationships`, all Lens boundaries/layers, `total_card_count`, `next_offset`. Cards are sorted by concept ID. |
| `query_workspace` | Required integer `concept_id`; optional `version_id`, `direction` (`in`, `out`, `both`; default `both`), `relationship_kind`, integer `depth` (minimum/default 1) | Version-wide typed graph traversal; returns the root plus reached concepts and traversed relationships. Has no Lens argument. |
| `preview_semantic_zoom` | Required positive `scale`; optional `version_id`, `lens_id` | Visible cards/relationships/layers and opacity at that scale. Read-only. |
| `apply_workspace_change` | Required `change`; optional `version_id`, `lens_id` | One validated and persisted edit, with `change_kind`, digest, version/Lens IDs, `selected_concept_ids`, and concept count. Write mode only. |

Version defaults to the current state; Lens defaults to that version's first
Lens. Use IDs from the overview instead of assuming `current` or `canvas` for
an existing document. Existing read-only workspaces expose four read tools;
existing write-enabled workspaces add `apply_workspace_change`. A missing
write-enabled workspace exposes only `create_workspace` until created.

Successful tools return the JSON object in `structuredContent` and as JSON
text in `content`. Tool failures set `isError: true` and return error text;
protocol failures may instead return a JSON-RPC `error`. Neither is success.

## Common Values

- Built-in card type IDs are `organization`, `team`, `project`, `process`,
  `repository`, `service`, `environment`, and `access`. The overview lists
  additional custom types in `custom_types`. Reuse the user's chosen vocabulary.
- Card/concept IDs are unsigned integers. Relationship, boundary, layer,
  version, and Lens IDs are strings. Do not invent IDs for new entities.
- A point is `{"x": 0.0, "y": 0.0}` in canvas units.
- Bounds are `{"origin":{"x":0.0,"y":0.0},"size":{"x":240.0,"y":120.0}}`.
  Use valid positive sizes; the server validates domain constraints.
- Relationship kinds are `contains`, `owns`, `operates`, `depends_on`,
  `implemented_in`, `delivers`, and `deployed_to`. Direction is source → target.
  `owns` and `operates` require a `team` source; `implemented_in` requires a
  `process` source and a `repository` target. The other kinds accept any types.
  Self-links and duplicate edges with the same kind/source/target are rejected.
- A zoom preview uses a ratio (`0.5` = 50%). Layer `start_percent`,
  `end_percent`, and `fade_percent` use percentages (`50` = 50%).
- Writes share the app's staged actions and save/backup/verification path.
  A `.bak` is not a multi-operation history or a substitute for a separate copy.

## All 18 Change Kinds

Put the fields below inside `change`, alongside `kind`. Fields are required
unless marked optional. The discovery schema leaves the nested object open;
use this table rather than guessing a patch format.

| `kind` | Fields |
| --- | --- |
| `create_card` | `type_id`, `title`, `bounds`; optional `summary` (default empty) |
| `update_card` | `concept_id`; optional `type_id`, `title`, `summary`, `details`, `attributes` (string-to-string map) |
| `delete_card` | `concept_id` |
| `move_cards` | `concept_ids` (integer array), `delta` (point) |
| `resize_card` | `concept_id`, `bounds` |
| `create_relationship` | `source_id`, `target_id`, `relationship_kind` |
| `change_relationship_kind` | `relationship_id`, `relationship_kind` |
| `remove_relationship` | `relationship_id` |
| `create_boundary` | `title`, `bounds`; optional `summary` (default empty) |
| `update_boundary` | `boundary_id`, `title`, `summary` |
| `move_boundary` | `boundary_id`, `delta` |
| `resize_boundary` | `boundary_id`, `bounds` |
| `remove_boundary` | `boundary_id` |
| `create_zoom_layer` | `title`, `start_percent`, `end_percent`, `fade_percent` |
| `update_zoom_layer` | `layer_id`, `title`, `start_percent`, `end_percent`, `fade_percent`, positive `card_scale` |
| `reorder_zoom_layer` | `layer_id`, zero-based `destination_index` |
| `delete_zoom_layer` | `layer_id` |
| `set_card_layer_mode` | `concept_id`, `mode`: `{"mode":"automatic"}` or `{"mode":"manual","layer_ids":["existing-layer-id"]}` |

`update_card` retains omitted fields. Supplying `attributes` replaces the
complete map; it is not a key-level patch. Other update operations above are
full-field updates. The first `create_zoom_layer` establishes the base layer;
later calls create additional layers. Inspect returned Lens data for new IDs.

Read results serialize `layer_mode` as `"automatic"` or
`{"manual":{"layer_ids":["id"]}}`. The write tool instead requires the
`mode` objects shown in the table. Translate the read value; do not pass it
back unchanged.

## Examples

These are tool argument objects, not raw JSON-RPC envelopes. IDs and titles
are illustrative: substitute values from the actual selected workspace.

Read one page:

```json
{"version_id":"current","lens_id":"canvas","offset":0,"limit":100}
```

Trace outgoing dependencies from a card with `query_workspace`:

```json
{"version_id":"current","concept_id":7,"direction":"out","relationship_kind":"depends_on","depth":2}
```

Create a card with `apply_workspace_change`:

```json
{
  "version_id": "current",
  "lens_id": "canvas",
  "change": {
    "kind": "create_card",
    "type_id": "service",
    "title": "Billing API",
    "summary": "Handles billing requests.",
    "bounds": {"origin":{"x":24.0,"y":48.0},"size":{"x":240.0,"y":120.0}}
  }
}
```

Read back the card using the returned `selected_concept_ids`, then update only
its summary:

```json
{"version_id":"current","lens_id":"canvas","change":{"kind":"update_card","concept_id":7,"summary":"Handles billing and invoicing requests."}}
```

Preview 25% zoom with `preview_semantic_zoom`:

```json
{"version_id":"current","lens_id":"canvas","scale":0.25}
```

## Limits That Change Agent Decisions

There is no batch transaction, expected-digest argument, MCP undo/redo, general
JSON patch, whole-document replacement, UI/camera control, automatic layout,
workspace switching, or creation of versions, Lenses, and custom types.
Reconfiguring the server is required to select a different workspace file.
Each call reloads the file, but that does not coordinate concurrent writers.
Do not substitute direct file edits to bypass an absent tool or permission.
