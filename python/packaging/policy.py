"""Policy for the Agent distribution, separate from executable product code."""

from __future__ import annotations

from importlib.metadata import distribution
from pathlib import Path
from typing import Any

from packaging.requirements import Requirement
from packaging.utils import canonicalize_name


def runtime_distributions() -> set[str]:
    """Resolve installed runtime metadata, without interpreting uv's private lock schema."""
    pending: list[tuple[str, frozenset[str]]] = [("knovum-agent", frozenset([""]))]
    visited: set[tuple[str, frozenset[str]]] = set()
    found: set[str] = set()
    while pending:
        declared, extras = pending.pop()
        name = canonicalize_name(declared)
        if (name, extras) in visited:
            continue
        visited.add((name, extras))
        found.add(name)
        for declaration in distribution(name).requires or []:
            requirement = Requirement(declaration)
            if requirement.marker is None or any(requirement.marker.evaluate({"extra": extra}) for extra in extras):
                pending.append((requirement.name, frozenset(requirement.extras | {""})))
    return found


def executable_options(platform: str) -> dict[str, Any]:
    options: dict[str, Any] = {"console": True, "exclude_binaries": True}
    if platform == "win32":
        options["manifest"] = str(Path(__file__).with_name("windows.manifest"))
    return options
