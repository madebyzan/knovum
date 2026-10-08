from __future__ import annotations

import hashlib
import json
from pathlib import Path

import pytest
from runtime.inventory import runtime_installation_inventory, write_inventory


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_interpreter_configuration(path: Path) -> None:
    path.write_text(
        json.dumps(
            {
                "supplier": "conda-forge",
                "target": "windows-x64",
                "platform": "win-64",
                "packages": [
                    {
                        "name": "python",
                        "version": "3.11.17",
                        "build_string": "fixture",
                        "url": "https://example.test/cpython.tar.gz",
                        "sha256": "a" * 64,
                        "size": 100,
                    }
                ],
            }
        )
    )


def write_distribution(root: Path) -> None:
    packages = root / "packages"
    metadata = packages / "demo_package-1.2.3.dist-info"
    metadata.mkdir(parents=True)
    (packages / "demo_native.pyd").write_bytes(b"MZ\0\0extension")
    (metadata / "LICENSE.txt").write_text("fixture license\n")
    (metadata / "METADATA").write_text(
        "Metadata-Version: 2.4\n"
        "Name: Demo_Package\n"
        "Version: 1.2.3\n"
        "License-Expression: MIT\n"
        "Classifier: License :: OSI Approved :: MIT License\n"
    )
    (metadata / "direct_url.json").write_text(
        json.dumps(
            {
                "url": "https://github.com/example/demo.git",
                "vcs_info": {"vcs": "git", "commit_id": "b" * 40},
            }
        )
    )
    (metadata / "RECORD").write_text(
        "demo_native.pyd,,\n"
        "demo_package-1.2.3.dist-info/LICENSE.txt,,\n"
        "demo_package-1.2.3.dist-info/METADATA,,\n"
        "demo_package-1.2.3.dist-info/direct_url.json,,\n"
        "demo_package-1.2.3.dist-info/RECORD,,\n"
    )


def test_inventory_separates_cpython_source_from_runtime_installation_files(
    tmp_path: Path,
) -> None:
    root = tmp_path / "runtime"
    (root / "bin").mkdir(parents=True)
    (root / "bin/python.exe").write_bytes(b"MZ\0\0interpreter")
    (root / "LICENSE.python.txt").write_text("fixture CPython license\n")
    source_license = root / "source-notices/audio-cpp/LICENSE"
    source_license.parent.mkdir(parents=True)
    source_license.write_text("fixture audio.cpp license\n")
    voice_library = root / "compiled-libraries/voice.dll"
    voice_library.parent.mkdir(parents=True)
    voice_library.write_bytes(b"MZ\0\0audio.cpp")
    write_distribution(root)
    configuration = tmp_path / "runtime-source.json"
    write_interpreter_configuration(configuration)
    lock = tmp_path / "uv.lock"
    lock.write_text(
        "version = 1\n\n"
        "[[package]]\n"
        'name = "demo-package"\n'
        'version = "1.2.3"\n'
        'source = { registry = "https://pypi.org/simple" }\n'
        'sdist = { url = "https://example.test/demo.tar.gz", hash = "sha256:'
        + "c"
        * 64
        + '", size = 10 }\n'
        'wheels = [{ url = "https://example.test/demo.whl", hash = "sha256:'
        + "d" * 64
        + '", size = 20 }]\n'
    )
    output = tmp_path / "evidence"

    supplier = {
        "name": "standalone-tts",
        "version": "source-revision",
        "license": "Apache-2.0",
        "files": [
            {
                "url": "https://example.test/tts.py",
                "path": "packages/tts.py",
                "sha256": "e" * 64,
            }
        ],
    }
    write_inventory(root, lock, output, configuration, "windows-x64", [supplier])

    sbom = json.loads((output / "sbom.cdx.json").read_text())
    evidence = json.loads((output / "dependency-evidence.json").read_text())
    from cyclonedx.schema import SchemaVersion
    from cyclonedx.validation.json import JsonStrictValidator

    assert (
        JsonStrictValidator(SchemaVersion.V1_6).validate_str(json.dumps(sbom)) is None
    )
    components = {item["name"]: item for item in sbom["components"]}
    assert components.keys() == {"CPython", "Demo_Package", "standalone-tts"}
    assert components["CPython"]["version"] == "3.11.17"
    assert components["CPython"]["externalReferences"] == [
        {"type": "distribution", "url": "https://example.test/cpython.tar.gz"},
    ]
    assert components["Demo_Package"]["purl"] == "pkg:pypi/demo-package@1.2.3"
    assert components["Demo_Package"]["licenses"] == [{"expression": "MIT"}]
    assert (
        evidence["dependencyLockSha256"]
        == hashlib.sha256(lock.read_bytes() + configuration.read_bytes()).hexdigest()
    )
    assert evidence["cpython"]["version"] == "3.11.17"
    assert evidence["cpython"]["source"]["url"] == "https://example.test/cpython.tar.gz"
    assert evidence["runtimeInstallation"]["compiledBinaries"] == [
        "bin/python.exe",
        "compiled-libraries/voice.dll",
    ]
    assert evidence["runtimeInstallation"]["notices"][0]["path"] == "LICENSE.python.txt"
    assert {
        notice["path"]: notice["sha256"]
        for notice in evidence["runtimeInstallation"]["notices"]
    }["source-notices/audio-cpp/LICENSE"] == sha256(source_license)
    assert "interpreter" not in evidence
    assert evidence["sourceComponents"] == [supplier]
    assert components["standalone-tts"]["licenses"] == [{"expression": "Apache-2.0"}]
    distribution = evidence["distributions"][0]
    assert distribution["compiledBinaries"] == ["packages/demo_native.pyd"]
    assert distribution["notices"][0]["path"].endswith("LICENSE.txt")
    assert distribution["source"]["origin"] == {"registry": "https://pypi.org/simple"}
    assert (
        distribution["source"]["installedDirectUrl"]["vcs_info"]["commit_id"]
        == "b" * 40
    )


def test_inventory_rejects_installed_distribution_absent_from_lock(
    tmp_path: Path,
) -> None:
    root = tmp_path / "runtime"
    root.mkdir()
    (root / "python.exe").write_bytes(b"MZ\0\0interpreter")
    write_distribution(root)
    configuration = tmp_path / "runtime-source.json"
    write_interpreter_configuration(configuration)
    lock = tmp_path / "uv.lock"
    lock.write_text("version = 1\npackage = []\n")

    with pytest.raises(ValueError, match="absent from uv.lock"):
        write_inventory(root, lock, tmp_path / "evidence", configuration, "windows-x64")


def test_runtime_installation_inventory_requires_compiled_files(tmp_path: Path) -> None:
    root = tmp_path / "runtime"
    root.mkdir()
    (root / "LICENSE.txt").write_text("license\n")
    configuration = tmp_path / "runtime-source.json"
    write_interpreter_configuration(configuration)

    with pytest.raises(ValueError, match="no compiled"):
        runtime_installation_inventory(root, configuration, "windows-x64")
