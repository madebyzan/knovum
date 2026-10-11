from __future__ import annotations

import importlib.metadata
import tomllib
from pathlib import Path

from knovum_agent_runtime.contracts.values import GRAPH_VERSION


def test_graph_identity_matches_manifest_lock_and_installed_sdks() -> None:
    root = Path(__file__).resolve().parents[2]
    manifest = tomllib.loads((root / "pyproject.toml").read_text())
    lock = tomllib.loads((root / "uv.lock").read_text())
    dependencies = manifest["project"]["dependencies"]
    packages = {package["name"]: package["version"] for package in lock["package"]}
    expected = "knovum-agent-v1"
    for name in ("deepagents", "langgraph"):
        pin = next(value.removeprefix(f"{name}==") for value in dependencies if value.startswith(f"{name}=="))
        assert packages[name] == pin
        assert importlib.metadata.version(name) == pin
        expected += f"/{name}-{pin}"
    assert expected == GRAPH_VERSION
