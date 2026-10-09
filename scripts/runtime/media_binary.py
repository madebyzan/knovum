"""Install one pinned static FFmpeg executable and qualify its OS dependencies."""

from __future__ import annotations

import json
import os
import re
import shutil
import stat
import zipfile
from pathlib import Path
from typing import Any

from runtime.artifacts import download
from runtime.commands import run

WINDOWS_SYSTEM_LIBRARIES = frozenset(
    [
        "advapi32.dll",
        "avicap32.dll",
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
        "msimg32.dll",
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
        raise ValueError("Windows media production requires the MSVC build tools")
    return max(
        paths,
        key=lambda path: tuple(int(part) for part in path.parents[3].name.split(".")),
    )


def install_static_binary(configuration: dict[str, Any], target: str, scratch: Path, root: Path) -> Path:
    source = configuration["targets"][target]
    archive = scratch / "ffmpeg.zip"
    download(source, archive)
    executable = root / "bin" / ("ffmpeg.exe" if target == "windows-x64" else "ffmpeg")
    executable.parent.mkdir(parents=True)
    notices = root / "notices"
    notices.mkdir()
    with zipfile.ZipFile(archive) as package:
        copy_archive_file(package, source["entry"], executable)
        for name, entry in source.get("archiveNotices", {}).items():
            copy_archive_file(package, entry, notices / name)
    executable.chmod(0o755)
    download(configuration["licenseSource"], notices / "COPYING.GPLv3")
    for notice in source.get("noticeSources", []):
        download(notice, notices / notice["fileName"])
    (notices / "source-offer.json").write_text(
        json.dumps(
            {
                "license": configuration["license"],
                "ffmpegSource": configuration["sourceOffer"],
                "supplierBuild": source["buildSource"],
                "correspondingSourceReview": "required",
            },
            indent=2,
        )
        + "\n"
    )
    return executable


def copy_archive_file(package: zipfile.ZipFile, entry: str, destination: Path) -> None:
    matches = [item for item in package.infolist() if item.filename == entry]
    if len(matches) != 1:
        raise ValueError(f"FFmpeg archive must contain exactly one selected entry: {entry}")
    item = matches[0]
    if (
        item.is_dir()
        or stat.S_IFMT(item.external_attr >> 16) not in {0, stat.S_IFREG}
        or item.file_size > 256 * 1024 * 1024
    ):
        raise ValueError(f"FFmpeg archive entry must be a bounded regular file: {entry}")
    with package.open(item) as source, destination.open("xb") as output:
        shutil.copyfileobj(source, output, 1024 * 1024)


def require_static_binary(executable: Path, target: str) -> None:
    if target.startswith("macos-"):
        output = run("otool", "-L", str(executable), capture=True)
        dependencies = [line.strip().split(" (", 1)[0] for line in output.splitlines() if line.startswith("\t")]
        if not dependencies or any(not name.startswith(("/usr/lib/", "/System/Library/")) for name in dependencies):
            raise ValueError("Static FFmpeg may depend only on macOS system libraries")
    else:
        output = run(
            str(dumpbin_path()),
            "/NOLOGO",
            "/DEPENDENTS",
            str(executable),
            capture=True,
            env=dict(os.environ, VSLANG="1033"),
        )
        dependencies = re.findall(r"^\s+([\w.+-]+\.(?:dll|drv))\s*$", output, flags=re.MULTILINE | re.IGNORECASE)
        system = Path(os.environ["SYSTEMROOT"]) / "System32"
        if not dependencies or any(
            not (
                name.lower().startswith(("api-ms-", "ext-ms-"))
                or name.lower() in WINDOWS_SYSTEM_LIBRARIES
                and (system / name).is_file()
            )
            for name in dependencies
        ):
            raise ValueError("Static FFmpeg may depend only on Windows system libraries")
