from pathlib import Path

import pytest

from runtime import media_files


def test_media_export_materializes_only_the_required_soname_before_removing_sdk_aliases(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    (tmp_path / "bin").mkdir()
    (tmp_path / "lib").mkdir()
    (tmp_path / "bin/ffmpeg").write_bytes(b"executable")
    original = tmp_path / "lib/libcodec.63.1.dylib"
    original.write_bytes(b"frozen compiled library")
    soname = tmp_path / "lib/libcodec.63.dylib"
    soname.symlink_to(original.name)
    (tmp_path / "lib/libcodec.dylib").symlink_to(original.name)
    sdk = tmp_path / "lib/icu/78.1"
    sdk.mkdir(parents=True)
    (sdk / "sdk-config").write_text("development only")
    (sdk.parent / "current").symlink_to(sdk.name, target_is_directory=True)

    def run(*args: str, **_kwargs: object) -> str:
        return f"{args[-1]}:\n\t@rpath/libcodec.63.dylib (compatibility version 63.0.0)\n"

    monkeypatch.setattr(media_files, "run", run)
    assert media_files.export_media_files(tmp_path, "macos-x64") == [
        "bin/ffmpeg",
        "lib/libcodec.63.dylib",
    ]
    assert soname.is_file() and not soname.is_symlink()
    assert soname.read_bytes() == b"frozen compiled library"
    assert not original.exists() and not (tmp_path / "lib/libcodec.dylib").exists()
    assert not (tmp_path / "lib/icu/current").is_symlink()
    assert not (tmp_path / "lib/icu").exists()
