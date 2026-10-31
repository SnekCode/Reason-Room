# GPUI Product-Shell Migration

Related: [Product Decisions](product-decisions.md) · [Roadmap Handoff](roadmap-handoff.md) · [Agent Environment](agent-environment.md) · [Wiki Index](index.md)

> **Historical record — GPUI experiment evidence.**
>
> On 2026-10-30, the product reset superseded this experiment as active product
> planning. This page does not authorize GPUI work, select a product shell, or
> revive the retired work-item process. See [Product
> Reset](product-reset-2026-10-30.md) and the active [project
> instructions](../../AGENTS.md).

Primary record: [Work Item 0021](../archive/work-items-2026-10-30/Testing/0021.md)

## 2026-10-29 — Requested Architecture Change Is Awaiting a Migration Boundary

The user requested a transition from the native `winit` plus `wgpu` product
shell to a GPUI product UI. This conflicts with the prior selected shell, so
the canonical work item is in HITL while the user specifies the replacement
baseline, coexistence rule, and required GPUI manual validation.

The prior native-shell decision remains the documented historical basis for
the currently active Milestone 2 work. No implementation, roadmap acceptance,
or removal of `crates/macos-app` is approved by this record.

Sources: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29);
`docs/wiki/product-decisions.md` (2026-10-28); `docs/roadmap.md`
(2026-10-29).

## 2026-10-29 — User Selected a GPUI Parity Experiment Before Final Adoption

The user selected a separate-branch GPUI candidate that must provide the
native application's user workflows and local-data safety while improving the
visual layout and controls. The experiment is a decision gate, not an
immediate replacement of the documented native product shell. The native
application remains the comparison baseline until the GPUI candidate passes
its defined automated and macOS manual validation.

The agent environment could not write `.git/index.lock`, so it could not make
the requested initial baseline commit or create the experimental branch. This
blocks safe dispatch of implementation work; it does not change the product
decision.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).

## 2026-10-29 — Eframe Thin-Client Candidate Superseded the GPUI Retest

The user selected the project-owned `eframe` thin-client build path, while
retaining the GPUI candidate's pointer-anchored zoom and pan interaction. Work
Item `0021` returned from Testing to In Progress for the `eframe` adaptation.
The GPUI candidate remains historical experiment evidence; it is not the
selected interim shell.

The new candidate keeps the existing UI-free workspace session and pure canvas
boundary. It must provide an isolated `eframe` launch route, full automated
validation, and a fresh manual parity record before any final shell decision.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29). See also
[Eframe Thin-Client Build Path](eframe-build-path.md).

## 2026-10-29 — Baseline Commit and Isolated Parity Branch Created

The Git metadata restriction was lifted. The chief recorded the pre-experiment
repository state in initial commit `b07392a` and created branch
`experiment/gpui-parity`. Work Item `0021` then moved from HITL to In Progress
on that branch. The native application remains the approved baseline while the
candidate establishes functional parity and improved UI quality.

Sources: `b07392a` (2026-10-29); `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).

## 2026-10-29 — Pinned GPUI Native-Pinch Gate

Research of the exact pinned GPUI 0.2.2 source found no public macOS magnify
event carrying the location and scale required by the application's
pointer-anchored zoom model. The GPUI candidate can receive wheel input, but
the established parity protocol does not treat wheel-only zoom as equivalent
to native two-finger pinch. The canonical work item returned to HITL for the
user to select an exact-version upgrade investigation, a changed interaction
requirement, or retention of the native shell.

Sources: `docs/gpui-pinch-capability.md` (2026-10-29);
`docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).

## 2026-10-29 — User Authorized a Limited GPUI Evaluation Gate

The user authorized a testable GPUI candidate for six workflows: moving
concept cards, creating concepts, adjusting relationships, cycling Lens views,
editing concept types, and editing concept names. This authorizes only the
interim evaluation work. It does not select a wheel-only replacement or waive
the native-pinch requirement for the final product-shell decision.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).

## 2026-10-29 — User Added Scrollable and Pannable Canvas Evaluation

The user added scrollable and pannable canvas navigation to the interim GPUI
evaluation. The candidate keeps its accepted pointer-anchored wheel zoom and
adds a separate Shift-wheel viewport-pan path alongside Shift hover-pan and
middle-button pan. This does not change the unresolved final native-pinch
product-shell gate.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).

## 2026-10-29 — GPUI Candidate Entered Manual Interim Testing

The canonical Work Item `0021` moved to Testing after the GPUI candidate
implemented the seven requested interim workflows: card movement, concept
creation, relationship adjustment, Lens cycling, type editing, name editing,
and scrollable/pannable canvas navigation. Recorded automated evidence is a
passing locked format, workspace test, workspace build, strict Clippy,
shell-script syntax, GPUI boundary guard, and diff check. The canvas test suite
now uses a deterministic inline schema-v1 source rather than a mutable user
workspace.

Manual evaluation must use an external writable copy of
`workspaces/gpui-evaluation.fixture.json` selected with
`INFINITE_NOTE_CANVAS_WORKSPACE`. Repository workspaces, including the modified
default workspace, are excluded from evaluation. The user must record all
seven workflow results and navigation evidence in the canonical Testing item
and `docs/tester-gpui-parity.md`. Native pinch remains a separate final
product-shell gate; this Testing handoff does not select GPUI or authorize
removal of `crates/macos-app`.

Sources: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29);
`docs/tester-gpui-parity.md` (2026-10-29);
`docs/wiki/agent-environment.md` (2026-10-29).

## 2026-10-29 — Automatic Overview Is Not Accepted for Normal Navigation

Manual GPUI evaluation found the automatic Overview render mode unintuitive
and disruptive. At its scale or density boundary, it replaces editable concept
cards with aggregate clusters during ordinary zoom and pan. This is not an
accepted parity behavior.

Work Item `0021` returned from Testing to active remediation. The candidate
must keep normal zoom and pan card-oriented. If aggregate clusters remain,
they must be an explicit and visibly non-editing map-overview control. The
bounded rendering rule remains necessary; it must not silently force a mode
change. This record does not alter the separate native-pinch final gate.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).

## 2026-10-29 — Overview Remediation Returned the Candidate to Retest

The GPUI candidate no longer selects aggregate Overview automatically from
zoom scale or visible-card density. Normal navigation remains card-oriented.
For a dense viewport, the candidate bounds individual rendering to 750 cards
and explains how to pan, zoom, or deliberately open the map overview.
Aggregate clusters are now opt-in through a visibly non-editing control, with
a paired return-to-cards control.

Locked format, workspace test, workspace build, strict Clippy, shell-script,
boundary, and diff gates passed. Work Item `0021` returned to Testing for a
user retest of card continuity through zoom and pan, plus the explicit
map-overview control when it is present. This does not change the separate
native-pinch final product-shell gate.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).
