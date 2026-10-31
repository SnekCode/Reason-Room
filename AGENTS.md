# Development Instructions

## Rust Standards

- Use Rust `1.97` or later and edition `2024`.
- Keep canvas mathematics, generated data, and visibility queries pure and testable.
- The current implementation path is a project-owned `egui` plus `eframe`
  thin-client build path. Keep all `eframe` rendering and painter code inside
  `crates/app`.
- Describe this path through local module boundaries and acceptance criteria.
  Do not reference an external project name, source, assets, or UI as an agent
  requirement.
- Keep `crates/workspace-session` UI-free. It owns typed workspace actions and
  local save, backup, verification, and recovery boundaries.
- Do not create a UI element for every record in a workspace.
- Pin external dependencies to exact versions or immutable Git revisions.
- Do not add network services, telemetry, cloud storage, or Obsidian compatibility without an approved product decision.

## Application Entry Point

- `./start.sh` is the only supported application launch entry point.
- Use `./start.sh` for manual evaluation and user instructions. It creates or
  reuses a safe external evaluation workspace by default.
- Do not instruct a user to start the application with `cargo run` or an
  internal `scripts/start-*` helper. Those are implementation details, not
  supported entry points.
- An explicit `INFINITE_NOTE_CANVAS_WORKSPACE` may select another writable
  local workspace. Do not use a workspace inside this repository.

## Required Commands

Run these commands before a handoff when dependencies are available:

```sh
cargo fmt --check
cargo test --workspace
cargo build --workspace
```

Run this command when Clippy can resolve dependencies:

```sh
cargo clippy --workspace --all-targets -- -D warnings
```

## Commit Requirement

- When a build succeeds and the application can run through `./start.sh`, commit
  the completed in-scope work before you continue or hand off.
- Do not include unrelated user changes in that commit. If its scope is not
  clear, ask the user before staging it.

## Test Requirements

- Add unit tests for changed camera transformations and generated canvas data.
- Keep tests deterministic.
- Record manual eframe candidate validation in
  `docs/project-memory/eframe-candidate.md`. Record later platform evidence in
  the appropriate dated project-memory record.
