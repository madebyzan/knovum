"""Configure the delivered CPython process's Windows UTF-8 ABI before signing."""

from __future__ import annotations

import os
from pathlib import Path
from xml.etree import ElementTree as ET

from runtime.commands import run

ASSEMBLY = "urn:schemas-microsoft-com:asm.v1"
APPLICATION = "urn:schemas-microsoft-com:asm.v3"
CODEPAGE = "http://schemas.microsoft.com/SMI/2019/WindowsSettings"


def manifest_tool() -> Path:
    sdk = Path(os.environ["PROGRAMFILES(X86)"]) / "Windows Kits/10/bin"
    tools = list(sdk.glob("10.*/x64/mt.exe"))
    if not tools:
        raise ValueError("Windows CPython production requires the Windows SDK Manifest Tool")
    return max(
        tools,
        key=lambda tool: tuple(int(part) for part in tool.parent.parent.name.split(".")),
    )


def write_utf8_policy(manifest: Path) -> None:
    ET.register_namespace("", ASSEMBLY)
    ET.register_namespace("asmv3", APPLICATION)
    ET.register_namespace("ws2019", CODEPAGE)
    tree = ET.parse(manifest)
    root = tree.getroot()
    application = root.find(f"{{{APPLICATION}}}application")
    if application is None:
        application = ET.SubElement(root, f"{{{APPLICATION}}}application")
    settings = application.find(f"{{{APPLICATION}}}windowsSettings")
    if settings is None:
        settings = ET.SubElement(application, f"{{{APPLICATION}}}windowsSettings")
    codepage = settings.find(f"{{{CODEPAGE}}}activeCodePage")
    if codepage is None:
        codepage = ET.SubElement(settings, f"{{{CODEPAGE}}}activeCodePage")
    codepage.text = "UTF-8"
    tree.write(manifest, encoding="utf-8", xml_declaration=True)


def configure_utf8(executable: Path, evidence: Path) -> None:
    tool = str(manifest_tool())
    original = evidence / "python-original.manifest"
    configured = evidence / "python-process.manifest"
    run(tool, f"-inputresource:{executable};#1", f"-out:{original}")
    configured.write_bytes(original.read_bytes())
    write_utf8_policy(configured)
    run(tool, "-validate_manifest", "-manifest", str(configured))
    run(tool, "-manifest", str(configured), f"-outputresource:{executable};#1")
