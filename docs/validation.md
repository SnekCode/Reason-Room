# Validation Status

This page records the scope of current evidence, not a release guarantee.
Update it when a meaningful platform or workflow check changes that scope.
Use dates and identify the tested revision or change; do not claim a behavior
was checked just because it exists in the source.

## Recorded Baseline: 2026-10-31

The local setup and MCP skill checks below were completed before or at
[`dea2f53`](https://github.com/SnekCode/Reason-Room/commit/dea2f537f37f27fcc4a081e831650e7d53605263),
which is also preserved on the [archive branch](history.md).

| Area | Evidence | Limits |
| --- | --- | --- |
| Toolchain | rustup Rust 1.97.1, rustfmt, Clippy, Wasm target, wasm-bindgen-cli 0.2.129 verified. | One existing Mac setup, not a clean-machine install matrix. |
| Automated native checks | 301 workspace tests, workspace build, strict Clippy, and formatting passed. | Tests do not establish complete interactive usability. |
| Desktop launch | `./start.sh` built and opened Reason Room on Apple Silicon, macOS 27.0.1. Visual inspection showed the example's eight cards, six relationships, controls, and inspector. | No claim of testing all file actions or older supported macOS versions. |
| Browser launch | `./start.sh --web` built and served the shared application; local browser inspection showed the example and controls. | Localhost launch, not a full browser or graphics compatibility matrix. |
| Static output | `./start.sh --web --build-only` completed and included HTML, JS, Wasm, and license. | Artifact generation does not prove every hosted interaction. |
| MCP | All six tools and 18 change kinds exercised on an isolated test file, including persisted readback, pagination, read-only write rejection, create-once behavior, and backups. | No concurrent-writer or full application/UI integration stress test. |
| GitHub CI | [Rust checks](https://github.com/SnekCode/Reason-Room/actions/runs/41234222504) and [WebAssembly site](https://github.com/SnekCode/Reason-Room/actions/runs/41234222444) passed for `dea2f53`. | Refer to current workflow runs for later commits. |

## Documentation Audit: 2026-10-31

The audit preserves the pre-audit revision on a dedicated branch, separates
current guides from planning, and replaces the old historical schema page with
a source-checked version 6 reference. Empty and layered JSON examples are
validated by the CLI. The documentation guard checks local links, heading
anchors, launch guidance, planning labels, and example validity.

The audit's local checks passed: 301 workspace tests, workspace build, strict
Clippy, formatting, and the documentation guard. Both JSON examples validated.
Focused checks demonstrated that missing link targets, missing heading anchors,
and an invalid explicit automatic layer override are rejected. A separate
source review checked the schema and user guide; it caught and corrected a
desktop/browser evaluation-workspace distinction.

The audit changes documentation and documentation checks, not application
behavior or the saved format. Historical validation notes remain on the archive
branch. Terminology standardization remains an explicit open product decision.

## Remaining Evidence and Known Limits

- Browser `localStorage` failures are not fully handled. Test blocked storage,
  quota exhaustion, and recovery after a failed restore/write before relying on
  browser persistence as a robust workflow.
- Check New, Open, Save, Save As, and supported live-refresh file access at the
  final hosted URL, including a reload and browser download behavior.
- Test a fresh installation and representative browser/graphics combinations.
  The macOS app declares a minimum version; that is not evidence of a tested
  operating-system matrix.
- Human and agent editing share a file but not a coordination protocol or
  multi-operation transaction. The app's refresh deferral does not establish
  safe concurrent authoring.
- Reassess dependency license/notice requirements and security reporting setup
  before distributing a stable binary release.

## Record Future Checks

For a new meaningful result, record the date and revision, platform/tool
versions, supported launch command, observed result, and what was not checked.
Keep the current evidence summary concise. Move superseded detailed research
to a separate historical snapshot when needed, following [history](history.md).
