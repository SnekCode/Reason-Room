# Eframe Thin-Client Build Path

Related: [Product Decisions](product-decisions.md) · [GPUI Product-Shell
Migration](gpui-shell-migration.md) · [Agent Environment](agent-environment.md)
· [Wiki Index](index.md)

> **Historical record — 2026-10-29 eframe evidence.**
>
> On 2026-10-30, the product reset made its linked record authoritative for
> product direction. This page preserves eframe boundary evidence, but it does
> not revive Work Item `0021` status or its old acceptance gates. The reset
> handoff directs readers to [project instructions](../../AGENTS.md) for the
> eframe UI boundary.

Primary record: [Work Item 0021](../archive/work-items-2026-10-30/Testing/0021.md)

## 2026-10-29 — Selected Interim Candidate Architecture

The user selected a project-owned `egui` plus `eframe` thin-client application
for the next Infinite Note Canvas parity candidate. The build path is a visual
and interaction direction, not an external project dependency. The user
separately selected the current candidate's pointer-anchored plain-wheel zoom
and Shift-wheel, Shift-hover, and middle-button pan interaction.

The adaptation applies these architecture rules:

- `crates/app` owns all `eframe` presentation and painter code.
- `crates/workspace-session` remains UI-free and owns typed staged actions,
  local persistence, candidate verification, backup, and restoration.
- `crates/canvas` remains UI-free and owns pure camera, visibility, and
  generated-data operations.
- The `eframe` shell consumes deterministic session data and issues typed
  actions; it does not duplicate local persistence in an app-local model.
- Normal canvas navigation remains card-oriented and bounded. Aggregate map
  presentation, if retained, is explicit and non-editing.

The build path uses an engine-first, toolkit-swappable core, command-oriented
actions, data-oriented view models, and a dedicated viewport boundary. It
applies only within this local workspace-graph product boundary; it does not
add image-editor scope, services, format support, automation, or network
features.

Agents receive only these local architecture rules and stated acceptance
criteria. An external project name, source, assets, or UI is not a requirement
for implementation.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).

## 2026-10-29 — Interim Eframe Gate Completed

Work Item `0021` recorded passing automated and macOS manual evidence for its
seven interim workflows. The user accepted the eframe thin-client direction as
the product shell for this work-item gate. The recorded repair also added
Lens-local camera restoration and a confirmation before a type change hides a
selected concept in the active Lens.

The decision does not close later Milestone 2 validation for content editing,
the full relationship inspector, generic undo and redo, save recovery,
relationship overlay, or native pinch.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).

## 2026-10-29 — Title And Type Scope Superseded

The user confirmed that Work Item `0021` superseded Work Item `0003`'s
stricter historical title-and-type checklist. The chief removed `0003` from
the remaining generic-history dependency and moved the same canonical record
to Completed as supersession evidence.

Sources: [Work Item 0021](../archive/work-items-2026-10-30/Complete/0021.md) and
[Work Item 0003](../archive/work-items-2026-10-30/Complete/0003.md) (2026-10-29).

## 2026-10-29 — Post-Gate Work Scheduled

After `0021` completed, the chief scheduled bounded independent slices for
content actions in `0004`, relationship-inspector presentation in `0005`, and
save-recovery validation planning in `0008`. Their canonical records contain
the exact file scopes and acceptance criteria.

The former `0003` file appeared in an undefined `OBE` directory with an
unfinished strict acceptance checklist. The chief moved the same canonical file
to HITL for a user decision on supersession or continued implementation. Work
Item `0006` remains blocked by that decision and by `0005`.

Sources: `docs/archive/work-items-2026-10-30/Testing/0004.md`, `docs/archive/work-items-2026-10-30/Complete/0005.md`,
`docs/archive/work-items-2026-10-30/Testing/0008.md`, and `docs/archive/work-items-2026-10-30/Complete/0003.md` (2026-10-29).

## 2026-10-29 — Eframe Candidate Entered Manual Testing

The project-owned `eframe` painter shell is integrated in `crates/app`. It
uses the UI-free `WorkspaceSession` for typed staged actions and local save
safety rather than introducing an app-local persistence model. Normal canvas
navigation uses the retained pointer-anchored plain-wheel zoom and Shift-wheel,
Shift-hover, and middle-button pan paths. The canvas is card-oriented and
bounded to 750 visible painted cards; persistent relationship lines remain
absent.

Locked workspace format, test, build, strict Clippy, direct eframe validation,
the retained shell-boundary guard, and diff validation passed. The canonical
work item moved to Testing for manual macOS evidence under
`docs/tester-eframe-parity.md`. Native pinch remains a separate final
product-shell gate.

Sources: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29);
`docs/tester-eframe-parity.md` (2026-10-29).

## 2026-10-29 — Isolated Eframe Launch Was Corrected and Verified

Initial parent integration found two launcher-environment defects: the new
scripts lacked executable modes, and their eframe dependency checks rejected a
valid workspace dependency declaration. The bounded script worker corrected
both defects without changing Rust or workspace data. The launcher continues
to reject repository workspaces and prints a safe external fixture-copy action.

The parent then started the release eframe candidate successfully with a fresh
external fixture copy. Work Item `0021` returned to Testing for user evaluation
of the seven workflows and retained navigation behavior.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).

## 2026-10-29 — Eframe-First Operating Path Published

The current path is to complete the project-owned `eframe` candidate and
its dependent work items. `0021` remains the manual-evaluation gate; native
and GPUI results are historical evidence and do not complete eframe work.

The root `./start.sh` is the only supported application launch entry point. It
is responsible for selecting a safe external evaluation workspace. Internal
Cargo invocations and `scripts/start-*` helpers are not supported user launch
paths.

The queue audit returned stale native-shell items `0003`, `0004`, `0005`, and
`0008` to root `Ready` state with `0021` prerequisites. No lower-priority
implementation may start until the manual gate is complete.

Sources: `AGENTS.md`, `docs/roadmap.md`, and canonical work items `0002`,
`0003`, `0004`, `0005`, `0008`, and `0021` (2026-10-29).

## 2026-10-29 — Root Start Command Preserves the Isolated Evaluation Rule

The user requested one normal launch command. The root `start.sh` now creates
an external evaluation copy in Application Support only on first run, reuses
that copy without overwriting changes, and honors an explicit workspace path.
It delegates final path validation and launching to the hardened eframe
launcher. Repository workspaces remain excluded.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).

## 2026-10-29 — Root Start Command Passed First-Run Integration

Parent integration ran `./start.sh` without an environment variable. It
created the stable external Application Support evaluation copy, printed its
location and refresh instruction, and started the release eframe candidate.
The canonical item returned to Testing for user evaluation; the wrapper does
not overwrite the evaluation copy on later runs.

Source: `docs/archive/work-items-2026-10-30/Complete/0021.md` (2026-10-29).
