# Semantic-Zoom Authoring Repair 2026-10-30

## Outcome

Three reported authoring failures have deterministic repairs in the active
`eframe` path:

- Removing a relationship also removes its identifier from every semantic zoom
  layer in the active version before validation. A saved layer can no longer
  retain a reference to a removed relationship.
- `Create zoom layer` now asks for the active zoom-range start and end in
  percent, plus one symmetric `Fade` value, before it saves a new layer. The
  first layer starts at `1%` to `65%`; the next starts at `66%` to `150%`;
  later layers start at `151%` to `409600%`. Each has a default fade of `10%`.
- Each authored lower-right detail-level rail segment is selectable. Selecting
  one opens that layer in the shared layer editor.
- Selecting a new relationship type in the relationship inspector now saves it
  at once. The normal saved notice reports the resulting type.

The new-layer percent settings are saved in the same staged mutation as layer
creation. They do not create an intermediate default-range save or an extra
history entry.

## Evidence and boundary

Deterministic workspace coverage verifies that relationship removal clears its
semantic-layer references. Session coverage verifies that initial layer range
and fade settings persist. App coverage verifies the rail selection target and
initial creation values.

`cargo fmt --check`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo build --workspace` passed on 2026-10-30. The active-guidance and
whitespace checks also passed.

Manual eframe evaluation remains deferred. The existing candidate window was
not closed or replaced. When evaluation resumes, use `./start.sh` after the
candidate can safely close. Verify relationship removal, a new layer's initial
range fields, a rail-segment click that opens the layer editor, and immediate
relationship-type saving.
