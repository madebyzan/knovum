"""Record the installed dependency graph, including binaries and license evidence."""

from __future__ import annotations

import importlib.metadata
import json
import re
import tomllib
from pathlib import Path
from typing import Any

from cyclonedx.model import ExternalReference, ExternalReferenceType, XsUri
from cyclonedx.model.bom import Bom
from cyclonedx.model.component import Component, ComponentType
from cyclonedx.model.license import LicenseExpression
from cyclonedx.output.json import JsonV1Dot6
from cyclonedx.schema import SchemaVersion
from cyclonedx.validation.json import JsonStrictValidator
from packageurl import PackageURL

from runtime.artifacts import digest
from runtime.binaries import compiled_binaries


def canonical(name: str) -> str:
    return re.sub(r"[-_.]+", "-", name).lower()


def write_inventory(
    root: Path,
    lock_path: Path,
    output: Path,
    interpreter_configuration: Path,
    target: str,
) -> None:
    root = root.resolve(strict=True)
    lock = tomllib.loads(lock_path.read_text())
    locked = {(canonical(package["name"]), package["version"]): package for package in lock["package"]}
    interpreter = interpreter_inventory(root, interpreter_configuration, target)
    bom = Bom()
    bom.components.add(
        Component(
            type=ComponentType.APPLICATION,
            name="CPython",
            version=interpreter["version"],
            bom_ref=f"cpython@{interpreter['version']}",
            external_references=[
                ExternalReference(
                    type=ExternalReferenceType.DISTRIBUTION,
                    url=XsUri(interpreter["source"]["url"]),
                )
            ],
        )
    )
    licenses = []
    for distribution in sorted(
        importlib.metadata.distributions(path=[str(root / "packages")]),
        key=lambda item: item.metadata["Name"],
    ):
        name, version = distribution.metadata["Name"], distribution.version
        package = locked.get((canonical(name), version))
        if package is None:
            raise ValueError(f"Installed distribution is absent from uv.lock: {name}=={version}")
        binaries = []
        notices = []
        files = distribution.files
        if files is None:
            raise ValueError(f"Installed distribution has no file record: {name}=={version}")
        for file in files:
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
        purl = PackageURL(type="pypi", name=canonical(name), version=version)
        bom.components.add(
            Component(
                type=ComponentType.LIBRARY,
                name=name,
                version=version,
                bom_ref=purl.to_string(),
                purl=purl,
                licenses=[LicenseExpression(expression)] if expression else [],
            )
        )
    if not licenses:
        raise ValueError("Runtime contains no installed distributions")
    output.mkdir()
    serialized = JsonV1Dot6(bom).output_as_string(indent=2)
    errors = JsonStrictValidator(SchemaVersion.V1_6).validate_str(serialized)
    if errors:
        raise ValueError(f"Generated CycloneDX inventory is invalid: {errors}")
    (output / "sbom.cdx.json").write_text(serialized + "\n", encoding="utf-8")
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
