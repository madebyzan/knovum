"""Record the installed dependency graph, including binaries and license evidence."""

from __future__ import annotations

import importlib.metadata
import json
import re
import tomllib
from pathlib import Path
from typing import Any

from runtime_artifacts import digest
from runtime_signing import compiled_binaries


def canonical(name: str) -> str:
    return re.sub(r"[-_.]+", "-", name).lower()


def write_inventory(root: Path, lock_path: Path, output: Path, interpreter_configuration: Path, target: str) -> None:
    lock = tomllib.loads(lock_path.read_text())
    locked = {(canonical(package["name"]), package["version"]): package for package in lock["package"]}
    interpreter = interpreter_inventory(root, interpreter_configuration, target)
    components = [
        {
            "type": "application",
            "name": "CPython",
            "version": interpreter["version"],
            "externalReferences": [{"type": "distribution", "url": interpreter["source"]["url"]}],
        }
    ]
    licenses = []
    for distribution in sorted(
        importlib.metadata.distributions(path=[str(root / "packages")]), key=lambda item: item.metadata["Name"]
    ):
        name, version = distribution.metadata["Name"], distribution.version
        package = locked.get((canonical(name), version))
        if package is None:
            raise ValueError(f"Installed distribution is absent from uv.lock: {name}=={version}")
        binaries = []
        notices = []
        for file in distribution.files or []:
            path = Path(str(distribution.locate_file(file))).resolve(strict=True)
            if not path.is_relative_to(root.resolve()) or not path.is_file():
                raise ValueError(f"Installed distribution declares an external/nonregular file: {name}/{file}")
            relative = path.relative_to(root).as_posix()
            if path.suffix.lower() in {".so", ".dylib", ".dll", ".pyd", ".exe"}:
                binaries.append(relative)
            if any(token in path.name.lower() for token in ("license", "copying", "notice")):
                notices.append({"path": relative, "sha256": digest(path)})
        source: dict[str, Any] = {"origin": package["source"]}
        if "sdist" in package:
            source["sdist"] = package["sdist"]
        if "wheels" in package:
            source["lockedWheels"] = package["wheels"]
        direct = distribution.read_text("direct_url.json")
        if direct:
            source["installedDirectUrl"] = json.loads(direct)
        expression = next(iter(distribution.metadata.get_all("License-Expression", [])), None)
        license_text = next(iter(distribution.metadata.get_all("License", [])), None)
        classifiers = [
            value for value in distribution.metadata.get_all("Classifier", []) if value.startswith("License ::")
        ]
        licenses.append(
            {
                "name": name,
                "version": version,
                "expression": expression,
                "metadataLicense": license_text,
                "classifiers": classifiers,
                "notices": notices,
                "compiledBinaries": binaries,
                "source": source,
            }
        )
        component: dict[str, Any] = {
            "type": "library",
            "name": name,
            "version": version,
            "purl": f"pkg:pypi/{canonical(name)}@{version}",
        }
        if expression:
            component["licenses"] = [{"expression": expression}]
        components.append(component)
    if not licenses:
        raise ValueError("Runtime contains no installed distributions")
    output.mkdir()
    (output / "sbom.cdx.json").write_text(
        json.dumps({"bomFormat": "CycloneDX", "specVersion": "1.6", "version": 1, "components": components}, indent=2)
        + "\n"
    )
    (output / "dependency-evidence.json").write_text(
        json.dumps(
            {
                "dependencyLockSha256": digest(lock_path),
                "reviewStatus": "required",
                "interpreter": interpreter,
                "distributions": licenses,
            },
            indent=2,
        )
        + "\n"
    )


def interpreter_inventory(root: Path, configuration_path: Path, target: str) -> dict[str, Any]:
    configuration = json.loads(configuration_path.read_text())
    notices = []
    for path in sorted(root.rglob("*")):
        if not path.is_file() or path.is_relative_to(root / "packages"):
            continue
        if any(token in path.name.lower() for token in ("license", "copying", "notice")):
            notices.append({"path": path.relative_to(root).as_posix(), "sha256": digest(path)})
    binaries = [
        path.relative_to(root).as_posix()
        for path in compiled_binaries(root, target)
        if not path.is_relative_to(root / "packages")
    ]
    if not binaries:
        raise ValueError("Interpreter inventory has no binaries files")
    return {
        "version": configuration["version"],
        "standaloneRelease": configuration["standaloneRelease"],
        "target": target,
        "source": configuration["targets"][target],
        "compiledBinaries": binaries,
        "notices": notices,
    }
