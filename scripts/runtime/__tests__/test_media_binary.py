from __future__ import annotations

import stat
import zipfile
from pathlib import Path

import pytest

from runtime import media_binary


def test_static_export_selects_only_the_executable(tmp_path: Path) -> None:
    archive = tmp_path / "build.zip"
    with zipfile.ZipFile(archive, "w") as package:
        package.writestr("bin/ffmpeg.exe", b"static executable")
        package.writestr("bin/ffprobe.exe", b"unused")
        package.writestr("../../outside", b"unselected")
    destination = tmp_path / "ffmpeg.exe"
    with zipfile.ZipFile(archive) as package:
        media_binary.copy_archive_file(package, "bin/ffmpeg.exe", destination)
    assert destination.read_bytes() == b"static executable"
    assert not (tmp_path / "ffprobe.exe").exists()


@pytest.mark.parametrize("kind", ["missing", "duplicate", "symlink", "directory", "special"])
def test_static_export_rejects_ambiguous_or_nonregular_entries(tmp_path: Path, kind: str) -> None:
    archive = tmp_path / "build.zip"
    entry = "ffmpeg/" if kind == "directory" else "ffmpeg"
    with zipfile.ZipFile(archive, "w") as package:
        if kind != "missing":
            info = zipfile.ZipInfo(entry)
            if kind == "symlink":
                info.create_system = 3
                info.external_attr = (stat.S_IFLNK | 0o777) << 16
            if kind == "special":
                info.create_system = 3
                info.external_attr = (stat.S_IFIFO | 0o600) << 16
            package.writestr(info, b"content")
        if kind == "duplicate":
            with pytest.warns(UserWarning, match="Duplicate"):
                package.writestr(entry, b"duplicate")
    with zipfile.ZipFile(archive) as package, pytest.raises(ValueError):
        media_binary.copy_archive_file(package, entry, tmp_path / "selected")
    assert not (tmp_path / "selected").exists()


@pytest.mark.parametrize("dependency", ["/usr/local/lib/vendor.dylib", "@rpath/libavcodec.dylib", ""])
def test_static_macos_build_rejects_external_dependency(
    monkeypatch: pytest.MonkeyPatch,
    dependency: str,
) -> None:
    monkeypatch.setattr(media_binary, "run", lambda *_args, **_kwargs: f"ffmpeg:\n\t{dependency} (version)")
    with pytest.raises(ValueError, match="system libraries"):
        media_binary.require_static_binary(Path("ffmpeg"), "macos-x64")


def test_static_macos_build_accepts_only_system_dependencies(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        media_binary,
        "run",
        lambda *_args, **_kwargs: (
            "ffmpeg:\n\t/usr/lib/libSystem.B.dylib (version)\n"
            "\t/System/Library/Frameworks/CoreVideo.framework/CoreVideo (version)"
        ),
    )
    media_binary.require_static_binary(Path("ffmpeg"), "macos-arm64")


@pytest.mark.parametrize(
    "dependency", ["KERNEL32.dll", "MSIMG32.dll", "AVICAP32.dll", "api-ms-win-crt-stdio-l1-1-0.dll"]
)
def test_static_windows_build_accepts_system_imports(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, dependency: str
) -> None:
    system = tmp_path / "System32"
    system.mkdir()
    (system / dependency).touch()
    monkeypatch.setenv("SYSTEMROOT", str(tmp_path))
    monkeypatch.setattr(media_binary, "dumpbin_path", lambda: Path("dumpbin.exe"))
    monkeypatch.setattr(media_binary, "run", lambda *_args, **_kwargs: f"    {dependency}\n")
    media_binary.require_static_binary(Path("ffmpeg.exe"), "windows-x64")


def test_static_windows_build_rejects_vendor_imports(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("SYSTEMROOT", str(tmp_path))
    monkeypatch.setattr(media_binary, "dumpbin_path", lambda: Path("dumpbin.exe"))
    monkeypatch.setattr(media_binary, "run", lambda *_args, **_kwargs: "    avcodec.dll\n")
    with pytest.raises(ValueError, match="system libraries"):
        media_binary.require_static_binary(Path("ffmpeg.exe"), "windows-x64")
