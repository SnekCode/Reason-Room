#!/usr/bin/env python3
"""Package the portable skill with stable paths, timestamps, and permissions."""

from pathlib import Path
import sys
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo


def package_skill(destination: Path) -> None:
    skill = Path(__file__).resolve().parents[1] / "skills" / "reason-room"
    with ZipFile(destination, "w", compression=ZIP_DEFLATED) as archive:
        for source in sorted(skill.rglob("*")):
            relative = source.relative_to(skill.parent)
            if not source.is_file() or any(
                part.startswith(".") or part == "__pycache__" for part in relative.parts
            ):
                continue
            entry = ZipInfo(relative.as_posix(), date_time=(1980, 1, 1, 0, 0, 0))
            entry.create_system = 3
            entry.external_attr = 0o100644 << 16
            archive.writestr(entry, source.read_bytes(), compress_type=ZIP_DEFLATED)


if __name__ == "__main__":
    package_skill(Path(sys.argv[1]))
