"""Qualify available Torch backends before freezing a model execution profile."""

from __future__ import annotations

import json
import platform
import sys


def torch_backends(minimum_memory: int) -> dict[str, bool]:
    import torch

    cuda = torch.cuda.is_available() and torch.cuda.mem_get_info(0)[0] >= minimum_memory
    # Torch 2.13 enumerates AMD devices, but its MPS allocator requires unified memory.
    mps = (
        platform.system() == "Darwin"
        and platform.machine() == "arm64"
        and torch.backends.mps.is_available()
        and torch.mps.recommended_max_memory() >= minimum_memory
    )
    return {"cuda": cuda, "mps": mps}


def transcription_backends(minimum_memory: int) -> dict[str, bool]:
    backends = torch_backends(minimum_memory)
    if backends["cuda"]:
        import ctranslate2

        backends["cuda"] = "float16" in ctranslate2.get_supported_compute_types("cuda", device_index=0)
    return backends


def main() -> None:
    detect = transcription_backends if sys.argv[4] == "transcription" else torch_backends
    print(json.dumps(detect(int(sys.argv[3]))))
