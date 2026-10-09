"""Owned streaming/cancellation boundary for the frozen audio.cpp voice ABI."""

from __future__ import annotations

import ctypes
import threading
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from knovum_inference.contracts.generated.synthesis import SpeechSynthesizeParams
from knovum_inference.errors import server_error
from knovum_inference.speech.compiled_library import load_compiled_library
from knovum_inference.speech.request import check_deadline
from knovum_inference.speech.synthesis.stream import SynthesisChunkPublisher

CALLBACK = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.POINTER(ctypes.c_float), ctypes.c_size_t, ctypes.c_int)
DEVICES = {"cpu": 0, "mps": 1, "cuda": 2, "vulkan": 3}


def load_library() -> Any:
    library = load_compiled_library("knovum_voice")
    library.knovum_voice_error.restype = ctypes.c_char_p
    library.knovum_voice_open.argtypes = [ctypes.c_char_p, ctypes.c_int, ctypes.c_int]
    library.knovum_voice_open.restype = ctypes.c_void_p
    library.knovum_voice_close.argtypes = [ctypes.c_void_p]
    library.knovum_voice_close.restype = None
    library.knovum_voice_generate.argtypes = [
        ctypes.c_void_p,
        ctypes.c_char_p,
        ctypes.POINTER(ctypes.c_float),
        ctypes.c_size_t,
        ctypes.c_char_p,
        ctypes.c_uint,
        ctypes.c_int,
        CALLBACK,
    ]
    library.knovum_audio_devices.argtypes = [ctypes.c_uint64]
    library.knovum_audio_devices.restype = ctypes.c_int
    if library.knovum_voice_abi() != 2:
        raise server_error("runtime_mismatch", "Compiled voice ABI is unsupported")
    return library


class CompiledVoice:
    def __init__(self, model: Path, device: str, threads: int) -> None:
        self.library = load_library()
        devices = self.library.knovum_voice_devices()
        if devices < 0:
            raise server_error("runtime_mismatch", "Compiled voice device discovery failed")
        if device not in DEVICES or not devices & (1 << DEVICES[device]):
            raise server_error("unsupported_device", "Frozen compiled voice backend is unavailable")
        self.handle = self.library.knovum_voice_open(str(model).encode("utf-8"), DEVICES[device], threads)
        if not self.handle:
            raise server_error("model_load_failed", self.library.knovum_voice_error().decode("utf-8"))

    def generate(
        self,
        params: SpeechSynthesizeParams,
        text: str,
        reference: NDArray[np.float32] | None,
        transcript: str | None,
        publisher: SynthesisChunkPublisher,
        cancelled: threading.Event,
    ) -> None:
        failure: BaseException | None = None

        def publish(sample_pointer: Any, count: int, rate: int) -> int:
            nonlocal failure
            try:
                if cancelled.is_set():
                    raise server_error("cancelled", "Synthesis was cancelled")
                check_deadline(params.deadline_ms)
                if not 0 < count <= 1024 * 1024 or rate != 48_000:
                    raise server_error("invalid_output", "Compiled speech chunk exceeds its format contract")
                samples = np.ctypeslib.as_array(sample_pointer, shape=(count,)).copy()
                publisher.publish(samples, rate, "model_chunk")
                if cancelled.is_set():
                    raise server_error("cancelled", "Synthesis was cancelled")
                check_deadline(params.deadline_ms)
                return 0
            except BaseException as error:
                # Python exceptions cannot unwind across a C callback. Re-raise after the compiled engine resets.
                failure = error
                return 1

        callback = CALLBACK(publish)
        pointer = None if reference is None else reference.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
        status = self.library.knovum_voice_generate(
            self.handle,
            text.encode("utf-8"),
            pointer,
            0 if reference is None else len(reference),
            None if transcript is None else transcript.encode("utf-8"),
            params.seed,
            4096,
            callback,
        )
        if failure is not None:
            raise failure
        if status:
            raise server_error("synthesis_failed", self.library.knovum_voice_error().decode("utf-8"))
