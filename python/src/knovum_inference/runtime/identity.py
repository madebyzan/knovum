from __future__ import annotations

import hashlib
from pathlib import Path

from knovum_inference.errors import server_error


def verify_runtime_digest(expected: str) -> None:
    packages_directory = Path(__file__).resolve().parents[2]
    manifest = packages_directory.parent / "runtime-manifest.json"
    try:
        actual = hashlib.sha256(manifest.read_bytes()).hexdigest()
    except OSError as error:
        raise server_error("runtime_missing", "Runtime manifest is missing") from error
    if actual != expected:
        raise server_error("runtime_mismatch", "Runtime manifest digest does not match")
