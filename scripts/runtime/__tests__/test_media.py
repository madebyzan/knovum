from __future__ import annotations

from pathlib import Path

import pytest

from runtime import build_media


@pytest.mark.parametrize(
    ("capability", "response", "message"),
    [
        ("-version", "ffmpeg version 8.0 ", "version differs"),
        ("-buildconf", "--enable-nonfree", "redistributable"),
        ("-filters", " no subtitle support ", "subtitle rendering"),
        ("-encoders", " libx264 ", "target platform H.264"),
    ],
)
def test_core_media_qualification_rejects_an_incomplete_target_build(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capability: str,
    response: str,
    message: str,
) -> None:
    outputs = {
        "-version": "ffmpeg version 9.0.1 Copyright",
        "-buildconf": "--enable-gpl --enable-libass",
        "-filters": " subtitles ",
        "-encoders": " h264_mf ",
    }
    outputs[capability] = response

    def run(*args: str, **_kwargs: object) -> str:
        assert args[0] == str(tmp_path / "Library/bin/ffmpeg.exe")
        return outputs[args[-1]]

    monkeypatch.setattr(build_media, "run", run)
    with pytest.raises(ValueError, match=message):
        build_media.relocated_smoke(tmp_path, "windows-x64")
