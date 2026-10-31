# Documentation History

The pre-audit repository is preserved on
[`archive/pre-docs-audit-2026-10-31`](https://github.com/SnekCode/Reason-Room/tree/archive/pre-docs-audit-2026-10-31).
Its starting commit is
[`dea2f537f37f27fcc4a081e831650e7d53605263`](https://github.com/SnekCode/Reason-Room/tree/dea2f537f37f27fcc4a081e831650e7d53605263).
The branch retains the source code alongside the documents that described it.

The archive includes the former wiki, project-memory records, discovery
observations, dated research protocols, prototype visual direction, and
superseded ADRs 0001 and 0002. It also preserves the old glossary, roadmap, and
historical schema page before the audit.

Use the archive to understand past reasoning and evidence. Its commands,
capabilities, and validation statements describe that revision and may no
longer apply. The [current documentation index](README.md) identifies the
maintained guides and planning pages. The GitHub wiki is not the documentation
source for this project.

Keep the archive branch as a historical snapshot. Record future implementation
changes in current guides, and record meaningful validation results in
[validation status](validation.md). New research should be clearly dated and
identified as research rather than introduced as current usage guidance.

To inspect the archive without changing your working checkout:

```sh
git fetch origin archive/pre-docs-audit-2026-10-31
git show origin/archive/pre-docs-audit-2026-10-31:docs/project-memory/eframe-candidate.md
```
