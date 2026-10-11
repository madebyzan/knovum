"""CycloneDX evidence for the actual frozen Agent file and dependency inventory."""

from __future__ import annotations

import json
import platform
from importlib.metadata import distribution
from pathlib import Path
from typing import Any

from cyclonedx.model import HashAlgorithm, HashType
from cyclonedx.model.bom import Bom
from cyclonedx.model.component import Component, ComponentType
from cyclonedx.model.license import LicenseExpression
from cyclonedx.output.json import JsonV1Dot6
from cyclonedx.schema import SchemaVersion
from cyclonedx.validation.json import JsonStrictValidator
from packageurl import PackageURL

from policy import runtime_distributions


def write_sbom(root: Path, files: list[dict[str, Any]]) -> None:
    bom = Bom()
    bom.components.add(Component(name="CPython", version=platform.python_version(), type=ComponentType.APPLICATION))
    for name in sorted(runtime_distributions()):
        installed = distribution(name)
        expression = next(iter(installed.metadata.get_all("License-Expression") or []), None)
        bom.components.add(
            Component(
                name=name,
                version=installed.version,
                purl=PackageURL(type="pypi", name=name, version=installed.version),
                licenses=[LicenseExpression(expression)] if expression else [],
            )
        )
    for file in files:
        if "sha256" in file:
            bom.components.add(
                Component(
                    name=file["path"],
                    type=ComponentType.FILE,
                    hashes=[HashType(alg=HashAlgorithm.SHA_256, content=file["sha256"])],
                )
            )
    output = JsonV1Dot6(bom).output_as_string(indent=2)
    if JsonStrictValidator(SchemaVersion.V1_6).validate_str(output):
        raise ValueError("Frozen Agent SBOM failed CycloneDX validation")
    json.loads(output)
    (root / "agent-sbom.cdx.json").write_text(output)
