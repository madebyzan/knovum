"""Forced alignment through the frozen audio.cpp public C API."""

from __future__ import annotations

import ctypes as c
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from knovum_inference.errors import server_error
from knovum_inference.speech.native_runtime import load_native_library


class ModelConfig(c.Structure):
    _fields_ = [(name, c.c_char_p) for name in ("family_hint", "config_id", "weight_id", "model_spec_override")]


class BackendConfig(c.Structure):
    _fields_ = [("backend", c.c_char_p), ("device", c.c_int), ("threads", c.c_int)]


def load_api() -> Any:
    library = load_native_library("audiocpp")
    pointer = c.c_void_p
    signatures = {
        "audiocpp_abi_version": ([], c.c_uint32),
        "audiocpp_last_error": ([], c.c_char_p),
        "audiocpp_registry_create": ([c.c_char_p, c.POINTER(pointer)], c.c_int),
        "audiocpp_registry_free": ([pointer], None),
        "audiocpp_model_load": ([pointer, c.c_char_p, c.POINTER(ModelConfig), pointer, c.POINTER(pointer)], c.c_int),
        "audiocpp_model_free": ([pointer], None),
        "audiocpp_session_create": (
            [pointer, c.c_char_p, c.c_char_p, c.POINTER(BackendConfig), pointer, c.POINTER(pointer)],
            c.c_int,
        ),
        "audiocpp_request_create": ([], pointer),
        "audiocpp_request_free": ([pointer], None),
        "audiocpp_request_set_text": ([pointer, c.c_char_p, c.c_char_p], c.c_int),
        "audiocpp_request_set_text_language": ([pointer, c.c_char_p], c.c_int),
        "audiocpp_request_set_audio": ([pointer, c.POINTER(c.c_float), c.c_size_t, c.c_int, c.c_int], c.c_int),
        "audiocpp_session_run": ([pointer, pointer, c.POINTER(pointer)], c.c_int),
        "audiocpp_result_free": ([pointer], None),
        "audiocpp_result_word_count": ([pointer], c.c_size_t),
        "audiocpp_result_word": (
            [
                pointer,
                c.c_size_t,
                c.POINTER(c.c_char_p),
                c.POINTER(c.c_int64),
                c.POINTER(c.c_int64),
                c.POINTER(c.c_float),
            ],
            c.c_int,
        ),
    }
    for name, (arguments, result) in signatures.items():
        function = getattr(library, name)
        function.argtypes, function.restype = arguments, result
    if library.audiocpp_abi_version() != 0x000200:
        raise server_error("runtime_mismatch", "Native alignment ABI differs from the frozen supplier")
    return library


class NativeAlignment:
    def __init__(self, model: Path, device: str, threads: int) -> None:
        self.library = load_api()
        registry, loaded, session = c.c_void_p(), c.c_void_p(), c.c_void_p()
        self._check(self.library.audiocpp_registry_create(None, c.byref(registry)))
        try:
            config = ModelConfig(b"qwen3_forced_aligner", None, None, None)
            self._check(
                self.library.audiocpp_model_load(
                    registry, str(model).encode("utf-8"), c.byref(config), None, c.byref(loaded)
                )
            )
            try:
                backend = {"cpu": b"cpu", "mps": b"metal", "cuda": b"cuda", "vulkan": b"vulkan"}[device]
                configuration = BackendConfig(backend, 0, threads)
                self._check(
                    self.library.audiocpp_session_create(
                        loaded, b"align", b"offline", c.byref(configuration), None, c.byref(session)
                    )
                )
                self.session = session
            finally:
                self.library.audiocpp_model_free(loaded)
        finally:
            self.library.audiocpp_registry_free(registry)

    def _check(self, status: int) -> None:
        if status:
            raise server_error("alignment_failed", self.library.audiocpp_last_error().decode("utf-8"))

    def align(self, samples: NDArray[np.float32], text: str, language: str) -> list[dict[str, Any]]:
        request = self.library.audiocpp_request_create()
        if not request:
            raise server_error("resource_exhausted", "Native alignment request allocation failed")
        result = c.c_void_p()
        try:
            # The aligner uses transcript language, not a generic language option.
            self._check(self.library.audiocpp_request_set_text(request, text.encode("utf-8"), None))
            self._check(self.library.audiocpp_request_set_text_language(request, language.encode("utf-8")))
            self._check(
                self.library.audiocpp_request_set_audio(
                    request, samples.ctypes.data_as(c.POINTER(c.c_float)), len(samples), 16_000, 1
                )
            )
            self._check(self.library.audiocpp_session_run(self.session, request, c.byref(result)))
            return self._words(result, len(samples), len(text))
        finally:
            if result.value:
                self.library.audiocpp_result_free(result)
            self.library.audiocpp_request_free(request)

    def _words(self, result: c.c_void_p, frames: int, characters: int) -> list[dict[str, Any]]:
        count = self.library.audiocpp_result_word_count(result)
        if count > characters:
            raise server_error("invalid_output", "Native alignment exceeds its transcript length")
        words: list[dict[str, Any]] = []
        for index in range(count):
            word, start, end = c.c_char_p(), c.c_int64(), c.c_int64()
            self._check(
                self.library.audiocpp_result_word(result, index, c.byref(word), c.byref(start), c.byref(end), None)
            )
            if not word.value or not 0 <= start.value < end.value <= frames:
                raise server_error("invalid_output", "Native alignment returned timing outside its audio")
            words.append(
                {"text": word.value.decode("utf-8"), "start_time": start.value / 16_000, "end_time": end.value / 16_000}
            )
        return words
