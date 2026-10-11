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


def review_inputs(project: Path, root: Path) -> str:
    """Bind license decisions to target-resolved packages and the frozen build policy."""
    import hashlib
    import json
    import platform

    inputs = {
        "python": platform.python_version(),
        "pyinstaller": distribution("pyinstaller").version,
        "packages": {name: distribution(name).version for name in sorted(runtime_distributions())},
        "compiledInputs": hashlib.sha256(
            (root / "_internal/build-evidence/agent-compiled-inputs.json").read_bytes()
        ).hexdigest(),
        "files": {
            name: hashlib.sha256((project / name).read_bytes()).hexdigest()
            for name in ["uv.lock", "pyproject.toml", "packaging/agent.spec", "packaging/policy.py"]
        },
    }
    return hashlib.sha256(json.dumps(inputs, sort_keys=True).encode()).hexdigest()


def validate_license_review(project: Path, root: Path, target: str, require_approved: bool) -> None:
    import json
    from datetime import datetime
    from urllib.parse import urlparse

    review = json.loads((project / "packaging/license-review.json").read_text())
    if review.get("schemaVersion") != 1 or target not in review.get("targets", {}):
        raise ValueError("Agent license review scope is invalid")
    if review.get("reviewStatus") == "pending" and not require_approved:
        return
    approval = review["targets"][target]
    if review.get("reviewStatus") != "approved" or not isinstance(approval, dict):
        raise ValueError("Agent license review is unresolved; production packaging is blocked")
    if approval.get("reviewInputSha256") != review_inputs(project, root):
        raise ValueError("Agent license review does not match the current target dependencies")
    evidence = urlparse(approval.get("evidenceLocation", ""))
    if (
        evidence.scheme != "https"
        or not evidence.hostname
        or evidence.username
        or evidence.password
        or evidence.fragment
    ):
        raise ValueError("Agent license review needs durable HTTPS evidence")
    if not isinstance(approval.get("reviewer"), str) or not approval["reviewer"].strip():
        raise ValueError("Agent license review needs an accountable reviewer")
    reviewed = datetime.fromisoformat(approval.get("reviewedAt", ""))
    if reviewed.tzinfo is None or reviewed > datetime.now(reviewed.tzinfo):
        raise ValueError("Agent license review needs a valid review date")
