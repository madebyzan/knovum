from __future__ import annotations

import json
import os
import shutil
from pathlib import Path

from runtime.artifacts import download
from runtime.commands import run
from runtime.inventory import write_inventory
from runtime.native import native_lock_path


def export_requirements(project: Path, destination: Path, *options: str) -> None:
    run(
        "uv",
        "export",
        "--project",
        str(project),
        "--frozen",
        "--no-emit-project",
        "--no-header",
        "--no-annotate",
        "--no-default-groups",
        *options,
        "--output-file",
        str(destination),
    )


def install_dependencies(
    runtime: Path,
    scratch: Path,
    executable: Path,
    repository: Path,
    profile: str,
    target: str,
) -> dict[str, object]:
    shared_project = repository / "python"
    project = shared_project / "runtimes" / profile
    environment = dict(
        os.environ,
        UV_LINK_MODE="copy",
        UV_PYTHON_DOWNLOADS="never",
        PYTHONDONTWRITEBYTECODE="1",
    )
    requirements = scratch / "requirements.txt"
    packages = runtime / "packages"
    evidence = scratch / "build-evidence"
    configuration = json.loads((shared_project / "runtime-source.json").read_text())
    supplier = configuration["nativeSupplier"]
    native_packages = supplier["pythonDistributions"][supplier["profiles"][profile]]
    options = tuple(part for name in native_packages for part in ("--no-emit-package", name))
    export_requirements(project, requirements, *options)
    run(
        "uv",
        "pip",
        "install",
        "--python",
        str(executable),
        "--target",
        str(packages),
        "--require-hashes",
        "--no-deps",
        "--only-binary",
        ":all:",
        "-r",
        str(requirements),
        env=environment,
    )
    voxcpm: dict[str, object] = {}
    source_components: list[dict[str, object]] = []
    if profile == "voice-generation":
        from runtime.voxcpm import install as install_voxcpm

        voxcpm = install_voxcpm(shared_project, scratch, runtime, target)
    if profile == "light-inference":
        from runtime.moss_runtime import install as install_moss

        source_components.append(install_moss(shared_project, packages))
    if voxcpm:
        source_components.append(voxcpm)
    write_inventory(
        runtime,
        project / "uv.lock",
        evidence,
        native_lock_path(shared_project, profile, target),
        target,
        source_components,
    )
    shutil.copy2(requirements, evidence / "requirements.txt")
    shutil.copy2(requirements, runtime / "runtime-requirements.txt")
    if profile == "voice-generation":
        shutil.copy2(scratch / "audio-cpp-source.tar.gz", evidence / "audio-cpp-source.tar.gz")
        for name in ("voice_bridge.cpp", "voice_bridge.cmake"):
            shutil.copy2(repository / "scripts/runtime" / name, evidence / name)
    if profile == "light-inference":
        policy = shared_project / "compiled-source-evidence.json"
        sources = evidence / "compiled-sources"
        sources.mkdir()
        for source in json.loads(policy.read_text())["sources"]:
            download(source, sources / source["fileName"])
        shutil.copy2(policy, evidence / policy.name)
    return voxcpm
