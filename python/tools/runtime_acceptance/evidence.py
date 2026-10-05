from __future__ import annotations

import hashlib
import json
import math
import re
from datetime import datetime
from pathlib import Path
from typing import Any


class EvidenceError(ValueError):
    """Raised when acceptance evidence does not satisfy the versioned contract."""


class EvidenceUnavailable(EvidenceError):
    """Raised when required real acceptance bytes are not present."""


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise EvidenceError(f"cannot read JSON evidence {path}: {error}") from error
    return require_object(value, str(path))


def require_object(value: object, location: str) -> dict[str, Any]:
    if not isinstance(value, dict) or not all(isinstance(key, str) for key in value):
        raise EvidenceError(f"{location} must be an object")
    return value


def require_keys(value: dict[str, Any], location: str, required: set[str], optional: set[str] | None = None) -> None:
    allowed = required | (optional or set())
    missing = required - value.keys()
    extra = value.keys() - allowed
    if missing:
        raise EvidenceError(f"{location} is missing keys: {', '.join(sorted(missing))}")
    if extra:
        raise EvidenceError(f"{location} has unknown keys: {', '.join(sorted(extra))}")


def require_string(value: object, location: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise EvidenceError(f"{location} must be a non-empty string")
    return value


def require_text(value: object, location: str) -> str:
    if not isinstance(value, str):
        raise EvidenceError(f"{location} must be a string")
    return value


def require_timestamp(value: object, location: str) -> datetime:
    raw = require_string(value, location)
    try:
        parsed = datetime.fromisoformat(raw.replace("Z", "+00:00"))
    except ValueError as error:
        raise EvidenceError(f"{location} must be an ISO-8601 timestamp") from error
    if parsed.tzinfo is None or parsed.utcoffset() is None:
        raise EvidenceError(f"{location} must include a UTC offset")
    return parsed


def require_number(value: object, location: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise EvidenceError(f"{location} must be a number")
    number = float(value)
    if not math.isfinite(number):
        raise EvidenceError(f"{location} must be finite")
    return number


def require_list(value: object, location: str) -> list[Any]:
    if not isinstance(value, list):
        raise EvidenceError(f"{location} must be an array")
    return value


def sha256_file(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def resolve_hashed_file(reference: object, base: Path, location: str) -> Path:
    item = require_object(reference, location)
    require_keys(item, location, {"path", "sha256"})
    relative = Path(require_string(item["path"], f"{location}.path"))
    if relative.is_absolute() or ".." in relative.parts:
        raise EvidenceError(f"{location}.path must stay below its evidence directory")
    expected = require_sha256(require_string(item["sha256"], f"{location}.sha256").lower(), f"{location}.sha256")
    root = base.resolve()
    candidate = root / relative
    current = root
    for part in relative.parts:
        current /= part
        if current.is_symlink():
            raise EvidenceError(f"{location}.path cannot traverse a symbolic link")
    path = candidate.resolve()
    if not path.is_relative_to(root):
        raise EvidenceError(f"{location}.path is not a regular evidence file")
    if not path.is_file():
        raise EvidenceUnavailable(f"{location}.path is unavailable: {path}")
    actual = sha256_file(path)
    if actual != expected:
        raise EvidenceError(f"{location} hash mismatch: expected {expected}, got {actual}")
    return path


def require_sha256(value: object, location: str) -> str:
    digest = require_string(value, location)
    if not re.fullmatch(r"[0-9a-f]{64}", digest):
        raise EvidenceError(f"{location} must be lowercase SHA-256")
    return digest
