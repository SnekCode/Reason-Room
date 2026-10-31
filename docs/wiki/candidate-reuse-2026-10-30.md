# Candidate Reuse Protection 2026-10-30

Related: [Eframe Thin-Client Build Path](eframe-build-path.md) · [Historical
Native Application Removal 2026-10-30](historical-native-removal-2026-10-30.md)
· [Product Reset 2026-10-30](product-reset-2026-10-30.md) · [Wiki Index](index.md)

## 2026-10-30 — Existing Candidate Instances Stay Untouched

On 2026-10-30, the eframe launch path detected that macOS rejected a second
instance of `Infinite Note Canvas Evaluation` with a LaunchServices spawn
error. The supported `./start.sh` path therefore gained an exact process-path
check before it builds or stages a new evaluation bundle. When the candidate is
already open, the launcher exits successfully with an explicit message. It
does not close, replace, or retarget the existing process.

The message states that the open candidate retains its existing workspace and
executable image. It tells the evaluator to close that window before a launch
that must load a rebuilt candidate or a different external workspace. This
prevents an opaque operating-system error and avoids a false claim that a new
build started.

This supersedes only the 2026-10-30 Desktop Capture Identity Repair statement
that the staged bundle is regenerated on every launch. A bundle is regenerated
only when no evaluation candidate is already open.

Source: `scripts/start-macos-eframe.sh`, observed `RBSRequestErrorDomain`
launch failure, and launcher reuse check (2026-10-30).
