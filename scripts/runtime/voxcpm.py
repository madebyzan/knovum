from __future__ import annotations

import tomllib
from pathlib import Path

from runtime.artifacts import digest
from runtime.commands import run


def install(
    project: Path, scratch: Path, packages: Path, build_python: Path, environment: dict[str, str]
) -> dict[str, str]:
    lock = tomllib.loads((project / "uv.lock").read_text())
    voxcpm = next(package for package in lock["package"] if package["name"] == "voxcpm")
    origin = voxcpm["source"]["git"]
    url, commit = origin.split("#")
    url = url.split("?")[0]
    if (
        url != "https://github.com/OpenBMB/VoxCPM.git"
        or len(commit) != 40
        or any(char not in "0123456789abcdef" for char in commit)
    ):
        raise ValueError("VoxCPM source must remain an exact reviewed upstream commit")
    source = scratch / "voxcpm-source"
    run("git", "init", str(source))
    run("git", "-C", str(source), "fetch", "--depth=1", url, commit)
    run("git", "-C", str(source), "checkout", "--detach", "FETCH_HEAD")
    if run("git", "-C", str(source), "rev-parse", "HEAD", capture=True) != commit:
        raise ValueError("VoxCPM source commit mismatch")
    source_archive = scratch / "voxcpm-source.tar"
    run("git", "-C", str(source), "archive", "--format=tar", "HEAD", "-o", str(source_archive))
    wheelhouse = scratch / "wheelhouse"
    environment["SETUPTOOLS_SCM_PRETEND_VERSION_FOR_VOXCPM"] = voxcpm["version"]
    run(
        "uv",
        "build",
        "--wheel",
        "--no-build-isolation",
        "--python",
        str(build_python),
        "--out-dir",
        str(wheelhouse),
        str(source),
        env=environment,
    )
    wheels = list(wheelhouse.glob("*.whl"))
    if len(wheels) != 1:
        raise ValueError("VoxCPM build must produce exactly one wheel")
    run(
        "uv",
        "pip",
        "install",
        "--python",
        str(build_python),
        "--target",
        str(packages),
        "--no-index",
        "--no-deps",
        str(wheels[0]),
        env=environment,
    )
    return {
        "commit": commit,
        "tree": run("git", "-C", str(source), "rev-parse", "HEAD^{tree}", capture=True),
        "sourceSha256": digest(source_archive),
        "wheelSha256": digest(wheels[0]),
    }
