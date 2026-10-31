# Eframe Candidate

> **Historical work-item evidence.** On 2026-10-30, the
> [product reset](../wiki/product-reset-2026-10-30.md) retired the work-item
> workflow that appears below. These dated entries remain eframe-candidate
> validation evidence, and this page remains the required record for later
> manual eframe candidate validation. They do not define active product
> milestones, taxonomy, persistence, or shell decisions. Use the
> [product roadmap](../roadmap.md) and [product vision](../product/vision.md)
> for active discovery. For manual application evaluation, use `./start.sh`
> only.

This page records dated decisions and validation boundaries for the
project-owned `eframe` candidate as of 2026-10-29. The archived work-item and
test records remain the sources for those dated entries.

## 2026-10-31: External MCP Refresh Pending Visual Check

The desktop candidate now checks its open local document twice per second for
a verified source change made outside the application, including a write from
the local MCP server. A changed source reloads into the current version and
Lens when they still exist. The refresh clears selections and undo/redo
history that describe the previous source revision. If the external document
does not validate, the resident canvas remains unchanged and the candidate
reports the failure once.

Refresh waits while a person is dragging, has an open dialog, is editing a
card, or has an authoring pane with unsaved input. It resumes automatically
when the canvas is idle. Automated coverage verifies unchanged-source polling,
valid external replacement, invalid-source preservation, and authoring-state
deferral. Formatting, full workspace tests, build, and strict Clippy passed.
Manual validation remains deferred. Start the rebuilt candidate with
`./start.sh`, use MCP to change a second process's card or boundary in the
open document, and confirm that the canvas refreshes without reopening it.

## 2026-10-31: Browser Canvas Restore and Local Refresh Pending Visual Check

The browser candidate now stores the complete last-opened canvas in the
browser's local storage immediately after an import, browser authoring action,
or accepted external refresh. Reloading the page restores that canvas instead
of returning to the bundled example.

`Open canvas` always uses the ordinary compatible file picker. When the
browser supports the File System Access API, `Open and refresh from file` gives
the candidate a person-approved file handle. The candidate checks that selected
local file twice per second and refreshes it after a valid MCP or other local
save. A refresh waits while a dialog, gesture, draft, or authoring surface is
active. Browser security does not permit the ordinary picker to watch later
changes to its selected file.

Native workspace tests, strict Clippy, the WebAssembly release build, and the
supported `./start.sh --web` launch path passed. Manual browser validation is
pending: open a local canvas in a Chromium-based browser, reload the page, and
confirm the same canvas returns. Then use MCP to update the selected source
and confirm the browser canvas refreshes without another Open action.

## 2026-10-31: Shared Card and Boundary Movement Pending Visual Check

Cards and visual boundaries now use one shared canvas-item containment rule
for drag preview and saved movement. A selected item moves every card and
visual boundary that it fully encloses, including nested descendants. An item
that only overlaps remains in place. Cards remain graph concepts with types,
relationships, and details. Visual boundaries remain Lens-local presentation
frames with no graph meaning.

Automated canvas, workspace-session, application, build, formatting, and
strict Clippy checks passed. `./start.sh` detected the already running eframe
candidate and did not replace or close it. Manual visual validation remains
deferred. When it resumes, close the existing candidate, start the rebuilt
candidate with `./start.sh`, then move a parent card and a parent boundary.
Confirm that each moves fully enclosed child cards and nested boundaries once,
while an overlapping sibling remains fixed.

## 2026-10-31: Container Relationship Readability Pending Visual Check

Relationship paths now paint above all card surfaces. The routed path still
ends at its source and target card edges, so it remains readable through a
parent card without drawing across endpoint card content. Relationship arrows
and labels remain above the route. Context trail entries for cards now show
the card name and, when available, its layer name; they no longer repeat the
word `Card`.

Automated workspace tests, build, formatting, and strict Clippy checks passed.
Manual visual validation remains deferred. Start the rebuilt candidate with
`./start.sh`, then inspect a relationship between children in different parent
cards. Confirm that its route stays visible through both parent surfaces and
that the Context trail shows card names without a card-type prefix.

## 2026-10-30: Unified Inspector Surface Pending Visual Check

The right-side inspector now has one slate control surface for all creation
and inspection flows. It has a visible border, a consistent inset, full-width
text fields, blue section headings, and a clear primary action. Creation panes
no longer add a second competing card inside the shared inspector. Concept,
boundary, relationship, and zoom-layer panes use the same heading and section
rhythm. Destructive actions remain at the end of their respective flows.

Full workspace tests, build, formatting, and strict Clippy passed. Desktop
capture returned a stale executable image during this pass, so visual
validation remains pending. When it resumes, run `./start.sh` after closing the
old candidate. Check Create card, a selected card, a selected boundary, a
selected relationship, and both zoom-layer panes at the normal window width.

## 2026-10-30: Canvas Context Menu and Narrow-Window Repair Pending Visual Check

Right-clicking the canvas now opens a menu for the exact item under the pointer.
Cards, visual boundaries, and relationships provide their matching inspect,
copy, or delete actions. Every canvas menu also provides creation of a card,
boundary, or zoom layer. Cards and boundaries created from this menu use the
clicked canvas position. `Center view` centers and only zooms out far enough to
fit every card and visual boundary in the current Lens.

The search type filter stays open while individual checkboxes are changed.
In the search field, Enter advances to the next result and Shift+Enter moves to
the previous result. Search input keeps keyboard focus while the result is
selected.

The application now lays out the canvas and inspector as explicitly separate
rectangles. This prevents the canvas painter from covering the layer pane in a
narrow window. Layer cross-section controls wrap within the available pane
width and the existing inspector scroll surface remains available.

Automated tests cover the non-overlapping layout geometry, centering behavior,
and search-result cycling. Manual eframe validation remains deferred. When it
resumes, use `./start.sh`, open the Layer cross-section in a narrow window,
change more than one type filter, cycle results with Enter and Shift+Enter, and
right-click each canvas target type before using `Center view`.

## 2026-10-30: Inspector, View Creation, and Canvas Interaction Repair Pending Visual Check

The shared inspector now closes when its last selected card, boundary, or
relationship is cleared. New versions and views use that same inspector rather
than a modal. A version always copies the selected version. A new view starts
blank by default; an explicit `Clone current view` checkbox preserves the
previous clone behavior when it is wanted.

Deleting a card now also removes its saved semantic-layer display override, so
the candidate document stays valid. Moving one card into, out of, or between
visual boundaries reports the resulting spatial context. The Context trail now
includes each visual boundary that supplies context across either complete
viewport axis, which is the same rule that makes it noninteractive.

Canvas clicks surrender stale inspector text focus, so Cmd+C and Cmd+V again
operate on selected cards after returning to the canvas. Relationship paths
remain a stable blue when they route through a boundary. They stay below card
hit targets, including cards that are spatial containers. Card names are
ellipsized and card summaries wrap and clip inside their card; summary text
still appears only at the detail level.

Automated coverage verifies blank and copied-view presentation behavior,
deletion with a semantic card-mode override, visual-boundary movement notices,
context-axis inclusion, relationship hit priority, stable route color, and
shared-pane creation state. Manual eframe validation remains deferred. When it
resumes, use `./start.sh`, test Cmd+C and Cmd+V after editing a card, create a
blank and a cloned view, move a card across a visual-boundary edge, and confirm
that a line below a container card cannot be selected through that card.

## 2026-10-30: Independent Layer and Context Controls Pending Visual Check

The upper-left Layer panel now contains its current-layer label, ordered layer
controls, and a functional zoom-percent slider. The slider uses the complete
camera range on a logarithmic track. It preserves the world point at the
viewport center while the person drags it.

The Context trail is a separate panel below the Layer panel. It uses the same
surface, border, type scale, and indentation treatment, but it sizes itself
from the visible nesting trail. It does not change the Layer panel's height.
Long context entries wrap within the Context trail.

Automated application tests cover logarithmic zoom-range mapping, stable
camera centering, slider geometry, and nested Context-trail indentation.
Manual eframe validation is deferred. When it resumes, use `./start.sh`, drag
the zoom slider, confirm that the canvas stays centered on the same world
point, and confirm that a deep context trail expands below rather than inside
the Layer panel.

## 2026-10-30: Result-Based Movement Feedback Pending Visual Check

Routine card movement and card resize no longer show a generic saved-message
toast. A movement toast appears only when a card enters, exits, or changes its
containing card. A resize toast appears only when it changes the card's
resolved zoom layer. The message names the affected card and its new spatial
context or layer.

A visual boundary that fully encloses the current viewport is context-only.
It stays visible in the context trail, but does not accept move or resize
focus. Zoom out until the boundary no longer encloses the viewport to edit it.
When more than one editable boundary contains the pointer, the smallest one
receives focus.

Automated application tests cover containment feedback, layer-change feedback,
nested-boundary priority, and context-boundary exclusion. Manual eframe
validation is deferred. When it resumes, use `./start.sh`, move a card without
changing its container, then move it into another card and confirm that only
the second move reports the new containment. Resize an automatic card across a
layer threshold and confirm that the reported layer changes. Zoom into a
nested boundary and confirm that its enclosing context cannot take direct
focus.

## 2026-10-30: Nested Boundary Movement and Context Trail Pending Visual Check

Moving a visual boundary now also moves each fully enclosed nested boundary.
The move uses the original outer-boundary bounds, so an intersecting sibling
does not move. The canvas drag preview uses the same containment result as the
saved move.

The upper-left canvas status is now a compact navigation trail. It presents
the current layer on one line, the zoom and editable-card count on the next,
then a `Context` tree. Each enclosing item identifies whether it is a
`Boundary` or `Card`; cards also show their base zoom layer when available.
The indentation reflects spatial containment from outer to inner.

Automated canvas and application tests cover saved movement, drag preview,
intersecting-boundary exclusion, and context labels. Manual eframe validation
is deferred. When it resumes, use `./start.sh`, drag an outer boundary that
contains a nested boundary, and confirm that both move together while a
boundary that crosses the outer edge stays in place. Then zoom into nested
boundaries and cards and confirm that the context trail remains readable and
ordered.

## 2026-10-30: Independent New-Canvas Documents Pending Visual Check

The native File menu and the visible canvas File menu now provide **New
Canvas...**, **Open...**, **Save**, and **Save As...**. Open and Save As use a
native file chooser. The focused new-canvas dialog asks for a canvas name and
shows a suggested independent local JSON path. A person can replace that path
before creation. The application creates and verifies the empty workspace
document before it switches the canvas to that document. Save re-verifies and
persists the active document. Save As creates and verifies a separate copy,
then switches to it. Creation and Save As use an exclusive file operation, so
they do not replace a file that already exists at the chosen path.

An empty document starts with a current-state version and one empty `Canvas`
Lens. Existing cards, relationships, layers, local history, selections, and
transient editor state do not transfer to the new document. The canvas header
shows the active document title after the switch.

Automated coverage verifies valid deterministic empty-document serialization,
default-view loading, successful creation and opening, explicit-save
verification, external-change rejection, preservation of an existing target
file, deterministic title-derived identifiers, and collision-resistant
suggested names. The first manual eframe check must use `./start.sh`, create a
canvas through File > New Canvas, confirm the new empty canvas and its title,
save it, save it as a new path, then reopen both saved documents. Confirm that
the displayed save paths exist and the original document was unchanged. Manual
evaluation remains deferred.

## 2026-10-30: Window Geometry and Card Containment Pending Visual Check

The main canvas window now restores its last size and screen position after a
normal close and relaunch. This application-chrome state is stored locally by
the desktop shell. It does not change any workspace data. The first visual
check must close the running candidate normally, relaunch it with `./start.sh`,
and confirm that the main window returns to its previous size and position.

Concept cards now provide spatial containment. A card is a visual boundary for
another card only when it fully encloses the other card and is larger in both
dimensions. If several cards enclose one card, the smallest enclosing card is
its immediate visual parent. Parent cards paint before their children, child
cards receive pointer priority, and relationship lines use active card
boundaries for their blue and amber context change. Spatial containment never
creates, edits, or removes a graph relationship.

Visible application strings now use plain ASCII punctuation. This removes the
unsupported middle-dot, arrow, infinity, and ellipsis glyphs that displayed as
square boxes in the candidate.

Automated coverage verifies nearest-parent selection, partial and equal-size
overlap rejection, child-first hit testing, and drag-preview containment.
Manual eframe validation is deferred.

## 2026-10-30: Exact Automatic and Manual Card Display Pending Visual Check

The card inspector now has two layer-display modes. `Automatic` uses card size
to resolve exactly one layer. `Manual` presents the available zoom layers as a
checkbox list and requires one or more selected layers. Selecting the final
checked layer is disabled with an explanation. A manual card is visible only
on its selected layers; relationship context can still keep an endpoint
available while a relationship is rendered.

Workspace schema version 6 migrates schema version 5 dedicated and
always-visible card policies to a manual selected-layer list. Deleting one
manual layer preserves the remaining selections; deleting the last selected
layer returns the card to automatic display. Automated workspace coverage
passes. No visual card-inspector result is recorded yet.

## 2026-10-30: Native Settings Menu and Separate Window Pending Visual Check

The application now uses platform menu conventions for settings. macOS provides
**Preferences…** in the application menu. Windows provides **Settings** in a
native menu bar. Both open one deferred native operating-system window. The
canvas does not contain an embedded settings panel.

macOS automatic window tabbing is disabled before either application window is
created. This prevents macOS from placing the canvas and Preferences in one
tab group when the person has selected a system-wide tabbing preference.

The first settings are session-only display controls for the canvas grid and
relationship labels. They update the visible map immediately and do not alter
workspace facts, saved layers, cards, or relationships. The native menu uses
the same event path as the window. Reopening it focuses the existing
preferences window rather than creating another one.

macOS build, tests, and strict Clippy passed. The installed toolchain has no
Windows target, so a Windows compile is deferred. The existing candidate was
not restarted and no manual visual result is recorded. When evaluation resumes,
close that candidate safely and use `./start.sh` to open the rebuilt candidate.

## 2026-10-30: View-All Recovery Mode Pending Visual Check

The top bar now provides a View all control when the active Lens has saved
zoom layers. It temporarily restores all cards and relationships in the Lens,
without changing saved layer settings or card policies. The active control says
Use layers, and the canvas states that it is showing all cards.

Automated app coverage verifies that this mode disables only semantic-layer
filtering. The open candidate was not restarted, so no new manual result is
recorded. When evaluation resumes, close that candidate safely and use
`./start.sh` to open the rebuilt candidate.

## 2026-10-30: Automatic Resize-Layer Repair Pending Visual Check

Automatic card placement now resolves from the card's current Lens-local size.
Historical layer membership no longer pins an automatic card after a resize.
Use the explicit Dedicated policy to pin a card to a layer.

The regression test adds old broad-layer membership to an automatic card, then
verifies that resizing still changes its resolved layer. Formatting, workspace
tests, strict Clippy, build, active-guidance, and whitespace checks passed.
No new manual result is recorded because the open candidate was intentionally
left running. When visual evaluation resumes, close that candidate safely and
use `./start.sh` to open the rebuilt candidate.

## 2026-10-30: Layer Cross-Section and Resize Cues Pending Visual Check

The shared-pane `Layers` view now displays a `Master context` baseline and one
range track for each saved semantic zoom layer. Each row lets a person set and
save Start, End, and Fade values in percent. The baseline is Lens orientation,
not a new saved parent layer; the implementation does not infer a hierarchy
between saved layers.

The lower-right layer rail now responds to hover, identifies the current layer
with its full wrapped name, and shows a full hovered-layer label. It remains
selectable. The card inspector now lists base-layer and deeper context
visibility. Resizing previews any automatic layer change, reports the saved
result, uses top-left anchoring by default, and uses Option for a center
anchor. Resize initiation now matches the visible corner handle. Relationship
lines remain visible and the non-functional visibility button is removed.

Automated tests cover full rail labels, cross-section range position,
precise resize targeting, live anchor-state derivation, automatic base-layer
visibility, and existing relationship rendering. No new manual visual result
is recorded. The open candidate was not closed or replaced. When visual
evaluation resumes, use `./start.sh` only after that candidate can safely
close.

Source: [Semantic-Zoom UI Contract](../wiki/semantic-zoom-ui-contract-2026-10-30.md).

## 2026-10-30: Semantic-Zoom Authoring Repair Pending Visual Check

Relationship removal now clears every saved semantic-layer reference before
candidate validation. This repairs the reported unknown-relationship error
when a removed relationship had been assigned to a layer. The create-layer
pane now exposes initial active zoom-range start and end values in percent and
one symmetric Fade value. The lower-right authored-layer rail is selectable;
selecting a layer opens it in the shared editor. A changed type in the
relationship inspector now saves immediately and reports the normal saved
notice.

Automated workspace, session, and app tests cover the reference cleanup,
initial percent settings, and rail selection. `cargo fmt --check`,
`cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D
warnings`, `cargo build --workspace`, active-guidance, and whitespace checks
passed on 2026-10-30. The existing evaluation candidate was intentionally left
open; no new manual visual result is recorded. When evaluation resumes, use
`./start.sh` only after the candidate can safely close.

Source: [Semantic-Zoom Authoring Repair 2026-10-30](../wiki/semantic-zoom-authoring-repair-2026-10-30.md).

## 2026-10-30: Percent Layer Editor and Dynamic Rail Pending Visual Check

The shared inspector now edits each zoom layer with an active zoom range in
the same percent unit shown on the canvas and one symmetric `Fade` value.
Separate directional fade controls no longer appear. The rail at the lower
right now creates one segment for every saved layer, names the active layer in
its header, and names both layers when a transition has equal opacity.

A selected card now has a `Layer visibility` selector with automatic
size-derived behavior, dedicated-layer selection, and always-visible context.
These choices use Lens-local presentation state and do not change concepts or
relationships. Normal staging refreshes the displayed frame after layer,
policy, resize, undo, and redo mutations.

Automated app, workspace, and session tests cover the range values, dynamic
rail, policy labels, resize-derived assignment, endpoint context, and staged
updates. No new human visual result is recorded. Manual eframe evaluation
remains deferred and must use `./start.sh` when it resumes.

## 2026-10-30: Create Menu and Direct Relationships Pending Visual Check

The top bar now has a `Create` menu for cards, visual boundaries, and zoom
layers. Each option opens the existing shared right pane in a focused creation
mode. The inspector remains in that same pane. The former `Author map` label
is no longer current.

Relationship creation is now direct. Command-click selection of two cards
shows the primary card and an immediate default `depends on` action. A drag
from the blue right-edge port of one card to the white left-edge port or body
of another card saves the same default relationship. Neither path asks for a
confirmation dialog. Relationship paths use blue outside visual boundaries
and amber inside them. This remains a visual cue only.

No new participant visual check is recorded. The existing evaluation window
was intentionally left open. When manual validation resumes, first close it
only if safe, then run `./start.sh` to inspect the rebuilt candidate.

Source: [Authoring Refinement 2026-10-30](../wiki/authoring-refinement-2026-10-30.md).

## 2026-10-30: Direct Authoring Cleanup Pending Visual Check

The candidate now presents direct authoring controls for cards, semantic zoom
layers, and visual boundaries. A new card asks first for a name and short
summary; its type remains optional. The Layers control creates named overview
and detail layers without requiring an initial settings form. A visual boundary
saves a name, optional short summary, and Lens-local rectangle. It provides
visual context only and does not create membership, ownership, responsibility,
or a graph relationship.

The Question-led study control no longer appears in the application interface.
The active candidate window was intentionally left open, so this changed UI
has not received a new visual check. Close the candidate window, then run
`./start.sh` to inspect the rebuilt direct-authoring interface. Automated
workspace, session, and app tests cover the saved boundary rule and the
authoring paths.

Source: direct-authoring cleanup (2026-10-30).

## 2026-10-30: Unified Map Authoring and Boundary Manipulation Pending Visual Check

The candidate now groups card, visual-boundary, and semantic-zoom-layer
creation in `Author map`. The old card and boundary dialogs are gone. A
selected boundary has a dedicated inspector for its name, short context, and
optional precise geometry. It moves when dragged and resizes from a visible
lower-right handle. Those actions save through local history and do not alter
cards or relationships.

The temporary Question-led study is removed from the compiled candidate. No
fresh manual visual result is recorded because participant evaluation is
deferred. When it resumes, use `./start.sh` only after the existing candidate
window is safely closed. Confirm that the authoring panel is easy to scan, a
boundary remains readable and selectable, and direct manipulation preserves
map orientation.

Source: [Unified Map Authoring 2026-10-30](../wiki/unified-map-authoring-2026-10-30.md).

## 2026-10-30: Candidate Reuse Protection

When the evaluation candidate is already open, macOS can reject a second
candidate instance with a LaunchServices spawn error. The supported launcher
now detects that exact candidate executable before it builds or stages another
bundle. It preserves the open candidate and reports that its existing
workspace and executable image continue to be used. To test a rebuilt
candidate or another external workspace, an evaluator must first close the
open candidate, then run `./start.sh` again.

The candidate was not closed during the reuse check. This is launch evidence,
not manual product validation. It supersedes only the earlier repair entry's
every-launch staging claim.

Source: launcher reuse repair (2026-10-30).

## 2026-10-30: Operational Atlas Scale Feedback Pending

The candidate now has a non-persistent semantic-scale rail that labels the
existing camera ranges as Broad, Working, and Detail. The selected Lens is
also identified as a View in the top bar. The rail uses no positional
transition: its bounds stay fixed and only its active-band emphasis changes.
The eframe presentation tokens distinguish product chrome from workspace
authored category color.

Focused deterministic tests cover the camera-threshold mapping, fixed rail
bounds, active-band opacity, and plain-language View label. Manual visual
validation remains deferred. When a participant is available, confirm that the
rail preserves canvas space, the active range is understandable while zooming,
and View versus detail-level language remains clear.

Source: `crates/app/src/main.rs` and [Operational Atlas Scale
Feedback](../wiki/operational-atlas-scale-feedback-2026-10-30.md) (2026-10-30).

## 2026-10-30: Scale Rail Saved-Layer Alignment Pending

An integration review found that the first scale-rail implementation used fixed
camera thresholds even when a Lens had saved layers with custom fade ranges.
The candidate now derives the rail from the resolved layer presentation when
saved layers exist. It preserves the fixed camera-scale wording only when a
Lens has no saved layers. If a saved layer set has no active layer, the rail
states that condition rather than showing an unrelated camera range.

Focused tests cover resolved custom fade transitions, an empty-layer gap, and
the pure layer-opacity projection. No visual result is recorded. When a
participant is available, confirm that the rail's displayed transition matches
the perceived detail transition and that a temporary empty-layer condition is
understandable.

Source: `crates/canvas/src/workspace.rs`, `crates/app/src/main.rs`, and
[Operational Atlas Scale Feedback](../wiki/operational-atlas-scale-feedback-2026-10-30.md)
(2026-10-30). This supersedes only the earlier camera-threshold rail statement.

## 2026-10-30: Desktop Capture Identity Repair

The eframe launcher now builds the release executable, stages it in an
`Infinite Note Canvas Evaluation.app` bundle, and opens that bundle through
macOS. Its bundle identifier is
`com.local.infinite-note-canvas.eframe-evaluation`. This gives the desktop
capture tool a candidate-specific application identity. It prevents the tool
from selecting the retired historical native application based on a shared
display name, and it makes the candidate available as a separately selectable
desktop application.

The staged bundle is regenerated from the release executable on every launch.
The app still receives the selected external workspace through
`INFINITE_NOTE_CANVAS_WORKSPACE`, and `./start.sh` remains the only supported
launch entry point.

Validation: `./start.sh` opened the candidate. Desktop capture listed a
running `Infinite Note Canvas Evaluation` application with the expected bundle
identifier. It then attached to that application's live window, titled
`Infinite Note Canvas`. The candidate remained open after this check.

Source: launcher repair (2026-10-30).

## 2026-10-30: Editable Semantic Zoom Layer Foundation

The candidate now saves ordered semantic zoom layers inside a Lens. A person
can create a base layer from the existing cards and relationships, add deeper
layers, set fade-in and fade-out scale ranges, set card size, and assign
selected cards or their relationships to a base layer. A card remains visible
as context when a deeper layer becomes active. New cards start in the broadest
layer, and new relationships start at the first layer where both endpoint cards
exist.

The pure projection, persistence migration, typed staged actions, base-level
uniqueness, card-context continuity, relationship endpoint visibility,
direct-creation defaults, deletion reassignment, persistence, and undo were
covered by deterministic workspace and session tests. Formatting, workspace
tests, build, strict Clippy, documentation guidance, and whitespace checks
passed on 2026-10-30.

An evaluator launched the candidate through `./start.sh` with a new writable
workspace at `/private/tmp/infinite-note-canvas-layer-check.XXXXXX.json`. The
launcher completed the release build and ran `target/release/infinite-note-canvas`.
Computer Use continued to attach to an already open historical native window
with the same app identity. The evaluator did not close the user-open window.
Manual visual validation of the new eframe layer editor therefore remains
incomplete.

Source: semantic-zoom-layer implementation and manual launch attempt
(2026-10-30).

## 2026-10-29: Study Side-Pane Layout Check

The study inspector now gives each color-key item its own row. The focal title
and key labels use the available pane width and wrap. An automated narrow-pane
check renders the data-reporting scenario at 160 pixels and verifies that the
focal text and every key entry remain inside the inspector boundary.

An evaluator then launched the candidate through `./start.sh` with a
disposable external workspace. The candidate built and ran, but Computer Use
attached to an already open historical native window with the same application
identity. That window has no eframe study pane. The evaluator did not close the
user-open window because it could contain unsaved work. Visual confirmation of
this eframe pane change therefore remains incomplete.

Source: manual launch attempt and automated layout test (2026-10-29).

## 2026-10-29: Boundary-Study Manual-Validation Blocker

On 2026-10-29, an evaluator started the boundary-study candidate with
`./start.sh`. The launch output showed a completed build and
`Running target/release/infinite-note-canvas`. Computer Use did not surface an
`eframe` application window for visual interaction. No boundary-study gesture,
rendering, or workspace-isolation behavior was observed.

This entry records launch evidence only. Manual visual validation of the
boundary-study slice remained incomplete and blocked. It does not accept the
slice or replace the observed-session requirements in the discovery protocol.

Source: manual evaluation attempt (2026-10-29); [Boundary-Study
Seam](../wiki/boundary-study-seam-2026-10-29.md); [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).

## 2026-10-29: Selected Implementation Path

The user selected a project-owned `egui` plus `eframe` thin-client build path
as the 2026-10-29 implementation path. At that date, `crates/app` owned
presentation and painter code. `crates/workspace-session` was the UI-free
typed-action and safe local persistence boundary. `crates/canvas` was the
UI-free pure canvas and visibility boundary.

The candidate retained pointer-anchored plain-wheel zoom, Shift-wheel,
Shift-hover, and middle-button pan. It was a functional-parity candidate, not
a final product-shell selection. Historical native and GPUI results did not
close eframe acceptance criteria.

Source: [Work Item 0021](../archive/work-items-2026-10-30/Complete/0021.md) (2026-10-29).

## 2026-10-29: Selected-Shell Undo and Redo Validation

The user passed selected-shell macOS validation for Work Item `0006`. The
application launched with `./start.sh` against a disposable external workspace
and remained open; the external workspace retained application and startup
logs. The user validated undo and redo ordering, divergent-edit redo clearing,
the 65-edit capacity sequence, excluded navigation and draft interactions, and
draft or deletion-dialog shortcut consumption.

Known limits remain: history retains only the newest 64 successful saved
edits, navigation and drafts do not create history entries, and history does
not survive relaunch. The validation launcher was ended with `TERM` after the
test. This validation does not accept a final product shell.

Source: [Work Item 0006](../archive/work-items-2026-10-30/Complete/0006.md) (2026-10-29).

## 2026-10-29: Relationship Inspector Manual Result

The canonical Work Item `0005` record contains a user-recorded `Pass` for the
selected-shell relationship inspector workflow. The tester noted that the
older detailed procedure was superseded after Work Item `0021`. This entry
records no result beyond that canonical record.

Source: [Work Item 0005](../archive/work-items-2026-10-30/Complete/0005.md) (2026-10-29).

## 2026-10-29: Lens-Filtered Type Change Confirmation

When a user selects a type that is excluded by the active Lens filter, the
eframe candidate now requires a confirmation before the safe local save. The
dialog explains that the concept remains in the workspace but becomes hidden in
that Lens. A type that remains visible saves without a confirmation dialog.

Focused app tests and the locked workspace format, test, build, and strict
Clippy gates passed. The user reported that the dialog box works in the macOS
candidate evaluation.

Source: [Work Item 0021](../archive/work-items-2026-10-30/Complete/0021.md) (2026-10-29).

## 2026-10-29: Supported Application Entry Point

`./start.sh` is the only supported application launch entry point. It creates
or reuses a writable external evaluation workspace and then delegates final
path validation to the eframe launcher. It preserves an explicitly supplied
`INFINITE_NOTE_CANVAS_WORKSPACE` when that file is writable and outside the
repository workspace directory.

Internal Cargo commands and `scripts/start-*` helpers are not supported user
launch instructions. They do not replace `./start.sh`.

Source: [Work Item 0021](../archive/work-items-2026-10-30/Complete/0021.md) (2026-10-29).

## 2026-10-29: Remaining Validation Boundary

Automated formatting, locked workspace tests, locked build, strict Clippy,
candidate validation, the shell-boundary check, and launch integration passed.
Work Item `0021` completed its required macOS manual evaluation for moving
concept cards, creating concepts, adjusting relationships, cycling Lens views,
editing concept types and names, and navigating by pan and zoom.

The 2026-10-29 Milestone 2 plan proposed later implementation and validation
for deletion, generic undo and redo, content editing, the relationship
inspector, selected-concept relationship overlay, and native pinch. That
retired plan does not prescribe active work after the product reset. Those
areas were not inherited from native-shell evidence.

Source: [Work Item 0021](../archive/work-items-2026-10-30/Complete/0021.md) and
[Eframe parity protocol](../tester-eframe-parity.md) (2026-10-29).

## 2026-10-29: Interim Gate Repair

The user reported that the eframe inspector reset concept-name input on each
keystroke, would not retain a selected concept type, and shared camera state
between Lenses. The repair stores each inspector draft outside per-frame card
data. It stores camera state by workspace version and Lens. A Lens with no
stored camera centers a retained selected concept card.

Focused app tests and the locked workspace format, test, build, and strict
Clippy gates passed. The later user retest passed. The candidate evaluation
gate is complete.

Source: [Work Item 0021](../archive/work-items-2026-10-30/Complete/0021.md) (2026-10-29).

## 2026-10-29: Local Save and Recovery Closeout

The user's Done transition for Work Item `0008` records a passing manual test
for local save, backup, and recovery behavior. The canonical Human Feedback
contains only the older pending template, so no detailed procedure or evidence
is recorded here.

Source: [Work Item 0008](../archive/work-items-2026-10-30/Complete/0008.md) (2026-10-29).

## 2026-10-29: Focused Manual-Retest Closeouts

The canonical records for Work Items `0022`, `0023`, `0025`, `0026`, `0027`,
and `0028` contain user-recorded confirmed manual results. Work Item `0022`
records a confirmed focused retest of the type-change failure toast and local
diagnostic path. Work Item `0023` records a confirmed focused retest from 2%
to 1% zoom. Work Item `0025` records `Confirmed works well`. Work Items
`0026`, `0027`, and `0028` each record `Confirmed` without detailed procedure
or evidence in their canonical Human Feedback.

These closeouts do not change the acceptance state of Work Items `0004`,
`0007`, `0029`, or `0030`.

Source: [Work Item 0022](../archive/work-items-2026-10-30/Complete/0022.md),
[Work Item 0023](../archive/work-items-2026-10-30/Complete/0023.md),
[Work Item 0025](../archive/work-items-2026-10-30/Complete/0025.md),
[Work Item 0026](../archive/work-items-2026-10-30/Complete/0026.md),
[Work Item 0027](../archive/work-items-2026-10-30/Complete/0027.md), and
[Work Item 0028](../archive/work-items-2026-10-30/Complete/0028.md) (2026-10-29).

## 2026-10-29: Undo and Redo Label Closeout

The canonical Work Item `0024` result records a confirmed focused manual test.
It confirms `Undo` and `Redo` toolbar labels without keystroke text and
preserved `Cmd-Z` and `Cmd-Shift-Z` behavior. This result closes only the
shortcut-label readability correction.

Source: [Work Item 0024](../archive/work-items-2026-10-30/Complete/0024.md) (2026-10-29).

## 2026-10-29: Bottom-Left Toast Hover Retest Closeout

The canonical Work Item `0029` result records a confirmed focused manual
retest. It confirms that an ordinary bottom-left notice stays visible while
hovered after its normal active time, fades after pointer exit, and leaves the
toolbar usable. This result closes the ordinary-notice hover-paused timing
correction only.

Source: [Work Item 0029](../archive/work-items-2026-10-30/Complete/0029.md) (2026-10-29).

## 2026-10-29: Toast Expiry Closeout

The canonical Work Item `0030` result records `confirmed.` for its
toast-expiry scope. Its Human Feedback uses a generic procedure and records no
detailed behavior or evidence. This closeout records no more specific
lifecycle claim.

Source: [Work Item 0030](../archive/work-items-2026-10-30/Complete/0030.md) (2026-10-29).

## 2026-10-30: Release Automation Study Check Pending

`./start.sh` rebuilt and started the eframe candidate for the revised Release
Controls study. Desktop inspection found a separate historical native app
window with the same display identity. That window showed the historical
Organization Landscape interface, not the eframe candidate. The eframe
process remained running, but the desktop tool could not select its window.

The desktop tool can observe the historical app but cannot minimize or close
it because Computer Use is disabled for its app bundle. No app state was
changed during this check.

No visual validation result is recorded for the revised study. The next manual
check must first make the eframe candidate window available, then confirm that
the Release controls lens separates the repository owner's normal action from
the automated pipeline and compliance gate.

Source: local launch and desktop inspection through `./start.sh` (2026-10-30).

## 2026-10-30: Semantic-Zoom Journey Check Pending

The Account responsibility study now starts at a fitted organizational
overview. At medium scale it shows the delivery path and shared controls. At
close scale it shows card summaries and longer relationship explanations. The
study also has a `Show overview` control. It returns the study to broad context
without changing the workspace camera.

Focused camera and projection tests, the full workspace test suite, the
workspace build, strict Clippy, and the active-guidance check passed. The
candidate was built and started through `./start.sh`. Desktop inspection
attached to a window titled `Acme Platform Landscape / Current State /
Organization Landscape`. That window showed the historical organization
landscape interface, not the eframe semantic-zoom study. No visual
semantic-zoom result is recorded.

Source: local launch and desktop inspection through `./start.sh` (2026-10-30).

## 2026-10-30: Study Inspector Width Correction Pending

A supplied eframe study screenshot showed the Key and answer text cut off at
the right edge. The one-line toolbar expanded the layout beyond the visible
window. The candidate now uses a wrapping toolbar so the canvas and inspector
receive the available window width.

The correction has focused automated coverage. A fresh visual check through
`./start.sh` must confirm that the Key and answer text remain visible at the
candidate window width.

Source: participant-supplied eframe screenshot received 2026-10-30.

## 2026-10-30: Browser WebAssembly Run

The shared `crates/app` application source compiled as a
`wasm32-unknown-unknown` release library. The generated browser bindings
started `InfiniteNoteCanvasApp` through `eframe::WebRunner` in an HTML canvas.
The local launch command was `./start.sh --web`; it bound only a static server
to `127.0.0.1:8080`.

Browser inspection displayed the bundled evaluation workspace, eight cards,
six relationships, canvas controls, semantic-scale controls, and the concept
inspector at `http://localhost:8080`. The visible browser banner stated
`Browser demo · changes reset on reload`. This proves that the canvas engine
and eframe UI execute in WebAssembly. It does not claim browser local-file
loading, local-file saves, backups, or recovery. That dated evidence describes
the initial browser implementation only.

Evidence: `cargo build --target wasm32-unknown-unknown --release --lib`,
generated WebAssembly bindings, `./start.sh --web`, and local browser
inspection on 2026-10-30.

## 2026-10-30: Browser Persistence Parity

Browser sessions use page-session workspace state. The File menu provides New
Canvas, Open Canvas, Save, and Save As. Open Canvas uses the browser file
picker to load a user-selected JSON source. Save and Save As create browser
downloads rather than writing an ambient desktop path. This keeps the shared
engine, user interface, typed actions, and history behavior without claiming
browser-local persistence or ambient desktop-path access.

The merged browser candidate was manually checked through `./start.sh --web`
on an alternate loopback port. The File menu did not contain an Open Saved
Canvas action. A selected local `jcwa.infinite-note.json` document replaced the
bundled evaluation workspace with the expected `jcwa` title, `Canvas` Lens,
and zero cards. Browser developer tools reported no application error during
the import; the only console entry was the static server's missing favicon.

## 2026-10-30: Preferences Defaults and Grouping Pending Visual Check

Preferences remains a separate native window. It now uses four tabs: Canvas,
New cards, New boundaries, and Zoom layers. New-card defaults include width,
height, and a preferred type. New-boundary defaults include width and height.
New-layer defaults include active range start, active range end, and fade. The
choices are saved as local application preferences and do not modify a
workspace until the user creates a new item.

Focused unit tests confirm the defaults apply to future authoring and respect
the minimum card size. No desktop visual result is recorded. A later manual
check through `./start.sh` must confirm the separate window, tab readability,
and that changed defaults are present after an application restart.

Source: local implementation and focused automated tests (2026-10-30).

## 2026-10-30: Nested Canvas Context and Offscreen Routes Pending Visual Check

The canvas status area now shows a nested `Context` trail only when the whole
visible world is enclosed. It lists visual boundaries and containing cards
from outer to inner. This keeps an enclosing boundary name available when its
outline is outside the viewport. A card that contains the viewport follows the
same rule as a visual boundary.

Relationship routing now resolves endpoint geometry for all relationship
endpoints before it clips the route to the viewport. A route that crosses the
viewport remains visible when both endpoint cards are outside it. A route that
does not cross the viewport does not paint. Arrowheads and relationship labels
remain at their actual route midpoint, so clipping does not create a false
direction or label cue.

Automated coverage verifies offscreen crossing routes, fully offscreen routes,
separate clipped route segments, enclosing context order, partial-overlap
exclusion, and equal viewport bounds. Manual eframe validation remains
deferred. When it resumes, use `./start.sh` and confirm the context trail while
zoomed inside nested boundaries and containing cards, then pan a route through
the viewport while both endpoint cards remain outside it.

## 2026-10-30: Circuit Crossings, Recent Document, and Card Context Pending Visual Check

Relationship lines now use a circuit-style bridge when two unrelated route
segments cross. The earlier route has a short gap. The later route draws a
small arch over it. A crossing is not a new relationship, and shared line
endpoints do not draw a bridge.

The desktop application saves the local path of the document opened through
New Canvas, Open, or Save As. A normal next start reopens that document. An
explicit `INFINITE_NOTE_CANVAS_WORKSPACE` launch still takes priority. The
standard launch path now sends its evaluation workspace as a fallback, rather
than an explicit selection, so it does not replace the saved recent document.

A card that fully contains a visible child remains rendered as visual context
at deeper semantic zoom layers. It does not gain a relationship or a changed
layer policy. When the containing card encloses the viewport, the existing
Context trail names it.

Automated coverage verifies bridge selection, shared endpoint exclusion,
recent-path precedence, fallback workspace selection, and containment context
across a semantic layer transition. Manual eframe validation remains deferred.

## 2026-10-30: Inspector Quick Editing Pending Visual Check

Double-clicking a concept card now focuses its Name field. Name, Summary, and
Details save when their edited field loses focus. This quiet save keeps the
existing validation and type-change confirmation path. Pending assertion input
blocks an automatic save until the assertion is added, canceled, or corrected.

The inspector no longer has directional movement buttons. Assertions are a
collapsed section at the bottom of the inspector. Relationship kinds are
changed with an inline list that contains only types valid for the selected
source and target. With no selected card, boundary, or relationship, the
inspector can collapse; the top bar provides Show inspector to reopen it.

Automated application, workspace, and session tests passed. Manual eframe
validation remains deferred. When it resumes, use `./start.sh`, double-click a
card, edit each text field and move focus away, change a compatible
relationship kind, then collapse and reopen the empty inspector.

## 2026-10-30: Canvas Find and Zoom-Layer Range Editing Pending Visual Check

The canvas now has a temporary top-right Search cards control. It matches card
names and summaries, selects and centers the active result, and provides
Previous and Next navigation. The adjacent Filter types menu limits only the
current canvas presentation. It does not change the saved Lens filter. Hidden
types also hide their relationships and cannot receive canvas selection,
resize, or relationship-drag input.

Zoom layers are now created inline in the cross-section. Existing range and
fade edits save automatically. Adjacent layers provide `Start = previous end`
and `End = next start` actions for quickly closing a range boundary. The old
separate new-layer pane and Save range button are removed.

Automated coverage verifies case-insensitive title and summary matching,
search camera centering, filtered nested-card hit testing, and the new inline
creation route. Full workspace tests, build, and strict Clippy passed. Manual
eframe validation remains deferred. When it resumes, use `./start.sh` after
closing the old candidate, search for two cards and cycle both directions,
filter one card type, create a layer from the cross-section, edit a range,
and use each adjacent-boundary action.

## 2026-10-30: Context Threshold and Boundary Route State Pending Visual Check

An enclosing visual boundary now enters the Context trail only when it spans a
complete viewport axis and occupies at least 50% of the viewport area. The
same rule excludes it from direct manipulation. Narrow horizontal and vertical
boundary strips remain editable and do not appear as top-level context.

Relationship routes retain the normal blue color when they stay entirely
outside or entirely inside a boundary. A route that crosses a boundary uses
amber only for its portion inside the boundary. Arrowheads and circuit-style
line jumps use the same route-specific state. This preserves the visual cue
for a boundary transition without incorrectly marking contained routes.

The in-window title, document title, and current view label were removed from
the command bar. The native window title supplies that identity. The first row
now contains only commands; the Version, View, Manage, and record-count row
remains below it.

Automated tests cover the 50% context threshold, boundary-crossing route
segments, and fully-contained routes. Full workspace tests, build, and strict
Clippy passed. Manual eframe validation remains deferred. When it resumes, use
`./start.sh`, pan and zoom across a narrow boundary strip and a large enclosing
boundary, then inspect a contained route and a route that enters a boundary.

## 2026-10-30: Authoring, Current-Layer Creation, and Dense Zoom Pending Visual Check

Card, boundary, version, view, and zoom-layer creation now use one framed
authoring treatment. Each opens with focus in its name field. The authoring
surface keeps the short explanation, fields, primary action, and cancel action
together without adding field-level panels.

The canvas search control now returns its actual painted bounds to canvas input.
The former fixed-height capture rectangle no longer blocks deselection or
inspector collapse below the visible control. Clipboard shortcuts now run
before pane widgets process the frame. A selected card also takes priority over
an empty authoring text field, so Command+C and Command+V continue to copy and
paste cards while a creation pane is open. Paste offsets every successive card
group.

When an authored zoom layer is active, a newly created card receives an
explicit one-layer policy for that active layer. Its world size is calculated
from the current camera and layer scale, so its starting screen size matches
the saved card default. The zoom-layer cross-section now edits a layer name,
range, fade, and card scale inline; it no longer opens another edit pane.

At 100% and below, the canvas grid keeps its normal spacing. Above 100%, it
adds a smaller world-grid subdivision at each zoom octave. Card boundaries now
use the same half-viewport and full-axis Context trail rule as visual
boundaries. The boundary inspector now uses direct drag and resize only; its
precise position and size controls are removed.

Automated coverage verifies active-layer sizing, high-zoom grid spacing, and
card-boundary Context trail behavior. Full workspace tests, build, and strict
Clippy passed. Manual eframe validation remains deferred. When it resumes, use
`./start.sh`, open each authoring flow, confirm name focus and pane sizing,
click below the search control to collapse the inspector, create a card in a
non-100% layer, then use Command+C and Command+V on a selected card.


## 2026-10-31: Reason Room macOS Installation and Launch

The working product name is Reason Room. The local macOS installation now uses
rustup-managed Rust 1.97.1 with rustfmt, Clippy, the
`wasm32-unknown-unknown` target, and wasm-bindgen-cli 0.2.129. Apple developer
tools and Python 3 were already installed. The compiler, Cargo, and browser
bindings versions were verified. The previous Homebrew Rust 1.89 compiler
failed before compilation because of an LLVM dynamic-library symbol mismatch.

The supported `./start.sh` launcher now prefers the rustup installation. It
built the release app and opened a window titled **Reason Room**. Visual
inspection showed the existing evaluation workspace with eight cards, six
relationships, semantic-scale controls, and the concept inspector. No
workspace edits were made for this launch check.

`./start.sh --web` on loopback port 8081 compiled the shared application for
WebAssembly, generated the browser bindings, and served the static site.
Browser inspection showed the Reason Room canvas with eight cards, six
relationships, and the canvas controls. `./start.sh --web --build-only` also
completed successfully. This is local launch evidence; it does not establish
GitHub Pages deployment or full browser file-action coverage.

Validation passed: workspace tests, workspace build, strict Clippy for all
targets, formatting, shell syntax, and the active-documentation guard. The
installation procedure and the compiler-error remedy are documented in
`docs/installation.md`.

Source: local installation, command output, and desktop/browser inspection on
2026-10-31, on Apple Silicon running macOS 27.0.1.
