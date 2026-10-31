# Development

> **Historical pre-reset evidence.** This page preserves dated development and
> validation notes. It is not active product, shell, launch, or work-item
> guidance. The [2026-10-30 product reset](../wiki/product-reset-2026-10-30.md)
> supersedes its planning assumptions; use the [product roadmap](../roadmap.md)
> and [product vision](../product/vision.md) for active discovery. For manual
> application evaluation, use `./start.sh` only. Do not use legacy `cargo run`
> or `scripts/*` launch commands shown below.

## 2026-10-27: Toolchain Baseline

The bootstrap environment reported `rustc 1.97.1` and `cargo 1.97.1`. The workspace declares Rust `1.97` and edition `2024`.

## 2026-10-27: Historical Verification Command Record

The historical GPUI protocol listed these repository-root checks after Cargo
could resolve the exact dependency:

```sh
cargo fmt --check
cargo test --workspace
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

It also recorded this macOS-prototype launch command:

```sh
cargo run -p infinite-note-canvas
```

## 2026-10-27: Platform Validation

macOS and Windows results must be recorded in `gpui-prototype.md`. A successful build does not prove input behavior, display scaling, trackpad behavior, or rendering performance on either platform.

## 2026-10-27: Historical Bootstrap Blockers

`git init` did not complete because the sandbox rejected Git repository creation under the available approval policy. The workspace files exist, but this environment has not created Git metadata.

Cargo could not download GPUI. A requested `cargo fetch` did not execute because approval review returned an automatic-review `404` before command execution. Run the Cargo verification commands after registry access and command approval permit the fetch.

The host did not have a full Xcode installation. The Metal tools required for native macOS GPUI validation were absent. Install full Xcode and its macOS components before a native macOS GPUI build attempt.

## 2026-10-27: Resolved Bootstrap Evidence

`git init` completed. Git metadata exists in the repository.

`cargo fetch` completed. `Cargo.lock` exists and the local Cargo cache contains `gpui` version `0.2.2`.

`cargo fmt --check` and `git diff --check` passed. The pure canvas module passed 11 dependency-free tests, including generated-summary and bounded-overview-cluster coverage.

## 2026-10-27: Native GPUI Build Blocker

The GPUI application did not compile or run as of 2026-10-27. Full Xcode and the `metal` and `metallib` tools were absent. GPUI shader compilation on macOS therefore stopped before native application validation. This blocker is separate from resolved Cargo dependency and Git setup.

## 2026-10-27: Offline Static Type-Check Evidence

An isolated temporary copy of the workspace enabled only GPUI's `runtime_shaders` feature and passed offline workspace `cargo check`. This configuration avoided the absent build-time `metal` compiler long enough to type-check GPUI and the application source. The application interaction type errors reported by this check were fixed.

The production Cargo manifests were not changed. They retain the default GPUI renderer. Do not use the temporary `runtime_shaders` result as evidence that the default renderer builds or runs. Full Xcode and the native macOS default-renderer build remain required before runtime validation. Performance, Retina scaling, Windows runtime and scaling, and native trackpad pinch validation remain separate and pending.

## 2026-10-27: Verified Dependency-Free Test

The pure canvas module passed 11 direct dependency-free Rust tests. This evidence does not validate Cargo dependency resolution, GPUI compilation, native rendering, performance, macOS Retina behavior, Windows behavior, or native trackpad pinch.

## 2026-10-28: Xcode and Metal Toolchain Component Blocker

The host reported Xcode `26.6` build `17F113`. The `metal` compiler and `metallib` linker resolved through `xcrun` at the Xcode and Metal Toolchain paths recorded in `gpui-prototype.md`.

The default GPUI build reported that the Metal Toolchain component was missing. It requested this command:

```sh
xcodebuild -downloadComponent MetalToolchain
```

The automated approval service returned `404` before this command ran. Do not treat the visible tool paths as proof that the required Metal Toolchain component is installed or that default GPUI shader compilation can succeed.

The canvas Clippy lint was fixed. Direct canvas tests passed with 11 tests. `cargo fmt --check` passed. `cargo clippy -p infinite-note-canvas-canvas --all-targets -- -D warnings` passed. Default macOS native validation and all Windows native validation remain pending.

## 2026-10-28: User-Terminal Metal Toolchain Boundary

The user's normal Terminal reported the Metal Toolchain installed. `xcrun --no-cache -sdk macosx metal -v` ran successfully. `xcrun --no-cache -sdk macosx --find metallib` resolved successfully.

The agent sandbox cannot access this installed toolchain state. Its default-renderer GPUI builds still stop during shader compilation before application compilation.

The historical protocol called for default-renderer build and runtime
validation in the user's normal Terminal and for recording results in
`gpui-prototype.md`. It did not treat sandbox build failures as evidence that
the user's Terminal could not build the default renderer.

## 2026-10-28: Sandboxed Xcode Mapping Permission Finding

The agent and user processes reported the same macOS account, home directory, and selected Xcode developer directory. The agent sandbox could execute the mounted Metal Toolchain binaries directly. It could not persist `~/Library/Developer/Xcode/XcodeToMetalToolchainIndexMapping.plist`, however. The failed mapping persistence caused sandboxed `xcrun` to fail even though the mounted toolchain binaries were executable.

The user's normal Terminal resolved the installed Metal Toolchain through `xcrun` correctly. Run native macOS GPUI validation in that Terminal context. Do not use the sandboxed Xcode component lookup or `xcrun` result as a user-Terminal result.

## 2026-10-28: Standard macOS Validation Runner

The historical protocol used `scripts/validate-macos-gpui.sh` from the
repository root in the user's normal Terminal as its macOS GPUI validation
runner.

The script checks macOS, Xcode, and the Metal Toolchain. It runs offline `cargo fmt --check`, workspace tests, workspace Clippy with warnings denied, and the workspace release build. It does not download dependencies and does not launch the application. Record the script output and separate interactive validation results in `gpui-prototype.md`.

## 2026-10-28: Post-Refactor Validation Evidence

The pure interaction refactor added eleven deterministic controller tests. The workspace then passed `cargo fmt --check`, `cargo test --workspace` with `22` passing tests, `cargo clippy --workspace --all-targets -- -D warnings`, and its release build.

The GPUI adapter duplicate-dispatch defect was corrected. Captured movement dispatches only middle-button pan. Left-button drag movement dispatches only selection continuation. A macOS release window was restarted after this refactor without shared-pointer automation.

The restart does not replace native input, performance, or display-scaling validation. Native trackpad pinch remains pending. All Windows release-build, launch, input, performance, and display-scaling requirements remain pending.

## 2026-10-28: Deterministic Pure-Model Scale Benchmark

The historical release benchmark invoked
`cargo run --release -p infinite-note-canvas-canvas --example scale_benchmark`
against the deterministic pure canvas path. It used `10,000` concepts and
`50,000` relationships. It reported: dataset generation `7.370 ms`; viewport
queries total `0.634 ms`, `p99 0.009 ms`; cluster queries total `5.359 ms`,
`p99 0.077 ms`; zoom `0.023 ms`; pan `0.024 ms`; clicks `0.443 ms`; and
marquee `1.149 ms`.

This command measures pure data and interaction logic. It does not measure GPU FPS, native GPUI rendering, native input delivery, input latency, or sustained viewport performance. Do not use these timings as proof that either macOS or Windows passes its native performance gate.

## 2026-10-28: Standard Windows Beta-Machine Validator

The historical protocol used `scripts/validate-windows-gpui.ps1` from the
repository root on the Windows beta machine as its GPUI validation runner.

The script checks its Windows environment and repository location. It records Windows, Rust, Cargo, and installed Rust-target information in a timestamped report under `$env:TEMP`. It then runs offline `cargo fmt --check`, workspace tests, workspace Clippy with warnings denied, and the workspace release build.

The script launches the release executable, waits for a bounded liveness check, and stops only the process it starts. It prints the required manual native-test checklist. It does not perform native input, display-scaling, or performance tests. Record those device-test results separately in `gpui-prototype.md`.

Native PowerShell parsing remains unverified on 2026-10-28 because PowerShell is not installed on the macOS host. Do not treat static review or the macOS safety scan as Windows PowerShell execution evidence.

## 2026-10-28: Live-Session Visual Validation Evidence

The live-session screenshot showed the macOS release window on screen at `100%` card level. The interface displayed `10,000` concepts, `50,000` relationships, and `30 visible / 30 rendered` cards.

Source: live-session screenshot provided on 2026-10-28.

Record this as visual launch and viewport-culling display evidence only. Do not use it as evidence for native input delivery, trackpad pinch, FPS, Retina scaling, or Windows behavior.

## 2026-10-29: Eframe Candidate Development Route

The 2026-10-29 implementation route selected the project-owned `eframe`
candidate. Its validation record used the standard formatting, test, build,
and Clippy checks and used `./start.sh` for manual application launch. That
entry point creates or reuses an external evaluation file.

The retired work-item process recorded eframe manual evidence in Work Item
`0021` and in `eframe-candidate.md`. The historical GPUI and native records
above do not validate the eframe candidate.

## 2026-10-30: Headless External-Agent Adapter

The local CLI now offers a standard-input/output MCP adapter. It scopes each
server process to one canonical workspace file and defaults to read-only.
`--allow-write` exposes a single validated-change tool. That tool supports the
card, relationship, boundary, and semantic-layer CRUD paths. It reuses typed
workspace-session actions and local save, backup, verification, and recovery
path. It does not write workspace JSON directly.

The adapter has no network listener and does not control a running desktop
application. This preserves the current local-only product decision. A running
desktop app does not yet reload an agent-authored workspace, so manual use must
close and reopen the app after an external-agent editing session.

Focused MCP tests cover read-only tool exposure plus safe card creation and
deletion. An automated stdio exchange also completed initialize, tool listing,
overview, and one saved card creation against an isolated temporary workspace.
Full workspace formatting, tests, build, and strict Clippy checks passed.

Source: [External Agent Support](../product/external-agent-support.md) (2026-10-30).
