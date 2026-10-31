# Historical Native Application Removal — 2026-10-30

Related: [Product Reset](product-reset-2026-10-30.md) · [Eframe Thin-Client
Build Path](eframe-build-path.md) · [Native MVP Validation](native-mvp-validation.md)
· [Script Retirement](script-retirement-2026-10-30.md) · [Wiki Index](index.md)

## 2026-10-30 — User Authorized Historical Native Source Removal

On 2026-10-30, the user authorized removal of the historical native
application after desktop inspection repeatedly attached to its window instead
of the eframe candidate. The removal deleted `crates/macos-app` from the
workspace and removed its direct Rust dependency declarations. Historical
documentation remained as dated evidence; it did not remain as a buildable or
launchable project application.

The removal also deleted the confirmed native bundle at
`dist/Infinite Note Canvas.app`. Its `Info.plist` identified bundle identifier
`com.local.infinite-note-canvas` and executable
`infinite-note-canvas-macos`. The separate `dist/Infinite Note Canvas GPUI.app`
bundle remained outside this removal because it was not the confirmed native
application target. A historical native process that had already loaded in
memory was not force-closed, to avoid discarding possible user changes.

Source: user cleanup request (2026-10-30); baseline commit `0ebe27b`
(`0ebe27b7b3a4a475ca0fe185eabb9a657dacd277`).
