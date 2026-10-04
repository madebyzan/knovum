"""Build-time artifact handling; never imported by the desktop worker."""

from __future__ import annotations

import hashlib
import shutil
import tarfile
import urllib.request
from pathlib import Path, PurePosixPath
from typing import Any


def digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def download(source: dict[str, Any], destination: Path) -> None:
    if not source["url"].startswith("https://"):
        raise ValueError("Build inputs require HTTPS")
    size = 0
    request = urllib.request.Request(source["url"], headers={"User-Agent": "Knovum runtime builder"})
    with urllib.request.urlopen(request, timeout=60) as response, destination.open("xb") as output:
        if not response.url.startswith("https://"):
            raise ValueError("Build input redirected outside HTTPS")
        while chunk := response.read(1024 * 1024):
            size += len(chunk)
            if size > source["size"]:
                raise ValueError("Build input exceeds its declared size")
            output.write(chunk)
    if size != source["size"] or digest(destination) != source["sha256"]:
        raise ValueError("Build input size or digest mismatch")


def extract_python(archive: Path, destination: Path) -> Path:
    with tarfile.open(archive) as source:
        seen: set[str] = set()
        for entry in source.getmembers():
            path = PurePosixPath(entry.name)
            if path.is_absolute() or ".." in path.parts or not path.parts or path.parts[0] != "python":
                raise ValueError(f"Unsafe standalone archive path: {entry.name}")
            if (
                "\\" in entry.name
                or str(path) in seen
                or not (entry.isfile() or entry.isdir() or entry.issym() or entry.islnk())
            ):
                raise ValueError(f"Invalid standalone archive entry: {entry.name}")
            if entry.islnk():
                target = PurePosixPath(entry.linkname)
                if (
                    target.is_absolute()
                    or ".." in target.parts
                    or not target.parts
                    or target.parts[0] != "python"
                    or "\\" in entry.linkname
                ):
                    raise ValueError(f"Standalone hard link must name an internal archive file: {entry.name}")
            seen.add(str(path))
        source.extractall(destination, filter="data")
    root = destination / "python"
    materialize_aliases(root)
    return root


def materialize_aliases(root: Path) -> None:
    """Retain standalone's internal file aliases without conveying filesystem links."""
    for path in sorted(root.rglob("*")):
        if not path.is_symlink():
            continue
        target = path.resolve(strict=True)
        if not target.is_relative_to(root.resolve()) or not target.is_file():
            raise ValueError(f"Standalone alias must name an internal regular file: {path}")
        path.unlink()
        shutil.copy2(target, path)


def archive_runtime(root: Path, destination: Path) -> None:
    with tarfile.open(destination, "w:gz", dereference=True) as archive:
        for path in sorted(root.rglob("*")):
            if path.is_symlink() or not (path.is_file() or path.is_dir()):
                raise ValueError(f"Runtime has a nonregular entry: {path}")
            archive.add(path, arcname=path.relative_to(root).as_posix(), recursive=False)
