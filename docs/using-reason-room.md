# Use Reason Room

Start with the [installation guide](installation.md), then launch the desktop
app with `./start.sh` or the browser app with `./start.sh --web`.

## Understand the Names on Screen

The application and older technical interfaces use different names for related
things. This guide follows control labels so you can find them; a final naming
convention has not been chosen.

| On screen | In the file format and MCP | Meaning |
| --- | --- | --- |
| Canvas | Workspace document | The document containing your mapped information and saved perspectives. |
| Card | Concept plus its placement | A visible representation of an item. Its content belongs to a version; its position belongs to a particular view. |
| View | Lens | A saved perspective with its own placements, filtering, boundaries, and zoom presentation. |
| Relationship | Typed relationship | A directed, meaningful connection between two items. |
| Zoom layer | Semantic zoom layer | A presentation rule controlling visibility and appearance at different zoom levels. |

Product-planning documents also use Entity and Connection. Those are design
language; they do not introduce additional saved object types. See the
[glossary](../GLOSSARY.md) for the domain distinctions.

## Explore the Examples

Choose **File → Examples → Static React website** or **Car manufacturing value
stream** in either app. The macOS application File menu offers the same choices.
Each click opens a fresh editable copy: desktop saves it in the local Canvases
directory without overwriting an existing file; the browser saves it in its
local workspace storage. Save or discard an active edit before switching.
Use Save As to keep a separate browser copy before replacing it with an example.

A new installation starts with the website example, fitted to an overview.
Existing desktop recent-file and browser restore choices are preserved.
The examples use one current Version and View, with three semantic zoom levels:

| Example | Overview | Process | Working detail |
| --- | --- | --- | --- |
| Static React website | Repository, Actions, AWS, Cloudflare, visitors, recovery | Source/docs/wiki, CI stages, DNS and hosting | Files, credential boundaries, caching, requests and rollback |
| Car manufacturing value stream | Mine through dealer and customer demand | Fabrication, transport, queues, factory operations and production control | Individual operations, timing assumptions, inventory and quality gates |

Zoom toward a large card: process steps appear at **24%**, individual work
records at **85%**. Parent cards remain as spatial context. Named boundaries
show account/company context, and directed relationships cross those boundaries.
Select a card to read its notes. The VSM is a fictional training example, with
explicit demand, takt, inventory and cycle-time assumptions; it is not measured
factory data. The website describes an illustrative cloud architecture and
does not provision accounts, DNS or hosting.

See [example authoring and maintenance](examples.md) for the MCP workflow.

- Use the wheel to zoom around the pointer. Shift plus wheel pans; a middle
  button drag also pans.
- Select a card, boundary, or relationship to open its inspector.
- Click a relationship line to hide the other lines temporarily and show
  two arrow buttons centered on the click point and aligned with the line’s
  direction marker, one toward each endpoint. Hover a button to see its
  endpoint name; click it to center that endpoint without changing zoom.
  Rolled-up lines navigate to their visible parent endpoints. The buttons close
  after navigation; click the isolated line again to reopen them. While selected,
  the line, label and direction marker stay fully visible outside their zoom
  layer, retaining the selected endpoints without revealing hidden cards. Click empty
  canvas, select another item, or press Escape to restore the other lines.
- Choose **Preferences** in the toolbar to change display and creation defaults.
  A fixed, centered dialog opens above all panels on desktop and in the browser.
  Canvas and filter interaction is blocked until you choose Done or press Escape.
  Controls use the same dark palette as the canvas, including on systems in light mode.
- Drag cards to arrange them. Use their resize handles to change their size.
- Use the zoom controls to move between scales. The **View all** switch in
  the canvas filter area shows every card and relationship; turn it off to
  return to layer filtering. It is stacked with a matching **Roll up
  relationships** switch, which defaults to On. On uses the nearest visible
  parent for relationships whose child endpoint is hidden; Off disables that
  fallback. Neither state changes card visibility. Visible children retain
  their own relationship endpoints and layer fade.
- The top-right **Built** timestamp identifies when the app was compiled, in UTC.
  It stays fixed when you reload a hosted page or reopen a local build.
- **Undo** and **Redo** apply to workspace edits in the current session.
  They are not a persistent version history and are not MCP tools.

Existing example positions and selected cards may reflect earlier use of the
evaluation copy. Creating your own canvas gives you a separate document.

## Create Your Own Canvas

1. Choose **File → New canvas...**.
2. Enter a name. On desktop, check the proposed save location or choose another
   writable location outside the source checkout.
3. Choose **Create canvas**. Desktop creation saves a new file before switching
   to it and does not replace an existing document.
4. Choose **Create → Card**, enter a name and optional short summary, and select
   a type if useful. Choose **Create card**.
5. Select the new card to edit its details; use the inspector's **Save** action
   to commit those edits.

Types currently include organization, team, project, process, repository,
service, environment, and access, plus custom types already saved in a
workspace. The product's goal of flexible organizational vocabulary is broader
than the current set of implemented relationship rules.

## Copy Cards Between Views

Select a card (or several cards), then use **Edit → Copy cards** or the normal
copy shortcut. Paste with **Edit → Paste cards**, the paste shortcut, or the
canvas context menu. A focused text field keeps ordinary text copy and paste,
even while a card remains selected.

Pasting into another View or Version opens a popup at the pointer. Choose
**Copy** for independent card content or **Reference** to share content edits
with the original, including across Versions. Position, layer policy and
relationships stay local to the destination. References use current content;
independent copies use the content captured when copied. A View can contain
one placement of a given reference; choose Copy if it is already present.
Pasting within the source View creates independent copies. Relationships are
not copied.

Paste first centers the card or group on the canvas pointer. If it collides,
it looks for the nearest clear position within half the group's width and
height of the pointer, using an elliptical distance limit. Hidden cards still
count as obstacles. If no nearby placement fits, a ghost follows the pointer
with **Click to paste**. Click to confirm any chosen location, including an
overlap, or press Escape to cancel without saving. Groups keep their relative
arrangement. With the pointer outside the canvas, paste uses its last canvas
position or the view center.

Deleting a card removes its reference and incident relationships from the active
View. The card's shared content and references in other Views remain available.
Search, type filters, View all, and relationship rollup controls are remembered
separately for each View during the session.

## Add Meaning and Visual Context

Drag from a card’s edge port to another card, then choose the relationship type
from the list beside the pointer. **Cancel** or Escape dismisses the list.
Connecting ports preserves the existing inspector selection.

To add a relationship using the inspector, select multiple cards, choose the
primary card, and use **Connect...** for a target. Choose the kind from the
list to create a directed relationship from the primary card to the target. Inspect relationships
to review or change their kind. Available kinds depend on the endpoint types:
**owns** and **operates** require a team source; **implemented in** requires a
process source and repository target.

Choose **Create → Boundary** to add a named rectangular area for visual
context. A boundary has no membership list. Moving a card inside it does not
assign ownership or create a relationship. In the boundary inspector, choose
**Zoom layer** to move it between saved layers, or **Master context** to keep
it visible across layers. Boundaries follow their selected layer's fade.

A card drawn inside another card is also spatial composition. It is different
from an explicit **contains** relationship. Some UI interactions treat spatially
nested cards as a group for movement or connection behavior, but position alone
does not establish a graph fact.

Relationship paths are calculated in canvas coordinates and reused while you
zoom or pan, including when semantic detail appears or leaves the viewport.
Moving or resizing cards and boundaries recalculates routes for the new layout.
Visibility still follows the active View and zoom layers.

## Work with Zoom Layers

Open **Layers** and choose **+ Create layer here** at the desired position in
the list. The creation form opens directly at that insertion point. Enter the
name and range, then create it. Later ranges shift when
needed to make room, retaining their widths. **Create → Zoom layer** opens a
new layer draft after the existing layers. Layers belong to one View. Their
active ranges and fade distances use zoom percentages: 100 means normal scale,
while 25 means a quarter of that scale.

Typed layer settings save when you press Enter or leave the field, so partial
numbers do not change the saved range. Dragging a numeric control still updates
its value directly.

Drag a layer to another row to reorder it. The existing zoom ranges stay in
their list positions, and the reordered layers take those ranges. For example,
moving Detail (85–200%) above Process (24–85%) gives Detail 24–85% and Process
85–200%. Layer membership, fade and card size stay with the named layer.

A card can use Automatic placement into a layer based on its displayed size,
or Manual selection of named layers. These are presentation choices, not
changes to the card's type or relationships. During an automatic resize, an
amber border indicates that the card will change layers. The resize preview
names the destination layer. Right-click a card and use **Move up a layer and
resize** or **Move down a layer and resize** for a centered resize that preserves
its aspect ratio and assigns the adjacent layer explicitly.

Card and boundary summaries fade in according to their displayed size, including
layer card-size settings. They do not wait for a fixed global zoom threshold.

## Save and Reopen

| Action | Desktop | Browser |
| --- | --- | --- |
| Open | **File → Open...** selects a local JSON document. | **File → Open canvas...** imports a selected JSON document into browser storage. |
| Save | Verifies and saves the active file, with a sibling backup. | Downloads JSON; it does not write back to the imported file. |
| Save As | Creates a separate local copy without replacing an existing file. | Downloads a separately named JSON copy. |
| Reload | The app can reopen its remembered file. | Restores the last canvas from this site's browser storage when available. |

Desktop edits are persisted through the workspace action path; **Save** also
verifies the active document. Browser storage is not a substitute for downloads
or backups. Keep separate copies of important work before testing new builds.
See [data and recovery](../README.md#data-and-recovery) and
[browser file access](webassembly.md#browser-file-access).

## Work with an Agent

Connect the local MCP server to the specific JSON file you want an agent to
inspect. Its tools can read that file and, when explicitly enabled, make
validated edits. The desktop app can refresh when that file changes, but it
defers refresh while an editor or gesture is active. Coordinate human and
agent authoring to avoid competing edits.

A browser import is a separate copy. To follow external file changes, use
**Open and refresh from file...** in a supported browser, or reimport the saved
JSON. A hosted browser canvas does not automatically share a desktop file.
See [agent setup](product/external-agent-support.md).
