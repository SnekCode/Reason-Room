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
- Select a card to inspect its name, type, summary, and details. Use
  **Show inspector** if the inspector is collapsed.
- Drag cards to arrange them. Use their resize handles to change their size.
- Use the zoom controls to move between scales. The **View all** toggle can
  show all cards and relationships for inspection; choose **Use layers** to
  return to layer filtering.
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

## Add Meaning and Visual Context

To add a relationship using the inspector, select multiple cards, choose the
primary card, and use **Create depends on** for a target. That creates a
directed dependency from the primary card to the target. Inspect relationships
to review or change their kind. Available kinds depend on the endpoint types:
**owns** and **operates** require a team source; **implemented in** requires a
process source and repository target.

Choose **Create → Boundary** to add a named rectangular area for visual
context. A boundary has no membership list. Moving a card inside it does not
assign ownership or create a relationship.

A card drawn inside another card is also spatial composition. It is different
from an explicit **contains** relationship. Some UI interactions treat spatially
nested cards as a group for movement or connection behavior, but position alone
does not establish a graph fact.

## Work with Zoom Layers

Choose **Create → Zoom layer** to add a presentation level. Layers belong to
one View. Their active ranges and fade distances are expressed as zoom
percentages: 100 means normal scale, while 25 means a quarter of that scale.

A card can use Automatic placement into a layer based on its displayed size,
or Manual selection of named layers. These are presentation choices, not
changes to the card's type or relationships. Changing a card's size can affect
its automatic layer choice. Inspect the layer controls and zoom through the
result to check that the intended context remains understandable.

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
