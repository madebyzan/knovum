from __future__ import annotations

import json
import os
import shutil
from pathlib import Path

from runtime.artifacts import digest, download
from runtime.commands import run
from runtime.inventory import write_inventory
from runtime.voxcpm import install as install_voxcpm


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
    runtime: Path, scratch: Path, executable: Path, repository: Path, profile: str, target: str
) -> dict[str, str]:
    shared_project = repository / "python"
    project = shared_project if profile == "inference" else shared_project / "runtimes/assistant"
    environment = dict(os.environ, UV_LINK_MODE="copy", UV_PYTHON_DOWNLOADS="never", PYTHONDONTWRITEBYTECODE="1")
    requirements = scratch / "requirements.txt"
    packages = runtime / "packages"
    evidence = scratch / "build-evidence"
    options = (
        ("--extra", "inference", "--extra", "model-resources", "--no-emit-package", "voxcpm")
        if profile == "inference"
        else ()
    )
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
    voxcpm: dict[str, str] = {}
    if profile == "inference":
        build_environment = scratch / "build-environment"
        run("uv", "venv", "--python", str(executable), str(build_environment), env=environment)
        build_python = build_environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
        build_requirements = scratch / "build-requirements.txt"
        export_requirements(shared_project, build_requirements, "--only-group", "runtime-build")
        run(
            "uv",
            "pip",
            "install",
            "--python",
            str(build_python),
            "--require-hashes",
            "--no-deps",
            "--only-binary",
            ":all:",
            "-r",
            str(build_requirements),
            env=environment,
        )
        # VoxCPM creates source evidence after the common inventory creates its directory.
        voxcpm = install_voxcpm(shared_project, scratch, packages, build_python, environment)
    write_inventory(runtime, project / "uv.lock", evidence, shared_project / "runtime-source.json", target)
    shutil.copy2(requirements, evidence / "requirements.txt")
    shutil.copy2(requirements, runtime / "runtime-requirements.txt")
    if profile == "inference":
        for name in ("build-requirements.txt", "voxcpm-source.tar"):
            shutil.copy2(scratch / name, evidence / name)
        voxcpm["requirementsSha256"] = digest(requirements)
        voxcpm["buildRequirementsSha256"] = digest(scratch / "build-requirements.txt")
        policy = shared_project / "compiled-source-evidence.json"
        sources = evidence / "compiled-sources"
        sources.mkdir()
        for source in json.loads(policy.read_text())["sources"]:
            download(source, sources / source["fileName"])
        shutil.copy2(policy, evidence / policy.name)
    return voxcpm
