# Project Memory

On 2026-10-27, this folder was created as the durable project memory for the infinite-note-canvas repository. It records dated decisions and validation evidence. It does not describe an always-current code state.

## Product

- [Product decisions](product-decisions.md) — confirmed product boundaries,
  data-model decisions, historical native-navigation requirements, and the
  selected interim eframe implementation direction.
- [Eframe candidate](eframe-candidate.md) — current project-owned eframe
  thin-client boundary, the supported `./start.sh` and `./start.sh --web`
  application entry points, browser WebAssembly evidence, and the remaining
  manual-validation limits.
- [GPUI prototype](gpui-prototype.md) — prototype gate, acceptance criteria, requested-versus-effective semantic rendering evidence, the 2026-10-28 pure `CanvasInteraction` controller with eleven deterministic tests, its corrected GPUI adapter dispatch, workspace and macOS release-window evidence, the live-session visual record of a `100%` card-level window with `10,000` concepts, `50,000` relationships, and `30 visible / 30 rendered` cards, deterministic pure-model benchmark results, validators, and the confirmed requirement that makes GPUI prototype-only: native CAD-style trackpad navigation with Shift-based pan and two-finger zoom or magnification.
- [Native macOS prototype](macos-native-prototype.md) — the `winit` and `wgpu` Metal shell, native pinch acceptance gate, controls, launch evidence, and macOS-only validation process.

## Engineering

- [Architecture](architecture.md) — historical prototype boundaries plus the
  current eframe presentation, UI-free session, and pure-canvas module seam.
- [Development](development.md) — local toolchain, verification commands, resolved dependency and Git setup evidence, standard macOS and Windows validation runners, the deterministic pure-model scale benchmark, the live-session visual validation record, and documented platform gates.
