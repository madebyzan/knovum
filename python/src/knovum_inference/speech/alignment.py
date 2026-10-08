from __future__ import annotations

import threading
import wave
from pathlib import Path
from typing import TYPE_CHECKING, Any, cast

from knovum_inference.contracts.generated.alignment import SpeechAlignParams
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech.request import check_deadline, milliseconds, verify_audio

if TYPE_CHECKING:
    from knovum_inference.speech.native_alignment import NativeAlignment

LANGUAGES = {
    "zh": "Chinese",
    "en": "English",
    "yue": "Cantonese",
    "fr": "French",
    "de": "German",
    "it": "Italian",
    "pt": "Portuguese",
    "ru": "Russian",
    "es": "Spanish",
}
MAX_SEGMENT_MS = 5 * 60 * 1000


class AlignmentEngine:
    def __init__(self, model_directory: Path, device: str, cpu_threads: int, model_spec_id: str) -> None:
        self._model_directory = model_directory
        self._device = device
        self._cpu_threads = cpu_threads
        self._loaded: tuple[Any, Any] | None = None
        self._native: NativeAlignment | None = None
        if model_spec_id not in {"qwen3-forced-aligner-0.6b-hf-v1", "qwen3-forced-aligner-0.6b-q8_0-v1"}:
            raise server_error("runtime_mismatch", "Forced alignment model is unsupported")
        self._model_spec_id = model_spec_id
        supported = (
            {"cpu", "cuda", "mps"}
            if model_spec_id == "qwen3-forced-aligner-0.6b-hf-v1"
            else {"cpu", "cuda", "mps", "vulkan"}
        )
        if device not in supported:
            raise server_error("unsupported_device", "Qwen3 forced alignment device is unsupported")

    def align(self, params: SpeechAlignParams, cancelled: threading.Event) -> dict[str, Any]:
        if params.model_spec_id != self._model_spec_id:
            raise server_error("runtime_mismatch", "Alignment request differs from its frozen model")
        if params.profile.device != self._device:
            raise server_error("runtime_mismatch", "Alignment request differs from its frozen execution device")
        audio_path = verify_audio(params.audio_ref)
        self._check(params.deadline_ms, cancelled)
        language = LANGUAGES.get(params.profile.language.lower())
        if language is None:
            raise server_error("unsupported_language", "Forced alignment language is unsupported")
        if not 1 <= len(params.text_segments) <= 8:
            raise invalid_params("Alignment requires one to eight text segments")
        units: list[dict[str, Any]] = []
        with wave.open(str(audio_path), "rb") as source:
            if (
                source.getnchannels() != 1
                or source.getsampwidth() != 2
                or source.getframerate() != 16_000
                or source.getcomptype() != "NONE"
            ):
                raise invalid_params("Alignment requires 16 kHz mono PCM16 audio")
            duration_ms = source.getnframes() * 1000 // 16_000
            for segment in params.text_segments:
                if (
                    not segment.text.strip()
                    or segment.end_ms <= segment.start_ms
                    or segment.end_ms - segment.start_ms > MAX_SEGMENT_MS
                    or segment.end_ms > duration_ms
                ):
                    raise invalid_params("Alignment segment is empty, outside the audio, or longer than five minutes")
            for segment in params.text_segments:
                self._check(params.deadline_ms, cancelled)
                samples = self._read_clip(source, segment.start_ms, segment.end_ms)
                words = self._align_clip(samples, segment.text, language)
                self._check(params.deadline_ms, cancelled)
                unit_kind = "character" if language in {"Chinese", "Cantonese"} else "word"
                for word in words:
                    start = word.get("start_time")
                    end = word.get("end_time")
                    start_ms = segment.start_ms + milliseconds(float(start)) if start is not None else None
                    end_ms = segment.start_ms + milliseconds(float(end)) if end is not None else None
                    if start_ms is not None and end_ms is not None and (end_ms <= start_ms or end_ms > segment.end_ms):
                        raise server_error("invalid_output", "Forced alignment returned timing outside its segment")
                    units.append(
                        {
                            "segmentId": segment.segment_id,
                            "text": str(word["text"]),
                            "unitKind": unit_kind,
                            "startMs": start_ms,
                            "endMs": end_ms,
                            "score": None,
                            "unalignedReason": None if start_ms is not None and end_ms is not None else "no_timing",
                        }
                    )
                if not words:
                    units.append(
                        {
                            "segmentId": segment.segment_id,
                            "text": segment.text,
                            "unitKind": unit_kind,
                            "startMs": None,
                            "endMs": None,
                            "score": None,
                            "unalignedReason": "no_alignment",
                        }
                    )
        aligned = sum(unit["startMs"] is not None for unit in units)
        self._check(params.deadline_ms, cancelled)
        return {
            "requestId": params.request_id,
            "attemptEpoch": params.attempt_epoch,
            "modelSpecId": params.model_spec_id,
            "units": units,
            "coverage": aligned / len(units),
        }

    @staticmethod
    def _check(deadline_ms: int, cancelled: threading.Event) -> None:
        if cancelled.is_set():
            raise server_error("cancelled", "Alignment was cancelled")
        check_deadline(deadline_ms)

    def _align_clip(self, samples: Any, text: str, language: str) -> list[dict[str, Any]]:
        if self._model_spec_id == "qwen3-forced-aligner-0.6b-q8_0-v1":
            if self._native is None:
                from knovum_inference.speech.native_alignment import NativeAlignment

                self._native = NativeAlignment(
                    self._model_directory / "qwen3-forced-aligner-0.6b-q8_0.gguf", self._device, self._cpu_threads
                )
            return self._native.align(samples, text, language)
        processor, model = self._load()
        inputs, word_lists = processor.prepare_forced_aligner_inputs(
            audio=[samples],
            transcript=[text],
            language=[language],
        )
        inputs = inputs.to(model.device, model.dtype)
        import torch

        with torch.inference_mode():
            output = model(**inputs)
        return cast(
            list[dict[str, Any]],
            processor.decode_forced_alignment(
                logits=output.logits,
                input_ids=inputs["input_ids"],
                word_lists=word_lists,
                timestamp_token_id=model.config.timestamp_token_id,
            )[0],
        )

    def _load(self) -> tuple[Any, Any]:
        if self._loaded is None:
            import torch
            from transformers import AutoModelForTokenClassification, AutoProcessor

            processor = AutoProcessor.from_pretrained(self._model_directory, local_files_only=True)
            model, loading = AutoModelForTokenClassification.from_pretrained(
                self._model_directory,
                local_files_only=True,
                dtype=torch.float32,
                output_loading_info=True,
            )
            if any(loading[key] for key in ("missing_keys", "unexpected_keys", "mismatched_keys", "error_msgs")):
                raise server_error("model_corrupt", "Forced alignment weights do not match the native architecture")
            model = model.to(self._device).eval()
            self._loaded = processor, model
        return self._loaded

    @staticmethod
    def _read_clip(source: wave.Wave_read, start_ms: int, end_ms: int) -> Any:
        import numpy as np

        start = start_ms * 16_000 // 1000
        end = min(source.getnframes(), (end_ms * 16_000 + 999) // 1000)
        source.setpos(start)
        pcm = source.readframes(end - start)
        if len(pcm) != (end - start) * 2:
            raise invalid_params("Alignment input has truncated PCM frames")
        return np.frombuffer(pcm, dtype="<i2").astype(np.float32) / 32768.0
