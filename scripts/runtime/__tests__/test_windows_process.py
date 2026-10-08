from pathlib import Path
from xml.etree import ElementTree as ET

import pytest

from runtime import windows_process


def test_utf8_manifest_preserves_existing_process_settings(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    executable = tmp_path / "runtime 中文/python.exe"
    executable.parent.mkdir()
    executable.write_bytes(b"controlled-python")
    evidence = tmp_path / "evidence"
    evidence.mkdir()
    original = """<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
      <assemblyIdentity name="python" version="3.11.17.0"/>
      <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3"><security><requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges></security></trustInfo>
      <application xmlns="urn:schemas-microsoft-com:asm.v3"><windowsSettings>
        <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
        <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
        <activeCodePage xmlns="http://schemas.microsoft.com/SMI/2019/WindowsSettings">Legacy</activeCodePage>
      </windowsSettings></application>
    </assembly>"""
    calls: list[tuple[str, ...]] = []

    def run(*arguments: str) -> str:
        calls.append(arguments)
        if any(argument.startswith("-inputresource:") for argument in arguments):
            Path(next(argument[5:] for argument in arguments if argument.startswith("-out:"))).write_text(original)
        return ""

    monkeypatch.setattr(windows_process, "manifest_tool", lambda: Path("controlled-mt.exe"))
    monkeypatch.setattr(windows_process, "run", run)
    windows_process.configure_utf8(executable, evidence)
    manifest = evidence / "python-process.manifest"
    root = ET.parse(manifest).getroot()
    assert root.findall("{urn:schemas-microsoft-com:asm.v1}assemblyIdentity")[0].attrib["name"] == "python"
    assert (
        root.findall(".//{urn:schemas-microsoft-com:asm.v3}requestedExecutionLevel")[0].attrib["level"] == "asInvoker"
    )
    assert root.findall(".//{http://schemas.microsoft.com/SMI/2016/WindowsSettings}longPathAware")[0].text == "true"
    assert root.findall(".//{http://schemas.microsoft.com/SMI/2005/WindowsSettings}dpiAware")[0].text == "true"
    assert root.findall(".//{http://schemas.microsoft.com/SMI/2019/WindowsSettings}activeCodePage")[0].text == "UTF-8"
    assert calls[0][1] == f"-inputresource:{executable};#1"
    assert "-validate_manifest" in calls[1]
    assert calls[2][-1] == f"-outputresource:{executable};#1"


def test_adds_process_utf8_without_replacing_original_manifest(tmp_path: Path) -> None:
    manifest = tmp_path / "python.manifest"
    manifest.write_text(
        '<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">'
        "<description>CPython</description></assembly>"
    )
    windows_process.write_utf8_policy(manifest)
    root = ET.parse(manifest).getroot()
    assert root.findall("{urn:schemas-microsoft-com:asm.v1}description")[0].text == "CPython"
    assert root.findall(".//{http://schemas.microsoft.com/SMI/2019/WindowsSettings}activeCodePage")[0].text == "UTF-8"


def test_manifest_tool_uses_installed_x64_windows_sdk(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("PROGRAMFILES(X86)", str(tmp_path))
    for version in ("10.0.19041.0", "10.0.26100.0"):
        tool = tmp_path / f"Windows Kits/10/bin/{version}/x64/mt.exe"
        tool.parent.mkdir(parents=True)
        tool.touch()
    assert windows_process.manifest_tool().parent.parent.name == "10.0.26100.0"
