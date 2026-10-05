"""Read evidence once at its boundary; enforce byte identity and containment."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import TypeVar

from pydantic import ValidationError

from acceptance.contracts import EvidenceModel, HashedFile

Model = TypeVar("Model", bound=EvidenceModel)


class EvidenceError(ValueError):
    """Acceptance evidence does not satisfy the current contract."""


class EvidenceUnavailable(EvidenceError):
    """Required real acceptance bytes are not present."""


def load_evidence(path: Path, model: type[Model]) -> Model:
    try:
        return model.model_validate_json(path.read_bytes())
    except (OSError, ValidationError) as error:
        raise EvidenceError(f"cannot read evidence {path}: {error}") from error


def sha256_file(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def resolve_hashed_file(reference: HashedFile, base: Path, location: str) -> Path:
    relative = Path(reference.path)
    if relative.is_absolute() or ".." in relative.parts:
        raise EvidenceError(f"{location}.path must stay below its evidence directory")
    root = base.resolve()
    current = root
    for part in relative.parts:
        current /= part
        if current.is_symlink():
            raise EvidenceError(f"{location}.path cannot traverse a symbolic link")
    path = (root / relative).resolve()
    if not path.is_relative_to(root):
        raise EvidenceError(f"{location}.path is not a regular evidence file")
    if not path.is_file():
        raise EvidenceUnavailable(f"{location}.path is unavailable: {path}")
    actual = sha256_file(path)
    if actual != reference.sha256:
        raise EvidenceError(f"{location} hash mismatch: expected {reference.sha256}, got {actual}")
    return path
