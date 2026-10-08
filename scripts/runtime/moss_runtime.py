"""Install the official standalone ONNX supplier unchanged, with source and notices."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from runtime.artifacts import download


def install(project: Path, packages: Path) -> dict[str, Any]:
    specification = json.loads((project / "runtime-source.json").read_text())["mossNano"]
    destination = packages / "moss_tts_nano_ort"
    destination.mkdir()
    for file in specification["files"]:
        download(file, destination / file["fileName"])
    return {
        "name": "MOSS-TTS-Nano standalone ONNX",
        "version": specification["revision"],
        "license": specification["license"],
        "files": [
            {**file, "path": f"packages/moss_tts_nano_ort/{file['fileName']}"} for file in specification["files"]
        ],
    }
