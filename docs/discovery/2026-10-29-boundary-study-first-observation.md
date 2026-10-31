# Boundary Discovery Session — 2026-10-29 — Representative ART Delivery Map

## Scope

- Participant context: product originator for a large-organization mapping tool.
- Example: a de-identified ART, cloud-account, repository, pipeline,
  deployment, and security-participation map.
- Question to answer: how do the teams, accounts, and delivery path relate in
  the displayed area?
- Prototype version: local BoundaryStudy revision launched through `./start.sh`.
- Observer: participant self-report with a supplied screenshot.

## Task Observations

| Task | What the participant did or said | Hesitation or intervention | Result |
| --- | --- | --- | --- |
| Orient in the example | The participant reported that the visual arrangement did not mean anything to them yet. | No question or stated meaning was visible for either boundary. | The participant could not explain the boundary treatment. |
| Create or revise a boundary | Not observed. | The session did not establish a reason to draw or revise a boundary. | No evidence. |
| Place or move an entity | Not observed. | The session did not establish what placement would communicate. | No evidence. |
| Change boundary view | Not observed. | The session did not establish what opening or closing a boundary would mean. | No evidence. |
| Trace a crossing connection | The participant understood that the connection color changed when it entered or crossed a visual boundary. | The color cue was the only identified behavior with a clear meaning. | The crossing cue made sense as a visual signal. |
| Recover broad context | Not observed. | No focused question or meaningful boundary context was established. | No evidence. |

## Meaning Check

- Boundary meaning stated by participant: no meaning was apparent yet.
- Difference between layout and business meaning: not observed.
- Meaning of the crossing connection: color showed that a connection entered or
  crossed a visual boundary; the purpose of the relationship was not yet made
  clear.
- Terms the participant used: boundary, node, connection, and changing color.
- Terms or visual cues that caused uncertainty: the two visual boundaries had
  labels, but did not state a question, purpose, or relationship meaning.

## Evidence Summary

- What the participant completed without guidance: recognized the crossing
  color cue.
- What required explanation or correction: the visual boundaries themselves
  did not communicate a useful interpretation.
- Assumptions changed by this session: a nested visual box plus a crossing
  color cue does not, by itself, establish a meaningful organizational lens or
  boundary treatment.
- Follow-up prototype question: can an explicit question-led scope, paired
  with readable relationship meaning, help a person explain why entities and
  connections appear together before they create or revise a visual boundary?
- Decision status: no decision. This is one participant observation and needs
  a changed treatment plus further representative evidence.

## Semantic Zoom Follow-Up

After zooming closer, the participant could read the dummy connection text.
They identified Caspian as depending on another entity, Ops and Infrastructure
as operating an entity, Deployment One as supporting Development Services ART,
and Community DevSecOps as validating the Delivery Pipeline.

The participant judged short directional relationship text to be a useful
medium-zoom level of detail. The observation suggests a treatment to test:
show short relationship language at medium zoom and reserve longer
explanations for close detail. It does not select a durable semantic-zoom
policy or relationship taxonomy.

Source: user feedback and supplied screenshot received 2026-10-29; [Boundary
Discovery Protocol](../product/boundary-discovery-protocol.md).

## Directed Relationship Follow-Up

The participant reported that the short relationship labels were readable, but
the direction of a relationship remained ambiguous. For example, the nearby
context suggested that Development Services ART supported Deployment One, but
the line did not make that source-to-target direction explicit.

The participant requested that the center marker of each relationship line
become an arrow that points toward the target. The experimental response uses
a midpoint arrowhead so direction remains visible on short paths. Crossing
color remains a separate cue.

This is a treatment to test. It does not select a persisted relationship model
or establish that an arrow conveys a business fact without the accompanying
participant-facing relationship text.

Source: user feedback received 2026-10-29; [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).

## Scenario Novelty Follow-Up

The participant reported becoming familiar with the representative delivery
and account example. They requested a different scenario for later study so
that familiarity with the fixture would not be mistaken for understanding the
visual treatment.

The next treatment adds a temporary data-reporting example. It asks how
customer events reach a finance reporting workspace and which teams operate,
validate, or rely on that flow. Its source-service, ingestion-pipeline,
curated-data-product, and reporting-workspace path uses its own role wording,
visual scope, and color key.

This is a variation for discovery. It does not select a persisted Lens,
organization taxonomy, relationship model, or validation result.

Source: user feedback received 2026-10-29; [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).

## Data Reporting Scenario Detail Follow-Up

On 2026-10-29, the participant supplied the exact temporary data-reporting
fixture wording. The path was Customer Events Service feeds Ingestion
Pipeline, Ingestion Pipeline transforms Curated Data Product, and Curated Data
Product publishes to Finance Reporting Workspace.

The participant stated that Finance Analytics depends on the reporting
workspace, Data Platform Team operates the data product, Analytics Engineering
Team manages the ingestion pipeline, and Data Governance Team validates the
data product.

This wording describes one discovery fixture. It does not select a persisted
Lens, organization taxonomy, relationship model, or validation result.

Source: user feedback received 2026-10-29; [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).

## Data Reporting Answer Correction

On 2026-10-29, the participant clarified that the preceding Data Reporting
text answered the displayed Data reporting lens question. It was not only a
request to define the fixture.

The participant described the directed path as Customer Events Service feeds
Ingestion Pipeline, Ingestion Pipeline transforms Curated Data Product, and
Curated Data Product publishes to Finance Reporting Workspace. The participant
also stated that Finance Analytics depends on the workspace, Data Platform Team
operates the data product, Analytics Engineering Team manages the pipeline,
and Data Governance Team validates the data product.

The participant described the example as straightforward. This is one positive
observation that the temporary lens communicated a basic unfamiliar flow. It
does not validate complex use cases or select a persisted Lens, organization
taxonomy, relationship model, or workspace schema.

Source: user clarification and supplied screenshot received 2026-10-29;
[Data Reporting Scenario Detail Follow-Up](#data-reporting-scenario-detail-follow-up)
and [Boundary Discovery Protocol](../product/boundary-discovery-protocol.md).

## Team Participation And Visual Kind Follow-Up

The participant reported that the directed delivery path was understandable.
They described a path through the Delivery Repository, Delivery Pipeline,
Deployment One, and Northstar AWS Account. They could identify Ops and
Infrastructure ART as operating Northstar, but the displayed relationships did
not directly answer which other teams participated.

The participant also reported that Development Services ART and Deployment One
were not visually distinguishable without reading labels. They requested an
on-screen color key when category colors were added.

The next treatment supplies an ordered delivery-path answer, separate active
team roles from Caspian's affected/dependent context, and adds fixture-only
node category colors with a visible key. Its visual scope contains the delivery
path rather than teams or affected context. These are treatments to test. They
do not select a persisted taxonomy, relationship model, or membership rule.

Source: user feedback received 2026-10-29; [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).

## Release Controls Treatment

On 2026-10-29, after the positive Data Reporting basic-flow observation, the
user authorized a temporary Release Controls study. The treatment tests a
shared control relationship separately from its delivery path.

The visual scope contains a Repository, Pipeline, Deployment, and Environment.
Security Assurance validates an external Release Compliance Gate, and that
control authorizes the Deployment. Active Team roles and dependent context
remain outside the visual scope and connect with directed relationships.

This is a treatment to test. It does not select a persisted Lens,
organization taxonomy, relationship model, workspace schema, or validation
result.

Source: user authorization received 2026-10-29; [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).

## Release Controls Answer Follow-Up

On 2026-10-29, the participant answered the Release controls lens question.
They identified the first delivery-path step from Partner API Repository to
Release Pipeline, but did not state the remaining Deployment-to-Environment
steps. They described Partner API Deployment as authorized by Release
Compliance Gate, but expressed uncertainty about the direction and full
control path.

The participant counted Data Consumer Team among active teams and asked what
the affected context was affected by. This showed that the original labels did
not make the distinction between active release roles and downstream
dependency clear enough.

The refined temporary treatment states the complete delivery path, the chain
from Security Assurance Team through Release Compliance Gate to Partner API
Deployment, and the active release teams. It labels Data Consumer Team as a
downstream dependent team that is not active in the release and can be
affected through Shared AWS Environment when a deployment or environment
change occurs.

This is one participant observation. It does not validate the treatment or
select a persisted Lens, organization taxonomy, relationship model, or
workspace schema.

Source: user feedback received 2026-10-29; [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).

## Release Controls Role Clarification

On 2026-10-29, the participant clarified the active Team part of their
Release Controls answer. They explicitly named Application Delivery Team,
Platform Operations Team, and Security Assurance Team. Platform Operations
Team was therefore not missed by the participant.

The participant also listed Data Consumer Team with the active Teams and
asked what could affect the affected context. The remaining ambiguity was the
difference between an active release role and a downstream dependent Team,
and the impact path through Shared AWS Environment.

This correction remains one participant observation. It does not validate the
treatment or select a persisted Lens, organization taxonomy, relationship
model, or workspace schema.

Source: user clarification received 2026-10-29; [Release Controls Answer
Follow-Up](#release-controls-answer-follow-up); [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).

## Release Automation Correction

On 2026-10-30, the participant corrected a false premise in the Release
Controls question. Assuming that Release Pipeline and Release Compliance Gate
are automated, the Partner API Repository Owner is the only human participant
in a normal release. The pipeline executes delivery and the gate executes the
compliance control. They do not require a Team to act for each release.

This invalidates the earlier `Active teams for this release` treatment. That
treatment incorrectly made Team accountability look like human participation.
The revised temporary treatment separates the repository owner's normal action
from automated delivery and authorization. It keeps Platform Operations Team
as accountability context because that Team operates Shared AWS Environment;
it does not state that the Team acts in the normal release. Data Consumer Team
remains a downstream dependency.

The participant did not state who owns the repository, who owns or operates
the gate, or whether Security Assurance Team acts only for exceptions. The
temporary fixture does not infer those facts. This is one participant
correction. It does not select a persisted Lens, organization taxonomy,
relationship model, workspace schema, or execution policy.

Source: user correction received 2026-10-30; [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).

## Release Automation Prompt Observation

On 2026-10-30, the participant described the normal release as reaching Shared
AWS Environment through Release Pipeline after Release Compliance Gate
authorized the deployment. This identified the automated path and control, but
did not identify the human participant.

When asked directly who performs the human action that starts the release, the
participant identified Partner API Repository Owner. This is partial evidence
that the automation and human-action distinction can be recovered with a
specific prompt. It is not evidence that the current visual treatment makes
the distinction apparent without a prompt.

The participant's supplied screenshot still showed the historical Organization
Landscape interface rather than the eframe Question-led study. Therefore this
observation is domain-feedback evidence only. It is not a visual-validation
result for the revised Release Controls treatment.

Source: user responses and supplied screenshot received 2026-10-30; [Boundary
Discovery Protocol](../product/boundary-discovery-protocol.md).

## Release Automation Grouping Treatment

The broad prompt identified the automated path before the human start action.
The next temporary treatment tests whether explicit visual grouping makes the
distinction easier to see. It places Partner API Repository Owner and Partner
API Repository in a `Normal human action` scope. It places Release Pipeline,
Release Compliance Gate, Partner API Deployment, and Shared AWS Environment in
an `Automated delivery and authorization` scope. The owner-to-repository
connection leads into the automated scope.

This changes one visual hypothesis only. It does not add a persisted boundary
meaning, relationship, ownership fact, or execution policy. The treatment
requires a fresh Question-led study check before it can be evaluated.

Source: [Release Automation Prompt Observation](#release-automation-prompt-observation);
[Boundary Discovery Protocol](../product/boundary-discovery-protocol.md).

## Scenario Novelty Correction

On 2026-10-30, the participant reported that the Release Controls example had
become too familiar to be a fair test. The next temporary scenario is
Workspace provisioning. It tests the same human-action and automation
treatment with a Workspace Request Owner, Workspace Access Request,
Provisioning Workflow, Access Policy Gate, Project Workspace Access, and
Collaboration Platform.

The fixture does not claim that these entities describe the participant's
organization. It is an unfamiliar comparison scenario for the same visual
hypothesis. A fresh check must use the new fixture before its result is
recorded.

Source: user feedback received 2026-10-30; [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).

## Workspace Provisioning Answer Observation

On 2026-10-30, the participant described the unfamiliar Workspace provisioning
scenario without using the side pane. They stated that Workspace Request Owner
submits an access request, the request starts Provisioning Workflow, Access
Policy Gate authorizes the provisioned access, and Project Members depend on
that access.

The response distinguished a human action, an automated process, an automated
control, and a downstream dependency in a new subject area. The participant
did not explicitly state that Project Workspace Access becomes available in
Collaboration Platform. This is a qualified positive observation, not a
complete end-to-end path result or a product decision.

Source: user response received 2026-10-30; [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).

## Semantic-Layer Transition And Detail-Scale Treatment

On 2026-10-30, the participant named the semantic-zoom mechanic a `layer`.
They specified three overlapping presentation layers for the Account
responsibility lens:

- Organizational overview: fully present through 75%, then fades out through
  85%.
- Delivery path and shared controls: fades in from 55% through 65%, then fades
  out from 160% through 170%.
- Delivery and control detail: fades in from 140% through 150%.

The participant also reported that the previous close-detail treatment showed
no useful additional information. Enlarging cards from the medium layer did
not make a workable detail view. The revised temporary treatment compacts the
delivery/control cards, removes unrelated organizational cards after their
layer fades, and reconnects the compact path before close-detail explanations
fade in.

This records one participant-specified treatment. A fresh visual check must
confirm whether the transitions preserve orientation and whether compact cards
make the close-detail layer useful. It does not select a persisted Lens,
layer model, organizational taxonomy, or card-size policy.

Source: user feedback received 2026-10-30; [Lenses and Semantic
Zoom](../product/lenses-and-semantic-zoom.md).

## Semantic-Layer Journey Observation

On 2026-10-30, the participant observed the Account responsibility lens at
140%, 145%, and 150%. The first detail-layer revision shifted cards toward a
separate compact arrangement during its fade. The participant identified this
as unexpected movement. The revised treatment keeps each card center and its
connection geometry fixed while the card shrinks in place.

The participant reported that the revised result looked good and matched their
intended experience. This is a positive observation that the three-layer
semantic-zoom treatment can preserve orientation while moving from
organizational context to compact delivery/control detail. It is one example
and does not yet validate the ranges, card behavior, or layer treatment for
other lenses and organizational maps.

Source: participant-supplied screenshots and feedback received 2026-10-30;
[Lenses and Semantic Zoom](../product/lenses-and-semantic-zoom.md).

## Workspace Provisioning Scope Meaning Observation

On 2026-10-30, the participant described the difference between the two
labeled scopes as automated processes versus human actions. This is positive
evidence that the `Normal human action` and automated-path scopes communicate
their intended distinction in an unfamiliar scenario without further
explanation.

This is one participant observation. It does not establish a persisted
boundary meaning or a general organizational taxonomy.

Source: user response received 2026-10-30; [Boundary Discovery
Protocol](../product/boundary-discovery-protocol.md).
