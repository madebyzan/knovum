from __future__ import annotations

from pathlib import Path

import pytest

from runtime import media_files


def put(root: Path, name: str) -> Path:
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"compiled file")
    return path


def test_media_export_retains_transitive_libraries_data_and_notices_without_sdk_tools(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    ffmpeg = put(tmp_path, "bin/ffmpeg")
    codec = put(tmp_path, "lib/libavcodec.63.dylib")
    opus = put(tmp_path, "lib/libopus.0.dylib")
    retained = {
        ffmpeg,
        codec,
        opus,
        put(tmp_path, "share/fonts/a.ttf"),
        put(tmp_path, "conda-notices/opus/LICENSE"),
    }
    put(tmp_path, "bin/other-tool")
    put(tmp_path, "include/opus.h")
    put(tmp_path, "lib/libopus.dylib")
    put(tmp_path, "lib/libopus.a")
    put(tmp_path, "share/man/manual")
    dependencies = {
        ffmpeg: ["@rpath/libavcodec.63.dylib"],
        codec: ["@rpath/libavcodec.63.dylib", "@rpath/libopus.0.dylib"],
        opus: ["/usr/lib/libSystem.B.dylib"],
    }

    def run(*args: str, **_kwargs: object) -> str:
        path = Path(args[-1])
        return str(path) + ":\n" + "\n".join(f"\t{name} (compatibility version 1.0.0)" for name in dependencies[path])

    monkeypatch.setattr(media_files, "run", run)
    graph = media_files.export_media_files(tmp_path, "macos-x64")
    assert set(tmp_path.rglob("*")) & retained == retained
    assert {p for p in tmp_path.rglob("*") if p.is_file()} == retained
    assert graph == ["bin/ffmpeg", "lib/libavcodec.63.dylib", "lib/libopus.0.dylib"]


def test_media_export_rejects_a_non_system_library_outside_the_frozen_prefix(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    put(tmp_path, "bin/ffmpeg")
    monkeypatch.setattr(media_files, "run", lambda *_args, **_kwargs: "ffmpeg:\n\t/usr/local/lib/unknown.dylib (x)")
    with pytest.raises(ValueError, match="outside the frozen"):
        media_files.export_media_files(tmp_path, "macos-x64")
    assert (tmp_path / "bin/ffmpeg").is_file()


def test_windows_media_export_retains_vendor_dlls_and_requires_resolved_dependencies(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    root = tmp_path / "runtime"
    ffmpeg = put(root, "Library/bin/ffmpeg.exe")
    codec = put(root, "Library/bin/avcodec-63.dll")
    put(root, "Library/bin/unused.dll")
    system = tmp_path / "Windows"
    put(system, "System32/kernel32.dll")
    monkeypatch.setenv("SYSTEMROOT", str(system))
    monkeypatch.setattr(media_files, "dumpbin_path", lambda: Path("dumpbin.exe"))

    def run(*args: str, **_kwargs: object) -> str:
        return (
            "    AVCODEC-63.DLL\n    KERNEL32.dll\n"
            if Path(args[-1]) == ffmpeg
            else "    api-ms-win-core-file-l1-1-0.dll\n"
        )

    monkeypatch.setattr(media_files, "run", run)
    assert media_files.export_media_files(root, "windows-x64") == [
        "Library/bin/avcodec-63.dll",
        "Library/bin/ffmpeg.exe",
    ]
    assert codec.is_file() and not (root / "Library/bin/unused.dll").exists()
    assert (system / "System32/kernel32.dll").is_file()
    put(system, "System32/missing-vendor.dll")
    monkeypatch.setattr(media_files, "run", lambda *_args, **_kwargs: "    missing-vendor.dll\n")
    with pytest.raises(ValueError, match="not present"):
        media_files.export_media_files(root, "windows-x64")


def test_windows_inspector_uses_the_latest_msvc_toolset_from_the_official_installer(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("PROGRAMFILES(X86)", str(tmp_path))
    older = tmp_path / "VC/Tools/MSVC/14.9.99999/bin/Hostx64/x64/dumpbin.exe"
    newer = tmp_path / "VC/Tools/MSVC/14.50.12345/bin/Hostx64/x64/dumpbin.exe"

    def run(*args: str, **_kwargs: object) -> str:
        assert args[0] == str(tmp_path / "Microsoft Visual Studio/Installer/vswhere.exe")
        assert "Microsoft.VisualStudio.Component.VC.Tools.x86.x64" in args
        return f"{older}\n{newer}\n"

    monkeypatch.setattr(media_files, "run", run)
    assert media_files.dumpbin_path() == newer
