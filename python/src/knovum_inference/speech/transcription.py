from __future__ import annotations

import hashlib
import json
import os
import platform
import wave
from collections.abc import Callable
from pathlib import Path
from threading import Event
from typing import Any

from knovum_inference.contracts.generated.transcription import (
    AsrEngine,
    SpeechTranscribeParams,
    SpeechTranscriptArtifact,
)
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech.request import attempt_directory, check_deadline, milliseconds, verify_audio

Progress = Callable[[str, int, str | None, dict[str, Any] | None], None]
MAX_TRANSCRIPT_BYTES = 64 * 1024 * 1024


class TranscriptionEngine:
    def __init__(self, model_directory: Path, work_directory: Path, cpu_threads: int) -> None:
        self._model_directory = model_directory
        self._work_directory = work_directory
        self._cpu_threads = cpu_threads
        self._engine: tuple[AsrEngine, Any] | None = None
        self._vad: Any = None

    def transcribe(self, params: SpeechTranscribeParams, progress: Progress, cancelled: Event) -> dict[str, Any]:
        audio_path = verify_audio(params.audio_ref)
        check_deadline(params.deadline_ms)
        progress("loading", 0, None, None)
        if params.profile.beam_size < 1:
            raise invalid_params("ASR beam size must be positive")
        if params.profile.language_policy.value == "fixed" and not params.profile.language:
            raise invalid_params("Fixed language policy requires a language")
        if params.profile.engine is AsrEngine.faster_whisper:
            artifact = self._faster(audio_path, params, progress, cancelled)
        elif params.profile.engine is AsrEngine.mlx_whisper:
            artifact = self._mlx(audio_path, params, progress, cancelled)
        else:
            raise server_error("unsupported_device", "ASR engine is not supported")
        check_deadline(params.deadline_ms)
        if cancelled.is_set():
            raise server_error("cancelled", "Transcription was cancelled")
        progress("publishing", 100, artifact["language"], None)
        return self._publish(params, artifact)

    def _publish(self, params: SpeechTranscribeParams, artifact: dict[str, Any]) -> dict[str, Any]:
        validated = SpeechTranscriptArtifact.model_validate(artifact).model_dump(by_alias=True)
        payload = json.dumps(validated, ensure_ascii=False, allow_nan=False, separators=(",", ":")).encode()
        if len(payload) > MAX_TRANSCRIPT_BYTES:
            raise server_error("resource_exhausted", "Transcript artifact exceeds its byte limit")
        directory = attempt_directory(self._work_directory, params.request_id)
        file_name = "transcript.json"
        target = directory / file_name
        temporary = directory / f".{file_name}.part"
        try:
            with temporary.open("xb") as output:
                output.write(payload)
                output.flush()
                os.fsync(output.fileno())
            os.replace(temporary, target)
        except OSError as error:
            temporary.unlink(missing_ok=True)
            raise server_error("resource_exhausted", "Transcript artifact cannot be written") from error
        return {
            "requestId": params.request_id,
            "attemptEpoch": params.attempt_epoch,
            "modelSpecId": params.model_spec_id,
            "transcriptRef": {
                "fileName": f"{directory.name}/{file_name}",
                "mediaType": "application/vnd.knovum.speech-transcript+json",
                "byteLength": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            },
        }

    def _faster(
        self, audio_path: Path, params: SpeechTranscribeParams, progress: Progress, cancelled: Event
    ) -> dict[str, Any]:
        if params.profile.device not in {"cpu", "cuda"}:
            raise server_error("unsupported_device", "faster-whisper device is not supported")
        system, machine = platform.system(), platform.machine().lower()
        if not (
            (system == "Windows" and machine in {"amd64", "x86_64"}) or (system == "Darwin" and machine == "x86_64")
        ):
            raise server_error("unsupported_device", "faster-whisper is not an accepted platform profile")
        if system == "Darwin" and params.profile.device != "cpu":
            raise server_error("unsupported_device", "Intel macOS ASR requires the CPU profile")
        try:
            from faster_whisper import WhisperModel
        except ImportError as error:
            raise server_error("runtime_mismatch", "faster-whisper is unavailable") from error
        if self._engine is None:
            self._engine = (
                AsrEngine.faster_whisper,
                WhisperModel(
                    str(self._model_directory),
                    device=params.profile.device,
                    compute_type=params.profile.compute_type,
                    cpu_threads=self._cpu_threads,
                    local_files_only=True,
                ),
            )
        engine, model = self._engine
        if engine is not AsrEngine.faster_whisper:
            raise server_error("runtime_mismatch", "ASR session engine changed")
        progress("transcribing", 0, None, None)
        segments, info = model.transcribe(
            str(audio_path),
            language=params.profile.language,
            beam_size=params.profile.beam_size,
            vad_filter=params.profile.vad_enabled,
            word_timestamps=True,
        )
        parts: list[dict[str, Any]] = []
        duration_ms = milliseconds(info.duration)
        for item in segments:
            check_deadline(params.deadline_ms)
            if cancelled.is_set():
                raise server_error("cancelled", "Transcription was cancelled")
            part = _faster_segment(item)
            parts.append(part)
            percent = min(99, part["endMs"] * 100 // max(1, duration_ms))
            progress("transcribing", percent, info.language, part)
        return _artifact(info.language, info.duration, info.duration_after_vad, parts)

    def _mlx(
        self, audio_path: Path, params: SpeechTranscribeParams, progress: Progress, cancelled: Event
    ) -> dict[str, Any]:
        if params.profile.device != "mps" or platform.system() != "Darwin" or platform.machine().lower() != "arm64":
            raise server_error("unsupported_device", "mlx-whisper requires Apple Silicon")
        if not params.profile.vad_enabled:
            raise invalid_params("Apple Silicon ASR profile requires Silero VAD")
        try:
            import mlx_whisper
        except ImportError as error:
            raise server_error("runtime_mismatch", "mlx-whisper is unavailable") from error
        speech_intervals, audio_duration_ms = self._speech_intervals(audio_path)
        if not speech_intervals:
            return {
                "language": "und",
                "durationMs": audio_duration_ms,
                "speechDurationMs": 0,
                "segments": [],
            }
        progress("transcribing", 0, None, None)
        transcription = mlx_whisper.transcribe(
            str(audio_path),
            path_or_hf_repo=str(self._model_directory),
            language=params.profile.language,
            word_timestamps=True,
            condition_on_previous_text=True,
        )
        segments = []
        for item in transcription.get("segments", []):
            part = _mlx_segment(item)
            if _overlaps_speech(part, speech_intervals):
                segments.append(part)
        language = transcription.get("language")
        if not isinstance(language, str) or not language:
            raise server_error("invalid_output", "mlx-whisper omitted the language")
        for part in segments:
            check_deadline(params.deadline_ms)
            if cancelled.is_set():
                raise server_error("cancelled", "Transcription was cancelled")
            percent = min(99, part["endMs"] * 100 // max(1, audio_duration_ms))
            progress("transcribing", percent, language, part)
        speech_duration_ms = sum(end - start for start, end in speech_intervals)
        return {
            "language": language,
            "durationMs": audio_duration_ms,
            "speechDurationMs": speech_duration_ms,
            "segments": segments,
        }

    def _speech_intervals(self, audio_path: Path) -> tuple[list[tuple[int, int]], int]:
        import numpy as np
        import torch
        from silero_vad import get_speech_timestamps, load_silero_vad

        with wave.open(str(audio_path), "rb") as source:
            if (
                source.getnchannels() != 1
                or source.getsampwidth() != 2
                or source.getframerate() != 16_000
                or source.getcomptype() != "NONE"
            ):
                raise invalid_params("Silero VAD requires 16 kHz mono PCM16 audio")
            frames = source.getnframes()
            if not 0 < frames <= 2 * 60 * 60 * 16_000:
                raise invalid_params("ASR input must contain audio and be at most two hours")
            pcm = source.readframes(frames)
            if len(pcm) != frames * 2:
                raise invalid_params("ASR input contains truncated PCM frames")
        if self._vad is None:
            self._vad = load_silero_vad(onnx=True)
        samples = torch.from_numpy(np.frombuffer(pcm, dtype="<i2").astype(np.float32) / 32768.0)
        ranges = get_speech_timestamps(samples, self._vad, sampling_rate=16_000)
        intervals = [(item["start"] * 1000 // 16_000, (item["end"] * 1000 + 15_999) // 16_000) for item in ranges]
        return intervals, (frames * 1000 + 15_999) // 16_000


def _faster_segment(segment: Any) -> dict[str, Any]:
    words = [
        {
            "text": word.word,
            "startMs": milliseconds(word.start),
            "endMs": milliseconds(word.end),
            "probability": float(word.probability),
        }
        for word in (segment.words or [])
        if word.start is not None and word.end is not None
    ]
    return {
        "startMs": milliseconds(segment.start),
        "endMs": milliseconds(segment.end),
        "text": segment.text.strip(),
        "words": words,
    }


def _mlx_segment(segment: dict[str, Any]) -> dict[str, Any]:
    words = [
        {
            "text": str(word["word"]),
            "startMs": milliseconds(float(word["start"])),
            "endMs": milliseconds(float(word["end"])),
            "probability": float(word["probability"]) if "probability" in word else None,
        }
        for word in segment.get("words", [])
    ]
    return {
        "startMs": milliseconds(float(segment["start"])),
        "endMs": milliseconds(float(segment["end"])),
        "text": str(segment["text"]).strip(),
        "words": words,
    }


def _artifact(language: str, duration: float, speech_duration: float, segments: list[dict[str, Any]]) -> dict[str, Any]:
    return {
        "language": language,
        "durationMs": milliseconds(duration),
        "speechDurationMs": milliseconds(speech_duration),
        "segments": segments,
    }


def _overlaps_speech(segment: dict[str, Any], intervals: list[tuple[int, int]]) -> bool:
    return any(segment["startMs"] < end and segment["endMs"] > start for start, end in intervals)
