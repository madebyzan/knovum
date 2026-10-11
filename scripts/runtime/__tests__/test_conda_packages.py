from __future__ import annotations

import json
from pathlib import Path

import pytest

from runtime import conda_packages


def test_conda_installation_preserves_notices_from_micromamba_package_cache(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    project = tmp_path / "project"
    project.mkdir()
    (project / "runtime-source.json").write_text(
        json.dumps(
            {
                "condaSupplier": {
                    "name": "conda-forge",
                    "micromambaVersion": "2.9.0",
                    "profiles": {"agent": "base"},
                }
            }
        )
    )
    lock = project / "conda-locks/base/macos-x64.json"
    lock.parent.mkdir(parents=True)
    package = {
        "name": "python",
        "version": "3.11.17",
        "sha256": "a" * 64,
        "url": "https://conda.anaconda.org/conda-forge/osx-64/python.conda",
    }
    lock.write_text(json.dumps({"target": "macos-x64", "supplier": "conda-forge", "packages": [package]}))
    scratch = tmp_path / "scratch"
    scratch.mkdir()

    def run(*args: str, **_kwargs: object) -> str:
        if args == ("micromamba", "--version"):
            return "2.9.0"
        extracted = scratch / "conda-cache/pkgs/https/conda.anaconda.org/conda-forge/osx-64/python"
        (extracted / "info/licenses").mkdir(parents=True)
        (extracted / "info/index.json").write_text("{}")
        (extracted / "info/licenses/LICENSE").write_text("Python license notice")
        metadata = scratch / "python/conda-meta"
        metadata.mkdir(parents=True)
        (metadata / "python.json").write_text(
            json.dumps({**package, "files": [], "extracted_package_dir": str(extracted)})
        )
        return ""

    monkeypatch.setattr(conda_packages, "run", run)
    runtime, inventory = conda_packages.install_conda_packages(project, "agent", "macos-x64", scratch)
    assert (runtime / "conda-notices/python/LICENSE").read_text() == "Python license notice"
    assert inventory["packages"][0]["url"] == package["url"]
    assert not (runtime / "conda-meta").exists()
    assert not (scratch / "conda-cache").exists()


def test_agent_locks_do_not_convey_tensor_inference_packages() -> None:
    root = Path(__file__).resolve().parents[3] / "python/conda-locks/base"
    for target in ("macos-x64", "macos-arm64", "windows-x64"):
        lock = json.loads((root / f"{target}.json").read_text())
        names = {package["name"] for package in lock["packages"]}
        assert not names.intersection({"pytorch", "libtorch", "torchaudio", "torchcodec", "cuda-version"})
        assert "python" in names
