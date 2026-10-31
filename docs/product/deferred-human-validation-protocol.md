# Deferred Human Validation Protocol

**Date:** 2026-10-30\
**Status:** preparation for later observed sessions. It records no participant
result and completes no discovery milestone.

## Purpose

Use this protocol after manual evaluation resumes. It prepares short sessions
for the discovery milestones in the [Product Roadmap](../roadmap.md). It keeps
observations separate from product decisions.

The facilitator must not teach the intended answer. The observer records what
the participant does and says before recording an interpretation. The observer
must use the participant's own words for entities, relationships, and
boundaries.

The [Semantic Zoom Evaluation](../../workspaces/semantic-zoom-evaluation.fixture.json)
fixture checks projection setup. It is not a representative organization and
does not prescribe product vocabulary.

## Session Setup and Reset

1. Choose a de-identified local workspace file outside this repository.
2. Copy the selected tracked fixture to that new workspace file.
3. Set `INFINITE_NOTE_CANVAS_WORKSPACE` to the new workspace file.
4. Start the candidate with `./start.sh` from the repository root.
5. Confirm that the window is **Reason Room** in the **Reason Room Evaluation** application.
6. Record the fixture name and candidate revision in the session record.

Use a new external workspace copy for every participant and every retry. Do
not edit a tracked fixture. Do not use a workspace inside `workspaces/`. The
normal launch path creates or reuses its own external evaluation copy, but an
explicit external copy gives a session an independent reset point.

Example setup from the repository root:

```sh
session_workspace="$(mktemp "${TMPDIR:-/tmp}/infinite-note-canvas-session.XXXXXX.json")"
cp workspaces/semantic-zoom-evaluation.fixture.json "$session_workspace"
INFINITE_NOTE_CANVAS_WORKSPACE="$session_workspace" ./start.sh
```

If the candidate stops or the desktop tool shows a different application,
close only the session candidate. Make a new external workspace copy. Then run
`./start.sh` again. Record the launch failure separately from participant
evidence.

## Rules for Every Session

- Give one task at a time.
- Ask open questions. Do not name an intended interpretation.
- Do not call placement ownership, membership, or containment unless the
  participant does.
- Do not call a Lens a zoom level. A Lens is a question-led view. Zoom changes
  detail inside that view.
- Do not infer readability, discoverability, or understanding from completion
  alone.
- If a requested action is unavailable, record the product gap. Do not replace
  it with a different action and mark the task complete.
- Stop a task when the participant asks for direct help. Record the prompt and
  the help given.

## Session A — Direct Semantic-Zoom Authoring

**Goal:** learn whether a person can shape and revise a broad-to-close
explanation without treating it as a settings exercise.

Prepare a small de-identified map with one question worth answering. Do not
start from a required entity type or relationship label.

1. Ask the participant to state the question that the view should answer.
2. Ask them to create or name the broad level of the explanation.
3. Ask them to add a deeper level and choose what detail belongs there.
4. Ask them to assign a card and a connection to the level where each first
   becomes useful.
5. Ask them to set or revise a transition range.
6. Ask them to use zoom to answer the original question.
7. Ask them to change the question or the detail and revise the view.

Observe the terms they use for levels, transitions, context, and detail. Record
whether the broad context remains understandable after close detail appears.
Do not infer that a saved layer model is understood merely because the
participant completed an assignment.

## Session B — Shared Understanding Questions

**Goal:** identify questions and language that a canvas must support without
fixing an organization taxonomy.

Prepare a representative de-identified situation. It may include teams,
operational environments, paths, controls, or other local language. It must
not require those categories.

1. Ask what question the participant would bring to this map.
2. Ask what they need to see at broad, working, and close detail.
3. Ask what they would call the visible things and connections.
4. Ask which terms are facts and which are only visual context.
5. Ask what information is missing before they can answer the question.

Record the participant's exact terms and the information they requested. Do
not convert a frequent term into a required entity type or relationship kind.

## Session C — Canvas Composition

**Goal:** learn whether direct composition supports a useful map without a
form-heavy workflow or silent business meaning.

Use an example with six to twelve entities, short descriptions, and at least
one connection that crosses an optional visual grouping. Use the [Boundary
Discovery Protocol](boundary-discovery-protocol.md) for the boundary-specific
tasks.

1. Ask the participant to create or rename the first entity needed for the
   question.
2. Ask them to add a short summary when it helps another person read the map.
3. Ask them to connect two entities and explain the connection.
4. Ask them to move an entity to improve the map.
5. Ask them to place an entity near or within a visual grouping, if the
   candidate exposes that action.
6. Ask what changed in the map and what did not change in the organization.

Observe where the participant looks for the next action. Record every
intervention. Do not infer ownership, responsibility, or containment from a
move or a nested position.

## Session D — Lenses and Semantic Zoom

**Goal:** learn whether a question-led Lens and zoomed detail remain distinct
and preserve orientation.

Prepare one map that can answer at least two different questions. Select one
question-led view only when the participant can state the question in plain
language.

1. Ask the participant to explain the current Lens question.
2. Ask them to zoom from broad context to a relevant path or responsibility.
3. Ask what stayed visible and what new detail appeared.
4. Ask them to return to a familiar broad view.
5. Ask them to change the Lens and say what changed in the view.
6. Ask whether the underlying map or facts changed.

Record confusion between a Lens change and a zoom change. Record loss of
context, unexpected motion, clipped labels, unreadable cards, and unclear
connection direction as observations. Do not mark the semantic-scale rail,
fade, or card-size treatment as successful without a participant explanation.

## Session E — Real-World Sensemaking

**Goal:** test whether an unfamiliar realistic example supports explanation of
responsibility, dependency, delivery, and shared control.

Use a de-identified example with irregular density, varied label lengths, and
crossing connections. Avoid a scenario that the participant has already
memorized from earlier sessions.

1. Ask the participant to trace one dependency or delivery path.
2. Ask who participates and what each participant contributes.
3. Ask which environment, control, or context affects the path, if present.
4. Ask what changes if one part of the path changes.
5. Ask the participant to show the evidence on the canvas.

Record the route the participant follows, the evidence they cite, and where
they lose confidence. Do not treat a correct answer as proof that the visual
treatment works for other examples or people.

## Session Record

Create one dated record under `docs/discovery/` for every session. Keep direct
observations and conclusions separate.

```md
# Discovery Session — YYYY-MM-DD — short-example-name

## Setup

- Participant context: [role or experience without identifying data]
- Example and fixture: [de-identified description]
- Candidate revision:
- Workspace reset method:
- Lens question:
- Observer:

## Observations

| Task | What the participant did or said | Hesitation or intervention | Result |
| --- | --- | --- | --- |
| Author or revise a zoom journey |  |  |  |
| Define a useful question and terms |  |  |  |
| Create, connect, move, or group |  |  |  |
| Change Lens and zoom detail |  |  |  |
| Trace an unfamiliar path |  |  |  |

## Meaning and Orientation

- Meaning stated for visual grouping:
- Difference between layout and organizational fact:
- Lens question as stated by participant:
- Detail that appeared during zoom:
- Context that remained useful:
- Labels or cues that caused uncertainty:

## Evidence Summary

- Completed without guidance:
- Required explanation or correction:
- Observation only; not a product decision:
- Follow-up question:
- Decision status: no decision / candidate pattern / repeated evidence needed
```

## After the Session

1. Preserve the external workspace only when it contains useful, de-identified
   evidence.
2. Record the session before changing the product model or interface.
3. Compare the observations with the decision gates in the relevant discovery
   protocol.
4. Add a product decision only after repeated evidence supports it.

## Related Material

- [Product Roadmap](../roadmap.md)
- [Discovery Plan](discovery-plan.md)
- [Boundary Discovery Protocol](boundary-discovery-protocol.md)
- [Autonomous Roadmap Execution Boundary](../discovery/2026-10-30-autonomous-roadmap-execution.md)
- [Eframe Candidate Evidence](../project-memory/eframe-candidate.md)
