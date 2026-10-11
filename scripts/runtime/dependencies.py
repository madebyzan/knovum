from __future__ import annotations

import json
import os
import shutil
from pathlib import Path

from runtime.commands import run
from runtime.conda_packages import conda_lock_path
from runtime.inventory import write_inventory


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
) -> None:
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
    supplier = configuration["condaSupplier"]
    conda_packages = supplier["pythonDistributions"][supplier["profiles"][profile]]
    options = tuple(part for name in conda_packages for part in ("--no-emit-package", name))
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
    write_inventory(
        runtime,
        project / "uv.lock",
        evidence,
        conda_lock_path(shared_project, profile, target),
        target,
    )
    shutil.copy2(requirements, evidence / "requirements.txt")
    shutil.copy2(requirements, runtime / "runtime-requirements.txt")
