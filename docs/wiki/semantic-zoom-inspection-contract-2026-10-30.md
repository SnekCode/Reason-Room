# Semantic Zoom Inspection Contract — 2026-10-30

Related: [Semantic Zoom Layer Foundation](semantic-zoom-layer-foundation-2026-10-30.md)
· [Autonomous Roadmap Execution](../discovery/2026-10-30-autonomous-roadmap-execution.md)
· [Product Roadmap](../roadmap.md)

## 2026-10-30 — Read-Only Projection Evidence

On 2026-10-30, implementation work following baseline commit
`0ebe27b7b3a4a475ca0fe185eabb9a657dacd277` added the CLI command
`semantic-zoom`. It reads a selected workspace version, Lens, and positive
finite scale. It returns a deterministic JSON record of the saved
`WorkspaceDocument::semantic_zoom_frame` projection.

The record contains only the selected version, Lens, scale, resolved card ID,
displayed bounds, opacity, card scale, and resolved relationship ID, endpoint,
and opacity. The CLI sorts cards and relationships by stable ID. It does not
reimplement projection rules and does not write the workspace file.

The tracked `workspaces/semantic-zoom-evaluation.fixture.json` uses generic
record names and overlapping fade ranges. CLI tests record byte-identical
output, broad, overlap, and close-scale context continuity, endpoint-gated
relationship visibility, deterministic invalid-input errors, and unchanged
source bytes.

This is automated projection evidence. It does not establish that people can
understand semantic zoom, author a Lens, or retain orientation. Those product
questions remain deferred until participant observation resumes.

Source: `crates/cli/src/main.rs`,
`workspaces/semantic-zoom-evaluation.fixture.json`, and workspace verification
commands inspected on 2026-10-30.
