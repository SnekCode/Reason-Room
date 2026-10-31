# Unified Map Authoring 2026-10-30

## Outcome

The eframe candidate now uses one authoring panel for creating cards, visual
boundaries, and semantic-zoom layers. The top bar keeps navigation,
relationship, history, and the `Author map` switch. Card and boundary creation
no longer open separate dialogs.

Selecting a saved boundary opens its own inspector. A person can change its
name and short context, move it by dragging, resize it by dragging its
lower-right handle, use precise geometry when necessary, or remove it. These
actions use the local save and undo/redo boundary.

## Semantic Boundary

The existing Lens-local visual-boundary decision is unchanged. Movement,
resizing, editing, and deletion affect only the selected Lens's visual
composition. They do not change card placement, membership, ownership,
responsibility, hierarchy, or relationships.

The temporary Question-led study fixture has been removed from the compiled
canvas and application paths. It is not available as a user mode or a source
of production data.

## Automated Evidence

On 2026-10-30, the following passed after the change:

- `cargo fmt --check`
- `cargo test --workspace`
- `cargo build --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `bash scripts/check-active-doc-guidance.sh`
- `git diff --check`

The deterministic tests cover boundary move, resize, minimum display size,
selection target priority, local persistence, undo/redo, and no graph change.
No fresh participant visual validation is recorded. The candidate should be
evaluated later with `./start.sh` after any existing evaluation window is
closed by its user.
