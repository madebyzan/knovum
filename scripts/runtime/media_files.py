"""Export FFmpeg's native dependency closure, runtime data and supplier notices."""

from __future__ import annotations

import os
import re
from pathlib import Path

from runtime.artifacts import materialize_aliases
from runtime.commands import run

# Supported Windows SDK/OS imports; third-party redistributables must stay in the frozen prefix.
WINDOWS_SYSTEM_LIBRARIES = frozenset(
    [
        "advapi32.dll",
        "avifil32.dll",
        "avrt.dll",
        "authz.dll",
        "bcrypt.dll",
        "bcryptprimitives.dll",
        "cfgmgr32.dll",
        "combase.dll",
        "comctl32.dll",
        "comdlg32.dll",
        "crypt32.dll",
        "cryptbase.dll",
        "cryptsp.dll",
        "d2d1.dll",
        "d3d9.dll",
        "d3d11.dll",
        "d3d12.dll",
        "d3dcompiler_47.dll",
        "dcomp.dll",
        "dbgcore.dll",
        "dbghelp.dll",
        "dinput8.dll",
        "dnsapi.dll",
        "dsound.dll",
        "dwmapi.dll",
        "dwrite.dll",
        "dxgi.dll",
        "dxva2.dll",
        "gdi32.dll",
        "gdi32full.dll",
        "glu32.dll",
        "hid.dll",
        "imagehlp.dll",
        "imm32.dll",
        "iphlpapi.dll",
        "kernel32.dll",
        "kernelbase.dll",
        "mf.dll",
        "mfplat.dll",
        "mfreadwrite.dll",
        "msacm32.dll",
        "msvcp_win.dll",
        "msvcrt.dll",
        "msvfw32.dll",
        "mswsock.dll",
        "ncrypt.dll",
        "netapi32.dll",
        "normaliz.dll",
        "nsi.dll",
        "ntdll.dll",
        "ole32.dll",
        "oleaut32.dll",
        "opengl32.dll",
        "powrprof.dll",
        "propsys.dll",
        "psapi.dll",
        "quartz.dll",
        "rpcrt4.dll",
        "sechost.dll",
        "secur32.dll",
        "sensapi.dll",
        "setupapi.dll",
        "shcore.dll",
        "shell32.dll",
        "shlwapi.dll",
        "ucrtbase.dll",
        "user32.dll",
        "userenv.dll",
        "usp10.dll",
        "uxtheme.dll",
        "version.dll",
        "wer.dll",
        "win32u.dll",
        "winhttp.dll",
        "wininet.dll",
        "winmm.dll",
        "winspool.drv",
        "winsta.dll",
        "wintrust.dll",
        "wldap32.dll",
        "ws2_32.dll",
        "wsock32.dll",
        "wtsapi32.dll",
        "xinput1_4.dll",
    ]
)


def dumpbin_path() -> Path:
    installer = Path(os.environ["PROGRAMFILES(X86)"]) / "Microsoft Visual Studio/Installer/vswhere.exe"
    output = run(
        str(installer),
        "-latest",
        "-products",
        "*",
        "-requires",
        "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
        "-find",
        "VC/Tools/MSVC/*/bin/Hostx64/x64/dumpbin.exe",
        capture=True,
    )
    paths = [Path(line) for line in output.splitlines() if line.strip()]
    if not paths:
        raise ValueError("Native Windows media production requires the MSVC build tools")
    return max(
        paths,
        key=lambda path: tuple(int(part) for part in path.parents[3].name.split(".")),
    )


def _mac_dependencies(path: Path, root: Path, entry: Path) -> list[Path]:
    output = run("otool", "-L", str(path), capture=True)
    files = []
    for line in output.splitlines():
        if not line.startswith("\t"):
            continue
        name = line.strip().split(" (", 1)[0]
        if name.startswith(("/usr/lib/", "/System/Library/")):
            continue
        if name.startswith("@rpath/"):
            dependency = root / "lib" / name.removeprefix("@rpath/")
        elif name.startswith("@loader_path/"):
            dependency = path.parent / name.removeprefix("@loader_path/")
        elif name.startswith("@executable_path/"):
            dependency = entry.parent / name.removeprefix("@executable_path/")
        else:
            dependency = Path(name)
        if not dependency.resolve().is_relative_to(root):
            raise ValueError(f"Media dependency is outside the frozen native prefix: {name}")
        if not dependency.is_file():
            raise ValueError(f"Media native dependency is not present: {name}")
        files.append(Path(os.path.abspath(dependency)))
    return files


def _windows_dependencies(path: Path, root: Path, inspector: Path) -> list[Path]:
    output = run(
        str(inspector),
        "/NOLOGO",
        "/DEPENDENTS",
        str(path),
        capture=True,
        env=dict(os.environ, VSLANG="1033"),
    )
    files = []
    system = Path(os.environ["SYSTEMROOT"]) / "System32"
    libraries = {path.name.lower(): path for path in (root / "Library/bin").iterdir() if path.is_file()}
    for name in re.findall(r"^\s+([\w.+-]+\.(?:dll|drv))\s*$", output, flags=re.MULTILINE | re.IGNORECASE):
        dependency = libraries.get(name.lower())
        if dependency is not None:
            if not dependency.resolve().is_relative_to(root):
                raise ValueError(f"Media dependency is outside the frozen native prefix: {name}")
            files.append(dependency.absolute())
        elif not _windows_os_library(name, system):
            raise ValueError(f"Media native dependency is not present: {name}")
    return files


def _windows_os_library(name: str, system: Path) -> bool:
    name = name.lower()
    if name.startswith(("api-ms-", "ext-ms-")):
        return True
    return name in WINDOWS_SYSTEM_LIBRARIES and (system / name).is_file()


def media_native_files(root: Path, target: str) -> set[Path]:
    root = root.resolve(strict=True)
    entry = root / ("Library/bin/ffmpeg.exe" if target == "windows-x64" else "bin/ffmpeg")
    if not entry.is_file():
        raise ValueError("The frozen native media package has no FFmpeg entry point")
    inspector = dumpbin_path() if target == "windows-x64" else None
    remaining = [entry]
    native: set[Path] = set()
    while remaining:
        path = remaining.pop()
        if path in native:
            continue
        native.add(path)
        dependencies = (
            _windows_dependencies(path, root, inspector)
            if inspector is not None
            else _mac_dependencies(path, root, entry)
        )
        remaining.extend(dependencies)
    return native


def export_media_files(root: Path, target: str) -> list[str]:
    root = root.resolve(strict=True)
    native = media_native_files(root, target)
    # This producer owns the prefix. Validate the complete graph before changing any file.
    data = (
        "etc",
        "share",
        "ssl",
        "Library/etc",
        "Library/share",
        "Library/ssl",
        "native-notices",
    )
    excluded = ("share/man", "share/doc", "Library/share/man", "Library/share/doc")
    retained = {
        path
        for path in root.rglob("*")
        if path in native or _runtime_data(path.relative_to(root).as_posix(), data, excluded)
    }
    # Keep canonical targets until each selected alias is exported as an independent file.
    materialize_aliases(root, retained)
    for path in sorted(root.rglob("*"), reverse=True):
        if not path.is_symlink() and path.is_dir():
            if not any(path.iterdir()):
                path.rmdir()
            continue
        relative = path.relative_to(root).as_posix()
        if path not in native and not _runtime_data(relative, data, excluded):
            path.unlink()
    return sorted(path.relative_to(root).as_posix() for path in native)


def _runtime_data(relative: str, data: tuple[str, ...], excluded: tuple[str, ...]) -> bool:
    return any(relative.startswith(prefix + "/") for prefix in data) and not any(
        relative.startswith(prefix + "/") for prefix in excluded
    )
