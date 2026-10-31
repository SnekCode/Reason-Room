# Semantic Zoom Layer Foundation — 2026-10-30

Related: [Product Reset](product-reset-2026-10-30.md) · [Product Roadmap](../roadmap.md) · [Eframe Thin-Client Build Path](eframe-build-path.md) · [Eframe Candidate](../project-memory/eframe-candidate.md)

## 2026-10-30 — Participant-Defined Layer Rule

On 2026-10-30, the participant defined a Lens as a view and semantic zoom as
the detail changes within that view. The participant specified that every card
needs a base zoom layer: when a broad organization card has deeper detail
attached, the organization card remains rendered as context while the person
zooms into the detail.

Source: participant semantic-zoom study session (2026-10-30).

## 2026-10-30 — Saved Layer Foundation

On 2026-10-30, implementation work following baseline commit
`0ebe27b7b3a4a475ca0fe185eabb9a657dacd277` added saved semantic zoom layers
inside a Lens. The work recorded a layer title, optional fade ranges, card
size, and one base assignment per card or relationship. The design kept cards
and relationships as shared records rather than copying them into separate
scale-specific maps.

This historical entry records the implementation decision and automated
evidence from the uncommitted worktree. It does not claim a committed code
state.

Source: [ADR 0002](../adr/0002-semantic-zoom-layers-belong-to-lenses.md),
`crates/canvas/src/workspace.rs`, and
`crates/workspace-session/src/evaluation_actions.rs` inspected on 2026-10-30.

## 2026-10-30 — Validation Limit

On 2026-10-30, formatting, workspace tests, build, strict Clippy, active
documentation guidance, and whitespace checks passed for the semantic-zoom
layer work. A supported `./start.sh` launch built and ran the eframe candidate,
but desktop control attached to a separately open historical application window
with the same identity. Direct human validation of layer authoring and visual
transitions therefore remained unrecorded.

Source: [Eframe Candidate](../project-memory/eframe-candidate.md) (2026-10-30).
