"""Seal the complete frozen Agent tree after PyInstaller and platform signing."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

MANIFEST = "agent-manifest.json"


def inventory(root: Path) -> list[dict[str, Any]]:
    boundary = root.resolve(strict=True)
    entries: list[dict[str, Any]] = []
    for path in sorted(root.rglob("*")):
        if path == root / MANIFEST:
            continue
        if path.is_symlink():
            if not path.resolve(strict=True).is_relative_to(boundary):
                raise ValueError(f"Agent link escapes its bundle: {path}")
            target = path.readlink()
            if target.is_absolute():
                raise ValueError("Agent links must be relative for relocation")
            entries.append({"path": path.relative_to(root).as_posix(), "link": str(target)})
        elif path.is_file():
            with path.open("rb") as stream:
                digest = hashlib.file_digest(stream, "sha256").hexdigest()
            entries.append({"path": path.relative_to(root).as_posix(), "size": path.stat().st_size, "sha256": digest})
        elif not path.is_dir():
            raise ValueError(f"Agent contains a special file: {path}")
    if not entries or len(entries) > 16384:
        raise ValueError("Agent inventory exceeds its bound")
    return entries


def seal(root: Path, target: str) -> bytes:
    executable = root / ("knovum-agent.exe" if target == "windows-x64" else "knovum-agent")
    if not executable.is_file() or executable.is_symlink():
        raise ValueError("Agent executable is missing")
    from sbom import write_sbom

    (root / "agent-sbom.cdx.json").unlink(missing_ok=True)
    write_sbom(root, inventory(root))
    manifest = json.dumps(
        {"schemaVersion": 1, "target": target, "files": inventory(root)},
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode()
    (root / MANIFEST).write_bytes(manifest)
    return manifest


def verify(root: Path, trusted: bytes) -> None:
    if (root / MANIFEST).read_bytes() != trusted or inventory(root) != json.loads(trusted)["files"]:
        raise ValueError("Agent distribution differs from the trusted build")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--verify-against", type=Path)
    parser.add_argument("--target", choices=["macos-x64", "macos-arm64", "windows-x64"])
    options = parser.parse_args()
    if options.verify_against is not None:
        verify(options.root, options.verify_against.read_bytes())
    else:
        if options.target is None:
            parser.error("--target is required when sealing")
        seal(options.root, options.target)
