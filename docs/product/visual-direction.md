# Proposed Visual Direction — Operational Atlas

**Status:** reversible design direction for prototype work. It does not select
a product visual identity, data taxonomy, or persisted boundary meaning.

**Date:** 2026-10-30

## Brief

The canvas helps a person trace how an organizational outcome happens across
teams, systems, environments, and controls. Its primary job is orientation:
start with a credible whole, approach a path or responsibility, and retain the
surrounding context while the explanation becomes more specific.

The direction below uses an **operational atlas** rather than a generic
dashboard. A map is an instrument for inspection. Its structure should be
quiet, deliberate, and legible before it is decorative. The semantic-scale
indicator is the memorable device. It tells a person what zoom is revealing
without making a Lens look like a global application mode or a settings panel.

This direction follows the product requirements for direct manipulation,
small inputs, explicit meaning, flexible language, and context at every scale.
It reflects the participant observations that a center arrow makes connection
direction clear, a visible key improves reading, and broad cards must stay as
context during closer inspection. It does not generalize a study fixture into
a required model.

Related: [Product Vision](vision.md), [Core Interaction Model](core-interaction-model.md),
[Lenses and Semantic Zoom](lenses-and-semantic-zoom.md), and
[Discovery Plan](discovery-plan.md).

## Compact Token System

Use the tokens as a prototype palette inside `crates/app`. Do not treat these
values as organization categories. A workspace-specific category color remains
an authored display choice, not a product vocabulary.

| Token | Value | Use |
| --- | --- | --- |
| Atlas ground | `#16222D` | Canvas field and non-semantic grid substrate. |
| Survey slate | `#243543` | Toolbar, inspector, and selected-card surface. |
| Paper signal | `#E8F0F5` | Primary text and high-contrast card content. |
| Quiet graphite | `#9FB2BF` | Secondary text, context cards, and inactive chrome. |
| Route blue | `#68C5E9` | Selection, direct action, and ordinary connection cue. |
| Control amber | `#F3B85B` | Shared-control emphasis and attention states. |
| Recovery red | `#EF8B8B` | Error and destructive feedback only. |

Use the platform proportional system font for all canvas and controls. Use
weight, size, and spacing rather than a second display typeface. The proposed
scale is: 20 px inspector title, 16 px card title, 13 px medium-zoom connection
label, and 12 px supporting text. The close-detail card may use 14 px body
text with a 1.35 line-height. No label needs all caps to establish hierarchy.

## Layout

The canvas stays central. The surrounding interface has three small,
purpose-specific areas. It does not use a permanent dashboard or a multi-card
control strip.

```text
+--------------------------------------------------------------------------------+
| Lens question             Broad / Working / Detail                  + Entity |
+--------------------------------------------------------------------------------+
|                                                                            |   |
|   translucent boundary title                                                | S |
|   +-------------------------------------------------------------+            | e |
|   |  Context card                  --> route label              |            | l |
|   |                                      [center arrow]          |            | e |
|   |                              Focus card                      |            | c |
|   +-------------------------------------------------------------+            | t |
|                                                                            | e |
|                                                 [Broad][Work][Detail]      | d |
|                                                 semantic scale rail         |   |
+--------------------------------------------------------------------------------+
```

- The top bar names the Lens question in plain language. It also has the
  smallest set of direct actions: create an entity, return to overview, and
  open advanced view authoring.
- The canvas owns spatial composition. Empty canvas space remains available for
  direct creation, panning, selection, moving, connecting, and placing within a
  boundary.
- The semantic scale rail floats at the lower edge of the canvas. It gives
  feedback during zoom. It is not a mandatory slider. The active band changes
  through a short fade when the zoom range changes.
- The selection inspector appears only after selection. It starts with name,
  short summary, and readable incoming or outgoing relationships. Assertions,
  type changes, and other low-frequency information stay behind explicit
  disclosure.

The top bar and inspector use left-aligned text. Canvas labels follow the
object they explain. Do not center broad sections of interface text.

## Object Language

### Cards

A card carries one primary fact: its name. At broad scale it can reduce to a
small title plate with an optional authored role cue. At working scale it shows
the title, one summary line, and a visible selected or focused state. At close
scale it can reveal the short summary and relationship explanations. It must
not grow into a form or make every stored record visible.

Broad context cards stay present at closer scales but shift to Quiet graphite,
reduced opacity, and a smaller footprint. The reduced treatment says “context”
without implying that the card no longer exists or that it has a lesser
business role.

### Connections

Every directed connection has a midpoint arrowhead. The line may use an
authored color, but arrow direction, label wording, and endpoint attachment
remain the primary meaning cues. At working scale the label is a short verb
phrase. At close scale it can cross-fade to the longer explanation. Only one
label treatment is fully opaque at a time.

### Boundaries

A boundary uses a restrained translucent fill, a deliberate outline, and a
title tab that stays attached to its perimeter. If the person supplies an
intent label, display it as a short sentence under the title, for example,
“Context for a release path.” A boundary is not a claim of ownership,
containment, or automation unless a person explicitly gives it that meaning.

### Lens and scale state

The interface states both dimensions separately: the Lens question identifies
the perspective, and the semantic scale rail identifies the detail level. The
person can use “Show overview” to return to a known broad view. This avoids
the common failure where a scale change looks like a data change.

## Accessibility and Readability Criteria

- Keep primary text and interactive controls at or above a 4.5:1 contrast ratio
  against their immediate surface. Do not convey selection, category, crossing,
  or error by color alone.
- Keep a card title readable at the smallest scale where the card remains
  individually selectable. Collapse its summary before reducing title text
  below the readable threshold.
- Show directedness with the center arrow and endpoint attachment, not only a
  text sentence or color.
- Keep keyboard focus visible on canvas actions, inspector controls, Lens
  controls, and scale-authoring controls. Make focus order follow the visual
  order.
- Honor reduced-motion settings. Semantic layer changes may cross-fade, but
  they must resolve without positional motion when reduced motion is enabled.
- Keep the color key on-screen when category color participates in the current
  answer. Its labels must wrap inside the inspector rather than clip.
- Preserve a stable spatial anchor during a scale change. Cards may change
  opacity or footprint, but their centers must not jump because a layer becomes
  active.

## Staged, Reversible Implementation Order

1. **Visual foundation.** Centralize prototype tokens and apply them to the
   canvas, selected state, inspector, relationship text, and notices. Add
   deterministic tests for any changed paint-bound or opacity calculations.
2. **Semantic scale feedback.** Add the non-persistent scale rail and a plain
   language Lens question label. Derive both only from existing camera and Lens
   state. Add tests for scale-band selection and reduced-motion state.
3. **Progressive card and connection detail.** Keep the existing context-card
   rule. Make title, summary, short relation label, and long relation
   explanation each have an explicit visibility rule. Add pure tests for every
   threshold and no-overlap transition.
4. **Selection-first inspection.** Reshape the inspector around the selected
   card’s name, summary, and readable relationships. Keep low-frequency data
   behind disclosure. Do not alter persistence or introduce mandatory fields.
5. **Boundary clarity prototype.** Add an optional display-only boundary intent
   label and test the distinction between visual grouping and an asserted
   relationship. Do not persist a boundary model until discovery evidence
   requires it.
6. **Deferred human review.** Use an unfamiliar scenario to test orientation,
   direct creation, connection direction, boundary interpretation, and layer
   authoring. Record observations before converting any prototype rule into a
   product decision.

## Self-Critique

This direction avoids a generic dark-dashboard treatment by making the canvas
an operational atlas with one clear navigational instrument: the semantic
scale rail. The surrounding chrome stays quiet. It does not add decorative
gradients, a menu of identical cards, or a fixed team/account/control visual
taxonomy. The palette is limited to reading, action, control, and recovery
roles, while user-authored colors remain separate from product chrome.
