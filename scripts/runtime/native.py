"""Install the frozen native dependency closure; never solve on a user's machine."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
from pathlib import Path
from typing import Any

from runtime.artifacts import digest, materialize_aliases
from runtime.commands import run


def native_lock_path(project: Path, profile: str, target: str) -> Path:
    configuration = json.loads((project / "runtime-source.json").read_text())
    group: str = configuration["nativeSupplier"]["profiles"][profile]
    return project / "native-locks" / group / f"{target}.json"


def dependency_digest(project: Path, profile: str, target: str) -> str:
    return hashlib.sha256(
        (project / "runtimes" / profile / "uv.lock").read_bytes()
        + native_lock_path(project, profile, target).read_bytes()
    ).hexdigest()


def install_native(project: Path, profile: str, target: str, scratch: Path) -> tuple[Path, dict[str, Any]]:
    runtime, record = install_locked_native(project, native_lock_path(project, profile, target), target, scratch)
    materialize_aliases(runtime)
    return runtime, record


def install_locked_native(project: Path, lock_path: Path, target: str, scratch: Path) -> tuple[Path, dict[str, Any]]:
    configuration = json.loads((project / "runtime-source.json").read_text())
    supplier = configuration["nativeSupplier"]
    version = run("micromamba", "--version", capture=True)
    if version != supplier["micromambaVersion"]:
        raise ValueError(f"Runtime builder requires micromamba {supplier['micromambaVersion']}")
    lock = json.loads(lock_path.read_text())
    if lock["target"] != target or lock["supplier"] != supplier["name"]:
        raise ValueError("Native dependency lock belongs to another target or supplier")
    spec = scratch / "native-explicit.txt"
    spec.write_text("@EXPLICIT\n" + "".join(f"{p['url']}#{p['sha256']}\n" for p in lock["packages"]))
    root = scratch / "native-cache"
    runtime = scratch / "python"
    run(
        "micromamba",
        "--no-rc",
        "create",
        "--yes",
        "--no-shortcuts",
        "--always-copy",
        "--prefix",
        str(runtime),
        "--root-prefix",
        str(root),
        "--file",
        str(spec),
        env=dict(os.environ, CONDA_PKGS_DIRS=str(root / "pkgs"), PYTHONDONTWRITEBYTECODE="1"),
    )
    metadata = runtime / "conda-meta"
    installed = {p["name"]: p for path in metadata.glob("*.json") if (p := json.loads(path.read_text()))}
    if set(installed) != {p["name"] for p in lock["packages"]}:
        raise ValueError("Installed native package graph differs from the frozen lock")
    notices = runtime / "native-notices"
    notices.mkdir()
    records = []
    for package in lock["packages"]:
        actual = installed[package["name"]]
        if any(actual[key] != package[key] for key in ("version", "sha256", "url")):
            raise ValueError(f"Installed native package identity differs: {package['name']}")
        extracted = Path(actual["extracted_package_dir"]).resolve(strict=True)
        if not extracted.is_relative_to((root / "pkgs").resolve()):
            raise ValueError("Native package evidence is outside the controlled build cache")
        info = extracted / "info"
        if not (info / "index.json").is_file():
            raise ValueError(f"Native package inventory is missing: {package['name']}")
        licenses = info / "licenses"
        if licenses.is_dir():
            shutil.copytree(licenses, notices / package["name"])
        records.append({**package, "files": actual["files"]})
    shutil.rmtree(metadata)
    # Files are independent copies; each exporter owns which internal aliases it materializes.
    # The retained inventory/notices own the evidence, so the package cache is no longer needed.
    # Release the producer's package cache before installing large inference dependencies.
    shutil.rmtree(root)
    record = {
        "supplier": supplier["name"],
        "micromambaVersion": version,
        "lockSha256": digest(lock_path),
        "packages": records,
    }
    return runtime, record
