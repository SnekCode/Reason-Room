# Reason Room Domain

These definitions distinguish the information being mapped from its visual
presentation. Product terminology is still under review; this glossary does
not resolve the choice between existing UI and technical names. The
[user guide](docs/using-reason-room.md#understand-the-names-on-screen) explains
how those names currently relate.

## Document and Perspectives

**Workspace**:
A document describing a mapped system, its alternative states, and the
perspectives used to understand them.
_Avoid_: Treating a workspace as the visible area of the screen.

**Workspace Version**:
A named state of the mapped information, representing the current state or an
alternative scenario.
_Avoid_: Git commit, event log.

**Current State**:
The root state representing the accepted present understanding of the mapped
system.
_Avoid_: Live synchronization, default branch.

**Planning Scenario**:
An alternative state based on another state, used to explore a possible change
without changing the current state.
_Avoid_: Required future state, Git branch.

**Lens**:
A saved perspective on a mapped state, with its own selection and arrangement
of information.
_Avoid_: Zoom layer when the whole selected perspective is intended.

**Lens Template**:
A starting description for a perspective that can be adapted to the map and
the question being explored.
_Avoid_: An unchangeable view or a different underlying system.

**Lens Layout**:
The arrangement of items within a particular perspective.
_Avoid_: An intrinsic position shared by every perspective.

## Items and Presentation

**Concept**:
A meaningful item in the mapped system, such as a team, process, or service.
_Avoid_: Treating its visual appearance as its meaning.

**Card**:
A visible representation of a concept within a perspective.
_Avoid_: Treating two representations as two independent concepts.

**Position**:
An item's location within a particular arrangement.
_Avoid_: An intrinsic property of the represented item.

**Visual Boundary**:
A named rectangular area providing visual context in one perspective, without
asserting membership, ownership, or another relationship.
_Avoid_: A semantic container or membership group.

**Spatial Containment**:
One card or area being drawn inside another as part of a visual composition.
_Avoid_: An assertion of the Contains relationship based only on position.

**Semantic Zoom Layer**:
A named presentation level within a perspective that determines what appears
at different scales.
_Avoid_: A separate system state or organizational hierarchy.

**Zoom Range**:
The interval of zoom levels where a presentation layer is fully active.
_Avoid_: Organizational scope, viewport area.

**Card Layer Mode**:
The policy governing a card's layer visibility: Automatic derives a layer from
its size, while Manual selects one or more named layers.
_Avoid_: Concept type, membership, responsibility.

## Classifications

**Team**:
A group that can own or operate work, or be required for its participation,
approval, expertise, or capacity.
_Avoid_: Confusing the group with the work or capability it manages.

**Custom Type**:
A user-defined classification extending the available kinds of mapped item.
_Avoid_: A new relationship meaning or a single item's unstructured label.

**Process**:
A repeatable sequence of work that produces or changes an outcome.
_Avoid_: Assuming a process must be automated.

**Automated Process**:
A process performed by software rather than by manual human effort.
_Avoid_: Treating automation as a separate kind of mapped item.

**Pipeline**:
An automated process coordinating machine-executed steps.
_Avoid_: Assuming every process is a pipeline.

**Repository**:
A source-controlled project or codebase represented in the map.
_Avoid_: Confusing the repository with the process it implements.

**Service**:
An operating capability that other work can use or depend on.
_Avoid_: A credential granting access to that capability.

**Access**:
An authorization dependency permitting work to use a capability, described
without recording secret values.
_Avoid_: The secret token, password, or private key itself.

**Execution Mode**:
Whether a process is performed manually, automatically, or through a mixture
of both.
_Avoid_: A distinct process classification.

## Relationships and Assertions

**Relationship**:
A directed statement of how one mapped item relates to another.
_Avoid_: Inferring meaning from proximity or visual nesting alone.

**Contains**:
A structural relationship placing an item within another item's organizational
or system scope.
_Avoid_: Ownership, operational responsibility, spatial containment alone.

**Owns**:
A relationship expressing a team's accountability for an item's purpose,
priority, and lifecycle.
_Avoid_: Day-to-day operating responsibility when accountability is intended.

**Operates**:
A relationship expressing a team's responsibility for running, maintaining, or
executing an item.
_Avoid_: Assuming that operating something also establishes ownership.

**Dependency**:
A relationship from an item to something it requires to operate or complete.
_Avoid_: A directionless association.

**Direct Team Dependency**:
A dependency on a team's participation, approval, expertise, or capacity rather
than a known capability managed by that team.
_Avoid_: Substituting a team for a known process, service, or access dependency.

**Implemented In**:
A relationship from a process to the repository containing its executable
definition.
_Avoid_: Reversing the direction without explaining it.

**Dependency Gap**:
An analysis finding that a direct team dependency lacks an identified concrete
process, service, pipeline, or access dependency behind it.
_Avoid_: Treating a missing explanation as evidence for an invented item.

**Assertion**:
A statement recorded by a person about the mapped system.
_Avoid_: Assuming that recording a statement independently verifies it.
