# Reason Room Domain

This glossary defines the concepts used to map organizational work, dependencies, and operating environments. It is the canonical vocabulary for workspace schema, canvas behavior, and agent-readable queries.

## Spatial Model

**Lens**:
A dynamic filtered perspective over the workspace graph. A lens determines which concepts and relationships are relevant to a question or dimension.
_Avoid_: Layer when the filtering behavior is intended

**Lens Template**:
A provided starting definition for a lens. Applying a lens template creates an editable dynamic lens; it does not impose a fixed view or a global traversal rule.
_Avoid_: Fixed layer, unchangeable view

**Lens Layout**:
The manually arranged positions of concepts within one lens. Changing a placement changes that lens only and does not alter the concept or its placements in other lenses.
_Avoid_: Global layout

**Visual Boundary**:
A named, Lens-local rectangular area that gives part of a map visual context.
It does not store member cards and does not create containment, ownership,
responsibility, or a relationship. A card may cross a boundary without
changing a workspace fact.
_Avoid_: Container when a semantic hierarchy is intended; membership group

**Semantic Zoom Layer**:
A named, user-defined presentation rule within a lens. It determines which
concepts and relationships appear at a scale, its active zoom-percent range,
and its symmetric fade distance. It does not change the underlying concepts,
relationships, or their meaning.
_Avoid_: Lens when the selected perspective is intended; containment layer
when organizational membership is intended

**Zoom Range**:
An inclusive user-facing percent interval that controls when a semantic zoom
layer is fully active. One fade value controls the same gradual transition on
both sides of this interval. It is a presentation control, not a fact about a
concept or relationship.
_Avoid_: Viewport range, organizational scope

**Card Layer Mode**:
A Lens-local presentation policy for one card. `Automatic` derives the card's
layer from its size relative to the default new-card size at 100% zoom.
`Dedicated` selects one named layer. `Always visible` ignores layer filtering.
It does not change the concept, its placement in another Lens, or a graph fact.
_Avoid_: Concept type, hierarchy, membership

**Position**:
The canvas coordinate of a concept placement within a lens layout. Position is not an intrinsic property of a concept.
_Avoid_: Concept coordinate

## Concepts

**Team**:
A group that can own or operate work and may itself be a dependency when another concept requires that group's participation, approval, expertise, or capacity.
_Avoid_: Owner group, department, squad

**Custom Type**:
A user-defined classification for a concept that extends the curated core type set. In version one, a custom type provides a name, color, and optional icon only. It does not define fields or relationship constraints.
_Avoid_: Untyped label, one-off implementation enum

**Process**:
A repeatable sequence of work that produces or changes an outcome. A process may be manual, automated, or mixed.
_Avoid_: Workflow, procedure, runbook

**Automated Process**:
A process performed by software rather than manual human effort. It remains a process and is not a separate concept type.
_Avoid_: Automation as a distinct concept type, bot

**Pipeline**:
An automated process that coordinates one or more machine-executed steps, often in response to a trigger or invocation by another process.
_Avoid_: Workflow when the process is specifically automated

**Repository**:
A source-controlled project or codebase represented as a map concept. A repository can implement or host an automated process.
_Avoid_: Repo as unstructured metadata

**Service**:
A system capability, such as Grafana or Confluence, that a process can use or depend on.
_Avoid_: Tool when the operating capability is intended

**Access**:
A non-secret authorization dependency that permits a process to use a service. Access is auditable through fields such as its owner, scope, lifecycle state, and expiration; those fields may be explicitly unknown. Access may be fulfilled by a token, service account, role, or other credential, but the workspace never stores the secret value.
_Avoid_: Token when the secret value is meant

**Workspace Version**:
A named local revision of a workspace graph. It can represent either the current state or a planning scenario, and it is not a source-control commit history.
_Avoid_: Git commit, event log

**Current State**:
The one named workspace version that represents the accepted present state of the mapped organization.
_Avoid_: Default branch, live synchronization

**Planning Scenario**:
A named alternative workspace version rooted in a selected workspace version. It is used to plan a possible change without altering the current state.
_Avoid_: Git branch, required future state

**Execution Mode**:
The way a process is performed: `manual`, `automated`, or `hybrid`.
_Avoid_: Process type, automation type

## Relationships

**Contains**:
A directed structural relationship from a concept to a concept within its organizational or system boundary. Contains does not assign ownership or operational responsibility.
_Avoid_: Owns when accountability is intended

**Owns**:
A directed relationship from a team to a concept for which that team is accountable for purpose, priority, and lifecycle.
_Avoid_: Operates when day-to-day running responsibility is intended

**Operates**:
A directed relationship from a team to a concept for which that team is responsible for running, maintaining, or executing.
_Avoid_: Owns when accountability is intended

**Dependency**:
A directed relationship from a source concept to something required for it to operate or complete. The target may be a team when the dependency is on that team's participation, approval, expertise, or capacity.
_Avoid_: Link, association

**Direct Team Dependency**:
A dependency whose target is a team because the work depends on that team's active participation, approval, expertise, or capacity. It is not used when the actual dependency is a process, pipeline, service, or access that the team manages.
_Avoid_: Team dependency when a concrete managed concept is known

**Implemented In**:
A directed relationship from a process or pipeline to the repository that contains its executable definition.
_Avoid_: Implements when the relationship direction is unclear

**Dependency Gap**:
A derived analysis result that identifies a direct team dependency without a known concrete process, pipeline, service, or access behind it. A dependency gap is not stored as relationship state and does not authorize an agent to infer a missing concept.
_Avoid_: Unresolved dependency as persisted data

**Assertion**:
A user-stated concept or relationship recorded without required evidence metadata. Notes may explain the assertion in version one.
_Avoid_: Verified fact
