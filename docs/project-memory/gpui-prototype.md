# GPUI Feasibility Prototype (Historical)

> **Historical pre-reset evidence.** This page preserves dated GPUI feasibility
> research and validation notes. It is not active product, shell, launch, or
> work-item guidance. The [2026-10-30 product reset](../wiki/product-reset-2026-10-30.md)
> supersedes its planning assumptions; use the [product roadmap](../roadmap.md)
> and [product vision](../product/vision.md) for active discovery. For manual
> application evaluation, use `./start.sh` only. Do not use legacy `cargo run`
> or `scripts/*` launch commands shown below.

## 2026-10-27: Decision Gate

GPUI is prototype-gated and not selected for production. This prototype must provide implementation and validation evidence before the desktop foundation is selected.

## 2026-10-27: Scope

The prototype generates 10,000 concept cards and stores 50,000 relationships. It must avoid creating a GPUI element for every record. It must demonstrate viewport-driven rendering and record status for macOS and Windows validation.

## 2026-10-27: Acceptance Evidence

| Requirement | macOS evidence | Windows evidence | Status |
| --- | --- | --- | --- |
| Release build completes | Passed on 2026-10-28 | Pending | macOS release build passed; Windows release build pending |
| App opens a desktop window | Passed on 2026-10-28 | Pending | Live-session screenshot shows the macOS release window visibly on screen; Windows window launch pending |
| 10,000 cards and 50,000 relationships load | Visual display passed on 2026-10-28 | Pending | Live-session screenshot reports 10,000 concepts and 50,000 relationships; Windows native validation pending |
| Mouse drag pan works | Pending | Pending | Pure camera and GPUI middle-button pan implementation, including a capture listener, verified by source review; native input validation pending |
| Pointer-anchored wheel zoom works from 25% to 400% | Pending | Pending | Pure camera and GPUI wheel-zoom implementation verified by source review; native input validation pending |
| Viewport rendering avoids 10,000 UI nodes | Visual display partial on 2026-10-28 | Pending | Live-session screenshot reports 30 visible and 30 rendered cards; this does not prove native element counts or performance |
| Card selection and selection rectangle work | Pending | Pending | Hit/rectangle query, click, Shift-click, and drag-move rectangle-selection implementation verified by source review; native input validation pending |
| Semantic zoom works | Pending | Pending | Overview clusters, cards, and detail cards with summaries implemented; native rendering validation pending |
| Native trackpad pinch works | Pending | Pending | Unproven; environment-gated |
| Sustained pan and zoom have no visible input lag | Pending | Pending | Unproven; performance test pending |
| Display scaling works at required scales | Pending | Pending | Unproven; Retina and Windows device tests pending |

## 2026-10-27: Constraints

GPUI is pre-1.0. This repository pins the crate to an exact version. The rendering adapter must stay isolated from pure graph and camera logic. No platform result may be inferred from another platform.

## 2026-10-27: Verified Bootstrap and Pure-Canvas Record

The workspace bootstrap exists. The pure `crates/canvas` module passed 11 direct, dependency-free Rust tests.

The tested pure module implements camera pan; world-to-screen and screen-to-world transforms; pointer-anchored zoom; a `25%` to `400%` zoom clamp; deterministic synthetic data for 10,000 cards and 50,000 relationships; generated summaries; uniform-grid viewport, point-hit, and normalized-rectangle queries; bounded overview clusters; and `Overview`, `Card`, and `Detail` semantic levels.

This record does not provide native GPUI validation. No GPUI build, application launch, macOS device test, or Windows device test has passed.

## 2026-10-27: Blockers

Cargo could not download GPUI. A requested `cargo fetch` did not execute because approval review returned an automatic-review `404` before command execution.

The host did not have a full Xcode installation. The Metal tools required for native macOS GPUI validation were absent.

Git metadata could not be initialized. The sandbox rejected Git repository creation under the available approval policy.

## 2026-10-27: Resolved Setup Evidence

The prior Cargo and Git setup blockers were superseded on 2026-10-27. `cargo fetch` completed, `git init` completed, `Cargo.lock` exists, and the local Cargo cache contains `gpui` version `0.2.2`.

## 2026-10-27: Implemented Application Interaction Evidence

The GPUI application source implements middle-button pan through a capture listener, pointer-anchored wheel zoom, click selection, Shift-click selection, drag-move rectangle selection, viewport culling, selected-card highlights, and three semantic render levels. The generated prototype data contains 10,000 cards and stores 50,000 relationships. Relationship lines are not rendered.

The pure canvas module passed 11 tests. `cargo fmt --check` and `git diff --check` passed.

## 2026-10-27: Bounded Semantic Rendering Evidence

Each generated concept card has a deterministic, human-readable summary. Detail cards render that summary with the existing concept fields.

The pure canvas module creates bounded overview clusters from visible cards. The GPUI adapter uses those clusters for requested `Overview` rendering.

When a requested `Card` or `Detail` render has more than `750` visible cards, the adapter intentionally changes its effective semantic level to `Overview`. It renders clustered output and instructs the user to zoom in before it renders individual cards. The user interface displays both the requested level and the effective level.

When the visible-card count is within the `750`-card bound, `Card` and `Detail` rendering preserve individual cards. The adapter preserves individual-card selection state in these bounded modes. `Detail` rendering also preserves the generated summary.

This evidence is from source review and dependency-free tests. An isolated temporary copy also passed an offline workspace `cargo check` with only GPUI's `runtime_shaders` feature enabled. Production manifests remain unchanged and retain the default GPUI renderer.

This record does not prove native rendering, native runtime behavior, performance, macOS Retina scaling, Windows scaling, Windows runtime behavior, or native trackpad pinch.

## 2026-10-27: Remaining Native Build Blocker

The GPUI application had not compiled or run as of 2026-10-27. Full Xcode and its Metal tools were absent, and GPUI shader compilation requires those tools on macOS. Therefore, no macOS or Windows acceptance result exists.

## 2026-10-27: Static GPUI Type-Check Evidence

An isolated temporary copy of the workspace enabled only GPUI's `runtime_shaders` feature and passed offline workspace `cargo check`. The temporary feature bypassed the build-time Metal shader compiler and allowed Cargo to type-check the GPUI dependency and application source.

The production Cargo manifests remained unchanged and retain the default GPUI renderer. The application interaction type errors found by the isolated check were fixed in the application source.

This is static type-check evidence only. It does not prove the default renderer builds, an application window opens, shaders compile or load, rendering works, input works, performance meets the gate, macOS Retina scaling works, Windows scaling works, native trackpad pinch works, or either macOS or Windows acceptance criterion passes. Full Xcode and native default-renderer validation remain required.

## 2026-10-28: Xcode and Metal Toolchain Component Evidence

The host reported Xcode `26.6` build `17F113`. `xcrun --find metal` resolved to `/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/metal`. `xcrun --find metallib` resolved to `/var/run/com.apple.security.cryptexd/mnt/com.apple.MobileAsset.MetalToolchain-v17.6.109.0.C81XBI/Metal.xctoolchain/usr/bin/metallib`.

The default GPUI build still did not compile its Metal shaders. The build reported that the Metal Toolchain component was missing and requested `xcodebuild -downloadComponent MetalToolchain`. The automated approval service returned `404` before that command ran. Therefore, the component installation is unproven and the default macOS renderer has not built or run.

The canvas Clippy lint was fixed. Direct canvas tests passed with 11 tests. `cargo fmt --check` passed. Package Clippy for `infinite-note-canvas-canvas` passed with warnings denied.

No default macOS release build, desktop-window launch, interactive validation, or performance measurement has passed. No Windows native build or device validation has passed. GPUI remains prototype-gated.

## 2026-10-28: User-Terminal Metal Toolchain Boundary

The user's normal Terminal reported the Metal Toolchain installed. Its `xcrun --no-cache -sdk macosx metal -v` command ran successfully. Its `xcrun --no-cache -sdk macosx --find metallib` command resolved `metallib`.

The agent sandbox cannot access that installed toolchain state. Its default-renderer GPUI build attempts still stop during shader compilation before application compilation.

Default-renderer build and runtime validation must run in the user's normal Terminal. Record those macOS results in this file. Record Windows acceptance evidence separately.

## 2026-10-28: Sandboxed Metal Toolchain Mapping Finding

The agent and user processes reported the same macOS account, home directory, and selected Xcode developer directory. The agent sandbox could execute the mounted Metal Toolchain binaries directly, but it could not persist `~/Library/Developer/Xcode/XcodeToMetalToolchainIndexMapping.plist`. Xcode then could not load the downloaded Metal Toolchain mapping in the sandbox, so sandboxed `xcrun` failed to resolve the toolchain.

The user's normal Terminal persisted and resolved the mapping correctly. Its `xcrun --no-cache -sdk macosx metal -v` command ran successfully, and its `xcrun --no-cache -sdk macosx --find metallib` command resolved `metallib`. Sandboxed `xcrun` and default-renderer build failures therefore do not provide evidence about the user's Terminal build result.

## 2026-10-28: Standard macOS User-Terminal Validation Runner

`scripts/validate-macos-gpui.sh` is the project-local standard runner for macOS GPUI validation in the user's normal Terminal. It checks the macOS, Xcode, and Metal Toolchain environment, then runs offline format, workspace test, workspace Clippy with warnings denied, and release-build checks.

The runner does not download dependencies and does not launch the application. A successful runner result does not by itself prove desktop-window launch, interactive input, performance, display scaling, native pinch, or the Windows gate. Record those results separately in this file.

## 2026-10-28: Pure Interaction Controller and Workspace Verification

Interaction decisions moved into the pure `CanvasInteraction` controller in `crates/canvas`. The controller owns camera state, selected card IDs, and private gesture state. It accepts typed `CanvasCommand` values for pan, selection, and pointer-anchored wheel zoom.

The GPUI application is a coordinate and event adapter. It converts GPUI window events to canvas-viewport coordinates, dispatches `CanvasCommand` values, requests redraws, and reads controller state for rendering. GPUI no longer owns interaction decisions.

Eleven deterministic interaction tests cover pan accumulation and termination, pointer-anchored zoom, click and Shift-click selection, empty click, below-threshold drag behavior, marquee selection, reverse-marquee normalization, Shift-marquee selection, and mismatched gesture-end isolation.

Workspace verification passed on 2026-10-28: `cargo fmt --check`, `cargo test --workspace` with `22` passing tests, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo build --workspace`.

This evidence proves pure interaction behavior and Rust workspace verification. Native GPUI pointer-event delivery remains a separate macOS and Windows validation gate. Native trackpad pinch also remains a separate validation gate.

## 2026-10-28: Interaction Adapter Correction and Release-Window Restart

The GPUI adapter had dispatched one captured mouse-move event to both pan and selection continuations during a left-button selection drag. The adapter was corrected so it dispatches captured movement only for middle-button pan. A left-button drag now dispatches one selection continuation and one redraw request.

The pure `CanvasInteraction` controller retains eleven deterministic tests. Workspace verification passed with `22` tests: `cargo fmt --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and the release workspace build.

After the interaction refactor, the macOS release window was restarted without shared-pointer automation. This records release-build and desktop-window evidence only. It does not prove native pan, wheel zoom, selection, semantic rendering, performance, display scaling, or native trackpad pinch.

Native trackpad pinch remains pending on macOS and Windows. All Windows release-build, launch, input, performance, and display-scaling requirements remain pending.

## 2026-10-28: Pure-Model Scale Benchmark Evidence

`crates/canvas/examples/scale_benchmark.rs` ran in release mode against the deterministic pure data path with `10,000` concepts and `50,000` stored relationships. Its results were: dataset generation `7.370 ms`; viewport queries total `0.634 ms` with `p99 0.009 ms`; cluster queries total `5.359 ms` with `p99 0.077 ms`; pointer-anchored zoom `0.023 ms`; pan `0.024 ms`; click selections `0.443 ms`; and marquee selections `1.149 ms`.

This is pure-model evidence. It measures deterministic data generation, viewport and cluster queries, and `CanvasInteraction` command traces. It does not measure GPU frame rate, native GPUI rendering, native event delivery, native input latency, display scaling, or sustained pan-and-zoom FPS. The native macOS and Windows performance gates remain pending.

## 2026-10-28: Standard Windows Beta-Machine Validation Runner

`scripts/validate-windows-gpui.ps1` is the standard validator for the Windows beta machine. It checks that it runs on Windows and from the repository root. It records Windows, Rust, Cargo, and installed Rust-target details in a timestamped report under the user's temporary directory.

The runner performs offline Rust format, workspace test, workspace Clippy with warnings denied, and release-build checks. It launches the built executable, waits for a bounded liveness check, and then stops only the process it started.

The runner prints the required native device-test checklist. It does not perform native pan, wheel zoom, selection, semantic rendering, trackpad or precision-touchpad gesture, display-scaling, or sustained-performance tests. A successful runner result therefore does not complete the Windows acceptance gate. Native PowerShell parsing of the script remains unverified as of 2026-10-28 because PowerShell is not installed on the macOS host.

## 2026-10-28: Live-Session Visual Launch and Culling Evidence

The live-session screenshot showed the running macOS release window visibly on screen at `100%` card level. Its interface reported `10,000` concepts, `50,000` relationships, and `30 visible / 30 rendered` cards.

Source: live-session screenshot provided on 2026-10-28.

This proves visual macOS launch and the displayed viewport-culling state only. It does not prove native input delivery, native trackpad pinch, sustained FPS, Retina display scaling, or any Windows behavior.

## 2026-10-28: CAD-Style Trackpad Navigation Requirement

The user confirmed that version one requires native CAD-style trackpad navigation, including Shift-based pan and two-finger zoom or magnification behavior. Wheel-only zoom does not satisfy this requirement.

The prototype binds wheel-scroll zoom but has no native pinch or magnify event path. Therefore, GPUI remains prototype-only. This is a confirmed product-decision boundary, not a replacement-foundation decision. No alternative desktop foundation has been selected.

## 2026-10-28: Control-Contract Alignment

The pure interaction controller now accepts a framework-neutral `PanBy` command and a pointer-anchored `ZoomAt` command with a scale factor. A future native shell can map a macOS magnify gesture or a Windows pinch gesture to this interface without encoding wheel deltas in the canvas domain.

The GPUI adapter maps Shift plus mouse movement to pan. It maps Command-click on macOS and Ctrl-click on Windows to selection extension. Middle-button drag and wheel zoom remain fallback controls. GPUI has no native pinch or magnify event path, so its wheel fallback does not satisfy the production requirement.

Workspace verification passed after this revision: `cargo fmt --check`, `cargo test --workspace` with `23` passing tests, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo build --workspace`.

## 2026-10-28: macOS Mouse-Navigation Observation

The user reported that movement around the canvas works well after the Shift-plus-mouse-movement pan revision. This is a macOS interactive observation of the GPUI prototype's mouse-navigation behavior.

This observation does not validate native trackpad pinch or magnify input, sustained frame rate, Retina scaling, or any Windows behavior. GPUI remains prototype-only because it does not expose the required native pinch or magnify event path.


## 2026-10-28: First Prototype Feasibility Gate Passed

The user accepted the first GPUI prototype as a pass after testing movement around the canvas. This passes the first feasibility gate: the Rust canvas core, viewport culling, semantic rendering, and basic macOS mouse navigation provide a viable basis for the product.

This scoped pass does not select GPUI as the production shell. Native pinch or magnify input and Windows validation remain separate requirements for the production-shell gate.

## 2026-10-29: Native macOS Interactive Validation Route

The active macOS shell is documented in `docs/project-memory/macos-native-prototype.md`. Record native macOS manual-validation evidence there, including the accepted Lens-local session camera result. This file retains GPUI feasibility history and does not duplicate native-shell evidence.

## 2026-10-29: Work Item 0001 macOS Title and Overlay Validation

The user recorded a passing macOS test for work item `0001` and described the result as “Adequate.” The window title contained the document name, state, and Lens only. Diagnostics and the `+ NEW` control appeared in a translucent dark-gray overlay at the bottom left. The user used `+ NEW` to create and cancel a draft. The overlay remained usable after a window resize.

The record does not provide the macOS version, display scale, window size, or tester identity. It does not establish coverage beyond the title, overlay placement and usability, draft create-and-cancel behavior, and resize observation described above. It does not prove broader product acceptance.
