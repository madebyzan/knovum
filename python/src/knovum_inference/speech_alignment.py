from __future__ import annotations

import tempfile
import wave
from pathlib import Path
from typing import Any

from knovum_inference.contracts import SpeechAlignParams
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech_common import check_deadline, milliseconds, verify_audio

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
    def __init__(self, model_directory: Path, work_directory: Path, device: str, cpu_threads: int) -> None:
        self._model_directory = model_directory
        self._work_directory = work_directory
        self._device = device
        self._cpu_threads = cpu_threads
        self._loaded: tuple[Any, Any] | None = None

    def align(self, params: SpeechAlignParams) -> dict[str, Any]:
        audio_path = verify_audio(params.audio_ref)
        check_deadline(params.deadline_ms)
        language = LANGUAGES.get(params.profile.language.lower())
        if language is None:
            raise server_error("unsupported_language", "Forced alignment language is unsupported")
        if params.profile.device != self._device:
            raise server_error("runtime_mismatch", "Alignment session device changed")
        if not params.text_segments:
            raise invalid_params("Alignment requires at least one text segment")
        processor, model = self._load()
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
                check_deadline(params.deadline_ms)
                if (
                    not segment.text.strip()
                    or segment.end_ms <= segment.start_ms
                    or segment.end_ms - segment.start_ms > MAX_SEGMENT_MS
                    or segment.end_ms > duration_ms
                ):
                    raise invalid_params("Alignment segment is empty, outside the audio, or longer than five minutes")
                clip = self._write_clip(source, segment.start_ms, segment.end_ms)
                try:
                    inputs, word_lists = processor.prepare_forced_aligner_inputs(
                        audio=[str(clip)],
                        transcript=[segment.text],
                        language=[language],
                    )
                    inputs = inputs.to(model.device, model.dtype)
                    import torch

                    with torch.inference_mode():
                        output = model(**inputs)
                    words = processor.decode_forced_alignment(
                        logits=output.logits,
                        input_ids=inputs["input_ids"],
                        word_lists=word_lists,
                        timestamp_token_id=model.config.timestamp_token_id,
                    )[0]
                finally:
                    clip.unlink(missing_ok=True)
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
        check_deadline(params.deadline_ms)
        return {
            "requestId": params.request_id,
            "attemptEpoch": params.attempt_epoch,
            "modelSpecId": params.model_spec_id,
            "units": units,
            "coverage": aligned / len(units),
        }

    def _load(self) -> tuple[Any, Any]:
        if self._loaded is None:
            import torch
            from transformers import AutoModelForTokenClassification, AutoProcessor

            torch.set_num_threads(self._cpu_threads)
            processor = AutoProcessor.from_pretrained(self._model_directory, local_files_only=True)
            model = (
                AutoModelForTokenClassification.from_pretrained(
                    self._model_directory,
                    local_files_only=True,
                    dtype=torch.float32,
                )
                .to(self._device)
                .eval()
            )
            self._loaded = processor, model
        return self._loaded

    def _write_clip(self, source: wave.Wave_read, start_ms: int, end_ms: int) -> Path:
        start = start_ms * 16_000 // 1000
        end = min(source.getnframes(), (end_ms * 16_000 + 999) // 1000)
        source.setpos(start)
        with tempfile.NamedTemporaryFile(dir=self._work_directory, suffix=".wav", delete=False) as temporary:
            path = Path(temporary.name)
        try:
            with wave.open(str(path), "wb") as output:
                output.setnchannels(1)
                output.setsampwidth(2)
                output.setframerate(16_000)
                output.writeframes(source.readframes(end - start))
        except BaseException:
            path.unlink(missing_ok=True)
            raise
        return path
