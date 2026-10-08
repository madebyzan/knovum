from __future__ import annotations

import json
from pathlib import Path

import pytest

from runtime import native


def test_native_installation_preserves_notices_from_micromamba_package_cache(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    project = tmp_path / "project"
    project.mkdir()
    (project / "runtime-source.json").write_text(
        json.dumps(
            {
                "nativeSupplier": {
                    "name": "conda-forge",
                    "micromambaVersion": "2.9.0",
                    "profiles": {"resources": "base"},
                }
            }
        )
    )
    lock = project / "native-locks/base/macos-x64.json"
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
        extracted = scratch / "native-cache/pkgs/https/conda.anaconda.org/conda-forge/osx-64/python"
        (extracted / "info/licenses").mkdir(parents=True)
        (extracted / "info/index.json").write_text("{}")
        (extracted / "info/licenses/LICENSE").write_text("Python license notice")
        metadata = scratch / "python/conda-meta"
        metadata.mkdir(parents=True)
        (metadata / "python.json").write_text(
            json.dumps({**package, "files": [], "extracted_package_dir": str(extracted)})
        )
        return ""

    monkeypatch.setattr(native, "run", run)
    runtime, record = native.install_native(project, "resources", "macos-x64", scratch)
    assert (runtime / "native-notices/python/LICENSE").read_text() == "Python license notice"
    assert record["packages"][0]["url"] == package["url"]
    assert not (runtime / "conda-meta").exists()
    assert not (scratch / "native-cache").exists()


def test_windows_tensor_lock_supplies_one_cuda_abi_and_its_runtime_libraries() -> None:
    lock = json.loads((Path(__file__).resolve().parents[3] / "python/native-locks/tensor/windows-x64.json").read_text())
    packages = {package["name"]: package for package in lock["packages"]}
    assert packages["cuda-version"]["version"] == "12.8"
    for name in ("pytorch", "libtorch", "torchaudio", "torchcodec"):
        assert packages[name]["build_string"].startswith("cuda128_")
    assert packages["libcudnn"]["version"].startswith("9.")
    assert "libcublas" in packages and "cuda-cudart" in packages
    assert all(len(package["sha256"]) == 64 for package in packages.values())
