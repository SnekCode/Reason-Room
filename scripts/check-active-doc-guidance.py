#!/usr/bin/env python3
"""Check maintained Markdown navigation and the documented workspace examples."""
from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
PLANNING = (
    "docs/roadmap.md",
    "docs/product/vision.md",
    "docs/product/core-interaction-model.md",
    "docs/product/lenses-and-semantic-zoom.md",
    "docs/product/discovery-plan.md",
)
REQUIRED = (
    "README.md", "AGENTS.md", "ARCHITECTURE.md", "GLOSSARY.md",
    "docs/README.md", "docs/installation.md", "docs/using-reason-room.md",
    "docs/workspace-schema.md", "docs/webassembly.md", "docs/history.md",
    "docs/validation.md", "docs/product/external-agent-support.md", *PLANNING,
)
LINK = re.compile(r"!?\[[^\]\n]*\]\((<[^>]+>|[^\s)]+)(?:\s+\"[^\"]*\")?\)")


def prose(source):
    """Ignore examples inside fenced blocks when checking Markdown links."""
    lines = []
    fence = None
    for line in source.splitlines():
        match = re.match(r"^\s{0,3}(`{3,}|~{3,})", line)
        if match:
            marker = match.group(1)
            if fence is None:
                fence = (marker[0], len(marker))
            elif marker[0] == fence[0] and len(marker) >= fence[1]:
                fence = None
            continue
        if fence is None:
            lines.append(line)
    return "\n".join(lines)


def heading_ids(source):
    ids = set()
    counts = {}
    for line in prose(source).splitlines():
        match = re.match(r"^\s{0,3}#{1,6}\s+(.+?)(?:\s+#+)?$", line)
        if not match:
            continue
        title = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", match.group(1))
        title = re.sub(r"<[^>]*>", "", title).lower()
        slug = re.sub(r"[^\w\-\s]", "", title)
        slug = re.sub(r"\s", "-", slug)
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        ids.add(slug if count == 0 else f"{slug}-{count}")
    return ids


def check_links(root, paths):
    errors = []
    for path in paths:
        for match in LINK.finditer(prose(path.read_text())):
            target = match.group(1).strip("<>")
            parsed = urlsplit(target)
            if parsed.scheme or parsed.netloc:
                continue
            destination = (path.parent / unquote(parsed.path)).resolve() if parsed.path else path
            if not destination.exists():
                errors.append(f"{path.relative_to(root)}: missing link target {target}")
            elif parsed.fragment and destination.suffix.lower() == ".md":
                fragment = unquote(parsed.fragment)
                if fragment not in heading_ids(destination.read_text()):
                    errors.append(f"{path.relative_to(root)}: missing heading {target}")
    return errors


def main():
    if Path.cwd().resolve() != ROOT:
        print(f"ERROR: Run this check from the repository root: {ROOT}", file=sys.stderr)
        return 1
    errors = [f"Missing maintained document: {name}" for name in REQUIRED if not (ROOT / name).is_file()]
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    paths = sorted({*ROOT.glob("*.md"), *ROOT.glob("docs/**/*.md"), *ROOT.glob("skills/**/*.md")})
    errors.extend(check_links(ROOT, paths))
    for name in PLANNING:
        if "**Planning document.**" not in (ROOT / name).read_text():
            errors.append(f"{name}: distinguish future intent with a Planning document label")
    for name in ("README.md", "AGENTS.md"):
        text = (ROOT / name).read_text()
        if "./start.sh" not in text or "only supported application" not in text:
            errors.append(f"{name}: retain the ./start.sh-only application launch guidance")
    for path in paths:
        text = path.read_text()
        if re.search(r"scripts/(new-work-item|run-work-item-agent)\.sh", text):
            errors.append(f"{path.relative_to(ROOT)}: do not revive retired work-item automation")
    cli = ROOT / "target/debug/infinite-note-canvas-cli"
    if sys.platform == "win32":
        cli = cli.with_suffix(".exe")
    if not cli.is_file():
        errors.append("Build the CLI before validating examples: cargo build --locked --package infinite-note-canvas-cli")
    else:
        for example in sorted((ROOT / "docs/examples").glob("*.json")):
            result = subprocess.run(
                [str(cli), "validate", "--workspace", str(example), "--format", "json"],
                text=True, capture_output=True,
            )
            if result.returncode:
                errors.append(f"{example.relative_to(ROOT)}: CLI rejected the documented example: {result.stderr.strip() or result.stdout.strip()}")
    if errors:
        print("\n".join(f"ERROR: {error}" for error in errors), file=sys.stderr)
        return 1
    print(f"PASS: {len(paths)} Markdown files, local links and headings, current/planned guidance, and workspace examples.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
