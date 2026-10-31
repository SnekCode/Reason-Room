# Author a canvas that unfolds with zoom

Use this guide for walkthroughs, examples, and systems with several levels of
detail. Build through MCP, using the read/change/read workflow in SKILL.md.

## Separate three kinds of structure

- A **visual boundary** is a named Lens-local region: an account, company,
  responsibility area, or system context. It has geometry, not graph membership.
- A **card can act as a boundary** by enclosing smaller cards. Use large parent
  rectangles, inset children and leave header space. Place detail cards inside
  their process cards to produce a second level of spatial nesting. Moving a
  parent can move its spatial descendants in the UI. Nesting alone creates no
  `contains`, `owns`, or other graph relationship.
- A **typed relationship** is a version-level directed fact. Add meaningful
  peer connections inside a region and connections crossing regions. Use the
  available kinds honestly; `owns` and `operates` need a team source, and
  `implemented_in` needs a process source and repository target. Use card notes
  to explain material versus information flows rather than inventing edge kinds.

## Plan the reading path before placing records

Give the overview a small set of recognizable stages. Let each stage be a large
card containing process cards, each with work records nested inside. Choose
coordinates so the full overview fits at a low scale and children become
readable as the pointer zooms into their parent. Leave gaps for connection routes
and room for region titles. Describe the entry point and zoom thresholds in a
card summary; select a card to read its full notes in the inspector.

Create semantic layers first, then cards. Example ranges are overview 1–24%,
process 24–85%, and detail 85% upward. These are authoring choices, not required
product defaults. `card_scale: 1` preserves nesting geometry. Assign manual
membership explicitly: overview cards in every level, process cards in process
and detail, and work records in detail only. Do not rely on rectangle size to
produce a specific narrative through automatic assignment.

Relationships can reveal their endpoints even when an endpoint's own layer
would hide it. A connection from an overview card to a detail card may therefore
expose that detail early. Prefer connections between peers at the same level
when progressive disclosure matters; verify the actual frame rather than
assuming membership alone controls visibility. Parent cards also remain visual
context for visible descendants.

## Verify the story through MCP and the app

Use `preview_semantic_zoom` at interior scales (for example 0.12, 0.45, 1.2),
then near range transitions. Check visible titles and bounds, relationship
endpoints, parent context and progressively richer content. `read_lens` must be
paged for larger maps. Use `query_workspace` to check cross-region connections
and their direction. A larger visible count alone does not prove a useful map.

Finally inspect the overview and a zoomed region in the actual app. MCP cannot
set a camera, fit a viewport, or test visual legibility. The example loader owns
the opening camera; no camera state is saved in the workspace format. Keep
illustrative metrics and cloud resources clearly labelled. Do not provision a
real service merely because a canvas describes one.
