# Validation Status

This page records the scope of current evidence, not a release guarantee.
Update it when a meaningful platform or workflow check changes that scope.
Use dates and identify the tested revision or change; do not claim a behavior
was checked just because it exists in the source.

## Recorded Baseline: 2026-10-31

The local setup and MCP skill checks below were completed before or at
[`dea2f53`](https://github.com/SnekCode/Reason-Room/commit/dea2f537f37f27fcc4a081e831650e7d53605263),
which is also preserved on the [archive branch](history.md).

| Area | Evidence | Limits |
| --- | --- | --- |
| Toolchain | rustup Rust 1.97.1, rustfmt, Clippy, Wasm target, wasm-bindgen-cli 0.2.129 verified. | One existing Mac setup, not a clean-machine install matrix. |
| Automated native checks | 301 workspace tests, workspace build, strict Clippy, and formatting passed. | Tests do not establish complete interactive usability. |
| Desktop launch | `./start.sh` built and opened Reason Room on Apple Silicon, macOS 27.0.1. Visual inspection showed the example's eight cards, six relationships, controls, and inspector. | No claim of testing all file actions or older supported macOS versions. |
| Browser launch | `./start.sh --web` built and served the shared application; local browser inspection showed the example and controls. | Localhost launch, not a full browser or graphics compatibility matrix. |
| Static output | `./start.sh --web --build-only` completed and included HTML, JS, Wasm, and license. | Artifact generation does not prove every hosted interaction. |
| MCP | All six tools and 18 change kinds exercised on an isolated test file, including persisted readback, pagination, read-only write rejection, create-once behavior, and backups. | No concurrent-writer or full application/UI integration stress test. |
| GitHub CI | [Rust checks](https://github.com/SnekCode/Reason-Room/actions/runs/41234222504) and [WebAssembly site](https://github.com/SnekCode/Reason-Room/actions/runs/41234222444) passed for `dea2f53`. | Refer to current workflow runs for later commits. |

## Documentation Audit: 2026-10-31

The audit preserves the pre-audit revision on a dedicated branch, separates
current guides from planning, and replaces the old historical schema page with
a source-checked version 6 reference. Empty and layered JSON examples are
validated by the CLI. The documentation guard checks local links, heading
anchors, launch guidance, planning labels, and example validity.

The audit's local checks passed: 301 workspace tests, workspace build, strict
Clippy, formatting, and the documentation guard. Both JSON examples validated.
Focused checks demonstrated that missing link targets, missing heading anchors,
and an invalid explicit automatic layer override are rejected. A separate
source review checked the schema and user guide; it caught and corrected a
desktop/browser evaluation-workspace distinction.

The audit changes documentation and documentation checks, not application
behavior or the saved format. Historical validation notes remain on the archive
branch. Terminology standardization remains an explicit open product decision.

## Built-in Example Refresh — 2026-10-31

On this Apple Silicon Mac, Rust 1.97.1 and wasm-bindgen 0.2.129:

- Authored both documents using the real CLI MCP server over stdio, following
  the Reason Room skill's discovery, mutation and read-back workflow. Website:
  237 saved mutations, 60 cards; VSM: 356 saved mutations, 90 cards. No workspace
  JSON was handwritten. Preview counts at 12/45/120% were 6/24/60 and 9/36/90.
- Local checks passed: 305 workspace tests, workspace build, strict Clippy,
  formatting, documentation links/examples and skill package validation.
  Example tests cover progressive visibility, transition context, two spatial
  nesting levels and cross-boundary edges. Camera and draft-switch guards have
  focused regression coverage.
- Native app launched with `./start.sh`. Both File menus expose the two
  examples. Loading creates a unique external copy. Manual zooming revealed
  process and work records while retaining enclosing context.
- Browser release built through `./start.sh --web --build-only` and was served
  locally by the supported web launcher. File > Examples and the three zoom
  levels were inspected visually. Browser example loading detaches watched-file
  access; an in-flight refresh is ignored after detachment.

The VSM uses fictional metrics, including 480 cars/day, 112.5 s/car takt and a
125 s/car paint bottleneck. Neither example provisions external services.
Windows menus, mobile layouts, all graphics combinations and real cloud/factory
behavior were not validated. Browser live-refresh picker permissions were not
exercised in this example-loading check.

## Curated Website Layout — 2026-10-31

Imported the user's revised website JSON unchanged. Only placement and boundary
geometry differs from the initial example; the 60 cards, 46 relationships and
semantic zoom settings are retained. All 305 tests, the workspace build, strict
Clippy, formatting, documentation checks and browser release build passed.
Visually loaded the revised bundled example through File > Examples in the local
browser build and confirmed the new group arrangement and opening overview.
The native binary was built; this layout revision was not separately inspected
in a newly launched native app.

## Stable Relationship Routing — 2026-10-31

Routes now use the complete Lens geometry in canvas coordinates, cached by card
and boundary layout. Camera changes, viewport clipping and semantic visibility
do not invalidate them. Painting and relationship hit testing share the same
projected paths. Terminal anchors still follow displayed card sizes; the cached
bend sequence remains fixed. Layout edits invalidate the cache.

On Apple Silicon macOS with Rust 1.97.1, all 308 workspace tests passed, along
with the workspace build, strict Clippy, formatting and documentation checks.
The browser release built through `./start.sh --web --build-only`. A regression
using the manufacturing example checks identical cached routes at 42%, 43%,
43.9%, overview and detail scales, with panning. Pure routing tests cover cache
reuse, geometry invalidation, nested endpoint context and missing endpoints.
Manual local browser checks confirmed that the same manufacturing route stayed
above the Stamping / Body welding / Body buffer cards at neighboring 42% and
43% zoom, rather than jumping below them. The rebuilt native app launched through
`./start.sh`; the same route stayed above those cards while zooming through
41–43%. No workspace format change is needed;
routes are derived in memory rather than saved into workspace JSON.

## Relationship Selection Inside Cards — 2026-11-01

Relationship hit testing now follows the paint order: visible relationship paths
receive hover, primary clicks and context menus over card surfaces. Card resize
handles and relationship creation ports retain primary-pointer priority. Card
surfaces away from lines remain interactive.

On Apple Silicon macOS with Rust 1.97.1, formatting, all workspace tests, the
workspace build and strict Clippy passed. A deterministic regression uses the
manufacturing example to find a painted relationship inside an interactive card,
checks hover and context-menu targeting, and feeds an egui primary-pointer event
to verify relationship selection without a card move gesture. It also checks
card targeting away from the line.

Attempted native evaluation through `./start.sh` with an external temporary copy
of the manufacturing workspace. The launcher reused an already-running app and
explicitly reported that it retained its earlier executable image. Native UI
interaction was unavailable because Computer Use approval was denied, so the
rebuilt app's visual behavior was not manually verified by the agent. The user
subsequently confirmed that the relationship-click bug was fixed in manual
evaluation. Browser and other platform interaction checks were not performed
for this change.

## Interaction Repair Evidence — 2026-11-02

The interaction repairs below are implemented following `9f062b1`, with
integrated automated evidence on Apple Silicon macOS and Rust 1.97.1.
Formatting, all 328 workspace tests (165 app, 112 canvas, 5 CLI and 46 session),
the workspace build, strict Clippy, the documentation guard and
`git diff --check` passed. `./start.sh --web --build-only` also passed.

On November 2 the user deferred native/browser launches to their own validation
after implementation. No interactive visual result is claimed. An earlier
`./start.sh --web` attempt built the site but could not bind localhost under
the sandbox (`Operation not permitted`); the escalated retry was interrupted
before a server handle was returned. No native launch was attempted for this
change. Windows Edge, hosted GitHub Pages and real platform clipboard checks
remain deferred.

| Requested behavior | Evidence and remaining verification |
| --- | --- |
| Browser preferences access (bug 1) | Toolbar exposes Preferences on all targets; a regression renders the shared controls with embedded viewports and confirms no root Close command. Real browser interaction remains to be checked. |
| Remove Show inspector (bug 2) | Removed the toolbar control; selecting a canvas entity remains the inspector entry point. |
| Edge-port clicks preserve inspector (bug 3) | Deterministic egui press/release regression drags between ports, preserves collapsed inspector and selection, and opens the kind chooser at the drop pointer. Relationship creation commits preserve selection. |
| Boundaries between layers (bug 4) | Inspector assigns optional Lens-local `layer_id`; model regression `boundary_layer_changes_validate_roundtrip_and_clear_when_layer_deleted` checks persistence, atomic rejection of unknown layers and deletion cleanup. Drawing and hit testing use the same layer opacity; View all restores boundaries. No manual boundary reassignment check. |
| Independent Views (bug 5) | User clarification: card content shared, position and relationships local to each View; removal deletes that View's card reference. Model regression creates, changes kind, removes connections independently in cloned Views, checks shared title projection, serializes/reloads, and checks regular/semantic projection counts. App integration switches Views and restores independent search, type filters, View all and relationship rollup; session tests verify reference removal preserves other Views/shared content and undo restores the reference. Explicit CLI/MCP concept deletion remains permanent; all 5 CLI and 46 session tests passed. Interactive View workflows remain to be checked. |
| System light-mode text contrast (bug 6) | Application fixes the egui theme to Dark to match its painted dark surfaces. A luminance test verifies ordinary dim labels exceed 4.5:1 against the lightest panel. Windows Edge rendering remains untested. |
| Cross-View/version paste (bugs 7, 11) | Model regression pastes captured cards into a blank View as shared references and into another Version as independent copies, preserving source placement/content and relationships. App integration switches to a blank View and pastes a shared reference at a specified canvas point. Interactive cross-View/version workflows remain to be checked. |
| Native/local browser paste (bug 8) | Headless egui regression feeds semantic Copy, verifies a nonempty platform CopyText command, then feeds semantic Paste and physical Command/Ctrl+V fallback; each paste creates a card. This checks application/platform-event handling, not the operating system or browser clipboard integration. Native and local-browser clipboard checks remain pending. |
| Paste at pointer, avoid overlap (bug 9) | Deterministic regressions anchor pasted visual bounds at the requested cursor point, preserve copied group geometry, and escape origin/occupied-card overlap. Collision geometry includes filtered-out placements, authored layer scaling and minimum painted shells at low zoom. Interactive pointer placement remains to be checked. |
| Summary size/fade (bug 10) | `summaries_follow_display_size_independently_of_camera_scale` and `summary_fade_uses_displayed_size_continuously` cover size-based onset/fade. `painted_summary_combines_size_and_layer_fade_and_shift_border_is_amber` inspects egui paint output to verify summary alpha combines size and layer opacity below the former global detail threshold. Referenced screenshots inspected; no native/browser visual comparison. |
| Text field clipboard (bug 12) | A real headless egui TextEdit retains focus with a card selected: Paste inserts summary text, Copy returns the selected text through the platform command, the card selection remains and no card is added. Native/browser clipboard integration remains to be checked. |
| Cursor relationship-kind chooser (enhancement 1) | Both port drop and inspector Connect open a compact kind list at the pointer. Choices use the endpoint compatibility rules; Cancel and Escape dismiss it. The port event regression verifies chooser state and position. |
| Relationship parent roll-up filter (enhancement 2) | O-I filter controls a presentation-only projection. `rollup_uses_visible_ancestor_without_forcing_hidden_child_and_preserves_fade` checks two ancestor levels, child fade, collapsed self-edge suppression and unavailable representatives. App regression `rollup_routes_hidden_child_edge_from_visible_parent_in_the_app` verifies the hidden original endpoint survives pre-routing filters, the visible-parent route paints, and cached world geometry remains unchanged across pan/adjacent zoom. No manual filter check. |
| View all filter toggle (enhancement 3) | Source places the View all checkbox beside O-I in canvas filter controls and removes its toolbar control. Existing `view_all_mode_suspends_semantic_layer_filtering` covers mode behavior; the independent-View controls regression covers per-View state. No manual toggle check. |
| Chosen zoom-layer insertion (enhancement 4) | `insertion_at_a_chosen_slot_shifts_later_ranges_without_changing_widths` checks ordinal/ranges. `clicking_create_between_layers_saves_the_chosen_ordinal_and_range` sends egui pointer press/release to the second insertion control and Create layer, then reads the saved JSON to verify inserted 150–200% and shifted 200–250% ranges. |
| Reorder layer ranges (first enhancement 5) | `reorder_moves_the_dragged_range_before_target_without_changing_width` verifies the requested 150–200% layer moves above 100–150% as 50–100%, preserving the other range. UI drop invokes the same staged model action. Real pointer drag not manually checked. |
| Resize shift cue/context layer movement (second enhancement 5) | Paint-output regression verifies amber shift stroke. Preview uses the same pure layer resolver as the committed resize. `moving_to_an_adjacent_layer_resizes_about_center_and_retains_aspect_ratio` checks resizing and explicit layer assignment; `card_context_layer_action_saves_a_centered_resize_in_the_active_view` invokes the context action and reads persisted JSON. No manual resize/context-menu check. |

Additional focused passing tests: `relationship_port_gesture_preserves_inspector_and_offers_type_at_drop`,
`preferences_open_as_an_embedded_window_without_native_viewports`, and
`chrome_visuals_keep_text_and_controls_on_the_dark_palette`.
The documentation guard passed after updating the interaction guide.
No Windows, Edge, deployed GitHub Pages, or native visual checks were performed
for these changes in this entry.

## Build Timestamp — 2026-11-02

Following `2feeab0`, the top-right toolbar label embeds the app compilation time
as `Built YYYY-MM-DD HH:MM:SS UTC` for native and browser artifacts. The host
build script uses only the Rust standard library and watches app/workspace
sources, manifests and the lockfile; reopening an unchanged artifact retains
its timestamp. No timestamp depends on page-load time or the viewer's timezone.

On this Apple Silicon Mac, formatting, all 328 workspace tests, the workspace
build, strict Clippy, documentation guard and diff checks passed. The supported
`./start.sh --web --build-only` completed. Generated native and Wasm build-script
outputs both contained the formatted UTC timestamp. A separate build-script
test passed epoch, day-boundary, leap-year and century cases (`rustc --edition=2024
--test crates/app/build.rs`). No app, browser or server was launched; interactive
placement and hosted-page verification remain for the user's evaluation.

## Layers Pane Repair — 2026-11-02

Following `49a2895`, the creation form renders at the selected insertion slot
and scrolls into view. Typed numeric layer settings update on Enter or when
the field loses focus, rather than saving partial keystrokes. Numeric dragging
still updates directly. Reordering moves named layers through the existing
range slots; it no longer tries to fit a range before the first layer. For
example, moving Detail (85–200%) above Process (24–85%) gives Detail 24–85%
and Process 85–200%. Membership, fade and card size stay with the named layer.
This supersedes the earlier reorder-width acceptance criterion above.

On Apple Silicon macOS 27.0.1 with Rust 1.97.1, formatting, all 331 workspace
tests (167 app, 113 canvas, 5 CLI and 46 session), the workspace build, strict
Clippy and the browser release build passed. Deterministic UI tests check
creation-form position and persisted insertion, partial number entry followed
by blur/Enter, and pointer dragging to the first row with persisted readback.
Model checks exercise every source/destination pair across three layers,
including unequal widths, a 1% first start, the maximum end, gaps, overlaps,
round-trip serialization, reverse moves and invalid-index atomic rejection.

Launched through `INFINITE_NOTE_CANVAS_WEB_PORT=8081 ./start.sh --web` because
port 8080 was occupied. In the local in-app browser, visually confirmed the
creation form opened between the clicked neighboring rows and scrolled into
view; Cancel dismissed it. Typed `2` into Detail's End field and observed that
the saved 85% start and range summary stayed unchanged. Completed `200`, clicked
away, and observed 85–200%. Dragged Detail above Process and observed the
24–85% / 85–200% range exchange with a successful save. These checks used the
fictional manufacturing example in browser-local storage. Native interaction,
Windows/Edge, deployed hosting and a broad drag/scroll platform matrix were
not checked.

## Validation Follow-up Repairs — 2026-11-02

Following `c0cba5b`, implemented the remaining accepted validation changes:

- Preferences is a fixed, centered modal on desktop and browser, above all
  panels. Canvas/filter input and canvas keyboard shortcuts are blocked until
  Done or Escape closes it. It no longer creates a separate native viewport.
- Cross-View and cross-Version paste prompts at the pointer for Copy or
  Reference. Copy creates independent captured content; Reference shares live
  content while keeping placement, layer policy and relationships local.
  A View cannot place the same reference twice; Copy remains available.
- Paste centers the card/group at the pointer and tries nearby clear placements
  within half its width/height using an elliptical limit. If none fits, a ghost
  follows the pointer with Click to paste; clicking explicitly accepts even an
  overlap. Escape cancels without saving. Collision checks include hidden cards.
- Roll up relationships and View all use matching vertically stacked pill
  switches. Roll-up defaults On. Both states preserve card visibility; Off
  disables fallback to visible parents. This supersedes the earlier behavior
  where relationships could force hidden cards to appear when roll-up was Off.

Shared cross-Version content is persisted as optional `content_reference`
identities in workspace format 7. Versions 1–6 migrate without introducing
links. Validation rejects inconsistent linked content; copies and cloned
Versions remain independent. Existing local save/backup/verification and
undo/redo boundaries apply to the complete linked-content edit.

On Apple Silicon macOS 27.0.1 with Rust 1.97.1, all 338 workspace tests passed
(171 app, 115 canvas, 5 CLI, 47 session), with formatting, workspace build,
strict Clippy, browser release build and documentation checks. Focused checks
cover real egui pointer selection of Copy, Reference and Cancel; no save while
choosing or positioning a ghost; explicit overlap confirmation and Escape
cancellation; centered modal placement and background-input blocking; reference
serialization, bidirectional edits, independent copies, source deletion and
cloned Versions; atomic shared-content save and undo/redo across Versions;
and unchanged card visibility with roll-up On/Off. Legacy CLI projection
expectations now exclude cards previously forced visible by relationships.

Launched the local browser through
`INFINITE_NOTE_CANVAS_WEB_PORT=8081 ./start.sh --web` using the fictional
manufacturing example. Visually confirmed matching stacked switches and
unchanged visible-card count when toggling roll-up; fixed centered Preferences
above filters, no movement on dragging, blocked filter clicks, and Escape
closure; a colliding paste ghost with unchanged card count and Escape
cancellation; and a pointer-positioned Copy/Reference popup in a blank View,
followed by a successful Reference paste. Cross-Version content propagation
and undo/redo were checked automatically rather than manually in the browser.
Native OS clipboard integration, Windows/Edge, hosted deployment and broad
platform visual checks remain unverified. Light-mode contrast, the longer
layer sorting/insertion investigation and boundary text wrapping remain
separate deferred validation items.

## Relationship Isolation and Endpoint Navigation — 2026-11-02

Following `852eac4`, clicking a relationship line temporarily isolates its
painted path, labels and direction markers and offers Source/Target buttons at
the clicked world position. The shared path list also governs hit testing, so
hidden lines cannot intercept clicks. The buttons pan to the displayed
endpoint's center without changing zoom or selecting an underlying card;
rolled-up relationships use the visible parent endpoint. The buttons close
after navigation while the relationship stays selected and isolated. Clicking
its line reopens them. Clearing selection, choosing a card/boundary, or Escape
restores the other lines. Navigation does not edit the workspace or its format.

On Apple Silicon macOS 27.0.1 with Rust 1.97.1, formatting, all 340 workspace
tests (173 app, 115 canvas, 5 CLI, 47 session), the workspace build, strict
Clippy, browser release build and documentation checks passed. Deterministic
pointer tests verify line clicks inside cards open the controls at the clicked
world point, isolate the path, and preserve card/port interaction priority.
Both buttons are exercised through egui pointer events and checked for exact
endpoint centering, unchanged zoom, retained relationship selection, no card
selection and no saved-file changes. Escape restores the full path list.
Pure camera tests cover negative world coordinates and minimum/maximum zoom.
A rolled-up relationship regression checks navigation to the visible parent
without revealing its hidden child.

Launched with `INFINITE_NOTE_CANVAS_WEB_PORT=8081 ./start.sh --web`. In the local
in-app browser using the manufacturing example, visually confirmed isolation
and the two buttons at a line click, then Source navigation with zoom retained
at 12% and the relationship inspector still selected. Browser Target
navigation and broad zoom/viewport combinations were not separately verified;
they have deterministic UI coverage. Native interaction, Windows/Edge and
hosted deployment were not checked for this change.

## Remaining Evidence and Known Limits

- Browser `localStorage` failures are not fully handled. Test blocked storage,
  quota exhaustion, and recovery after a failed restore/write before relying on
  browser persistence as a robust workflow.
- Check New, Open, Save, Save As, and supported live-refresh file access at the
  final hosted URL, including a reload and browser download behavior.
- Test a fresh installation and representative browser/graphics combinations.
  The macOS app declares a minimum version; that is not evidence of a tested
  operating-system matrix.
- Human and agent editing share a file but not a coordination protocol or
  multi-operation transaction. The app's refresh deferral does not establish
  safe concurrent authoring.
- Reassess dependency license/notice requirements and security reporting setup
  before distributing a stable binary release.

## Record Future Checks

For a new meaningful result, record the date and revision, platform/tool
versions, supported launch command, observed result, and what was not checked.
Keep the current evidence summary concise. Move superseded detailed research
to a separate historical snapshot when needed, following [history](history.md).
