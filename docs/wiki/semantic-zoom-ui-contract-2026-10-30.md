# Semantic-Zoom UI Contract — 2026-10-30

Related: [Semantic Zoom Layer Foundation](semantic-zoom-layer-foundation-2026-10-30.md) · [Authoring Refinement](authoring-refinement-2026-10-30.md) · [Lenses and Semantic Zoom](../product/lenses-and-semantic-zoom.md)

## 2026-10-30 — Layer Cross-Section and Resize Feedback

The top bar now opens a shared-pane **Layers** cross-section. It presents a
non-persistent `Master context` baseline from `0%` to infinity, then one track
for each saved layer. Each track shows its active range in blue and its
symmetric fade distance in amber. A person can change the Start, End, and Fade
values in the same percent unit used by canvas zoom, then save that range from
the row. The individual layer editor remains available for name, card-size,
membership, and deletion controls.

The baseline is orientation for the Lens, not an additional saved layer. The
cross-section does not introduce a parent-child layer hierarchy. Saved layers
can overlap and are still ordered presentation rules within the active Lens.

The lower-right layer rail now has a visible hover response and a full
untruncated label for the current layer. Hovering a rail segment shows that
segment's full name. Clicking either a segment or the current-layer header
opens the existing layer editor.

Card resizing now limits resize initiation to the painted lower-right handle.
The pointer changes to the diagonal resize cursor only there. Normal card body
dragging keeps the move cursor. Top-left anchoring remains the default; holding
Option changes the anchor to the center for the live resize. During a resize,
the canvas states the current and prospective automatic layer. After a save,
the notice states whether the automatic base layer changed. The card inspector
also lists a card's base layer and every authored layer where it remains
visible as context.

Relationship lines are now always available on the map. The non-functional
top-bar `Relationship lines` control is removed.

Source: user-directed authoring refinement, 2026-10-30. Automated visual-model
coverage exists; manual eframe evaluation remains deferred.

## 2026-10-30 — User-Directed Percent Range Controls

On 2026-10-30, the user directed that a zoom layer use the same percentage
shown in the canvas status text. A layer must therefore expose one **Active
zoom range** with a start and end percentage, and one **Fade** percentage.

`Fade: 11%` means opacity rises over the 11 percentage points immediately
before the active range starts and falls over the 11 percentage points
immediately after it ends. The active range stays fully opaque. The interface
must not use separate “Fade in” and “Fade out” controls.

The focused editor layout is:

1. Name
2. Active zoom range: `from [  ] %` `to [  ] %`
3. Fade: `[  ] %`
4. Card size, if retained as a layer-level presentation control
5. Save layer and Delete layer

The model boundary should accept and return percentage values. Conversion to a
camera multiplier belongs in the pure workspace projection, not in eframe
widgets. Legacy pairwise fade ranges require deterministic migration into one
active range and one symmetric fade amount. If an exact conversion is
impossible, migration must retain the old behavior until a person saves the
layer, rather than silently moving a layer’s visibility range.

Source: user direction in the semantic-zoom refinement session, 2026-10-30.

## 2026-10-30 — Dynamic Layer Rail

On 2026-10-30, inspection found that `semantic_scale_rail` divided every Lens
into fixed Broad, Working, and Detail bands. It chose the last layer with
positive opacity. A new saved layer starts with no fade ranges, which resolves
to positive opacity at every scale. The rail therefore appears stuck at Detail
even though it receives a fresh semantic-zoom frame each canvas paint.

The rail must render one ordered segment per defined layer, not three fixed
semantic bands. It must use the layer title and saved active percent range.
At a transition, it must emphasize the layer with higher opacity and show both
titles when opacity ties. With no saved layers, the existing camera-scale rail
may remain as a separate fallback. A no-active-layer gap must state that fact.

The pure presentation projection should give the app the layer ID, title,
active percent range, fade percent, and resolved opacity. The app must derive
rail segments only from that projection. Creation, save, undo, redo, and Lens
selection must all obtain the next rail from the same projection.

Focused tests:

- Adding a fourth layer changes the rail segment count and preserves order.
- A newly created layer has a finite default range and does not become active
  at every zoom percentage.
- The rail emphasis follows projection opacity at the fade boundary.
- Undo and redo restore the rail’s segment count and active label.

Source: `crates/app/src/main.rs` and `crates/canvas/src/workspace.rs` inspected
on 2026-10-30.

## 2026-10-30 — Card Layer Presence Policy

On 2026-10-30, the user directed that card size can determine a card’s layer
instead of requiring manual assignment in the layer editor. This remains
Lens-local presentation policy. It must not alter the card’s type, facts,
relationships, ownership, or membership.

The card inspector should contain one compact **Layer visibility** selector:

- **Automatic (from size)** — default. Match the stored card dimensions against the
  default new-card dimensions at 100% zoom, then select the layer whose size
  interval contains that ratio.
- **Dedicated: [layer]** — select a saved layer directly from the same compact
  menu. The card uses that saved layer regardless of its size.
- **Always visible** — ignore layer filtering for this card in this Lens.

This keeps the common action direct and exposes edge-case control only when it
is required. The selected policy and dedicated target must be saved in the
Lens presentation state, with typed staged actions and deterministic
validation. Moving or resizing a card must recompute only cards using **Match
card size**.

Automatic size assignment takes precedence over historical layer membership.
Use **Dedicated** to intentionally pin a card to one layer.

## 2026-10-30 — View-All Recovery Mode

The top bar provides **View all** whenever the active Lens has saved zoom
layers. It temporarily shows the complete Lens dataset: every in-Lens card and
relationship remains available regardless of the current zoom layer. The active
state uses **Use layers** and a blue emphasis. The canvas status says
`Showing all cards` while this state is active.

This is a temporary viewing choice. It does not change saved layer ranges,
card-size policies, card positions, relationships, or the selected Lens. The
layer rail remains visible as orientation and still opens layer editing.

When a relationship connects cards from different layers, both endpoint cards
must remain visible while either endpoint’s layer is active. Relationship
projection must perform this endpoint closure without changing the underlying
relationship fact. The UI should describe this as context, not membership.

Focused tests:

- 100%-baseline card dimensions map to the expected automatic layer.
- Resize crosses a size boundary and changes only automatic policy.
- Dedicated and always-visible policies resist size changes.
- A cross-layer relationship makes both endpoints visible in either endpoint
  layer, without changing graph records.
- Removing a layer gives a deterministic reassignment or validation result for
  a dedicated-card policy.

Source: user direction in the semantic-zoom refinement session, 2026-10-30.

## 2026-10-30 — Planned App-to-Model Seam

The app should consume one typed layer choice/query record with ID, title,
active start percent, active end percent, fade percent, and card-scale or
size-band data. It should stage one typed layer-settings update, one card
layer-visibility-policy update, and normal resize actions. It should not
reimplement percent conversion, auto-layer selection, endpoint closure, or
layer deletion behavior.

The workspace-model implementation now provides the percent settings and card
policy through typed staged actions. The eframe candidate consumes those
actions without converting percentage values or deriving automatic card
placement in the UI. The dynamic rail derives one segment per projected layer
and names the strongest layer, or each tied layer, in its header.

Automated unit coverage verifies dynamic rail length and ordering, active and
tied labels, percent-draft values, and layer-policy labels. Manual eframe
evaluation remains deferred.
