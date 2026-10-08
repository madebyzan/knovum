"""Report available compiled native voice backends before model admission."""

from __future__ import annotations

import json
import sys

from knovum_inference.speech.synthesis.native_voice import load_library


def preferred_device(mask: int) -> str:
    if mask < 0 or not mask & 1:
        raise RuntimeError("Native voice backend inventory is invalid")
    for flag, device in ((4, "cuda"), (2, "mps"), (8, "vulkan")):
        if mask & flag:
            return device
    return "cpu"


def main() -> None:
    library = load_library()
    print(json.dumps({"device": preferred_device(library.knovum_audio_devices(int(sys.argv[3])))}))
