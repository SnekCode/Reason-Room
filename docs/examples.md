# Built-in Examples

The native and browser apps share the catalog in
[`crates/app/src/examples.rs`](../crates/app/src/examples.rs). **File → Examples**
opens one of two bundled documents in [`workspaces/`](../workspaces/):

- [Static React website](../workspaces/static-react-website.infinite-note.json):
  GitHub source, documentation and wiki; Actions build and publish; private S3
  origin and CloudFront HTTPS; Cloudflare DNS; visitors and recovery.
- [Car manufacturing value stream](../workspaces/car-manufacturing-vsm.infinite-note.json):
  mining, fabrication, inbound transport, receiving, body, paint, final assembly,
  finished goods, outbound transport, dealer and demand feedback.

Both have three nested spatial levels, named visual boundaries, peer and
cross-boundary relationships, and three semantic zoom ranges. Process detail
appears at 24%; work records at 85%. These are illustrative maps, not deployment
instructions or measured production data. The VSM explicitly separates takt,
cycle time, queues and transport. Its paint bottleneck is intentional.

The website has 60 cards and the VSM 90. MCP previews at 12%, 45%, and 120%
show respectively 6/24/60 and 9/36/90 visible cards. A graph edge can bring its
endpoints into view, so the examples keep peer connections at matching levels.

## Authoring through MCP

The [authoring script](../scripts/build-example-workspaces.py) invokes the real
CLI MCP server over stdio. It creates new files, discovers tools, reads context,
applies one change at a time, checks saved digests and reads back after every
mutation. It also queries relationships and previews the three levels. It does
not directly serialize or modify workspace JSON.

Build `infinite-note-canvas-cli` in release mode, then run:

```sh
python3 scripts/build-example-workspaces.py --output-directory /absolute/path/outside-checkout/examples
```

Use a new output directory: the script refuses to overwrite a document. Inspect
the results through MCP and in the app before copying the verified JSON into
`workspaces/`. Generation is explicit maintenance work, not part of startup or
CI. The two files include only core types and one Version/View because MCP does
not currently create custom types, Versions or Views.

The installable [Reason Room skill](../skills/reason-room/SKILL.md) includes
[progressive authoring guidance](../skills/reason-room/references/progressive-canvases.md)
for visual boundaries, cards as containers, cross-boundary relationships and
semantic disclosure. Two legacy format/CLI fixtures remain under `tests/fixtures`
solely for regression coverage; they are not bundled or offered as examples.

## Loading and preserving work

Desktop loads create unique files in the platform's local Canvases directory.
Existing files are never replaced. Browser loads replace the active browser-local
canvas and detach an existing watched-file connection. Export a browser canvas
with Save As before switching if you want to retain a separate copy. Neither
platform writes to the bundled source. An active dialog or unsaved card, boundary
or layer inspector edit prevents example loading.

New launches default to the website example; recent desktop files and restored
browser canvases still take priority. Example loading fits the overview below
the floating controls and opens with the inspector collapsed. User workspaces
with an example identity also get this opening fit; a renamed title does not
change that identity.
